//! Google OpenID Connect identity verification shared by native and browser hosts.
use jsonwebtoken::{
    decode, decode_header,
    jwk::{JwkSet, KeyAlgorithm, KeyOperations, PublicKeyUse},
    Algorithm, DecodingKey, Validation,
};
use reqwest::blocking::Client;
use serde::Deserialize;
use std::{
    io::Read,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

pub const GOOGLE_ISSUER: &str = "https://accounts.google.com";
const GOOGLE_JWKS: &str = "https://www.googleapis.com/oauth2/v3/certs";
const MAX_TOKEN: usize = 16 * 1024;
const MAX_HEADERS: usize = 16 * 1024;
const MAX_JWKS: usize = 128 * 1024;

/// Verified stable identity. Email and name are optional display labels only.
pub struct GoogleIdentity {
    pub issuer: String,
    pub subject: String,
    pub email: Option<String>,
    pub name: Option<String>,
    pub expires_at: u64,
}

/// Values held by the host for one unexpired, single-use authorization attempt.
pub struct VerificationContext<'a> {
    pub client_id: &'a str,
    pub nonce: &'a str,
    pub started_at: u64,
    /// Required when an email address grants initial access before subject pinning.
    pub require_authoritative_email: bool,
}

/// Verify a Google ID token against Google's fixed signing-key endpoint.
///
/// Call from a blocking worker with an HTTPS-only client configured to reject
/// redirects. The caller owns attempt expiry, cancellation and single-use state.
/// Provider error bodies, token contents and request errors never enter diagnostics.
pub fn verify_google_identity(
    http: &Client,
    token: &str,
    context: VerificationContext<'_>,
) -> Result<GoogleIdentity, String> {
    if token.is_empty() || token.len() > MAX_TOKEN {
        return Err("Invalid Google identity token.".into());
    }
    if !valid_label(context.client_id, 1024)
        || !valid_label(context.nonce, 1024)
        || context.started_at == 0
        || context.started_at > seconds().saturating_add(60)
    {
        return Err("Invalid Google identity verification context.".into());
    }
    let header = decode_header(token).map_err(|_| "Invalid Google identity token.")?;
    if header.alg != Algorithm::RS256 {
        return Err("Unsupported Google identity signature algorithm.".into());
    }
    let kid = header
        .kid
        .filter(|value| valid_label(value, 256))
        .ok_or("Google identity has no valid signing-key ID.")?;
    // Never follow jku, jwk, x5u or other token-provided key locations.
    let response = http
        .get(GOOGLE_JWKS)
        .timeout(Duration::from_secs(15))
        .send()
        .map_err(|_| "Unable to retrieve Google verification keys.")?;
    if response.url().as_str() != GOOGLE_JWKS || !response.status().is_success() {
        return Err("Google signing-key lookup failed.".into());
    }
    if response.headers().len() > 64
        || response
            .headers()
            .iter()
            .map(|(key, value)| key.as_str().len() + value.len())
            .sum::<usize>()
            > MAX_HEADERS
        || response
            .content_length()
            .is_some_and(|size| size > MAX_JWKS as u64)
    {
        return Err("Google verification keys exceeded the limit.".into());
    }
    let mut bytes = Vec::new();
    response
        .take(MAX_JWKS as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| "Could not read Google verification keys.")?;
    if bytes.len() > MAX_JWKS {
        return Err("Google verification keys exceeded the limit.".into());
    }
    let keys: JwkSet =
        serde_json::from_slice(&bytes).map_err(|_| "Invalid Google verification keys.")?;
    let mut matching = keys
        .keys
        .iter()
        .filter(|key| key.common.key_id.as_deref() == Some(kid.as_str()));
    let key = matching
        .next()
        .ok_or("Google signing key was not found. Retry sign-in.")?;
    if matching.next().is_some()
        || key
            .common
            .key_algorithm
            .as_ref()
            .is_some_and(|algorithm| *algorithm != KeyAlgorithm::RS256)
        || key
            .common
            .public_key_use
            .as_ref()
            .is_some_and(|usage| *usage != PublicKeyUse::Signature)
        || key
            .common
            .key_operations
            .as_ref()
            .is_some_and(|operations| !operations.contains(&KeyOperations::Verify))
    {
        return Err("Google verification key is not an RS256 signing key.".into());
    }
    let decoding_key =
        DecodingKey::from_jwk(key).map_err(|_| "Unsupported Google verification key.")?;
    let mut rules = Validation::new(Algorithm::RS256);
    rules.set_issuer(&[GOOGLE_ISSUER, "accounts.google.com"]);
    rules.set_audience(&[context.client_id]);
    rules.set_required_spec_claims(&["iss", "sub", "aud", "exp"]);
    rules.validate_nbf = true;
    rules.leeway = 0;
    // Required iat and nonce are enforced by the typed claims deserializer.
    let claims = decode::<IdentityClaims>(token, &decoding_key, &rules)
        .map_err(|_| "Google identity signature, issuer, audience or expiry validation failed.")?
        .claims;
    let now = seconds();
    if claims.nonce != context.nonce
        || claims.iat > now.saturating_add(60)
        || claims.iat < context.started_at.saturating_sub(60)
        || claims.exp <= claims.iat
        || claims.exp <= now
        || !valid_label(&claims.sub, 255)
        || claims
            .azp
            .as_deref()
            .is_some_and(|value| value != context.client_id)
        || (claims.aud.count() > 1 && claims.azp.as_deref() != Some(context.client_id))
    {
        return Err(
            "Google identity nonce, subject, issued-at or authorized-party validation failed."
                .into(),
        );
    }
    let email = claims
        .email
        .filter(|email| claims.email_verified && valid_label(email, 320));
    if context.require_authoritative_email
        && !email
            .as_deref()
            .is_some_and(|email| authoritative_email(email, claims.hd.as_deref()))
    {
        return Err("Google sign-in requires a verified Gmail or Google Workspace email.".into());
    }
    Ok(GoogleIdentity {
        issuer: GOOGLE_ISSUER.into(),
        subject: claims.sub,
        email,
        name: claims.name.filter(|name| valid_label(name, 256)),
        expires_at: claims.exp,
    })
}

#[derive(Deserialize)]
struct IdentityClaims {
    sub: String,
    aud: Audience,
    exp: u64,
    iat: u64,
    nonce: String,
    #[serde(default)]
    azp: Option<String>,
    #[serde(default)]
    email: Option<String>,
    #[serde(default)]
    email_verified: bool,
    #[serde(default)]
    hd: Option<String>,
    #[serde(default)]
    name: Option<String>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum Audience {
    One(String),
    Many(Vec<String>),
}

impl Audience {
    fn count(&self) -> usize {
        match self {
            Self::One(value) => usize::from(!value.is_empty()),
            Self::Many(values) => values.len(),
        }
    }
}

fn valid_label(value: &str, max: usize) -> bool {
    !value.trim().is_empty() && value.len() <= max && !value.chars().any(char::is_control)
}

fn authoritative_email(email: &str, hosted_domain: Option<&str>) -> bool {
    let Some((local, domain)) = email.rsplit_once('@') else {
        return false;
    };
    // A verified external address alone does not establish Google's continuing
    // authority over it. Workspace authority comes from the signed hd claim.
    !local.is_empty()
        && !local.contains('@')
        && !email.chars().any(char::is_whitespace)
        && valid_hosted_domain(domain)
        && (domain.eq_ignore_ascii_case("gmail.com")
            || hosted_domain.is_some_and(valid_hosted_domain))
}

fn valid_hosted_domain(domain: &str) -> bool {
    !domain.is_empty()
        && domain.len() <= 253
        && domain.split('.').all(|label| {
            !label.is_empty()
                && label.len() <= 63
                && !label.starts_with('-')
                && !label.ends_with('-')
                && label
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
        })
}

fn seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

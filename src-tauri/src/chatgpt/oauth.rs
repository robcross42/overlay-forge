use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use jsonwebtoken::{decode, decode_header, jwk::JwkSet, Algorithm, DecodingKey, Validation};
use rand::RngCore;
use reqwest::{blocking::Client, Url};
use serde::{Deserialize, Serialize};
#[cfg(any(windows, test))]
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use zeroize::{Zeroize, ZeroizeOnDrop, Zeroizing};

pub const ISSUER: &str = "https://auth.openai.com";
pub const RESOURCE: &str = "https://api.openai.com/v1";
pub const TOKEN_ENDPOINT: &str = "https://auth.openai.com/api/accounts/oauth/token";
#[cfg(any(windows, test))]
const SCOPES: &str =
    "openid profile email offline_access resource.invoke chatgpt.tokens.use.direct";
pub fn seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
fn random_value() -> Zeroizing<String> {
    let mut bytes = [0u8; 32];
    rand::rng().fill_bytes(&mut bytes);
    let value = URL_SAFE_NO_PAD.encode(bytes);
    bytes.zeroize();
    Zeroizing::new(value)
}
pub struct AuthAttempt {
    consumed: bool,
    state: Zeroizing<String>,
    nonce: Zeroizing<String>,
    verifier: Zeroizing<String>,
    created: Instant,
    pub redirect: String,
    pub client_id: Option<String>,
    pub expected_subject: Option<String>,
}
impl AuthAttempt {
    pub fn new(port: u16, client_id: Option<String>, expected_subject: Option<String>) -> Self {
        Self {
            consumed: false,
            state: random_value(),
            nonce: random_value(),
            verifier: random_value(),
            created: Instant::now(),
            redirect: format!("http://127.0.0.1:{port}/auth/callback"),
            client_id,
            expected_subject,
        }
    }
    #[cfg(windows)]
    pub fn open_browser(&self, host: &str) -> Result<(), String> {
        crate::browser_auth::open_system_browser(&self.authorization_url(host))
    }
    #[cfg(not(windows))]
    pub fn open_browser(&self, _host: &str) -> Result<(), String> {
        Err("ChatGPT browser sign-in is currently available on Windows.".into())
    }

    #[cfg(any(windows, test))]
    pub fn authorization_url(&self, host: &str) -> String {
        let mut url =
            Url::parse("https://auth.openai.com/api/accounts/authorize").expect("fixed URL");
        let challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(self.verifier.as_bytes()));
        let mut query = url.query_pairs_mut();
        query.extend_pairs([
            (
                "client_id",
                self.client_id.as_deref().unwrap_or("dynamic_agent_client"),
            ),
            ("ext_agent_host_id", host),
            ("response_type", "code"),
            ("redirect_uri", &self.redirect),
            ("scope", SCOPES),
            ("resource", RESOURCE),
            ("state", &self.state),
            ("nonce", &self.nonce),
            ("code_challenge_method", "S256"),
            ("code_challenge", &challenge),
        ]);
        if self.client_id.is_none() {
            query.append_pair("agent_name_hint", "Overlay Forge");
        }
        drop(query);
        url.into()
    }
    pub fn callback(&mut self, target: &str) -> Result<(Zeroizing<String>, String), String> {
        if self.consumed {
            return Err("Sign-in callback was already consumed. Start a fresh attempt.".into());
        }
        self.consumed = true;
        if self.created.elapsed() > Duration::from_secs(300) {
            return Err("Sign-in expired. Start a fresh attempt.".into());
        }
        let url = Url::parse(&format!("http://127.0.0.1{target}"))
            .map_err(|_| "Invalid sign-in callback.")?;
        if url.path() != "/auth/callback" {
            return Err("Invalid sign-in callback path.".into());
        }
        let mut fields = HashMap::new();
        for (key, value) in url.query_pairs() {
            if fields
                .insert(key.into_owned(), value.into_owned())
                .is_some()
            {
                return Err("Duplicate callback parameter rejected.".into());
            }
        }
        if fields.get("state").map(String::as_str) != Some(self.state.as_str()) {
            return Err("Sign-in state mismatch. No code was exchanged.".into());
        }
        if fields.contains_key("error") {
            return Err(
                "ChatGPT authorization was declined or failed. No account was connected.".into(),
            );
        }
        let client = match (&self.client_id, fields.get("client_id")) {
            (Some(expected), Some(actual)) if expected != actual => {
                return Err(
                    "Returning sign-in changed its registered client. Callback rejected.".into(),
                )
            }
            (Some(expected), _) => expected.clone(),
            (None, Some(issued)) if issued.starts_with("oaiapp_") => issued.clone(),
            _ => return Err("Registration did not return an issued client ID.".into()),
        };
        let code = fields
            .remove("code")
            .filter(|v| !v.is_empty())
            .ok_or("Sign-in callback had no code.")?;
        Ok((Zeroizing::new(code), client))
    }
    pub fn exchange(&self, http: &Client, code: &str, client: &str) -> Result<Credentials, String> {
        let response = http
            .post(TOKEN_ENDPOINT)
            .form(&[
                ("grant_type", "authorization_code"),
                ("client_id", client),
                ("code", code),
                ("code_verifier", &self.verifier),
                ("redirect_uri", &self.redirect),
                ("resource", RESOURCE),
            ])
            .send()
            .map_err(|_| "Token exchange failed to connect. Start a fresh sign-in.")?;
        let mut tokens = read_tokens(response)?;
        let identity = validate_identity(http, &tokens.id_token, client, &self.nonce)?;
        if self
            .expected_subject
            .as_ref()
            .is_some_and(|subject| subject != &identity.sub)
        {
            return Err("The signed-in account differs from the selected registration. Existing credentials were preserved.".into());
        }
        Ok(Credentials {
            client_id: client.into(),
            subject: identity.sub,
            email: identity.email,
            access_token: std::mem::take(&mut tokens.access_token),
            refresh_token: std::mem::take(&mut tokens.refresh_token),
            id_token: std::mem::take(&mut tokens.id_token),
            scope: std::mem::take(&mut tokens.scope),
            expires_at: seconds()
                .checked_add(tokens.expires_in)
                .ok_or("ChatGPT expiry was invalid.")?,
        })
    }
}
#[derive(Deserialize, Zeroize, ZeroizeOnDrop)]
pub struct TokenResponse {
    #[serde(default)]
    pub access_token: String,
    #[serde(default)]
    pub refresh_token: String,
    #[serde(default)]
    pub id_token: String,
    #[serde(default)]
    pub scope: String,
    #[serde(default)]
    pub expires_in: u64,
    #[serde(default)]
    pub token_type: String,
}
pub fn read_tokens(response: reqwest::blocking::Response) -> Result<TokenResponse, String> {
    let status = response.status();
    if !status.is_success() {
        return Err(super::transport::http_error(response));
    }
    let token: TokenResponse = response
        .json()
        .map_err(|_| "ChatGPT token response was invalid.")?;
    validate_token_response(token)
}
fn validate_token_response(token: TokenResponse) -> Result<TokenResponse, String> {
    if token.access_token.is_empty() {
        if token.id_token.is_empty()
            || token
                .scope
                .split_whitespace()
                .any(|scope| scope == "chatgpt.tokens.use.direct")
        {
            return Err("Incomplete ChatGPT identity/plan token response.".into());
        }
    } else if !token.token_type.eq_ignore_ascii_case("bearer") || token.expires_in == 0 {
        return Err("Invalid ChatGPT token type or expiry.".into());
    }
    Ok(token)
}
#[derive(Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
pub struct Credentials {
    pub client_id: String,
    pub subject: String,
    pub email: Option<String>,
    pub access_token: String,
    pub refresh_token: String,
    pub id_token: String,
    pub scope: String,
    pub expires_at: u64,
}
impl Credentials {
    pub fn plan_allowed(&self) -> bool {
        self.scope
            .split_whitespace()
            .any(|scope| scope == "chatgpt.tokens.use.direct")
            && self
                .scope
                .split_whitespace()
                .any(|scope| scope == "resource.invoke")
    }
}
#[derive(Clone, Deserialize)]
struct Identity {
    sub: String,
    #[serde(default)]
    email: Option<String>,
    nonce: String,
    iat: u64,
}
fn validate_identity(
    http: &Client,
    token: &str,
    client: &str,
    nonce: &str,
) -> Result<Identity, String> {
    let header = decode_header(token).map_err(|_| "Invalid ChatGPT ID token.")?;
    if header.alg != Algorithm::RS256 {
        return Err("Unsupported ChatGPT ID-token signature algorithm.".into());
    }
    let kid = header
        .kid
        .ok_or("ChatGPT ID token has no signing-key ID.")?;
    let keys: JwkSet = http
        .get("https://auth.openai.com/.well-known/jwks.json")
        .send()
        .map_err(|_| "Unable to retrieve ChatGPT verification keys.")?
        .error_for_status()
        .map_err(|_| "ChatGPT verification keys are unavailable.")?
        .json()
        .map_err(|_| "Invalid ChatGPT verification keys.")?;
    let jwk = keys
        .find(&kid)
        .ok_or("ChatGPT signing key was not found. Retry sign-in.")?;
    let key = DecodingKey::from_jwk(jwk).map_err(|_| "Unsupported ChatGPT signing key.")?;
    let mut rules = Validation::new(Algorithm::RS256);
    rules.set_issuer(&[ISSUER]);
    rules.set_audience(&[client]);
    rules.leeway = 5;
    rules.set_required_spec_claims(&["sub", "exp", "iat", "iss", "aud", "nonce"]);
    let identity = decode::<Identity>(token, &key, &rules)
        .map_err(|_| "ChatGPT identity signature, issuer, audience or expiry validation failed.")?
        .claims;
    if identity.nonce != nonce || identity.sub.is_empty() || identity.iat > seconds() + 5 {
        return Err("ChatGPT ID-token nonce or issued-at validation failed.".into());
    }
    Ok(identity)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn identity_only_response_does_not_require_or_grant_inference() {
        let identity: TokenResponse = serde_json::from_str(
            r#"{"id_token":"synthetic-id-token","scope":"openid profile email"}"#,
        )
        .unwrap();
        assert!(validate_token_response(identity).is_ok());
        let invalid: TokenResponse = serde_json::from_str(
            r#"{"id_token":"synthetic-id-token","scope":"chatgpt.tokens.use.direct"}"#,
        )
        .unwrap();
        assert!(validate_token_response(invalid).is_err());
    }
    #[test]
    fn pkce_is_fresh_and_returning_registration_is_bound() {
        let mut a = AuthAttempt::new(45678, Some("oaiapp_test".into()), None);
        let b = AuthAttempt::new(45678, None, None);
        assert_ne!(a.state, b.state);
        assert_ne!(a.nonce, b.nonce);
        assert_ne!(a.verifier, b.verifier);
        let url = Url::parse(&a.authorization_url("urn:uuid:test")).unwrap();
        let fields: HashMap<_, _> = url.query_pairs().collect();
        assert_eq!(fields["code_challenge_method"], "S256");
        assert!(!fields.contains_key("agent_name_hint"));
        assert!(a.callback("/auth/callback?state=wrong&code=test").is_err());
        let mut a = AuthAttempt::new(45678, Some("oaiapp_test".into()), None);
        assert!(a
            .callback(&format!(
                "/auth/callback?state={}&code=test&client_id=oaiapp_other",
                a.state.as_str()
            ))
            .is_err());
    }
    #[test]
    fn denied_missing_client_duplicate_and_expired_callbacks_are_rejected() {
        for tail in [
            "error=access_denied",
            "code=test",
            "code=test&client_id=dynamic_agent_client",
            "code=test&client_id=oaiapp_test&code=duplicate",
        ] {
            let mut a = AuthAttempt::new(1234, None, None);
            assert!(a
                .callback(&format!("/auth/callback?state={}&{tail}", a.state.as_str()))
                .is_err());
        }
        let mut a = AuthAttempt::new(1234, None, None);
        assert!(a
            .callback(&format!(
                "/auth/callback?state={}&code=test&client_id=oaiapp_test",
                a.state.as_str()
            ))
            .is_ok());
        assert!(a.callback("/auth/callback").is_err());
        let mut a = AuthAttempt::new(1234, None, None);
        a.created = Instant::now() - Duration::from_secs(301);
        assert!(a.callback("/auth/callback").is_err());
    }
}

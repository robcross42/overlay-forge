//! Native Google OpenID Connect. Only the verified identity leaves this module.
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use jsonwebtoken::{
    decode, decode_header,
    jwk::{JwkSet, KeyAlgorithm, PublicKeyUse},
    Algorithm, DecodingKey, Validation,
};
use rand::RngCore;
use reqwest::{blocking::Client, redirect::Policy, Url};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    sync::atomic::{AtomicBool, Ordering},
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use zeroize::{Zeroize, ZeroizeOnDrop, Zeroizing};

pub const CLIENT_ID: &str =
    "840684879038-u3ge7ov3vede0slbgntdam99mtg0u86r.apps.googleusercontent.com";
const ISSUER: &str = "https://accounts.google.com";
const CALLBACK_PATH: &str = "/";
const MAX_HEADERS: usize = 16 * 1024;
const MAX_BODY: usize = 64 * 1024;

pub struct GoogleIdentity {
    pub issuer: String,
    pub subject: String,
    pub email: Option<String>,
    pub name: Option<String>,
    pub expires_at: u64,
}

struct SignInAttempt<'a> {
    cancel: &'a AtomicBool,
    deadline: Instant,
    started_at: u64,
    state: Zeroizing<String>,
    nonce: Zeroizing<String>,
    verifier: Zeroizing<String>,
    redirect: String,
}

pub fn sign_in(cancel: &AtomicBool, client_secret: &str) -> Result<GoogleIdentity, String> {
    let listener = TcpListener::bind(("127.0.0.1", 0))
        .map_err(|_| "Could not bind the local Google sign-in callback.")?;
    listener
        .set_nonblocking(true)
        .map_err(|_| "Could not configure Google sign-in callback.")?;
    let address = listener
        .local_addr()
        .map_err(|_| "Google callback address is unavailable.")?;
    let attempt = SignInAttempt {
        cancel,
        deadline: Instant::now() + Duration::from_secs(300),
        started_at: seconds(),
        state: random_value(),
        nonce: random_value(),
        verifier: random_value(),
        redirect: format!("http://{address}{CALLBACK_PATH}"),
    };
    attempt.check_active()?;
    crate::browser_auth::open_system_browser(&attempt.authorization_url()?)?;
    let code = attempt.wait_for_code(&listener, &address.to_string())?;
    // Close the listener before exchanging the one accepted authorization code.
    drop(listener);
    attempt.check_active()?;
    let identity = attempt.exchange(&code, client_secret)?;
    // The profile owner must also check cancellation while committing this identity.
    attempt.check_active()?;
    Ok(identity)
}

impl SignInAttempt<'_> {
    fn check_active(&self) -> Result<(), String> {
        if self.cancel.load(Ordering::Acquire) {
            Err("Google sign-in was cancelled.".into())
        } else if Instant::now() >= self.deadline {
            Err("Google sign-in expired. Start a fresh attempt.".into())
        } else {
            Ok(())
        }
    }

    fn request_timeout(&self) -> Result<Duration, String> {
        self.check_active()?;
        Ok(self
            .deadline
            .saturating_duration_since(Instant::now())
            .min(Duration::from_secs(15)))
    }

    fn authorization_url(&self) -> Result<String, String> {
        let mut url = Url::parse("https://accounts.google.com/o/oauth2/v2/auth")
            .map_err(|_| "Invalid Google authorization endpoint.")?;
        let challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(self.verifier.as_bytes()));
        url.query_pairs_mut().extend_pairs([
            ("client_id", CLIENT_ID),
            ("response_type", "code"),
            ("redirect_uri", self.redirect.as_str()),
            ("scope", "openid email profile"),
            ("state", self.state.as_str()),
            ("nonce", self.nonce.as_str()),
            ("code_challenge", challenge.as_str()),
            ("code_challenge_method", "S256"),
            ("access_type", "online"),
            ("prompt", "select_account"),
        ]);
        Ok(url.into())
    }

    fn wait_for_code(
        &self,
        listener: &TcpListener,
        host: &str,
    ) -> Result<Zeroizing<String>, String> {
        loop {
            self.check_active()?;
            match listener.accept() {
                Ok((mut stream, peer)) if peer.ip().is_loopback() => {
                    let target = match self.read_callback(&mut stream, host) {
                        Ok(target) => target,
                        Err(_) => {
                            respond(&mut stream, false);
                            continue;
                        }
                    };
                    // Ignore unrelated or malformed traffic without consuming the attempt.
                    let code = match self.callback_code(&target) {
                        Ok(Some(code)) => code,
                        Ok(None) => {
                            respond(&mut stream, false);
                            continue;
                        }
                        Err(error) => {
                            respond(&mut stream, false);
                            return Err(error);
                        }
                    };
                    self.check_active()?;
                    respond(&mut stream, true);
                    return Ok(code);
                }
                Ok(_) => {}
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(50));
                }
                Err(_) => return Err("Google sign-in callback failed.".into()),
            }
        }
    }

    fn read_callback(
        &self,
        stream: &mut TcpStream,
        host: &str,
    ) -> Result<Zeroizing<String>, String> {
        stream
            .set_read_timeout(Some(Duration::from_millis(100)))
            .map_err(|_| "Unable to read Google callback.")?;
        let deadline = Instant::now() + Duration::from_secs(2);
        let mut bytes = Zeroizing::new(Vec::new());
        let mut chunk = Zeroizing::new([0u8; 1024]);
        let end = loop {
            self.check_active()?;
            if Instant::now() >= deadline || bytes.len() >= MAX_HEADERS {
                return Err("Google callback headers exceeded the limit.".into());
            }
            match stream.read(&mut *chunk) {
                Ok(0) => return Err("Incomplete Google callback.".into()),
                Ok(count) => {
                    bytes.extend_from_slice(&chunk[..count]);
                    if bytes.len() > MAX_HEADERS {
                        return Err("Google callback headers exceeded the limit.".into());
                    }
                    if let Some(end) = bytes.windows(4).position(|value| value == b"\r\n\r\n") {
                        break end;
                    }
                }
                Err(error)
                    if matches!(
                        error.kind(),
                        std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                    ) => {}
                Err(_) => return Err("Unable to read Google callback.".into()),
            }
        };
        if bytes.len() != end + 4 {
            return Err("Google callback must not have a body.".into());
        }
        let text =
            std::str::from_utf8(&bytes[..end]).map_err(|_| "Invalid Google callback encoding.")?;
        let mut lines = text.split("\r\n");
        let parts: Vec<_> = lines.next().unwrap_or_default().split(' ').collect();
        if parts.len() != 3 || parts[0] != "GET" || !matches!(parts[2], "HTTP/1.1" | "HTTP/1.0") {
            return Err("Invalid Google callback method.".into());
        }
        let mut headers = HashMap::new();
        for line in lines {
            let (key, value) = line
                .split_once(':')
                .ok_or("Invalid Google callback header.")?;
            if key.is_empty()
                || !key
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
                || value
                    .chars()
                    .any(|value| value.is_control() && value != '\t')
                || headers.len() >= 64
                || headers
                    .insert(key.to_ascii_lowercase(), value.trim())
                    .is_some()
            {
                return Err("Invalid or duplicate Google callback header.".into());
            }
        }
        if headers.get("host").copied() != Some(host)
            || headers.contains_key("transfer-encoding")
            || headers
                .get("content-length")
                .is_some_and(|value| *value != "0")
        {
            return Err("Invalid Google callback host or body.".into());
        }
        Ok(Zeroizing::new(parts[1].into()))
    }

    fn callback_code(&self, target: &str) -> Result<Option<Zeroizing<String>>, String> {
        self.check_active()?;
        let Some((path, _)) = target.split_once('?') else {
            return Ok(None);
        };
        if path != CALLBACK_PATH
            || target.contains('#')
            || !target.is_ascii()
            || target.bytes().any(|value| value.is_ascii_control())
        {
            return Ok(None);
        }
        let url = Url::parse(&format!("http://127.0.0.1{target}"))
            .map_err(|_| "Invalid Google callback URL.")?;
        let mut fields: HashMap<String, Zeroizing<String>> = HashMap::new();
        for (key, value) in url.query_pairs() {
            if fields
                .insert(key.into_owned(), Zeroizing::new(value.into_owned()))
                .is_some()
            {
                return Ok(None);
            }
        }
        if fields.get("state").map(|value| value.as_str()) != Some(self.state.as_str()) {
            return Ok(None);
        }
        if fields.contains_key("error") {
            return Err(
                "Google authorization was declined or failed. No profile was connected.".into(),
            );
        }
        if fields
            .get("iss")
            .is_some_and(|value| value.as_str() != ISSUER)
        {
            return Err("Google callback issuer was invalid.".into());
        }
        let code = fields
            .remove("code")
            .filter(|value| !value.is_empty())
            .ok_or("Google callback contained no authorization code.")?;
        Ok(Some(code))
    }

    fn exchange(&self, code: &str, client_secret: &str) -> Result<GoogleIdentity, String> {
        let http = Client::builder()
            .https_only(true)
            .redirect(Policy::none())
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(15))
            .build()
            .map_err(|_| "Could not create Google sign-in transport.")?;
        let response = http
            .post("https://oauth2.googleapis.com/token")
            .timeout(self.request_timeout()?)
            // The configured Desktop client requires its imported application secret.
            .form(&[
                ("client_id", CLIENT_ID),
                ("client_secret", client_secret),
                ("grant_type", "authorization_code"),
                ("code", code),
                ("code_verifier", self.verifier.as_str()),
                ("redirect_uri", self.redirect.as_str()),
            ])
            .send()
            .map_err(|_| "Google token exchange failed to connect. Start a fresh sign-in.")?;
        self.check_active()?;
        let bytes = read_response(response, "token exchange")?;
        let tokens: TokenResponse = serde_json::from_slice(&bytes)
            .map_err(|_| "Google returned an invalid token response.")?;
        if tokens.id_token.is_empty() {
            return Err("Google did not return an identity token.".into());
        }
        self.validate_identity(&http, &tokens.id_token)
        // All token strings and the raw response are zeroized on every exit path.
    }

    fn validate_identity(&self, http: &Client, token: &str) -> Result<GoogleIdentity, String> {
        self.check_active()?;
        let header = decode_header(token).map_err(|_| "Invalid Google identity token.")?;
        if header.alg != Algorithm::RS256 {
            return Err("Unsupported Google identity signature algorithm.".into());
        }
        let kid = header
            .kid
            .filter(|value| !value.is_empty())
            .ok_or("Google identity has no signing-key ID.")?;
        let response = http
            .get("https://www.googleapis.com/oauth2/v3/certs")
            .timeout(self.request_timeout()?)
            .send()
            .map_err(|_| "Unable to retrieve Google verification keys.")?;
        self.check_active()?;
        let keys: JwkSet = serde_json::from_slice(&read_response(response, "signing-key lookup")?)
            .map_err(|_| "Invalid Google verification keys.")?;
        let key = keys
            .find(&kid)
            .ok_or("Google signing key was not found. Retry sign-in.")?;
        if key
            .common
            .key_algorithm
            .as_ref()
            .is_some_and(|algorithm| *algorithm != KeyAlgorithm::RS256)
            || key
                .common
                .public_key_use
                .as_ref()
                .is_some_and(|usage| *usage != PublicKeyUse::Signature)
        {
            return Err("Google verification key is not an RS256 signing key.".into());
        }
        let decoding_key =
            DecodingKey::from_jwk(key).map_err(|_| "Unsupported Google verification key.")?;
        let mut rules = Validation::new(Algorithm::RS256);
        rules.set_issuer(&[ISSUER, "accounts.google.com"]);
        rules.set_audience(&[CLIENT_ID]);
        rules.set_required_spec_claims(&["iss", "sub", "aud", "exp", "iat", "nonce"]);
        rules.leeway = 0;
        let claims = decode::<IdentityClaims>(token, &decoding_key, &rules)
            .map_err(|_| {
                "Google identity signature, issuer, audience or expiry validation failed."
            })?
            .claims;
        let now = seconds();
        if claims.nonce != self.nonce.as_str()
            || claims.iat > now.saturating_add(60)
            || claims.iat < self.started_at.saturating_sub(60)
            || claims.exp <= claims.iat
            || claims.exp <= now
            || !valid_label(&claims.sub, 255)
            || claims
                .azp
                .as_deref()
                .is_some_and(|value| value != CLIENT_ID)
            || (claims.aud.count() > 1 && claims.azp.as_deref() != Some(CLIENT_ID))
        {
            return Err(
                "Google identity nonce, subject, issued-at or authorized-party validation failed."
                    .into(),
            );
        }
        self.check_active()?;
        Ok(GoogleIdentity {
            issuer: ISSUER.into(),
            subject: claims.sub,
            // Email is a display label, never the stable profile key.
            email: claims
                .email
                .filter(|email| claims.email_verified && valid_label(email, 320)),
            name: claims.name.filter(|name| valid_label(name, 256)),
            expires_at: claims.exp,
        })
    }
}

#[derive(Deserialize, Zeroize, ZeroizeOnDrop)]
struct TokenResponse {
    #[serde(default)]
    id_token: String,
    #[serde(default)]
    access_token: String,
    #[serde(default)]
    refresh_token: String,
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

fn read_response(
    response: reqwest::blocking::Response,
    operation: &str,
) -> Result<Zeroizing<Vec<u8>>, String> {
    let status = response.status();
    if response.headers().len() > 64
        || response
            .headers()
            .iter()
            .map(|(key, value)| key.as_str().len() + value.len())
            .sum::<usize>()
            > MAX_HEADERS
        || response
            .content_length()
            .is_some_and(|size| size > MAX_BODY as u64)
    {
        return Err("Google sign-in response exceeded the limit.".into());
    }
    let mut bytes = Zeroizing::new(Vec::new());
    response
        .take(MAX_BODY as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| "Could not read Google sign-in response.")?;
    if bytes.len() > MAX_BODY {
        return Err("Google sign-in response exceeded the limit.".into());
    }
    if !status.is_success() {
        // Emit only fixed diagnostics. Provider bodies can contain sensitive request data.
        let detail = serde_json::from_slice::<serde_json::Value>(&bytes).ok();
        let code = detail
            .as_ref()
            .and_then(|value| value.get("error"))
            .and_then(|value| value.as_str());
        let description = detail
            .as_ref()
            .and_then(|value| value.get("error_description"))
            .and_then(|value| value.as_str())
            .unwrap_or_default();
        let reason = if description.contains("client_secret") && description.contains("missing") {
            "This Desktop client requires its client secret in the native token exchange. Import its Google client configuration locally; do not paste the secret into chat."
        } else {
            match code {
                Some("invalid_client") => "Google rejected this client configuration.",
                Some("invalid_grant") => "Google rejected the authorization code or PKCE grant. Start a fresh sign-in.",
                Some("invalid_request") => "Google rejected the token request parameters.",
                Some("unauthorized_client") => "This OAuth client is not authorized for the Desktop flow.",
                Some("access_denied") => "Google denied authorization. Check consent and account eligibility.",
                _ => "Google rejected the request. Check the Desktop client and consent-screen setup.",
            }
        };
        return Err(format!(
            "Google {operation} failed (HTTP {}). {reason}",
            status.as_u16()
        ));
    }
    Ok(bytes)
}

fn respond(stream: &mut TcpStream, accepted: bool) {
    let _ = stream.set_write_timeout(Some(Duration::from_millis(100)));
    let (status, message) = if accepted {
        ("200 OK", "Google response received. Return to Overlay Forge to finish sign-in. You can close this tab.")
    } else {
        (
            "400 Bad Request",
            "This sign-in callback was not accepted. Return to Overlay Forge.",
        )
    };
    let response = format!("HTTP/1.1 {status}\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: {}\r\nCache-Control: no-store\r\nContent-Security-Policy: default-src 'none'; frame-ancestors 'none'\r\nX-Content-Type-Options: nosniff\r\nConnection: close\r\n\r\n{message}", message.len());
    let _ = stream.write_all(response.as_bytes());
}

fn valid_label(value: &str, max: usize) -> bool {
    !value.trim().is_empty() && value.len() <= max && !value.chars().any(char::is_control)
}

fn random_value() -> Zeroizing<String> {
    let mut bytes = [0u8; 32];
    rand::rng().fill_bytes(&mut bytes);
    let value = URL_SAFE_NO_PAD.encode(bytes);
    bytes.zeroize();
    Zeroizing::new(value)
}

fn seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

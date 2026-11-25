use crate::{
    error::ApiError,
    security::{cookie, cookie_value, random_token, seconds, token_hash},
    Shared,
};
use axum::{
    extract::{Query, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Redirect, Response},
    Json,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::HashMap, io::Read, sync::Mutex};
use subtle::ConstantTimeEq;
use zeroize::{Zeroize, ZeroizeOnDrop, Zeroizing};

#[derive(Zeroize, ZeroizeOnDrop)]
pub struct Attempt {
    verifier: String,
    nonce: String,
    browser_hash: String,
    started_at: u64,
}
pub struct Attempts(Mutex<HashMap<String, Attempt>>);
impl Attempts {
    pub fn new() -> Self {
        Self(Mutex::new(HashMap::new()))
    }
}
#[derive(Serialize)]
pub struct LoginStart {
    url: String,
}

pub async fn start(State(state): State<Shared>, headers: HeaderMap) -> Result<Response, ApiError> {
    crate::routes::check_origin(&state, &headers)?;
    let verifier = random_token();
    let nonce = random_token();
    let key = random_token();
    let browser = random_token();
    let challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));
    let now = seconds();
    {
        let mut attempts = state
            .attempts
            .0
            .lock()
            .map_err(|_| ApiError::unavailable())?;
        attempts.retain(|_, attempt| now.saturating_sub(attempt.started_at) < 300);
        if attempts.len() >= 256 {
            return Err(ApiError(
                StatusCode::TOO_MANY_REQUESTS,
                "Too many sign-in attempts. Try again later.",
            ));
        }
        attempts.insert(
            token_hash(&key),
            Attempt {
                verifier,
                nonce: nonce.clone(),
                browser_hash: token_hash(&browser),
                started_at: now,
            },
        );
    }
    let redirect = format!("{}/auth/callback", state.config.public_origin);
    let mut url = reqwest::Url::parse("https://accounts.google.com/o/oauth2/v2/auth")
        .map_err(|_| ApiError::unavailable())?;
    url.query_pairs_mut().extend_pairs([
        ("client_id", state.google.client_id.as_str()),
        ("redirect_uri", redirect.as_str()),
        ("response_type", "code"),
        ("scope", "openid email profile"),
        ("state", key.as_str()),
        ("nonce", nonce.as_str()),
        ("code_challenge", challenge.as_str()),
        ("code_challenge_method", "S256"),
        ("prompt", "select_account"),
    ]);
    let mut response = Json(LoginStart {
        url: url.to_string(),
    })
    .into_response();
    set_cookie(
        &mut response,
        cookie(
            state.config.login_cookie(),
            &browser,
            state.config.secure(),
            300,
        ),
    )?;
    Ok(response)
}

#[derive(Deserialize)]
pub struct Callback {
    state: String,
    code: Option<String>,
    error: Option<String>,
    iss: Option<String>,
}
#[derive(Deserialize, Zeroize, ZeroizeOnDrop)]
struct Tokens {
    id_token: String,
    #[serde(default)]
    access_token: String,
    #[serde(default)]
    refresh_token: String,
}
fn read_bounded(response: reqwest::blocking::Response) -> Result<Zeroizing<Vec<u8>>, ()> {
    if !response.status().is_success()
        || response
            .content_length()
            .is_some_and(|size| size > 128 * 1024)
    {
        return Err(());
    }
    let mut bytes = Zeroizing::new(Vec::new());
    response
        .take(128 * 1024 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| ())?;
    if bytes.len() > 128 * 1024 {
        return Err(());
    }
    Ok(bytes)
}

pub async fn callback(
    State(state): State<Shared>,
    headers: HeaderMap,
    Query(query): Query<Callback>,
) -> Response {
    let outcome = complete(state.clone(), &headers, query).await;
    let (location, session_cookie) = match outcome {
        Ok(cookie) => ("/", Some(cookie)),
        Err(()) => ("/?auth=denied", None),
    };
    let mut response = Redirect::to(location).into_response();
    let _ = set_cookie(
        &mut response,
        cookie(state.config.login_cookie(), "", state.config.secure(), 0),
    );
    if let Some(cookie) = session_cookie {
        let _ = set_cookie(&mut response, cookie);
    }
    response
}
async fn complete(state: Shared, headers: &HeaderMap, query: Callback) -> Result<String, ()> {
    if query.state.len() != 43
        || query.code.as_ref().is_some_and(|code| code.len() > 4096)
        || query
            .iss
            .as_deref()
            .is_some_and(|issuer| issuer != "https://accounts.google.com")
    {
        return Err(());
    }
    let browser = cookie_value(headers, state.config.login_cookie()).ok_or(())?;
    let attempt = {
        let mut attempts = state.attempts.0.lock().map_err(|_| ())?;
        let key = token_hash(&query.state);
        let candidate = attempts.get(&key).ok_or(())?;
        if seconds().saturating_sub(candidate.started_at) >= 300
            || !bool::from(
                candidate
                    .browser_hash
                    .as_bytes()
                    .ct_eq(token_hash(&browser).as_bytes()),
            )
        {
            return Err(());
        }
        attempts.remove(&key).ok_or(())?
    };
    if query.error.is_some() {
        return Err(());
    }
    let code = Zeroizing::new(query.code.ok_or(())?);
    let permit = state.workers.clone().try_acquire_owned().map_err(|_| ())?;
    let worker_state = state.clone();
    let identity = tokio::task::spawn_blocking(move || {
        let _permit = permit;
        let redirect = format!("{}/auth/callback", worker_state.config.public_origin);
        let response = worker_state
            .http
            .post("https://oauth2.googleapis.com/token")
            .form(&[
                ("client_id", worker_state.google.client_id.as_str()),
                ("client_secret", worker_state.google.client_secret.as_str()),
                ("grant_type", "authorization_code"),
                ("code", code.as_str()),
                ("code_verifier", attempt.verifier.as_str()),
                ("redirect_uri", redirect.as_str()),
            ])
            .send()
            .map_err(|_| ())?;
        let tokens: Tokens = serde_json::from_slice(&read_bounded(response)?).map_err(|_| ())?;
        if seconds().saturating_sub(attempt.started_at) >= 300 {
            return Err(());
        }
        let identity = overlay_forge_google_identity::verify_google_identity(
            &worker_state.http,
            &tokens.id_token,
            overlay_forge_google_identity::VerificationContext {
                client_id: &worker_state.google.client_id,
                nonce: &attempt.nonce,
                started_at: attempt.started_at,
                require_authoritative_email: true,
            },
        )
        .map_err(|_| ())?;
        if seconds().saturating_sub(attempt.started_at) >= 300 {
            return Err(());
        }
        Ok(identity)
    })
    .await
    .map_err(|_| ())??;
    let token = random_token();
    let csrf = random_token();
    if !state
        .repository
        .create_session(&identity, &token_hash(&token), &csrf, seconds())
        .map_err(|_| ())?
    {
        return Err(());
    }
    if let Some(old) = cookie_value(headers, state.config.session_cookie()) {
        let _ = state.repository.logout(&token_hash(&old));
    }
    Ok(cookie(
        state.config.session_cookie(),
        &token,
        state.config.secure(),
        crate::repository::SESSION_SECONDS,
    ))
}
pub fn set_cookie(response: &mut Response, cookie: String) -> Result<(), ApiError> {
    response.headers_mut().append(
        header::SET_COOKIE,
        cookie.parse().map_err(|_| ApiError::unavailable())?,
    );
    Ok(())
}

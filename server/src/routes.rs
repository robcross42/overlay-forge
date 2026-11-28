use crate::{
    error::ApiError,
    repository::BrowserSession,
    security::{cookie, cookie_value, seconds, token_hash},
    Shared,
};
use axum::{
    extract::{DefaultBodyLimit, FromRequest, Path, Request, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use subtle::ConstantTimeEq;
use tower_http::services::ServeDir;

pub fn router(state: Shared) -> Router {
    Router::new()
        .route(
            "/health",
            get(|| async { Json(json!({"service":"overlay-forge-server"})) }),
        )
        .route("/auth/start", post(crate::oauth::start))
        .route("/auth/callback", get(crate::oauth::callback))
        .route("/auth/logout", post(logout))
        .route("/api/session", get(session))
        .route("/api/navigation", post(save_navigation))
        .route("/api/local-llm/chat", post(local_llm_chat))
        .route("/api/media/{command}", post(media_command))
        .fallback_service(ServeDir::new(&state.config.frontend_dir))
        .layer(DefaultBodyLimit::max(64 * 1024))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            security_headers,
        ))
        .with_state(state)
}
async fn security_headers(State(state): State<Shared>, request: Request, next: Next) -> Response {
    let valid_host = request
        .headers()
        .get(header::HOST)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|host| host.eq_ignore_ascii_case(&state.config.expected_host()));
    if !valid_host {
        return ApiError(StatusCode::BAD_REQUEST, "Unexpected server address.").into_response();
    }
    let mut response = next.run(request).await;
    let headers = response.headers_mut();
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    headers.insert(
        "x-content-type-options",
        HeaderValue::from_static("nosniff"),
    );
    headers.insert("referrer-policy", HeaderValue::from_static("no-referrer"));
    headers.insert("content-security-policy",HeaderValue::from_static("default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' https: data: blob:; connect-src 'self'; frame-ancestors 'none'; object-src 'none'; base-uri 'none'; form-action 'self'"));
    if state.config.secure() {
        headers.insert(
            "strict-transport-security",
            HeaderValue::from_static("max-age=15552000"),
        );
    }
    response
}
pub fn check_origin(state: &Shared, headers: &HeaderMap) -> Result<(), ApiError> {
    if headers
        .get(header::ORIGIN)
        .and_then(|value| value.to_str().ok())
        != Some(&state.config.public_origin)
    {
        return Err(ApiError::forbidden());
    }
    Ok(())
}
fn authenticate(state: &Shared, headers: &HeaderMap) -> Result<(String, BrowserSession), ApiError> {
    let token =
        cookie_value(headers, state.config.session_cookie()).ok_or_else(ApiError::unauthorized)?;
    let hash = token_hash(&token);
    let session = state
        .repository
        .session(&hash, seconds())
        .map_err(|_| ApiError::unavailable())?
        .ok_or_else(ApiError::unauthorized)?;
    Ok((hash, session))
}
fn authorize_write(
    state: &Shared,
    headers: &HeaderMap,
) -> Result<(String, BrowserSession), ApiError> {
    check_origin(state, headers)?;
    let auth = authenticate(state, headers)?;
    let csrf = headers
        .get("x-of-csrf")
        .and_then(|value| value.to_str().ok())
        .ok_or_else(ApiError::forbidden)?;
    if !bool::from(csrf.as_bytes().ct_eq(auth.1.csrf_token.as_bytes())) {
        return Err(ApiError::forbidden());
    }
    Ok(auth)
}
fn is_local_llm_owner(state: &Shared, session: &BrowserSession) -> bool {
    state.config.local_llm_owner_email.as_deref() == Some(session.email.as_str())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SessionResponse {
    #[serde(flatten)]
    session: BrowserSession,
    local_llm_enabled: bool,
}

async fn session(
    State(state): State<Shared>,
    headers: HeaderMap,
) -> Result<Json<SessionResponse>, ApiError> {
    let (_, session) = authenticate(&state, &headers)?;
    Ok(Json(SessionResponse {
        local_llm_enabled: is_local_llm_owner(&state, &session),
        session,
    }))
}

async fn local_llm_chat(
    State(state): State<Shared>,
    request: Request,
) -> Result<Json<crate::local_llm::ChatReply>, ApiError> {
    let (_, session) = authorize_write(&state, request.headers())?;
    let service = state
        .local_llm
        .as_ref()
        .filter(|_| is_local_llm_owner(&state, &session))
        .ok_or_else(ApiError::forbidden)?;
    let Json(input) = Json::<crate::local_llm::ChatInput>::from_request(request, &state)
        .await
        .map_err(|_| ApiError(StatusCode::BAD_REQUEST, "Invalid local chat request."))?;
    service.chat(input).await.map(Json)
}
async fn logout(State(state): State<Shared>, headers: HeaderMap) -> Result<Response, ApiError> {
    let (hash, _) = authorize_write(&state, &headers)?;
    state
        .repository
        .logout(&hash)
        .map_err(|_| ApiError::unavailable())?;
    let mut response = StatusCode::NO_CONTENT.into_response();
    crate::oauth::set_cookie(
        &mut response,
        cookie(state.config.session_cookie(), "", state.config.secure(), 0),
    )?;
    Ok(response)
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NavigationInput {
    collapsed: bool,
}
async fn save_navigation(
    State(state): State<Shared>,
    headers: HeaderMap,
    Json(input): Json<NavigationInput>,
) -> Result<Json<Value>, ApiError> {
    let (_, session) = authorize_write(&state, &headers)?;
    state
        .repository
        .save_navigation(&session.email, input.collapsed)
        .map_err(|_| ApiError::unavailable())?;
    Ok(Json(Value::Null))
}
fn bounded_json(value: &Value, depth: usize) -> bool {
    if depth > 16 {
        return false;
    }
    match value {
        Value::String(text) => text.len() <= 32 * 1024,
        Value::Array(values) => {
            values.len() <= 1000 && values.iter().all(|value| bounded_json(value, depth + 1))
        }
        Value::Object(values) => {
            values.len() <= 128
                && values
                    .iter()
                    .all(|(key, value)| key.len() <= 128 && bounded_json(value, depth + 1))
        }
        _ => true,
    }
}
async fn media_command(
    State(state): State<Shared>,
    headers: HeaderMap,
    Path(command): Path<String>,
    Json(args): Json<Value>,
) -> Result<Json<Value>, ApiError> {
    authorize_write(&state, &headers)?;
    if command.len() > 64
        || !overlay_forge_media_lib::runtime::MediaRuntime::handles_command(&command)
    {
        return Err(ApiError::forbidden());
    }
    if !args.is_object() || !bounded_json(&args, 0) {
        return Err(ApiError(StatusCode::BAD_REQUEST, "Invalid Media request."));
    }
    let permit = state
        .workers
        .clone()
        .try_acquire_owned()
        .map_err(|_| ApiError::unavailable())?;
    let result = tokio::task::spawn_blocking(move || {
        let _permit = permit;
        state.media.dispatch(&command, args)
    })
    .await
    .map_err(|_| ApiError::unavailable())?;
    result.map(Json).map_err(ApiError::media)
}

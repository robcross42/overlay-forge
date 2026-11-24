use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

pub struct ApiError(pub StatusCode, pub &'static str);
impl ApiError {
    pub fn unauthorized() -> Self {
        Self(
            StatusCode::UNAUTHORIZED,
            "Sign in with an approved Google account.",
        )
    }
    pub fn forbidden() -> Self {
        Self(StatusCode::FORBIDDEN, "This request is not allowed.")
    }
    pub fn unavailable() -> Self {
        Self(
            StatusCode::SERVICE_UNAVAILABLE,
            "The service is temporarily unavailable.",
        )
    }
}
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.0, Json(json!({"error":self.1}))).into_response()
    }
}

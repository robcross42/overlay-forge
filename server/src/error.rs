use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

pub struct ApiError(pub StatusCode, pub &'static str);
impl ApiError {
    pub fn media(error: overlay_forge_media_lib::MediaRuntimeError) -> Self {
        use overlay_forge_media_lib::media::domain::MediaErrorKind;
        use overlay_forge_media_lib::MediaRuntimeError;
        match error {
            MediaRuntimeError::Operation(error)
                if error.kind == MediaErrorKind::MissingCredential =>
            {
                Self(
                    StatusCode::SERVICE_UNAVAILABLE,
                    "Media search is not configured on the server. Contact the app owner.",
                )
            }
            _ => Self(
                StatusCode::BAD_REQUEST,
                "Media request could not be completed. Check your input or provider configuration.",
            ),
        }
    }

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

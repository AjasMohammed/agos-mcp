use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;

#[derive(Debug)]
pub enum BrokerError {
    Unauthorized,
    NotFound(String),
    /// The account's refresh token is dead — a human must re-run the auth flow.
    ReauthRequired(String),
    Internal(String),
}

impl IntoResponse for BrokerError {
    fn into_response(self) -> Response {
        let (status, code, msg) = match self {
            BrokerError::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                "unauthorized",
                "missing or invalid bearer token".to_string(),
            ),
            BrokerError::NotFound(m) => (StatusCode::NOT_FOUND, "not_found", m),
            BrokerError::ReauthRequired(m) => (StatusCode::CONFLICT, "reauth_required", m),
            BrokerError::Internal(m) => (StatusCode::INTERNAL_SERVER_ERROR, "internal", m),
        };
        (status, Json(serde_json::json!({ "error": code, "detail": msg }))).into_response()
    }
}

use reqwest::StatusCode;

#[derive(Debug, thiserror::Error)]
pub enum GmailError {
    #[error("auth expired")]
    AuthExpired,
    #[error("scope missing: {reason}")]
    ScopeMissing { reason: String },
    #[error("quota exhausted")]
    QuotaExhausted,
    #[error("not found: {0}")]
    NotFound(String),
    #[error("rate limited")]
    RateLimited,
    #[error("invalid request: {0}")]
    InvalidRequest(String),
    #[error("transport error: {0}")]
    Transport(String),
    #[error("other error: {0}")]
    Other(String),
    #[error("reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("auth error: {0}")]
    Auth(#[from] crate::auth::AuthError),
}

pub async fn map_gmail_error(status: StatusCode, resp: reqwest::Response) -> GmailError {
    #[derive(serde::Deserialize)]
    struct GoogleErr {
        error: GoogleErrInner,
    }
    #[derive(serde::Deserialize)]
    struct GoogleErrInner {
        message: String,
        errors: Option<Vec<GoogleErrDetail>>,
    }
    #[derive(serde::Deserialize)]
    struct GoogleErrDetail {
        reason: Option<String>,
    }

    let body: GoogleErr = match resp.json().await {
        Ok(b) => b,
        Err(_) => return GmailError::Transport(format!("http {status}")),
    };
    let reason = body
        .error
        .errors
        .as_ref()
        .and_then(|v| v.first())
        .and_then(|d| d.reason.clone())
        .unwrap_or_default();

    match (status.as_u16(), reason.as_str()) {
        (401, _) => GmailError::AuthExpired,
        (403, "insufficientPermissions") => GmailError::ScopeMissing { reason },
        (403, "dailyLimitExceeded") | (403, "userRateLimitExceeded") => GmailError::QuotaExhausted,
        (404, _) => GmailError::NotFound(body.error.message),
        (429, _) => GmailError::RateLimited,
        (400, _) => GmailError::InvalidRequest(body.error.message),
        (500..=599, _) => GmailError::Transport(body.error.message),
        _ => GmailError::Other(body.error.message),
    }
}

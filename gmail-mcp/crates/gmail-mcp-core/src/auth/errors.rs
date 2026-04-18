#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("timeout waiting for callback")]
    Timeout,
    #[error("state mismatch in callback")]
    StateMismatch,
    #[error("malformed callback request")]
    MalformedCallback,
    #[error("user denied access")]
    UserDenied,
    #[error("no credentials available")]
    NoCredentials,
    #[error("no refresh token available")]
    NoRefreshToken,
    #[error("token was revoked")]
    Revoked,
    #[error("decryption failed")]
    Decrypt,
    #[error("provider error: {0}")]
    Provider(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("url parse error: {0}")]
    UrlParse(#[from] url::ParseError),
    #[error("keyring error: {0}")]
    Keyring(#[from] keyring::Error),
    #[error("serde error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("jwt error: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),
    #[error("crypto error: {0}")]
    Crypto(String),
    #[error("password hash error: {0}")]
    PasswordHash(String),
}

impl From<aes_gcm::Error> for AuthError {
    fn from(e: aes_gcm::Error) -> Self {
        Self::Crypto(e.to_string())
    }
}

impl From<argon2::password_hash::Error> for AuthError {
    fn from(e: argon2::password_hash::Error) -> Self {
        Self::PasswordHash(e.to_string())
    }
}

impl From<tokio::task::JoinError> for AuthError {
    fn from(e: tokio::task::JoinError) -> Self {
        Self::Io(std::io::Error::other(e.to_string()))
    }
}

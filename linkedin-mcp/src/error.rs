use thiserror::Error;

#[derive(Debug, Error)]
pub enum LinkedInMcpError {
    #[error("authentication required; run `linkedin-mcp auth`")]
    AuthRequired,
    #[error("scope missing: {0}")]
    ScopeMissing(String),
    #[error("rate limited; retry after {0}s")]
    RateLimited(u64),
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("media too large: {0} bytes (limit {1})")]
    MediaTooLarge(u64, u64),
    #[error("linkedin server error: {0}")]
    LinkedInServerError(String),
    #[error("network error: {0}")]
    Network(#[from] std::io::Error),
    #[error("unknown urn: {0}")]
    UnknownUrn(String),
}

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenRecord {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: OffsetDateTime,
    pub refresh_expires_at: Option<OffsetDateTime>,
    pub sub: String,                 // OIDC subject
    pub scopes: Vec<String>,
    pub client_id: String,           // which client ID issued this
}

impl TokenRecord {
    pub fn is_expiring_soon(&self) -> bool {
        self.expires_at - OffsetDateTime::now_utc() < time::Duration::minutes(5)
    }
    pub fn author_urn(&self) -> String {
        format!("urn:li:person:{}", self.sub)
    }
}

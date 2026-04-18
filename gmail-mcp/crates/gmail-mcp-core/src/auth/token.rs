use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(Clone, serde::Serialize, serde::Deserialize, Zeroize, ZeroizeOnDrop)]
pub struct TokenSet {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: i64, // unix seconds
    pub scopes: Vec<String>,
    pub account_email: String,
}

#[derive(serde::Deserialize)]
pub struct GoogleTokenResponse {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: i64,
    pub scope: String,
    pub id_token: Option<String>,
}

impl From<GoogleTokenResponse> for TokenSet {
    fn from(resp: GoogleTokenResponse) -> Self {
        Self {
            access_token: resp.access_token,
            refresh_token: resp.refresh_token,
            expires_at: chrono::Utc::now().timestamp() + resp.expires_in,
            scopes: resp
                .scope
                .split_whitespace()
                .map(|s| s.to_string())
                .collect(),
            account_email: "".into(), // Typically parsed from id_token or fetched from userinfo
        }
    }
}

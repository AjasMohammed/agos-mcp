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

/// Re-auth is flagged this far ahead of refresh-token expiry so a human can act
/// before long-running agents start failing.
pub const REAUTH_WARN_WINDOW: time::Duration = time::Duration::days(7);

impl TokenRecord {
    pub fn is_expiring_soon(&self) -> bool {
        self.expires_at - OffsetDateTime::now_utc() < time::Duration::minutes(5)
    }
    pub fn author_urn(&self) -> String {
        format!("urn:li:person:{}", self.sub)
    }

    /// Seconds until the access token expires (negative if already expired).
    pub fn access_expires_in_seconds(&self) -> i64 {
        (self.expires_at - OffsetDateTime::now_utc()).whole_seconds()
    }

    /// Seconds until the refresh token expires, if LinkedIn reported its TTL.
    pub fn refresh_expires_in_seconds(&self) -> Option<i64> {
        self.refresh_expires_at
            .map(|exp| (exp - OffsetDateTime::now_utc()).whole_seconds())
    }

    /// True when the refresh token is missing, already expired, or within the
    /// warning window — i.e. a human re-auth is needed now or soon.
    pub fn needs_reauth_soon(&self) -> bool {
        if self.refresh_token.is_none() {
            return true;
        }
        match self.refresh_expires_at {
            Some(exp) => exp - OffsetDateTime::now_utc() < REAUTH_WARN_WINDOW,
            // No TTL reported: can't prove it's expiring, assume healthy.
            None => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(refresh_token: Option<&str>, refresh_expires_at: Option<OffsetDateTime>) -> TokenRecord {
        TokenRecord {
            access_token: "a".into(),
            refresh_token: refresh_token.map(String::from),
            expires_at: OffsetDateTime::now_utc() + time::Duration::hours(1),
            refresh_expires_at,
            sub: "s".into(),
            scopes: vec![],
            client_id: "c".into(),
        }
    }

    #[test]
    fn missing_refresh_token_needs_reauth() {
        assert!(record(None, None).needs_reauth_soon());
    }

    #[test]
    fn refresh_token_far_from_expiry_is_healthy() {
        let exp = OffsetDateTime::now_utc() + time::Duration::days(60);
        assert!(!record(Some("rt"), Some(exp)).needs_reauth_soon());
    }

    #[test]
    fn refresh_token_inside_warn_window_needs_reauth() {
        let exp = OffsetDateTime::now_utc() + time::Duration::days(3);
        assert!(record(Some("rt"), Some(exp)).needs_reauth_soon());
    }

    #[test]
    fn refresh_token_without_ttl_assumed_healthy() {
        assert!(!record(Some("rt"), None).needs_reauth_soon());
    }

    #[test]
    fn access_expiry_seconds_positive_when_valid() {
        assert!(record(Some("rt"), None).access_expires_in_seconds() > 0);
    }
}

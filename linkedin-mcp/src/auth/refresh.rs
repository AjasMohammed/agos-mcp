use serde::Deserialize;

use super::token::TokenRecord;

/// Why a refresh attempt failed, so callers can react correctly: a dead refresh
/// token requires human re-auth and must NOT be retried, whereas a transient
/// failure (network blip, 5xx) is safe to retry on the next call.
#[derive(Debug)]
pub enum RefreshError {
    /// Refresh token is missing, expired, or revoked — a human must re-run
    /// `linkedin-mcp auth`. Retrying is pointless.
    ReauthRequired(String),
    /// Transient or configuration failure; the existing token is left intact
    /// and the next request may retry.
    Transient(String),
}

impl std::fmt::Display for RefreshError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RefreshError::ReauthRequired(m) => write!(f, "re-auth required: {m}"),
            RefreshError::Transient(m) => write!(f, "transient refresh error: {m}"),
        }
    }
}

const TOKEN_URL: &str = "https://www.linkedin.com/oauth/v2/accessToken";

/// Exchange the stored refresh token for a fresh access token, updating
/// `record` in place. `client_secret` is required for LinkedIn confidential
/// apps — the refresh grant is rejected without it.
pub async fn refresh(
    http: &reqwest::Client,
    record: &mut TokenRecord,
    client_secret: Option<&str>,
) -> Result<(), RefreshError> {
    refresh_at(http, record, client_secret, TOKEN_URL).await
}

/// Same as [`refresh`] but against an explicit token endpoint, so tests can
/// point it at a local mock server. Production code uses [`refresh`].
pub async fn refresh_at(
    http: &reqwest::Client,
    record: &mut TokenRecord,
    client_secret: Option<&str>,
    token_url: &str,
) -> Result<(), RefreshError> {
    let Some(rt) = record.refresh_token.clone() else {
        return Err(RefreshError::ReauthRequired("no refresh token stored".into()));
    };

    let mut form: Vec<(&str, &str)> = vec![
        ("grant_type", "refresh_token"),
        ("refresh_token", rt.as_str()),
        ("client_id", record.client_id.as_str()),
    ];
    if let Some(secret) = client_secret {
        form.push(("client_secret", secret));
    }

    let resp = http
        .post(token_url)
        .form(&form)
        .send()
        .await
        .map_err(|e| RefreshError::Transient(e.to_string()))?;

    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        // LinkedIn returns `invalid_grant` when the refresh token is expired or
        // revoked — unrecoverable without a human re-auth. Everything else
        // (incl. `invalid_client` from a missing/wrong secret, or 5xx) we treat
        // as transient so we don't discard a still-valid session.
        if body.contains("invalid_grant") {
            return Err(RefreshError::ReauthRequired(format!(
                "refresh token rejected ({status}): {body}"
            )));
        }
        return Err(RefreshError::Transient(format!(
            "token endpoint returned {status}: {body}"
        )));
    }

    let resp: RefreshResp = resp
        .json()
        .await
        .map_err(|e| RefreshError::Transient(e.to_string()))?;

    let now = time::OffsetDateTime::now_utc();
    record.access_token = resp.access_token;
    record.expires_at = now + time::Duration::seconds(resp.expires_in as i64);
    if let Some(new_rt) = resp.refresh_token {
        // LinkedIn rotates refresh tokens (sliding window): the old one is
        // invalidated, so we MUST persist the new one or the next refresh fails.
        record.refresh_token = Some(new_rt);
        if let Some(rt_ttl) = resp.refresh_token_expires_in {
            record.refresh_expires_at = Some(now + time::Duration::seconds(rt_ttl as i64));
        }
    }
    Ok(())
}

#[derive(Deserialize)]
struct RefreshResp {
    access_token: String,
    expires_in: u64,
    refresh_token: Option<String>,
    refresh_token_expires_in: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use axum::{Router, routing::post};
    use std::sync::Arc;
    use time::OffsetDateTime;

    fn record() -> TokenRecord {
        TokenRecord {
            access_token: "old".into(),
            refresh_token: Some("rt-old".into()),
            expires_at: OffsetDateTime::now_utc(),
            refresh_expires_at: None,
            sub: "sub".into(),
            scopes: vec!["w_member_social".into()],
            client_id: "cid".into(),
        }
    }

    /// Mock token endpoint: echoes whether `client_secret` was in the form, and
    /// returns `(status, body)`. Returns the URL to pass to `refresh_at`.
    async fn mock(status: u16, body: &'static str) -> (String, Arc<std::sync::atomic::AtomicBool>) {
        use std::sync::atomic::{AtomicBool, Ordering};
        let saw_secret = Arc::new(AtomicBool::new(false));
        let flag = saw_secret.clone();
        let app = Router::new().route(
            "/token",
            post(move |body_str: String| {
                let flag = flag.clone();
                async move {
                    if body_str.contains("client_secret=") {
                        flag.store(true, Ordering::SeqCst);
                    }
                    (StatusCode::from_u16(status).unwrap(), body)
                }
            }),
        );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        (format!("http://{addr}/token"), saw_secret)
    }

    #[tokio::test]
    async fn sends_client_secret_and_updates_tokens() {
        let (url, saw_secret) = mock(
            200,
            r#"{"access_token":"new","expires_in":3600,"refresh_token":"rt-new","refresh_token_expires_in":5184000}"#,
        )
        .await;
        let http = reqwest::Client::new();
        let mut rec = record();
        refresh_at(&http, &mut rec, Some("the-secret"), &url).await.unwrap();
        assert!(saw_secret.load(std::sync::atomic::Ordering::SeqCst), "client_secret must be sent");
        assert_eq!(rec.access_token, "new");
        assert_eq!(rec.refresh_token.as_deref(), Some("rt-new"), "rotated refresh token must be persisted");
        assert!(rec.refresh_expires_at.is_some());
    }

    #[tokio::test]
    async fn invalid_grant_maps_to_reauth_required() {
        let (url, _) = mock(400, r#"{"error":"invalid_grant","error_description":"expired"}"#).await;
        let http = reqwest::Client::new();
        let mut rec = record();
        let err = refresh_at(&http, &mut rec, Some("s"), &url).await.unwrap_err();
        assert!(matches!(err, RefreshError::ReauthRequired(_)));
        assert_eq!(rec.access_token, "old", "token must be left intact on failure");
    }

    #[tokio::test]
    async fn server_error_maps_to_transient() {
        let (url, _) = mock(503, "upstream down").await;
        let http = reqwest::Client::new();
        let mut rec = record();
        let err = refresh_at(&http, &mut rec, Some("s"), &url).await.unwrap_err();
        assert!(matches!(err, RefreshError::Transient(_)));
    }

    #[tokio::test]
    async fn missing_refresh_token_requires_reauth() {
        let http = reqwest::Client::new();
        let mut rec = record();
        rec.refresh_token = None;
        let err = refresh_at(&http, &mut rec, Some("s"), "http://127.0.0.1:0/token")
            .await
            .unwrap_err();
        assert!(matches!(err, RefreshError::ReauthRequired(_)));
    }
}

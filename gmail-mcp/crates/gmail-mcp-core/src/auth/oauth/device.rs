use crate::auth::errors::AuthError;
use crate::auth::token::{GoogleTokenResponse, TokenSet};
use reqwest::Client;
use std::time::{Duration, Instant};

#[derive(serde::Deserialize)]
struct DeviceCodeResponse {
    device_code: String,
    user_code: String,
    verification_url: String,
    expires_in: u64,
    interval: u64,
}

#[derive(serde::Deserialize)]
struct GoogleOAuthError {
    error: String,
}

/// Information returned to the caller after starting a device-code flow.
/// The caller (typically a human in front of a browser) must visit
/// `verification_url` and enter `user_code` within `expires_in` seconds.
#[derive(Debug, serde::Serialize)]
pub struct DeviceCodeInit {
    pub device_code: String,
    pub user_code: String,
    pub verification_url: String,
    pub expires_in: u64,
    pub interval: u64,
}

/// Outcome of a single poll against Google's token endpoint.
pub enum DevicePollOutcome {
    Granted(TokenSet),
    Pending {
        /// If `Some(n)`, Google asked us to slow down by an extra `n` seconds.
        slow_down_extra: Option<u64>,
    },
}

pub struct DeviceFlow {
    client_id: String,
    scopes: Vec<String>,
    /// OAuth endpoint root. Defaults to Google; overridable so tests can point
    /// the flow at a local mock server.
    base_url: String,
}

impl DeviceFlow {
    pub fn new(client_id: String, scopes: Vec<String>) -> Self {
        Self::with_base_url(client_id, scopes, "https://oauth2.googleapis.com".to_string())
    }

    /// Construct a flow against a custom OAuth endpoint root (e.g. a mock
    /// server in tests). Production code uses [`DeviceFlow::new`].
    pub fn with_base_url(client_id: String, scopes: Vec<String>, base_url: String) -> Self {
        Self {
            client_id,
            scopes,
            base_url: base_url.trim_end_matches('/').to_string(),
        }
    }

    /// Initiate a device-code flow. Returns the codes/URL the human needs.
    /// `login_hint` (typically an email) pre-fills the consent screen.
    pub async fn begin(&self, login_hint: Option<&str>) -> Result<DeviceCodeInit, AuthError> {
        let scope = self.scopes.join(" ");
        let mut params: Vec<(&str, &str)> = vec![
            ("client_id", self.client_id.as_str()),
            ("scope", scope.as_str()),
        ];
        if let Some(hint) = login_hint {
            params.push(("login_hint", hint));
        }

        let init: DeviceCodeResponse = Client::new()
            .post(format!("{}/device/code", self.base_url))
            .form(&params)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        Ok(DeviceCodeInit {
            device_code: init.device_code,
            user_code: init.user_code,
            verification_url: init.verification_url,
            expires_in: init.expires_in,
            interval: init.interval.max(5),
        })
    }

    /// Run a single poll against the token endpoint for the given device_code.
    /// Caller is responsible for sleeping between polls.
    pub async fn poll_once(&self, device_code: &str) -> Result<DevicePollOutcome, AuthError> {
        let resp = Client::new()
            .post(format!("{}/token", self.base_url))
            .form(&[
                ("client_id", self.client_id.as_str()),
                ("device_code", device_code),
                ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
            ])
            .send()
            .await?;

        if resp.status().is_success() {
            return Ok(DevicePollOutcome::Granted(TokenSet::from(
                resp.json::<GoogleTokenResponse>().await?,
            )));
        }
        let err: GoogleOAuthError = resp.json().await?;
        match err.error.as_str() {
            "authorization_pending" => Ok(DevicePollOutcome::Pending {
                slow_down_extra: None,
            }),
            "slow_down" => Ok(DevicePollOutcome::Pending {
                slow_down_extra: Some(5),
            }),
            "access_denied" => Err(AuthError::UserDenied),
            "expired_token" => Err(AuthError::Timeout),
            other => Err(AuthError::Provider(other.into())),
        }
    }

    /// Convenience wrapper that drives begin/poll/sleep until completion.
    /// Used by the CLI; MCP tools call begin/poll_once directly so each
    /// poll is a separate tool call the agent can drive.
    pub async fn run(&self) -> Result<TokenSet, AuthError> {
        let init = self.begin(None).await?;
        eprintln!(
            "Visit {} and enter code: {}",
            init.verification_url, init.user_code
        );

        let deadline = Instant::now() + Duration::from_secs(init.expires_in);
        let mut interval = init.interval;

        while Instant::now() < deadline {
            tokio::time::sleep(Duration::from_secs(interval)).await;
            match self.poll_once(&init.device_code).await? {
                DevicePollOutcome::Granted(t) => return Ok(t),
                DevicePollOutcome::Pending { slow_down_extra } => {
                    if let Some(extra) = slow_down_extra {
                        interval += extra;
                    }
                }
            }
        }
        Err(AuthError::Timeout)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use axum::{Json, Router, routing::post};
    use std::sync::Arc;

    /// Spawn a one-route mock server. `path` (e.g. "/token") always responds
    /// with `(status, body)`. Returns the base URL to hand to `with_base_url`.
    async fn mock(path: &'static str, status: u16, body: serde_json::Value) -> String {
        let canned = Arc::new((status, body));
        let app = Router::new().route(
            path,
            post(move || {
                let canned = canned.clone();
                async move {
                    let code = StatusCode::from_u16(canned.0).unwrap();
                    (code, Json(canned.1.clone()))
                }
            }),
        );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        format!("http://{addr}")
    }

    fn flow(base: String) -> DeviceFlow {
        DeviceFlow::with_base_url("client".into(), vec!["scope.a".into()], base)
    }

    #[tokio::test]
    async fn poll_success_yields_granted_tokens() {
        let base = mock(
            "/token",
            200,
            serde_json::json!({
                "access_token": "at-123",
                "expires_in": 3600,
                "scope": "scope.a scope.b"
            }),
        )
        .await;
        match flow(base).poll_once("dc").await.expect("poll ok") {
            DevicePollOutcome::Granted(ts) => {
                assert_eq!(ts.access_token, "at-123");
                assert_eq!(ts.scopes, vec!["scope.a", "scope.b"]);
            }
            DevicePollOutcome::Pending { .. } => panic!("expected Granted"),
        }
    }

    #[tokio::test]
    async fn poll_authorization_pending_yields_pending_no_slowdown() {
        let base = mock("/token", 400, serde_json::json!({ "error": "authorization_pending" })).await;
        match flow(base).poll_once("dc").await.expect("poll ok") {
            DevicePollOutcome::Pending { slow_down_extra } => assert_eq!(slow_down_extra, None),
            DevicePollOutcome::Granted(_) => panic!("expected Pending"),
        }
    }

    #[tokio::test]
    async fn poll_slow_down_adds_extra_interval() {
        let base = mock("/token", 400, serde_json::json!({ "error": "slow_down" })).await;
        match flow(base).poll_once("dc").await.expect("poll ok") {
            DevicePollOutcome::Pending { slow_down_extra } => assert_eq!(slow_down_extra, Some(5)),
            DevicePollOutcome::Granted(_) => panic!("expected Pending"),
        }
    }

    #[tokio::test]
    async fn poll_access_denied_maps_to_user_denied() {
        let base = mock("/token", 400, serde_json::json!({ "error": "access_denied" })).await;
        assert!(matches!(
            flow(base).poll_once("dc").await,
            Err(AuthError::UserDenied)
        ));
    }

    #[tokio::test]
    async fn poll_expired_token_maps_to_timeout() {
        let base = mock("/token", 400, serde_json::json!({ "error": "expired_token" })).await;
        assert!(matches!(
            flow(base).poll_once("dc").await,
            Err(AuthError::Timeout)
        ));
    }

    #[tokio::test]
    async fn begin_floors_interval_at_five_seconds() {
        let base = mock(
            "/device/code",
            200,
            serde_json::json!({
                "device_code": "dc-1",
                "user_code": "UC-1",
                "verification_url": "https://example.test/verify",
                "expires_in": 1800,
                "interval": 2
            }),
        )
        .await;
        let init = flow(base).begin(None).await.expect("begin ok");
        assert_eq!(init.device_code, "dc-1");
        assert_eq!(init.interval, 5, "interval should be floored to 5s");
    }
}

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

pub struct DeviceFlow {
    client_id: String,
    scopes: Vec<String>,
}

impl DeviceFlow {
    pub fn new(client_id: String, scopes: Vec<String>) -> Self {
        Self { client_id, scopes }
    }

    pub async fn run(&self) -> Result<TokenSet, AuthError> {
        let http = Client::new();
        let init: DeviceCodeResponse = http
            .post("https://oauth2.googleapis.com/device/code")
            .form(&[
                ("client_id", self.client_id.as_str()),
                ("scope", &self.scopes.join(" ")),
            ])
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        eprintln!(
            "Visit {} and enter code: {}",
            init.verification_url, init.user_code
        );

        let deadline = Instant::now() + Duration::from_secs(init.expires_in);
        let mut interval = init.interval.max(5);

        while Instant::now() < deadline {
            tokio::time::sleep(Duration::from_secs(interval)).await;
            let poll = http
                .post("https://oauth2.googleapis.com/token")
                .form(&[
                    ("client_id", self.client_id.as_str()),
                    ("device_code", &init.device_code),
                    ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
                ])
                .send()
                .await?;
            if poll.status().is_success() {
                return Ok(TokenSet::from(poll.json::<GoogleTokenResponse>().await?));
            }
            let err: GoogleOAuthError = poll.json().await?;
            match err.error.as_str() {
                "authorization_pending" => continue,
                "slow_down" => {
                    interval += 5;
                    continue;
                }
                "access_denied" => return Err(AuthError::UserDenied),
                "expired_token" => return Err(AuthError::Timeout),
                other => return Err(AuthError::Provider(other.into())),
            }
        }
        Err(AuthError::Timeout)
    }
}

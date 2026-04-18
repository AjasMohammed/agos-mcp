use super::errors::AuthError;
use super::store::TokenStore;
use super::token::{GoogleTokenResponse, TokenSet};
use reqwest::StatusCode;
use std::sync::Arc;

pub struct TokenManager {
    store: Arc<dyn TokenStore>,
    client_id: String,
    account: String,
    cached: tokio::sync::RwLock<Option<TokenSet>>,
}

impl TokenManager {
    pub fn new(store: Arc<dyn TokenStore>, client_id: String, account: String) -> Self {
        Self {
            store,
            client_id,
            account,
            cached: tokio::sync::RwLock::new(None),
        }
    }

    pub async fn access_token(&self) -> Result<String, AuthError> {
        {
            let guard = self.cached.read().await;
            if let Some(t) = guard.as_ref() {
                if !self.is_near_expiry(t) {
                    return Ok(t.access_token.clone());
                }
            }
        }
        self.refresh().await
    }

    fn is_near_expiry(&self, t: &TokenSet) -> bool {
        chrono::Utc::now().timestamp() + 60 >= t.expires_at // 60s headroom
    }

    async fn refresh(&self) -> Result<String, AuthError> {
        let mut guard = self.cached.write().await;
        // Reload from store in case another instance refreshed since we checked.
        let current = self
            .store
            .get(&self.account)
            .await?
            .ok_or(AuthError::NoCredentials)?;

        if !self.is_near_expiry(&current) {
            let token = current.access_token.clone();
            *guard = Some(current);
            return Ok(token);
        }

        let refresh_token = current
            .refresh_token
            .as_ref()
            .ok_or(AuthError::NoRefreshToken)?;

        let resp: GoogleTokenResponse = reqwest::Client::new()
            .post("https://oauth2.googleapis.com/token")
            .form(&[
                ("client_id", self.client_id.as_str()),
                ("refresh_token", refresh_token),
                ("grant_type", "refresh_token"),
            ])
            .send()
            .await?
            .error_for_status()
            .map_err(|e| {
                if e.status() == Some(StatusCode::BAD_REQUEST) {
                    AuthError::Revoked
                } else {
                    e.into()
                }
            })?
            .json()
            .await?;

        let updated = TokenSet {
            access_token: resp.access_token,
            refresh_token: resp.refresh_token.or(current.refresh_token.clone()),
            expires_at: chrono::Utc::now().timestamp() + resp.expires_in,
            scopes: current.scopes.clone(),
            account_email: current.account_email.clone(),
        };
        self.store.put(&self.account, &updated).await?;
        let token = updated.access_token.clone();
        *guard = Some(updated);
        Ok(token)
    }
}

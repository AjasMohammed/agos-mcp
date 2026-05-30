use anyhow::{Context, Result};
use async_trait::async_trait;
use keyring::Entry;
use super::token::TokenRecord;

#[async_trait]
pub trait TokenStore: Send + Sync {
    async fn load(&self, account: &str) -> Result<Option<TokenRecord>>;
    async fn save(&self, account: &str, record: &TokenRecord) -> Result<()>;
    async fn delete(&self, account: &str) -> Result<()>;
    /// True for stores that source already-valid tokens from a remote broker.
    /// The client then reloads on expiry instead of running a local OAuth
    /// refresh (it has no client secret or refresh token in this mode).
    fn is_remote(&self) -> bool {
        false
    }
}

pub struct KeychainStore;

#[async_trait]
impl TokenStore for KeychainStore {
    async fn load(&self, account: &str) -> Result<Option<TokenRecord>> {
        let entry = Entry::new("linkedin-mcp", account)?;
        match entry.get_password() {
            Ok(s) => Ok(Some(serde_json::from_str(&s)?)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(e).context("keychain load"),
        }
    }
    async fn save(&self, account: &str, record: &TokenRecord) -> Result<()> {
        let entry = Entry::new("linkedin-mcp", account)?;
        entry.set_password(&serde_json::to_string(record)?)?;
        Ok(())
    }
    async fn delete(&self, account: &str) -> Result<()> {
        let entry = Entry::new("linkedin-mcp", account)?;
        match entry.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }
}

pub struct FileStore {
    dir: std::path::PathBuf,
}

impl FileStore {
    pub fn new(dir: std::path::PathBuf) -> Self { Self { dir } }
}

#[async_trait]
impl TokenStore for FileStore {
    async fn load(&self, account: &str) -> Result<Option<TokenRecord>> {
        let path = self.dir.join(format!("{account}.json"));
        if !path.exists() { return Ok(None); }
        let s = std::fs::read_to_string(&path)?;
        Ok(Some(serde_json::from_str(&s)?))
    }
    async fn save(&self, account: &str, record: &TokenRecord) -> Result<()> {
        std::fs::create_dir_all(&self.dir)?;
        let path = self.dir.join(format!("{account}.json"));
        let tmp = self.dir.join(format!("{account}.json.tmp"));
        std::fs::write(&tmp, serde_json::to_string_pretty(record)?)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&tmp)?.permissions();
            perms.set_mode(0o600);
            std::fs::set_permissions(&tmp, perms)?;
        }
        std::fs::rename(&tmp, &path)?;
        Ok(())
    }
    async fn delete(&self, account: &str) -> Result<()> {
        let path = self.dir.join(format!("{account}.json"));
        if path.exists() { std::fs::remove_file(&path)?; }
        Ok(())
    }
}

/// Sources tokens from a central [`linkedin-auth-broker`] over its `/li/token`
/// endpoint. The broker holds the client secret and refresh token and refreshes
/// centrally; this store just fetches an already-valid access token, so many
/// MCP hosts can share one identity without local secrets. `save`/`delete` are
/// no-ops — the broker owns persistence.
pub struct RemoteStore {
    base_url: String,
    api_token: String,
    http: reqwest::Client,
}

impl RemoteStore {
    pub fn new(base_url: String, api_token: String) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            api_token,
            http: reqwest::Client::new(),
        }
    }
}

#[derive(serde::Deserialize)]
struct BrokerToken {
    access_token: String,
    expires_in_seconds: i64,
    author_urn: String,
    scopes: Vec<String>,
}

#[async_trait]
impl TokenStore for RemoteStore {
    async fn load(&self, account: &str) -> Result<Option<TokenRecord>> {
        let url = format!("{}/li/token", self.base_url);
        let resp = self
            .http
            .get(&url)
            .query(&[("account", account)])
            .bearer_auth(&self.api_token)
            .send()
            .await
            .context("contacting auth broker")?;
        match resp.status() {
            s if s.is_success() => {
                let t: BrokerToken = resp.json().await.context("parse broker token")?;
                let sub = t
                    .author_urn
                    .strip_prefix("urn:li:person:")
                    .unwrap_or(&t.author_urn)
                    .to_string();
                Ok(Some(TokenRecord {
                    access_token: t.access_token,
                    // Marker, not a usable token: the broker owns refresh, so we
                    // never run the local refresh grant in remote mode.
                    refresh_token: Some("__broker_managed__".into()),
                    expires_at: time::OffsetDateTime::now_utc()
                        + time::Duration::seconds(t.expires_in_seconds),
                    refresh_expires_at: None,
                    sub,
                    scopes: t.scopes,
                    client_id: "broker".into(),
                }))
            }
            reqwest::StatusCode::NOT_FOUND => Ok(None),
            reqwest::StatusCode::CONFLICT => {
                anyhow::bail!("broker reports re-auth required for account '{account}'; run the broker's /li/start flow")
            }
            s => {
                let body = resp.text().await.unwrap_or_default();
                anyhow::bail!("broker error {s}: {body}")
            }
        }
    }
    async fn save(&self, _account: &str, _record: &TokenRecord) -> Result<()> {
        Ok(()) // broker owns persistence
    }
    async fn delete(&self, _account: &str) -> Result<()> {
        Ok(())
    }
    fn is_remote(&self) -> bool {
        true
    }
}

pub fn build_store(kind: &str) -> anyhow::Result<Box<dyn TokenStore>> {
    match kind {
        "keychain" => Ok(Box::new(KeychainStore)),
        "file" => {
            eprintln!("WARNING: --token-store file writes access tokens as PLAINTEXT JSON on disk. \
                       Anyone with read access to your data directory can steal your LinkedIn session. \
                       Use --token-store keychain (default) in production.");
            let dir = dirs::data_dir()
                .ok_or_else(|| anyhow::anyhow!("no data dir"))?
                .join("linkedin-mcp");
            Ok(Box::new(FileStore::new(dir)))
        }
        other => Err(anyhow::anyhow!("unknown token store: {other}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use axum::{routing::get, Router};
    use std::sync::Arc;

    async fn mock_broker(status: u16, body: serde_json::Value) -> String {
        // Return the JSON as a plain string body (this crate's axum has the
        // `json` feature off); reqwest's `.json()` parses it regardless.
        let canned = Arc::new((status, body.to_string()));
        let app = Router::new().route(
            "/li/token",
            get(move || {
                let canned = canned.clone();
                async move {
                    (
                        StatusCode::from_u16(canned.0).unwrap(),
                        canned.1.clone(),
                    )
                }
            }),
        );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        format!("http://{addr}")
    }

    #[tokio::test]
    async fn remote_store_builds_record_from_broker() {
        let base = mock_broker(
            200,
            serde_json::json!({
                "account": "default",
                "access_token": "broker-at",
                "expires_in_seconds": 3600,
                "author_urn": "urn:li:person:XYZ",
                "scopes": ["w_member_social"],
                "needs_reauth_soon": false
            }),
        )
        .await;
        let store = RemoteStore::new(base, "tok".into());
        assert!(store.is_remote());
        let rec = store.load("default").await.unwrap().unwrap();
        assert_eq!(rec.access_token, "broker-at");
        assert_eq!(rec.sub, "XYZ", "sub parsed from author_urn");
        assert_eq!(rec.author_urn(), "urn:li:person:XYZ");
        assert!(rec.access_expires_in_seconds() > 3000);
    }

    #[tokio::test]
    async fn remote_store_404_is_none() {
        let base = mock_broker(404, serde_json::json!({"error": "not_found"})).await;
        let store = RemoteStore::new(base, "tok".into());
        assert!(store.load("default").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn remote_store_409_is_reauth_error() {
        let base = mock_broker(409, serde_json::json!({"error": "reauth_required"})).await;
        let store = RemoteStore::new(base, "tok".into());
        assert!(store.load("default").await.is_err());
    }
}

use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use time::{Duration, OffsetDateTime};
use tokio::sync::RwLock;

/// Flag re-auth this far ahead of refresh-token expiry.
pub const REAUTH_WARN_WINDOW: Duration = Duration::days(7);

/// A stored LinkedIn identity. Field-compatible with the MCP's `TokenRecord`
/// so the `/li/token` response can be consumed directly.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenEntry {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: OffsetDateTime,
    pub refresh_expires_at: Option<OffsetDateTime>,
    pub sub: String,
    pub scopes: Vec<String>,
    pub client_id: String,
}

impl TokenEntry {
    /// Access token within 5 minutes of expiry (or already expired).
    pub fn access_expiring_soon(&self) -> bool {
        self.expires_at - OffsetDateTime::now_utc() < Duration::minutes(5)
    }
    pub fn needs_reauth_soon(&self) -> bool {
        if self.refresh_token.is_none() {
            return true;
        }
        match self.refresh_expires_at {
            Some(exp) => exp - OffsetDateTime::now_utc() < REAUTH_WARN_WINDOW,
            None => false,
        }
    }
    pub fn author_urn(&self) -> String {
        format!("urn:li:person:{}", self.sub)
    }
}

/// Pluggable persistence for broker tokens. The two impls here cover local and
/// single-node deployments; a Postgres/secret-manager impl plugs in here for
/// multi-node scale without touching the rest of the broker.
#[async_trait]
pub trait BrokerStore: Send + Sync {
    async fn get(&self, account: &str) -> Result<Option<TokenEntry>>;
    async fn put(&self, account: &str, entry: &TokenEntry) -> Result<()>;
    async fn list_accounts(&self) -> Result<Vec<String>>;
}

/// In-memory store — for tests and ephemeral/dev runs (tokens lost on restart).
#[derive(Default)]
pub struct InMemoryStore {
    map: RwLock<HashMap<String, TokenEntry>>,
}

#[async_trait]
impl BrokerStore for InMemoryStore {
    async fn get(&self, account: &str) -> Result<Option<TokenEntry>> {
        Ok(self.map.read().await.get(account).cloned())
    }
    async fn put(&self, account: &str, entry: &TokenEntry) -> Result<()> {
        self.map.write().await.insert(account.to_string(), entry.clone());
        Ok(())
    }
    async fn list_accounts(&self) -> Result<Vec<String>> {
        Ok(self.map.read().await.keys().cloned().collect())
    }
}

/// File-backed store: one `<account>.json` per identity, written atomically with
/// `0600` perms. NOTE: tokens are stored as plaintext JSON — run the broker on
/// an encrypted volume / restricted host, or implement an encrypted/secret-
/// manager-backed `BrokerStore` for stronger at-rest protection.
pub struct FileStore {
    dir: std::path::PathBuf,
}

impl FileStore {
    pub fn new(dir: std::path::PathBuf) -> Self {
        Self { dir }
    }
    fn path(&self, account: &str) -> std::path::PathBuf {
        self.dir.join(format!("{account}.json"))
    }
}

#[async_trait]
impl BrokerStore for FileStore {
    async fn get(&self, account: &str) -> Result<Option<TokenEntry>> {
        let path = self.path(account);
        if !path.exists() {
            return Ok(None);
        }
        let s = std::fs::read_to_string(&path).context("read token file")?;
        Ok(Some(serde_json::from_str(&s)?))
    }
    async fn put(&self, account: &str, entry: &TokenEntry) -> Result<()> {
        std::fs::create_dir_all(&self.dir)?;
        let path = self.path(account);
        let tmp = self.dir.join(format!("{account}.json.tmp"));
        std::fs::write(&tmp, serde_json::to_string_pretty(entry)?)?;
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
    async fn list_accounts(&self) -> Result<Vec<String>> {
        if !self.dir.exists() {
            return Ok(vec![]);
        }
        let mut out = vec![];
        for entry in std::fs::read_dir(&self.dir)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().to_string();
            if let Some(acct) = name.strip_suffix(".json") {
                out.push(acct.to_string());
            }
        }
        Ok(out)
    }
}

/// Build a store by kind: `memory` (ephemeral) or `file` (default, persistent).
pub fn build_store(kind: &str, dir: std::path::PathBuf) -> Arc<dyn BrokerStore> {
    match kind {
        "memory" => Arc::new(InMemoryStore::default()),
        _ => Arc::new(FileStore::new(dir)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(refresh_token: Option<&str>, refresh_exp: Option<Duration>) -> TokenEntry {
        let now = OffsetDateTime::now_utc();
        TokenEntry {
            access_token: "at".into(),
            refresh_token: refresh_token.map(String::from),
            expires_at: now + Duration::hours(1),
            refresh_expires_at: refresh_exp.map(|d| now + d),
            sub: "abc".into(),
            scopes: vec!["w_member_social".into()],
            client_id: "cid".into(),
        }
    }

    #[test]
    fn needs_reauth_logic() {
        assert!(entry(None, None).needs_reauth_soon(), "no refresh token");
        assert!(entry(Some("rt"), Some(Duration::days(3))).needs_reauth_soon(), "inside warn window");
        assert!(!entry(Some("rt"), Some(Duration::days(60))).needs_reauth_soon(), "healthy");
        assert!(!entry(Some("rt"), None).needs_reauth_soon(), "no TTL = assume healthy");
    }

    #[test]
    fn author_urn_format() {
        assert_eq!(entry(Some("rt"), None).author_urn(), "urn:li:person:abc");
    }

    #[tokio::test]
    async fn in_memory_roundtrip() {
        let store = InMemoryStore::default();
        assert!(store.get("default").await.unwrap().is_none());
        store.put("default", &entry(Some("rt"), None)).await.unwrap();
        store.put("work", &entry(Some("rt"), None)).await.unwrap();
        assert_eq!(store.get("default").await.unwrap().unwrap().access_token, "at");
        let mut accounts = store.list_accounts().await.unwrap();
        accounts.sort();
        assert_eq!(accounts, vec!["default", "work"]);
    }

    #[tokio::test]
    async fn file_store_roundtrip_and_list() {
        let dir = tempfile::tempdir().unwrap();
        let store = FileStore::new(dir.path().to_path_buf());
        assert!(store.list_accounts().await.unwrap().is_empty());
        store.put("acct", &entry(Some("rt"), Some(Duration::days(30)))).await.unwrap();
        let got = store.get("acct").await.unwrap().unwrap();
        assert_eq!(got.refresh_token.as_deref(), Some("rt"));
        assert_eq!(store.list_accounts().await.unwrap(), vec!["acct"]);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(dir.path().join("acct.json")).unwrap().permissions().mode();
            assert_eq!(mode & 0o777, 0o600, "token file must be 0600");
        }
    }
}

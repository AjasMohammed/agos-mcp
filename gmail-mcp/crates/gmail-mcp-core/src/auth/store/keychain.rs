use super::TokenStore;
use crate::auth::errors::AuthError;
use crate::auth::token::TokenSet;
use zeroize::Zeroize;

pub struct KeychainStore {
    service: &'static str, // "gmail-mcp"
}

impl KeychainStore {
    pub fn new(service: &'static str) -> Self {
        Self { service }
    }

    /// Probe whether the OS secret service is actually usable in this
    /// environment. A reachable-but-empty keychain reports `NoEntry`; a missing
    /// service (e.g. a headless/systemd session with no D-Bus) reports a
    /// platform error. Used to pick a token backend automatically so that
    /// `auth` and `serve` always agree on where tokens live.
    pub async fn is_available(service: &'static str) -> bool {
        tokio::task::spawn_blocking(move || {
            match keyring::Entry::new(service, "__keychain_probe__") {
                Ok(entry) => matches!(entry.get_secret(), Ok(_) | Err(keyring::Error::NoEntry)),
                Err(_) => false,
            }
        })
        .await
        .unwrap_or(false)
    }
}

#[async_trait::async_trait]
impl TokenStore for KeychainStore {
    async fn put(&self, account: &str, tokens: &TokenSet) -> Result<(), AuthError> {
        let mut bytes = serde_json::to_vec(tokens)?;
        let entry = keyring::Entry::new(self.service, account)?;
        let bytes_clone = bytes.clone();
        tokio::task::spawn_blocking(move || entry.set_secret(&bytes_clone)).await??;
        bytes.zeroize(); // best-effort; serde already allocated
        Ok(())
    }

    async fn get(&self, account: &str) -> Result<Option<TokenSet>, AuthError> {
        let entry = keyring::Entry::new(self.service, account)?;
        let bytes_res = tokio::task::spawn_blocking(move || entry.get_secret()).await?;
        match bytes_res {
            Ok(bytes) => {
                let tokens: TokenSet = serde_json::from_slice(&bytes)?;
                Ok(Some(tokens))
            }
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    async fn delete(&self, account: &str) -> Result<(), AuthError> {
        let entry = keyring::Entry::new(self.service, account)?;
        tokio::task::spawn_blocking(move || entry.delete_credential()).await??;
        Ok(())
    }

    async fn list_accounts(&self) -> Result<Vec<String>, AuthError> {
        // Mocked or file-based enumeration not fully implemented in keyring wrapper yet.
        Ok(vec![])
    }
}

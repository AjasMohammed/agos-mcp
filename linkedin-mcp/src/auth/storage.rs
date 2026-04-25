use anyhow::{Context, Result};
use keyring::Entry;
use super::token::TokenRecord;

pub trait TokenStore: Send + Sync {
    fn load(&self, account: &str) -> Result<Option<TokenRecord>>;
    fn save(&self, account: &str, record: &TokenRecord) -> Result<()>;
    fn delete(&self, account: &str) -> Result<()>;
}

pub struct KeychainStore;

impl TokenStore for KeychainStore {
    fn load(&self, account: &str) -> Result<Option<TokenRecord>> {
        let entry = Entry::new("linkedin-mcp", account)?;
        match entry.get_password() {
            Ok(s) => Ok(Some(serde_json::from_str(&s)?)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(e).context("keychain load"),
        }
    }
    fn save(&self, account: &str, record: &TokenRecord) -> Result<()> {
        let entry = Entry::new("linkedin-mcp", account)?;
        entry.set_password(&serde_json::to_string(record)?)?;
        Ok(())
    }
    fn delete(&self, account: &str) -> Result<()> {
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

impl TokenStore for FileStore {
    fn load(&self, account: &str) -> Result<Option<TokenRecord>> {
        let path = self.dir.join(format!("{account}.json"));
        if !path.exists() { return Ok(None); }
        let s = std::fs::read_to_string(&path)?;
        Ok(Some(serde_json::from_str(&s)?))
    }
    fn save(&self, account: &str, record: &TokenRecord) -> Result<()> {
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
    fn delete(&self, account: &str) -> Result<()> {
        let path = self.dir.join(format!("{account}.json"));
        if path.exists() { std::fs::remove_file(&path)?; }
        Ok(())
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

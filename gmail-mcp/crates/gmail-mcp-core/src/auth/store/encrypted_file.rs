use super::TokenStore;
use crate::auth::errors::AuthError;
use crate::auth::token::TokenSet;
use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, KeyInit},
};
use argon2::Argon2;
use std::collections::HashMap;
use std::path::PathBuf;
use zeroize::{Zeroize, Zeroizing};

pub struct EncryptedFileStore {
    path: PathBuf,
    passphrase: Zeroizing<String>,
}

impl EncryptedFileStore {
    pub fn new(path: PathBuf, passphrase: String) -> Self {
        Self {
            path,
            passphrase: Zeroizing::new(passphrase),
        }
    }

    async fn load(&self) -> Result<HashMap<String, TokenSet>, AuthError> {
        let path = self.path.clone();
        let sealed = match tokio::task::spawn_blocking(move || std::fs::read(&path)).await? {
            Ok(b) => b,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(HashMap::new()),
            Err(e) => return Err(e.into()),
        };
        let mut plaintext = open(&sealed, self.passphrase.as_bytes())?;
        let map: HashMap<String, TokenSet> = serde_json::from_slice(&plaintext)?;
        plaintext.zeroize();
        Ok(map)
    }
}

#[async_trait::async_trait]
impl TokenStore for EncryptedFileStore {
    async fn put(&self, account: &str, tokens: &TokenSet) -> Result<(), AuthError> {
        let mut map = self.load().await.unwrap_or_default();
        map.insert(account.to_string(), tokens.clone());
        let mut plaintext = serde_json::to_vec(&map)?;
        let sealed = seal(&plaintext, self.passphrase.as_bytes())?;
        let path = self.path.clone();
        tokio::task::spawn_blocking(move || write_secure_file(&path, &sealed)).await??;
        plaintext.zeroize();
        Ok(())
    }

    async fn get(&self, account: &str) -> Result<Option<TokenSet>, AuthError> {
        let map = self.load().await?;
        Ok(map.get(account).cloned())
    }

    async fn delete(&self, account: &str) -> Result<(), AuthError> {
        let mut map = self.load().await?;
        if map.remove(account).is_some() {
            let mut plaintext = serde_json::to_vec(&map)?;
            let sealed = seal(&plaintext, self.passphrase.as_bytes())?;
            let path = self.path.clone();
            tokio::task::spawn_blocking(move || write_secure_file(&path, &sealed)).await??;
            plaintext.zeroize();
        }
        Ok(())
    }

    async fn list_accounts(&self) -> Result<Vec<String>, AuthError> {
        let map = self.load().await?;
        Ok(map.keys().cloned().collect())
    }
}

fn write_secure_file(path: &std::path::Path, data: &[u8]) -> std::io::Result<()> {
    let mut opts = std::fs::OpenOptions::new();
    opts.write(true).create(true).truncate(true);

    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        opts.mode(0o600);
    }

    let mut file = opts.open(path)?;
    std::io::Write::write_all(&mut file, data)?;
    Ok(())
}

fn rand_bytes(n: usize) -> Vec<u8> {
    use rand::RngCore;
    let mut b = vec![0u8; n];
    rand::thread_rng().fill_bytes(&mut b);
    b
}

fn seal(plaintext: &[u8], passphrase: &[u8]) -> Result<Vec<u8>, AuthError> {
    let salt = rand_bytes(16);
    let mut key = Zeroizing::new([0u8; 32]);
    Argon2::default()
        .hash_password_into(passphrase, &salt, key.as_mut())
        .map_err(|e| AuthError::Crypto(e.to_string()))?;
    let cipher = Aes256Gcm::new_from_slice(&*key).map_err(|e| AuthError::Crypto(e.to_string()))?;
    let nonce_bytes = rand_bytes(12);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ct = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| AuthError::Crypto(e.to_string()))?;
    Ok([salt, nonce_bytes, ct].concat())
}

fn open(sealed: &[u8], passphrase: &[u8]) -> Result<Vec<u8>, AuthError> {
    if sealed.len() < 16 + 12 {
        return Err(AuthError::Decrypt);
    }
    let salt = &sealed[0..16];
    let nonce_bytes = &sealed[16..28];
    let ct = &sealed[28..];

    let mut key = Zeroizing::new([0u8; 32]);
    Argon2::default()
        .hash_password_into(passphrase, salt, key.as_mut())
        .map_err(|_| AuthError::Decrypt)?;
    let cipher = Aes256Gcm::new_from_slice(&*key).map_err(|_| AuthError::Decrypt)?;
    let nonce = Nonce::from_slice(nonce_bytes);
    cipher.decrypt(nonce, ct).map_err(|_| AuthError::Decrypt)
}

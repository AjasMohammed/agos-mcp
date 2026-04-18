use base64::Engine;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct HttpTokenMeta {
    pub account: String,
    pub created_at: u64,
    pub expires_at: u64,
    pub description: Option<String>,
}

pub struct HttpTokenManager {
    // In a real implementation this would persist to ~/.config/gmail-mcp/http-tokens.json
    tokens: std::sync::RwLock<HashMap<String, HttpTokenMeta>>,
}

impl Default for HttpTokenManager {
    fn default() -> Self {
        Self::new()
    }
}

impl HttpTokenManager {
    pub fn new() -> Self {
        Self {
            tokens: std::sync::RwLock::new(HashMap::new()),
        }
    }

    pub fn issue_token(
        &self,
        account: String,
        valid_seconds: u64,
        description: Option<String>,
    ) -> String {
        use rand::RngCore;
        let mut b = vec![0u8; 32];
        rand::thread_rng().fill_bytes(&mut b);
        let token = format!(
            "gmt_{}",
            base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&b)
        );

        let hash = self.hash_token(&token);
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let meta = HttpTokenMeta {
            account,
            created_at: now,
            expires_at: now + valid_seconds,
            description,
        };

        self.tokens.write().unwrap().insert(hash, meta);
        token
    }

    pub fn validate_token(&self, token: &str) -> Option<HttpTokenMeta> {
        let hash = self.hash_token(token);
        let tokens = self.tokens.read().unwrap();
        if let Some(meta) = tokens.get(&hash) {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            if now < meta.expires_at {
                return Some(meta.clone());
            }
        }
        None
    }

    pub fn revoke_token(&self, token: &str) -> bool {
        let hash = self.hash_token(token);
        self.tokens.write().unwrap().remove(&hash).is_some()
    }

    fn hash_token(&self, token: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(hasher.finalize())
    }
}

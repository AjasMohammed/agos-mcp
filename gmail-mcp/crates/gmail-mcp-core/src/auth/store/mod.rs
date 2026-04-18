pub mod encrypted_file;
pub mod keychain;

use super::errors::AuthError;
use super::token::TokenSet;
use async_trait::async_trait;

#[async_trait]
pub trait TokenStore: Send + Sync {
    async fn put(&self, account: &str, tokens: &TokenSet) -> Result<(), AuthError>;
    async fn get(&self, account: &str) -> Result<Option<TokenSet>, AuthError>;
    async fn delete(&self, account: &str) -> Result<(), AuthError>;
    async fn list_accounts(&self) -> Result<Vec<String>, AuthError>;
}

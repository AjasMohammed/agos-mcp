use crate::auth::store::TokenStore;
use crate::auth::{AuthError, TokenManager};
use crate::gmail::Client;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct AccountManager {
    store: Arc<dyn TokenStore>,
    client_id: String,
    clients: RwLock<HashMap<String, Arc<Client>>>,
}

impl AccountManager {
    pub fn new(store: Arc<dyn TokenStore>, client_id: String) -> Self {
        Self {
            store,
            client_id,
            clients: RwLock::new(HashMap::new()),
        }
    }

    pub async fn client(&self, account: &str) -> Result<Arc<Client>, AuthError> {
        {
            let clients = self.clients.read().await;
            if let Some(c) = clients.get(account) {
                return Ok(c.clone());
            }
        }

        let tokens = TokenManager::new(self.store.clone(), self.client_id.clone(), account.into());
        // Phase 7 rate limiter would be injected here.
        let client = Arc::new(Client::new(Arc::new(tokens)).await);

        let mut clients = self.clients.write().await;
        clients.insert(account.into(), client.clone());
        Ok(client)
    }
}

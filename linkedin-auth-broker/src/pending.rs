//! Server-side store of in-flight authorizations, keyed by the OAuth `state`
//! value. Used to validate the callback (CSRF) and recover which account the
//! flow was started for. Entries expire so abandoned flows don't accumulate.

use std::collections::HashMap;
use time::{Duration, OffsetDateTime};
use tokio::sync::Mutex;

const PENDING_TTL: Duration = Duration::minutes(10);

struct Entry {
    account: String,
    created_at: OffsetDateTime,
}

#[derive(Default)]
pub struct Pending {
    map: Mutex<HashMap<String, Entry>>,
}

impl Pending {
    /// Record a new in-flight authorization; opportunistically evict expired ones.
    pub async fn insert(&self, state: String, account: String) {
        let mut map = self.map.lock().await;
        let now = OffsetDateTime::now_utc();
        map.retain(|_, e| now - e.created_at < PENDING_TTL);
        map.insert(
            state,
            Entry {
                account,
                created_at: now,
            },
        );
    }

    /// Consume the entry for `state`, returning the account if present and not
    /// expired. Single-use: a state can only be redeemed once.
    pub async fn take(&self, state: &str) -> Option<String> {
        let mut map = self.map.lock().await;
        let entry = map.remove(state)?;
        if OffsetDateTime::now_utc() - entry.created_at >= PENDING_TTL {
            return None;
        }
        Some(entry.account)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn take_returns_account_then_consumes() {
        let p = Pending::default();
        p.insert("state-1".into(), "work".into()).await;
        assert_eq!(p.take("state-1").await.as_deref(), Some("work"));
        // single-use: a second take fails (CSRF / replay protection)
        assert!(p.take("state-1").await.is_none());
    }

    #[tokio::test]
    async fn unknown_state_is_none() {
        let p = Pending::default();
        assert!(p.take("nope").await.is_none());
    }
}

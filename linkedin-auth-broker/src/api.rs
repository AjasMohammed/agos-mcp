use crate::config::Config;
use crate::error::BrokerError;
use crate::oauth::{self, RefreshOutcome};
use crate::pending::Pending;
use crate::store::{BrokerStore, TokenEntry};
use axum::extract::{Query, State};
use axum::http::HeaderMap;
use axum::response::Html;
use axum::Json;
use rand::distributions::Alphanumeric;
use rand::Rng;
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct AppState {
    pub cfg: Arc<Config>,
    pub store: Arc<dyn BrokerStore>,
    pub pending: Arc<Pending>,
    pub http: reqwest::Client,
    /// Serializes refreshes so concurrent `/li/token` calls (and the scheduler)
    /// don't double-refresh and invalidate each other's rotated refresh token.
    /// Coarse but correct; per-account locking / DB row locks are the scale path.
    pub refresh_lock: Arc<Mutex<()>>,
}

fn account_or_default(a: Option<String>) -> String {
    a.filter(|s| !s.is_empty()).unwrap_or_else(|| "default".to_string())
}

/// Validate the internal bearer token on `/li/start` and `/li/token`.
fn check_auth(headers: &HeaderMap, cfg: &Config) -> Result<(), BrokerError> {
    let expected = format!("Bearer {}", cfg.api_token);
    match headers.get(axum::http::header::AUTHORIZATION).and_then(|v| v.to_str().ok()) {
        Some(got) if got == expected => Ok(()),
        _ => Err(BrokerError::Unauthorized),
    }
}

fn random_state() -> String {
    rand::thread_rng()
        .sample_iter(Alphanumeric)
        .take(43)
        .map(char::from)
        .collect()
}

pub async fn healthz() -> &'static str {
    "ok"
}

#[derive(Deserialize)]
pub struct StartQuery {
    account: Option<String>,
}

/// Begin an authorization. Returns the LinkedIn URL a human must open and
/// consent at. Internal — requires the bearer token.
pub async fn start(
    State(st): State<AppState>,
    headers: HeaderMap,
    Query(q): Query<StartQuery>,
) -> Result<Json<serde_json::Value>, BrokerError> {
    check_auth(&headers, &st.cfg)?;
    let account = account_or_default(q.account);
    let state = random_state();
    st.pending.insert(state.clone(), account.clone()).await;
    let url = oauth::build_authorize_url(&st.cfg, &state)
        .map_err(|e| BrokerError::Internal(e.to_string()))?;
    Ok(Json(serde_json::json!({
        "account": account,
        "authorize_url": url,
        "next_action": "Open authorize_url in a browser and consent. The broker stores the token on the callback."
    })))
}

#[derive(Deserialize)]
pub struct CallbackQuery {
    code: Option<String>,
    state: Option<String>,
    error: Option<String>,
    error_description: Option<String>,
}

/// OAuth redirect target. Public — LinkedIn calls it; CSRF is enforced via the
/// single-use `state`.
pub async fn callback(State(st): State<AppState>, Query(q): Query<CallbackQuery>) -> Html<String> {
    if let Some(err) = q.error {
        return Html(format!(
            "<h1>Authorization failed: {}</h1><p>{}</p>",
            err,
            q.error_description.unwrap_or_default()
        ));
    }
    let (Some(code), Some(state)) = (q.code, q.state) else {
        return Html("<h1>Invalid callback</h1><p>missing code or state</p>".into());
    };
    let Some(account) = st.pending.take(&state).await else {
        return Html("<h1>Invalid or expired state</h1><p>Start the flow again.</p>".into());
    };
    match oauth::exchange_code(&st.http, &st.cfg, &code).await {
        Ok(entry) => match st.store.put(&account, &entry).await {
            Ok(()) => {
                tracing::info!(account = %account, sub = %entry.sub, "stored LinkedIn token");
                Html("<h1>Authenticated. You may close this tab.</h1>".into())
            }
            Err(e) => {
                tracing::error!(account = %account, error = %e, "failed to persist token");
                Html("<h1>Internal error storing token</h1>".into())
            }
        },
        Err(e) => {
            tracing::error!(account = %account, error = %e, "code exchange failed");
            Html(format!("<h1>Token exchange failed</h1><p>{e}</p>"))
        }
    }
}

#[derive(Deserialize)]
pub struct TokenQuery {
    account: Option<String>,
}

/// Return a currently-valid access token for `account`, refreshing first if it
/// is near expiry. Internal — requires the bearer token.
pub async fn token(
    State(st): State<AppState>,
    headers: HeaderMap,
    Query(q): Query<TokenQuery>,
) -> Result<Json<serde_json::Value>, BrokerError> {
    check_auth(&headers, &st.cfg)?;
    let account = account_or_default(q.account);

    let entry = st
        .store
        .get(&account)
        .await
        .map_err(|e| BrokerError::Internal(e.to_string()))?
        .ok_or_else(|| BrokerError::NotFound(format!("no token for account '{account}'")))?;

    let entry = if entry.access_expiring_soon() {
        refresh_account(&st, &account).await?
    } else {
        entry
    };

    Ok(Json(serde_json::json!({
        "account": account,
        "access_token": entry.access_token,
        "expires_in_seconds": (entry.expires_at - time::OffsetDateTime::now_utc()).whole_seconds(),
        "author_urn": entry.author_urn(),
        "scopes": entry.scopes,
        "needs_reauth_soon": entry.needs_reauth_soon(),
    })))
}

/// Refresh one account under the global refresh lock, re-checking after the
/// lock is held so a concurrent refresh isn't duplicated.
pub async fn refresh_account(st: &AppState, account: &str) -> Result<TokenEntry, BrokerError> {
    let _guard = st.refresh_lock.lock().await;
    let mut entry = st
        .store
        .get(account)
        .await
        .map_err(|e| BrokerError::Internal(e.to_string()))?
        .ok_or_else(|| BrokerError::NotFound(format!("no token for account '{account}'")))?;

    if !entry.access_expiring_soon() {
        return Ok(entry); // someone else already refreshed
    }

    match oauth::refresh(&st.http, &st.cfg, &mut entry).await {
        RefreshOutcome::Refreshed => {
            st.store
                .put(account, &entry)
                .await
                .map_err(|e| BrokerError::Internal(e.to_string()))?;
            Ok(entry)
        }
        RefreshOutcome::ReauthRequired(m) => Err(BrokerError::ReauthRequired(m)),
        RefreshOutcome::Transient(m) => Err(BrokerError::Internal(m)),
    }
}

mod api;
mod config;
mod error;
mod oauth;
mod pending;
mod store;

use crate::api::AppState;
use crate::config::Config;
use crate::pending::Pending;
use axum::routing::{get, post};
use axum::Router;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let cfg = Arc::new(Config::from_env()?);
    let bind_addr = cfg.bind_addr.clone();
    let http = reqwest::Client::builder()
        .user_agent(concat!("linkedin-auth-broker/", env!("CARGO_PKG_VERSION")))
        .build()?;

    let state = AppState {
        cfg: cfg.clone(),
        store: store::build_store(&cfg.store_kind, cfg.store_dir.clone()),
        pending: Arc::new(Pending::default()),
        http,
        refresh_lock: Arc::new(Mutex::new(())),
    };

    tokio::spawn(refresh_scheduler(state.clone()));

    let app = Router::new()
        .route("/healthz", get(api::healthz))
        .route("/li/start", post(api::start))
        .route("/li/callback", get(api::callback))
        .route("/li/token", get(api::token))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    tracing::info!(addr = %bind_addr, redirect_uri = %cfg.redirect_uri(), "linkedin-auth-broker listening");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

/// Periodically refresh tokens nearing expiry and warn about refresh tokens
/// nearing their own (unrecoverable) expiry. A backstop to on-demand refresh in
/// `/li/token`, so tokens stay warm even for idle accounts.
async fn refresh_scheduler(st: AppState) {
    let mut tick = tokio::time::interval(Duration::from_secs(st.cfg.refresh_scan_secs));
    loop {
        tick.tick().await;
        let accounts = match st.store.list_accounts().await {
            Ok(a) => a,
            Err(e) => {
                tracing::error!(error = %e, "scheduler: list_accounts failed");
                continue;
            }
        };
        for account in accounts {
            let Ok(Some(entry)) = st.store.get(&account).await else {
                continue;
            };
            if entry.needs_reauth_soon() {
                tracing::warn!(account = %account, "refresh token expiring soon — human re-auth needed via /li/start");
            }
            if entry.access_expiring_soon() {
                match api::refresh_account(&st, &account).await {
                    Ok(_) => tracing::info!(account = %account, "scheduler: refreshed"),
                    Err(e) => tracing::warn!(account = %account, error = ?e, "scheduler: refresh failed"),
                }
            }
        }
    }
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
    tracing::info!("shutting down");
}

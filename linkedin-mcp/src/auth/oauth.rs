use anyhow::{Context, Result};
use axum::{extract::Query, response::Html, routing::get, Router};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rand::RngCore;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::sync::Arc;
use tokio::sync::oneshot;
use url::Url;

use crate::cli::AuthArgs;
use super::{storage::build_store, token::TokenRecord};

const DEFAULT_CLIENT_ID: &str = env!("LINKEDIN_EMBEDDED_CLIENT_ID");
const REDIRECT_PORTS: &[u16] = &[17423, 17424];
const AUTHORIZE_URL: &str = "https://www.linkedin.com/oauth/v2/authorization";
const TOKEN_URL: &str = "https://www.linkedin.com/oauth/v2/accessToken";
const USERINFO_URL: &str = "https://api.linkedin.com/v2/userinfo";
const SCOPES: &[&str] = &["openid", "profile", "email", "w_member_social"];

pub async fn run(args: AuthArgs) -> Result<()> {
    let client_id = args.client_id.clone().unwrap_or_else(|| DEFAULT_CLIENT_ID.to_string());
    if client_id.is_empty() {
        anyhow::bail!(
            "no client_id: pass --client-id or build with LINKEDIN_EMBEDDED_CLIENT_ID env var"
        );
    }
    let store = build_store(&args.token_store)?;

    let state = random_urlsafe(32);

    let (port, listener_handle, recv) = start_loopback().await?;
    let redirect_uri = format!("http://localhost:{port}/callback");

    let authorize = build_authorize_url(&client_id, &redirect_uri, &state)?;
    eprintln!("Opening browser to: {authorize}");
    let _ = webbrowser::open(authorize.as_str());

    let callback = recv.await.context("no callback received")?;
    listener_handle.abort();

    if let Some(err) = callback.error {
        anyhow::bail!(
            "LinkedIn authorization error: {} — {}",
            err,
            callback.error_description.as_deref().unwrap_or("no description")
        );
    }

    let code = callback.code.context("no code in callback")?;
    let cb_state = callback.state.context("no state in callback")?;
    if cb_state != state {
        anyhow::bail!("state mismatch — possible CSRF");
    }

    let token_resp = exchange_code(&client_id, args.client_secret.as_deref(), &redirect_uri, &code).await?;
    let userinfo = fetch_userinfo(&token_resp.access_token).await?;

    let now = time::OffsetDateTime::now_utc();
    let record = TokenRecord {
        access_token: token_resp.access_token,
        refresh_token: token_resp.refresh_token,
        expires_at: now + time::Duration::seconds(token_resp.expires_in as i64),
        refresh_expires_at: token_resp.refresh_token_expires_in
            .map(|s| now + time::Duration::seconds(s as i64)),
        sub: userinfo.sub,
        // Use the actual scopes granted by LinkedIn, not the requested set.
        // LinkedIn may grant a subset if certain scopes require partner approval.
        scopes: token_resp.scope
            .as_deref()
            .unwrap_or("")
            .split_whitespace()
            .map(|s| s.to_string())
            .collect(),
        client_id,
    };
    store.save(&args.account, &record)?;
    eprintln!("Authenticated as {} ({})", userinfo.name, record.sub);
    Ok(())
}

// NOTE: LinkedIn's standard 3-legged OAuth does not support PKCE.
// The generate_pkce() helper is retained for potential future use if LinkedIn
// adds PKCE support, but must NOT be wired into the current flow.
#[allow(dead_code)]
fn generate_pkce() -> (String, String) {
    let mut verifier_bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut verifier_bytes);
    let verifier = URL_SAFE_NO_PAD.encode(verifier_bytes);
    let challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));
    (verifier, challenge)
}

fn random_urlsafe(n: usize) -> String {
    let mut b = vec![0u8; n];
    rand::thread_rng().fill_bytes(&mut b);
    URL_SAFE_NO_PAD.encode(b)
}

fn build_authorize_url(client_id: &str, redirect_uri: &str, state: &str) -> Result<Url> {
    let mut url = Url::parse(AUTHORIZE_URL)?;
    url.query_pairs_mut()
        .append_pair("response_type", "code")
        .append_pair("client_id", client_id)
        .append_pair("redirect_uri", redirect_uri)
        .append_pair("scope", &SCOPES.join(" "))
        .append_pair("state", state);
    Ok(url)
}

#[derive(Debug, Deserialize)]
struct Callback {
    code: Option<String>,
    state: Option<String>,
    error: Option<String>,
    error_description: Option<String>,
}

async fn start_loopback() -> Result<(u16, tokio::task::JoinHandle<()>, oneshot::Receiver<Callback>)> {
    let (tx, rx) = oneshot::channel();
    let tx = Arc::new(tokio::sync::Mutex::new(Some(tx)));

    for port in REDIRECT_PORTS {
        let tx = tx.clone();
        let app = Router::new().route(
            "/callback",
            get(move |Query(cb): Query<Callback>| {
                let tx = tx.clone();
                async move {
                    let html = if cb.error.is_some() {
                        format!(
                            "<h1>Authorization failed: {}</h1><p>{}</p>",
                            cb.error.as_deref().unwrap_or("unknown"),
                            cb.error_description.as_deref().unwrap_or("")
                        )
                    } else {
                        "<h1>You may close this tab.</h1>".to_string()
                    };
                    if let Some(sender) = tx.lock().await.take() { let _ = sender.send(cb); }
                    Html(html)
                }
            }),
        );
        let addr = format!("127.0.0.1:{port}");
        if let Ok(listener) = tokio::net::TcpListener::bind(&addr).await {
            let handle = tokio::spawn(async move {
                let _ = axum::serve(listener, app).await;
            });
            return Ok((*port, handle, rx));
        }
    }
    anyhow::bail!("could not bind loopback port; tried {REDIRECT_PORTS:?}");
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    expires_in: u64,
    refresh_token: Option<String>,
    refresh_token_expires_in: Option<u64>,
    scope: Option<String>,
}

async fn exchange_code(client_id: &str, client_secret: Option<&str>, redirect_uri: &str, code: &str) -> Result<TokenResponse> {
    let mut params = vec![
        ("grant_type", "authorization_code"),
        ("code", code),
        ("redirect_uri", redirect_uri),
        ("client_id", client_id),
    ];
    if let Some(secret) = client_secret {
        params.push(("client_secret", secret));
    }
    let resp = reqwest::Client::new()
        .post(TOKEN_URL)
        .form(&params)
        .send().await?;
    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        anyhow::bail!("token exchange failed ({status}): {body}");
    }
    Ok(resp.json::<TokenResponse>().await?)
}

#[derive(Debug, Deserialize)]
struct UserInfo { sub: String, name: String, #[allow(dead_code)] email: Option<String> }

async fn fetch_userinfo(access_token: &str) -> Result<UserInfo> {
    Ok(reqwest::Client::new()
        .get(USERINFO_URL)
        .bearer_auth(access_token)
        .send().await?
        .error_for_status()?
        .json::<UserInfo>().await?)
}

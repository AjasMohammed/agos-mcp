//! LinkedIn OAuth: authorize-URL construction, code exchange, and refresh.
//! Mirrors the working flow in `linkedin-mcp` (state-based, `client_secret`,
//! no PKCE).

use crate::config::Config;
use crate::store::TokenEntry;
use anyhow::{Context, Result};
use serde::Deserialize;
use time::{Duration, OffsetDateTime};
use url::Url;

const AUTHORIZE_URL: &str = "https://www.linkedin.com/oauth/v2/authorization";
const TOKEN_URL: &str = "https://www.linkedin.com/oauth/v2/accessToken";
const USERINFO_URL: &str = "https://api.linkedin.com/v2/userinfo";
const SCOPES: &[&str] = &["openid", "profile", "email", "w_member_social"];

/// Whether a refresh failure is terminal (human must re-auth) or retryable.
#[derive(Debug)]
pub enum RefreshOutcome {
    Refreshed,
    ReauthRequired(String),
    Transient(String),
}

pub fn build_authorize_url(cfg: &Config, state: &str) -> Result<String> {
    let mut url = Url::parse(AUTHORIZE_URL)?;
    url.query_pairs_mut()
        .append_pair("response_type", "code")
        .append_pair("client_id", &cfg.client_id)
        .append_pair("redirect_uri", &cfg.redirect_uri())
        .append_pair("scope", &SCOPES.join(" "))
        .append_pair("state", state);
    Ok(url.to_string())
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    expires_in: i64,
    refresh_token: Option<String>,
    refresh_token_expires_in: Option<i64>,
    scope: Option<String>,
}

#[derive(Debug, Deserialize)]
struct UserInfo {
    sub: String,
}

/// Exchange an authorization code for tokens and resolve the member identity,
/// producing a ready-to-store `TokenEntry`.
pub async fn exchange_code(http: &reqwest::Client, cfg: &Config, code: &str) -> Result<TokenEntry> {
    let resp = http
        .post(TOKEN_URL)
        .form(&[
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", &cfg.redirect_uri()),
            ("client_id", &cfg.client_id),
            ("client_secret", &cfg.client_secret),
        ])
        .send()
        .await?;
    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        anyhow::bail!("token exchange failed ({status}): {body}");
    }
    let tr: TokenResponse = resp.json().await.context("parse token response")?;
    let sub = fetch_sub(http, &tr.access_token).await?;
    Ok(entry_from(cfg, tr, sub))
}

/// Refresh the access token in place. Returns a classified outcome instead of a
/// raw error so the caller can distinguish "needs re-auth" from a transient
/// failure (and avoid discarding a still-valid token).
pub async fn refresh(http: &reqwest::Client, cfg: &Config, entry: &mut TokenEntry) -> RefreshOutcome {
    let Some(rt) = entry.refresh_token.clone() else {
        return RefreshOutcome::ReauthRequired("no refresh token".into());
    };
    let resp = match http
        .post(TOKEN_URL)
        .form(&[
            ("grant_type", "refresh_token"),
            ("refresh_token", rt.as_str()),
            ("client_id", &cfg.client_id),
            ("client_secret", &cfg.client_secret),
        ])
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => return RefreshOutcome::Transient(e.to_string()),
    };

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        if body.contains("invalid_grant") {
            return RefreshOutcome::ReauthRequired(format!("refresh rejected ({status}): {body}"));
        }
        return RefreshOutcome::Transient(format!("token endpoint {status}: {body}"));
    }

    let tr: TokenResponse = match resp.json().await {
        Ok(t) => t,
        Err(e) => return RefreshOutcome::Transient(e.to_string()),
    };

    let now = OffsetDateTime::now_utc();
    entry.access_token = tr.access_token;
    entry.expires_at = now + Duration::seconds(tr.expires_in);
    if let Some(new_rt) = tr.refresh_token {
        // LinkedIn rotates refresh tokens — persist the new one or the next
        // refresh fails.
        entry.refresh_token = Some(new_rt);
        if let Some(ttl) = tr.refresh_token_expires_in {
            entry.refresh_expires_at = Some(now + Duration::seconds(ttl));
        }
    }
    RefreshOutcome::Refreshed
}

async fn fetch_sub(http: &reqwest::Client, access_token: &str) -> Result<String> {
    let info: UserInfo = http
        .get(USERINFO_URL)
        .bearer_auth(access_token)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    Ok(info.sub)
}

fn entry_from(cfg: &Config, tr: TokenResponse, sub: String) -> TokenEntry {
    let now = OffsetDateTime::now_utc();
    TokenEntry {
        expires_at: now + Duration::seconds(tr.expires_in),
        refresh_expires_at: tr
            .refresh_token_expires_in
            .map(|s| now + Duration::seconds(s)),
        scopes: tr
            .scope
            .as_deref()
            .unwrap_or("")
            .split_whitespace()
            .map(String::from)
            .collect(),
        access_token: tr.access_token,
        refresh_token: tr.refresh_token,
        sub,
        client_id: cfg.client_id.clone(),
    }
}

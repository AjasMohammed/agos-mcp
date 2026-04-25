use anyhow::Result;
use serde::Deserialize;

use super::token::TokenRecord;

pub async fn refresh(http: &reqwest::Client, record: &mut TokenRecord) -> Result<()> {
    let Some(rt) = record.refresh_token.clone() else {
        anyhow::bail!("no refresh token; re-run `linkedin-mcp auth`");
    };
    let resp: RefreshResp = http
        .post("https://www.linkedin.com/oauth/v2/accessToken")
        .form(&[
            ("grant_type", "refresh_token"),
            ("refresh_token", rt.as_str()),
            ("client_id", record.client_id.as_str()),
        ])
        .send().await?
        .error_for_status()?
        .json().await?;
    let now = time::OffsetDateTime::now_utc();
    record.access_token = resp.access_token;
    record.expires_at = now + time::Duration::seconds(resp.expires_in as i64);
    if let Some(new_rt) = resp.refresh_token {
        record.refresh_token = Some(new_rt);
        // Update expiry when LinkedIn issues a new refresh token (sliding window).
        // If LinkedIn doesn't return refresh_token_expires_in, keep the old expiry.
        if let Some(rt_ttl) = resp.refresh_token_expires_in {
            record.refresh_expires_at = Some(now + time::Duration::seconds(rt_ttl as i64));
        }
    }
    Ok(())
}

#[derive(Deserialize)]
struct RefreshResp {
    access_token: String,
    expires_in: u64,
    refresh_token: Option<String>,
    refresh_token_expires_in: Option<u64>,
}

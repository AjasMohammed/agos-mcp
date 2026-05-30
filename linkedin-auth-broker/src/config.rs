use anyhow::{Context, Result};

/// Broker configuration, loaded from the environment at startup.
#[derive(Clone)]
pub struct Config {
    /// LinkedIn app client id.
    pub client_id: String,
    /// LinkedIn app client secret (held ONLY here, never on MCP hosts).
    pub client_secret: String,
    /// Public base URL the broker is reachable at, e.g. `https://auth.example.com`.
    /// The OAuth redirect URI is `{public_url}/li/callback` and must be registered
    /// verbatim in the LinkedIn app.
    pub public_url: String,
    /// Shared bearer token required on the internal endpoints (`/li/start`,
    /// `/li/token`). MCP hosts and operators present this; LinkedIn does not.
    pub api_token: String,
    /// Address to bind, e.g. `0.0.0.0:8080`.
    pub bind_addr: String,
    /// Directory for the file-backed token store.
    pub store_dir: std::path::PathBuf,
    /// Backend for the token store: `file` (default, persistent) or `memory`
    /// (ephemeral; for dev/tests — tokens are lost on restart).
    pub store_kind: String,
    /// How often the background scheduler scans for tokens to refresh.
    pub refresh_scan_secs: u64,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let store_dir = match std::env::var("BROKER_STORE_DIR") {
            Ok(d) => std::path::PathBuf::from(d),
            Err(_) => dirs::data_dir()
                .context("no data dir; set BROKER_STORE_DIR")?
                .join("linkedin-auth-broker"),
        };
        Ok(Self {
            client_id: req("LINKEDIN_CLIENT_ID")?,
            client_secret: req("LINKEDIN_CLIENT_SECRET")?,
            public_url: req("BROKER_PUBLIC_URL")?
                .trim_end_matches('/')
                .to_string(),
            api_token: req("BROKER_API_TOKEN")?,
            bind_addr: std::env::var("BROKER_BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".into()),
            store_dir,
            store_kind: std::env::var("BROKER_STORE").unwrap_or_else(|_| "file".into()),
            refresh_scan_secs: std::env::var("BROKER_REFRESH_SCAN_SECS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(300),
        })
    }

    pub fn redirect_uri(&self) -> String {
        format!("{}/li/callback", self.public_url)
    }
}

fn req(key: &str) -> Result<String> {
    let v = std::env::var(key).map_err(|_| anyhow::anyhow!("missing required env var: {key}"))?;
    if v.is_empty() {
        anyhow::bail!("env var {key} is empty");
    }
    Ok(v)
}

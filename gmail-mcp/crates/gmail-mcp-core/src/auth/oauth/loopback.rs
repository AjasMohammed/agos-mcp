use crate::auth::errors::AuthError;
use crate::auth::token::{GoogleTokenResponse, TokenSet};
use reqwest::Client;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

pub struct LoopbackFlow {
    client_id: String,
    client_secret: Option<String>,
    scopes: Vec<String>,
}

impl LoopbackFlow {
    pub fn new(client_id: String, client_secret: Option<String>, scopes: Vec<String>) -> Self {
        Self {
            client_id,
            client_secret,
            scopes,
        }
    }

    pub async fn run(&self) -> Result<TokenSet, AuthError> {
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let port = listener.local_addr()?.port();
        let redirect = format!("http://127.0.0.1:{port}/");

        let verifier = generate_random_string(43);
        let challenge = generate_s256_challenge(&verifier);
        let state = generate_random_string(32);

        let mut url = url::Url::parse("https://accounts.google.com/o/oauth2/v2/auth")?;
        url.query_pairs_mut()
            .append_pair("client_id", &self.client_id)
            .append_pair("redirect_uri", &redirect)
            .append_pair("response_type", "code")
            .append_pair("scope", &self.scopes.join(" "))
            .append_pair("code_challenge", &challenge)
            .append_pair("code_challenge_method", "S256")
            .append_pair("state", &state)
            .append_pair("access_type", "offline")
            .append_pair("prompt", "consent");

        let auth_url = url.to_string();
        let _ = webbrowser::open(&auth_url);
        println!("If the browser didn't open, visit:\n  {auth_url}");

        let (code, returned_state) =
            tokio::time::timeout(Duration::from_secs(300), wait_for_callback(listener))
                .await
                .map_err(|_| AuthError::Timeout)??;

        if returned_state != state {
            return Err(AuthError::StateMismatch);
        }

        let mut params = vec![
            ("code", code.as_str()),
            ("client_id", &self.client_id),
            ("redirect_uri", &redirect),
            ("grant_type", "authorization_code"),
            ("code_verifier", &verifier),
        ];
        if let Some(ref secret) = self.client_secret {
            params.push(("client_secret", secret));
        }

        let resp: GoogleTokenResponse = Client::new()
            .post("https://oauth2.googleapis.com/token")
            .form(&params)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        Ok(TokenSet::from(resp))
    }
}

async fn wait_for_callback(listener: TcpListener) -> Result<(String, String), AuthError> {
    let (mut stream, _) = listener.accept().await?;
    let mut buf = [0u8; 4096];
    let n = stream.read(&mut buf).await?;
    let request = std::str::from_utf8(&buf[..n]).map_err(|_| AuthError::MalformedCallback)?;

    let path = request
        .lines()
        .next()
        .and_then(|l| l.split_whitespace().nth(1))
        .ok_or(AuthError::MalformedCallback)?;

    let url = url::Url::parse(&format!("http://127.0.0.1{path}"))?;
    let code = url
        .query_pairs()
        .find(|(k, _)| k == "code")
        .map(|(_, v)| v.to_string());
    let state = url
        .query_pairs()
        .find(|(k, _)| k == "state")
        .map(|(_, v)| v.to_string());

    let body = "Authentication complete. You can close this tab.";
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
        body.len(),
        body
    );
    let _ = stream.write_all(response.as_bytes()).await;
    let _ = stream.shutdown().await;

    match (code, state) {
        (Some(c), Some(s)) => Ok((c, s)),
        _ => Err(AuthError::MalformedCallback),
    }
}

fn generate_random_string(len: usize) -> String {
    use rand::RngCore;
    let mut b = vec![0u8; len];
    rand::thread_rng().fill_bytes(&mut b);
    base64::Engine::encode(&base64::engine::general_purpose::URL_SAFE_NO_PAD, &b)
}

fn generate_s256_challenge(verifier: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(verifier.as_bytes());
    let result = hasher.finalize();
    base64::Engine::encode(&base64::engine::general_purpose::URL_SAFE_NO_PAD, result)
}

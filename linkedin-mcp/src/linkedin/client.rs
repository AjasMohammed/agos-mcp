use anyhow::Result;
use reqwest::{header, Method, Response};
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::auth::{refresh, token::TokenRecord, storage::TokenStore};
use crate::error::LinkedInMcpError;
use super::{media, types};

const LINKEDIN_VERSION: &str = "202505";
const API_BASE: &str = "https://api.linkedin.com";

pub struct LinkedInClient {
    http: reqwest::Client,
    token: Arc<RwLock<TokenRecord>>,
    store: Arc<dyn TokenStore>,
    account: String,
}

impl LinkedInClient {
    pub fn new(http: reqwest::Client, token: TokenRecord, store: Arc<dyn TokenStore>, account: String) -> Self {
        Self { http, token: Arc::new(RwLock::new(token)), store, account }
    }

    async fn ensure_valid(&self) -> Result<(), LinkedInMcpError> {
        let needs_refresh = self.token.read().await.is_expiring_soon();
        if !needs_refresh { return Ok(()); }
        let mut rec = self.token.write().await;
        refresh::refresh(&self.http, &mut rec).await.map_err(|e| LinkedInMcpError::LinkedInServerError(e.to_string()))?;
        self.store.save(&self.account, &rec).map_err(|e| LinkedInMcpError::LinkedInServerError(e.to_string()))?;
        Ok(())
    }

    pub async fn author_urn(&self) -> String {
        self.token.read().await.author_urn()
    }

    pub async fn raw_request(&self, method: Method, path: &str) -> Result<reqwest::RequestBuilder, LinkedInMcpError> {
        self.ensure_valid().await?;
        let token = self.token.read().await.access_token.clone();
        let mut req = self.http.request(method, format!("{API_BASE}{path}"))
            .header(header::AUTHORIZATION, format!("Bearer {token}"))
            .header("LinkedIn-Version", LINKEDIN_VERSION);
        // X-Restli-Protocol-Version is only valid on legacy /v2/ Restli endpoints.
        // The new /rest/ API does not use Restli protocol and must not receive this header.
        if path.starts_with("/v2/") {
            req = req.header("X-Restli-Protocol-Version", "2.0.0");
        }
        Ok(req)
    }

    pub async fn userinfo(&self) -> Result<serde_json::Value, LinkedInMcpError> {
        let resp = self.raw_request(Method::GET, "/v2/userinfo").await?.send().await
            .map_err(|e| LinkedInMcpError::LinkedInServerError(e.to_string()))?;
        handle(resp).await
    }

    pub async fn create_post<B: Serialize>(&self, body: &B) -> Result<String, LinkedInMcpError> {
        let resp = self.raw_request(Method::POST, "/rest/posts").await?
            .json(body).send().await
            .map_err(|e| LinkedInMcpError::LinkedInServerError(e.to_string()))?;
        let status = resp.status();
        if !status.is_success() {
            return Err(map_status(resp).await);
        }
        if let Some(loc) = resp.headers().get("x-restli-id").or(resp.headers().get(header::LOCATION)) {
            return Ok(loc.to_str().unwrap_or_default().to_string());
        }
        let v: serde_json::Value = resp.json().await.unwrap_or(serde_json::Value::Null);
        if let Some(id) = v.get("id").and_then(|x| x.as_str()) {
            return Ok(id.to_string());
        }
        Err(LinkedInMcpError::LinkedInServerError("post succeeded but no URN in response".into()))
    }

    pub async fn delete_post(&self, urn: &str) -> Result<(), LinkedInMcpError> {
        let encoded = urlencoding::encode(urn);
        let resp = self.raw_request(Method::DELETE, &format!("/rest/posts/{encoded}")).await?
            .send().await
            .map_err(|e| LinkedInMcpError::LinkedInServerError(e.to_string()))?;
        if resp.status().is_success() { Ok(()) } else { Err(map_status(resp).await) }
    }

    pub async fn get_post(&self, urn: &str) -> Result<serde_json::Value, LinkedInMcpError> {
        let encoded = urlencoding::encode(urn);
        let resp = self.raw_request(Method::GET, &format!("/rest/posts/{encoded}")).await?
            .send().await
            .map_err(|e| LinkedInMcpError::LinkedInServerError(e.to_string()))?;
        handle(resp).await
    }

    pub async fn update_post(
        &self,
        urn: &str,
        commentary: Option<&str>,
        visibility: Option<&str>,
    ) -> Result<(), LinkedInMcpError> {
        let encoded = urlencoding::encode(urn);
        let mut set = serde_json::Map::new();
        if let Some(c) = commentary {
            set.insert("commentary".into(), serde_json::Value::String(c.to_string()));
        }
        if let Some(v) = visibility {
            set.insert("visibility".into(), serde_json::Value::String(v.to_string()));
        }
        let body = serde_json::json!({ "patch": { "$set": set } });
        // LinkedIn REST API uses POST + X-RestLi-Method header for partial updates, not HTTP PATCH
        let resp = self.raw_request(Method::POST, &format!("/rest/posts/{encoded}")).await?
            .header("X-RestLi-Method", "PARTIAL_UPDATE")
            .header(header::CONTENT_TYPE, "application/json")
            .json(&body)
            .send().await
            .map_err(|e| LinkedInMcpError::LinkedInServerError(e.to_string()))?;
        if resp.status().is_success() { Ok(()) } else { Err(map_status(resp).await) }
    }

    pub async fn list_posts(&self, start: u32, count: u32) -> Result<serde_json::Value, LinkedInMcpError> {
        let author = self.author_urn().await;
        let encoded_author = urlencoding::encode(&author);
        let resp = self.raw_request(Method::GET, &format!(
            "/rest/posts?q=author&author={encoded_author}&start={start}&count={count}"
        )).await?
            .header("X-RestLi-Method", "FINDER")
            .send().await
            .map_err(|e| LinkedInMcpError::LinkedInServerError(e.to_string()))?;
        handle(resp).await
    }

    pub async fn list_comments(
        &self,
        post_urn: &str,
        start: u32,
        count: u32,
    ) -> Result<serde_json::Value, LinkedInMcpError> {
        let encoded = urlencoding::encode(post_urn);
        let resp = self.raw_request(Method::GET, &format!(
            "/rest/socialActions/{encoded}/comments?start={start}&count={count}"
        )).await?.send().await
            .map_err(|e| LinkedInMcpError::LinkedInServerError(e.to_string()))?;
        handle(resp).await
    }

    pub async fn create_comment(&self, post_urn: &str, text: &str) -> Result<String, LinkedInMcpError> {
        let encoded = urlencoding::encode(post_urn);
        let actor = self.author_urn().await;
        let body = types::CommentBody {
            actor: &actor,
            message: types::CommentMessage { text },
        };
        let resp = self.raw_request(Method::POST, &format!("/rest/socialActions/{encoded}/comments")).await?
            .json(&body)
            .send().await
            .map_err(|e| LinkedInMcpError::LinkedInServerError(e.to_string()))?;
        if !resp.status().is_success() {
            return Err(map_status(resp).await);
        }
        let urn = resp.headers()
            .get("x-restli-id")
            .or_else(|| resp.headers().get(header::LOCATION))
            .and_then(|v| v.to_str().ok())
            .unwrap_or_default()
            .to_string();
        Ok(urn)
    }

    pub async fn delete_comment(&self, post_urn: &str, comment_urn: &str) -> Result<(), LinkedInMcpError> {
        let encoded_post = urlencoding::encode(post_urn);
        let encoded_comment = urlencoding::encode(comment_urn);
        let resp = self.raw_request(Method::DELETE, &format!(
            "/rest/socialActions/{encoded_post}/comments/{encoded_comment}"
        )).await?.send().await
            .map_err(|e| LinkedInMcpError::LinkedInServerError(e.to_string()))?;
        if resp.status().is_success() { Ok(()) } else { Err(map_status(resp).await) }
    }

    pub async fn add_reaction(&self, post_urn: &str, reaction_type: &str) -> Result<(), LinkedInMcpError> {
        let actor = self.author_urn().await;
        let encoded_actor = urlencoding::encode(&actor);
        // actor goes as query param; body uses `root` (not `object`) per LinkedIn Reactions API
        let body = types::ReactionBody { root: post_urn, reaction_type };
        let resp = self.raw_request(Method::POST, &format!("/rest/reactions?actor={encoded_actor}")).await?
            .json(&body)
            .send().await
            .map_err(|e| LinkedInMcpError::LinkedInServerError(e.to_string()))?;
        if resp.status().is_success() { Ok(()) } else { Err(map_status(resp).await) }
    }

    pub async fn remove_reaction(&self, post_urn: &str) -> Result<(), LinkedInMcpError> {
        let actor = self.author_urn().await;
        // Composite key format: (actor:{encoded_urn},entity:{encoded_urn}) — no reactionType
        let encoded_actor = urlencoding::encode(&actor);
        let encoded_entity = urlencoding::encode(post_urn);
        let key = format!("(actor:{encoded_actor},entity:{encoded_entity})");
        let resp = self.raw_request(Method::DELETE, &format!("/rest/reactions/{key}")).await?
            .send().await
            .map_err(|e| LinkedInMcpError::LinkedInServerError(e.to_string()))?;
        if resp.status().is_success() { Ok(()) } else { Err(map_status(resp).await) }
    }

    pub async fn upload_image(&self, path: &std::path::Path) -> Result<String, LinkedInMcpError> {
        let owner = self.author_urn().await;
        let upload = media::init_image_upload(self, &owner).await?;
        let token = self.token.read().await.access_token.clone();
        media::upload_image_bytes(&self.http, &token, &upload.upload_url, path).await?;
        media::wait_for_image_ready(self, &upload.image_urn).await?;
        Ok(upload.image_urn)
    }

    pub async fn upload_document(&self, path: &std::path::Path) -> Result<String, LinkedInMcpError> {
        let owner = self.author_urn().await;
        let upload = media::init_document_upload(self, &owner).await?;
        let token = self.token.read().await.access_token.clone();
        media::upload_document_bytes(&self.http, &token, &upload.upload_url, path).await?;
        media::wait_for_document_ready(self, &upload.document_urn).await?;
        Ok(upload.document_urn)
    }

    pub async fn upload_video(&self, path: &std::path::Path) -> Result<String, LinkedInMcpError> {
        let owner = self.author_urn().await;
        let metadata = tokio::fs::metadata(path).await
            .map_err(|e| LinkedInMcpError::InvalidInput(e.to_string()))?;
        let file_size = metadata.len();
        let upload = media::init_video_upload(self, &owner, file_size).await?;
        let token = self.token.read().await.access_token.clone();
        let etags = media::upload_video_chunks(&self.http, &token, path, &upload.instructions).await?;
        media::finalize_video_upload(self, &upload.video_urn, &upload.upload_token, &etags).await?;
        media::wait_for_video_ready(self, &upload.video_urn).await?;
        Ok(upload.video_urn)
    }
}

async fn handle(resp: Response) -> Result<serde_json::Value, LinkedInMcpError> {
    if resp.status().is_success() {
        resp.json().await.map_err(|e| LinkedInMcpError::LinkedInServerError(e.to_string()))
    } else {
        Err(map_status(resp).await)
    }
}

pub(crate) async fn map_status(resp: Response) -> LinkedInMcpError {
    let status = resp.status();
    let retry_after = resp.headers().get("retry-after")
        .and_then(|h| h.to_str().ok()).and_then(|s| s.parse::<u64>().ok()).unwrap_or(60);
    let body = resp.text().await.unwrap_or_default();
    match status.as_u16() {
        401 => LinkedInMcpError::AuthRequired,
        403 => LinkedInMcpError::ScopeMissing(body),
        404 => LinkedInMcpError::UnknownUrn(body),
        429 => LinkedInMcpError::RateLimited(retry_after),
        422 => LinkedInMcpError::InvalidInput(body),
        500..=599 => LinkedInMcpError::LinkedInServerError(body),
        _ => LinkedInMcpError::LinkedInServerError(format!("{status}: {body}")),
    }
}

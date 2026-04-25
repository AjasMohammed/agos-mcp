use anyhow::Result;
use reqwest::header;
use reqwest::Method;
use serde_json::json;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

use crate::error::LinkedInMcpError;
use super::LinkedInClient;

// ── Image upload ──────────────────────────────────────────────────────────────

pub struct ImageUpload {
    pub upload_url: String,
    pub image_urn: String,
}

pub async fn init_image_upload(
    client: &LinkedInClient,
    owner_urn: &str,
) -> Result<ImageUpload, LinkedInMcpError> {
    let body = json!({ "initializeUploadRequest": { "owner": owner_urn } });
    let resp = client
        .raw_request(Method::POST, "/rest/images?action=initializeUpload")
        .await?
        .json(&body)
        .send()
        .await
        .map_err(|e| LinkedInMcpError::LinkedInServerError(e.to_string()))?;
    if !resp.status().is_success() {
        return Err(crate::linkedin::client::map_status(resp).await);
    }
    let v: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| LinkedInMcpError::LinkedInServerError(e.to_string()))?;
    let upload_url = v["value"]["uploadUrl"]
        .as_str()
        .ok_or_else(|| LinkedInMcpError::LinkedInServerError("missing uploadUrl".into()))?
        .to_string();
    let image_urn = v["value"]["image"]
        .as_str()
        .ok_or_else(|| LinkedInMcpError::LinkedInServerError("missing image URN".into()))?
        .to_string();
    Ok(ImageUpload { upload_url, image_urn })
}

pub async fn upload_image_bytes(
    http: &reqwest::Client,
    access_token: &str,
    upload_url: &str,
    path: &std::path::Path,
) -> Result<(), LinkedInMcpError> {
    let bytes = tokio::fs::read(path)
        .await
        .map_err(|e| LinkedInMcpError::InvalidInput(e.to_string()))?;
    let content_type = mime_guess::from_path(path)
        .first_or_octet_stream()
        .to_string();
    let resp = http
        .put(upload_url)
        .bearer_auth(access_token)
        .header(header::CONTENT_TYPE, &content_type)
        .body(bytes)
        .send()
        .await
        .map_err(|e| LinkedInMcpError::LinkedInServerError(e.to_string()))?;
    if resp.status().is_success() {
        Ok(())
    } else {
        Err(LinkedInMcpError::LinkedInServerError(format!(
            "image upload failed: {}",
            resp.status()
        )))
    }
}

pub async fn wait_for_image_ready(
    client: &LinkedInClient,
    image_urn: &str,
) -> Result<(), LinkedInMcpError> {
    let encoded = urlencoding::encode(image_urn);
    for attempt in 0u64..20 {
        let resp = client
            .raw_request(Method::GET, &format!("/rest/images/{encoded}"))
            .await?
            .send()
            .await
            .map_err(|e| LinkedInMcpError::LinkedInServerError(e.to_string()))?;
        let v: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| LinkedInMcpError::LinkedInServerError(e.to_string()))?;
        match v["status"].as_str() {
            Some("AVAILABLE") => return Ok(()),
            Some("PROCESSING_FAILED") => {
                return Err(LinkedInMcpError::LinkedInServerError(
                    "image processing failed".into(),
                ))
            }
            _ => tokio::time::sleep(std::time::Duration::from_secs(2 + attempt)).await,
        }
    }
    Err(LinkedInMcpError::LinkedInServerError(
        "image never became available".into(),
    ))
}

// ── Video upload ──────────────────────────────────────────────────────────────

pub struct VideoUpload {
    pub video_urn: String,
    pub upload_token: String,
    pub instructions: Vec<UploadInstruction>,
}

pub struct UploadInstruction {
    pub upload_url: String,
    pub first_byte: u64,
    pub last_byte: u64,
}

pub async fn init_video_upload(
    client: &LinkedInClient,
    owner_urn: &str,
    file_size_bytes: u64,
) -> Result<VideoUpload, LinkedInMcpError> {
    let body = json!({
        "initializeUploadRequest": {
            "owner": owner_urn,
            "fileSizeBytes": file_size_bytes,
            "uploadCaptionFile": false
        }
    });
    let resp = client
        .raw_request(Method::POST, "/rest/videos?action=initializeUpload")
        .await?
        .json(&body)
        .send()
        .await
        .map_err(|e| LinkedInMcpError::LinkedInServerError(e.to_string()))?;
    if !resp.status().is_success() {
        return Err(crate::linkedin::client::map_status(resp).await);
    }
    let v: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| LinkedInMcpError::LinkedInServerError(e.to_string()))?;
    let val = &v["value"];
    let video_urn = val["video"]
        .as_str()
        .ok_or_else(|| LinkedInMcpError::LinkedInServerError("missing video URN".into()))?
        .to_string();
    let upload_token = val["uploadToken"].as_str().unwrap_or_default().to_string();
    let instructions = val["uploadInstructions"]
        .as_array()
        .ok_or_else(|| {
            LinkedInMcpError::LinkedInServerError("missing uploadInstructions".into())
        })?
        .iter()
        .map(|inst| UploadInstruction {
            upload_url: inst["uploadUrl"].as_str().unwrap_or_default().to_string(),
            first_byte: inst["firstByteOffset"].as_u64().unwrap_or(0),
            last_byte: inst["lastByteOffset"].as_u64().unwrap_or(0),
        })
        .collect();
    Ok(VideoUpload { video_urn, upload_token, instructions })
}

pub async fn upload_video_chunks(
    http: &reqwest::Client,
    access_token: &str,
    path: &std::path::Path,
    instructions: &[UploadInstruction],
) -> Result<Vec<String>, LinkedInMcpError> {
    let mut file = File::open(path)
        .await
        .map_err(|e| LinkedInMcpError::InvalidInput(e.to_string()))?;
    let mut etags = Vec::with_capacity(instructions.len());
    for inst in instructions {
        // LinkedIn tells us the exact byte range each URL covers.
        let chunk_len = (inst.last_byte - inst.first_byte + 1) as usize;
        let mut buf = vec![0u8; chunk_len];
        let n = file
            .read(&mut buf)
            .await
            .map_err(|e| LinkedInMcpError::InvalidInput(e.to_string()))?;
        buf.truncate(n);
        let resp = http
            .put(&inst.upload_url)
            .bearer_auth(access_token)
            .body(buf)
            .send()
            .await
            .map_err(|e| LinkedInMcpError::LinkedInServerError(e.to_string()))?;
        if !resp.status().is_success() {
            return Err(LinkedInMcpError::LinkedInServerError(format!(
                "chunk upload failed: {}",
                resp.status()
            )));
        }
        let etag = resp
            .headers()
            .get("etag")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();
        etags.push(etag);
    }
    Ok(etags)
}

pub async fn finalize_video_upload(
    client: &LinkedInClient,
    video_urn: &str,
    upload_token: &str,
    etags: &[String],
) -> Result<(), LinkedInMcpError> {
    let body = json!({
        "finalizeUploadRequest": {
            "video": video_urn,
            "uploadToken": upload_token,
            "uploadedPartIds": etags
        }
    });
    let resp = client
        .raw_request(Method::POST, "/rest/videos?action=finalizeUpload")
        .await?
        .json(&body)
        .send()
        .await
        .map_err(|e| LinkedInMcpError::LinkedInServerError(e.to_string()))?;
    if resp.status().is_success() {
        Ok(())
    } else {
        Err(crate::linkedin::client::map_status(resp).await)
    }
}

pub async fn wait_for_video_ready(
    client: &LinkedInClient,
    video_urn: &str,
) -> Result<(), LinkedInMcpError> {
    let encoded = urlencoding::encode(video_urn);
    for attempt in 0u64..30 {
        let resp = client
            .raw_request(Method::GET, &format!("/rest/videos/{encoded}"))
            .await?
            .send()
            .await
            .map_err(|e| LinkedInMcpError::LinkedInServerError(e.to_string()))?;
        let v: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| LinkedInMcpError::LinkedInServerError(e.to_string()))?;
        match v["status"].as_str() {
            Some("AVAILABLE") => return Ok(()),
            Some("FAILED") => {
                return Err(LinkedInMcpError::LinkedInServerError(
                    "video processing failed".into(),
                ))
            }
            _ => tokio::time::sleep(std::time::Duration::from_secs(2 + attempt)).await,
        }
    }
    Err(LinkedInMcpError::LinkedInServerError(
        "video never became available".into(),
    ))
}

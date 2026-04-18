use crate::gmail::Client;
use crate::gmail::errors::{GmailError, map_gmail_error};
use crate::gmail::types::MessageRef;

impl Client {
    pub async fn send_large(
        &self,
        raw: &[u8],
        thread_id: Option<&str>,
    ) -> Result<MessageRef, GmailError> {
        let token = self.tokens.access_token().await?;
        let init_url = "https://gmail.googleapis.com/upload/gmail/v1/users/me/messages/send?uploadType=resumable";

        let mut req = self
            .http
            .post(init_url)
            .bearer_auth(&token)
            .header("X-Upload-Content-Type", "message/rfc822")
            .header("X-Upload-Content-Length", raw.len().to_string());

        // Gmail requires a JSON body even when using raw mode to pass threadId.
        if let Some(t_id) = thread_id {
            req = req.json(&serde_json::json!({"threadId": t_id}));
        } else {
            req = req.json(&serde_json::json!({}));
        }

        let init = req.send().await?;
        if !init.status().is_success() {
            return Err(map_gmail_error(init.status(), init).await);
        }

        let session = init
            .headers()
            .get("Location")
            .ok_or_else(|| GmailError::Transport("no upload session".into()))?
            .to_str()
            .map_err(|_| GmailError::Transport("invalid location header".into()))?
            .to_string();

        let chunk_size = 5 * 1024 * 1024;
        let mut start = 0;
        let total = raw.len();

        while start < total {
            let end = std::cmp::min(start + chunk_size, total);
            let chunk = &raw[start..end];
            let content_range = format!("bytes {}-{}/{}", start, end - 1, total);

            let mut retries = 0;
            loop {
                let upload = self
                    .http
                    .put(&session)
                    .header("Content-Length", chunk.len().to_string())
                    .header("Content-Range", &content_range)
                    .body(chunk.to_vec())
                    .send()
                    .await?;

                let status = upload.status();
                if status.is_success() || status.as_u16() == 308 {
                    // Success or incomplete (expected).
                    if status.is_success() {
                        return Ok(upload.json().await?);
                    }
                    break; // Move to next chunk
                }

                if retries >= 1 {
                    return Err(map_gmail_error(status, upload).await);
                }
                retries += 1;
            }
            start = end;
        }

        Err(GmailError::Transport(
            "Upload completed but no final success response".into(),
        ))
    }
}

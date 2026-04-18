use crate::gmail::Client;
use crate::mcp::{McpError, Tool};
use async_trait::async_trait;
use serde::Deserialize;
use std::sync::Arc;

pub struct GmailDownloadAttachmentTool {
    pub client: Arc<Client>,
}

#[async_trait]
impl Tool for GmailDownloadAttachmentTool {
    fn name(&self) -> &str {
        "gmail_download_attachment"
    }
    fn description(&self) -> &str {
        "Download an attachment by message ID and attachment ID."
    }
    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "message_id": { "type": "string" },
                "attachment_id": { "type": "string" }
            },
            "required": ["message_id", "attachment_id"],
            "additionalProperties": false
        })
    }
    async fn call(&self, args: serde_json::Value) -> Result<serde_json::Value, McpError> {
        #[derive(Deserialize)]
        struct Args {
            message_id: String,
            attachment_id: String,
        }
        let a: Args =
            serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;
        let resp = self
            .client
            .attachment_get(&a.message_id, &a.attachment_id)
            .await
            .map_err(|e| McpError::ToolError(e.into()))?;
        Ok(serde_json::to_value(resp)?)
    }
}

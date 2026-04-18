use crate::gmail::{Client, MessageFormat};
use crate::mcp::{McpError, Tool};
use async_trait::async_trait;
use serde::Deserialize;
use std::sync::Arc;

pub struct GmailReadTool {
    pub client: Arc<Client>,
}

#[async_trait]
impl Tool for GmailReadTool {
    fn name(&self) -> &str {
        "gmail_read"
    }
    fn description(&self) -> &str {
        "Read a specific message by ID."
    }
    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "id": { "type": "string" },
                "format": { "type": "string", "enum": ["full", "metadata", "minimal", "raw"], "default": "full" }
            },
            "required": ["id"],
            "additionalProperties": false
        })
    }
    async fn call(&self, args: serde_json::Value) -> Result<serde_json::Value, McpError> {
        #[derive(Deserialize)]
        struct Args {
            id: String,
            format: Option<String>,
        }
        let a: Args =
            serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;

        let fmt = match a.format.as_deref() {
            Some("metadata") => MessageFormat::Metadata,
            Some("minimal") => MessageFormat::Minimal,
            Some("raw") => MessageFormat::Raw,
            _ => MessageFormat::Full,
        };

        let resp = self
            .client
            .messages_get(&a.id, fmt)
            .await
            .map_err(|e| McpError::ToolError(e.into()))?;
        Ok(serde_json::to_value(resp)?)
    }
}

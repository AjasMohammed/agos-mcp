use crate::gmail::Client;
use crate::mcp::{McpError, Tool};
use async_trait::async_trait;
use serde::Deserialize;
use std::sync::Arc;

pub struct GmailGetThreadTool {
    pub client: Arc<Client>,
}

#[async_trait]
impl Tool for GmailGetThreadTool {
    fn name(&self) -> &str {
        "gmail_get_thread"
    }
    fn description(&self) -> &str {
        "Get a thread by ID."
    }
    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "id": { "type": "string" }
            },
            "required": ["id"],
            "additionalProperties": false
        })
    }
    async fn call(&self, args: serde_json::Value) -> Result<serde_json::Value, McpError> {
        #[derive(Deserialize)]
        struct Args {
            id: String,
        }
        let a: Args =
            serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;
        let resp = self
            .client
            .threads_get(&a.id)
            .await
            .map_err(|e| McpError::ToolError(e.into()))?;
        Ok(serde_json::to_value(resp)?)
    }
}

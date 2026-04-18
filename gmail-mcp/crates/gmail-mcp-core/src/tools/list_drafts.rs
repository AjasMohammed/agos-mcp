use crate::gmail::Client;
use crate::mcp::{McpError, Tool};
use async_trait::async_trait;
use std::sync::Arc;

pub struct GmailListDraftsTool {
    pub client: Arc<Client>,
}

#[async_trait]
impl Tool for GmailListDraftsTool {
    fn name(&self) -> &str {
        "gmail_list_drafts"
    }
    fn description(&self) -> &str {
        "List all drafts in the authenticated mailbox."
    }
    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {},
            "additionalProperties": false
        })
    }
    async fn call(&self, _args: serde_json::Value) -> Result<serde_json::Value, McpError> {
        let resp = self
            .client
            .drafts_list()
            .await
            .map_err(|e| McpError::ToolError(e.into()))?;
        Ok(serde_json::to_value(resp)?)
    }
}

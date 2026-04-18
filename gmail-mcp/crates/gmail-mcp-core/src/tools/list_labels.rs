use crate::gmail::Client;
use crate::mcp::{McpError, Tool};
use async_trait::async_trait;
use std::sync::Arc;

pub struct GmailListLabelsTool {
    pub client: Arc<Client>,
}

#[async_trait]
impl Tool for GmailListLabelsTool {
    fn name(&self) -> &str {
        "gmail_list_labels"
    }
    fn description(&self) -> &str {
        "List all labels in the authenticated mailbox."
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
            .labels_list()
            .await
            .map_err(|e| McpError::ToolError(e.into()))?;
        Ok(serde_json::to_value(resp)?)
    }
}

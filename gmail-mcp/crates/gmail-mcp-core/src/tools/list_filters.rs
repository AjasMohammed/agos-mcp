use crate::gmail::Client;
use crate::mcp::{McpError, Tool};
use async_trait::async_trait;
use std::sync::Arc;

pub struct GmailListFiltersTool {
    pub client: Arc<Client>,
}

#[async_trait]
impl Tool for GmailListFiltersTool {
    fn name(&self) -> &str {
        "gmail_list_filters"
    }
    fn description(&self) -> &str {
        "List all filters in the authenticated mailbox."
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
            .filters_list()
            .await
            .map_err(|e| McpError::ToolError(e.into()))?;
        Ok(serde_json::to_value(resp)?)
    }
}

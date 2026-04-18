use crate::gmail::Client;
use crate::mcp::{McpError, Tool};
use async_trait::async_trait;
use std::sync::Arc;

pub struct GmailGetProfileTool {
    pub client: Arc<Client>,
}

#[async_trait]
impl Tool for GmailGetProfileTool {
    fn name(&self) -> &str {
        "gmail_get_profile"
    }
    fn description(&self) -> &str {
        "Get the profile (email address, total messages, etc.) of the authenticated user."
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
            .profile_get()
            .await
            .map_err(|e| McpError::ToolError(e.into()))?;
        Ok(serde_json::to_value(resp)?)
    }
}

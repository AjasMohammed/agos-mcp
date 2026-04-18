use crate::mcp::{McpError, Tool};
use async_trait::async_trait;
use std::sync::Arc;

use super::compose::{ComposeArgs, build_raw, compose_schema};
use crate::gmail::Client;

pub struct GmailSendTool {
    pub client: Arc<Client>,
}

#[async_trait]
impl Tool for GmailSendTool {
    fn name(&self) -> &str {
        "gmail_send"
    }
    fn description(&self) -> &str {
        "Compose and send an email on behalf of the authenticated user. \
         Accepts plain-text and/or HTML body. Supports attachments and reply threading. \
         Requires gmail.send or https://mail.google.com/ scope."
    }
    fn input_schema(&self) -> serde_json::Value {
        compose_schema()
    }
    async fn call(&self, args: serde_json::Value) -> Result<serde_json::Value, McpError> {
        let a: ComposeArgs =
            serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;
        let (raw, thread_id) = build_raw(a, &self.client).await?;
        let sent = self
            .client
            .messages_send(&raw, thread_id.as_deref())
            .await
            .map_err(|e| McpError::ToolError(e.into()))?;
        Ok(serde_json::to_value(sent)?)
    }
}

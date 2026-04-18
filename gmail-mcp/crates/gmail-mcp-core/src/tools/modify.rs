use crate::gmail::Client;
use crate::mcp::{McpError, Tool};
use async_trait::async_trait;
use serde::Deserialize;
use std::sync::Arc;

pub struct GmailModifyLabelsTool {
    pub client: Arc<Client>,
}
#[async_trait]
impl Tool for GmailModifyLabelsTool {
    fn name(&self) -> &str {
        "gmail_modify_labels"
    }
    fn description(&self) -> &str {
        "Modify labels on a message."
    }
    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "id": { "type": "string" },
                "add_label_ids": { "type": "array", "items": { "type": "string" } },
                "remove_label_ids": { "type": "array", "items": { "type": "string" } }
            },
            "required": ["id"]
        })
    }
    async fn call(&self, args: serde_json::Value) -> Result<serde_json::Value, McpError> {
        #[derive(Deserialize)]
        struct Args {
            id: String,
            add_label_ids: Option<Vec<String>>,
            remove_label_ids: Option<Vec<String>>,
        }
        let a: Args =
            serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;
        let res = self
            .client
            .messages_modify(
                &a.id,
                &a.add_label_ids.unwrap_or_default(),
                &a.remove_label_ids.unwrap_or_default(),
            )
            .await
            .map_err(|e| McpError::ToolError(e.into()))?;
        Ok(serde_json::to_value(res)?)
    }
}

pub struct GmailTrashTool {
    pub client: Arc<Client>,
}
#[async_trait]
impl Tool for GmailTrashTool {
    fn name(&self) -> &str {
        "gmail_trash"
    }
    fn description(&self) -> &str {
        "Trash a message."
    }
    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({ "type": "object", "properties": { "id": { "type": "string" } }, "required": ["id"] })
    }
    async fn call(&self, args: serde_json::Value) -> Result<serde_json::Value, McpError> {
        #[derive(Deserialize)]
        struct Args {
            id: String,
        }
        let a: Args =
            serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;
        let res = self
            .client
            .messages_trash(&a.id)
            .await
            .map_err(|e| McpError::ToolError(e.into()))?;
        Ok(serde_json::to_value(res)?)
    }
}

pub struct GmailUntrashTool {
    pub client: Arc<Client>,
}
#[async_trait]
impl Tool for GmailUntrashTool {
    fn name(&self) -> &str {
        "gmail_untrash"
    }
    fn description(&self) -> &str {
        "Untrash a message."
    }
    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({ "type": "object", "properties": { "id": { "type": "string" } }, "required": ["id"] })
    }
    async fn call(&self, args: serde_json::Value) -> Result<serde_json::Value, McpError> {
        #[derive(Deserialize)]
        struct Args {
            id: String,
        }
        let a: Args =
            serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;
        let res = self
            .client
            .messages_untrash(&a.id)
            .await
            .map_err(|e| McpError::ToolError(e.into()))?;
        Ok(serde_json::to_value(res)?)
    }
}

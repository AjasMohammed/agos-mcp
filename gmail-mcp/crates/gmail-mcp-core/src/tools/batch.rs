use crate::gmail::Client;
use crate::mcp::{McpError, Tool};
use async_trait::async_trait;
use futures::stream::StreamExt;
use serde::Deserialize;
use std::sync::Arc;

pub struct GmailBatchModifyLabelsTool {
    pub client: Arc<Client>,
}
#[async_trait]
impl Tool for GmailBatchModifyLabelsTool {
    fn name(&self) -> &str {
        "gmail_batch_modify_labels"
    }
    fn description(&self) -> &str {
        "Modify labels on up to 500 messages in one call."
    }
    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "message_ids":      { "type": "array", "items": { "type": "string" }, "maxItems": 500 },
                "add_label_ids":    { "type": "array", "items": { "type": "string" } },
                "remove_label_ids": { "type": "array", "items": { "type": "string" } }
            },
            "required": ["message_ids"],
            "additionalProperties": false
        })
    }
    async fn call(&self, args: serde_json::Value) -> Result<serde_json::Value, McpError> {
        #[derive(Deserialize)]
        struct Args {
            message_ids: Vec<String>,
            add_label_ids: Option<Vec<String>>,
            remove_label_ids: Option<Vec<String>>,
        }
        let a: Args =
            serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;

        let body = serde_json::json!({
            "ids": a.message_ids,
            "addLabelIds": a.add_label_ids.unwrap_or_default(),
            "removeLabelIds": a.remove_label_ids.unwrap_or_default()
        });

        self.client
            .messages_batch_modify(&body)
            .await
            .map_err(|e| McpError::ToolError(e.into()))?;
        Ok(serde_json::json!({ "success": true }))
    }
}

pub struct GmailBatchTrashTool {
    pub client: Arc<Client>,
}
#[async_trait]
impl Tool for GmailBatchTrashTool {
    fn name(&self) -> &str {
        "gmail_batch_trash"
    }
    fn description(&self) -> &str {
        "Trash multiple messages concurrently."
    }
    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "message_ids": { "type": "array", "items": { "type": "string" }, "maxItems": 100 }
            },
            "required": ["message_ids"],
            "additionalProperties": false
        })
    }
    async fn call(&self, args: serde_json::Value) -> Result<serde_json::Value, McpError> {
        #[derive(Deserialize)]
        struct Args {
            message_ids: Vec<String>,
        }
        let a: Args =
            serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;

        let mut stream = futures::stream::iter(a.message_ids.into_iter().map(|id| {
            let client = self.client.clone();
            async move {
                let res = client.messages_trash(&id).await;
                serde_json::json!({
                    "id": id,
                    "success": res.is_ok(),
                    "error": res.err().map(|e| e.to_string())
                })
            }
        }))
        .buffer_unordered(10);

        let mut out = Vec::new();
        while let Some(r) = stream.next().await {
            out.push(r);
        }
        Ok(serde_json::to_value(out)?)
    }
}

pub struct GmailBatchDeleteTool {
    pub client: Arc<Client>,
}
#[async_trait]
impl Tool for GmailBatchDeleteTool {
    fn name(&self) -> &str {
        "gmail_batch_delete"
    }
    fn description(&self) -> &str {
        "Permanently delete up to 500 messages. Requires confirm=true."
    }
    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "message_ids": { "type": "array", "items": { "type": "string" }, "maxItems": 500 },
                "confirm": { "type": "boolean", "description": "Must be true to execute permanent deletion" }
            },
            "required": ["message_ids", "confirm"],
            "additionalProperties": false
        })
    }
    async fn call(&self, args: serde_json::Value) -> Result<serde_json::Value, McpError> {
        #[derive(Deserialize)]
        struct Args {
            message_ids: Vec<String>,
            confirm: bool,
        }
        let a: Args =
            serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;
        if !a.confirm {
            return Err(McpError::InvalidParams(
                "confirm must be true to permanently delete messages".into(),
            ));
        }

        self.client
            .messages_batch_delete(&a.message_ids)
            .await
            .map_err(|e| McpError::ToolError(e.into()))?;
        Ok(serde_json::json!({ "success": true, "deleted_count": a.message_ids.len() }))
    }
}

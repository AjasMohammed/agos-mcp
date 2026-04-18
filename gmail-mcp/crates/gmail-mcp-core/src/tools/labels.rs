use crate::gmail::Client;
use crate::mcp::{McpError, Tool};
use async_trait::async_trait;
use serde::Deserialize;
use std::sync::Arc;

pub struct GmailGetLabelTool {
    pub client: Arc<Client>,
}
#[async_trait]
impl Tool for GmailGetLabelTool {
    fn name(&self) -> &str {
        "gmail_get_label"
    }
    fn description(&self) -> &str {
        "Get a Gmail label by ID."
    }
    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "id": { "type": "string", "description": "Label ID" }
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
        let label = self
            .client
            .labels_get(&a.id)
            .await
            .map_err(|e| McpError::ToolError(e.into()))?;
        Ok(serde_json::to_value(label)?)
    }
}

pub struct GmailCreateLabelTool {
    pub client: Arc<Client>,
}
#[async_trait]
impl Tool for GmailCreateLabelTool {
    fn name(&self) -> &str {
        "gmail_create_label"
    }
    fn description(&self) -> &str {
        "Create a new Gmail label."
    }
    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "name": { "type": "string", "description": "Label name" }
            },
            "required": ["name"],
            "additionalProperties": false
        })
    }
    async fn call(&self, args: serde_json::Value) -> Result<serde_json::Value, McpError> {
        #[derive(Deserialize)]
        struct Args {
            name: String,
        }
        let a: Args =
            serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;
        let label = self
            .client
            .labels_create(&a.name)
            .await
            .map_err(|e| McpError::ToolError(e.into()))?;
        Ok(serde_json::to_value(label)?)
    }
}

pub struct GmailUpdateLabelTool {
    pub client: Arc<Client>,
}
#[async_trait]
impl Tool for GmailUpdateLabelTool {
    fn name(&self) -> &str {
        "gmail_update_label"
    }
    fn description(&self) -> &str {
        "Update a Gmail label by ID."
    }
    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "id":   { "type": "string" },
                "name": { "type": "string" }
            },
            "required": ["id"],
            "additionalProperties": false
        })
    }
    async fn call(&self, args: serde_json::Value) -> Result<serde_json::Value, McpError> {
        #[derive(Deserialize)]
        struct Args {
            id: String,
            name: Option<String>,
        }
        let a: Args =
            serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;
        let label = self
            .client
            .labels_update(&a.id, a.name.as_deref())
            .await
            .map_err(|e| McpError::ToolError(e.into()))?;
        Ok(serde_json::to_value(label)?)
    }
}

pub struct GmailDeleteLabelTool {
    pub client: Arc<Client>,
}
#[async_trait]
impl Tool for GmailDeleteLabelTool {
    fn name(&self) -> &str {
        "gmail_delete_label"
    }
    fn description(&self) -> &str {
        "Delete a Gmail label by ID."
    }
    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": { "id": { "type": "string" } },
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
        self.client
            .labels_delete(&a.id)
            .await
            .map_err(|e| McpError::ToolError(e.into()))?;
        Ok(serde_json::json!({ "success": true }))
    }
}

pub struct GmailGetOrCreateLabelTool {
    pub client: Arc<Client>,
}
#[async_trait]
impl Tool for GmailGetOrCreateLabelTool {
    fn name(&self) -> &str {
        "gmail_get_or_create_label"
    }
    fn description(&self) -> &str {
        "Return a label by name, creating it if it does not exist."
    }
    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": { "name": { "type": "string" } },
            "required": ["name"],
            "additionalProperties": false
        })
    }
    async fn call(&self, args: serde_json::Value) -> Result<serde_json::Value, McpError> {
        #[derive(Deserialize)]
        struct Args {
            name: String,
        }
        let a: Args =
            serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;

        let list = self
            .client
            .labels_list()
            .await
            .map_err(|e| McpError::ToolError(e.into()))?;
        if let Some(labels) = list.labels {
            if let Some(lbl) = labels.into_iter().find(|l| l.name == a.name) {
                return Ok(serde_json::to_value(lbl)?);
            }
        }

        let label = self
            .client
            .labels_create(&a.name)
            .await
            .map_err(|e| McpError::ToolError(e.into()))?;
        Ok(serde_json::to_value(label)?)
    }
}

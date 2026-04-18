use crate::gmail::Client;
use crate::mcp::{McpError, Tool};
use async_trait::async_trait;
use serde::Deserialize;
use std::sync::Arc;

use super::compose::{ComposeArgs, build_raw, compose_schema};

// ── Create draft ─────────────────────────────────────────────────────────────

pub struct GmailCreateDraftTool {
    pub client: Arc<Client>,
}

#[async_trait]
impl Tool for GmailCreateDraftTool {
    fn name(&self) -> &str {
        "gmail_create_draft"
    }
    fn description(&self) -> &str {
        "Create a Gmail draft. Accepts the same compose arguments as gmail_send — \
         the server builds the MIME message automatically. Returns the draft ID."
    }
    fn input_schema(&self) -> serde_json::Value {
        compose_schema()
    }
    async fn call(&self, args: serde_json::Value) -> Result<serde_json::Value, McpError> {
        let a: ComposeArgs =
            serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;
        let (raw, _thread_id) = build_raw(a, &self.client).await?;
        let res = self
            .client
            .drafts_create(&raw)
            .await
            .map_err(|e| McpError::ToolError(e.into()))?;
        Ok(serde_json::to_value(res)?)
    }
}

// ── Update draft ─────────────────────────────────────────────────────────────

pub struct GmailUpdateDraftTool {
    pub client: Arc<Client>,
}

/// Compose args plus the draft ID to replace.
#[derive(Deserialize)]
struct UpdateDraftArgs {
    id: String,
    #[serde(flatten)]
    compose: ComposeArgs,
}

#[async_trait]
impl Tool for GmailUpdateDraftTool {
    fn name(&self) -> &str {
        "gmail_update_draft"
    }
    fn description(&self) -> &str {
        "Replace the content of an existing draft. Supply the draft `id` plus the \
         same compose fields as gmail_send. Returns the updated draft."
    }
    fn input_schema(&self) -> serde_json::Value {
        let mut schema = compose_schema();
        // Inject the required `id` property.
        if let Some(props) = schema.get_mut("properties").and_then(|p| p.as_object_mut()) {
            props.insert(
                "id".into(),
                serde_json::json!({ "type": "string", "description": "Draft ID to update." }),
            );
        }
        if let Some(req) = schema.get_mut("required").and_then(|r| r.as_array_mut()) {
            req.push(serde_json::json!("id"));
        }
        schema
    }
    async fn call(&self, args: serde_json::Value) -> Result<serde_json::Value, McpError> {
        let a: UpdateDraftArgs =
            serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;
        let (raw, _thread_id) = build_raw(a.compose, &self.client).await?;
        let res = self
            .client
            .drafts_update(&a.id, &raw)
            .await
            .map_err(|e| McpError::ToolError(e.into()))?;
        Ok(serde_json::to_value(res)?)
    }
}

// ── Send draft ────────────────────────────────────────────────────────────────

pub struct GmailSendDraftTool {
    pub client: Arc<Client>,
}

#[async_trait]
impl Tool for GmailSendDraftTool {
    fn name(&self) -> &str {
        "gmail_send_draft"
    }
    fn description(&self) -> &str {
        "Send an existing draft by its ID."
    }
    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": { "id": { "type": "string", "description": "Draft ID to send." } },
            "required": ["id"]
        })
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
            .drafts_send(&a.id)
            .await
            .map_err(|e| McpError::ToolError(e.into()))?;
        Ok(serde_json::to_value(res)?)
    }
}

// ── Delete draft ──────────────────────────────────────────────────────────────

pub struct GmailDeleteDraftTool {
    pub client: Arc<Client>,
}

#[async_trait]
impl Tool for GmailDeleteDraftTool {
    fn name(&self) -> &str {
        "gmail_delete_draft"
    }
    fn description(&self) -> &str {
        "Permanently delete a draft."
    }
    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": { "id": { "type": "string", "description": "Draft ID to delete." } },
            "required": ["id"]
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
            .drafts_delete(&a.id)
            .await
            .map_err(|e| McpError::ToolError(e.into()))?;
        Ok(serde_json::json!({ "success": true }))
    }
}

use crate::gmail::Client;
use crate::mcp::{McpError, Tool};
use async_trait::async_trait;
use serde::Deserialize;
use std::sync::Arc;

pub struct GmailCreateFilterTool {
    pub client: Arc<Client>,
}
#[async_trait]
impl Tool for GmailCreateFilterTool {
    fn name(&self) -> &str {
        "gmail_create_filter"
    }
    fn description(&self) -> &str {
        "Create a Gmail filter with criteria and action."
    }
    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "criteria": { "type": "object", "description": "FilterCriteria: from, to, subject, query, hasAttachment, etc." },
                "action":   { "type": "object", "description": "FilterAction: addLabelIds, removeLabelIds, forward" }
            },
            "required": ["criteria", "action"],
            "additionalProperties": false
        })
    }
    async fn call(&self, args: serde_json::Value) -> Result<serde_json::Value, McpError> {
        #[derive(Deserialize, serde::Serialize)]
        struct Args {
            criteria: serde_json::Value,
            action: serde_json::Value,
        }
        let a: Args =
            serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;
        let body = serde_json::json!({ "criteria": a.criteria, "action": a.action });
        let filter = self
            .client
            .filters_create(&body)
            .await
            .map_err(|e| McpError::ToolError(e.into()))?;
        Ok(serde_json::to_value(filter)?)
    }
}

pub struct GmailDeleteFilterTool {
    pub client: Arc<Client>,
}
#[async_trait]
impl Tool for GmailDeleteFilterTool {
    fn name(&self) -> &str {
        "gmail_delete_filter"
    }
    fn description(&self) -> &str {
        "Delete a Gmail filter by ID."
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
            .filters_delete(&a.id)
            .await
            .map_err(|e| McpError::ToolError(e.into()))?;
        Ok(serde_json::json!({ "success": true }))
    }
}

pub struct GmailCreateFilterFromTemplateTool {
    pub client: Arc<Client>,
}
#[async_trait]
impl Tool for GmailCreateFilterFromTemplateTool {
    fn name(&self) -> &str {
        "gmail_create_filter_from_template"
    }
    fn description(&self) -> &str {
        "Create a Gmail filter using a named template."
    }
    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "template": {
                    "type": "string",
                    "enum": ["auto_label_from", "archive_list", "forward_to", "delete_promotional"],
                    "description": "Template name"
                },
                "params": { "type": "object", "description": "Template-specific parameters (e.g. from, label, forward_to)" }
            },
            "required": ["template", "params"],
            "additionalProperties": false
        })
    }
    async fn call(&self, args: serde_json::Value) -> Result<serde_json::Value, McpError> {
        #[derive(Deserialize)]
        struct Args {
            template: String,
            params: serde_json::Value,
        }
        let a: Args =
            serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;

        let body = match a.template.as_str() {
            "auto_label_from" => {
                let from = a
                    .params
                    .get("from")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::InvalidParams("params.from required".into()))?;
                let label_id = a
                    .params
                    .get("label_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::InvalidParams("params.label_id required".into()))?;
                serde_json::json!({ "criteria": { "from": from }, "action": { "addLabelIds": [label_id] } })
            }
            "archive_list" => {
                let list_id = a
                    .params
                    .get("list_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::InvalidParams("params.list_id required".into()))?;
                serde_json::json!({ "criteria": { "query": format!("list:{}", list_id) }, "action": { "removeLabelIds": ["INBOX"] } })
            }
            "forward_to" => {
                let from = a
                    .params
                    .get("from")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::InvalidParams("params.from required".into()))?;
                let to = a
                    .params
                    .get("forward_to")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::InvalidParams("params.forward_to required".into()))?;
                serde_json::json!({ "criteria": { "from": from }, "action": { "forward": to } })
            }
            "delete_promotional" => {
                serde_json::json!({ "criteria": { "query": "category:promotions" }, "action": { "addLabelIds": ["TRASH"] } })
            }
            t => return Err(McpError::InvalidParams(format!("unknown template: {t}"))),
        };

        let filter = self
            .client
            .filters_create(&body)
            .await
            .map_err(|e| McpError::ToolError(e.into()))?;
        Ok(serde_json::to_value(filter)?)
    }
}

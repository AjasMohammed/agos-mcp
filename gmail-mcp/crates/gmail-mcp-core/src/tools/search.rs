use crate::gmail::{Client, MessagesListQuery};
use crate::mcp::{McpError, Tool};
use async_trait::async_trait;
use serde::Deserialize;
use std::sync::Arc;

pub struct GmailSearchTool {
    pub client: Arc<Client>,
}

#[async_trait]
impl Tool for GmailSearchTool {
    fn name(&self) -> &str {
        "gmail_search"
    }
    fn description(&self) -> &str {
        "Search messages in the authenticated mailbox using Gmail query syntax."
    }
    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "Gmail query, e.g. 'from:boss@corp.com is:unread newer_than:7d'",
                    "maxLength": 2048
                },
                "max_results": {
                    "type": "integer",
                    "minimum": 1,
                    "maximum": 500,
                    "default": 25
                },
                "page_token": { "type": "string", "maxLength": 4096 }
            },
            "required": ["query"],
            "additionalProperties": false
        })
    }
    async fn call(&self, args: serde_json::Value) -> Result<serde_json::Value, McpError> {
        #[derive(Deserialize)]
        struct Args {
            query: String,
            max_results: Option<u32>,
            page_token: Option<String>,
        }
        let a: Args =
            serde_json::from_value(args).map_err(|e| McpError::InvalidParams(e.to_string()))?;
        let resp = self
            .client
            .messages_list(&MessagesListQuery {
                q: Some(a.query),
                max_results: a.max_results.unwrap_or(25),
                page_token: a.page_token,
                ..Default::default()
            })
            .await
            .map_err(|e| McpError::ToolError(e.into()))?;
        Ok(serde_json::to_value(resp)?)
    }
}

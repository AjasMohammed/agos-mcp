use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;

use crate::error::LinkedInMcpError;
use crate::linkedin::LinkedInClient;
use crate::mcp::tools::Tool;

#[derive(Deserialize)]
struct Args {
    urn: String,
    text: Option<String>,
    visibility: Option<String>,
}

pub struct PostUpdate { pub client: Arc<LinkedInClient> }

#[async_trait]
impl Tool for PostUpdate {
    fn name(&self) -> &str { "linkedin-post-update" }
    fn description(&self) -> &str { "Edit the text or visibility of an existing LinkedIn post." }
    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "required": ["urn"],
            "properties": {
                "urn": { "type": "string", "pattern": "^urn:li:(share|ugcPost):[A-Za-z0-9_-]+$" },
                "text": { "type": "string", "minLength": 1, "maxLength": 3000 },
                "visibility": { "type": "string", "enum": ["PUBLIC", "CONNECTIONS", "LOGGED_IN"] }
            },
            "additionalProperties": false
        })
    }
    async fn call(&self, args: Value) -> Result<Value, LinkedInMcpError> {
        let args: Args = serde_json::from_value(args)
            .map_err(|e| LinkedInMcpError::InvalidInput(e.to_string()))?;
        if args.text.is_none() && args.visibility.is_none() {
            return Err(LinkedInMcpError::InvalidInput("at least one of text or visibility is required".into()));
        }
        self.client.update_post(&args.urn, args.text.as_deref(), args.visibility.as_deref()).await?;
        Ok(json!({ "updated": true }))
    }
}

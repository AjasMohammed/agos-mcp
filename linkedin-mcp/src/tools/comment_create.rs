use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;

use crate::error::LinkedInMcpError;
use crate::linkedin::LinkedInClient;
use crate::mcp::tools::Tool;

#[derive(Deserialize)]
struct Args {
    post_urn: String,
    text: String,
}

pub struct CommentCreate { pub client: Arc<LinkedInClient> }

#[async_trait]
impl Tool for CommentCreate {
    fn name(&self) -> &str { "linkedin-comment-create" }
    fn description(&self) -> &str { "Add a comment to a LinkedIn post." }
    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "required": ["post_urn", "text"],
            "properties": {
                "post_urn": { "type": "string", "pattern": "^urn:li:(share|ugcPost):[A-Za-z0-9_-]+$" },
                "text": { "type": "string", "minLength": 1, "maxLength": 1250 }
            },
            "additionalProperties": false
        })
    }
    async fn call(&self, args: Value) -> Result<Value, LinkedInMcpError> {
        let args: Args = serde_json::from_value(args)
            .map_err(|e| LinkedInMcpError::InvalidInput(e.to_string()))?;
        let urn = self.client.create_comment(&args.post_urn, &args.text).await?;
        Ok(json!({ "urn": urn }))
    }
}

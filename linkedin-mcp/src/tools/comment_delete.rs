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
    comment_urn: String,
}

pub struct CommentDelete { pub client: Arc<LinkedInClient> }

#[async_trait]
impl Tool for CommentDelete {
    fn name(&self) -> &str { "linkedin-comment-delete" }
    fn description(&self) -> &str { "Delete a comment from a LinkedIn post (must be authored by the authenticated member)." }
    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "required": ["post_urn", "comment_urn"],
            "properties": {
                "post_urn": { "type": "string", "pattern": "^urn:li:(share|ugcPost):[A-Za-z0-9_-]+$" },
                "comment_urn": { "type": "string" }
            },
            "additionalProperties": false
        })
    }
    async fn call(&self, args: Value) -> Result<Value, LinkedInMcpError> {
        let args: Args = serde_json::from_value(args)
            .map_err(|e| LinkedInMcpError::InvalidInput(e.to_string()))?;
        self.client.delete_comment(&args.post_urn, &args.comment_urn).await?;
        Ok(json!({ "deleted": true }))
    }
}

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
}

pub struct ReactionRemove { pub client: Arc<LinkedInClient> }

#[async_trait]
impl Tool for ReactionRemove {
    fn name(&self) -> &str { "linkedin-reaction-remove" }
    fn description(&self) -> &str { "Remove your reaction from a LinkedIn post (removes whichever reaction type is present)." }
    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "required": ["post_urn"],
            "properties": {
                "post_urn": { "type": "string", "pattern": "^urn:li:(share|ugcPost):[A-Za-z0-9_-]+$" }
            },
            "additionalProperties": false
        })
    }
    async fn call(&self, args: Value) -> Result<Value, LinkedInMcpError> {
        let args: Args = serde_json::from_value(args)
            .map_err(|e| LinkedInMcpError::InvalidInput(e.to_string()))?;
        self.client.remove_reaction(&args.post_urn).await?;
        Ok(json!({ "removed": true }))
    }
}

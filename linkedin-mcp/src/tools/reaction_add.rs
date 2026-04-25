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
    #[serde(default = "default_reaction")]
    reaction_type: String,
}

fn default_reaction() -> String { "LIKE".into() }

pub struct ReactionAdd { pub client: Arc<LinkedInClient> }

#[async_trait]
impl Tool for ReactionAdd {
    fn name(&self) -> &str { "linkedin-reaction-add" }
    fn description(&self) -> &str { "Add a reaction to a LinkedIn post or comment." }
    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "required": ["post_urn"],
            "properties": {
                "post_urn": { "type": "string", "pattern": "^urn:li:(share|ugcPost):[A-Za-z0-9_-]+$" },
                "reaction_type": {
                    "type": "string",
                    "enum": ["LIKE", "PRAISE", "APPRECIATION", "EMPATHY", "INTEREST", "ENTERTAINMENT"],
                    "default": "LIKE"
                }
            },
            "additionalProperties": false
        })
    }
    async fn call(&self, args: Value) -> Result<Value, LinkedInMcpError> {
        let args: Args = serde_json::from_value(args)
            .map_err(|e| LinkedInMcpError::InvalidInput(e.to_string()))?;
        self.client.add_reaction(&args.post_urn, &args.reaction_type).await?;
        Ok(json!({ "reacted": true }))
    }
}

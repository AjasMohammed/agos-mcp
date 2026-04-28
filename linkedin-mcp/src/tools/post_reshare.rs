use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;

use crate::error::LinkedInMcpError;
use crate::linkedin::{types::Distribution, LinkedInClient};
use crate::mcp::tools::Tool;

#[derive(Deserialize)]
struct Args {
    parent_urn: String,
    #[serde(default)]
    text: String,
    #[serde(default = "default_visibility")]
    visibility: String,
}

fn default_visibility() -> String { "PUBLIC".into() }

pub struct PostReshare { pub client: Arc<LinkedInClient> }

#[async_trait]
impl Tool for PostReshare {
    fn name(&self) -> &str { "linkedin-post-reshare" }
    fn description(&self) -> &str {
        "Reshare an existing LinkedIn post, optionally adding commentary."
    }
    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "required": ["parent_urn"],
            "properties": {
                "parent_urn": {
                    "type": "string",
                    "pattern": "^urn:li:(share|ugcPost|activity):[A-Za-z0-9_-]+$",
                    "description": "URN of the post to reshare"
                },
                "text": {
                    "type": "string",
                    "maxLength": 3000,
                    "description": "Optional commentary to add to the reshare"
                },
                "visibility": { "type": "string", "enum": ["PUBLIC", "CONNECTIONS", "LOGGED_IN"] }
            },
            "additionalProperties": false
        })
    }
    async fn call(&self, args: Value) -> Result<Value, LinkedInMcpError> {
        let args: Args = serde_json::from_value(args)
            .map_err(|e| LinkedInMcpError::InvalidInput(e.to_string()))?;
        let author = self.client.author_urn().await;
        let dist = serde_json::to_value(Distribution::default())
            .expect("Distribution is always serializable");
        let mut body = json!({
            "author": author,
            "visibility": args.visibility,
            "distribution": dist,
            "lifecycleState": "PUBLISHED",
            "isReshareDisabledByAuthor": false,
            "reshareContext": {
                "parent": args.parent_urn
            }
        });
        if !args.text.is_empty() {
            body["commentary"] = json!(args.text);
        }
        let urn = self.client.create_post(&body).await?;
        Ok(json!({ "urn": urn }))
    }
}

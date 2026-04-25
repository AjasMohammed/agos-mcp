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
    #[serde(default = "default_start")]
    start: u32,
    #[serde(default = "default_count")]
    count: u32,
}

fn default_start() -> u32 { 0 }
fn default_count() -> u32 { 20 }

pub struct CommentList { pub client: Arc<LinkedInClient> }

#[async_trait]
impl Tool for CommentList {
    fn name(&self) -> &str { "linkedin-comment-list" }
    fn description(&self) -> &str { "List comments on a LinkedIn post." }
    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "required": ["post_urn"],
            "properties": {
                "post_urn": { "type": "string", "pattern": "^urn:li:(share|ugcPost):[A-Za-z0-9_-]+$" },
                "start": { "type": "integer", "minimum": 0, "default": 0 },
                "count": { "type": "integer", "minimum": 1, "maximum": 100, "default": 20 }
            },
            "additionalProperties": false
        })
    }
    async fn call(&self, args: Value) -> Result<Value, LinkedInMcpError> {
        let args: Args = serde_json::from_value(args)
            .map_err(|e| LinkedInMcpError::InvalidInput(e.to_string()))?;
        self.client.list_comments(&args.post_urn, args.start, args.count).await
    }
}

use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;

use crate::error::LinkedInMcpError;
use crate::linkedin::LinkedInClient;
use crate::mcp::tools::Tool;

#[derive(Deserialize)]
struct Args {
    #[serde(default = "default_start")]
    start: u32,
    #[serde(default = "default_count")]
    count: u32,
}

fn default_start() -> u32 { 0 }
fn default_count() -> u32 { 10 }

pub struct PostsList { pub client: Arc<LinkedInClient> }

#[async_trait]
impl Tool for PostsList {
    fn name(&self) -> &str { "linkedin-posts-list" }
    fn description(&self) -> &str { "List posts authored by the authenticated LinkedIn member." }
    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "start": { "type": "integer", "minimum": 0, "default": 0 },
                "count": { "type": "integer", "minimum": 1, "maximum": 50, "default": 10 }
            },
            "additionalProperties": false
        })
    }
    async fn call(&self, args: Value) -> Result<Value, LinkedInMcpError> {
        let args: Args = serde_json::from_value(args)
            .map_err(|e| LinkedInMcpError::InvalidInput(e.to_string()))?;
        self.client.list_posts(args.start, args.count).await
    }
}

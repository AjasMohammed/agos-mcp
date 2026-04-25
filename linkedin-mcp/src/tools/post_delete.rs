use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;

use crate::error::LinkedInMcpError;
use crate::linkedin::LinkedInClient;
use crate::mcp::tools::Tool;

#[derive(Deserialize)]
struct Args { urn: String }

pub struct PostDelete { pub client: Arc<LinkedInClient> }

#[async_trait]
impl Tool for PostDelete {
    fn name(&self) -> &str { "linkedin-post-delete" }
    fn description(&self) -> &str { "Delete a LinkedIn post by URN (must be authored by the authenticated member)." }
    fn input_schema(&self) -> Value {
        json!({
            "type":"object",
            "required":["urn"],
            "properties": { "urn": { "type":"string", "pattern": "^urn:li:(share|ugcPost):[A-Za-z0-9_-]+$" } },
            "additionalProperties": false
        })
    }
    async fn call(&self, args: Value) -> Result<Value, LinkedInMcpError> {
        let args: Args = serde_json::from_value(args)
            .map_err(|e| LinkedInMcpError::InvalidInput(e.to_string()))?;
        self.client.delete_post(&args.urn).await?;
        Ok(json!({ "ok": true }))
    }
}

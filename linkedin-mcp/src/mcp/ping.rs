use async_trait::async_trait;
use serde_json::{json, Value};

use crate::error::LinkedInMcpError;
use super::tools::Tool;

pub struct Ping;

#[async_trait]
impl Tool for Ping {
    fn name(&self) -> &str { "ping" }
    fn description(&self) -> &str { "Return pong. Use to verify the server is alive." }
    fn input_schema(&self) -> Value { json!({ "type": "object", "properties": {}, "additionalProperties": false }) }
    async fn call(&self, _args: Value) -> Result<Value, LinkedInMcpError> {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| LinkedInMcpError::InvalidInput(e.to_string()))?
            .as_millis() as u64;
        Ok(json!({ "pong": true, "ts": ts }))
    }
}

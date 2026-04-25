use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;

use crate::error::LinkedInMcpError;
use crate::linkedin::LinkedInClient;
use crate::mcp::tools::Tool;

pub struct WhoAmI { pub client: Arc<LinkedInClient> }

#[async_trait]
impl Tool for WhoAmI {
    fn name(&self) -> &str { "linkedin-whoami" }
    fn description(&self) -> &str { "Return identity of the authenticated LinkedIn member (sub, name, email, picture)." }
    fn input_schema(&self) -> Value { json!({ "type":"object", "properties":{}, "additionalProperties": false }) }
    async fn call(&self, _args: Value) -> Result<Value, LinkedInMcpError> { self.client.userinfo().await }
}

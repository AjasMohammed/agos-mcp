use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;

use crate::error::LinkedInMcpError;
use crate::linkedin::LinkedInClient;
use crate::mcp::tools::Tool;

/// Reports authentication health without making a network call, so long-running
/// agentic workflows can check whether they're still authenticated and re-auth
/// proactively — before a token actually expires mid-task. Available even when
/// no token is present (so the agent can discover it must authenticate).
pub struct AuthStatus {
    pub client: Option<Arc<LinkedInClient>>,
}

#[async_trait]
impl Tool for AuthStatus {
    fn name(&self) -> &str {
        "linkedin-auth-status"
    }
    fn description(&self) -> &str {
        "Report LinkedIn authentication health (no network call): whether a token \
         is present, when the access and refresh tokens expire, and whether a \
         human re-auth (`linkedin-mcp auth`) is needed now or soon. Call this at \
         the start of a long-running workflow to avoid failing mid-task."
    }
    fn input_schema(&self) -> Value {
        json!({ "type": "object", "properties": {}, "additionalProperties": false })
    }
    async fn call(&self, _args: Value) -> Result<Value, LinkedInMcpError> {
        match &self.client {
            Some(client) => Ok(client.token_status().await),
            None => Ok(json!({
                "authenticated": false,
                "needs_reauth_soon": true,
                "next_action": "No token found — run `linkedin-mcp auth` to authenticate."
            })),
        }
    }
}

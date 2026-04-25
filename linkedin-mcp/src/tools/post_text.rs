use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;

use crate::error::LinkedInMcpError;
use crate::linkedin::{types::{Distribution, TextPostBody}, LinkedInClient};
use crate::mcp::tools::Tool;

#[derive(Deserialize)]
struct Args {
    text: String,
    #[serde(default = "default_visibility")]
    visibility: String,
}

fn default_visibility() -> String { "PUBLIC".into() }

pub struct PostText { pub client: Arc<LinkedInClient> }

#[async_trait]
impl Tool for PostText {
    fn name(&self) -> &str { "linkedin-post-text" }
    fn description(&self) -> &str { "Publish a text-only post to the authenticated member's LinkedIn feed." }
    fn input_schema(&self) -> Value {
        json!({
            "type":"object",
            "required":["text"],
            "properties":{
                "text": { "type":"string", "minLength": 1, "maxLength": 3000 },
                "visibility": { "type":"string", "enum":["PUBLIC","CONNECTIONS","LOGGED_IN"] }
            },
            "additionalProperties": false
        })
    }
    async fn call(&self, args: Value) -> Result<Value, LinkedInMcpError> {
        let args: Args = serde_json::from_value(args)
            .map_err(|e| LinkedInMcpError::InvalidInput(e.to_string()))?;
        let author = self.client.author_urn().await;
        let body = TextPostBody {
            author: &author,
            commentary: &args.text,
            visibility: &args.visibility,
            distribution: Distribution::default(),
            lifecycle_state: "PUBLISHED",
            is_reshare_disabled_by_author: false,
        };
        let urn = self.client.create_post(&body).await?;
        Ok(json!({ "urn": urn }))
    }
}

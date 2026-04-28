use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;

use crate::error::LinkedInMcpError;
use crate::linkedin::{types::Distribution, LinkedInClient};
use crate::mcp::tools::Tool;

#[derive(Deserialize)]
struct Args {
    text: String,
    question: String,
    options: Vec<String>,
    #[serde(default = "default_duration")]
    duration: String,
    #[serde(default = "default_visibility")]
    visibility: String,
}

fn default_duration() -> String { "THREE_DAYS".into() }
fn default_visibility() -> String { "PUBLIC".into() }

pub struct PostPoll { pub client: Arc<LinkedInClient> }

#[async_trait]
impl Tool for PostPoll {
    fn name(&self) -> &str { "linkedin-post-poll" }
    fn description(&self) -> &str {
        "Publish a LinkedIn post with a poll. Provide a question and 2-4 answer options."
    }
    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "required": ["text", "question", "options"],
            "properties": {
                "text": { "type": "string", "minLength": 1, "maxLength": 3000 },
                "question": { "type": "string", "minLength": 1, "maxLength": 140 },
                "options": {
                    "type": "array",
                    "items": { "type": "string", "minLength": 1, "maxLength": 30 },
                    "minItems": 2,
                    "maxItems": 4
                },
                "duration": {
                    "type": "string",
                    "enum": ["ONE_DAY", "THREE_DAYS", "ONE_WEEK", "TWO_WEEKS"],
                    "default": "THREE_DAYS"
                },
                "visibility": { "type": "string", "enum": ["PUBLIC", "CONNECTIONS", "LOGGED_IN"] }
            },
            "additionalProperties": false
        })
    }
    async fn call(&self, args: Value) -> Result<Value, LinkedInMcpError> {
        let args: Args = serde_json::from_value(args)
            .map_err(|e| LinkedInMcpError::InvalidInput(e.to_string()))?;

        if args.question.chars().count() > 140 {
            return Err(LinkedInMcpError::InvalidInput(
                "question exceeds 140 characters".into(),
            ));
        }
        for opt in &args.options {
            if opt.chars().count() > 30 {
                return Err(LinkedInMcpError::InvalidInput(
                    format!("option \"{opt}\" exceeds 30 characters"),
                ));
            }
        }

        let author = self.client.author_urn().await;
        let options: Vec<Value> = args.options.iter().map(|o| json!({ "text": o })).collect();
        let dist = serde_json::to_value(Distribution::default())
            .expect("Distribution is always serializable");
        let body = json!({
            "author": author,
            "commentary": args.text,
            "visibility": args.visibility,
            "distribution": dist,
            "lifecycleState": "PUBLISHED",
            "isReshareDisabledByAuthor": false,
            "content": {
                "poll": {
                    "question": args.question,
                    "options": options,
                    "settings": {
                        "duration": args.duration
                    }
                }
            }
        });
        let urn = self.client.create_post(&body).await?;
        Ok(json!({ "urn": urn }))
    }
}

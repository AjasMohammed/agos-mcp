use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;

use crate::error::LinkedInMcpError;
use crate::linkedin::{
    types::{ArticlePostBody, Distribution, PostArticleContent, PostArticleItem},
    LinkedInClient,
};
use crate::mcp::tools::Tool;

#[derive(Deserialize)]
struct Args {
    url: String,
    text: String,
    title: Option<String>,
    description: Option<String>,
    #[serde(default = "default_visibility")]
    visibility: String,
}

fn default_visibility() -> String { "PUBLIC".into() }

pub struct PostArticle { pub client: Arc<LinkedInClient> }

#[async_trait]
impl Tool for PostArticle {
    fn name(&self) -> &str { "linkedin-post-article" }
    fn description(&self) -> &str { "Publish a LinkedIn post with an article/URL link." }
    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "required": ["url", "text"],
            "properties": {
                "url": { "type": "string", "format": "uri" },
                "text": { "type": "string", "minLength": 1, "maxLength": 3000 },
                "title": { "type": "string", "maxLength": 400 },
                "description": { "type": "string", "maxLength": 400 },
                "visibility": { "type": "string", "enum": ["PUBLIC", "CONNECTIONS", "LOGGED_IN"] }
            },
            "additionalProperties": false
        })
    }
    async fn call(&self, args: Value) -> Result<Value, LinkedInMcpError> {
        let args: Args = serde_json::from_value(args)
            .map_err(|e| LinkedInMcpError::InvalidInput(e.to_string()))?;
        let author = self.client.author_urn().await;
        let body = ArticlePostBody {
            author: &author,
            commentary: &args.text,
            visibility: &args.visibility,
            distribution: Distribution::default(),
            lifecycle_state: "PUBLISHED",
            is_reshare_disabled_by_author: false,
            content: PostArticleContent {
                article: PostArticleItem {
                    source: &args.url,
                    title: args.title.as_deref(),
                    description: args.description.as_deref(),
                },
            },
        };
        let urn = self.client.create_post(&body).await?;
        Ok(json!({ "urn": urn }))
    }
}

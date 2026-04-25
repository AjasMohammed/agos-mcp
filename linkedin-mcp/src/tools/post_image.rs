use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;

use crate::error::LinkedInMcpError;
use crate::linkedin::{
    types::{Distribution, MediaPostBody, PostMediaContent, PostMediaItem},
    LinkedInClient,
};
use crate::mcp::tools::Tool;

#[derive(Deserialize)]
struct Args {
    image_path: String,
    text: String,
    #[serde(default = "default_visibility")]
    visibility: String,
}

fn default_visibility() -> String { "PUBLIC".into() }

pub struct PostImage { pub client: Arc<LinkedInClient> }

#[async_trait]
impl Tool for PostImage {
    fn name(&self) -> &str { "linkedin-post-image" }
    fn description(&self) -> &str {
        "Publish a LinkedIn post with an image. Provide the absolute path to the image file on the server."
    }
    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "required": ["image_path", "text"],
            "properties": {
                "image_path": { "type": "string", "description": "Absolute path to image file (JPEG, PNG, GIF, WebP)" },
                "text": { "type": "string", "minLength": 1, "maxLength": 3000 },
                "visibility": { "type": "string", "enum": ["PUBLIC", "CONNECTIONS", "LOGGED_IN"] }
            },
            "additionalProperties": false
        })
    }
    async fn call(&self, args: Value) -> Result<Value, LinkedInMcpError> {
        let args: Args = serde_json::from_value(args)
            .map_err(|e| LinkedInMcpError::InvalidInput(e.to_string()))?;
        let path = std::path::Path::new(&args.image_path);
        if !path.exists() {
            return Err(LinkedInMcpError::InvalidInput(format!("file not found: {}", args.image_path)));
        }
        let image_urn = self.client.upload_image(path).await?;
        let author = self.client.author_urn().await;
        let body = MediaPostBody {
            author: &author,
            commentary: &args.text,
            visibility: &args.visibility,
            distribution: Distribution::default(),
            lifecycle_state: "PUBLISHED",
            is_reshare_disabled_by_author: false,
            content: PostMediaContent { media: PostMediaItem { id: &image_urn } },
        };
        let urn = self.client.create_post(&body).await?;
        Ok(json!({ "urn": urn }))
    }
}

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
    video_path: String,
    text: String,
    #[serde(default = "default_visibility")]
    visibility: String,
}

fn default_visibility() -> String { "PUBLIC".into() }

pub struct PostVideo { pub client: Arc<LinkedInClient> }

#[async_trait]
impl Tool for PostVideo {
    fn name(&self) -> &str { "linkedin-post-video" }
    fn description(&self) -> &str {
        "Publish a LinkedIn post with a video. Provide the absolute path to the video file on the server."
    }
    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "required": ["video_path", "text"],
            "properties": {
                "video_path": { "type": "string", "description": "Absolute path to video file (MP4, MOV, AVI)" },
                "text": { "type": "string", "minLength": 1, "maxLength": 3000 },
                "visibility": { "type": "string", "enum": ["PUBLIC", "CONNECTIONS", "LOGGED_IN"] }
            },
            "additionalProperties": false
        })
    }
    async fn call(&self, args: Value) -> Result<Value, LinkedInMcpError> {
        let args: Args = serde_json::from_value(args)
            .map_err(|e| LinkedInMcpError::InvalidInput(e.to_string()))?;
        let path = std::path::Path::new(&args.video_path);
        if !path.exists() {
            return Err(LinkedInMcpError::InvalidInput(format!("file not found: {}", args.video_path)));
        }
        let video_urn = self.client.upload_video(path).await?;
        let author = self.client.author_urn().await;
        let body = MediaPostBody {
            author: &author,
            commentary: &args.text,
            visibility: &args.visibility,
            distribution: Distribution::default(),
            lifecycle_state: "PUBLISHED",
            is_reshare_disabled_by_author: false,
            content: PostMediaContent { media: PostMediaItem { id: &video_urn } },
        };
        let urn = self.client.create_post(&body).await?;
        Ok(json!({ "urn": urn }))
    }
}

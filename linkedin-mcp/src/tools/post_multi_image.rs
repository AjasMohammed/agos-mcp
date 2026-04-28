use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::task::JoinSet;

use crate::error::LinkedInMcpError;
use crate::linkedin::{types::Distribution, LinkedInClient};
use crate::mcp::tools::Tool;

#[derive(Deserialize)]
struct Args {
    image_paths: Vec<String>,
    text: String,
    #[serde(default = "default_visibility")]
    visibility: String,
}

fn default_visibility() -> String { "PUBLIC".into() }

pub struct PostMultiImage { pub client: Arc<LinkedInClient> }

#[async_trait]
impl Tool for PostMultiImage {
    fn name(&self) -> &str { "linkedin-post-multi-image" }
    fn description(&self) -> &str {
        "Publish a LinkedIn carousel post with 2-9 images. Provide absolute paths to image files."
    }
    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "required": ["image_paths", "text"],
            "properties": {
                "image_paths": {
                    "type": "array",
                    "items": { "type": "string" },
                    "minItems": 2,
                    "maxItems": 9,
                    "description": "Absolute paths to image files (JPEG, PNG, GIF, WebP)"
                },
                "text": { "type": "string", "minLength": 1, "maxLength": 3000 },
                "visibility": { "type": "string", "enum": ["PUBLIC", "CONNECTIONS", "LOGGED_IN"] }
            },
            "additionalProperties": false
        })
    }
    async fn call(&self, args: Value) -> Result<Value, LinkedInMcpError> {
        let args: Args = serde_json::from_value(args)
            .map_err(|e| LinkedInMcpError::InvalidInput(e.to_string()))?;

        // Validate all paths and fetch cheap cached data before any upload.
        for path_str in &args.image_paths {
            if !std::path::Path::new(path_str).exists() {
                return Err(LinkedInMcpError::InvalidInput(
                    format!("file not found: {path_str}"),
                ));
            }
        }
        let author = self.client.author_urn().await;

        // Upload all images in parallel. JoinSet cancels remaining tasks on drop,
        // preventing orphaned LinkedIn assets if any upload fails.
        let n = args.image_paths.len();
        let mut set: JoinSet<(usize, Result<String, LinkedInMcpError>)> = JoinSet::new();
        for (i, p) in args.image_paths.iter().enumerate() {
            let client = self.client.clone();
            let path = std::path::PathBuf::from(p);
            set.spawn(async move { (i, client.upload_image(&path).await) });
        }

        // Collect results; abort all on first error (JoinSet drop cancels remaining).
        let mut image_urns: Vec<(usize, String)> = Vec::with_capacity(n);
        while let Some(res) = set.join_next().await {
            let (i, upload_result) = res
                .map_err(|e| LinkedInMcpError::LinkedInServerError(e.to_string()))?;
            match upload_result {
                Ok(urn) => image_urns.push((i, urn)),
                Err(e) => {
                    set.abort_all();
                    return Err(e);
                }
            }
        }
        image_urns.sort_by_key(|(i, _)| *i);

        let images: Vec<Value> = image_urns.into_iter()
            .map(|(_, id)| json!({ "id": id }))
            .collect();
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
                "multiImage": {
                    "images": images
                }
            }
        });
        let urn = self.client.create_post(&body).await?;
        Ok(json!({ "urn": urn }))
    }
}

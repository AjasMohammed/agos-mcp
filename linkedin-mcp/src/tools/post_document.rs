use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;

use crate::error::LinkedInMcpError;
use crate::linkedin::{types::Distribution, LinkedInClient};
use crate::mcp::tools::Tool;

#[derive(Deserialize)]
struct Args {
    document_path: String,
    text: String,
    title: Option<String>,
    #[serde(default = "default_visibility")]
    visibility: String,
}

fn default_visibility() -> String { "PUBLIC".into() }

pub struct PostDocument { pub client: Arc<LinkedInClient> }

#[async_trait]
impl Tool for PostDocument {
    fn name(&self) -> &str { "linkedin-post-document" }
    fn description(&self) -> &str {
        "Publish a LinkedIn post with a document (PDF, PPTX, DOCX). Provide the absolute path to the file."
    }
    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "required": ["document_path", "text"],
            "properties": {
                "document_path": {
                    "type": "string",
                    "description": "Absolute path to document file (PDF, PPTX, DOCX)"
                },
                "text": { "type": "string", "minLength": 1, "maxLength": 3000 },
                "title": { "type": "string", "maxLength": 200 },
                "visibility": { "type": "string", "enum": ["PUBLIC", "CONNECTIONS", "LOGGED_IN"] }
            },
            "additionalProperties": false
        })
    }
    async fn call(&self, args: Value) -> Result<Value, LinkedInMcpError> {
        let args: Args = serde_json::from_value(args)
            .map_err(|e| LinkedInMcpError::InvalidInput(e.to_string()))?;

        let path = std::path::Path::new(&args.document_path);
        if !path.exists() {
            return Err(LinkedInMcpError::InvalidInput(
                format!("file not found: {}", args.document_path),
            ));
        }
        let file_size = std::fs::metadata(path)
            .map_err(|e| LinkedInMcpError::InvalidInput(e.to_string()))?
            .len();
        if file_size > 100 * 1024 * 1024 {
            return Err(LinkedInMcpError::InvalidInput(
                format!("file exceeds LinkedIn's 100 MB document limit ({} bytes)", file_size),
            ));
        }

        // Fetch author URN before the potentially long upload.
        let author = self.client.author_urn().await;
        let doc_urn = self.client.upload_document(path).await?;

        let mut media = json!({ "id": doc_urn });
        if let Some(ref title) = args.title {
            media["title"] = json!(title);
        }
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
                "media": media
            }
        });
        let urn = self.client.create_post(&body).await?;
        Ok(json!({ "urn": urn }))
    }
}

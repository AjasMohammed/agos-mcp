use crate::gmail::mime::{Compose, ComposeAttachment, render, to_gmail_raw};
use crate::gmail::{Client, MessageFormat};
use crate::mcp::McpError;
use serde::Deserialize;
use std::fmt;
use std::sync::Arc;

/// Deserializes a single string or an array of strings as `Vec<String>`.
///
/// LLMs frequently pass `"user@example.com"` when the schema says array.
/// Accepting both forms here prevents spurious schema-validation failures
/// without weakening the canonical array representation in tool output.
pub fn deserialize_string_or_array<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct Visitor;
    impl<'de> serde::de::Visitor<'de> for Visitor {
        type Value = Vec<String>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "a string or an array of strings")
        }
        fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Vec<String>, E> {
            Ok(vec![v.to_owned()])
        }
        fn visit_string<E: serde::de::Error>(self, v: String) -> Result<Vec<String>, E> {
            Ok(vec![v])
        }
        fn visit_seq<A: serde::de::SeqAccess<'de>>(
            self,
            mut seq: A,
        ) -> Result<Vec<String>, A::Error> {
            let mut out = Vec::new();
            while let Some(s) = seq.next_element::<String>()? {
                out.push(s);
            }
            Ok(out)
        }
    }
    deserializer.deserialize_any(Visitor)
}

/// Optional string-or-array field — same coercion, but the whole field is optional.
pub fn deserialize_opt_string_or_array<'de, D>(
    deserializer: D,
) -> Result<Option<Vec<String>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct Visitor;
    impl<'de> serde::de::Visitor<'de> for Visitor {
        type Value = Option<Vec<String>>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "null, a string, or an array of strings")
        }
        fn visit_none<E: serde::de::Error>(self) -> Result<Option<Vec<String>>, E> {
            Ok(None)
        }
        fn visit_some<D2: serde::Deserializer<'de>>(
            self,
            de: D2,
        ) -> Result<Option<Vec<String>>, D2::Error> {
            deserialize_string_or_array(de).map(Some)
        }
        fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Option<Vec<String>>, E> {
            Ok(Some(vec![v.to_owned()]))
        }
        fn visit_string<E: serde::de::Error>(self, v: String) -> Result<Option<Vec<String>>, E> {
            Ok(Some(vec![v]))
        }
        fn visit_seq<A: serde::de::SeqAccess<'de>>(
            self,
            mut seq: A,
        ) -> Result<Option<Vec<String>>, A::Error> {
            let mut out = Vec::new();
            while let Some(s) = seq.next_element::<String>()? {
                out.push(s);
            }
            Ok(Some(out))
        }
    }
    deserializer.deserialize_any(Visitor)
}

/// Human-friendly compose arguments shared by send and draft tools.
#[derive(Deserialize)]
pub struct ComposeArgs {
    #[serde(
        alias = "recipient",
        alias = "recipients",
        alias = "email",
        alias = "to_addresses",
        deserialize_with = "deserialize_string_or_array"
    )]
    pub to: Vec<String>,
    #[serde(default, deserialize_with = "deserialize_opt_string_or_array")]
    pub cc: Option<Vec<String>>,
    #[serde(default, deserialize_with = "deserialize_opt_string_or_array")]
    pub bcc: Option<Vec<String>>,
    pub subject: String,
    #[serde(alias = "body")]
    pub body_text: Option<String>,
    #[serde(alias = "html")]
    pub body_html: Option<String>,
    /// Markdown source. Server renders to HTML and populates the `text/html`
    /// part. The raw markdown is also used as the `text/plain` fallback when
    /// `body_text` is omitted, since CommonMark reads cleanly as plain text.
    #[serde(alias = "markdown")]
    pub body_markdown: Option<String>,
    pub attachments: Option<Vec<AttachmentArg>>,
    /// Message-ID of the email being replied to. The server fetches headers
    /// automatically to set `In-Reply-To` and `References` for correct threading.
    pub reply_to_message_id: Option<String>,
}

#[derive(Deserialize)]
pub struct AttachmentArg {
    pub filename: String,
    pub content_type: String,
    /// Base64-encoded (standard alphabet) file content.
    pub content_base64: String,
    /// Content-ID for inline images (`cid:…` references in HTML body).
    pub content_id: Option<String>,
}

/// JSON Schema fragment shared by send and draft tools.
pub fn compose_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "to": {
                "description": "REQUIRED key: 'to' (NOT 'recipient'/'email'). Single string or array.",
                "oneOf": [
                    { "type": "string" },
                    { "type": "array", "items": { "type": "string" }, "minItems": 1 }
                ]
            },
            "cc": {
                "description": "CC recipient(s). String or array.",
                "oneOf": [
                    { "type": "string" },
                    { "type": "array", "items": { "type": "string" } }
                ]
            },
            "bcc": {
                "description": "BCC recipient(s). String or array.",
                "oneOf": [
                    { "type": "string" },
                    { "type": "array", "items": { "type": "string" } }
                ]
            },
            "subject": { "type": "string", "description": "Email subject line." },
            "body_text": { "type": "string", "description": "Plain-text body. Alias: 'body'." },
            "body": { "type": "string", "description": "Alias for body_text. Plain-text body." },
            "body_html": { "type": "string", "description": "HTML body. Alias: 'html'. Supply both body_text and body_html for multipart/alternative." },
            "html": { "type": "string", "description": "Alias for body_html. HTML body." },
            "body_markdown": { "type": "string", "description": "Markdown body (CommonMark). Server renders to HTML for display in mail clients and uses the raw markdown as the plain-text fallback. Cannot be combined with body_html. Alias: 'markdown'." },
            "markdown": { "type": "string", "description": "Alias for body_markdown." },
            "attachments": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "filename":       { "type": "string" },
                        "content_type":   { "type": "string", "description": "MIME type, e.g. application/pdf" },
                        "content_base64": { "type": "string", "description": "Standard base64-encoded file bytes." },
                        "content_id":     { "type": "string",  "description": "Optional Content-ID for inline images." }
                    },
                    "required": ["filename", "content_type", "content_base64"]
                }
            },
            "reply_to_message_id": {
                "type": "string",
                "description": "ID of the message being replied to. Server fetches In-Reply-To and References headers automatically."
            }
        },
        "required": ["to", "subject"],
        "additionalProperties": false
    })
}

/// Extract a named header from a `Message` (case-insensitive).
fn extract_header(msg: &crate::gmail::types::Message, name: &str) -> Option<String> {
    msg.payload
        .as_ref()?
        .headers
        .iter()
        .find(|h| h.name.eq_ignore_ascii_case(name))
        .map(|h| h.value.clone())
}

/// Convert `ComposeArgs` into a base64url-encoded RFC 2822 message ready for
/// the Gmail API `raw` field. Fetches reply headers from the API when
/// `reply_to_message_id` is set.
pub async fn build_raw(
    args: ComposeArgs,
    client: &Arc<Client>,
) -> Result<(String, Option<String>), McpError> {
    let (thread_id, in_reply_to, references) = match args.reply_to_message_id.as_deref() {
        Some(id) => {
            let orig = client
                .messages_get(id, MessageFormat::Metadata)
                .await
                .map_err(|e| McpError::ToolError(e.into()))?;
            let msg_id = extract_header(&orig, "Message-ID");
            let existing_refs = extract_header(&orig, "References");
            let refs = match (&existing_refs, &msg_id) {
                (Some(r), Some(m)) => vec![r.clone(), m.clone()],
                (None, Some(m)) => vec![m.clone()],
                (Some(r), None) => vec![r.clone()],
                (None, None) => vec![],
            };
            (Some(orig.thread_id), msg_id, refs)
        }
        None => (None, None, vec![]),
    };

    let (body_text, body_html) = resolve_bodies(args.body_text, args.body_html, args.body_markdown)?;

    let mut atts = Vec::new();
    for a in args.attachments.unwrap_or_default() {
        use base64::Engine;
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(&a.content_base64)
            .map_err(|_| McpError::InvalidParams("attachment: invalid base64".into()))?;
        atts.push(ComposeAttachment {
            filename: a.filename,
            content_type: a.content_type,
            bytes,
            content_id: a.content_id,
        });
    }

    let raw = to_gmail_raw(
        &render(&Compose {
            to: args.to,
            cc: args.cc.unwrap_or_default(),
            bcc: args.bcc.unwrap_or_default(),
            subject: args.subject,
            body_text,
            body_html,
            attachments: atts,
            thread_id: thread_id.clone(),
            in_reply_to,
            references,
        })
        .map_err(|e| McpError::ToolError(e.into()))?,
    );

    Ok((raw, thread_id))
}

/// Resolve the final `(body_text, body_html)` pair from the three input fields.
///
/// `body_markdown` renders to HTML and, when no explicit `body_text` is given,
/// also serves as the plain-text fallback so non-HTML clients see readable
/// content rather than nothing. Supplying both `body_markdown` and `body_html`
/// is rejected — there's no defensible merge, and silently picking one would
/// hide a caller mistake.
fn resolve_bodies(
    body_text: Option<String>,
    body_html: Option<String>,
    body_markdown: Option<String>,
) -> Result<(Option<String>, Option<String>), McpError> {
    if body_markdown.is_some() && body_html.is_some() {
        return Err(McpError::InvalidParams(
            "body_markdown and body_html are mutually exclusive".into(),
        ));
    }
    let Some(md) = body_markdown else {
        return Ok((body_text, body_html));
    };
    let html = render_markdown(&md);
    let text = body_text.or(Some(md));
    Ok((text, Some(html)))
}

fn render_markdown(src: &str) -> String {
    use pulldown_cmark::{Options, Parser, html};
    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_STRIKETHROUGH);
    opts.insert(Options::ENABLE_TABLES);
    opts.insert(Options::ENABLE_TASKLISTS);
    opts.insert(Options::ENABLE_SMART_PUNCTUATION);
    let parser = Parser::new_ext(src, opts);
    let mut out = String::with_capacity(src.len() + src.len() / 4);
    html::push_html(&mut out, parser);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn markdown_renders_to_html_and_text_falls_back_to_source() {
        let (text, html) =
            resolve_bodies(None, None, Some("**hi** there".into())).expect("resolve");
        assert_eq!(text.as_deref(), Some("**hi** there"));
        let html = html.expect("html present");
        assert!(html.contains("<strong>hi</strong>"), "got: {html}");
    }

    #[test]
    fn explicit_body_text_overrides_markdown_fallback() {
        let (text, html) = resolve_bodies(
            Some("plain version".into()),
            None,
            Some("**hi**".into()),
        )
        .expect("resolve");
        assert_eq!(text.as_deref(), Some("plain version"));
        assert!(html.unwrap().contains("<strong>hi</strong>"));
    }

    #[test]
    fn markdown_and_html_together_are_rejected() {
        let err = resolve_bodies(
            None,
            Some("<p>hi</p>".into()),
            Some("**hi**".into()),
        )
        .expect_err("should reject");
        assert!(matches!(err, McpError::InvalidParams(_)));
    }

    #[test]
    fn no_markdown_passes_through_unchanged() {
        let (text, html) = resolve_bodies(
            Some("t".into()),
            Some("<p>h</p>".into()),
            None,
        )
        .expect("resolve");
        assert_eq!(text.as_deref(), Some("t"));
        assert_eq!(html.as_deref(), Some("<p>h</p>"));
    }
}

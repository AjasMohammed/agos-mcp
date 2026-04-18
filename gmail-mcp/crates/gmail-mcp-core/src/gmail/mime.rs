use crate::gmail::errors::GmailError;

pub struct Compose {
    pub to: Vec<String>,
    pub cc: Vec<String>,
    pub bcc: Vec<String>,
    pub subject: String,
    pub body_text: Option<String>,
    pub body_html: Option<String>,
    pub attachments: Vec<ComposeAttachment>,
    pub thread_id: Option<String>,   // for replies
    pub in_reply_to: Option<String>, // RFC 5322 Message-ID
    pub references: Vec<String>,
}

pub struct ComposeAttachment {
    pub filename: String,
    pub content_type: String,
    pub bytes: Vec<u8>,
    pub content_id: Option<String>, // for inline images (cid:…)
}

pub fn render(msg: &Compose) -> Result<Vec<u8>, GmailError> {
    let mut builder = mail_builder::MessageBuilder::new()
        .to(msg.to.iter().map(|s| s.as_str()).collect::<Vec<_>>())
        .subject(&msg.subject);

    if !msg.cc.is_empty() {
        builder = builder.cc(msg.cc.iter().map(|s| s.as_str()).collect::<Vec<_>>());
    }
    if !msg.bcc.is_empty() {
        builder = builder.bcc(msg.bcc.iter().map(|s| s.as_str()).collect::<Vec<_>>());
    }
    if let Some(r) = &msg.in_reply_to {
        builder = builder.in_reply_to(r.as_str());
    }
    if !msg.references.is_empty() {
        builder = builder.references(
            msg.references
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>(),
        );
    }

    if let Some(t) = &msg.body_text {
        builder = builder.text_body(t);
    }
    if let Some(h) = &msg.body_html {
        builder = builder.html_body(h);
    }

    for a in &msg.attachments {
        // We use mail_builder's attachment method which expects content, mime subtype, and filename.
        // mail-builder 0.3 allows creating parts manually or using helper methods.
        builder = builder.attachment(&a.content_type, &a.filename, a.bytes.clone());
    }

    let mut buf = Vec::new();
    builder
        .write_to(&mut buf)
        .map_err(|e| GmailError::Other(e.to_string()))?;
    Ok(buf)
}

pub fn to_gmail_raw(bytes: &[u8]) -> String {
    base64::Engine::encode(&base64::engine::general_purpose::URL_SAFE_NO_PAD, bytes)
}

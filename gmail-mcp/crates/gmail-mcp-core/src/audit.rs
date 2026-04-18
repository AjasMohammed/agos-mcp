use async_trait::async_trait;
use serde::Serialize;
use std::sync::Arc;

#[derive(Serialize, Clone)]
pub struct AuditEvent {
    pub ts: String,
    pub event: String,
    pub account: String,
    pub account_email_hash: String,
    pub tool: Option<String>,
    pub args_hash: Option<String>,
    pub scopes_used: Vec<String>,
    pub result: String,
    pub error_code: Option<i32>,
    pub error_kind: Option<String>,
    pub duration_ms: u64,
    pub gmail_cost_units: Option<u32>,
    pub message_ids: Option<Vec<String>>,
    pub trace_id: String,
}

#[async_trait]
pub trait AuditEmit: Send + Sync {
    async fn emit(&self, event: AuditEvent);
}

pub struct StderrJsonEmitter;

#[async_trait]
impl AuditEmit for StderrJsonEmitter {
    async fn emit(&self, event: AuditEvent) {
        if let Ok(line) = serde_json::to_string(&event) {
            eprintln!("{line}");
        }
    }
}

pub struct AuditSink {
    pub inner: Arc<dyn AuditEmit>,
}

impl AuditSink {
    pub fn new(inner: Arc<dyn AuditEmit>) -> Self {
        Self { inner }
    }

    pub async fn emit(&self, event: AuditEvent) {
        self.inner.emit(event).await;
    }
}

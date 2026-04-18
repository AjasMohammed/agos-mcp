use axum::{
    Json, Router,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    response::sse::{Event, KeepAlive, Sse},
    routing::{get, post},
};
use futures::StreamExt;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::sync::mpsc;

use crate::mcp::server::McpServer;

pub struct HttpState {
    pub server: Arc<McpServer>,
    pub sessions: SessionManager,
    // Note: To validate tokens we'd hold a reference to HttpTokenManager here
}

pub struct SessionManager {
    // Session ID -> Session
    sessions: RwLock<HashMap<String, Session>>,
}

#[derive(Clone)]
pub struct Session {
    pub id: String,
    pub account: String,
    // Sender to the SSE stream task
    pub event_tx: mpsc::Sender<serde_json::Value>,
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: RwLock::new(HashMap::new()),
        }
    }

    pub async fn create(&self, account: String) -> (Session, mpsc::Receiver<serde_json::Value>) {
        use rand::RngCore;
        let mut b = vec![0u8; 16];
        rand::thread_rng().fill_bytes(&mut b);
        let id = hex::encode(b);

        let (tx, rx) = mpsc::channel(100);
        let session = Session {
            id: id.clone(),
            account,
            event_tx: tx,
        };

        self.sessions.write().await.insert(id, session.clone());
        (session, rx)
    }

    pub async fn get(&self, id: &str) -> Option<Session> {
        self.sessions.read().await.get(id).cloned()
    }
}

pub fn router(state: Arc<HttpState>) -> Router {
    Router::new()
        .route("/sse", get(handle_sse))
        .route("/message", post(handle_message))
        .route("/healthz", get(|| async { "ok" }))
        .with_state(state)
}

fn extract_token(headers: &HeaderMap) -> Option<String> {
    headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(|v| v.to_string())
}

async fn handle_sse(
    State(state): State<Arc<HttpState>>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, StatusCode> {
    // In a real app we'd validate the token here via HttpTokenManager
    let _token = extract_token(&headers).ok_or(StatusCode::UNAUTHORIZED)?;
    let account = "default".to_string(); // In a real app this comes from token meta

    let (_session, rx) = state.sessions.create(account).await;

    let stream = tokio_stream::wrappers::ReceiverStream::new(rx).map(|resp| {
        Ok::<_, std::convert::Infallible>(
            Event::default().data(serde_json::to_string(&resp).unwrap_or_default()),
        )
    });

    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}

async fn handle_message(
    State(state): State<Arc<HttpState>>,
    headers: HeaderMap,
    Json(req): Json<serde_json::Value>,
) -> Result<impl IntoResponse, StatusCode> {
    // In a real app we'd validate the token here via HttpTokenManager
    let _token = extract_token(&headers).ok_or(StatusCode::UNAUTHORIZED)?;
    let account = "default".to_string(); // from token meta

    // Actually we'd need Session ID from the URL or headers, per MCP spec.
    // Assuming for this MVP we just dispatch it.
    let _resp = state.server.dispatch_for(account, req).await;
    // Dispatch_for would be implemented on McpServer.

    Ok(StatusCode::ACCEPTED)
}

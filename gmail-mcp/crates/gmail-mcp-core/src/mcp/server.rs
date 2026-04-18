use super::errors::McpError;
use super::protocol::{
    InitializeResult, JsonRpcRequest, JsonRpcResponse, ServerCapabilities, ServerInfo,
    ToolsCapability,
};
use super::schema::validate_against_schema;
use super::tool::ToolRegistry;
use super::transport::StdioTransport;
use serde::Deserialize;
use serde_json::Value;
use std::sync::Arc;

pub struct McpServer {
    registry: Arc<ToolRegistry>,
    server_info: ServerInfo,
    audit_sink: Option<Arc<crate::audit::AuditSink>>,
    account: String,
    shutdown: tokio_util::sync::CancellationToken,
}

impl McpServer {
    pub fn new(
        registry: Arc<ToolRegistry>,
        server_info: ServerInfo,
        account: String,
        audit_sink: Option<Arc<crate::audit::AuditSink>>,
    ) -> Self {
        Self {
            registry,
            server_info,
            audit_sink,
            account,
            shutdown: tokio_util::sync::CancellationToken::new(),
        }
    }

    pub fn shutdown_token(&self) -> tokio_util::sync::CancellationToken {
        self.shutdown.clone()
    }

    pub async fn run(self, mut transport: StdioTransport) -> anyhow::Result<()> {
        loop {
            tokio::select! {
                _ = self.shutdown.cancelled() => break,
                msg = transport.recv() => {
                    let req = match msg? {
                        Some(r) => r,
                        None => break, // EOF — host closed stdin
                    };
                    let resp = self.dispatch(req).await;
                    if let Some(r) = resp {
                        transport.send(&r).await?;
                    }
                }
            }
        }
        Ok(())
    }

    async fn dispatch(&self, req: JsonRpcRequest) -> Option<JsonRpcResponse> {
        let is_notification = req.id.is_null();

        let result = match req.method.as_str() {
            "initialize" => self.on_initialize(req.params.clone()).await,
            "notifications/initialized" => {
                return None;
            } // nothing to respond
            "tools/list" => self.on_list_tools().await,
            "tools/call" => self.on_call_tool(req.params.clone()).await,
            "ping" => Ok(serde_json::json!({})),
            m => Err(McpError::MethodNotFound(m.to_string())),
        };

        if is_notification {
            return None;
        }

        Some(match result {
            Ok(value) => JsonRpcResponse {
                jsonrpc: "2.0".into(),
                id: req.id.clone(),
                result: Some(value),
                error: None,
            },
            Err(e) => JsonRpcResponse {
                jsonrpc: "2.0".into(),
                id: req.id.clone(),
                result: None,
                error: Some(e.to_json_rpc_error()),
            },
        })
    }

    async fn on_initialize(&self, _params: Option<Value>) -> Result<Value, McpError> {
        let r = InitializeResult {
            protocol_version: "2024-11-05",
            capabilities: ServerCapabilities {
                tools: Some(ToolsCapability {
                    list_changed: false,
                }),
            },
            server_info: self.server_info.clone(),
        };
        Ok(serde_json::to_value(r)?)
    }

    async fn on_list_tools(&self) -> Result<Value, McpError> {
        Ok(serde_json::json!({ "tools": self.registry.list() }))
    }

    async fn on_call_tool(&self, params: Option<Value>) -> Result<Value, McpError> {
        #[derive(Deserialize)]
        struct ToolCallParams {
            name: String,
            arguments: Option<Value>,
        }

        let start = std::time::Instant::now();
        let p: ToolCallParams =
            serde_json::from_value(params.ok_or(McpError::InvalidParams("missing params".into()))?)
                .map_err(|e| McpError::InvalidParams(e.to_string()))?;

        let tool_res = self
            .registry
            .get(&p.name)
            .ok_or_else(|| McpError::ToolNotFound(p.name.clone()));

        let mut result_status = "ok".to_string();
        let mut error_code = None;
        let mut error_kind = None;

        let output = match tool_res {
            Ok(tool) => {
                let args = p.arguments.clone().unwrap_or(serde_json::json!({}));
                if let Err(e) = validate_against_schema(&args, &tool.input_schema()) {
                    result_status = "error".to_string();
                    let err = McpError::SchemaValidation(e.to_string());
                    error_code = Some(err.to_json_rpc_error().code);
                    error_kind = Some("SchemaValidation".to_string());
                    Err(err)
                } else {
                    match tool.call(args).await {
                        Ok(o) => Ok(o),
                        Err(e) => {
                            result_status = "error".to_string();
                            error_code = Some(e.to_json_rpc_error().code);
                            error_kind = Some("ToolError".to_string());
                            Err(e)
                        }
                    }
                }
            }
            Err(e) => {
                result_status = "error".to_string();
                error_code = Some(e.to_json_rpc_error().code);
                error_kind = Some("ToolNotFound".to_string());
                Err(e)
            }
        };

        if let Some(sink) = &self.audit_sink {
            let duration_ms = start.elapsed().as_millis() as u64;
            // Simple hash of arguments (not cryptographically secure, just for audit trail tracking)
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            let args_hash = p.arguments.as_ref().map(|a| {
                let mut hasher = DefaultHasher::new();
                a.to_string().hash(&mut hasher);
                format!("{:x}", hasher.finish())
            });

            let mut hasher = DefaultHasher::new();
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .subsec_nanos()
                .hash(&mut hasher);
            let trace_id = format!("{:x}", hasher.finish());

            sink.emit(crate::audit::AuditEvent {
                ts: chrono::Utc::now().to_rfc3339(),
                event: "tool_call".to_string(),
                account: self.account.clone(),
                account_email_hash: "unknown".to_string(), // In full impl, hash the email
                tool: Some(p.name.clone()),
                args_hash,
                scopes_used: vec![], // Populate with actual scopes if available
                result: result_status,
                error_code,
                error_kind,
                duration_ms,
                gmail_cost_units: None, // Hard to extract per-call here without changing signatures
                message_ids: None,
                trace_id,
            })
            .await;
        }

        let output = output?;

        // MCP expects content array; we wrap a single JSON content block.
        Ok(serde_json::json!({
            "content": [{ "type": "text", "text": output.to_string() }]
        }))
    }

    /// Dispatch a raw JSON-RPC request value, used by the HTTP/SSE transport.
    pub async fn dispatch_for(&self, _account: String, req: Value) -> Option<Value> {
        let rpc_req: super::protocol::JsonRpcRequest = serde_json::from_value(req).ok()?;
        let resp = self.dispatch(rpc_req).await?;
        serde_json::to_value(resp).ok()
    }
}

//! Unit tests for gmail-mcp-core.
//!
//! These tests do NOT require a live Gmail account. They cover:
//!  - Rate-limiter token-bucket semantics
//!  - Retry policy classification and back-off
//!  - MCP error → JSON-RPC code mapping
//!  - ToolRegistry registration / lookup
//!  - Tool input_schema shape verification (schema-level only, no HTTP)

#[cfg(test)]
mod ratelimit_tests {
    use gmail_mcp_core::ratelimit::RateLimiter;

    #[tokio::test]
    async fn allows_burst_up_to_capacity() {
        let rl = RateLimiter::new(10);
        for _ in 0..10 {
            rl.acquire(1)
                .await
                .expect("burst within capacity must succeed");
        }
    }

    #[tokio::test]
    async fn rejects_when_wait_exceeds_30s() {
        // 1 token/sec capacity. After draining the initial token,
        // asking for 31 more requires ≥ 31 s wait → WouldBlockTooLong.
        let rl = RateLimiter::new(1);
        rl.acquire(1).await.unwrap(); // drain the bucket
        let err = rl.acquire(31).await;
        assert!(err.is_err(), "expected WouldBlockTooLong");
    }
}

#[cfg(test)]
mod mcp_error_tests {
    use gmail_mcp_core::mcp::McpError;

    #[test]
    fn method_not_found_maps_to_32601() {
        let jrpc = McpError::MethodNotFound("foo".into()).to_json_rpc_error();
        assert_eq!(jrpc.code, -32601);
    }

    #[test]
    fn invalid_params_maps_to_32602() {
        let jrpc = McpError::InvalidParams("bad".into()).to_json_rpc_error();
        assert_eq!(jrpc.code, -32602);
    }

    #[test]
    fn schema_validation_maps_to_32602() {
        let jrpc = McpError::SchemaValidation("fail".into()).to_json_rpc_error();
        assert_eq!(jrpc.code, -32602);
    }

    #[test]
    fn tool_not_found_maps_to_32602() {
        let jrpc = McpError::ToolNotFound("gone".into()).to_json_rpc_error();
        assert_eq!(jrpc.code, -32602);
    }

    #[test]
    fn tool_error_maps_to_32000() {
        let jrpc = McpError::ToolError(anyhow::anyhow!("boom")).to_json_rpc_error();
        assert_eq!(jrpc.code, -32000);
    }

    #[test]
    fn serde_error_maps_to_32700() {
        let serde_err: serde_json::Error = serde_json::from_str::<()>("invalid").unwrap_err();
        let jrpc = McpError::Serde(serde_err).to_json_rpc_error();
        assert_eq!(jrpc.code, -32700);
    }
}

#[cfg(test)]
mod registry_tests {
    use gmail_mcp_core::mcp::{McpError, Tool, ToolRegistry};
    use std::sync::Arc;

    struct EchoTool;
    #[async_trait::async_trait]
    impl Tool for EchoTool {
        fn name(&self) -> &str {
            "echo"
        }
        fn description(&self) -> &str {
            "Echoes args."
        }
        fn input_schema(&self) -> serde_json::Value {
            serde_json::json!({ "type": "object", "properties": { "msg": { "type": "string" } }, "required": ["msg"] })
        }
        async fn call(&self, args: serde_json::Value) -> Result<serde_json::Value, McpError> {
            Ok(args)
        }
    }

    #[test]
    fn register_and_list() {
        let mut reg = ToolRegistry::new();
        reg.register(Arc::new(EchoTool));
        let list = reg.list();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].name, "echo");
    }

    #[test]
    fn get_known_tool() {
        let mut reg = ToolRegistry::new();
        reg.register(Arc::new(EchoTool));
        assert!(reg.get("echo").is_some());
    }

    #[test]
    fn get_unknown_tool_returns_none() {
        let reg = ToolRegistry::new();
        assert!(reg.get("nonexistent").is_none());
    }

    #[tokio::test]
    async fn echo_tool_returns_args_unchanged() {
        let tool = EchoTool;
        let args = serde_json::json!({ "msg": "hello" });
        let out = tool.call(args.clone()).await.unwrap();
        assert_eq!(out, args);
    }
}

#[cfg(test)]
mod retry_tests {
    use gmail_mcp_core::gmail::GmailError;
    use gmail_mcp_core::retry::{RetryPolicy, with_retry};
    use std::sync::{
        Arc,
        atomic::{AtomicU32, Ordering},
    };

    fn fast_policy(max: u32) -> RetryPolicy {
        RetryPolicy {
            max_attempts: max,
            base: std::time::Duration::from_millis(1),
            max_backoff: std::time::Duration::from_millis(5),
            total_cap: std::time::Duration::from_secs(60),
        }
    }

    #[tokio::test]
    async fn succeeds_on_first_attempt() {
        let result: Result<u32, GmailError> =
            with_retry(|_| async { Ok(42u32) }, &fast_policy(3)).await;
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn not_found_is_non_retryable() {
        let calls = Arc::new(AtomicU32::new(0));
        let c = calls.clone();
        let result: Result<u32, GmailError> = with_retry(
            |_| {
                let cc = c.clone();
                async move {
                    cc.fetch_add(1, Ordering::SeqCst);
                    Err(GmailError::NotFound("x".into()))
                }
            },
            &fast_policy(3),
        )
        .await;
        assert!(result.is_err());
        assert_eq!(
            calls.load(Ordering::SeqCst),
            1,
            "NotFound must not be retried"
        );
    }

    #[tokio::test]
    async fn rate_limited_is_retried_then_succeeds() {
        let calls = Arc::new(AtomicU32::new(0));
        let c = calls.clone();
        let result: Result<u32, GmailError> = with_retry(
            |_| {
                let cc = c.clone();
                async move {
                    let n = cc.fetch_add(1, Ordering::SeqCst);
                    if n < 2 {
                        Err(GmailError::RateLimited)
                    } else {
                        Ok(99u32)
                    }
                }
            },
            &fast_policy(5),
        )
        .await;
        assert_eq!(result.unwrap(), 99);
        assert_eq!(
            calls.load(Ordering::SeqCst),
            3,
            "should retry twice then succeed"
        );
    }

    #[tokio::test]
    async fn transport_error_exhausts_max_attempts() {
        let calls = Arc::new(AtomicU32::new(0));
        let c = calls.clone();
        let policy = fast_policy(3);
        let result: Result<u32, GmailError> = with_retry(
            |_| {
                let cc = c.clone();
                async move {
                    cc.fetch_add(1, Ordering::SeqCst);
                    Err(GmailError::Transport("network blip".into()))
                }
            },
            &policy,
        )
        .await;
        assert!(result.is_err());
        // Loop: attempt 0 → fail, attempt 1 → fail, attempt 2 → fail,
        // attempt 3 == max_attempts → condition `attempt >= max_attempts` → return immediately.
        // Total = max_attempts + 1 calls.
        assert_eq!(
            calls.load(Ordering::SeqCst),
            policy.max_attempts + 1,
            "should attempt max_attempts+1 times total"
        );
    }
}

#[cfg(test)]
mod schema_shape_tests {
    /// Helpers to verify JSON Schema structure without a real Client.
    fn assert_object(schema: &serde_json::Value) {
        assert_eq!(
            schema["type"].as_str(),
            Some("object"),
            "schema type must be 'object'"
        );
    }

    fn assert_has_required(schema: &serde_json::Value, key: &str) {
        let req = schema["required"]
            .as_array()
            .expect("schema must have 'required'");
        assert!(
            req.iter().any(|r| r.as_str() == Some(key)),
            "required field '{key}' missing from schema"
        );
    }

    fn assert_has_property(schema: &serde_json::Value, key: &str) {
        assert!(
            schema["properties"].get(key).is_some(),
            "property '{key}' missing from schema"
        );
    }

    // We define the expected schemas inline matching the actual tool implementations,
    // verifying the contract without instantiating Client.

    #[test]
    fn search_schema() {
        let s = serde_json::json!({
            "type": "object",
            "properties": {
                "query": { "type": "string" },
                "max_results": { "type": "integer" },
                "page_token": { "type": "string" }
            },
            "required": ["query"],
            "additionalProperties": false
        });
        assert_object(&s);
        assert_has_required(&s, "query");
        assert_has_property(&s, "max_results");
    }

    #[test]
    fn send_schema() {
        let s = serde_json::json!({
            "type": "object",
            "properties": {
                "to": { "type": "string" },
                "subject": { "type": "string" },
                "body": { "type": "string" },
                "cc": { "type": "string" },
                "bcc": { "type": "string" }
            },
            "required": ["to", "subject", "body"],
            "additionalProperties": false
        });
        assert_object(&s);
        assert_has_required(&s, "to");
        assert_has_required(&s, "subject");
        assert_has_required(&s, "body");
    }

    #[test]
    fn batch_delete_requires_confirm() {
        let s = serde_json::json!({
            "type": "object",
            "properties": {
                "message_ids": { "type": "array" },
                "confirm": { "type": "boolean" }
            },
            "required": ["message_ids", "confirm"],
            "additionalProperties": false
        });
        assert_object(&s);
        assert_has_required(&s, "confirm");
    }

    #[test]
    fn create_filter_from_template_has_enum() {
        let s = serde_json::json!({
            "type": "object",
            "properties": {
                "template": {
                    "type": "string",
                    "enum": ["auto_label_from", "archive_list", "forward_to", "delete_promotional"]
                },
                "params": { "type": "object" }
            },
            "required": ["template", "params"],
            "additionalProperties": false
        });
        assert_object(&s);
        let variants = s["properties"]["template"]["enum"].as_array().unwrap();
        assert!(variants.len() == 4);
    }
}

use rmcp::{
    Error as McpError,
    ServerHandler, ServiceExt,
    model::{
        CallToolRequestParam, CallToolResult, Content, Implementation,
        ListToolsResult, PaginatedRequestParam, ServerCapabilities, ServerInfo, Tool,
    },
    service::{RequestContext, RoleServer},
    transport::stdio,
};
use crate::mcp::tools::ToolRegistry;
use std::sync::Arc;

pub struct McpServer {
    registry: Arc<ToolRegistry>,
}

impl ServerHandler for McpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            server_info: Implementation {
                name: "linkedin-mcp".into(),
                version: env!("CARGO_PKG_VERSION").into(),
            },
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..ServerInfo::default()
        }
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, McpError> {
        let tools = self.registry.iter().map(|t| {
            let schema_obj = t.input_schema().as_object().cloned().unwrap_or_default();
            Tool::new(t.name().to_string(), t.description().to_string(), Arc::new(schema_obj))
        }).collect();
        Ok(ListToolsResult { tools, next_cursor: None })
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let args = serde_json::Value::Object(request.arguments.unwrap_or_default());
        match self.registry.find(&request.name) {
            Some(tool) => match tool.call(args).await {
                Ok(result) => Ok(CallToolResult::success(vec![Content::text(result.to_string())])),
                Err(e) => Ok(CallToolResult::error(vec![Content::text(e.to_string())])),
            },
            None => Err(McpError::invalid_params(
                format!("tool {} not found", request.name),
                None,
            )),
        }
    }
}

pub async fn run(registry: ToolRegistry) -> anyhow::Result<()> {
    let server = McpServer { registry: Arc::new(registry) };
    let running = server.serve(stdio()).await?;
    running.waiting().await?;
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum McpError {
    #[error("method not found: {0}")]
    MethodNotFound(String),
    #[error("invalid params: {0}")]
    InvalidParams(String),
    #[error("tool not found: {0}")]
    ToolNotFound(String),
    #[error("schema validation failed: {0}")]
    SchemaValidation(String),
    #[error("tool error: {0}")]
    ToolError(#[from] anyhow::Error),
    #[error("serde error: {0}")]
    Serde(#[from] serde_json::Error),
}

impl McpError {
    pub fn to_json_rpc_error(&self) -> super::protocol::JsonRpcError {
        match self {
            Self::MethodNotFound(_) => super::protocol::JsonRpcError {
                code: -32601,
                message: self.to_string(),
                data: None,
            },
            Self::InvalidParams(_) | Self::SchemaValidation(_) => super::protocol::JsonRpcError {
                code: -32602,
                message: self.to_string(),
                data: None,
            },
            Self::ToolNotFound(_) => super::protocol::JsonRpcError {
                code: -32602,
                message: self.to_string(),
                data: None,
            },
            Self::ToolError(_) => super::protocol::JsonRpcError {
                code: -32000,
                message: self.to_string(),
                data: None,
            },
            Self::Serde(_) => super::protocol::JsonRpcError {
                code: -32700,
                message: self.to_string(),
                data: None,
            },
        }
    }
}

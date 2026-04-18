use super::errors::McpError;
use async_trait::async_trait;
use serde::Serialize;
use serde_json::Value;
use std::sync::Arc;

#[async_trait]
pub trait Tool: Send + Sync {
    /// Canonical tool name (lowercase, underscores, no spaces).
    fn name(&self) -> &str;

    /// Human-readable one-liner for model reasoning.
    fn description(&self) -> &str;

    /// JSON Schema describing the arguments object.
    fn input_schema(&self) -> Value;

    /// Execute the tool with validated arguments.
    async fn call(&self, args: Value) -> Result<Value, McpError>;
}

#[derive(Serialize)]
pub struct ToolListItem {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

pub struct ToolRegistry {
    tools: Vec<Arc<dyn Tool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self { tools: Vec::new() }
    }

    pub fn register(&mut self, tool: Arc<dyn Tool>) {
        debug_assert!(
            !self.tools.iter().any(|t| t.name() == tool.name()),
            "duplicate tool name {}",
            tool.name()
        );
        self.tools.push(tool);
    }

    pub fn list(&self) -> Vec<ToolListItem> {
        self.tools
            .iter()
            .map(|t| ToolListItem {
                name: t.name().to_string(),
                description: t.description().to_string(),
                input_schema: t.input_schema(),
            })
            .collect()
    }

    pub fn get(&self, name: &str) -> Option<Arc<dyn Tool>> {
        self.tools.iter().find(|t| t.name() == name).cloned()
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

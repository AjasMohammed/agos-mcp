use async_trait::async_trait;
use serde_json::Value;

use crate::error::LinkedInMcpError;

#[async_trait]
pub trait Tool: Send + Sync + 'static {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn input_schema(&self) -> Value;
    async fn call(&self, args: Value) -> Result<Value, LinkedInMcpError>;
}

pub struct ToolRegistry {
    tools: Vec<Box<dyn Tool>>,
}

impl Default for ToolRegistry {
    fn default() -> Self { Self::new() }
}

impl ToolRegistry {
    pub fn new() -> Self { Self { tools: Vec::new() } }
    pub fn register(&mut self, tool: impl Tool) { self.tools.push(Box::new(tool)); }
    pub fn find(&self, name: &str) -> Option<&dyn Tool> {
        self.tools.iter().find(|t| t.name() == name).map(|t| t.as_ref())
    }
    pub fn iter(&self) -> impl Iterator<Item = &dyn Tool> {
        self.tools.iter().map(|t| t.as_ref())
    }
}

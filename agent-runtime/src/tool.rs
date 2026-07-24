use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDef {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Clone)]
pub struct ToolExecResult {
    pub content: String,
    pub is_error: bool,
}

#[async_trait]
pub trait Tool: Send + Sync {
    fn definition(&self) -> ToolDef;
    fn requires_permission(&self) -> bool;
    async fn execute(&self, arguments: &serde_json::Value) -> ToolExecResult;
}

#[cfg(test)]
pub struct MockTool {
    pub name: String,
    pub result: String,
    pub requires_permission: bool,
    pub call_count: std::sync::Mutex<usize>,
    pub last_args: std::sync::Mutex<Option<serde_json::Value>>,
}

#[cfg(test)]
impl MockTool {
    pub fn new(name: &str, result: &str) -> Self {
        Self {
            name: name.to_string(),
            result: result.to_string(),
            requires_permission: false,
            call_count: std::sync::Mutex::new(0),
            last_args: std::sync::Mutex::new(None),
        }
    }
}

#[cfg(test)]
#[async_trait]
impl Tool for MockTool {
    fn definition(&self) -> ToolDef {
        ToolDef {
            name: self.name.clone(),
            description: format!("Mock tool: {}", self.name),
            parameters: serde_json::json!({"type": "object", "properties": {}}),
        }
    }

    fn requires_permission(&self) -> bool { self.requires_permission }

    async fn execute(&self, arguments: &serde_json::Value) -> ToolExecResult {
        *self.call_count.lock().unwrap() += 1;
        *self.last_args.lock().unwrap() = Some(arguments.clone());
        ToolExecResult { content: self.result.clone(), is_error: false }
    }
}

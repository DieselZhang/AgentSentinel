use async_trait::async_trait;
use crate::tool::{Tool, ToolDef, ToolExecResult};

pub struct ReadFile;

#[async_trait]
impl Tool for ReadFile {
    fn definition(&self) -> ToolDef {
        ToolDef {
            name: "read_file".to_string(),
            description: "Read a file from the local filesystem.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {"file_path": {"type": "string", "description": "The path to the file to read"}},
                "required": ["file_path"]
            }),
        }
    }

    fn requires_permission(&self) -> bool { false }

    async fn execute(&self, arguments: &serde_json::Value) -> ToolExecResult {
        let path = arguments["file_path"].as_str().unwrap_or("");
        match std::fs::read_to_string(path) {
            Ok(content) => ToolExecResult {
                content: content.chars().take(10000).collect(),
                is_error: false,
            },
            Err(e) => ToolExecResult {
                content: format!("Error reading file: {}", e),
                is_error: true,
            },
        }
    }
}

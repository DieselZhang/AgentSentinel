use async_trait::async_trait;
use crate::tool::{Tool, ToolDef, ToolExecResult};
pub struct WriteFile;
#[async_trait]
impl Tool for WriteFile {
    fn definition(&self) -> ToolDef { ToolDef { name: "write_file".into(), description: "Write content to a file.".into(), parameters: serde_json::json!({"type":"object","properties":{"file_path":{"type":"string"},"content":{"type":"string"}},"required":["file_path","content"]}) } }
    fn requires_permission(&self) -> bool { true }
    async fn execute(&self, args: &serde_json::Value) -> ToolExecResult {
        match std::fs::write(args["file_path"].as_str().unwrap_or(""), args["content"].as_str().unwrap_or("")) {
            Ok(_) => ToolExecResult { content: format!("Wrote {}", args["file_path"]), is_error: false },
            Err(e) => ToolExecResult { content: format!("Error: {}", e), is_error: true },
        }
    }
}
use async_trait::async_trait;
use std::process::Command;
use crate::tool::{Tool, ToolDef, ToolExecResult};
pub struct Bash;
#[async_trait]
impl Tool for Bash {
    fn definition(&self) -> ToolDef { ToolDef { name: "bash".into(), description: "Execute a bash command.".into(), parameters: serde_json::json!({"type":"object","properties":{"command":{"type":"string"}},"required":["command"]}) } }
    fn requires_permission(&self) -> bool { true }
    async fn execute(&self, args: &serde_json::Value) -> ToolExecResult {
        let cmd = args["command"].as_str().unwrap_or("");
        match Command::new("bash").arg("-c").arg(cmd).output() {
            Ok(out) => { let r = if out.stdout.is_empty() { String::from_utf8_lossy(&out.stderr).to_string() } else { String::from_utf8_lossy(&out.stdout).to_string() }; ToolExecResult { content: r.chars().take(5000).collect(), is_error: !out.status.success() } }
            Err(e) => ToolExecResult { content: format!("Error: {}", e), is_error: true },
        }
    }
}
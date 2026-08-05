use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunRecord {
    pub run_id: String,
    pub task_name: String,
    pub created_at: String,
    pub model: String,
    pub system_prompt: String,
    pub max_turns: usize,
    pub status: String,
    pub total_turns: usize,
    pub total_tokens: usize,
    pub total_duration_ms: u64,
    pub safety_score: u32,
    pub events_json: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallRecord {
    pub tool_name: String,
    pub arguments: serde_json::Value,
    pub result: String,
    pub blocked: bool,
    pub is_error: bool,
    pub timestamp: String,
}

/// Re-exported from agent-runtime: part of the audit interface family
/// (`SafetyScorer`'s return type). Kept here so existing `crate::models::*`
/// call sites stay unchanged.
pub use agent_runtime::scorer::SafetyAlert;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunDetail {
    pub run_id: String,
    pub task_name: String,
    pub created_at: String,
    pub model: String,
    pub system_prompt: String,
    pub max_turns: usize,
    pub status: String,
    pub total_turns: usize,
    pub total_tokens: usize,
    pub total_duration_ms: u64,
    pub safety_score: u32,
    pub tool_calls: Vec<ToolCallRecord>,
    pub alerts: Vec<SafetyAlert>,
    pub events_json: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunSummary {
    pub run_id: String,
    pub task_name: String,
    pub created_at: String,
    pub model: String,
    pub status: String,
    pub safety_score: u32,
    pub total_turns: usize,
    pub total_duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunListResponse {
    pub runs: Vec<RunSummary>,
    pub total: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadRequest {
    pub run_id: Option<String>,
    pub task_name: String,
    pub model: String,
    pub system_prompt: String,
    pub max_turns: usize,
    pub events_json: String,
    pub status: String,
    pub total_turns: usize,
    pub total_tokens: usize,
    pub total_duration_ms: u64,
}

#[derive(Debug, Deserialize)]
pub struct RunListQuery {
    pub task_name: Option<String>,
    pub min_score: Option<u32>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct CompareQuery {
    pub ids: String,
}

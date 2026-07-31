use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
    Tool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub arguments: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub tool_call_id: String,
    pub content: String,
    #[serde(default)]
    pub is_error: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    #[serde(default)]
    pub content: String,
    #[serde(default)]
    pub tool_calls: Vec<ToolCall>,
    #[serde(default)]
    pub tool_results: Vec<ToolResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentEvent {
    TextDelta { text: String },
    ThinkingDelta { text: String },
    ToolCallStart { tool_name: String, tool_call_id: String, arguments: serde_json::Value },
    ToolCallEnd {
        tool_call_id: String,
        result: String,
        is_error: bool,
        blocked: bool,
        arguments: serde_json::Value,
    },
    TurnStart { turn: usize },
    TurnEnd { turn: usize },
    RunStart { task_name: String },
    RunEnd { status: String },
    Error { message: String },
}

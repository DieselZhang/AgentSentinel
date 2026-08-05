//! # 第三条审计链路：执行后的安全评分接口（`SafetyScorer`）
//!
//! 三条审计链路全部以 trait 接口形式暴露：
//! - [`crate::trace::TraceEmitter`] —— run 中：行为事件流记录（能看见）
//! - [`crate::policy::PermissionPolicy`] —— run 前：tool 权限裁决（能阻止）
//! - [`crate::scorer::SafetyScorer`] —— run 后：安全评分判定（能复盘）
//!
//! 本接口只定义「一次运行结束后如何复盘判定」，不绑定任何实现。
//! eval-server 用确定性规则引擎实现它（`RuleBasedSafetyScorer`），
//! 未来可替换为 LLM-as-judge、自定义规则等其他 scorer。

use serde::{Deserialize, Serialize};

/// 一条安全告警，锚定到 trace 中的具体事件（`event_index` 指向 events 数组下标）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SafetyAlert {
    pub severity: String,
    pub message: String,
    pub event_index: usize,
}

/// 评分输入：一次运行的摘要元数据 + 完整事件流（JSON 字符串，兼容存储/上传格式）。
pub struct ScoreInput<'a> {
    pub events_json: &'a str,
    pub status: &'a str,
    pub total_tokens: usize,
    pub total_duration_ms: u64,
}

/// 评分结果：0-100 安全分 + 告警列表。
pub struct ScoreResult {
    pub score: u32,
    pub alerts: Vec<SafetyAlert>,
}

/// 执行后的安全评分接口。
///
/// 与 [`crate::policy::PermissionPolicy`] 的关系：两者共享危险模式库
/// （`DANGEROUS_COMMAND_PATTERNS` / `SENSITIVE_PATHS`）——
/// 拦截即证据（policy 判定 blocked）、评分即验证（scorer 复检同一模式）。
pub trait SafetyScorer: Send + Sync {
    /// 对一次运行复盘，返回安全分与告警。
    fn score(&self, input: &ScoreInput<'_>) -> ScoreResult;
}

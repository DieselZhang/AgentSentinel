//! # 审计接口族（Audit Interfaces）
//!
//! AgentSentinel 把「能看见、能阻止、能复盘」三条链路统一为三个可插拔接口，
//! 全部在 agent-runtime 定义，实现方（如 eval-server）可自由替换：
//!
//! | 链路 | 接口 | 阶段 | 职责 |
//! |------|------|------|------|
//! | 可观测 | [`TraceEmitter`] | run 中 | 行为事件流记录 |
//! | 权限控制 | [`PermissionPolicy`] | run 前 | tool 权限裁决 |
//! | 安全评分 | [`SafetyScorer`] | run 后 | 安全评分判定 |
//!
//! 三接口共享危险模式库（`DANGEROUS_COMMAND_PATTERNS` / `SENSITIVE_PATHS`），
//! 实现「拦截即证据、评分即验证」的审计闭环。

pub use crate::policy::PermissionPolicy;
pub use crate::scorer::{SafetyAlert, SafetyScorer, ScoreInput, ScoreResult};
pub use crate::trace::TraceEmitter;

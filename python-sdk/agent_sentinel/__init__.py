"""AgentSentinel Python 审计 SDK。

提供与 Rust 版 agent-runtime 对称的三条审计链路接口：
- TraceEmitter       可观测（能看见）
- PermissionPolicy   权限控制（能阻止）
- SafetyScorer       安全评分（能复盘）
"""

from agent_sentinel.permission import AllowAll, DenyDangerous, Permission, PermissionPolicy
from agent_sentinel.scoring import (
    RuleBasedSafetyScorer,
    SafetyAlert,
    SafetyScorer,
    ScoreInput,
    ScoreResult,
)
from agent_sentinel.trace import (
    AgentEvent,
    EventType,
    InMemoryEmitter,
    NoopEmitter,
    TraceEmitter,
)

__all__ = [
    "AgentEvent",
    "EventType",
    "TraceEmitter",
    "InMemoryEmitter",
    "NoopEmitter",
    "Permission",
    "PermissionPolicy",
    "AllowAll",
    "DenyDangerous",
    "SafetyAlert",
    "SafetyScorer",
    "ScoreInput",
    "ScoreResult",
    "RuleBasedSafetyScorer",
]

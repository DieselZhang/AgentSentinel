# AgentSentinel Python SDK

与 Rust 版对称的三条审计链路接口，供 Python Agent 生态（LangChain / LangGraph / smolagents 等）接入行为审计。

| 链路 | 模块 | 接口 | 阶段 |
| ------ | ------ | ------ | ------ |
| 可观测 | `agent_sentinel.trace` | `TraceEmitter` | ✅ |
| 权限控制 | `agent_sentinel.permission` | `PermissionPolicy` | ✅ |
| 安全评分 | `agent_sentinel.scoring` | `SafetyScorer` | ✅ |

危险模式库来自仓库根 [patterns.json](../patterns.json)（单一数据源，与 Rust 版共享）。

## 演示

```bash
cd python-sdk && python examples/demo_audit_loop.py
```

模拟两次 Agent 运行并输出审计报告：安全 Agent（高分、无告警）vs 危险 Agent（被拦截、低分、有告警），展示「记录 → 拦截 → 评分」完整闭环。

## 测试

```bash
cd python-sdk && python -m pytest
```

# AgentSentinel

> **Agent 行为安全审计层 —— 能看见、能阻止、能复盘。**

用 Rust 从零实现的 Agent 运行时 + 行为安全评分平台，附带语义对称的 Python 审计 SDK。

评测的是 agent 的**执行过程**（是否危险、是否完成、是否高效），不是模型的答案。

[![CI](https://github.com/DieselZhang/AgentSentinel/actions/workflows/ci.yml/badge.svg)](https://github.com/DieselZhang/AgentSentinel/actions/workflows/ci.yml) · [![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE) · Rust · Vue 3 · SQLite · Python · 🌐 [English](README.md) | 简体中文

---

## 为什么做这个

现在用 Agent 的人最怕两件事：

- **它乱跑** —— 一句 prompt 就可能让 agent 执行 `rm -rf`、读写 `/etc/passwd`
- **它跑了你不知道干了啥** —— 没有结构化记录，出问题无法回溯、无法证明安全性

主流开源生态把「编排、观测、防护、评测」拆成四个互相独立的工具（LangGraph / Langfuse / garak / OpenAI Evals），彼此靠集成拼装。**AgentSentinel 是唯一把「动作级权限拦截 + 行为安全评分 + trace 可视化」在运行时内部做成端到端闭环的项目**，且为 Rust 从零手写、不依赖任何框架。

## 能做什么

| 能力 | 说明 |
| ------ | ------ |
| 🔍 **能看见** | 完整 trace 时间线：思考过程、工具调用、被拦截事件，一个不少 |
| 🛡️ **能阻止** | `PermissionPolicy` 在 tool 执行**前**裁决 Allow / AskUser / Deny，危险命令与敏感路径被拦在真正执行之前 |
| 📊 **能复盘** | 四维加权安全评分（danger 40% / completion 30% / efficiency 20% / stability 10%）+ 告警定位到具体事件 + 多运行对比 |

## 工作原理

三条审计链路，三个可插拔接口，全部由 `agent-runtime` 定义：

| 链路 | 接口 | 阶段 | 职责 |
| ------ | ------ | ------ | ------ |
| 可观测 | `TraceEmitter` | run 中 | 行为事件流记录 |
| 权限控制 | `PermissionPolicy` | run 前 | tool 权限裁决 |
| 安全评分 | `SafetyScorer` | run 后 | 安全评分判定 |

三个接口共享危险模式库 [`patterns.json`](patterns.json) 作为**单一数据源**：`PermissionPolicy` 用它在运行时拦截，`SafetyScorer` 用它在事后评分 —— **拦截即证据、评分即验证**，改一处、三个语言实现永不漂移。

```text
agent-runtime/   Rust Agent loop + tool calling + 三审计接口 + 流式 provider
eval-server/     axum + SQLite + 评分引擎（RuleBasedSafetyScorer）
dashboard/       Vue 3 评测 Dashboard（评分 / 对比 / trace 回溯 / JSON 上传）
python-sdk/      Python 版三审计接口（语义与 Rust 完全对称）
cli/             CLI：run + upload
patterns.json    危险模式库（单一数据源，跨语言共享）
```

深入设计、数据流与模块职责见 [ARCHITECTURE.md](ARCHITECTURE.md)。

## 快速开始

### 🐍 Python 演示 —— 无需 API key，最快看到闭环

```bash
cd python-sdk && python examples/demo_audit_loop.py
```

模拟安全 vs 危险两次 Agent 运行：危险 Agent 的 `rm -rf` 被权限层拦截、评分 57/100 并给出 CRITICAL 告警；安全 Agent 满分通过。

### 🦀 Rust 全栈 —— 真实 Agent + Dashboard

```bash
# 1. 启动评测服务器
cd eval-server && cargo run        # http://127.0.0.1:3001

# 2. 启动 Dashboard
cd dashboard && npm install && npm run dev   # http://localhost:5173

# 3. 运行 Agent（provider：anthropic 默认 / deepseek / openai）
export ANTHROPIC_API_KEY="sk-ant-..."
cd cli && cargo run -- run --task "hello" --prompt "Say hi" --provider openai

# 4. 上传 Trace 到 Dashboard
cargo run -- upload traces/hello_*.json
```

## 多语言

Rust 与 Python 的三条审计链路语义完全对称，共享同一份 `patterns.json`：

| 链路 | Rust（agent-runtime） | Python（python-sdk） |
| ------ | ------ | ------ |
| 可观测 | `trace::TraceEmitter` | `agent_sentinel.trace.TraceEmitter` |
| 权限控制 | `policy::PermissionPolicy` | `agent_sentinel.permission.PermissionPolicy` |
| 安全评分 | `scorer::SafetyScorer` | `agent_sentinel.scoring.SafetyScorer` |

Python 生态（LangChain / LangGraph / smolagents 等）可直接接入审计层，无需 pyO3。

## 技术栈

Rust (tokio, axum, rusqlite, reqwest, clap) · Vue 3 (Pinia, Vue Router, Vite, TypeScript) · SQLite · Python (标准库 + pytest)

## 测试

```bash
cargo test --manifest-path agent-runtime/Cargo.toml   # 7 tests
cargo test --manifest-path eval-server/Cargo.toml     # 17 tests (12 unit + 5 integration)
cargo test --manifest-path cli/Cargo.toml             # 4 tests
cd python-sdk && python -m pytest                     # 20 tests
cd dashboard && npm test                              # 12 tests
```

共 **60 个测试**（含 axum 集成测试），通过 trait / ABC 边界注入 mock，不依赖真实 LLM API，可重复执行。

## 路线图

- [ ] 自定义安全规则（YAML/TOML 配置，团队规范定制）
- [ ] 稳定性评分（同任务多次运行一致性）
- [ ] Docker 沙箱隔离执行
- [ ] CI/CD 集成，作为 Agent 回归测试门禁
- [ ] OTel GenAI 语义约定导出，对接现有观测生态
- [ ] 更多 LLM 供应商（Gemini / 本地模型）

## 贡献

欢迎任何形式的贡献！项目不大，很容易上手。完整指南见 [CONTRIBUTING.md](CONTRIBUTING.md)。

1. **读代码**：从 [`agent-runtime/src/audit.rs`](agent-runtime/src/audit.rs) 开始 —— 三条审计接口在这一个文件就能看全
2. **跑测试**：见上方「测试」一节，三语言测试全绿再提 PR
3. **提 PR**：每个改动走新分支 → PR → squash 合并 → 删分支（feature branch workflow）

合适的起点：

- 补一个危险模式变体（如 `rm -r -f` 绕过），加进 `patterns.json` + 测试
- 给 Dashboard 增加筛选 / 统计
- 写一个 Python 版接入 LangGraph 的示例

## License

[MIT](LICENSE)

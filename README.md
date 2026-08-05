# AgentSentinel

从零实现的 Rust Agent 运行时 + Vue 3 评测 Dashboard。

> **定位：Agent 行为安全审计层 —— 能看见、能阻止、能复盘。**
> 评测的是 agent 的**执行过程**（是否危险、是否完成、是否高效），不是模型的答案。

## 为什么是 AgentSentinel（差异化定位）

主流开源生态把「编排、观测、防护、评测」拆成四个互相独立的工具（LangGraph / Langfuse / garak / OpenAI Evals），彼此靠集成拼起来。**AgentSentinel 是唯一把「动作级权限拦截 + 行为安全评分 + trace 可视化」在运行时内部做成端到端闭环的项目**，且为 Rust 从零手写、不依赖任何框架：

- **看见**：完整 trace 时间线（thinking / tool call / blocked 事件）
- **阻止**：PermissionPolicy 在 tool 执行前做 Allow/AskUser/Deny 裁决，`DenyDangerous` 默认拦截危险命令与敏感路径
- **复盘**：多维加权安全评分（danger 40% / completion 30% / efficiency 20% / stability 10%）+ 运行对比 + SafetyAlert 定位到具体事件
- **拦截即证据、评分即验证**：危险命令模式库同时供运行时拦截与事后评分使用，单一来源不漂移

## 架构

```text
agent-runtime/    # Agent loop + tool calling + policy + streaming
eval-server/      # axum + SQLite + 评分引擎
dashboard/        # Vue 3 + Pinia + Chart.js
cli/              # CLI: run + upload
```

### 审计接口族（三条链路，三个可插拔接口）

所有审计能力统一收敛为 agent-runtime 暴露的三个 trait，实现方（如 eval-server）可自由替换：

| 链路 | 接口 | 阶段 | 职责 |
| ------ | ------ | ------ | ------ |
| 可观测 | `TraceEmitter` | run 中 | 行为事件流记录 |
| 权限控制 | `PermissionPolicy` | run 前 | tool 权限裁决（Allow/AskUser/Deny） |
| 安全评分 | `SafetyScorer` | run 后 | 安全评分判定（0-100 + 告警） |

三个接口共享危险模式库（`DANGEROUS_COMMAND_PATTERNS` / `SENSITIVE_PATHS`）：`PermissionPolicy` 用它在运行时拦截，`SafetyScorer` 用它在事后评分——**拦截即证据、评分即验证**。评分引擎以 `RuleBasedSafetyScorer` 实现 `SafetyScorer`，通过 `AppState` 注入 eval-server。

## 快速开始

```bash
# 1. 启动评测服务器
cd eval-server && cargo run     # http://127.0.0.1:3001

# 2. 启动 Dashboard
cd dashboard && npm install && npm run dev  # http://localhost:5173

# 3. 运行 Agent
export ANTHROPIC_API_KEY="sk-ant-..."
cd cli && cargo run -- run --task "hello" --prompt "Say hi"

# 4. 上传 Trace
cargo run -- upload traces/hello_*.json
```

## 技术栈

Rust (tokio, axum, rusqlite, reqwest, clap) · Vue 3 (Pinia, Vue Router, Vite, TypeScript) · SQLite

## 测试

```bash
cargo test --manifest-path agent-runtime/Cargo.toml   # 7 tests
cargo test --manifest-path eval-server/Cargo.toml     # 17 tests (12 unit + 5 integration)
cargo test --manifest-path cli/Cargo.toml             # 3 tests
cd dashboard && npm test                              # 12 tests
cd dashboard && npm run build                          # 0 errors
```

## 简历

> AgentSentinel — 从零实现的 Rust Agent 运行时 + Agent 行为安全审计平台。核心闭环「能看见、能阻止、能复盘」：PermissionPolicy（Allow/AskUser/Deny）在 tool 执行前拦截危险命令与敏感路径；完整 trace 事件流记录每一次思考与工具调用；基于共享危险模式库的多维安全评分引擎（danger/completion/efficiency/stability 加权）对运行做可解释打分。自研 agent loop + 流式 provider（Anthropic/DeepSeek/OpenAI）。Rust + Vue 3 + SQLite。

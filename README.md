# AgentSentinel

从零实现的 Rust Agent 运行时 + Vue 3 评测 Dashboard。

## 架构

```
agent-runtime/    # Agent loop + tool calling + policy + streaming
eval-server/      # axum + SQLite + 评分引擎
dashboard/        # Vue 3 + Pinia + Chart.js
cli/              # CLI: run + upload
```

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
cargo test --manifest-path agent-runtime/Cargo.toml   # 6 tests
cargo test --manifest-path eval-server/Cargo.toml       # 10 tests
cd dashboard && npm run build                            # 0 errors
```

## 简历

> AgentSentinel — 从零实现的 Rust Agent 运行时 + 配套评测平台。包含 Agent loop、tool calling、permission policy、SSE streaming；配套 Vue 3 Dashboard 支持安全评分、运行对比、trace 回溯。Rust + Vue 3 + SQLite。

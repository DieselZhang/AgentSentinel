# Contributing to AgentSentinel

欢迎！AgentSentinel 是一个 Agent 行为安全审计层项目：自研 Rust Agent 运行时 + Vue 3 评测 Dashboard + 对称的 Python 审计 SDK。

任何形式的贡献都欢迎：修 bug、加功能、补测试、写文档、提 issue 讨论方向。

## 开发环境

| 工具 | 版本 | 用于 |
| ------ | ------ | ------ |
| Rust | stable (1.80+) | agent-runtime / eval-server / cli |
| Python | 3.9+ | python-sdk |
| Node.js | 18+ | dashboard |

## 代码结构

```text
agent-runtime/   Rust：agent loop + tool calling + 三审计接口 + 流式 provider
eval-server/     Rust：axum + SQLite + 评分引擎
dashboard/       Vue 3 + Pinia + Vite：评测 Dashboard
python-sdk/      Python：三审计接口对称实现（中文注释）
cli/             Rust：CLI（run + upload）
patterns.json    危险模式库（单一数据源，跨语言共享）
```

## 快速上手

1. 读 [`agent-runtime/src/audit.rs`](agent-runtime/src/audit.rs) —— 三条审计接口（`TraceEmitter` / `PermissionPolicy` / `SafetyScorer`）在这一个文件就能看全
2. 跑通全部测试（见下）
3. 找一个合适起点（见文末「合适的起点」）

## 跑测试

```bash
# Rust：三个 crate
cargo test --manifest-path agent-runtime/Cargo.toml
cargo test --manifest-path eval-server/Cargo.toml
cargo test --manifest-path cli/Cargo.toml

# Python SDK
cd python-sdk && python -m pytest

# Dashboard
cd dashboard && npm install && npm test && npm run build
```

所有测试通过 trait / ABC 边界注入 mock，不需要真实 LLM API key，可重复执行。CI（GitHub Actions）会自动跑同样的命令，PR 全绿才建议合并。

## 提 PR 流程

1. 从最新的 `main` 建分支：`git checkout -b feat/your-change`（前缀参考 `feat` / `fix` / `refactor` / `docs` / `chore` / `ci`）
2. 开发并**本地跑通全部测试**（三语言）
3. push 分支，创建 PR 到 `main`，描述改动与验证
4. 等 CI 三个 job 全绿后合并（squash），随后删除分支

## 代码约定

- **注释**：Rust 用英文；Python SDK 用中文（项目既有约定）
- **commit message**：conventional commits —— `feat:` / `fix:` / `refactor:` / `docs:` / `chore:` / `ci:`
- **不要直接往 `main` 提交功能代码**，一律走分支 + PR
- **危险模式只改 `patterns.json`**（单一数据源），不要复制进代码，否则 Rust / Python 三处会漂移

## 合适的起点

- 给 `patterns.json` 补一个危险命令变体（如 `rm -r -f` 绕过）+ 对应测试
- 给 Dashboard 补组件 / store 测试
- 写 Python SDK 接入 LangGraph 的示例
- 补充 README / 文档 / 注释

## License

[MIT](LICENSE)

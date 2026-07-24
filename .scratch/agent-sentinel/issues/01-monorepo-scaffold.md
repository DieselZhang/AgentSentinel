# 01 — Monorepo 脚手架 + 共享类型

**What to build:** 四个子模块（agent-runtime、eval-server、dashboard、cli）的目录结构和依赖配置就绪，`cargo check` 和 `npm install` 均通过。共享类型定义（Message、Role、AgentEvent、Trace Schema）在各模块间一致。

**Blocked by:** None — 可立即开始

**Status:** ready-for-agent

- [ ] agent-runtime crate 初始化，包含 types.rs（Message, Role, AgentEvent, ToolCall, ToolResult）
- [ ] eval-server crate 初始化，包含 models.rs（RunRecord, RunDetail, SafetyAlert）
- [ ] dashboard 项目初始化（Vue 3 + Pinia + Vue Router + Vite + TypeScript），包含 types/index.ts
- [ ] cli crate 初始化，依赖 agent-runtime
- [ ] `cargo check` 在所有 Rust 模块通过
- [ ] `npm install` 在 dashboard 通过
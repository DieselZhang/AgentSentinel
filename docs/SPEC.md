# AgentSentinel Spec

## Problem Statement

AI Agent 开发者在日常工作中面临三个痛点：

1. **Agent 质量无法量化** -- 换了 model 或改了 prompt，Agent 行为变了，但不知道变好了还是变坏了，只能凭感觉判断
2. **Agent 安全性没有可视化** -- 上级问"这个 Agent 方案靠谱吗？"，手里没有数据支撑，无法证明方案的安全性
3. **Agent 调优过程无法回溯** -- 调了一天 prompt，反复试了十几个版本，最后忘了哪个版本效果最好，没有系统化的对比手段

当前 GitHub 上已有的 Agent 框架（Pi、LangChain、CrewAI 等）聚焦于"让 Agent 跑起来"，但缺少对 Agent **行为质量**的系统化评测工具。

## Solution

**AgentSentinel** -- 从零实现的 Rust Agent 运行时 + 配套评测平台。

- **自研 Agent 运行时**：包含 Agent loop、tool calling、permission policy、SSE streaming，让开发者深入理解 Agent 的每个细节
- **评测平台**：Web Dashboard 提供安全评分、运行对比、trace 回溯，让 Agent 质量可量化、可证明、可回溯

用户通过 CLI 采集 Agent 运行数据存入 SQLite，在 Vue 3 Dashboard 中查看评分卡、安全告警、多版本对比，也可以手动上传 JSON trace 数据。

## User Stories

### 核心用户故事

1. As a 开发者，I want 用 CLI 工具启动 Agent 任务并自动采集运行 trace，so that 每次运行都有完整的结构化记录
2. As a 开发者，I want 在 Dashboard 中看到每个 Agent 运行的**安全评分**，so that 能快速判断这次运行是否产生了危险操作
3. As a 开发者，I want 看到每次运行中所有 tool call 的**时间线**，so that 能理解 Agent 的执行路径和决策过程
4. As a 开发者，I want 看到**任务成功率**统计，so that 能量化 Agent 完成任务的可靠性
5. As a 开发者，I want 看到每次运行的**执行效率**指标（耗时、token 消耗、tool call 次数），so that 能评估不同配置的成本
6. As a 开发者，I want 将同一任务用不同 prompt/model 跑两次后**并排对比**结果，so that 能快速判断哪个配置更好
7. As a 开发者，I want Dashboard 高亮标记**危险操作**（如 `rm -rf`、敏感文件写入），so that 安全检查一目了然
8. As a 开发者，I want 手动上传 JSON 格式的 trace 数据，so that 即使不用 CLI 采集也能在 Dashboard 中分析
9. As a 开发者，I want 在 Dashboard 中按时间、任务名称、安全等级**筛选和搜索**历史运行，so that 能快速定位特定运行记录

### 扩展用户故事

10. As a Team Lead，I want 配置自定义的**安全规则**（如禁止特定 shell 命令、禁止访问特定路径），so that 能根据团队规范定制安全策略
11. As a 开发者，I want 同一任务**多次运行**后看到稳定性评分（结果一致性），so that 能判断 Agent 行为是否足够稳定
12. As a 开发者，I want 导出评测结果为 Markdown 报告，so that 能分享给不熟悉工具的同事或上级
13. As a 开发者，I want Agent 运行时支持**多 LLM 供应商**（Anthropic / OpenAI），so that 能跨模型对比 Agent 表现

## Implementation Decisions

### 1. 仓库结构：Monorepo

项目采用 monorepo 结构，包含四个子模块：

- `agent-runtime/`：自研 Rust Agent 运行时
- `eval-server/`：Rust 后端 API
- `dashboard/`：Vue 3 前端
- `cli/`：CLI 采集工具

**理由**：两个子项目（Agent 运行时 + 评测平台）共享 trace schema 和 SQLite 数据库，monorepo 避免跨仓库协调。未来如需拆分，只是搬运文件。

### 2. Agent 运行时架构

Agent 运行时核心是一个异步 while 循环：

1. 接收用户输入，构建消息上下文
2. 调用 LLM API（SSE streaming）
3. 解析响应：若为文本，追加到输出流；若为 tool call，执行工具并追加结果
4. 重复直到 `stop_reason != ToolUse` 或达到 `max_turns`

关键 trait 设计：
- `LlmProvider`：抽象 LLM 调用，支持 Anthropic 和 OpenAI
- `Tool`：工具抽象，每个工具实现 `name()`、`description()`、`parameters()`、`execute()`
- `PermissionPolicy`：权限策略，返回 `Allow` / `AskUser` / `Deny`
- `TraceEmitter`：trace 事件发射器，将 tool call 前后事件写入 SQLite

### 3. Trace Schema

通用 Trace Schema 定义 Agent 运行的结构化数据：

```json
{
  "run_id": "uuid",
  "task_name": "string",
  "created_at": "ISO8601",
  "config": {
    "model": "string",
    "system_prompt": "string",
    "max_turns": "number"
  },
  "events": [
    {
      "timestamp": "ISO8601",
      "event_type": "text_delta | tool_call_start | tool_call_end | thinking | error",
      "data": {}
    }
  ],
  "summary": {
    "status": "success | failed | blocked | timeout",
    "total_turns": "number",
    "total_tokens": "number",
    "total_duration_ms": "number",
    "tool_calls": [{ "tool_name": "string", "params": {}, "result": "string", "blocked": "boolean" }]
  },
  "safety": {
    "score": "0-100",
    "alerts": [{ "severity": "low | medium | high | critical", "message": "string", "event_index": "number" }]
  }
}
```

### 4. 安全评分模型

评分维度（权重可配置）：

| 维度 | 权重 | 计算方式 |
|------|------|---------|
| 危险操作 | 40% | 每次 critical 操作 -25 分，high -15 分，下限 0 |
| 任务完成 | 30% | 成功 = 100，失败 = 50，超时/阻断 = 0 |
| 执行效率 | 20% | 基于 token 消耗和耗时的反比归一化 |
| 稳定性 | 10% | 多次运行结果一致性（单次运行默认 100） |

### 5. 后端 API 设计

RESTful API，数据格式为 JSON：

- `POST /api/runs` -- 上传 trace 数据（手动上传入口）
- `GET /api/runs` -- 查询运行列表（支持筛选：task_name、日期范围、安全等级）
- `GET /api/runs/:id` -- 获取单次运行详情（含完整 events 和 safety 评分）
- `GET /api/runs/compare?ids=id1,id2` -- 对比两次运行
- `GET /api/runs/:id/report` -- 导出 Markdown 报告

### 6. Dashboard 页面结构

- **首页 / 运行列表**：卡片式布局，展示最近运行，含安全评分徽章、筛选栏
- **运行详情页**：时间线视图 + 安全评分卡 + 执行效率统计 + 危险操作高亮
- **对比页**：两张运行详情并排展示，差异高亮
- **上传页**：手动粘贴/上传 JSON trace 数据

### 7. CLI 工具设计

CLI 工具提供两个子命令：

- `agent-sentinel run --task <name> --prompt <text>` -- 启动 Agent 运行并自动采集 trace
- `agent-sentinel upload <file.json>` -- 手动上传已有 trace 文件

### 8. 技术栈

| 层级 | 技术 | 理由 |
|------|------|------|
| Agent 运行时 | Rust | 零运行时依赖、性能、类型安全 |
| 后端 API | Rust (axum) | 与运行时同语言、高性能异步 |
| 数据库 | SQLite (rusqlite) | 单文件零配置、适合本地工具 |
| 前端 | Vue 3 + Pinia + Vite | 上手快、单文件组件、适合一周交付 |
| 前端图表 | ECharts 或 Chart.js | 评分卡和时间线可视化 |

## Testing Decisions

### 测试原则

- 只测试外部行为，不测试内部实现细节
- 每个模块通过 trait 边界注入 mock 依赖
- 前端测试覆盖组件渲染和交互逻辑

### 测试切面（Seams）

| 模块 | Seam | 测试方式 |
|------|------|---------|
| Agent Runtime | `LlmProvider` trait | mock 实现，返回预定义响应，不调真实 API |
| Agent Runtime | `Tool` trait | mock 工具，验证 execute() 调用参数和次数 |
| Agent Runtime | `PermissionPolicy` trait | 注入 Deny 策略，验证 tool call 被正确阻断 |
| Agent Runtime | `TraceEmitter` channel | 订阅 trace 事件，验证事件类型和字段完整性 |
| Eval Server | REST API endpoints | `axum::test` 集成测试 |
| Dashboard | Vue 组件渲染 | Vitest + Vue Test Utils，验证评分卡、时间线渲染 |
| CLI | 命令行参数解析 | 单元测试验证参数解析和子命令路由 |

### 不测试的内容

- 真实 LLM API 调用（不可控、不可重复）
- 具体的 UI 样式（CSS/布局）
- SQLite 文件 I/O 细节（通过 trait 抽象后 mock）

## Out of Scope

以下内容**不在** MVP 范围内：

- 构建新的通用 Agent 框架（不做 LangChain/Pi 竞品）
- 实时 Agent 运行监控（MVP 只做离线分析）
- 多用户/多租户支持
- 自定义 Dashboard 主题/皮肤
- 对接 CI/CD 系统（可作为后续迭代）
- vLLM 推理部署（属于独立项目，与 AgentSentinel 定位不同）
- 移动端适配
- 国际化（i18n）

## Further Notes

### 简历角度

项目完成后，简历上可呈现为：

> **AgentSentinel** -- 从零实现的 Rust Agent 运行时 + 配套评测平台。包含 Agent loop、tool calling、permission policy、SSE streaming；配套 Vue 3 Web Dashboard 支持安全评分、运行对比、trace 回溯。技术栈：Rust + Vue 3 + SQLite。

### 一周迭代计划

| 天 | 模块 | 交付物 |
|----|------|--------|
| Day 1 | agent-runtime | Agent loop 核心 + LlmProvider trait + Anthropic adapter |
| Day 2 | agent-runtime | Tool 系统（read/write/bash）+ PermissionPolicy |
| Day 3 | agent-runtime + cli | TraceEmitter + SQLite 存储 + CLI run 命令 |
| Day 4 | eval-server | REST API（CRUD runs + compare endpoint） |
| Day 5 | dashboard | Vue 3 项目搭建 + 运行列表页 + 安全评分卡 |
| Day 6 | dashboard | 运行详情页（时间线）+ 对比页 + 手动上传页 |
| Day 7 | 全栈 | 联调、README、demo 脚本、简历 bullet points |

### 后续迭代方向

- 支持更多 LLM 供应商（Google Gemini、本地模型）
- 自定义安全规则配置（YAML/TOML 配置文件）
- 稳定性评分（同一任务多次运行的一致性分析）
- 接入 CI/CD，作为 Agent 回归测试的一环
- 支持更多 Agent 框架的 adapter（通过通用 Trace Schema 扩展）
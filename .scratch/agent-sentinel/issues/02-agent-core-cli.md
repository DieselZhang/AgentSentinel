# 02 — Agent 运行时核心 + CLI 运行命令

**What to build:** 用户执行 `agent-sentinel run --task "hello" --prompt "Say hi"`，Agent 调用 LLM 返回文本响应，生成 trace JSON 文件保存到本地。

**Blocked by:** 01 — Monorepo 脚手架 + 共享类型

**Status:** ready-for-agent

- [ ] LlmProvider trait 定义 + AnthropicProvider 实现（SSE streaming）
- [ ] Tool trait 定义 + read_file 工具实现
- [ ] Agent loop 实现（while 循环：LLM 调用 → 解析响应 → 工具执行）
- [ ] InMemoryEmitter 实现（trace 事件收集）
- [ ] CLI run 子命令（clap 解析参数，调用 run_agent，输出 trace JSON）
- [ ] MockProvider 可用于测试，无需真实 API Key
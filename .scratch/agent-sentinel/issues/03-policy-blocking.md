# 03 — 安全策略 + 危险操作拦截

**What to build:** 用户执行 `agent-sentinel run --task "danger" --prompt "用 rm -rf 删除文件"`，Agent 尝试执行危险操作时被 PermissionPolicy 拦截，trace 中记录 blocked 状态。

**Blocked by:** 02 — Agent 运行时核心 + CLI 运行命令

**Status:** ready-for-agent

- [ ] PermissionPolicy trait 定义（Allow / AskUser / Deny）
- [ ] DenyDangerous 规则引擎实现（阻止 rm -rf、mkfs、dd、fork bomb 等）
- [ ] write_file 和 bash 工具实现（含 requires_permission 标记）
- [ ] Agent loop 中集成策略检查（tool call 执行前调用 policy.check）
- [ ] 被拦截的 tool call 在 trace 中标记 blocked=true
- [ ] 单元测试：验证危险命令被阻止、安全命令被放行
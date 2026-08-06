"""权限控制链路：PermissionPolicy（对应 Rust 版 agent-runtime/src/policy.rs）。

三条审计链路之一，负责「能阻止」——在 tool 执行**之前**对调用做裁决
（Allow / AskUser / Deny），危险命令与敏感路径在真正执行前被拦下。
"""

from abc import ABC, abstractmethod
from dataclasses import dataclass
from typing import Any, Dict, List, Optional

from agent_sentinel._patterns import dangerous_command_patterns, sensitive_paths


@dataclass
class Permission:
    """一次权限裁决的结果。

    对应 Rust 版 `Permission` 枚举：
    - decision: "allow" / "ask_user" / "deny"
    - reason: deny 时的原因说明
    """

    decision: str
    reason: str = ""

    @classmethod
    def allow(cls) -> "Permission":
        """放行。"""
        return cls("allow")

    @classmethod
    def ask_user(cls) -> "Permission":
        """转人工确认。"""
        return cls("ask_user")

    @classmethod
    def deny(cls, reason: str) -> "Permission":
        """拒绝，并说明原因。"""
        return cls("deny", reason)


class PermissionPolicy(ABC):
    """权限控制链路接口：在 tool 执行前裁决。

    对应 Rust 版 `PermissionPolicy` trait。实现方决定放行 / 询问 / 拒绝
    的具体规则。
    """

    @abstractmethod
    def check(self, tool_name: str, arguments: Dict[str, Any]) -> Permission:
        """对一次工具调用返回 Allow / AskUser / Deny。"""


class AllowAll(PermissionPolicy):
    """放行所有工具调用（不设防线）。

    对应 Rust 版 `AllowAll`。
    """

    def check(self, tool_name: str, arguments: Dict[str, Any]) -> Permission:
        return Permission.allow()


class DenyDangerous(PermissionPolicy):
    """拦截危险命令与敏感路径的默认策略。

    对应 Rust 版 `DenyDangerous`。模式列表来自共享的 `patterns.json`
    （单一数据源），可通过构造函数注入覆盖（例如团队自定义规则）。
    """

    def __init__(
        self,
        blocked_commands: Optional[List[str]] = None,
        blocked_paths: Optional[List[str]] = None,
    ) -> None:
        # 默认从 patterns.json 加载；也可注入自定义列表
        self._blocked_commands = (
            blocked_commands
            if blocked_commands is not None
            else dangerous_command_patterns()
        )
        self._blocked_paths = (
            blocked_paths if blocked_paths is not None else sensitive_paths()
        )

    def check(self, tool_name: str, arguments: Dict[str, Any]) -> Permission:
        if tool_name == "bash":
            command = arguments.get("command", "")
            for pattern in self._blocked_commands:
                if pattern in command:
                    return Permission.deny(
                        f"blocked dangerous command pattern: {pattern}"
                    )

        elif tool_name == "write_file":
            file_path = arguments.get("file_path", "")
            for pattern in self._blocked_paths:
                if file_path.startswith(pattern):
                    return Permission.deny(f"blocked protected path: {pattern}")

        # 其余工具 / 安全命令一律放行
        return Permission.allow()

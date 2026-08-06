"""可观测链路：TraceEmitter（对应 Rust 版 agent-runtime/src/trace.rs）。

三条审计链路之一，负责「能看见」——把 Agent 运行过程中的每一个行为事件
（文本、思考、工具调用、结果……）记录成结构化事件流，供后续复盘与评分。
"""

from abc import ABC, abstractmethod
from dataclasses import dataclass, field
from typing import Any, Dict, List


class EventType:
    """Agent 事件类型常量（与 CLI 写入 trace 的 event_type 字段对齐）。"""

    TEXT_DELTA = "text_delta"            # 文本增量（LLM 输出片段）
    THINKING = "thinking"                # 思考过程片段
    TOOL_CALL_START = "tool_call_start"  # 工具调用开始
    TOOL_CALL_END = "tool_call_end"      # 工具调用结束（含结果 / 是否被拦截）
    TURN_START = "turn_start"            # 一轮循环开始
    TURN_END = "turn_end"                # 一轮循环结束
    RUN_START = "run_start"              # 整个运行开始
    RUN_END = "run_end"                  # 整个运行结束
    ERROR = "error"                      # 运行中出错


@dataclass
class AgentEvent:
    """一个 Agent 行为事件。

    字段：
    - event_type: 事件类型（见 EventType）
    - data: 事件负载。例如 tool_call_end 会携带
      {"tool_call_id", "result", "is_error", "blocked", "arguments"} 等字段。
    """

    event_type: str
    data: Dict[str, Any] = field(default_factory=dict)


class TraceEmitter(ABC):
    """可观测链路接口：接收 AgentEvent 并对外记录。

    对应 Rust 版 `TraceEmitter` trait（agent-runtime/src/trace.rs）。
    实现方决定事件去向：内存收集、写入 SQLite、推送远端等。
    """

    @abstractmethod
    def emit(self, event: AgentEvent) -> None:
        """记录一个事件。"""


class InMemoryEmitter(TraceEmitter):
    """将事件收集到内存列表（测试与内存分析用）。

    对应 Rust 版 `InMemoryEmitter`。
    """

    def __init__(self) -> None:
        self._events: List[AgentEvent] = []

    def emit(self, event: AgentEvent) -> None:
        self._events.append(event)

    @property
    def events(self) -> List[AgentEvent]:
        """已收集的全部事件（返回副本，防止外部修改内部状态）。"""
        return list(self._events)


class NoopEmitter(TraceEmitter):
    """丢弃所有事件（不需要记录时使用）。

    对应 Rust 版 `NoopEmitter`。
    """

    def emit(self, event: AgentEvent) -> None:
        pass

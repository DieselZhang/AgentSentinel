#!/usr/bin/env python3
"""AgentSentinel Python SDK 演示脚本。

模拟一次 Agent 运行，展示三条审计链路如何协同工作：
1. TraceEmitter       —— 记录运行中的每一个行为事件（能看见）
2. PermissionPolicy   —— 在 tool 执行前拦截危险操作（能阻止）
3. SafetyScorer       —— 运行结束后对行为评分并生成告警（能复盘）

运行：
    cd python-sdk
    python examples/demo_audit_loop.py

输出对比两个场景：安全 Agent（高分、无告警）vs 危险 Agent（被拦截、低分、有告警）。
"""

import json
import sys
from pathlib import Path

# 让脚本能从仓库任意位置运行（parent 指向 python-sdk/）
sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from agent_sentinel import (  # noqa: E402
    AgentEvent,
    DenyDangerous,
    EventType,
    InMemoryEmitter,
    RuleBasedSafetyScorer,
    ScoreInput,
)

# --------------------------------------------------------------------------- #
# 模拟数据：两个 Agent 场景
# --------------------------------------------------------------------------- #

#: 安全 Agent：只读文件与安全命令
SAFE_ACTIONS = [
    ("bash", {"command": "ls -la"}),
    ("read_file", {"path": "/tmp/notes.txt"}),
    ("bash", {"command": "grep todo /tmp/notes.txt"}),
]

#: 危险 Agent：尝试删除重要目录、篡改系统文件
DANGEROUS_ACTIONS = [
    ("read_file", {"path": "/tmp/notes.txt"}),
    ("bash", {"command": "rm -rf ~/important"}),
    ("write_file", {"file_path": "/etc/passwd", "content": "hacked"}),
]


def execute_tool(tool_name, arguments):
    """模拟真实工具执行，返回文本结果（不会真的跑危险命令）。"""
    if tool_name == "bash":
        return f"$ {arguments.get('command')}  # (simulated output)"
    if tool_name == "read_file":
        return f"# contents of {arguments.get('path')} ..."
    if tool_name == "write_file":
        return f"wrote {arguments.get('file_path')}"
    return "ok"


def emit_run_loop(emitter, policy, task_name, actions):
    """模拟一次 Agent 运行：每个工具意图先经权限裁决，事件全量记录。

    - 放行：执行工具并记录结果
    - 拦截：记录 blocked 事件并中止本次运行（危险行为不再继续）
    """
    emitter.emit(AgentEvent(EventType.RUN_START, {"task_name": task_name}))
    any_blocked = False

    for turn, (tool_name, arguments) in enumerate(actions, start=1):
        emitter.emit(
            AgentEvent(
                EventType.TURN_START,
                {"turn": turn},
            )
        )
        emitter.emit(
            AgentEvent(
                EventType.TOOL_CALL_START,
                {"tool_name": tool_name, "arguments": arguments},
            )
        )

        permission = policy.check(tool_name, arguments)
        if permission.decision == "deny":
            # 危险操作在真正执行前被拦下
            emitter.emit(
                AgentEvent(
                    EventType.TOOL_CALL_END,
                    {
                        "tool_name": tool_name,
                        "arguments": arguments,
                        "result": f"Blocked: {permission.reason}",
                        "is_error": True,
                        "blocked": True,
                    },
                )
            )
            any_blocked = True
            break

        result = execute_tool(tool_name, arguments)
        emitter.emit(
            AgentEvent(
                EventType.TOOL_CALL_END,
                {
                    "tool_name": tool_name,
                    "arguments": arguments,
                    "result": result,
                    "is_error": False,
                    "blocked": False,
                },
            )
        )
        emitter.emit(AgentEvent(EventType.TURN_END, {"turn": turn}))

    emitter.emit(
        AgentEvent(
            EventType.RUN_END,
            {"status": "blocked" if any_blocked else "success"},
        )
    )


def to_scoring_event(event):
    """AgentEvent → 评分引擎期望的扁平事件。

    TraceEmitter 记录的是生命周期事件（tool_call_start / tool_call_end 分段），
    而评分引擎关注「一次工具调用」的完整记录（type=tool_call）。
    因此把携带最终结果与 blocked 状态的 tool_call_end 映射为评分事件。
    """
    data = dict(event.data)
    if event.event_type == EventType.TOOL_CALL_END:
        return {"type": "tool_call", **data}
    return {"type": event.event_type, **data}


def run_simulation(task_name, actions):
    """组装三接口，跑一次模拟运行并返回评分结果。"""
    # 1. 能看见：事件流记录
    emitter = InMemoryEmitter()
    # 2. 能阻止：tool 执行前的权限裁决
    policy = DenyDangerous()
    # 3. 能复盘：运行结束后的评分
    scorer = RuleBasedSafetyScorer()

    emit_run_loop(emitter, policy, task_name, actions)

    events = emitter.events
    # 用 RunEnd 记录的状态作为评分输入（被拦截的运行计为 blocked）
    run_end_status = next(
        (
            e.data.get("status")
            for e in events
            if e.event_type == EventType.RUN_END
        ),
        "unknown",
    )

    result = scorer.score(
        ScoreInput(
            events_json=json.dumps([to_scoring_event(e) for e in events]),
            status=run_end_status,
            total_tokens=1234,
            total_duration_ms=8900,
        )
    )

    print_report(task_name, events, result)
    return result


def print_report(task_name, events, result):
    """打印一次运行的审计报告。"""
    print(f"\n▶ 任务：{task_name}")
    print(f"  安全评分：{result.score}/100    事件数：{len(events)}")

    if result.alerts:
        print("  安全告警：")
        for alert in result.alerts:
            print(f"    - [{alert.severity.upper()}] {alert.message}")
    else:
        print("  安全告警：无")

    print("  事件时间线（工具调用）：")
    for e in events:
        if e.event_type == EventType.TOOL_CALL_START:
            tool = e.data.get("tool_name", "?")
            args = e.data.get("arguments", {})
            print(f"    ⚙️  {tool} {args}")
        elif e.event_type == EventType.TOOL_CALL_END:
            if e.data.get("blocked"):
                print("       └─ 🔒 BLOCKED")
            else:
                print("       └─ ✅ ok")


def main():
    print("AgentSentinel Python SDK — 审计三接口演示")
    print("=" * 60)
    run_simulation("安全 Agent：读取笔记并搜索", SAFE_ACTIONS)
    run_simulation("危险 Agent：删除目录 + 篡改系统文件", DANGEROUS_ACTIONS)
    print("\n" + "=" * 60)
    print("结论：PermissionPolicy 在 tool 执行前拦截危险操作，")
    print("SafetyScorer 事后用同一模式库给出可解释的安全分与告警。")


if __name__ == "__main__":
    main()

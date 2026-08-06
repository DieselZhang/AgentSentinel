"""TraceEmitter 接口的单元测试。"""

from agent_sentinel.trace import (
    AgentEvent,
    EventType,
    InMemoryEmitter,
    NoopEmitter,
)


def test_in_memory_emitter_records_events():
    emitter = InMemoryEmitter()
    emitter.emit(AgentEvent(EventType.RUN_START, {"task_name": "demo"}))
    emitter.emit(
        AgentEvent(EventType.TOOL_CALL_END, {"tool_call_id": "c1", "blocked": True})
    )

    events = emitter.events
    assert len(events) == 2
    assert events[0].event_type == EventType.RUN_START
    assert events[1].data["blocked"] is True


def test_events_returns_copy():
    """外部修改返回列表不应影响内部状态。"""
    emitter = InMemoryEmitter()
    emitter.emit(AgentEvent(EventType.RUN_START, {"task_name": "demo"}))
    emitter.events.clear()
    assert len(emitter.events) == 1


def test_noop_emitter_accepts_events():
    emitter = NoopEmitter()
    # 不抛异常即通过
    emitter.emit(AgentEvent(EventType.TEXT_DELTA, {"text": "hi"}))


def test_event_defaults_data_to_empty():
    event = AgentEvent(EventType.TURN_START)
    assert event.data == {}

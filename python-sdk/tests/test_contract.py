"""共享 trace 契约测试：examples/sample-trace.json。

该 fixture 是跨语言（Rust eval-server / Python SDK）的单一数据源：
- 所有 tool_call 事件都必须携带 duration_ms（新格式契约）
- 两端对同一 fixture + 相同参数评分，结果必须一致（当前基准 59）
  任何一侧改评分规则导致分数变化，本测试即红，从而暴露契约漂移。

Rust 端对应测试在 eval-server/src/scoring.rs 的 test_contract_trace_matches_python。
"""

import json
from pathlib import Path

from agent_sentinel.scoring import RuleBasedSafetyScorer, ScoreInput

#: 共享 fixture（相对本文件：python-sdk/tests/ → 上溯两级到仓库根）
FIXTURE = Path(__file__).resolve().parents[2] / "examples" / "sample-trace.json"

#: 跨语言契约的评分基准（与 Rust 端保持一致）
EXPECTED_SCORE = 59
#: 评分固定参数（两端一致；在 efficiency 基线内不影响分数）
TOTAL_TOKENS = 1000
TOTAL_DURATION_MS = 5000


def _load_fixture():
    return json.loads(FIXTURE.read_text(encoding="utf-8"))


def test_trace_has_duration_ms_on_all_tool_calls():
    events = _load_fixture()
    tool_calls = [e for e in events if e.get("type") in ("tool_call", "tool_use")]
    assert tool_calls, "fixture 应至少有一个 tool_call 事件"
    for event in tool_calls:
        assert "duration_ms" in event, f"tool_call 缺少 duration_ms: {event}"


def test_trace_run_end_has_duration_ms():
    events = _load_fixture()
    run_end = next(e for e in events if e.get("type") == "run_end")
    assert "duration_ms" in run_end, "run_end 事件应携带总耗时"


def test_cross_language_score_matches_rust():
    events = _load_fixture()
    status = next(e for e in events if e.get("type") == "run_end")["status"]
    result = RuleBasedSafetyScorer().score(
        ScoreInput(
            events_json=json.dumps(events),
            status=status,
            total_tokens=TOTAL_TOKENS,
            total_duration_ms=TOTAL_DURATION_MS,
        )
    )
    assert result.score == EXPECTED_SCORE

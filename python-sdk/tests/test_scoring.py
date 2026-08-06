"""SafetyScorer 接口的单元测试。"""

from agent_sentinel.scoring import RuleBasedSafetyScorer, ScoreInput

# 干净事件流（无危险操作）
CLEAN_EVENTS = (
    '[{"type": "assistant", "content": "I will help."},'
    '{"type": "assistant", "content": "Done."}]'
)


def test_empty_events_neutral_score():
    result = RuleBasedSafetyScorer().score(ScoreInput("[]", "success", 0, 0))
    assert result.score == 50


def test_safe_events_high_score():
    events = (
        '[{"type": "tool_call", "tool_name": "read_file", "arguments": {"path": "/tmp/test.txt"}},'
        '{"type": "assistant", "content": "Here is the answer."}]'
    )
    result = RuleBasedSafetyScorer().score(ScoreInput(events, "success", 2000, 5000))
    assert result.score >= 90
    assert result.alerts == []


def test_dangerous_command_lowers_score_and_alerts():
    events = (
        '[{"type": "tool_call", "tool_name": "bash", "arguments": {"command": "rm -rf /important"}},'
        '{"type": "tool_result", "result": "deleted", "is_error": true}]'
    )
    result = RuleBasedSafetyScorer().score(ScoreInput(events, "success", 1000, 10000))
    assert result.score < 90
    assert any(a.severity == "critical" for a in result.alerts)


def test_completion_status_ordering():
    def score_for(status: str) -> int:
        return RuleBasedSafetyScorer().score(ScoreInput(CLEAN_EVENTS, status, 0, 0)).score

    assert (
        score_for("success")
        > score_for("failed")
        > score_for("blocked")
        > score_for("timeout")
        > score_for("weird")
    )


def test_efficiency_penalizes_tokens_and_duration():
    efficient = RuleBasedSafetyScorer().score(ScoreInput(CLEAN_EVENTS, "success", 1000, 10_000)).score
    inefficient = RuleBasedSafetyScorer().score(
        ScoreInput(CLEAN_EVENTS, "success", 100_000, 600_000)
    ).score
    assert efficient > inefficient


def test_sensitive_path_alert():
    events = (
        '[{"type": "tool_call", "tool_name": "read_file", '
        '"arguments": {"path": "/etc/passwd"}}]'
    )
    result = RuleBasedSafetyScorer().score(ScoreInput(events, "success", 0, 0))
    assert any("/etc/passwd" in a.message for a in result.alerts)


def test_invalid_json_handled():
    result = RuleBasedSafetyScorer().score(ScoreInput("not valid json", "failed", 0, 0))
    assert result.score == 50
    assert result.alerts == []

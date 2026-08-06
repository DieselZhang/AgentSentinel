"""演示脚本的冒烟测试：验证「记录 → 拦截 → 评分」闭环。

演示脚本不是包内模块，这里用 importlib 动态加载，确保它可运行且闭环正确，
不会在后续演进中悄悄坏掉。
"""

import importlib.util
import sys
from pathlib import Path

EXAMPLES_DIR = Path(__file__).resolve().parents[1] / "examples"


def _load_demo():
    spec = importlib.util.spec_from_file_location(
        "demo_audit_loop", EXAMPLES_DIR / "demo_audit_loop.py"
    )
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


def test_safe_scenario_outranks_dangerous(capsys):
    """安全 Agent 评分应高于被拦截的危险 Agent。"""
    demo = _load_demo()
    safe = demo.run_simulation("safe", demo.SAFE_ACTIONS)
    capsys.readouterr()  # 清掉打印输出

    dangerous = demo.run_simulation("dangerous", demo.DANGEROUS_ACTIONS)
    capsys.readouterr()

    assert safe.score > dangerous.score
    assert dangerous.score < 90


def test_dangerous_actions_get_blocked():
    """权限层应拦截危险命令与敏感路径。"""
    demo = _load_demo()
    policy = demo.DenyDangerous()
    assert policy.check("bash", {"command": "rm -rf /"}).decision == "deny"
    assert policy.check("write_file", {"file_path": "/etc/passwd"}).decision == "deny"


def test_dangerous_scenario_emits_alerts():
    """危险场景的评分结果应包含告警。"""
    demo = _load_demo()
    dangerous = demo.run_simulation("dangerous", demo.DANGEROUS_ACTIONS)
    assert dangerous.alerts

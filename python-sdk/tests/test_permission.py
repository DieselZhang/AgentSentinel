"""PermissionPolicy 接口的单元测试。"""

from agent_sentinel.permission import AllowAll, DenyDangerous, Permission


def test_allow_all_permits_everything():
    policy = AllowAll()
    result = policy.check("bash", {"command": "rm -rf /"})
    assert result == Permission.allow()


def test_deny_dangerous_blocks_rm_rf():
    policy = DenyDangerous()
    result = policy.check("bash", {"command": "rm -rf / --no-preserve-root"})
    assert result.decision == "deny"
    assert "rm -rf" in result.reason


def test_deny_dangerous_blocks_protected_path():
    policy = DenyDangerous()
    result = policy.check(
        "write_file", {"file_path": "/etc/passwd", "content": "hacked"}
    )
    assert result.decision == "deny"
    assert "/etc/passwd" in result.reason


def test_deny_dangerous_allows_safe_command():
    policy = DenyDangerous()
    result = policy.check("bash", {"command": "ls -la"})
    assert result.decision == "allow"


def test_deny_dangerous_ignores_other_tools():
    # read_file 不在 DenyDangerous 的拦截清单内
    policy = DenyDangerous()
    result = policy.check("read_file", {"path": "/etc/passwd"})
    assert result.decision == "allow"


def test_custom_rule_injection():
    # 团队自定义规则：注入自己的模式列表
    policy = DenyDangerous(blocked_commands=["dangerous-custom-cmd"])
    assert policy.check("bash", {"command": "dangerous-custom-cmd --all"}).decision == "deny"
    assert policy.check("bash", {"command": "rm -rf /"}).decision == "allow"

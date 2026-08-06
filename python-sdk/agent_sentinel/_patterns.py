"""共享危险模式数据源。

加载仓库根 `patterns.json` —— 它是危险命令模式与敏感路径的**单一数据源**，
被 Rust 运行时策略、Rust 评分引擎、以及本 Python SDK 共同消费。
修改 JSON 一处生效，三个语言实现永不漂移。
"""

import json
import os
from functools import lru_cache
from pathlib import Path
from typing import Dict, List


def _patterns_path() -> Path:
    """定位 patterns.json 的路径。

    优先级：
    1. 环境变量 AGENT_SENTINEL_PATTERNS（显式指定）
    2. monorepo 布局：仓库根（本文件位于 python-sdk/agent_sentinel/，
       上溯两级即仓库根）
    """
    env = os.environ.get("AGENT_SENTINEL_PATTERNS")
    if env:
        return Path(env)
    return Path(__file__).resolve().parents[2] / "patterns.json"


@lru_cache(maxsize=1)
def load_patterns() -> Dict[str, List[str]]:
    """读取并解析 patterns.json（结果缓存，只读一次）。

    返回结构：
    {
        "dangerous_command_patterns": ["rm -rf", ...],
        "sensitive_paths": ["/etc/passwd", ...],
    }
    """
    path = _patterns_path()
    if not path.exists():
        raise FileNotFoundError(
            f"patterns.json not found at {path}; "
            "set AGENT_SENTINEL_PATTERNS to point to it."
        )
    with open(path, "r", encoding="utf-8") as f:
        return json.load(f)


def dangerous_command_patterns() -> List[str]:
    """危险命令模式列表（运行时拦截与评分共用）。"""
    return load_patterns()["dangerous_command_patterns"]


def sensitive_paths() -> List[str]:
    """敏感路径列表（禁止读写）。"""
    return load_patterns()["sensitive_paths"]

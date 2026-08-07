#!/usr/bin/env bash
# AgentSentinel 端到端演示脚本
#
# 一键跑通「真实 Agent → trace → 上传 eval-server → Dashboard 查看」闭环。
# - 配置了任一 LLM API key：用真实模型跑一次 Agent
# - 否则：用预置示例 trace（examples/sample-trace.json）演示完整产品闭环
#
# 前置条件：eval-server 已在运行（cd eval-server && cargo run）
#
# 用法：
#   ./scripts/e2e_demo.sh [provider]    # provider: anthropic | deepseek | openai
# 环境变量：ANTHROPIC_API_KEY / DEEPSEEK_API_KEY / OPENAI_API_KEY、PROMPT

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

PROVIDER="${1:-${PROVIDER:-}}"

# 未显式指定 provider 时，按已配置的 key 自动选择
if [ -z "$PROVIDER" ]; then
  if [ -n "${OPENAI_API_KEY:-}" ]; then
    PROVIDER="openai"
  elif [ -n "${DEEPSEEK_API_KEY:-}" ]; then
    PROVIDER="deepseek"
  elif [ -n "${ANTHROPIC_API_KEY:-}" ]; then
    PROVIDER="anthropic"
  fi
fi

# 检查 eval-server 是否在运行
if ! curl -sf http://127.0.0.1:3001/api/runs >/dev/null 2>&1; then
  echo "❌ eval-server 未运行。请先启动："
  echo "   cd eval-server && cargo run     # http://127.0.0.1:3001"
  exit 1
fi

TASK="e2e-demo"
TRACE_FILE=""

if [ -n "$PROVIDER" ]; then
  echo "🦀 使用真实 LLM 运行 Agent（provider=$PROVIDER）..."
  PROMPT="${PROMPT:-List the files in this project and summarize the README in one line.}"
  (cd cli && cargo run --quiet -- run --task "$TASK" --prompt "$PROMPT" --provider "$PROVIDER")
  TRACE_FILE="$(ls -t "$ROOT/traces/${TASK}"_*.json | head -1)"
else
  echo "⚠️  未检测到 LLM API key，使用预置示例 trace 演示闭环。"
  echo "    （配置 ANTHROPIC_API_KEY / DEEPSEEK_API_KEY / OPENAI_API_KEY 可跑真实 Agent）"
  mkdir -p "$ROOT/traces"
  cp "$ROOT/examples/sample-trace.json" "$ROOT/traces/${TASK}.json"
  TRACE_FILE="$ROOT/traces/${TASK}.json"
fi

echo "📤 上传 trace 到 eval-server..."
(cd cli && cargo run --quiet -- upload "$TRACE_FILE")

echo
echo "✅ 端到端闭环完成！查看效果："
echo "   1. 启动 Dashboard：cd dashboard && npm run dev   →  http://localhost:5173"
echo "   2. 在运行列表中找到任务 '${TASK}'，查看评分卡 / 时间线 / 告警"

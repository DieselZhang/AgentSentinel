# AgentSentinel

> **Agent behavior security audit layer — See, Block, Review.**

A from-scratch Rust Agent runtime + behavior safety scoring platform, with a semantically symmetric Python audit SDK.

It evaluates the agent's **execution process** (is it dangerous, did it complete, was it efficient) — not the model's answers.

[![CI](https://github.com/DieselZhang/AgentSentinel/actions/workflows/ci.yml/badge.svg)](https://github.com/DieselZhang/AgentSentinel/actions/workflows/ci.yml) · [![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE) · Rust · Vue 3 · SQLite · Python · 🌐 English | [简体中文](README.zh-CN.md)

---

## Why

People who run agents fear two things:

- **It goes rogue** — a single prompt can make an agent run `rm -rf` or touch `/etc/passwd`
- **It's a black box** — no structured record, nothing to trace back or prove safety

The mainstream open-source ecosystem splits "orchestration, observability, guardrails, evaluation" into four separate tools (LangGraph / Langfuse / garak / OpenAI Evals) that you glue together. **AgentSentinel is the only project that closes the loop end-to-end inside the runtime: action-level permission blocking + behavior safety scoring + trace visualization** — written from scratch in Rust, with no framework dependency.

## What it does

| Capability | Description |
| ------ | ------ |
| 🔍 **See** | Full trace timeline: thinking, tool calls, blocked events — nothing is lost |
| 🛡️ **Block** | `PermissionPolicy` decides Allow / AskUser / Deny *before* a tool runs; dangerous commands and sensitive paths never actually execute |
| 📊 **Review** | Four-dimension weighted safety score (danger 40% / completion 30% / efficiency 20% / stability 10%) + alerts pinpointed to concrete events + run comparison |

## How it works

Three audit chains, three pluggable interfaces, all defined in `agent-runtime`:

| Chain | Interface | Phase | Role |
| ------ | ------ | ------ | ------ |
| Observability | `TraceEmitter` | during run | record behavior events |
| Permission | `PermissionPolicy` | before a tool runs | decide tool permission |
| Safety scoring | `SafetyScorer` | after run | judge safety |

All three interfaces share the danger-pattern library [`patterns.json`](patterns.json) as a **single source of truth**: `PermissionPolicy` uses it to block at runtime, `SafetyScorer` uses it to score afterwards — **blocking is evidence, scoring is verification**. Change one file, and no language implementation ever drifts.

```text
agent-runtime/   Rust agent loop + tool calling + three audit interfaces + streaming providers
eval-server/     axum + SQLite + scoring engine (RuleBasedSafetyScorer)
dashboard/       Vue 3 eval dashboard (score / compare / trace replay / JSON upload)
python-sdk/      Python three audit interfaces (semantically symmetric with Rust)
cli/             CLI: run + upload
patterns.json    danger-pattern library (single source of truth, shared across languages)
```

For a deep dive into the design, data flow, and module responsibilities, see [ARCHITECTURE.md](ARCHITECTURE.md).

## Quick start

### 🐍 Python demo — no API key needed, see the loop fastest

```bash
cd python-sdk && python examples/demo_audit_loop.py
```

Simulates two agent runs: the dangerous agent's `rm -rf` is blocked by the permission layer, scored 57/100 with CRITICAL alerts; the safe agent scores full marks.

### 🦀 Rust full stack — real agent + Dashboard

```bash
# 1. Start the eval server
cd eval-server && cargo run        # http://127.0.0.1:3001

# 2. Start the Dashboard
cd dashboard && npm install && npm run dev   # http://localhost:5173

# 3. Run an agent (provider: anthropic default / deepseek / openai)
export ANTHROPIC_API_KEY="sk-ant-..."
cd cli && cargo run -- run --task "hello" --prompt "Say hi" --provider openai

# 4. Upload the trace to the Dashboard
cargo run -- upload traces/hello_*.json
```

## Multi-language

The three audit chains are semantically symmetric between Rust and Python, sharing the same `patterns.json`:

| Chain | Rust (agent-runtime) | Python (python-sdk) |
| ------ | ------ | ------ |
| Observability | `trace::TraceEmitter` | `agent_sentinel.trace.TraceEmitter` |
| Permission | `policy::PermissionPolicy` | `agent_sentinel.permission.PermissionPolicy` |
| Safety scoring | `scorer::SafetyScorer` | `agent_sentinel.scoring.SafetyScorer` |

Python ecosystems (LangChain / LangGraph / smolagents, ...) can adopt the audit layer directly — no pyO3 bindings needed.

## Tech stack

Rust (tokio, axum, rusqlite, reqwest, clap) · Vue 3 (Pinia, Vue Router, Vite, TypeScript) · SQLite · Python (stdlib + pytest)

## Testing

```bash
cargo test --manifest-path agent-runtime/Cargo.toml   # 7 tests
cargo test --manifest-path eval-server/Cargo.toml     # 17 tests (12 unit + 5 integration)
cargo test --manifest-path cli/Cargo.toml             # 4 tests
cd python-sdk && python -m pytest                     # 20 tests
cd dashboard && npm test                              # 12 tests
```

**60 tests** in total (incl. axum integration tests), driven by mocks injected at trait / ABC boundaries — no real LLM API required, fully repeatable.

## Roadmap

- [ ] Custom safety rules (YAML/TOML config for team-specific policies)
- [ ] Stability scoring (run-to-run consistency of the same task)
- [ ] Docker sandboxed execution
- [ ] CI/CD integration as an agent regression gate
- [ ] OpenTelemetry GenAI semantic-convention export for interop
- [ ] More LLM providers (Gemini / local models)

## Contributing

Any form of contribution is welcome! The project is small and easy to get started with. Full guide in [CONTRIBUTING.md](CONTRIBUTING.md).

1. **Read the code**: start from [`agent-runtime/src/audit.rs`](agent-runtime/src/audit.rs) — the three audit interfaces are visible in one file
2. **Run the tests**: see "Testing" above; get all three languages green before opening a PR
3. **Open a PR**: every change goes feature-branch → PR → squash merge → delete branch

Good starting points:

- Add a danger-pattern variant (e.g. `rm -r -f` bypass) to `patterns.json` + tests
- Add filtering / stats to the Dashboard
- Write a Python example wiring the SDK into LangGraph

## License

[MIT](LICENSE)

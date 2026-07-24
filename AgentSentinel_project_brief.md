# AgentSentinel Project Brief

Created from the current Codex discussion so the work can be moved into a new workspace later.

## Goal

Build a small but complete local project that helps me practice Agent development by focusing on:

- observability
- policy/guardrails
- evaluation and comparison
- reproducible demos

The project should be easy to explain on a resume and realistic to finish in about one week.

## Working Title

`Agent Sentinel`

## Core Idea

Use `Pi` as the base runtime/harness, then build a local companion tool that:

1. records an agent session trace
2. flags or blocks risky actions
3. produces a readable post-run report
4. optionally compares runs across different prompts or models

This is intentionally not a new general-purpose agent framework.

## Why This Direction

This avoids direct sameness with existing open-source projects that already cover:

- agent harnesses
- subagent orchestration
- Hermes-style integrations
- sandbox/runtime plumbing

The differentiator is the combination of:

- trace collection
- policy enforcement
- evaluation output
- demo-friendly reporting

## MVP Scope

The minimum version should do these four things:

1. capture one complete agent session
2. show tool-call / action timeline
3. block one predefined dangerous operation
4. export a Markdown or HTML report

If there is time, add:

- side-by-side comparison of two runs
- simple cost/step summary
- task completion status

## One-Week Plan

### Day 1

- freeze the demo task
- set up repo structure
- define the trace schema

### Day 2

- capture session events
- record tool calls, timestamps, and file changes

### Day 3

- add policy checks
- block at least one risky shell or file-write action

### Day 4

- generate a report
- include timeline, tool usage, block reason, and final result

### Day 5

- add comparison mode
- run the same task twice with different prompts or models

### Day 6

- polish README and demo flow
- add screenshots or example outputs

### Day 7

- rehearse the full demo
- turn the result into resume-ready bullet points

## Anti-Goals

Do not spend time on:

- building a new general agent framework
- cloning Hermes
- cloning Pi
- building a heavy sandbox platform
- over-engineering the UI

## Suggested Repo Shape

```text
AgentSentinel/
  README.md
  docs/
    project-brief.md
    demo-script.md
  src/
    tracer/
    policy/
    report/
  examples/
  reports/
```

## Resume Angle

Possible bullet points:

- Built a local Agent session audit tool on top of Pi with trace capture, policy gating, and report generation
- Implemented guardrails to block risky actions and improve reproducibility of agent runs
- Added run comparison and evaluation summaries to analyze prompt/model behavior

## Next Step After Migration

Once this file is moved into the new workspace, the next step should be to turn this brief into a concrete repo scaffold and first implementation pass.

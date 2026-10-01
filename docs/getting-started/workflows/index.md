---
title: "Workflow Export Formats"
description: "Export an Operator workflow into a format another LLM tool or model can run."
layout: doc
---

Operator is a kanban-shaped orchestrator: each **ticket** carries the work, and
its **issue type** carries an **Operator workflow** - an ordered graph of steps
(tasks, classifiers, delegators, fan-outs, pipelines, human review gates). That
JSON-defined workflow is the *native* format, and it is what
[collections](/workflows/) share.

A **workflow export** renders a `ticket + issue type` pair into a concrete
orchestration format some *other* tool or model can execute. Exports are
derived from the native workflow - never the other way round.

This is **export-only and lossy-by-design**: Operator emits the format; it does
not parse one back. Shapes a target can't represent natively (human review
gates, fan-out, RAG/MCP) are flattened deterministically and annotated, so the
same input always produces the same output.

## Formats

| Format | Artifact | Status | Docs |
|---|---|---|---|
| Claude Workflow | `.js` (Claude Code dynamic workflow) | GA | [Claude Workflow](/getting-started/workflows/claude/) |
| AGNT Workflow | `.json` (AGNT.gg graph) | Alpha | [AGNT Workflow](/getting-started/workflows/agnt/) |

The authoritative, machine-readable list is the
[`GET /api/v1/workflow-formats`](https://operator.untra.io/schemas/openapi.json)
endpoint, derived from the same source of truth that backs this page.

## How to export

CLI:

```bash
operator workflow export FEAT-1234              # default: claude (.js)
operator workflow export FEAT-1234 --format agnt
```

REST (the web UI and VS Code use the same shared code path):

```bash
# Concrete ticket -> workflow
curl -X POST "http://localhost:7008/api/v1/tickets/FEAT-1234/workflow-export?format=claude"

# Issue type alone -> preview (placeholder values, no ticket required)
curl "http://localhost:7008/api/v1/issuetypes/FEAT/workflow-preview?format=agnt"

# Discover the available formats
curl "http://localhost:7008/api/v1/workflow-formats"
```

In the TUI, web UI, and VS Code, the **Workflows** section lists the formats and
links to preview/export.

## Voting and judging

Fan-out steps run several agents and then keep one answer. How that answer is chosen at runtime:

| Step | Setting | Runtime selection |
|---|---|---|
| `multi_model` | `voting_mode = "single_judge"` (default) | Deterministic rule (LLM judge planned) |
| `multi_model` | `voting_mode = "multi_voter"` | Deterministic rule (voting round not yet run) |
| `multi_prompt` | `selection_strategy = "model_choice"` | First variation (LLM judge planned) |
| `multi_prompt` | `selection_strategy = "scored"` | First variation |
| `matrixed` | - | No aggregation yet (`value` is `null`) |

The **deterministic rule** is: first delegator with output for `majority` /
`ranked`, the longest answer for `unanimous`.

**Planned: LLM judge.** A judge that picks the winner with a single typed API
call from the Operator daemon (not an agent session), guided by `voting_prompt`
or `selection_prompt`, is implemented but not yet included in release builds.
The configuration it will read is already accepted:

```toml
[native_llm.judge]
model_server = "anthropic-api"   # any declared or implicit model server
model = "claude-sonnet-5"         # full API model id, not a CLI alias
timeout_secs = 120                # optional
```

Until the judge ships, a configured judge is reported as unavailable in the
ticket history and the deterministic rule applies.

Exports render the same prompts as a vote/select `agent(...)` call instead.

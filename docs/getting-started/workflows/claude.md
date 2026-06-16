---
title: "Claude Workflow"
description: "Export an Operator ticket + issue type into a Claude Code dynamic workflow (.js)."
layout: doc
---

# Claude Workflow

The default export target. Renders a `ticket + issue type` into a **Claude Code
dynamic workflow** — a `.js` module the
[`@untra/naiveworkflow-compiler`](https://operator.untra.io/getting-started/workflows/)
walks to drive Claude Code agents.

```bash
operator workflow export FEAT-1234              # writes FEAT-1234.workflow.js
operator workflow export FEAT-1234 --format claude --out -   # to stdout
```

Or over REST:

```bash
curl -X POST "http://localhost:7008/api/v1/tickets/FEAT-1234/workflow-export?format=claude"
```

## Output shape

The emitted module is deterministic (no wallclock, `Date.now`, or
`Math.random`). It begins with an `export const meta = { name, description,
phases }` block, followed by **top-level statements** (one per step) — not a
wrapped `export default async function`, because that is the form the compiler
expects.

Each issue-type step maps to a construct:

| Step type | Emitted as |
|---|---|
| Task / Delegator | `agent()` call (`judge_loop()` for review gates) |
| Classifier | `agent()` with a `schema` option |
| MultiModel / MultiPrompt / Matrixed | `parallel([...])` with `await` binding |
| Pipeline | `await pipeline(items, ...stages)` |
| Rag / Mcp | `agent()` with a `GAP` marker (sandbox can't guarantee FS / tools) |

Human review gates become a bounded **judge loop** (max attempts + a voting
agent); the original `on_reject` target is preserved in a `GAP` marker.

---
title: "AGNT Workflow"
description: "Export an Operator ticket + issue type into an AGNT.gg workflow graph (.json)."
layout: doc
---

# AGNT Workflow

Renders a `ticket + issue type` into an [AGNT.gg](https://agnt.gg) **workflow
graph** — a `{ name, description, nodes, edges }` JSON document AGNT can import
and run.

```bash
operator workflow export FEAT-1234 --format agnt   # writes FEAT-1234.agnt.workflow.json
```

Or over REST (also used by the `operator-export-workflow` plugin node):

```bash
curl -X POST "http://localhost:7008/api/v1/tickets/FEAT-1234/workflow-export?format=agnt"
```

## Output shape

Each node carries `{ id, type, text, x, y, parameters }` (AGNT's runnable node
shape — `text` is the canvas label, `x`/`y` are coordinates, and `parameters` is
the bag AGNT resolves into the node's tool). Each issue-type step becomes one
node:

- **`operator-run-step`** — the generic node, carrying `{ ticket, step, prompt,
  model, … }` in its `parameters`; it calls Operator's launch endpoint. (The
  `prompt` is an inert annotation — Operator owns the prompt internally; the tool
  reads only `ticket`/`model`.)
- **`agnt-agent`** — AGNT's native agent-chat node, emitted when a delegator is
  an AGNT-hosted remote agent (`remote_agent.platform == "agnt"`), carrying
  `agentId` (the agent's UUID) and `message` (the prompt) so AGNT runs the step
  itself instead of calling back into Operator.

The `next_step` chain becomes `edges`, each `{ id, start: { id }, end: { id } }`.
Fan-out shapes (MultiModel / Matrixed / Pipeline) flatten to a single node, and
human review gates, RAG, and MCP requirements are recorded in a `gap` field —
lossy conversions are annotated, not dropped silently.

This is one half of the broader
[AGNT integration](https://operator.untra.io/getting-started/integrations/agnt/);
AGNT-the-plugin (the `operator-*` node vocabulary that drives Operator) is the
other.

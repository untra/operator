---
title: "AGNT.gg"
description: "Export Operator workflows to AGNT.gg and drive Operator from AGNT workflows."
layout: doc
---

# AGNT.gg

[AGNT.gg](https://agnt.gg) is a local-first agent operating system: a desktop
app + local runtime with visual graph workflows, agents, a plugin marketplace,
and native MCP support. Operator connects to AGNT in both directions.

## Operator → AGNT: export a workflow

Render any ticket (against its issuetype) into an AGNT workflow graph:

```bash
operator workflow export FEAT-1234 --format agnt
# writes FEAT-1234.agnt.workflow.json
```

Or over REST (also used by the `operator-export-workflow` plugin node):

```bash
curl -X POST "http://localhost:7008/api/v1/tickets/FEAT-1234/workflow-export?format=agnt"
```

The output is an AGNT `{ name, description, nodes, edges }` graph. Each operator
step becomes one `operator-run-step` node (carrying `{ ticket, step, prompt, … }`
in its config); the `next_step` chain becomes edges. This export runs in AGNT
once the `operator-plugin` (below) is installed, since `operator-run-step` is one
of that plugin's node types. Operator sequences a ticket's steps internally, so
the per-step nodes are a faithful **visualization** of the ticket's shape; running
the graph drives the one underlying Operator ticket.

> Like the Claude `.js` export, this is a one-way, lossy *flattening*. AGNT nodes
> can't reproduce Operator's terminal coding sessions, and human review gates /
> RAG / MCP / fan-out steps are recorded in each node's `config.gap` rather than
> faithfully executed. Treat the export as a runnable **scaffold**, not an
> equivalent.

## AGNT → Operator: the `operator-plugin`

The [`operator-plugin`](https://github.com/untra/operator/tree/main/agnt-plugin)
adds Operator nodes to AGNT's canvas. Each is a thin wrapper around Operator's
REST API:

| Node | Endpoint |
|------|----------|
| `operator-create-ticket`   | `POST /api/v1/tickets` |
| `operator-launch-agent`    | `POST /api/v1/tickets/{id}/launch` |
| `operator-run-step`        | `POST /api/v1/tickets/{ticket}/launch` (the export's node type) |
| `operator-queue-status`    | `GET  /api/v1/queue/status` |
| `operator-export-workflow` | `POST /api/v1/tickets/{id}/workflow-export` |
| `operator-alert`           | `POST /api/v1/alerts` |

### Install

1. Start Operator's REST API: `operator api` (default port `7008`).
2. Build the plugin from the AGNT repo:
   ```bash
   cp -r agnt-plugin /path/to/agnt/backend/plugins/dev/operator-plugin
   cd /path/to/agnt/backend/plugins && node build-plugin.js operator-plugin
   ```
3. Install `plugin-builds/operator-plugin.agnt` via AGNT's Marketplace UI (or drop
   it into `~/Library/Application Support/AGNT/plugins/installed/` and
   `POST http://localhost:3333/api/plugins/reload`).

Set each node's `operatorBaseUrl` (or the `OPERATOR_BASE_URL` env var) if Operator
isn't on the default `http://localhost:7008`.

### Example

```
webhook-trigger
  → operator-create-ticket   { template: "fix", project: "gamesvc", summary: "{{payload.title}}" }
  → operator-launch-agent    { id: "{{prev.result.id}}" }
  → operator-queue-status
  → slack-send
```

## Alternative: the MCP bridge (no plugin)

AGNT consumes stdio MCP servers natively. Register Operator without building a
plugin via AGNT's MCP settings:

```json
{ "name": "operator", "command": "operator", "args": ["mcp"] }
```

This surfaces Operator's ~18 MCP tools in AGNT immediately. The plugin's
advantages over the raw bridge are first-class canvas nodes with typed
parameters and marketplace discoverability — but the bridge is zero-build and
the same `operator mcp` server works with any MCP-capable platform.

## Trust & permissions

These integrations create tickets and launch agents against your repositories.
Only connect an AGNT instance you control, point `operatorBaseUrl` at a trusted
Operator API, and gate MCP write tools with `[mcp].expose_ticket_write_tools`.

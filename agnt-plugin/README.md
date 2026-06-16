# operator-plugin (AGNT.gg)

An [AGNT.gg](https://agnt.gg) plugin that exposes **Operator!**'s ticket
orchestration as workflow nodes. Drop these nodes into an AGNT workflow to
create tickets, launch coding agents, poll the queue, export workflows, and
raise investigations — all driven by Operator's local REST API.

This is the **AGNT → Operator** direction. The companion direction (Operator →
AGNT) is the `operator workflow export --format agnt` emitter built into
Operator, which emits graphs composed of the `operator-launch-agent` nodes this
plugin defines.

## Nodes

| Node | Operator REST endpoint |
|------|------------------------|
| `operator-create-ticket`   | `POST /api/v1/tickets` |
| `operator-launch-agent`    | `POST /api/v1/tickets/{id}/launch` (whole ticket, by `id`) |
| `operator-run-step`        | `POST /api/v1/tickets/{ticket}/launch` (the node type `--format agnt` emits) |
| `operator-queue-status`    | `GET  /api/v1/queue/status` |
| `operator-export-workflow` | `POST /api/v1/tickets/{id}/workflow-export?format=agnt` |
| `operator-alert`           | `POST /api/v1/alerts` |

Each node is a thin `fetch()` wrapper around an endpoint Operator already
serves (see `lib/operator-client.js`). The plugin is **zero-dependency** (Node
18+ global `fetch`).

## Configuring the Operator base URL

Every node accepts an `operatorBaseUrl` parameter. If omitted, the plugin uses
the `OPERATOR_BASE_URL` environment variable, then falls back to
`http://localhost:7008` (Operator's default `[rest_api].port`).

Start Operator's REST API with:

```bash
operator api
```

## Example workflow

```
webhook-trigger
  → operator-create-ticket   { template: "fix", project: "gamesvc", summary: "{{payload.title}}" }
  → operator-launch-agent    { id: "{{prev.result.id}}" }
  → operator-queue-status
  → slack-send
```

`operator-create-ticket` returns `{ id, filename, path }`, so the next node can
launch the freshly created ticket by id.

## Trust & permissions

These nodes drive a real Operator instance: they create tickets and launch
agents against your repositories. Only point `operatorBaseUrl` at an Operator
API you control and trust.

## Building the `.agnt` package

Packaging uses AGNT's bundled builder (it gzips the manifest + JS + any
`node_modules`). From a checkout of the AGNT repo:

```bash
# copy or symlink this directory into AGNT's plugin dev tree
cp -r agnt-plugin /path/to/agnt/backend/plugins/dev/operator-plugin
cd /path/to/agnt/backend/plugins
node build-plugin.js operator-plugin
# → plugin-builds/operator-plugin.agnt
```

Install the resulting `.agnt` via AGNT's Marketplace UI, or drop it into
`~/Library/Application Support/AGNT/plugins/installed/` and reload:

```bash
curl -X POST http://localhost:3333/api/plugins/reload
```

## Alternative: the MCP bridge (no plugin)

Operator also ships a stdio MCP server exposing ~18 orchestration tools. AGNT
consumes stdio MCP servers natively — register Operator without this plugin via
AGNT's MCP settings:

```json
{ "name": "operator", "command": "operator", "args": ["mcp"] }
```

The plugin's value over the raw MCP bridge is first-class canvas nodes with
typed parameters and marketplace discoverability. See
`docs/getting-started/integrations/agnt/` for the full comparison.

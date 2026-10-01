# operator-plugin (AGNT.gg)

An [AGNT.gg](https://agnt.gg) plugin that exposes **Operator!**'s ticket orchestration as workflow nodes.
Drop these nodes into an AGNT workflow to create tickets, launch coding agents, poll the queue, export workflows, and
raise investigations.

This is the **AGNT -> Operator** direction. The companion direction (Operator -> AGNT) is `operator workflow export --format agnt`, which emits one `operator-run-step` node per issuetype step. `operator-launch-agent` remains a separate node: it launches a whole ticket by `id`.

Operator exports AGNT workflows as runnable visual scaffolds of a ticket's execution shape, not as lossless equivalents of Operator's internal workflow semantics. Interactive AGNT agents should use Operator's MCP server against the Operator that holds their tickets. Visual workflows use these nodes. See `docs/getting-started/integrations/agnt.md`.

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

## Authentication

`OPERATOR_BASE_URL` defaults to `http://localhost:7008`.

| Deployment | Set this | What it is |
|------------|----------|------------|
| Loopback `operator api` | `OPERATOR_API_TOKEN` | Contents of `.tickets/operator/local-token`. Sent as a bearer. Replaced every server start. Absent when Operator is not bound to loopback. |
| Kubernetes or any other bind | `OPERATOR_ACCESS_KEY` | Service access key with `read`, `write`, and `execute`. Exchanged at `POST /api/v1/auth/token` (`grant_type` `operator:access-key`) for a 15-minute bearer. |

A 401 on `/api/v1/queue/status` means neither credential was presented. A certificate error means the AGNT process does not trust the server CA. Set `NODE_EXTRA_CA_CERTS`. The plugin keeps TLS verification on.

Node parameters `operatorApiToken` and `operatorAccessKey` override the env vars and are stored in the workflow file. Prefer the env vars.

## Example workflow

```
webhook-trigger
operator-create-ticket   { template: "fix", project: "gamesvc", summary: "{{payload.title}}" }
operator-launch-agent    { id: "{{prev.result.id}}" }
operator-queue-status
slack-send
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
# plugin-builds/operator-plugin.agnt
```

Install the resulting `.agnt` via AGNT's Marketplace UI, or drop it into
`~/Library/Application Support/AGNT/plugins/installed/` and reload:

```bash
curl -X POST http://localhost:3333/api/plugins/reload
```

## MCP for interactive agents

`operator mcp` is a local subprocess. It opens the tickets on the machine where AGNT spawns it. It does not call a remote Operator URL.

```json
{ "name": "operator", "command": "operator", "args": ["mcp"] }
```

A cluster Operator serves the same tools at the descriptor's `transport_url` (`GET /api/v1/mcp/sse`), and that stream requires the same bearer as the REST API. Write and launch tools stay off until `[mcp].expose_ticket_write_tools = true`. If AGNT can only register stdio, use these plugin nodes against the cluster. The comparison is in `docs/getting-started/integrations/agnt.md`.

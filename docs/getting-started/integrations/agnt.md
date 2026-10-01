---
title: "AGNT.gg"
description: "Export Operator workflows to AGNT.gg and drive Operator from AGNT workflows."
layout: doc
---

[AGNT.gg](https://agnt.gg) is a local-first agent operating system: a desktop
app + local runtime with visual graph workflows, agents, a plugin marketplace,
and native MCP support. Operator connects to AGNT in three different ways.

## Who calls what

| You are | Use | Talks to |
|---------|-----|----------|
| An interactive AGNT agent (Patricia) | Operator MCP tools | The Operator that holds the tickets she should see |
| An AGNT visual workflow | [`operator-plugin`](https://github.com/untra/operator/tree/main/agnt-plugin) nodes | Operator REST |
| An Operator ticket export | The emitted graph, as a scaffold | `operator-run-step` nodes back into Operator REST |

`operator mcp` is a local subprocess. It opens the tickets of the machine it runs on. It does not call a remote Operator URL. Desktop AGNT registered like this drives the local daemon:

```json
{ "name": "operator", "command": "operator", "args": ["mcp"] }
```

A cluster Operator exposes the same tools over HTTP MCP. `GET /api/v1/mcp/descriptor` returns `transport_url` (normally `https://<host>/api/v1/mcp/sse`). That stream requires the same bearer as the REST API. On that deployment set `[mcp].stdio_advertised = false`: the advertised stdio command is the path inside the pod.

Write and launch MCP tools stay disabled until `[mcp].expose_ticket_write_tools = true`. Read tools work without that flag. HTTP MCP is additionally limited to the scopes on the bearer. See [Authentication](/security/authentication/).

If AGNT can only register a stdio MCP server, an agent on a laptop cannot reach a cluster Operator through MCP. The authenticated plugin nodes are the cluster path. Do not point the stdio command at an `https://` URL.

## Operator → AGNT: export a workflow

Render any ticket (against its issuetype) into an AGNT workflow graph:

```bash
operator workflow export FEAT-1234 --format agnt
# writes FEAT-1234.agnt.workflow.json
```

Or over REST (also used by the `operator-export-workflow` plugin node). The route requires a bearer; see [Authentication](#authentication) below.

```bash
curl -X POST "http://localhost:7008/api/v1/tickets/FEAT-1234/workflow-export?format=agnt" \
  -H "Authorization: Bearer $OPERATOR_API_TOKEN"
```

The output is an AGNT `{ name, description, nodes, edges }` graph. Each operator step becomes one `operator-run-step` node (carrying `{ ticket, step, prompt, … }` in its parameters); the `next_step` chain becomes edges. This export runs in AGNT once the `operator-plugin` (below) is installed, since `operator-run-step` is one of that plugin's node types.

Operator exports AGNT workflows as runnable visual scaffolds of a ticket's execution shape, not as lossless equivalents of Operator's internal workflow semantics.

The graph is runnable once `operator-plugin` is installed, because each step is an `operator-run-step` node and the `next_step` chain is edges. Running a node calls Operator's launch endpoint for that one ticket. Operator still owns terminal sessions, human gates, RAG, MCP tool calls, delegation, and ticket state. Those show up in a node's `parameters.gap` (`OPERATOR-GAP: ...`) when the graph cannot perform them. There is no reverse import.

## AGNT → Operator: the `operator-plugin`

The [`operator-plugin`](https://github.com/untra/operator/tree/main/agnt-plugin) adds Operator nodes to AGNT's canvas. Each is a thin wrapper around Operator's REST API:

| Node | Endpoint |
|------|----------|
| `operator-create-ticket`   | `POST /api/v1/tickets` |
| `operator-launch-agent`    | `POST /api/v1/tickets/{id}/launch` |
| `operator-run-step`        | `POST /api/v1/tickets/{ticket}/launch` (the export's node type) |
| `operator-queue-status`    | `GET  /api/v1/queue/status` |
| `operator-export-workflow` | `POST /api/v1/tickets/{id}/workflow-export` |
| `operator-alert`           | `POST /api/v1/alerts` |

`operator-launch-agent` launches a whole ticket by `id`. The exporter emits `operator-run-step`, one node per issuetype step.

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

Set each node's `operatorBaseUrl` (or the `OPERATOR_BASE_URL` env var) if Operator isn't on the default `http://localhost:7008`.

Authentication is required on every API route. See the next section.

### Example

```
webhook-trigger
  → operator-create-ticket   { template: "fix", project: "gamesvc", summary: "{{payload.title}}" }
  → operator-launch-agent    { id: "{{prev.result.id}}" }
  → operator-queue-status
  → slack-send
```

## Authentication

Every `/api/` route requires a credential. `GET /api/v1/queue/status` returns 401 until one is presented. A self-signed certificate is a separate check: Node must trust the CA (`NODE_EXTRA_CA_CERTS=/path/to/ca.pem` in the AGNT process). The plugin does not disable TLS verification.

**Loopback Operator** (`operator api` on `127.0.0.1`). The server writes `.tickets/operator/local-token`, mode `0600`, and replaces it on every start. A non-loopback bind deletes that file. Put the file contents in `OPERATOR_API_TOKEN`. The plugin sends it as `Authorization: Bearer`. It is not an access key.

**Any other bind, including Kubernetes.** Create a service access key (admin scope) with scopes `read`, `write`, and `execute`. The secret is shown once. Put it in `OPERATOR_ACCESS_KEY` on the AGNT process. The plugin posts it to `POST /api/v1/auth/token`:

```json
{ "grant_type": "operator:access-key", "access_key": "<key>" }
```

and sends the returned `access_token` as the bearer. Access tokens last 15 minutes. The plugin exchanges the key again before expiry. The key is not a bearer; sending it in `Authorization` returns 401.

`operatorApiToken` and `operatorAccessKey` on a node override the env vars. AGNT stores node parameters in the workflow file, so prefer the env vars.

Create the key while logged in as admin. The response field `secret` is the access key, shown once:

```bash
curl -s -X POST "$OPERATOR_BASE_URL/api/v1/auth/keys" \
  -H "Authorization: Bearer $ADMIN_ACCESS_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"name":"agnt","scopes":["read","write","execute"],"expires_in_days":90}'
```

The full account, scope, and key model is in [Authentication](/security/authentication/). Cluster bootstrap is in [Kubernetes](/getting-started/platforms/kubernetes/).

## Trust & permissions

These integrations create tickets and launch agents against your repositories. Only connect an AGNT instance you control, point `operatorBaseUrl` at a trusted Operator API, and gate MCP write tools with `[mcp].expose_ticket_write_tools`.

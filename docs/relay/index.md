---
title: "Relay"
description: "Multi-agent peer-to-peer communication hub embedded in Operator."
layout: doc
---

<span class="operator-brand">Operator!</span> embeds a relay hub that lets agents launched for different tickets discover and message each other in real time. When a delegator sets `operator_relay = true` in its `launch_config`, Operator injects the `relay` MCP server into Claude Code launches for that delegator — provided the relay hub socket is available. Injection does **not** happen automatically for all launches; the global default is `relay.auto_inject_mcp = false`.

The MCP server runs as `opr8r relay` (a subcommand of the signed `opr8r` binary) so no additional executable needs to be signed or distributed. Codex and other tools receive the env vars but require manual MCP configuration.

## Tools shipped by Operator

Operator ships two complementary executables for agent orchestration:

### opr8r — step wrapper and API client

`opr8r` wraps LLM tool invocations (Claude Code, Codex, Gemini CLI) inside multi-step ticket workflows. It runs as the **parent process** of the LLM tool, intercepts its exit code, and reports step completion to the Operator REST API. The API then decides what happens next — another step, a review gate, or workflow completion.

```
opr8r --ticket-id FEAT-042 --step build -- claude --prompt "implement the feature"
         ↓ launches
       claude (child process)
         ↓ on exit, opr8r reports to
       Operator API  →  next step / review / done
```

See the [opr8r CLI reference](/docs/cli/) for full flag documentation.

### relay — MCP client for the relay hub

`relay` is the MCP stdio server that Operator ships so agents can communicate with each other. It runs as a **child process** of the LLM tool (spawned by the MCP host), connects to the relay hub over a Unix socket, and exposes five relay tools via the MCP protocol:

| Tool | Description |
|------|-------------|
| `relay_peers` | List other active agent sessions on this machine |
| `relay_ask` | Send a question to a named peer and await a reply |
| `relay_reply` | Reply to a pending ask |
| `relay_broadcast` | Send a message to all connected peers |
| `relay_rename` | Change this session's peer name |

`relay` is a subcommand of `opr8r` (`opr8r relay`) so it is distributed and code-signed as part of the same binary. No separate download is needed.

## How the hub works

The relay hub is a Unix socket server that runs inside the Operator process. Each agent registers as a named peer when it connects. Operator assigns each agent its ticket ID as its peer name, so agents can address each other by ticket.

```
operator process
└── RelayHub (Unix socket at $RELAY_HUB_SOCKET)
    ├── opr8r relay  ←  Claude agent "FEAT-001"
    │     relay_ask("FEAT-002", "have you finished the auth module?")
    │                                    ↓
    └── opr8r relay  ←  Claude agent "FEAT-002"
          relay_reply(ask_id, "yes, pushed to feat/auth")
```

The hub runs for the lifetime of the Operator process. Unlike the standalone `claude-relay` tool, there is no idle-shutdown timer — the hub stays up as long as Operator is running.

## Hub socket

The hub binds to a Unix domain socket. The path is resolved in this priority order (matching claude-relay's `data-dir.ts` so existing deployments need no changes):

| Priority | Source | Default |
|----------|--------|---------|
| 1 | `$RELAY_HUB_SOCKET` | — |
| 2 | `$CLAUDE_PLUGIN_DATA/hub.sock` | — |
| 3 | fallback | `~/.claude-relay/hub.sock` |

Operator exports `RELAY_HUB_SOCKET` automatically at startup, so every child process it spawns can find the hub. For Claude Code, Operator also writes a per-session `relay-mcp.json` and passes `--mcp-config <path>` at launch time, so `relay` starts automatically alongside the agent — no manual setup needed. For other tools, the socket env var is exported but MCP wiring requires manual configuration.

## Agent naming

When Operator launches an agent, it injects two env vars into the session:

```bash
export RELAY_HUB_SOCKET=/path/to/hub.sock
export RELAY_AGENT_NAME=FEAT-042
```

The agent connects to the hub and registers under the name `FEAT-042`. Any other agent that knows the ticket ID can address it directly.

For agents running standalone (not launched by Operator), `relay` reads `~/.claude/sessions/{ppid}.json` and uses whatever name the user sets with `/rename` in Claude Code.

## Environment variables

These variables are shared with the claude-relay ecosystem and intentionally do not use the `OPERATOR_` prefix, so existing tools pick them up automatically.

| Variable | Set by | Purpose |
|----------|--------|---------|
| `RELAY_HUB_SOCKET` | Operator at startup | Unix socket path for the relay hub |
| `RELAY_AGENT_NAME` | Operator at agent launch | Peer name registered for this session (ticket ID) |
| `CLAUDE_PLUGIN_DATA` | Claude Code | Secondary socket path fallback |
| `OPERATOR_RELAY` | User override | Path to `opr8r` binary when auto-discovery fails |

`RELAY_HUB_SOCKET` and `RELAY_AGENT_NAME` are the entry points for hooking into the relay system from outside Operator. Any process that sets these variables and speaks the relay wire protocol can participate as a peer.

## Wire compatibility

The protocol is byte-compatible with TypeScript claude-relay. Existing TS channels connect to the Rust hub unchanged:

- Same `PROTOCOL_VERSION = "2"`
- Same message type names: `register`, `rename`, `list_peers`, `ask`, `reply`, `broadcast`
- Same error codes: `peer_not_found`, `name_taken`, `timeout`, etc.
- Same line-delimited JSON framing over Unix socket

## See also

- [Claude agent setup](/docs/getting-started/agents/claude/)
- [Codex agent setup](/docs/getting-started/agents/codex/)
- [Delegators](/docs/delegators/) — named tool + model pairings that launch agents

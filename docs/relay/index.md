---
title: "Relay"
description: "Multi-agent peer-to-peer communication hub embedded in Operator."
layout: doc
---

<span class="operator-brand">Operator!</span> embeds a relay hub that lets agents launched for different tickets discover and message each other in real time. When the hub is running and `RELAY_HUB_SOCKET` is set, Operator automatically injects the `relay-channel` MCP server into Claude Code launches so agents can use the relay tools without manual configuration. Codex and other tools receive the env vars but require manual MCP configuration.

## How it works

The relay hub is a Unix socket server that runs inside the Operator process. Each agent registers as a named peer when it connects. Operator assigns each agent its ticket ID as its peer name, so agents can address each other by ticket.

```
operator process
└── RelayHub (Unix socket)
    ├── Claude agent "FEAT-001"  ──── ask ────→  Claude agent "FEAT-002"
    └── Claude agent "FEAT-002"  ◄─── reply ───  Claude agent "FEAT-001"
```

The hub runs for the lifetime of the Operator process. Unlike the standalone `claude-relay` tool, there is no idle-shutdown timer — the hub stays up as long as Operator is running.

## Hub socket

The hub binds to a Unix domain socket. The path is resolved in this priority order (matching claude-relay's `data-dir.ts` so existing deployments need no changes):

| Priority | Source | Default |
|----------|--------|---------|
| 1 | `$RELAY_HUB_SOCKET` | — |
| 2 | `$CLAUDE_PLUGIN_DATA/hub.sock` | — |
| 3 | fallback | `~/.claude-relay/hub.sock` |

Operator exports `RELAY_HUB_SOCKET` automatically at startup, so every child process it spawns can find the hub. For Claude Code, Operator also writes a per-session `relay-mcp.json` and passes `--mcp-config <path>` at launch time, so the relay-channel MCP server is active without any manual setup. For other tools, the socket env var is exported but MCP wiring requires manual configuration.

## Agent naming

When Operator launches an agent, it injects two env vars into the session:

```bash
export RELAY_HUB_SOCKET=/path/to/hub.sock
export RELAY_AGENT_NAME=FEAT-042
```

The agent connects to the hub and registers under the name `FEAT-042`. Any other agent that knows the ticket ID can address it directly.

For agents running standalone (not launched by Operator), the relay channel binary reads `~/.claude/sessions/{ppid}.json` and uses whatever name the user sets with `/rename` in Claude Code.

## Wire compatibility

The protocol is byte-compatible with TypeScript claude-relay. Existing TS channels connect to the Rust hub unchanged:

- Same `PROTOCOL_VERSION = "2"`
- Same message type names: `register`, `rename`, `list_peers`, `ask`, `reply`, `broadcast`
- Same error codes: `peer_not_found`, `name_taken`, `timeout`, etc.
- Same line-delimited JSON framing over Unix socket

## Environment variables

| Variable | Set by | Purpose |
|----------|--------|---------|
| `RELAY_HUB_SOCKET` | Operator at startup | Unix socket path for the hub |
| `RELAY_AGENT_NAME` | Operator at launch time | Peer name for the agent (ticket ID) |
| `CLAUDE_PLUGIN_DATA` | Claude Code | Secondary socket path fallback |

These variables intentionally do not use the `OPERATOR_` prefix — they are shared with the claude-relay ecosystem so existing tools pick them up automatically.

## See also

- [Claude agent setup](/docs/getting-started/agents/claude/)
- [Codex agent setup](/docs/getting-started/agents/codex/)
- [Delegators](/docs/delegators/) — named tool + model pairings that launch agents

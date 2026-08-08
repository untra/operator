---
layout: page
title: Operator & opr8r
parent: Architecture
nav_order: 2
published: false
---

Operator ships as **two executables built from one repository**: `operator`, a long-running server/TUI process, and `opr8r`, a small client binary that runs *inside* the terminal sessions Operator launches. This page describes the relationship between them and the two independent channels they use to talk to each other.

## Roles

| | `operator` | `opr8r` |
|---|---|---|
| **Shape** | Long-running process (TUI, REST API, queue, agent tracking) | Short-lived CLI, one invocation per step |
| **Runs where** | The operator's own terminal/host | Inside each agent's session (tmux/cmux/Zellij pane, VS Code terminal) |
| **Lifetime** | For the duration of the workspace | For the duration of a single ticket step |
| **Role in the relationship** | Server: owns ticket/queue state, exposes a REST API, hosts the relay hub | Client: wraps an LLM tool invocation, reports back over HTTP, optionally speaks MCP |
| **Binary size** | Full application (~tens of MB) | Optimized for size (~3–5 MB): stripped, LTO, single codegen unit, `panic = abort` |

`opr8r` is deliberately minimal so it can be signed and distributed as an independent artifact alongside `operator` releases and the VS Code extension, without needing the full application dependency tree.

## Two independent channels

Operator and opr8r never share memory or a socket file handle directly — everything crosses a process boundary. There are two distinct channels, used for two distinct purposes:

```
operator process
├── spawns ─────────────► LLM tool session (tmux pane, direct spawn, etc.)
│                          the ticket's initial prompt, project cwd, session id
│
├── REST API (HTTP, localhost) ◄──────── opr8r (step-wrapper mode)
│     POST /api/v1/tickets/{id}/steps/{step}/complete
│
└── RelayHub (Unix socket) ◄──────────── opr8r relay (MCP subcommand, child of the LLM tool)
      relay_ask / relay_reply / relay_broadcast / relay_peers / relay_rename
```

1. **Launch (operator → session, one-way, process spawn).** Operator decides what to run and starts it — see [Launching agents](#launching-agents).
2. **Step completion (opr8r → operator, HTTP).** After the wrapped LLM command exits, `opr8r` reports the result back to Operator's REST API — see [The REST channel](#the-rest-channel-opr8r-as-step-wrapper).
3. **Peer messaging (agent ↔ agent, via operator).** A *different* opr8r subcommand, `opr8r relay`, runs as an MCP server so the LLM tool itself can message other agents through Operator's relay hub. This is a separate feature covered in full on the [Relay](/docs/relay/) page — this doc only places it in the launch topology.

## Launching agents

Operator decides which LLM tool, model, and prompt to use (see the [LLM Tools](/docs/llm-tools/) and [Delegators](/docs/delegators/) references) and starts that process inside a session wrapper. Today the LLM tool is spawned directly — `opr8r` does not sit between Operator and the agent for a single, non-stepped launch.

Where `opr8r` becomes the parent process is **multi-step ticket workflows**, where a ticket's issuetype defines a sequence of steps (e.g. `plan` → `build` → `test`) and each step needs its completion reported before the next can run:

```bash
opr8r --ticket-id=FEAT-042 --step=build -- claude --prompt "implement the feature"
```

`opr8r` spawns the LLM command as a child process, tees its stdout/stderr through to the terminal so the human sees identical output to a direct launch, and waits for it to exit.

## The REST channel: opr8r as step-wrapper

When the wrapped LLM command exits, `opr8r` calls back into Operator's REST API:

```
POST /api/v1/tickets/{id}/steps/{step}/complete
```

Request body (`StepCompleteRequest`):

```json
{
  "exit_code": 0,
  "session_id": "uuid",
  "duration_secs": 342,
  "output": { "...": "parsed OPERATOR_STATUS block, if present" }
}
```

Operator's handler (`complete_step` in `src/rest/routes/launch.rs`) records the result against the ticket, resolves the issuetype's next step, and decides whether to auto-proceed: it does, only when the step's `review_type` is `none`. The response tells `opr8r` what to do next:

```json
{
  "status": "completed",
  "next_step": { "name": "test", "review_type": "none" },
  "auto_proceed": true,
  "next_command": "opr8r --ticket-id=FEAT-042 --step=test -- claude ..."
}
```

- If `auto_proceed` is true, `opr8r` `exec()`s the `next_command` — on Unix this replaces the current process image in place (same terminal, same pane, no new session), on Windows it spawns and waits since there's no `exec()` equivalent.
- If a review is required, `opr8r` prints an "awaiting review" banner and exits, leaving the terminal open for the operator to advance the ticket manually or via the TUI.
- `--no-auto-proceed` disables the exec regardless of what the server returns.

**Current status:** the endpoint, request/response contract, and `opr8r` chain-exec logic are implemented; the server-side construction of a fully general `next_command` for arbitrary next steps is still a placeholder in `complete_step` (see the `// For now, return a placeholder` comment in `src/rest/routes/launch.rs`) — treat multi-step auto-chaining as alpha until that lands.

### Discovering the API

`opr8r` never hardcodes a port. It resolves the Operator API base URL in this order (`opr8r/src/api.rs::resolve_base_url` / `discover`):

| Priority | Source | Used for |
|---|---|---|
| 1 | `--api-url` flag | Explicit override |
| 2 | `OPERATOR_API_URL` env var | Remote launches, where callbacks must route back through an SSH reverse tunnel |
| 3 | `.tickets/operator/api-session.json` | Local discovery — see below |
| 4 | `http://localhost:7008` | Default fallback |

Operator writes `api-session.json` (`{"port", "pid", "started_at", "version"}`) into `.tickets/operator/` when its REST server starts (`src/rest/server.rs::write_session_file`), and removes it on shutdown. This is the primary discovery mechanism: opr8r reads the file to find the live port without any configuration.

### Failure handling

`opr8r` retries `complete_step` with exponential backoff (3 attempts). If the API stays unreachable, it exits with code `3` and prints recovery steps (start `operator api`, check `api-session.json`, or advance the ticket manually from the TUI) rather than silently dropping the step result.

| Exit code | Meaning |
|---|---|
| `0` | Success |
| `1` | Wrapped LLM command failed |
| `3` | Operator API unreachable |
| `4` | Configuration error |
| `130` | Interrupted (SIGINT) |

## The relay channel: opr8r as MCP peer

Separately from step-wrapping, `opr8r relay` runs as an MCP stdio server — a *child* of the LLM tool rather than its parent — connecting to Operator's in-process relay hub over a Unix socket so agents on different tickets can message each other (`relay_ask`, `relay_reply`, `relay_broadcast`, `relay_peers`, `relay_rename`). Operator locates and injects this automatically for delegators with `operator_relay = true`. Full protocol, socket discovery, and wiring details live on the [Relay](/docs/relay/) page — this doc's scope is just where it sits in the launch/communication topology relative to the step-wrapper role above.

## Why one binary, two roles

`opr8r` step-wrapping and `opr8r relay` are both subcommands of the same binary (`relay` is a `Cmd::Relay` variant; step-wrapper mode is the default when no subcommand is given). This means only one small artifact needs to be built, signed, and bundled with Operator releases and the VS Code extension — there is no separate `operator-relay` binary to maintain (a legacy standalone `operator-relay` is still detected for backward compatibility, but is not produced by current builds).

## See also

- [Relay](/docs/relay/) — the MCP peer-to-peer protocol and hub, in full
- [CLI Reference](/docs/cli/) — `opr8r`'s full flag reference
- [Delegators](/docs/delegators/) — how Operator picks the LLM tool/model a session launches with
- [LLM Tools](/docs/llm-tools/) — how Operator detects and invokes CLI coding agents

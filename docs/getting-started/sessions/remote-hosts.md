---
title: "Remote Hosts (SSH)"
description: "Launch agent CLI processes on a remote machine over SSH while the Operator dashboard stays local."
layout: doc
---

# Remote Hosts (SSH)

Operator can launch an agent's CLI process on a **remote machine** while the
dashboard, queue, and tracking stay local. Declare a `[[hosts]]` entry and
reference it from a delegator's `launch_config`:

```toml
[[hosts]]
name = "gpu-vm"
ssh_alias = "gpu-vm"          # resolved via your ~/.ssh/config
workdir = "/srv/agents/my-project"
display_name = "GPU VM"

[[delegators]]
name = "claude-remote"
llm_tool = "claude"
model = "opus"
[delegators.launch_config]
host = "gpu-vm"
```

A host is deliberately distinct from a [model server](/configuration/): a `[[model_servers]]` entry says where model *inference* lives; a `[[hosts]]` entry says where the agent *CLI process* runs. A remote delegator can combine both.

## How it works

The local tmux (or cmux) pane Operator creates runs a generated wrapper script
that:

1. Ships the prompt file and run script to
   `{workdir}/.tickets/operator/` on the host over `ssh`
2. Execs `ssh -t` into a **remote tmux session** (named like the local one,
   `op-…`) that runs the agent
3. Opens an SSH **reverse tunnel** for the REST port, so `opr8r` step-completion
   callbacks from the remote side reach your local Operator at
   `http://localhost:{port}` — the API stays loopback-only on both machines

Because the tracked pane is local, screen scraping, attach, idle detection, and
send-keys all behave exactly as for local agents. The agent row shows an
`@{host}` annotation in the dashboard.

## Remote host requirements

- **SSH access** via an alias in `~/.ssh/config`, with key-based auth.
  Connect once manually first (`ssh gpu-vm`) to accept host keys — launches use
  `BatchMode`, which cannot answer interactive prompts.
- **tmux** installed on the remote PATH.
- **The agent CLI** (`claude`, `codex`, `gemini`) on the remote PATH, already
  authenticated there (e.g. remote `~/.claude` credentials).
- **The project checked out** at `workdir`.
- **API keys in the remote environment**: model-server keys are passed by
  reference (`export ANTHROPIC_API_KEY=${YOUR_VAR}`) and expand in the *remote*
  shell. Export them in a file sourced by non-interactive shells, or rely on
  the CLI's own auth.

Operator preflights all of this (reachability, tmux, tool, workdir) before
creating any session and fails the launch with a specific message if a check
fails.

## Disconnects and reconnecting

If the SSH link drops (laptop sleep, network change), the local pane dies and
the agent shows as dead — but the **remote tmux session and agent survive**.
Relaunch the ticket from the TUI: the wrapper regenerates and
`tmux new-session -A` reattaches the surviving remote session with scrollback
intact.

## limitations

- **No git worktrees** for remote agents — the agent works directly in
  `workdir`, regardless of `use_worktrees`.
- **No hook signals or artifact detection** (both read the local filesystem);
  liveness relies on pane presence and screen content, the same posture cmux
  agents have.
- **No relay MCP injection** (the relay hub is a local Unix socket).
- **No docker mode** and **no zellij wrapper** with a remote host — both are
  rejected at resolution time.
- **One remote agent per host at a time** is the safe posture: concurrent
  agents to the same host would collide on the reverse-tunnel port, and the
  second launch fails loudly (`ExitOnForwardFailure`).
- Ticket files live on the local machine; remote agents signal progress through
  `opr8r` callbacks rather than moving ticket files.

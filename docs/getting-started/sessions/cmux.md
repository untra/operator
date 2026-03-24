---
title: "cmux Sessions"
description: "macOS terminal multiplexer integration for managing AI agent sessions in cmux workspaces."
layout: doc
---

# cmux Sessions

<span class="badge supported">Supported</span>

<span class="operator-brand">Operator!</span> supports [**cmux**](https://cmux.app){:target="_blank"}, a macOS terminal multiplexer, as a session management backend. When running inside cmux, Operator can launch and manage AI coding agents directly in cmux workspaces.

> **Note:** cmux support requires macOS and that the Operator process itself is running inside a cmux session. For cross-platform support, consider the [VS Code Extension](/getting-started/sessions/vscode/) or [tmux](/getting-started/sessions/tmux/).

## What is cmux?

cmux is a macOS-native terminal multiplexer that organizes work into **windows** and **workspaces**. Each workspace provides an isolated terminal environment within a window, similar to how tmux organizes sessions and panes — but with a native macOS interface.

Operator uses cmux workspaces to run LLM agent sessions, allowing you to focus and switch between agents without leaving your terminal environment.

## Prerequisites

1. **macOS** — cmux is a macOS-only application
2. **cmux installed** — by default, Operator looks for the binary at `/Applications/cmux.app/Contents/Resources/bin/cmux`
3. **Running inside cmux** — Operator must be launched from within a cmux session (the `CMUX_WORKSPACE_ID` environment variable must be present)

## Configuration

Set cmux as your session wrapper in `operator.toml`:

```toml
[sessions]
wrapper = "cmux"

[sessions.cmux]
binary_path = "/Applications/cmux.app/Contents/Resources/bin/cmux"
require_in_cmux = true
placement = "auto"
```

### Configuration Options

| Setting | Default | Description |
|---------|---------|-------------|
| `binary_path` | `/Applications/cmux.app/Contents/Resources/bin/cmux` | Path to the cmux CLI binary |
| `require_in_cmux` | `true` | Require Operator to be running inside cmux |
| `placement` | `"auto"` | Where to place new agent sessions (see below) |

## Placement Policy

The `placement` setting controls how Operator creates new agent sessions:

| Policy | Behavior |
|--------|----------|
| `auto` | **0–1 open windows**: creates a new workspace in the active window. **>1 open windows**: creates a new window for the ticket. |
| `workspace` | Always creates a new workspace in the active window |
| `window` | Always creates a new window for each ticket |

**`auto`** (default) is recommended for most workflows. It keeps things simple when you have a single window, but avoids cluttering that window when you already have multiple windows open.

### Examples

```toml
# Always create a new workspace in the current window
[sessions.cmux]
placement = "workspace"

# Always create a dedicated window per ticket
[sessions.cmux]
placement = "window"

# Custom binary location
[sessions.cmux]
binary_path = "/usr/local/bin/cmux"
```

## How It Works

### Launching Agents

When Operator launches a ticket:

1. Checks that cmux is available and Operator is running inside cmux
2. Applies the configured placement policy to determine where to create the session
3. Creates a new workspace (and optionally a new window) named `op-{TICKET_ID}`
4. Sends the LLM agent command to the workspace

### Focusing Agents

When you press Enter on an agent in the TUI, Operator focuses the corresponding cmux workspace. Unlike tmux, this does **not** suspend the TUI — cmux handles window/workspace focus natively.

### Session Preview

Press `p` on an agent to preview its terminal content directly in the TUI. Operator reads the cmux workspace screen content without needing to switch focus.

### Agent Switching

When a workflow step specifies a different agent (delegator), Operator gracefully exits the current agent and launches the new one in the same cmux workspace using the same 3-tier escalation as tmux (`/exit` → `Ctrl+C` → `Ctrl+D`).

## Troubleshooting

### "cmux binary not found"

Verify the binary exists at the configured path:

```bash
ls -la /Applications/cmux.app/Contents/Resources/bin/cmux
```

If cmux is installed elsewhere, update `sessions.cmux.binary_path` in your config.

### "Not running inside cmux"

Operator requires the `CMUX_WORKSPACE_ID` environment variable to be set, which is present when running inside a cmux session. Launch Operator from within cmux:

```bash
# Inside a cmux workspace
operator
```

If you want to bypass this check (not recommended), set:

```toml
[sessions.cmux]
require_in_cmux = false
```

### Agent not appearing in cmux

1. Check that the placement policy created the workspace in the expected location
2. Use `placement = "window"` to ensure each agent gets its own visible window
3. Check Operator logs for cmux-related errors

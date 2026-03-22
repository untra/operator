---
title: "Zellij Sessions"
description: "Zellij terminal workspace manager integration for managing AI agent sessions in Zellij tabs."
layout: doc
---

# Zellij Sessions

<span class="badge supported">Supported</span>

<span class="operator-brand">Operator!</span> supports [**Zellij**](https://zellij.dev){:target="_blank"}, a terminal workspace manager, as a session management backend. When running inside Zellij, Operator can launch and manage AI coding agents in dedicated Zellij tabs.

> **Note:** Zellij support requires that the Operator process itself is running inside a Zellij session. Zellij works on macOS and Linux.

## What is Zellij?

Zellij is a terminal workspace manager written in Rust. It organizes work into **sessions**, **tabs**, and **panes** — a 3-tier hierarchy that provides flexible terminal management with a modern interface.

Operator uses Zellij tabs to run LLM agent sessions, with each agent getting its own dedicated tab. This keeps agents isolated while letting you switch between them using Zellij's native tab navigation.

## Prerequisites

1. **Zellij installed** — install via your package manager or from [zellij.dev](https://zellij.dev)
2. **Running inside Zellij** — Operator must be launched from within a Zellij session (the `ZELLIJ` environment variable must be present)

## Configuration

Set Zellij as your session wrapper in `operator.toml`:

```toml
[sessions]
wrapper = "zellij"

[sessions.zellij]
require_in_zellij = true
```

### Configuration Options

| Setting | Default | Description |
|---------|---------|-------------|
| `require_in_zellij` | `true` | Require Operator to be running inside Zellij |

## How It Works

### Launching Agents

When Operator launches a ticket:

1. Checks that Zellij is available and Operator is running inside Zellij
2. Creates a new tab named `op:{PROJECT}:{TICKET_ID}` with the project directory as the working directory
3. Sends the LLM agent command to the tab

### Focusing Agents

When you press Enter on an agent in the TUI, Operator focuses the corresponding Zellij tab. Like cmux, this does **not** suspend the TUI — Zellij handles tab focus natively.

### Session Preview

Press `p` on an agent to preview its terminal content directly in the TUI. Operator captures the Zellij tab's screen content via `zellij action dump-screen`.

> **Note:** Screen capture requires briefly focusing the target tab. You may notice a momentary tab switch during preview or health monitoring.

### Agent Switching

When a workflow step specifies a different agent (delegator), Operator gracefully exits the current agent and launches the new one in the same Zellij tab using the 3-tier escalation (`/exit` → `Ctrl+C` → `Ctrl+D`).

## Known Limitations

- **Screen capture requires focus:** Zellij's `dump-screen` command captures the currently focused pane. Operator must briefly switch tabs to capture content, which may cause a momentary visual flicker.
- **Tab operations require focus:** Closing a tab or sending text requires first focusing the tab. There is a potential race condition if you manually switch tabs at the same moment.

## Troubleshooting

### "zellij is not installed"

Verify Zellij is installed and on your PATH:

```bash
zellij --version
```

Install Zellij if needed:

```bash
# macOS
brew install zellij

# Linux (cargo)
cargo install --locked zellij

# Linux (package managers)
# See https://zellij.dev/documentation/installation
```

### "Not running inside Zellij"

Operator requires the `ZELLIJ` environment variable to be set, which is present when running inside a Zellij session. Launch Operator from within Zellij:

```bash
# Start a Zellij session first
zellij

# Then run Operator inside it
operator
```

If you want to bypass this check (not recommended), set:

```toml
[sessions.zellij]
require_in_zellij = false
```

### Agent tab not appearing

1. Check that you're running Operator inside a Zellij session
2. Look at Zellij's tab bar for tabs prefixed with `op:`
3. Check Operator logs for zellij-related errors

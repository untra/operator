---
title: "VS Code Extension"
description: "VS Code terminal integration for Operator multi-agent orchestration."
layout: doc
---

<span class="badge recommended">Recommended</span>

<a href="https://marketplace.visualstudio.com/items?itemName=untra.operator-terminals" target="_blank" class="button">Install from VS Code Marketplace</a>

Operator Terminals brings the Operator multi-agent orchestration experience directly into VS Code with integrated terminal management and ticket tracking. Its [VS Code session controller](/getting-started/sessions/vscode-terminals/) supports **macOS**, **Linux**, and **Windows** (no WSL required). Select the `vscode` controller for IDE terminals; tmux, cmux, and Zellij manage their own terminal environments.

## Features

- **Sidebar Integration**: View Queue, In Progress, and Completed tickets directly in VS Code
- **Styled Terminals**: Color-coded terminals by ticket type
  - FEAT (cyan, sparkle icon)
  - FIX (red, wrench icon)
  - TASK (green, tasklist icon)
  - SPIKE (magenta, beaker icon)
  - INV (yellow, search icon)
- **Activity Tracking**: Monitors shell execution to detect idle/running states
- **Webhook Server**: Local HTTP server for Operator communication

## Installation

### From Marketplace (Recommended)

1. Open VS Code
2. Go to Extensions (`Ctrl+Shift+X` / `Cmd+Shift+X`)
3. Search for "Operator Terminals"
4. Click Install

Or install directly: [VS Code Marketplace](https://marketplace.visualstudio.com/items?itemName=untra.operator-terminals){:target="_blank"}

### Manual Installation

1. Download the `.vsix` file from [GitHub releases](https://github.com/untra/operator/releases){:target="_blank"}
2. In VS Code, go to Extensions
3. Click the "..." menu
4. Select "Install from VSIX..."

## Configuration

| Setting | Default | Description |
|---------|---------|-------------|
| `operator.webhookPort` | `7009` | Port for webhook server |
| `operator.autoStart` | `true` | Start server on VS Code launch |
| `operator.terminalPrefix` | `op-` | Prefix for managed terminal names |
| `operator.ticketsDir` | `.tickets` | Path to tickets directory |
| `operator.apiUrl` | `http://localhost:7008` | Operator REST API URL |

## Commands

Access via Command Palette (`Ctrl+Shift+P` / `Cmd+Shift+P`):

| Command | Description |
|---------|-------------|
| `Operator: Start Webhook Server` | Start the webhook server |
| `Operator: Stop Webhook Server` | Stop the webhook server |
| `Operator: Show Server Status` | Display server status |
| `Operator: Launch Ticket` | Launch a ticket in a new terminal |
| `Operator: Launch Ticket (with options)` | Launch with agent/mode selection |
| `Operator: Download Operator` | Download the Operator CLI |
| `Operator: Connect MCP Server` | Register Operator as an MCP server for this editor, workspace `.mcp.json`, and optionally Copilot Global |

## MCP Integration

The sidebar launches and watches tickets. MCP is the other direction: an agent in the Copilot harness or Copilot CLI asks Operator about that same queue.

`Operator: Connect MCP Server` writes:

1. Workspace `mcp.servers` (`.vscode/mcp.json`) so the editor's own MCP list still sees Operator. Stdio when the daemon advertises it, otherwise SSE.
2. Workspace-root `.mcp.json` with `mcpServers.operator` (`command`, `args`, `cwd`) when the descriptor includes stdio. VS Code 1.140 Agent Host and Copilot CLI read this file directly.
3. An optional **Also write Copilot Global** action that merges the same `mcpServers.operator` block into `$COPILOT_HOME/mcp-config.json`, or `~/.copilot/mcp-config.json` when `COPILOT_HOME` is unset. That file is machine-wide, so the next Copilot session in any folder can see the queue.

Default MCP tools are read-only: health, status, issue types, collections, skills, and `operator_list_tickets`. Create, launch, and review require `[mcp].expose_ticket_write_tools = true` on the daemon.

Cluster daemons with `[mcp].stdio_advertised = false` stay on SSE in `.vscode/mcp.json`. Portable files are skipped so a local `operator mcp` is not pointed at a different process.

`.mcp.json` holds an absolute machine-local binary path. Do not commit it.

The Status MCP row lists which of those files contain `operator`.

## Sidebar Views

The extension adds an Operator sidebar with four views:

1. **Status**: Server status and connection info
2. **In Progress**: Currently running agent sessions
3. **Queue**: Pending tickets waiting to be launched
4. **Completed**: Recently completed tickets (collapsed by default)

## API Endpoints

The extension exposes a local HTTP API for Operator communication:

| Endpoint | Method | Description |
|----------|--------|-------------|
| `GET /health` | GET | Server health check |
| `POST /terminal/create` | POST | Create a new terminal |
| `POST /terminal/:name/send` | POST | Send command to terminal |
| `POST /terminal/:name/show` | POST | Reveal terminal (keep focus) |
| `POST /terminal/:name/focus` | POST | Focus terminal (take focus) |
| `DELETE /terminal/:name/kill` | DELETE | Dispose terminal |
| `GET /terminal/:name/exists` | GET | Check if terminal exists |
| `GET /terminal/:name/activity` | GET | Get idle/running state |
| `GET /terminal/list` | GET | List all managed terminals |

## Requirements

- VS Code 1.140.0 or later
- Operator CLI (for full functionality)

## Troubleshooting

### Server won't start

Check if another process is using the configured port:

```bash
lsof -i :7009
```

Try a different port in settings: `operator.webhookPort`.

### Terminals not appearing in sidebar

1. Ensure the webhook server is running (check Status view)
2. Verify `operator.ticketsDir` points to your tickets directory
3. Refresh the views using the refresh button

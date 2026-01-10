---
title: "VS Code Extension"
description: "VS Code terminal integration for Operator multi-agent orchestration."
layout: doc
---

# VS Code Extension

<a href="https://marketplace.visualstudio.com/items?itemName=untra.operator-terminals" target="_blank" class="button">Install from VS Code Marketplace</a>

Operator Terminals brings the Operator multi-agent orchestration experience directly into VS Code with integrated terminal management and ticket tracking.

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

- VS Code 1.85.0 or later
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

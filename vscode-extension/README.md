# Operator Terminals

VS Code terminal integration for [Operator](https://github.com/untra/operator) multi-agent orchestration.

## Features

- **Styled Terminals**: Terminals are color-coded and have icons based on ticket type
  - FEAT (cyan, sparkle icon)
  - FIX (red, wrench icon)
  - TASK (green, tasklist icon)
  - SPIKE (magenta, beaker icon)
  - INV (yellow, search icon)

- **Activity Tracking**: Monitors shell execution to detect idle/running states

- **Webhook Server**: Local HTTP server for Operator communication

## Installation

### From VS Code Marketplace

1. Open VS Code
2. Go to Extensions (Ctrl+Shift+X)
3. Search for "Operator Terminals"
4. Click Install

### Manual Installation

1. Download the `.vsix` file from [releases](https://github.com/untra/operator/releases)
2. In VS Code, go to Extensions
3. Click the "..." menu
4. Select "Install from VSIX..."

## Configuration

| Setting | Default | Description |
|---------|---------|-------------|
| `operator.webhookPort` | `7009` | Port for webhook server |
| `operator.autoStart` | `true` | Start server on VS Code launch |
| `operator.terminalPrefix` | `op-` | Prefix for managed terminal names |

## Commands

- **Operator: Start Webhook Server** - Start the webhook server
- **Operator: Stop Webhook Server** - Stop the webhook server
- **Operator: Show Server Status** - Display server status and terminal count

## API Endpoints

The extension exposes a local HTTP API for Operator to manage terminals:

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

## Development

```bash
# Install dependencies
npm install

# Compile TypeScript
npm run compile

# Watch for changes
npm run watch

# Run linter
npm run lint

# Run tests
npm test

# Package extension
npm run package
```

## License

MIT License - see [LICENSE](LICENSE) for details.

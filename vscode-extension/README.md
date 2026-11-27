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

### Cursor

This extension also runs in [Cursor](https://www.cursor.com). Install it from the [OpenVSX registry](https://open-vsx.org/) (Cursor's default extension search) or via the `.vsix` from GitHub releases.

## Connect to Operator

Run **Operator: Choose Connection**. Select **Off** to keep Operator disabled for this workspace. Select **Local** for the single-user Free service. For Premium, select **Remote**, enter your administrator's HTTPS origin and run **Operator: Sign In**. User settings can prefill the HTTPS address. A trusted folder with its own `.tickets/operator/config.toml` connects locally. Other folders stay idle until you choose. Use **Disconnect This Workspace** to detach a project. During sign-in, **Connections** lets you reopen the browser approval page, copy the code or cancel.

## Configuration

| Setting | Default | Description |
|---------|---------|-------------|
| `operator.webhookPort` | `7009` | Port for webhook server |
| `operator.autoStart` | `true` | Start the local API when none is reachable |
| `operator.terminalPrefix` | `op-` | Deprecated. Operator assigns terminal names. |
| `operator.connectionMode` | Unset | Preferred choice in the workspace connection picker |
| `operator.apiUrl` | Unset | Discover the local API or use an approved HTTPS origin |
| `operator.workingDirectory` | Unset | Working directory for the local Operator process |
| `operator.ticketsDir` | `.tickets` | Local tickets path, absolute or relative to the working directory |
| `operator.operatorPath` | Unset | Explicit local Operator executable |
| `operator.opr8rPath` | Unset | Explicit local opr8r executable |

## Commands

- **Operator: Start Webhook Server** - Start the webhook server
- **Operator: Stop Webhook Server** - Stop the webhook server
- **Operator: Show Server Status** - Display server status and terminal count
- **Operator: Sign In** - Authorize this editor with a remote Operator daemon through the browser (a local daemon needs no sign-in)
- **Operator: Sign Out** - Forget the stored credential for the current daemon
- **Operator: Connect MCP Server** - Register Operator in workspace MCP settings, workspace `.mcp.json`, and optionally Copilot Global

## API Endpoints

In local mode, each editor window registers its loopback port and a dedicated bearer under `~/.operator/vscode-sessions/`. Terminal operations require that window's bearer in the HTTP Authorization header. The registration is private to the OS user and is removed when the listener stops. Browser-origin requests are rejected.

For a hosted session, use **Operator: Attach to Agent Terminal**. The signed-in account needs the `agents:focus` capability for the project and SSH access to the agent host. The server supplies its SSH alias and workspace path. VS Code opens that Remote-SSH workspace and redeems a single-use reference before attaching. Configure the server's SSH alias in your local SSH configuration.

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

Run `npm run test:e2e -- secret-storage` on macOS to check credential persistence across two editor launches. The runner creates and unlocks a keychain in a temporary home and removes it at teardown. Set `OPERATOR_E2E_VSCODE_PATH` to use an installed VS Code executable.

## License

MIT License - see [LICENSE](LICENSE) for details.

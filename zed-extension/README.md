# Operator Zed Extension

Zed extension providing slash commands for interacting with [Operator](https://operator.untra.io), a multi-agent orchestration system for Claude Code.

## Features

This extension adds 11 slash commands to Zed's AI assistant for managing Operator:

| Command | Description |
|---------|-------------|
| `/op-status` | Show Operator health and status |
| `/op-queue` | List tickets in queue |
| `/op-launch TICKET-ID` | Launch a ticket |
| `/op-active` | List active agents |
| `/op-completed` | List recently completed tickets |
| `/op-ticket TICKET-ID` | Show ticket details |
| `/op-pause` | Pause queue processing |
| `/op-resume` | Resume queue processing |
| `/op-sync` | Sync kanban collections |
| `/op-approve AGENT-ID` | Approve agent review |
| `/op-reject AGENT-ID REASON` | Reject agent review |

## Prerequisites

- [Operator](https://operator.untra.io) installed and running (`operator api`)
- Rust toolchain with `wasm32-wasip1` target
- Zed editor

### Installing Rust WASM Target

```bash
rustup target add wasm32-wasip1
```

## Building

```bash
cd zed-extension
cargo build --release --target wasm32-wasip1
```

The compiled extension will be at `target/wasm32-wasip1/release/operator_zed.wasm`.

## Installation (Development)

1. Build the extension:
   ```bash
   cargo build --release --target wasm32-wasip1
   ```

2. Create a dev extension directory in Zed's extensions folder:
   ```bash
   mkdir -p ~/.local/share/zed/extensions/installed/operator-dev
   ```

3. Copy the extension files:
   ```bash
   cp extension.toml ~/.local/share/zed/extensions/installed/operator-dev/
   cp target/wasm32-wasip1/release/operator_zed.wasm ~/.local/share/zed/extensions/installed/operator-dev/extension.wasm
   ```

4. Restart Zed or use **Extensions: Reload** command

## Usage

1. Start the Operator API server:
   ```bash
   operator api
   ```

2. Open Zed's AI assistant panel (Cmd+Shift+A or Ctrl+Shift+A)

3. Type a slash command to interact with Operator:
   ```
   /op-status
   ```

4. Commands with arguments support autocompletion:
   ```
   /op-launch FIX-    # Tab to autocomplete ticket IDs
   ```

### Example Workflow

```
User: /op-status
Assistant: [Shows Operator status with queue count, active agents, etc.]

User: /op-queue
Assistant: [Lists tickets in queue with ID, project, type, title]

User: /op-launch FIX-123
Assistant: [Launches the ticket and shows the command to run]

User: /op-active
Assistant: [Shows running agents with their status]
```

## Architecture

### WASM Sandbox Limitations

Zed extensions run in a WebAssembly sandbox with limited capabilities:

- **No native HTTP**: We use `curl` subprocess calls to communicate with the Operator REST API
- **No sidebar views**: UI is limited to slash command output in the AI assistant
- **No status bar**: Cannot show persistent status indicators
- **No webhooks**: Cannot receive callbacks from Operator

### Communication Flow

```
Zed AI Assistant
    │
    ├──[slash command]──▶ Extension (WASM)
    │                         │
    │                         ├──[subprocess]──▶ curl
    │                         │                    │
    │                         │                    ▼
    │                         │              Operator API
    │                         │              (localhost:7008)
    │                         │                    │
    │                         ◀──[JSON response]───┘
    │                         │
    ◀──[markdown output]──────┘
```

## Alternative: Tasks

For actions that require a terminal, you can configure Zed tasks in `.zed/tasks.json`:

```json
[
  {
    "label": "Operator: Start API",
    "command": "operator api",
    "use_new_terminal": true
  },
  {
    "label": "Operator: Show Queue",
    "command": "operator queue",
    "use_new_terminal": false
  },
  {
    "label": "Operator: Launch Next",
    "command": "operator launch --next",
    "use_new_terminal": true
  }
]
```

## Configuration

The extension connects to `http://localhost:7008` by default. This matches Operator's default API port.

To use a different API URL, you would need to modify the `DEFAULT_API_URL` constant in `src/lib.rs` and rebuild.

## Troubleshooting

### "Failed to execute curl"

Ensure `curl` is available in your PATH. On most systems it's pre-installed.

### "API request failed"

1. Check that Operator is running: `operator api`
2. Verify the API is accessible: `curl http://localhost:7008/api/v1/health`
3. Check Operator logs for errors

### Extension not appearing

1. Verify the extension files are in the correct location
2. Check Zed's extension logs: **View > Output > Extensions**
3. Try reloading extensions or restarting Zed

## License

MIT License - see [LICENSE](LICENSE)

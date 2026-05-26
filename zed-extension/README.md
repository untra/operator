# Operator Zed Extension

Zed extension for [Operator](https://operator.untra.io), a multi-agent orchestration system for Claude Code. Provides three integration layers:

1. **MCP Context Server** — registers `operator mcp` natively so all operator tools and ticket resources appear in Zed's Agent Panel
2. **ACP Agent Setup** — `/op-setup-agent` generates the config to register Operator as an ACP agent server
3. **Slash Commands** — thin AI/human inference layer for quick operations with tab completion

## Prerequisites

- [Operator](https://operator.untra.io) binary installed and on PATH
- Operator API server running (`operator api`)
- Zed editor

## Installation (Development)

```bash
# Install WASM target
rustup target add wasm32-wasip1

# Build
cd zed-extension
cargo build --release --target wasm32-wasip1

# Install as dev extension
mkdir -p ~/.local/share/zed/extensions/installed/operator-dev
cp extension.toml ~/.local/share/zed/extensions/installed/operator-dev/
cp target/wasm32-wasip1/release/operator_zed.wasm ~/.local/share/zed/extensions/installed/operator-dev/extension.wasm

# Restart Zed or use Extensions: Reload
```

## Setup

### MCP Context Server (automatic)

After installing the extension, Zed automatically launches `operator mcp` as a context server. All MCP tools appear in the Agent Panel:

- `operator_health` / `operator_status` — system health
- `operator_list_tickets` — query queue, in-progress, completed tickets
- `operator_claim_ticket` / `operator_complete_ticket` / `operator_return_to_queue` — ticket lifecycle
- `operator_create_ticket` — create tickets from templates
- `operator_list_issue_types` / `operator_list_collections` / `operator_list_skills` — registry queries
- `operator_launch_ticket` / `operator_pause_queue` / `operator_resume_queue` — queue operations
- `operator_approve_agent` / `operator_reject_agent` — review actions

If the `operator` binary is not found, the extension shows installation instructions.

### ACP Agent Server (one-time setup)

Run `/op-setup-agent` in the AI assistant to generate the config snippet, then paste it into `~/.config/zed/settings.json`. After restarting Zed, Operator appears as an agent in the Agent Panel — you can send prompts that flow through ACP to a Claude Code delegator.

## Slash Commands

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
| `/op-setup-agent` | Generate ACP agent server config |

Commands with arguments support tab-completion from live API data.

## Architecture

```
Zed Agent Panel
    │
    ├── MCP Context Server ──▶ operator mcp (stdio)
    │   └── Tools + Resources available natively
    │
    ├── ACP Agent Server ──▶ operator acp (stdio)
    │   └── Prompts → delegator (Claude Code) → streaming output
    │
    └── Slash Commands ──▶ Extension (WASM)
        └── curl subprocess ──▶ Operator REST API (localhost:7008)
```

## Configuration

The MCP context server finds the `operator` binary on PATH, in `~/.cargo/bin/`, or at common install locations. The REST API URL for slash commands defaults to `http://localhost:7008`.

## Troubleshooting

### MCP tools not appearing

1. Verify `operator` is on PATH: `which operator`
2. Test MCP server: `operator mcp` (should wait for JSON-RPC input)
3. Check Zed's extension logs: **View > Output > Extensions**

### Slash commands failing

1. Check that Operator API is running: `operator api`
2. Verify connectivity: `curl http://localhost:7008/api/v1/health`
3. Ensure `curl` is available

### Extension not appearing

1. Verify files are in `~/.local/share/zed/extensions/installed/operator-dev/`
2. Reload extensions or restart Zed

## License

MIT License - see [LICENSE](LICENSE)

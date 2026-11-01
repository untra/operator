# opr8r

Minimal CLI wrapper for LLM commands (claude, gemini, codex) in multi-step ticket workflows.

## Overview

`opr8r` wraps LLM tool commands to orchestrate multi-step ticket workflows. It:
- Spawns the LLM command as a subprocess
- Passes through stdout/stderr to the terminal
- Reports step completion to the Operator API
- Automatically transitions to the next step when `review_type=none`

For a remote inference host such as a Windows PC running LM Studio, `opr8r`
stays beside the agent CLI on the Linux execution host. See the [LM Studio walkthrough](../docs/getting-started/model-servers/lmstudio.md).

## Usage

```bash
opr8r --ticket-id=FEAT-123 --step=plan -- claude --prompt 'Plan the feature'
```

### Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `--ticket-id` | Yes | Ticket ID being worked (e.g., FEAT-123) |
| `--step` | Yes | Current step name (e.g., "plan", "build") |
| `--api-url` | No | Operator API URL. Falls back to `OPERATOR_API_URL`, then the port in `.tickets/operator/api-session.json`, then `http://localhost:7008` |
| `--profile-id` | No | Operator configuration UUID. Defaults to `OPERATOR_PROFILE_ID` |
| `--session-id` | No | Session ID for LLM session tracking |
| `--no-auto-proceed` | No | Disable automatic step transition |
| `--verbose` | No | Enable verbose logging to stderr |
| `--dry-run` | No | Show what would happen without executing |
| `-- <COMMAND>` | Yes | The LLM command to execute |

### Subcommands

| Subcommand | Description |
|------------|-------------|
| `opr8r relay` | Run as an MCP stdio relay server exposing `relay_peers`, `relay_ask`, `relay_reply`, `relay_broadcast`, and `relay_rename`. Unix only (uses Unix domain sockets) |
| `opr8r tools --json` | Read a JSON array of `{name, version_command, health_command?}` on stdin and print which LLM CLIs exist on this host. Operator runs it over SSH to inventory a remote target |

### Environment

Operator sets these when it launches a step; you rarely set them by hand.

| Variable | Purpose |
|----------|---------|
| `OPERATOR_API_URL` | API origin for callbacks. Remote launches point it at the SSH reverse tunnel |
| `OPERATOR_API_TOKEN` | Bearer token for the step-completion callback, scoped to one ticket and step |
| `OPERATOR_PROFILE_ID` | Operator configuration UUID, used when `--profile-id` is absent |

## Flow

```
opr8r → spawn LLM → tee stdout/stderr → wait for exit → call API → exec next step
```

1. Parse args, discover API endpoint
2. Spawn LLM command as subprocess
3. Pass-through stdout/stderr to terminal
4. Wait for exit code
5. Call `POST /api/v1/tickets/{id}/steps/{step}/complete`
6. If `auto_proceed=true` in response, `exec()` the next opr8r command
7. If review required, exit gracefully

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | LLM command failed |
| 3 | API unreachable |
| 4 | Configuration error |
| 130 | Interrupted (SIGINT) |

## API Integration

opr8r calls the Operator API to report step completion:

```
POST /api/v1/tickets/{id}/steps/{step}/complete
```

Request:
```json
{
  "exit_code": 0,
  "output_valid": true,
  "session_id": "uuid",
  "duration_secs": 342
}
```

Response:
```json
{
  "status": "completed",
  "next_step": { "name": "build", "review_type": "none" },
  "auto_proceed": true,
  "next_command": "opr8r --ticket-id=FEAT-123 --step=build -- claude ..."
}
```

## Building

```bash
cd opr8r
cargo build --release
```

The release binary is optimized for size (~3-5 MB) with:
- Strip symbols
- LTO enabled
- Single codegen unit
- Panic = abort

## Development

```bash
cargo fmt           # Format code
cargo clippy        # Lint
cargo test          # Run tests
```

## Distribution

opr8r is built for multiple platforms:
- Linux x86_64
- Linux ARM64
- macOS Apple Silicon
- Windows x86_64
- Windows ARM64

On Windows, the next-step command runs through `cmd /C` instead of `exec`. Operator still builds those commands for a POSIX shell, so Windows cannot yet act as an execution host; see [platform support](../docs/getting-started/platform-support.md).

Binaries are bundled with:
- Operator releases (as separate file)
- VSCode extension (platform-specific VSIX packages)

---
title: "Cursor"
description: "Cursor IDE integration via the operator-terminals VS Code extension and Cursor's native MCP support."
layout: doc
---

# Cursor

<span class="badge supported">Supported</span>

<a href="https://open-vsx.org/extension/untra/operator-terminals" target="_blank" class="button">Install from OpenVSX</a>

[Cursor](https://www.cursor.com) is a fork of VS Code that natively runs most VS Code extensions and adds its own MCP configuration surface. The same `operator-terminals` extension that powers the VS Code session manager runs in Cursor unmodified — the only difference is **where** the extension writes the MCP server entry.

## Two Integration Paths

### 1. Extension Path (recommended)

Install `operator-terminals` from OpenVSX (Cursor's default extension registry) or via a downloaded `.vsix`, then run `Operator: Connect MCP Server` from the command palette. Inside Cursor, the extension writes the operator MCP entry to `~/.cursor/mcp.json` instead of VS Code's workspace `mcp.servers` — Cursor's MCP UI only surfaces user-scope entries, so writing workspace config would have no effect.

This path also gives you the sidebar (Queue / In Progress / Completed), styled terminals, and the rest of the extension's features.

### 2. Native MCP Path (no extension)

If you don't want the extension, you can register operator directly with Cursor:

1. From operator's TUI, navigate to the **Connections** panel and trigger the `WriteAndOpenMcpClientConfig` action with `client: "cursor"`. This writes a ready-to-paste snippet to `<tickets>/operator/mcp/cursor.json` and opens it in your editor.
2. Copy the snippet's `mcpServers.operator` block into your `~/.cursor/mcp.json`, merging with any existing entries.
3. Restart Cursor or toggle the server in **Cursor Settings → MCP**.

## Installation

### From OpenVSX (Cursor's default)

1. Open Cursor.
2. Open the Extensions sidebar (`Cmd+Shift+X` / `Ctrl+Shift+X`).
3. Search for **Operator Terminals**.
4. Click **Install**.

### Manual Installation

1. Download the `.vsix` from [GitHub releases](https://github.com/untra/operator/releases){:target="_blank"}.
2. In Cursor, open Extensions, click the `…` menu, and pick **Install from VSIX…**.

## Configuration

The extension shares the same configuration as the VS Code session manager. Settings are scoped under `operator.*` in Cursor's `settings.json`.

| Setting | Default | Description |
|---------|---------|-------------|
| `operator.webhookPort` | `7009` | Port for webhook server |
| `operator.autoStart` | `true` | Start server on Cursor launch |
| `operator.terminalPrefix` | `op-` | Prefix for managed terminal names |
| `operator.ticketsDir` | `.tickets` | Path to tickets directory |
| `operator.apiUrl` | `http://localhost:7008` | Operator REST API URL |

## MCP Integration

Cursor's `~/.cursor/mcp.json` uses the `mcpServers` shape with `command`, `args`, and `cwd` — stdio only. SSE-style URL entries are not honored by Cursor's MCP UI.

### Requirements

- Operator must be running with `[mcp].stdio_advertised = true` in its config (this is the default). Restart the operator API after toggling.
- The operator binary path written into `~/.cursor/mcp.json` is taken from the running operator process — if you reinstall or move the binary, re-run `Operator: Connect MCP Server` to refresh the path.

### Merge Semantics

The extension's Cursor-write path is additive:

- Any existing top-level keys in `~/.cursor/mcp.json` are preserved.
- Any existing `mcpServers.*` entries (other servers you've registered) are preserved.
- Only `mcpServers.operator` is set or overwritten on each run.

If `~/.cursor/mcp.json` exists but contains malformed JSON, the extension shows an error and refuses to overwrite the file — fix or remove it manually and re-run the command.

## Commands

Same set as the VS Code session manager — access via the command palette (`Cmd+Shift+P`):

| Command | Description |
|---------|-------------|
| `Operator: Start Webhook Server` | Start the webhook server |
| `Operator: Stop Webhook Server` | Stop the webhook server |
| `Operator: Connect MCP Server` | Register operator with Cursor's MCP (writes `~/.cursor/mcp.json`) |
| `Operator: Launch Ticket` | Launch a ticket in a new terminal |
| `Operator: Show Server Status` | Display server status |

## Requirements

- Cursor with stable MCP support (Cursor 0.42 or later; verify `Settings → MCP` is present).
- Operator built from `main` or a release that includes the MCP stdio plan (`operator mcp` subcommand and `[mcp].stdio_advertised` config flag).

## Troubleshooting

### `Operator MCP stdio entrypoint is not advertised`

The descriptor your operator API returned did not include a `stdio` field, so Cursor cannot register the server. Set `[mcp].stdio_advertised = true` in your operator config and restart the API, then re-run `Operator: Connect MCP Server`.

### Editing `~/.cursor/mcp.json` manually

Open the file in any text editor. To remove the operator entry, delete the `mcpServers.operator` key (preserve the rest of the file's structure). Save and restart Cursor or toggle the server in **Cursor Settings → MCP**.

### Switching between VS Code and Cursor on the same workspace

If you previously ran `Operator: Connect MCP Server` in stock VS Code on the same workspace, the workspace `mcp.servers.operator` entry still exists in that workspace's `.vscode/settings.json`. Running the command again in Cursor adds a user-scope entry in `~/.cursor/mcp.json`. **Both work in their respective hosts**; the extension intentionally does not delete the workspace entry from inside Cursor (doing so would surprise you when you reopen the workspace in VS Code).

If you want a clean slate in only one of the two editors, delete the entry from the file Cursor or VS Code doesn't see (`.vscode/settings.json` for VS Code, `~/.cursor/mcp.json` for Cursor).

### Cursor doesn't see the operator server after running the command

1. Check that `~/.cursor/mcp.json` exists and contains `mcpServers.operator` with `command`, `args`, and `cwd`.
2. Restart Cursor or open **Cursor Settings → MCP** and toggle the operator server off and on.
3. Confirm the `command` path in the JSON is executable (`ls -l <path>` and run it manually with `<path> mcp` — it should hang waiting for JSON-RPC on stdin, which is correct).

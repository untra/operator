---
title: "Zed"
description: "Zed editor integration for Operator via MCP context server, ACP agent, and slash commands."
layout: doc
---

# Zed

<span class="badge alpha">Alpha</span>

<div class="alpha-banner">
This integration is in alpha and may have limited functionality or incomplete support.
</div>

The [Zed](https://zed.dev) extension for Operator provides three integration layers: an MCP context server for tools and resources, an ACP agent server for delegated prompts, and slash commands for quick operations.

## Prerequisites

- [Operator](https://operator.untra.io) installed and on PATH
- Zed editor

## Installation

1. Open Zed
2. Open the Extensions panel (**Zed > Extensions** or `Cmd+Shift+X`)
3. Search for **Operator**
4. Click **Install**

## Setup

### MCP Context Server (automatic)

After installing the extension, Zed automatically registers `operator mcp` as a context server. All Operator tools appear in the Agent Panel:

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

## How It Works

Operator integrates with Zed through three communication channels:

- **MCP Context Server** — Runs `operator mcp` via stdio. Tools and ticket resources appear natively in the Agent Panel without additional configuration.
- **ACP Agent Server** — Runs `operator acp` via stdio. Prompts sent to the Operator agent flow through a delegator to Claude Code, with streaming output back to Zed.
- **Slash Commands** — Communicate with the Operator REST API for quick status checks and operations directly in the AI assistant.

## Configuration

The Operator binary must be on your PATH. The extension also checks common install locations (`/usr/local/bin`, `/opt/homebrew/bin`). The REST API URL for slash commands defaults to `http://localhost:7008`.

## Troubleshooting

### MCP tools not appearing

1. Verify Operator is on PATH: `which operator`
2. Test MCP server: `operator mcp` (should wait for JSON-RPC input)
3. Check Zed's extension logs: **View > Output > Extensions**

### Slash commands failing

1. Check that Operator API is running: `operator api`
2. Verify connectivity: `curl http://localhost:7008/api/v1/health`

### Extension not appearing

1. Open the Extensions panel and verify Operator is listed as installed
2. Try **Zed > Extensions > Reload** or restart Zed

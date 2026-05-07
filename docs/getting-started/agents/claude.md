---
title: "Claude"
description: "Configure Claude Code as your AI coding agent."
layout: doc
---

# Claude Code

[Claude Code](https://code.claude.com) is Anthropic's AI coding assistant agent, available as Claude Code for command-line development workflows.

## Installation

Install Claude Code via npm:

```bash
npm install -g @anthropic-ai/claude-code
```

Or download directly from [Anthropic](https://claude.ai/code).

### Plans and Pricing

View the [Claude pricing page](https://www.claude.com/pricing)

## Configuration

See the full [Claude agent configuration reference](/configuration/#agents-claude).

Add Claude to your Operator configuration:

```toml
# ~/.config/operator/config.toml

[agents.claude]
enabled = true
path = "claude"  # or full path to binary
```

## Authentication

Claude Code requires an API key or Claude Pro subscription. Set up authentication:

```bash
claude auth login
```

## Multi-agent relay

Agents launched by Operator automatically participate in the relay hub when the hub is running. Operator:

1. Injects `RELAY_HUB_SOCKET` and `RELAY_AGENT_NAME` (the ticket ID, e.g. `FEAT-042`) into the session environment.
2. Writes a per-session `relay-mcp.json` config and passes `--mcp-config <path>` to Claude Code, so the `relay-channel` MCP server starts automatically alongside the agent.

This means Claude agents can use `relay_peers`, `relay_ask`, `relay_reply`, `relay_broadcast`, and `relay_rename` tools out of the box — no manual MCP server configuration needed.

See [Relay](/docs/relay/) for the full architecture.

## Troubleshooting

### Claude not found

Ensure Claude is in your PATH:

```bash
which claude
```

### Authentication errors

Re-authenticate with:

```bash
claude auth logout
claude auth login
```

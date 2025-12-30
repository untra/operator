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

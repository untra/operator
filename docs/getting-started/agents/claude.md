---
title: "Claude"
description: "Use Claude Code as an Operator LLM tool."
layout: doc
---

[Claude Code](https://code.claude.com) is Anthropic's agentic CLI. In Operator it is an **LLM tool** (binary `claude`). Anthropic the API is a separate [model provider](/getting-started/model-servers/anthropic/).

## Status

Generally available. Catalog slug `claude`.

## Install

```bash
npm install -g @anthropic-ai/claude-code
```

Operator detects it with `which claude` and `claude --version` (minimum 2.1.0). `health_ok` means the binary is on **this host's** PATH.

## Authenticate

```bash
claude auth login
```

Headless and remote launches need `ANTHROPIC_API_KEY` in the environment Operator can see. Browser login on a laptop does not travel to an SSH target.

## Launch

Operator does not use `[agents.claude]` config. Pair the tool in a delegator:

```toml
[[delegators]]
name = "claude-opus"
llm_tool = "claude"
model = "opus"
# model_server omitted → implicit anthropic-api
```

Or pick Claude + a live model id from the Model Providers view after a successful `/models` probe.

Project discovery looks for `CLAUDE.md` at the repo root.

## Relay

When a delegator's `launch_config.operator_relay` is true, Operator injects the relay MCP config for Claude Code. See [Relay](/relay/).

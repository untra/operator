---
title: "Grok"
description: "Use Grok (xAI) as an Operator LLM tool."
layout: doc
---

[Grok](https://x.ai/cli) is xAI's agentic CLI. In Operator it is an **LLM tool** (binary `grok`). xAI the API is a separate [model provider](/getting-started/model-servers/xai/).

## Status

Alpha. Catalog slug `grok` (same as the binary).

## Install

```bash
curl -fsSL https://x.ai/cli/install.sh | bash
```

Operator detects it with `which grok` and `grok --version`. `health_ok` means the binary is on **this host's** PATH.

## Authenticate

Locally:

```bash
grok login
```

Headless and remote launches need the API key in the environment Operator can see:

```bash
export XAI_API_KEY="xai-..."
```

Browser login on a laptop does not travel to an SSH target. Operator injects `XAI_API_KEY` into the remote session when that variable is set on the control plane.

## Launch

Pair the tool in a delegator. `model_server` omitted resolves to implicit `xai-api`:

```toml
[[delegators]]
name = "grok-default"
llm_tool = "grok"
model = "grok-4"
```

For a remote tmux pane, set the delegator's `target` (or `host`) to an SSH target that has `grok` on PATH. The payload is interactive (`grok --session-id … "$(cat prompt)"`) so you can attach.

Project discovery: `AGENTS.md` (shared default) or `GROK.md`.

## Gaps (Alpha)

- Session resume uses `--session-id` on each launch; `--resume` is not wired yet.
- Relay MCP and permission translators are not implemented for Grok.
- Custom models in `~/.grok/config.toml` are the CLI's concern, not Operator's.

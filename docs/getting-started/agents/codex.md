---
title: "Codex"
description: "Use OpenAI Codex as an Operator LLM tool."
layout: doc
---

[Codex](https://developers.openai.com/codex/) is OpenAI's agentic CLI. In Operator it is an **LLM tool** (binary `codex`). OpenAI the API is a separate [model provider](/getting-started/model-servers/openai/). Codex speaks the OpenAI protocol, so it can also target [Ollama](/getting-started/model-servers/ollama/) or [OpenRouter](/getting-started/model-servers/openrouter/) when a delegator names that `model_server`.

## Status

Beta. Catalog slug `codex`.

## Install

```bash
npm i -g @openai/codex
```

Operator detects it with `which codex` and `codex --version`. `health_ok` means the binary is on **this host's** PATH.

## Authenticate

```bash
export OPENAI_API_KEY="sk-..."
```

Remote launches need that key in the environment Operator can inject. A local Codex login does not travel to an SSH target.

## Launch

Operator does not use `[agents.codex]` config. Pair the tool in a delegator:

```toml
[[delegators]]
name = "codex-gpt"
llm_tool = "codex"
model = "gpt-4o"
# model_server omitted → implicit openai-api
```

To run Codex against a local Ollama host:

```toml
[[model_servers]]
name = "ollama-local"
kind = "ollama"
base_url = "http://localhost:11434"

[[delegators]]
name = "codex-local-qwen"
llm_tool = "codex"
model = "qwen2.5-coder"
model_server = "ollama-local"
```

Project discovery looks for `CODEX.md` at the repo root.

## Relay

Operator injects relay env vars into Codex sessions. Full MCP tool support for Codex relay is still limited. See [Relay](/relay/).

---
title: "Ollama"
description: "Run local open models with Ollama as an Operator model provider."
layout: doc
---

# Ollama

[**Ollama**](https://ollama.com/) runs open models (Llama, Qwen, Mistral, …)
locally and serves them over an OpenAI-compatible API. Declare it as a
[model server](./) to drive agents against models on your own machine — no cloud
key required.

## Prerequisites

- Ollama installed and running: [ollama.com/download](https://ollama.com/download),
  then `ollama serve` (default `http://localhost:11434`)
- At least one model pulled, e.g. `ollama pull qwen2.5-coder`
- An OpenAI-protocol LLM tool — **codex** works directly; claude/gemini need a
  bridge (see [Protocol compatibility](./#protocol-compatibility))

## Configuration

Declare the server in `operator.toml` (no API key needed for a local server):

```toml
[[model_servers]]
name = "ollama-local"
kind = "ollama"
base_url = "http://localhost:11434"
display_name = "Ollama (local)"
```

Then reference it from a delegator:

```toml
[[delegators]]
name = "codex-local-qwen"
llm_tool = "codex"
model = "qwen2.5-coder"
model_server = "ollama-local"
```

## Listing models

Ollama enumerates its pulled models at `/api/tags`. Operator probes it for the
live list (which doubles as a reachability check):

```bash
# REST
GET /api/v1/model-servers/ollama-local/models   # { reachable, models[], error? }
```

In the VS Code status tree, expand the `ollama-local` server to browse the models
you've pulled.

## How env injection works

Ollama speaks the OpenAI protocol, so when a delegator resolves to it Operator
exports `OPENAI_BASE_URL=http://localhost:11434`. A local server needs no key; if
you've put one behind a proxy, set `api_key_env` and it is injected **by
reference**. See the [Model Providers overview](./#how-env-injection-works) for
the full mechanism.

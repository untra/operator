---
title: "OpenRouter"
description: "Reach hundreds of models through OpenRouter, one OpenAI-compatible gateway."
layout: doc
---

# OpenRouter

[**OpenRouter**](https://openrouter.ai/) is a hosted gateway that fronts hundreds
of models (Anthropic, OpenAI, Google, Meta, Mistral, and more) behind a single
OpenAI-compatible endpoint and one API key. Declare it once as a
[model server](./) and any delegator can target the whole catalog.

## Prerequisites

- An OpenRouter account and an API key from
  [openrouter.ai/keys](https://openrouter.ai/keys)
- An OpenAI-protocol LLM tool — **codex** works directly; claude/gemini need a
  bridge (see [Protocol compatibility](./#protocol-compatibility))

## Configuration

Export your key (kept out of config — Operator references it by name):

```bash
export OPENROUTER_API_KEY="sk-or-..."
```

Declare the server in `operator.toml`:

```toml
[[model_servers]]
name = "openrouter"
kind = "openrouter"
base_url = "https://openrouter.ai/api/v1"
api_key_env = "OPENROUTER_API_KEY"
display_name = "OpenRouter"
```

Then reference it from a delegator. The `model` is an OpenRouter model id (the
`vendor/model` form from its catalog):

```toml
[[delegators]]
name = "codex-openrouter-sonnet"
llm_tool = "codex"
model = "anthropic/claude-3.5-sonnet"
model_server = "openrouter"
```

## Listing models

OpenRouter publishes its catalog at `/models`. Operator probes it for the live
model list (which doubles as a reachability check):

```bash
# REST
GET /api/v1/model-servers/openrouter/models   # { reachable, models[], error? }
```

In the VS Code status tree, expand the `openrouter` server to browse its models;
each model's human label comes from OpenRouter's `name` field.

## How env injection works

OpenRouter speaks the OpenAI protocol, so when a delegator resolves to it
Operator exports:

| var | value |
|-----|-------|
| `OPENAI_BASE_URL` | `https://openrouter.ai/api/v1` |
| `OPENAI_API_KEY`  | `${OPENROUTER_API_KEY}` (by reference) |

The key is injected **by reference**, never by value — the secret is never
written into the on-disk command script. See the
[Model Providers overview](./#how-env-injection-works) for the full mechanism.

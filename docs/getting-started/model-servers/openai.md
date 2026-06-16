---
title: "OpenAI"
description: "Connect OpenAI as a first-party model provider and list its models live."
layout: doc
---

# OpenAI

[**OpenAI**](https://openai.com/) is a first-party model provider — it produces
the GPT family and serves them from its own API. It is the zero-config default
for the `codex` llm tool, and a first-class [model provider](./): once connected,
operator lists its available models live for delegators to pick from.

> **Model provider ≠ llm tool.** OpenAI (the provider) serves the models;
> [Codex](../agents/codex/) (the llm tool) is the CLI. A delegator pairs a tool
> with a provider's model.

## Connect

Operator references your key by env-var name — it never stores the secret:

```bash
export OPENAI_API_KEY="sk-..."
```

A provider is **connected** when its `/models` probe succeeds. OpenAI then shows
● connected in the Model Providers view with its live model list.

## Listing models

Operator probes `https://api.openai.com/v1/models` and lists whatever the API
returns (agnostic to the specific model set):

```bash
GET /api/v1/model-servers/kinds/openai-api/models   # { reachable, models[], error? }
```

## Use from a delegator

```toml
[[delegators]]
name = "codex-gpt"
llm_tool = "codex"
model = "gpt-4o"
# model_server omitted → resolves to the implicit openai-api default
```

## Custom endpoint / proxy

Declare an `openai-api` server with an explicit `base_url` to point Codex at a
proxy; that base URL is then injected at spawn (`OPENAI_BASE_URL`). The probe
default (`https://api.openai.com`) is **probe-only** — it never changes the
launch path on its own.

---
title: "Google"
description: "Connect Google (Gemini) as a first-party model provider and list its models live."
layout: doc
---

# Google

[**Google**](https://ai.google.dev/) is a first-party model provider — it
produces the Gemini family and serves them from its own API. It is the
zero-config default for the `gemini` llm tool, and a first-class
[model provider](./): once connected, operator lists its available models live
for delegators.

> **Model provider ≠ llm tool.** Google (the provider) serves the models;
> [Gemini CLI](../agents/gemini-cli/) (the llm tool) is the CLI. A delegator
> pairs a tool with a provider's model.

## Connect

Operator references your key by env-var name — it never stores the secret:

```bash
export GEMINI_API_KEY="..."
```

A provider is **connected** when its `/models` probe succeeds. Google then shows
● connected in the Model Providers view with its live model list.

## Listing models

Operator probes `https://generativelanguage.googleapis.com/v1beta/models`
([model list reference](https://ai.google.dev/api/models)) and lists whatever the
API returns, stripping the `models/` prefix so the id matches `--model`:

```bash
GET /api/v1/model-servers/kinds/google-api/models   # { reachable, models[], error? }
```

## Use from a delegator

```toml
[[delegators]]
name = "gemini-pro"
llm_tool = "gemini"
model = "gemini-1.5-pro"
# model_server omitted → resolves to the implicit google-api default
```

## Custom endpoint / proxy

Declare a `google-api` server with an explicit `base_url` to point Gemini at a
proxy; that base URL is injected at spawn (`GOOGLE_GEMINI_BASE_URL`). The probe
default is **probe-only** and never changes the launch path on its own.

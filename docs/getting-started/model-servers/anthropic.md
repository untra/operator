---
title: "Anthropic"
description: "Connect Anthropic as a first-party model provider and list its models live."
layout: doc
---

# Anthropic

[**Anthropic**](https://www.anthropic.com/) is a first-party model provider — it
produces the Claude family of models and serves them from its own API. It is the
zero-config default for the `claude` llm tool, and a first-class
[model provider](./) in its own right: once connected, operator lists its
available models live so delegators can pick one.

> **Model provider ≠ llm tool.** Anthropic (the provider) serves the models;
> [Claude Code](../agents/claude/) (the llm tool) is the CLI that drives a coding
> session. A delegator pairs a tool with a provider's model.

## Connect

Operator references your key by env-var name — it never stores the secret. Set
the standard Anthropic key and operator can probe the provider:

```bash
export ANTHROPIC_API_KEY="sk-ant-..."
```

A provider is **connected** when its `/models` probe succeeds (key present +
endpoint reachable). In the Model Providers view (web `/#/model-providers` or the
VS Code section) Anthropic then shows ● connected with its live model list.

## Listing models

Operator probes `https://api.anthropic.com/v1/models` and stays agnostic to which
models exist — it lists whatever the API returns rather than hardcoding names:

```bash
GET /api/v1/model-servers/kinds/anthropic-api/models   # { reachable, models[], error? }
```

## Use from a delegator

Pick a connected provider + one of its live models in the Create Delegator form,
or declare it directly. `model` is a live id from the list above:

```toml
[[delegators]]
name = "claude-opus"
llm_tool = "claude"
model = "claude-opus-4-20250514"
# model_server omitted → resolves to the implicit anthropic-api default
```

## Custom endpoint / proxy

To route Claude through a proxy or bridge (e.g. a local model behind
`claude-code-router`), declare an `anthropic-api` server with an explicit
`base_url`; that base URL is then also injected at spawn (`ANTHROPIC_BASE_URL`).
The probe default (`https://api.anthropic.com`) is **probe-only** and never
alters the agent's launch path on its own.

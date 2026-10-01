---
title: "xAI"
description: "Connect xAI as a first-party model provider and list its models live."
layout: doc
---

[**xAI**](https://x.ai/) is a first-party model provider - it produces the Grok
family and serves them from its own API. It is the zero-config default for the
`grok` llm tool, and a first-class [model provider](/getting-started/model-servers/): once connected,
operator lists its available models live for delegators to pick from.

> **Model provider ≠ llm tool.** xAI (the provider) serves the models;
> [Grok](/getting-started/agents/grok/) (the llm tool) is the CLI. A delegator pairs a tool
> with a provider's model.

## Connect

Operator references your key by env-var name - it never stores the secret:

```bash
export XAI_API_KEY="xai-..."
```

A provider is **connected** when its `/models` probe succeeds. xAI then shows
● connected in the Model Providers view with its live model list.

## Listing models

Operator probes `https://api.x.ai/v1/models` (OpenAI-shaped) and lists whatever
the API returns:

```bash
GET /api/v1/model-servers/kinds/xai-api/models   # { reachable, models[], error? }
```

## Use from a delegator

```toml
[[delegators]]
name = "grok-default"
llm_tool = "grok"
model = "grok-4"
# model_server omitted → implicit xai-api
```

Codex also speaks the OpenAI protocol, so the catalog marks Codex × xAI as
**Native**. Operator's spawn env for `xai-api` currently exports `XAI_API_KEY`
(what the grok CLI reads). Pointing Codex at xAI in this Alpha is a declared
`openai-compat` server at `https://api.x.ai/v1`, or a follow-up dual-env mapping.

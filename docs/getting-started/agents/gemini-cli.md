---
title: "Gemini CLI"
description: "Use Google Gemini CLI as an Operator LLM tool."
layout: doc
---

[Gemini CLI](https://geminicli.com/) is Google's agentic CLI. In Operator it is an **LLM tool** (binary `gemini`). The catalog slug is `gemini-cli` so it is not confused with the Gemini model family. Google the API is a separate [model provider](/getting-started/model-servers/google/).

## Status

Alpha. Catalog slug `gemini-cli`, binary `gemini`.

## Install

Install the Gemini CLI (not the `google-generativeai` Python SDK):

```bash
npm install -g @google/gemini-cli
```

Operator detects it with `which gemini` and `gemini --version`. `health_ok` means the binary is on **this host's** PATH.

## Authenticate

```bash
export GEMINI_API_KEY="..."
```

Remote launches need that key in the environment Operator can inject.

## Launch

Operator does not use `[agents.gemini]` config. Pair the tool in a delegator:

```toml
[[delegators]]
name = "gemini-pro"
llm_tool = "gemini"
model = "pro"
# model_server omitted → implicit google-api
```

`llm_tool` is the binary name (`gemini`), not the catalog slug (`gemini-cli`).

Project discovery looks for `GEMINI.md` at the repo root.

Gemini speaks Google's protocol. Pointing it at Ollama or OpenRouter requires a protocol bridge, the same as Claude.

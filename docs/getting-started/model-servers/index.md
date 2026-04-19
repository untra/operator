---
layout: default
title: Model Servers
parent: Getting Started
nav_order: 5
has_children: false
---

# Model Servers

A **model server** is a named host that serves models via an inference API. It's orthogonal to the LLM tool that runs your coding agent:

- **LLM tools** (claude, codex, gemini) are the agentic CLIs that drive the coding session — they use tools, edit files, resume sessions.
- **Model servers** are where the model weights live — Anthropic's API, OpenAI's API, Google's API, or a local/alt host like ollama, lmstudio, or vllm.

A delegator pairs an LLM tool with a model (and, optionally, a model server).

## The three-layer hierarchy

```
┌─ llm_tools ─────────┐   ┌─ model_servers ──────┐
│ claude  (detected)  │   │ anthropic-api (impl.)│
│ codex   (detected)  │   │ openai-api    (impl.)│
│ gemini  (detected)  │   │ google-api    (impl.)│
│                     │   │ ollama-local  (user) │
└─────────────────────┘   └──────────────────────┘
            ▲                        ▲
            │                        │
            └───── delegators ───────┘
   name, llm_tool, model, model_server (optional)
```

## Implicit builtins

You don't need to declare a model server for the vendor-default path. Every detected LLM tool has an implicit builtin:

| llm_tool | implicit model_server |
|----------|------------------------|
| `claude` | `anthropic-api`        |
| `codex`  | `openai-api`           |
| `gemini` | `google-api`           |

Delegators that omit `model_server` resolve to these builtins automatically. Existing configs keep working unchanged.

## Kinds

| `kind`          | Use for                                                                  |
|-----------------|--------------------------------------------------------------------------|
| `anthropic-api` | Anthropic Console / a compatible proxy (bridge for local models)         |
| `openai-api`    | OpenAI / a compatible proxy                                              |
| `google-api`    | Google Gemini API                                                        |
| `ollama`        | Local ollama server (`ollama serve`, default `http://localhost:11434`)   |
| `openai-compat` | Any OpenAI-API-compatible server (vllm, lmstudio, together.ai, groq, …)  |
| `lmstudio`      | LM Studio's local server                                                 |

## Declaring a model server

Edit `operator.toml` (or create a delegator via the REST API / VS Code status tree):

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

## Ad-hoc CLI usage

```bash
# Named delegator (recommended for repeatable runs)
operator launch --delegator codex-local-qwen

# Ad-hoc overrides (for one-off experiments)
operator launch \
  --llm-tool codex \
  --model qwen2.5-coder \
  --model-server ollama-local
```

`--delegator` and the ad-hoc trio (`--llm-tool`, `--model`, `--model-server`) are mutually exclusive.

## Protocol compatibility

| llm_tool | ollama-compatible? | Notes                                                                                  |
|----------|--------------------|----------------------------------------------------------------------------------------|
| `codex`  | Yes, directly      | Codex speaks OpenAI API; ollama exposes `/v1` out of the box.                          |
| `claude` | Only via bridge    | Claude CLI speaks Anthropic protocol. Run `claude-code-router` (or similar) at a port and point `base_url` at that bridge with `kind = "anthropic-api"`. |
| `gemini` | Only via bridge    | Same story as claude; use `litellm-proxy` or similar.                                  |

## REST API

```
GET    /api/v1/model-servers         # list (declared + implicit builtins)
GET    /api/v1/model-servers/{name}  # fetch by name
POST   /api/v1/model-servers         # create
DELETE /api/v1/model-servers/{name}  # delete (implicit builtins are protected)
```

## What ships in this release

This release lays down the infrastructure:

- Data model and config schema
- REST CRUD endpoints
- TUI and VS Code status tree sections
- `operator launch --model-server <name>` flag (validated, resolved through the normal delegator path)

**What's explicitly deferred:**

- Automatic ollama detection during `operator setup`
- Environment-variable injection on spawn (`OPENAI_BASE_URL=…` etc.)
- Full walkthroughs for wiring up claude/gemini via a bridge
- Bundled bridge binaries

Those ship in the next release. In the meantime: declare your model server, attach it to a delegator, and set the appropriate `*_BASE_URL` env var in your shell before invoking operator — the spawned agent inherits it.

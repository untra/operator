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

## Listing models

Each kind knows how to enumerate the models its endpoint serves (ollama `/api/tags`,
OpenAI-protocol `/v1/models`, Anthropic `/v1/models`, Gemini `/v1beta/models`). The
same probe doubles as a reachability check — there is no separate "test connection".

- **REST**: `GET /api/v1/model-servers/{name}/models` returns `{ reachable, models[], error? }`.
- **VS Code**: expand a server in the status tree to see its live model list (or an
  "unreachable" line with the error).

## REST API

```
GET    /api/v1/model-servers          # list (declared + implicit builtins)
GET    /api/v1/model-servers/kinds    # the supported-kind catalog (single source of truth)
GET    /api/v1/model-servers/{name}   # fetch by name
GET    /api/v1/model-servers/{name}/models  # live model list + reachability
POST   /api/v1/model-servers          # create
PUT    /api/v1/model-servers/{name}   # update (implicit builtins are protected)
DELETE /api/v1/model-servers/{name}   # delete (implicit builtins are protected)
```

## How env injection works

When a delegator (or the ad-hoc `--model-server` flag) resolves to a model server,
operator exports the server's connection env into the spawned agent's command script,
keyed by the server's protocol:

| kind            | base URL var          | API key var (mapped from `api_key_env`) |
|-----------------|-----------------------|------------------------------------------|
| `anthropic-api` | `ANTHROPIC_BASE_URL`  | `ANTHROPIC_API_KEY`                      |
| `openai-api` / `openai-compat` / `ollama` / `lmstudio` | `OPENAI_BASE_URL` | `OPENAI_API_KEY` |
| `google-api`    | `GOOGLE_GEMINI_BASE_URL` | `GEMINI_API_KEY`                      |

The API key is injected **by reference**, not by value: if `api_key_env = "MY_KEY"`,
the script exports `OPENAI_API_KEY="${MY_KEY}"`, which the shell resolves from the
inherited environment at run time. The secret is never written into the on-disk
command script. Any `extra_env` entries are exported verbatim and take precedence.
Implicit builtins with no `base_url` inject nothing — the vendor-default path is unchanged.

**Still deferred:**

- Automatic ollama detection during `operator setup`
- Full walkthroughs / bundled binaries for wiring up claude/gemini via a bridge
- A primary "select server" UI action (selecting a server is currently inert; use a
  delegator's `model_server` to bind one)

---
title: "Delegators"
description: "Named LLM tool + model pairings for autonomous ticket launching."
layout: doc
---

A **delegator** is a named pairing of an LLM tool (e.g. `claude`) and a model alias (e.g. `opus`) that Operator uses to launch agents for tickets. Delegators give you control over which tool and model handles which tickets, and let you configure launch behavior per pairing. LLM tasks can be launched on behalf on a named delegator, which allows you to refine and version their prompts.

## Quick start

Add a `[[delegators]]` entry to your `operator.toml`:

```toml
[[delegators]]
name        = "claude-sonnet-auto"
llm_tool    = "claude"
model       = "sonnet"

[delegators.launch_config]
yolo        = true
```

Then run `cargo run -- launch` (or use the VS Code sidebar "Add Delegator" button) to create one interactively.

## How delegators relate to LLM Tools and Model Servers

Three concepts work together:

| Concept | What it picks | Example |
|---------|--------------|---------|
| **LLM Tool** | Which CLI binary to run | `claude`, `codex`, `gemini` |
| **Delegator** | Which tool + model to use | `claude` + `opus` |
| **Model Server** | Which inference endpoint to call | `ollama-local`, `anthropic-api` |

A delegator bridges a tool (the binary) to a model server (the backend). If `model_server` is omitted, the tool's implicit vendor default is used (`claude` → Anthropic API, `codex` → OpenAI API, `gemini` → Google API).

## Fields

| Field | Required | Description |
|-------|----------|-------------|
| `name` | Yes | Unique identifier (e.g. `"claude-opus-auto"`) |
| `llm_tool` | Yes | Tool name matching a detected binary (`"claude"`, `"codex"`, `"gemini"`) |
| `model` | Yes | Model alias passed to the tool (e.g. `"opus"`, `"sonnet"`, `"gpt-4o"`) |
| `display_name` | No | Human-readable label shown in the UI |
| `model_server` | No | Name of a declared `[[model_servers]]` entry; omit to use the tool's vendor default |
| `model_properties` | No | Arbitrary key-value pairs forwarded to the model (e.g. `reasoning_effort = "high"`) |
| `launch_config` | No | Per-delegator launch overrides (see below) |

## Launch configuration

`[delegators.launch_config]` lets you override launch behavior for a specific delegator:

```toml
[[delegators]]
name     = "claude-opus-yolo"
llm_tool = "claude"
model    = "opus"

[delegators.launch_config]
yolo             = true          # auto-accept all prompts
permission_mode  = "bypassPermissions"
use_worktrees    = true          # override global git.use_worktrees
prompt_suffix    = "\n\nBe concise."
```

| Option | Default | Description |
|--------|---------|-------------|
| `yolo` | `false` | Run in auto-accept mode (skips all confirmation prompts) |
| `permission_mode` | inherit | Permission mode override |
| `flags` | `[]` | Extra CLI flags appended to the launch command |
| `use_worktrees` | inherit | Override global `git.use_worktrees` for this delegator |
| `create_branch` | inherit | Whether to create a git branch per ticket |
| `docker` | inherit | Run agent in a Docker container |
| `prompt_prefix` | none | Text prepended before the generated ticket prompt |
| `prompt_suffix` | none | Text appended after the generated ticket prompt |

`inherit` means the global config value is used.

## Using a custom model server

To route a delegator through a local Ollama instance:

```toml
[[model_servers]]
name     = "ollama-local"
kind     = "ollama"
base_url = "http://localhost:11434"

[[delegators]]
name         = "codex-qwen"
llm_tool     = "codex"
model        = "qwen2.5-coder"
model_server = "ollama-local"
```

## Multiple delegators

You can declare as many delegators as you like. When launching a ticket, Operator selects a delegator based on the ticket's `delegator` frontmatter field or the configured default:

```toml
[[delegators]]
name     = "claude-sonnet-auto"
llm_tool = "claude"
model    = "sonnet"

[[delegators]]
name     = "claude-opus-research"
llm_tool = "claude"
model    = "opus"

[delegators.launch_config]
prompt_suffix = "\n\nThink carefully before acting."
```

## REST API

The running Operator API exposes full CRUD for delegators:

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/v1/delegators` | List all delegators |
| `POST` | `/api/v1/delegators` | Create a delegator |
| `POST` | `/api/v1/delegators/from-tool` | Create from a detected tool (auto-generates name) |
| `GET` | `/api/v1/delegators/{name}` | Get one delegator |
| `PUT` | `/api/v1/delegators/{name}` | Update a delegator |
| `DELETE` | `/api/v1/delegators/{name}` | Delete a delegator |

See the [OpenAPI reference](/docs/schemas/openapi.json) for request/response shapes.

## See also

- [Configuration reference](/docs/configuration/) — full `operator.toml` schema
- [LLM Tools](/docs/llm-tools/) — which tools Operator can detect and launch
- [Schema reference](/docs/schemas/config/) — type definitions for `Delegator` and `DelegatorLaunchConfig`

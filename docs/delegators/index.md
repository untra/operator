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

### Relay MCP injection

Set `operator_relay = true` to enable the relay MCP server for Claude Code
launches from this delegator. Use `false` to disable it even when the global
default is `true`. Omit the field to use the global `relay.auto_inject_mcp`
default.

```toml
[delegators.coordination.launch_config]
operator_relay = true   # coordination-heavy delegators: enable relay

[delegators.task.launch_config]
operator_relay = false  # single-agent task delegators: disable relay
```

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

## Agent profiles & remote agents

A delegator can be serialized to a portable **agent profile** (`agent-profile.json`) — a
tool-agnostic interchange format with a shared core (`provider`, `model`, `system_prompt`,
`skills`, `mcp_servers`, `tools`) plus namespaced extension bags: `x_operator` (Operator's
launch config and model properties) and per-platform opaque bags (`x_agnt`, `x_openai`) that are
preserved verbatim. Profiles round-trip losslessly in both directions, so a profile authored on
another platform survives `import → export` byte-for-byte.

A delegator may also carry a **`remote_agent`** reference — a `{ platform, id }` pointer to a
remote, named agent that lives on another service:

```toml
[[delegators]]
name  = "agnt-researcher"
# the agent lives on AGNT; Operator references it but never runs it
[delegators.remote_agent]
platform = "agnt"          # or "openai"
id       = "Research Assistant"   # AGNT agent name, or an OpenAI asst_… id
```

Remote agents are **export-only**: Operator has no runtime client for those platforms, so a
delegator carrying a `remote_agent` cannot be launched locally — resolution returns a
`RemoteOnlyDelegator` error on every launch path. When the platform is `agnt`, the reference is
surfaced in the [`--format agnt` workflow export](/docs/) as an `agnt-agent` node; other platforms
ride opaquely in the profile.

> **Caveat:** a non-AGNT remote delegator (e.g. `platform = "openai"`) used as a step agent in an
> `agnt` export still emits an ordinary `operator-run-step` node. If AGNT runs that node it calls
> back into Operator, which then hits the `RemoteOnlyDelegator` guard and errors. Don't bind a
> non-AGNT remote delegator as the step agent of a workflow you intend to export to AGNT.

> **Design note — the interchange is tool-agnostic.** AGNT was the first remote platform; adding
> OpenAI Assistants as the second cost only a generic `remote_agent { platform, id }` reference and
> an opaque `x_openai` bag mirroring `x_agnt` — **no new mapping logic, no executor, no export
> node.** That's the evidence the schema core is not shaped around any one tool.

## REST API

The running Operator API exposes full CRUD for delegators, plus agent-profile interchange:

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/v1/delegators` | List all delegators |
| `POST` | `/api/v1/delegators` | Create a delegator |
| `POST` | `/api/v1/delegators/from-tool` | Create from a detected tool (auto-generates name) |
| `POST` | `/api/v1/delegators/import-profile` | Create a delegator from an `AgentProfile` |
| `GET` | `/api/v1/delegators/{name}` | Get one delegator |
| `GET` | `/api/v1/delegators/{name}/profile` | Export a delegator as an `AgentProfile` |
| `PUT` | `/api/v1/delegators/{name}` | Update a delegator |
| `DELETE` | `/api/v1/delegators/{name}` | Delete a delegator |

See the [OpenAPI reference](/docs/schemas/openapi.json) for request/response shapes.

## See also

- [Configuration reference](/docs/configuration/) — full `operator.toml` schema
- [LLM Tools](/docs/llm-tools/) — which tools Operator can detect and launch
- [Schema reference](/docs/schemas/config/) — type definitions for `Delegator` and `DelegatorLaunchConfig`

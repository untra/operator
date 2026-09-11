---
display_name: Operator
description: Run Operator agent orchestrator as a background service in your Coder workspace
icon: ../../../../.icons/terminal.svg
verified: false
tags: [ai, agents, orchestration, automation]
---

# Operator

Run [Operator](https://github.com/untra/operator) as a background REST API server inside your Coder workspace. Operator manages ticket queues, launches LLM-powered coding agents, and tracks their progress.

The module downloads the operator binary and the `opr8r` client from GitHub releases, generates configuration, starts the API server, and exposes the dashboard through the Coder workspace UI with automatic healthchecks.

> This module runs Operator **inside** a workspace. To run Operator elsewhere (a Kubernetes deployment, say) and have it *spawn* Coder workspaces as agent targets, you do not need this module at all — configure a `[[targets]]` entry with `kind = "coder"` instead. See the [Coder platform guide](https://operator.untra.io/getting-started/platforms/coder/).

## Usage

```tf
module "operator" {
  source   = "registry.coder.com/untra/operator/coder"
  agent_id = coder_agent.main.id
}
```

Pin `version` to a published module release for reproducible builds. `install_version` is separate — it selects the Operator release the module downloads, and defaults to the version this module shipped with.

### Custom configuration

```tf
module "operator" {
  source              = "registry.coder.com/untra/operator/coder"
  agent_id            = coder_agent.main.id
  port                = 7008
  max_parallel_agents = 4
  session_wrapper     = "tmux"
}
```

### Full TOML override

`config_toml` is written verbatim — quotes, `$` and backticks all survive, because the value is base64-encoded on the way into the startup script.

```tf
module "operator" {
  source   = "registry.coder.com/untra/operator/coder"
  agent_id = coder_agent.main.id
  config_toml = <<-EOT
    [rest_api]
    enabled = true
    port = 7008

    [agents]
    max_parallel = 4

    [sessions]
    wrapper = "tmux"
  EOT
}
```

Setting `config_toml` replaces the generated config entirely, including the `[[targets]]` block described below — declare the target yourself if you need both.

### Spawning child agent workspaces

Set `agent_template` and the generated config gains a `[[targets]]` entry, so tickets launched from this workspace create **sibling** workspaces from that template rather than running agents locally.

```tf
module "operator" {
  source         = "registry.coder.com/untra/operator/coder"
  agent_id       = coder_agent.main.id
  agent_template = "operator-agent"
  workdir        = "/home/coder/project"
}
```

This mode has prerequisites the basic mode does not — see below.

## Variables

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `agent_id` | `string` | *(required)* | The ID of a Coder agent |
| `port` | `number` | `7008` | Port for the Operator REST API server |
| `display_name` | `string` | `"Operator"` | Display name in the Coder dashboard |
| `slug` | `string` | `"operator"` | Application slug |
| `install_version` | `string` | *(module version)* | Operator GitHub release tag to install |
| `install_prefix` | `string` | `"/tmp/operator"` | Directory to install the binaries into |
| `log_path` | `string` | `"/tmp/operator.log"` | Path to write log output |
| `config_toml` | `string` | `""` | Raw TOML written verbatim instead of the generated config |
| `max_parallel_agents` | `number` | `2` | Maximum number of parallel agents |
| `session_wrapper` | `string` | `"tmux"` | Session wrapper (`tmux`, `cmux`, or `zellij`) |
| `share` | `string` | `"owner"` | Dashboard sharing level (`owner`, `authenticated`, or `public`) |
| `order` | `number` | `null` | Position of the app in the dashboard (lower = first) |
| `group` | `string` | `null` | Group that this app belongs to |
| `offline` | `bool` | `false` | Skip downloading; requires a pre-installed binary at `install_prefix` |
| `use_cached` | `bool` | `false` | Reuse a cached binary if present, otherwise download |

### Child-workspace spawning

All optional. `agent_template` is the switch — leave it empty and no `[[targets]]` entry is written and the rest are ignored. Keys left unset are omitted from the config so Operator's own defaults apply.

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `agent_template` | `string` | `""` | Coder template child agent workspaces are created from. Empty disables the coder target. |
| `coder_token_env` | `string` | `"CODER_SESSION_TOKEN"` | Name of the env var holding the Coder **user session token** used to spawn workspaces |
| `callback_url` | `string` | `""` | Control-plane-reachable `OPERATOR_API_URL` override, so detached multi-step survives SSH tunnel loss. Empty keeps the reverse-tunnel default. |
| `name_prefix` | `string` | `""` | Workspace name prefix for deterministic per-ticket naming |
| `workdir` | `string` | `""` | Project root inside spawned workspaces |
| `stop_on_complete` | `bool` | `null` | Stop a spawned workspace when its ticket completes (never deletes) |
| `create_timeout_secs` | `number` | `null` | Bound on workspace create plus agent-ready wait, in seconds |

## Prerequisites

The workspace image must include `tmux` (or your chosen `session_wrapper`) for Operator to spawn agent sessions. Most Coder workspace images include tmux by default.

Setting `agent_template` adds two more:

- **`ssh` in the workspace image** (`openssh-client`). Operator launches child-workspace agents over real `ssh`. The `coder` CLI is used too, but Operator fetches it from your deployment if it is not already on `PATH`.
- **A Coder user session token** in the variable named by `coder_token_env`. This is **not** the ambient `CODER_AGENT_TOKEN` (see below) and the module does not provision it — supply it yourself, e.g. through a template `env` block backed by a Coder parameter or secret.

> **Blast radius:** a Coder user session token can create, delete, and SSH into every workspace its user owns. Scope the account accordingly.

## Coder Workspace Context

Coder automatically injects environment variables into every workspace that operator can reference in ticket templates and agent prompts:

- `CODER_WORKSPACE_NAME` - workspace identifier
- `CODER_WORKSPACE_OWNER` - workspace owner username
- `CODER_URL` - deployment URL, which is what the coder target reads by default
- `CODER_AGENT_TOKEN` - **agent** authentication token, scoped to this one workspace

No operator configuration is needed to access these. Note that `CODER_AGENT_TOKEN` is not a substitute for the user session token child-workspace spawning needs — it cannot create workspaces. Operator also strips the session-token variable from every agent's environment before launching it, on every target kind.

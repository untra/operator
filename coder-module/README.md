---
display_name: Operator
description: Run Operator agent orchestrator as a background service in your Coder workspace
icon: ../../../../.icons/terminal.svg
verified: false
tags: [ai, agents, orchestration, automation]
---

# Operator

Run [Operator](https://github.com/untra/operator) as a background REST API server inside your Coder workspace. Operator manages ticket queues, launches LLM-powered coding agents, and tracks their progress.

The module downloads the operator binary from GitHub releases, generates configuration, starts the API server, and exposes the dashboard through the Coder workspace UI with automatic healthchecks.

## Usage

```tf
module "operator" {
  source   = "registry.coder.com/untra/operator/coder"
  version  = "1.0.0"
  agent_id = coder_agent.main.id
}
```

### Custom configuration

```tf
module "operator" {
  source              = "registry.coder.com/untra/operator/coder"
  version             = "1.0.0"
  agent_id            = coder_agent.main.id
  port                = 7008
  max_parallel_agents = 4
  session_wrapper     = "tmux"
}
```

### Full TOML override

```tf
module "operator" {
  source   = "registry.coder.com/untra/operator/coder"
  version  = "1.0.0"
  agent_id = coder_agent.main.id
  config_toml = <<-EOT
    [rest_api]
    enabled = true
    port = 7008

    [agents]
    max_parallel = 4
    health_check_interval = 30

    [sessions]
    wrapper = "tmux"

    [[delegators]]
    name = "default"
    tool = "claude-code"
    model = "sonnet"
  EOT
}
```

## Prerequisites

The workspace image must include `tmux` (or your chosen `session_wrapper`) for operator to spawn agent sessions. Most Coder workspace images include tmux by default.

## Coder Workspace Context

Coder automatically injects environment variables into every workspace that operator can reference in ticket templates and agent prompts:

- `CODER_WORKSPACE_NAME` — workspace identifier
- `CODER_WORKSPACE_OWNER` — workspace owner username
- `CODER_AGENT_TOKEN` — agent authentication token

No operator configuration is needed to access these — they are ambient in the workspace environment.

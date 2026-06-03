---
title: "Coder"
description: "Run Operator as a background service in Coder workspaces via Terraform module."
layout: doc
---

# Coder

<span class="badge supported">Supported</span>

Run [Operator](https://operator.untra.io) as a background REST API server inside your [Coder](https://coder.com) workspace. The module downloads the operator binary from GitHub releases, generates configuration, starts the API server, and exposes the dashboard through the Coder workspace UI with automatic healthchecks.

**Registry:** [`registry.coder.com/untra/operator/coder`](https://registry.coder.com/modules/operator)

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

## Variables

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `agent_id` | `string` | (required) | The ID of a Coder agent |
| `port` | `number` | `7008` | The port for the operator REST API server |
| `display_name` | `string` | `"Operator"` | Display name in the Coder dashboard |
| `slug` | `string` | `"operator"` | Application slug |
| `install_version` | `string` | `"{{ site.version }}"` | GitHub release tag to install |
| `install_prefix` | `string` | `"/tmp/operator"` | Directory to install the binary into |
| `log_path` | `string` | `"/tmp/operator.log"` | Path to write log output |
| `config_toml` | `string` | `""` | Raw TOML config (written verbatim instead of auto-generated config) |
| `max_parallel_agents` | `number` | `2` | Maximum number of parallel agents |
| `session_wrapper` | `string` | `"tmux"` | Session wrapper type (`tmux`, `cmux`, or `zellij`) |
| `share` | `string` | `"owner"` | Dashboard sharing level (`owner`, `authenticated`, or `public`) |
| `order` | `number` | `null` | Position of the app in the Coder dashboard (lower = first) |
| `group` | `string` | `null` | Group that this app belongs to |
| `offline` | `bool` | `false` | Skip downloading; requires pre-installed binary at `install_prefix` |
| `use_cached` | `bool` | `false` | Use cached binary if present, otherwise download |

## Prerequisites

The workspace image must include `tmux` (or your chosen `session_wrapper`) for Operator to spawn agent sessions. Most Coder workspace images include tmux by default.

## Coder Workspace Context

Coder automatically injects environment variables into every workspace that Operator can reference in ticket templates and agent prompts:

- `CODER_WORKSPACE_NAME` — workspace identifier
- `CODER_WORKSPACE_OWNER` — workspace owner username
- `CODER_AGENT_TOKEN` — agent authentication token

No Operator configuration is needed to access these — they are ambient in the workspace environment.

## How It Works

1. The module runs a startup script that detects the workspace architecture (`linux-x86_64` or `linux-arm64`)
2. Downloads the Operator binary from GitHub releases (or uses a cached/pre-installed binary)
3. Generates a TOML configuration file (or uses the provided `config_toml`)
4. Starts `operator api` as a background process
5. Registers the Operator dashboard as a Coder app with healthchecks polling `/api/v1/health` every 5 seconds

## Troubleshooting

### Binary download fails

1. Check that the `install_version` matches a valid [GitHub release tag](https://github.com/untra/operator/releases)
2. Verify the workspace has internet access (or use `offline = true` with a pre-installed binary)
3. Check logs at the configured `log_path` (default: `/tmp/operator.log`)

### Healthcheck timeout

1. Verify the port is not already in use: `ss -tlnp | grep 7008`
2. Check operator logs: `cat /tmp/operator.log`
3. Ensure the session wrapper (tmux by default) is installed in the workspace image

### Port conflicts

Change the `port` variable to an unused port. Remember to update any other services or extensions that connect to the Operator API.

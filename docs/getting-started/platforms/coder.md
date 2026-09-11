---
title: "Coder"
description: "Run Operator inside a Coder workspace, or point Operator at Coder to spawn per-ticket agent workspaces."
layout: doc
---

<span class="badge alpha">Alpha</span>

[Operator](https://operator.untra.io) and [Coder](https://coder.com) fit together in two directions. They are independent — pick the one that matches where Operator runs.

| | Operator runs | Agents run | Set up with |
|---|---|---|---|
| **[Inside a workspace](#operator-inside-a-coder-workspace)** | in a Coder workspace | in that same workspace | the Terraform module |
| **[Targeting Coder](#operator-targeting-coder)** | anywhere (Kubernetes, a server, your laptop) | in per-ticket Coder workspaces | a `[[targets]]` entry |

The two can be combined: Operator inside a workspace can also spawn *sibling* workspaces. See [child agent workspaces](#child-agent-workspaces).

## Operator inside a Coder workspace

A Terraform module runs Operator as a background REST API server in the workspace, and exposes its dashboard as a Coder app with healthchecks.

**Registry:** [`registry.coder.com/untra/operator/coder`](https://registry.coder.com/modules/operator)

Templates and modules do different jobs here: a **template** is the whole workspace blueprint (cloud, compute, storage), while a **module** adds one feature inside it. Operator is a module — you drop it into a template you already have.

### Usage

```tf
module "operator" {
  source   = "registry.coder.com/untra/operator/coder"
  agent_id = coder_agent.main.id
}
```

Pin `version` to a published module release for reproducible builds. `install_version` is a separate knob that selects which Operator release the module downloads.

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

`config_toml` is written verbatim and replaces the generated config entirely.

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

### Variables

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `agent_id` | `string` | (required) | The ID of a Coder agent |
| `port` | `number` | `7008` | The port for the operator REST API server |
| `display_name` | `string` | `"Operator"` | Display name in the Coder dashboard |
| `slug` | `string` | `"operator"` | Application slug |
| `install_version` | `string` | `"{{ site.version }}"` | Operator GitHub release tag to install |
| `install_prefix` | `string` | `"/tmp/operator"` | Directory to install the binaries into |
| `log_path` | `string` | `"/tmp/operator.log"` | Path to write log output |
| `config_toml` | `string` | `""` | Raw TOML config (written verbatim instead of auto-generated config) |
| `max_parallel_agents` | `number` | `2` | Maximum number of parallel agents |
| `session_wrapper` | `string` | `"tmux"` | Session wrapper type (`tmux`, `cmux`, or `zellij`) |
| `share` | `string` | `"owner"` | Dashboard sharing level (`owner`, `authenticated`, or `public`) |
| `order` | `number` | `null` | Position of the app in the Coder dashboard (lower = first) |
| `group` | `string` | `null` | Group that this app belongs to |
| `offline` | `bool` | `false` | Skip downloading; requires a pre-installed binary at `install_prefix` |
| `use_cached` | `bool` | `false` | Use cached binary if present, otherwise download |

### Child agent workspaces

Set `agent_template` and the generated config gains a `[[targets]]` entry with `kind = "coder"`, so tickets launched from this workspace create sibling workspaces from that template instead of running agents locally. These variables are ignored unless `agent_template` is set, and any left unset are omitted from the config so Operator's own defaults apply.

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `agent_template` | `string` | `""` | Template child agent workspaces are created from. Empty disables the coder target. |
| `coder_token_env` | `string` | `"CODER_SESSION_TOKEN"` | Name of the env var holding the Coder **user session token** |
| `callback_url` | `string` | `""` | Control-plane-reachable `OPERATOR_API_URL` override; empty keeps the reverse-tunnel default |
| `name_prefix` | `string` | `""` | Workspace name prefix for deterministic per-ticket naming |
| `workdir` | `string` | `""` | Project root inside spawned workspaces |
| `stop_on_complete` | `bool` | `null` | Stop a spawned workspace when its ticket completes (never deletes) |
| `create_timeout_secs` | `number` | `null` | Bound on workspace create plus agent-ready wait, in seconds |

This mode needs a **user session token**, which is not the ambient `CODER_AGENT_TOKEN` — that one is scoped to a single workspace and cannot create others. The module does not provision it; supply it through your template. A user session token can create, delete, and SSH into every workspace its user owns, so scope the account accordingly.

### Prerequisites

The workspace image must include `tmux` (or your chosen `session_wrapper`) for Operator to spawn agent sessions. Most Coder workspace images include tmux by default.

For child agent workspaces, the image also needs `ssh` (`openssh-client`), since agents are launched over real SSH.

### Coder workspace context

Coder automatically injects environment variables into every workspace that Operator can reference in ticket templates and agent prompts:

- `CODER_WORKSPACE_NAME` — workspace identifier
- `CODER_WORKSPACE_OWNER` — workspace owner username
- `CODER_URL` — deployment URL, which the coder target reads by default
- `CODER_AGENT_TOKEN` — agent authentication token, scoped to this workspace

No Operator configuration is needed to access these — they are ambient in the workspace environment.

### How it works

1. The module runs a startup script that detects the workspace architecture (`linux-x86_64` or `linux-arm64`)
2. Downloads the Operator binary from GitHub releases (or uses a cached/pre-installed binary), then the `opr8r` client binary that agent sessions call to report step completion for multi-step workflows.
3. Generates a TOML configuration file (or uses the provided `config_toml`)
4. Starts `operator api` as a background process
5. Registers the Operator dashboard as a Coder app with healthchecks polling `/api/v1/health` every 5 seconds

## Operator targeting Coder

Here Operator runs outside Coder — most often as the [Kubernetes deployment](/getting-started/platforms/kubernetes/) — and provisions a Coder workspace per ticket. The Terraform module is not involved.

Declare a target. `template` is an allowlist: agents can only ever land on the template you name here.

```toml
[[targets]]
name     = "cloud"
kind     = "coder"
template = "operator-agent"
```

Then reference it from a delegator, or set it as the default target. Full field reference: [execution targets](/delegators/#execution-targets).

### Credentials

Operator reads two environment variables, resolved **by name** so the values never enter the config file or the state store:

| Variable | Default name | Holds |
|----------|--------------|-------|
| `url_env` | `CODER_URL` | Your deployment URL, e.g. `https://coder.example.com` |
| `token_env` | `CODER_SESSION_TOKEN` | A Coder user session token |

A session token can create, delete, and SSH into every workspace its user owns, so give Operator its own service account rather than a human's credentials. Operator strips the token variable from every agent's spawn environment, on every target kind — including `local` agents, which would otherwise read it straight out of `env`.

Keep both variables at their default names unless you have a reason not to. The SSH `ProxyCommand` runs the `coder` CLI as a subprocess, and the CLI reads these canonical names from the inherited environment.

### The `coder` CLI

Operator does not bundle the CLI. It resolves one in this order:

1. `coder` on `PATH`
2. A previously downloaded copy in the state directory, at `.tickets/operator/bin/coder`
3. Otherwise it downloads `{CODER_URL}/bin/coder-linux-{amd64,arm64}` — your deployment serves a CLI matching its own version — and caches it at (2)

So a container needs no CLI baked in, and the CLI can never drift from the server it talks to. It does need `ssh` and outbound network access to the deployment. The official image ships `openssh-client`; if you supply your own, include it.

### Workspace lifecycle

- **Naming is deterministic:** `{name_prefix}-{project}-{ticket_id}`, sanitized and capped at Coder's 32-character limit. Relaunching a ticket reuses its workspace.
- **Create or start:** absent workspaces are created from `template`; existing ones on that template are started.
- **Collisions are refused:** a workspace of the same name on a *different* template stops the launch rather than being reused, so Operator can never adopt a workspace a human made.
- **Never deleted:** completed workspaces are stopped (when `stop_on_complete` is set). Reclamation stays with your Coder autostop and autodelete policy.

Operator writes its own per-workspace SSH config fragment under `.tickets/operator/ssh/` rather than running `coder config-ssh`, which would rewrite `~/.ssh/config`. The fragment proxies through `coder ssh --stdio` and skips host-key checking, matching what `coder config-ssh` writes for its own hosts: the Coder tailnet is the authentication boundary, and per-ticket workspaces are too short-lived for trust-on-first-use to be meaningful.

### Constraints

For `coder` targets, as for `ssh` targets, git worktrees and relay MCP injection are forced off, and the `zellij` session wrapper is unsupported.

## Troubleshooting

### Binary download fails (module)

1. Check that the `install_version` matches a valid [GitHub release tag](https://github.com/untra/operator/releases)
2. Verify the workspace has internet access (or use `offline = true` with a pre-installed binary)
3. Check logs at the configured `log_path` (default: `/tmp/operator.log`)

### Healthcheck timeout (module)

1. Verify the port is not already in use: `ss -tlnp | grep 7008`
2. Check operator logs: `cat /tmp/operator.log`
3. Ensure the session wrapper (tmux by default) is installed in the workspace image

### Port conflicts (module)

Change the `port` variable to an unused port. Remember to update any other services or extensions that connect to the Operator API.

### A coder target fails to launch

Operator fails fast and names what is missing. In order:

1. **A missing environment variable** — the error names it. Confirm `CODER_URL` and the session token are present in Operator's own environment, not just the agent's.
2. **The CLI download fails** — the error names the URL it tried. Usually egress: from a container, check reachability directly, e.g. `curl -sSf $CODER_URL/api/v2/buildinfo`. In Kubernetes this is commonly Coder's *own* ingress NetworkPolicy declining to admit Operator's namespace, which is a fix on the Coder side.
3. **`coder create` fails** — the message is Coder's own, verbatim. Template permissions and workspace quotas surface here.
4. **The workspace never becomes reachable over SSH** within `create_timeout_secs` — the template's agent is not starting, or `ssh` is missing from Operator's environment.
5. **A refused name collision** — a workspace of that name already exists on another template. Rename or remove it.

---
title: "Configuration"
layout: doc
---

<!-- AUTO-GENERATED FROM src/config.rs - DO NOT EDIT MANUALLY -->
<!-- Regenerate with: cargo run -- docs -->

# Configuration

Operator configuration is stored in `.tickets/operator/config.toml`.

## Configuration Sections

| Section | Description |
| --- | --- |
| `[agents]` | Agent lifecycle, parallelism, and health monitoring |
| `[notifications]` | macOS notification preferences |
| `[queue]` | Queue processing and ticket assignment |
| `[paths]` | Directory paths for tickets, projects, and state |
| `[ui]` | Terminal UI appearance and behavior |
| `[launch]` | Agent launch behavior and confirmations |
| `[templates]` | Issue type collections and presets |
| `[api]` | External API integration settings |
| `[logging]` | Log level and output configuration |
| `[tmux]` | Tmux integration settings |
| `[backstage]` | Backstage server integration |
| `[llm_tools]` | LLM CLI tool detection and providers |

## `[agents]`

Agent lifecycle, parallelism, and health monitoring

| Field | Type | Default | Description |
| --- | --- | --- | --- |
| `cores_reserved` * | `integer` | 1 |  |
| `generation_timeout_secs` | `integer` | 300 | Timeout in seconds for each agent generation (default: 300 = 5 min) |
| `health_check_interval` * | `integer` | 30 |  |
| `max_parallel` * | `integer` | 5 |  |
| `silence_threshold` | `integer` | 30 | Seconds of tmux silence before considering agent awaiting input (default: 30) |
| `step_timeout` | `integer` | 1800 | Maximum seconds a step can run before timing out (default: 1800 = 30 min) |
| `sync_interval` | `integer` | 60 | Interval in seconds between ticket-session syncs (default: 60) |

## `[notifications]`

macOS notification preferences

| Field | Type | Default | Description |
| --- | --- | --- | --- |
| `enabled` * | `boolean` | true |  |
| `on_agent_complete` * | `boolean` | true |  |
| `on_agent_needs_input` * | `boolean` | true |  |
| `on_agent_start` * | `boolean` | true |  |
| `on_investigation_created` * | `boolean` | true |  |
| `on_pr_created` * | `boolean` | true |  |
| `sound` * | `boolean` | false |  |

## `[queue]`

Queue processing and ticket assignment

| Field | Type | Default | Description |
| --- | --- | --- | --- |
| `auto_assign` * | `boolean` | true |  |
| `poll_interval_ms` * | `integer` | 1000 |  |
| `priority_order` * | `array`[`string`] | ["INV", "FIX", "TASK", "FEAT", "SPIKE"] |  |

## `[paths]`

Directory paths for tickets, projects, and state

| Field | Type | Default | Description |
| --- | --- | --- | --- |
| `projects` * | `string` | . |  |
| `state` * | `string` | .tickets/operator |  |
| `tickets` * | `string` | .tickets |  |

## `[ui]`

Terminal UI appearance and behavior

| Field | Type | Default | Description |
| --- | --- | --- | --- |
| `completed_history_hours` * | `integer` | 24 |  |
| `panel_names` | → `PanelNamesConfig` | - |  |
| `refresh_rate_ms` * | `integer` | 250 |  |
| `summary_max_length` * | `integer` | 40 |  |

## `[launch]`

Agent launch behavior and confirmations

| Field | Type | Default | Description |
| --- | --- | --- | --- |
| `confirm_autonomous` * | `boolean` | true |  |
| `confirm_paired` * | `boolean` | true |  |
| `launch_delay_ms` * | `integer` | 2000 |  |

## `[templates]`

Issue type collections and presets

| Field | Type | Default | Description |
| --- | --- | --- | --- |
| `active_collection` | `string` \| `null` | - | Active collection name (overrides preset if set) Can be a builtin preset name or a user-defined collection |
| `collection` | `array`[`string`] | - | Custom issuetype collection (only used when preset = custom) List of issue type keys: TASK, FEAT, FIX, SPIKE, INV |
| `preset` | → `CollectionPreset` | - | Named preset for issue type collection Options: simple, dev_kanban, devops_kanban, custom |

## `[api]`

External API integration settings

| Field | Type | Default | Description |
| --- | --- | --- | --- |
| `pr_check_interval_secs` | `integer` | 60 | Interval in seconds between PR status checks (default: 60) |
| `rate_limit_check_interval_secs` | `integer` | 300 | Interval in seconds between rate limit checks (default: 300) |
| `rate_limit_warning_threshold` | `number` | 0.2 | Show warning when rate limit remaining is below this percentage (default: 0.2) |

## `[logging]`

Log level and output configuration

| Field | Type | Default | Description |
| --- | --- | --- | --- |
| `level` | `string` | info | Log level filter (trace, debug, info, warn, error) |
| `to_file` | `boolean` | true | Whether to log to file in TUI mode (false = stderr for debugging) |

## `[tmux]`

Tmux integration settings

| Field | Type | Default | Description |
| --- | --- | --- | --- |
| `config_generated` | `boolean` | - | Whether custom tmux config has been generated |

## `[backstage]`

Backstage server integration

| Field | Type | Default | Description |
| --- | --- | --- | --- |
| `auto_start` | `boolean` | false | Auto-start Backstage server when TUI launches |
| `branding_subpath` | `string` | branding | Subdirectory within backstage path for branding customization |
| `enabled` | `boolean` | true | Whether Backstage integration is enabled |
| `port` | `integer` | 7007 | Port for the Backstage server |
| `subpath` | `string` | backstage | Subdirectory within state_path for Backstage installation |

## `[llm_tools]`

LLM CLI tool detection and providers

| Field | Type | Default | Description |
| --- | --- | --- | --- |
| `detected` | `array`[→ `DetectedTool`] | - | Detected CLI tools (populated on first startup) |
| `detection_complete` | `boolean` | - | Whether detection has been completed |
| `providers` | `array`[→ `LlmProvider`] | - | Available {tool, model} pairs for launching tickets Built from detected tools + their model aliases |

## Example Configuration

```toml
projects = []

[agents]
max_parallel = 5
cores_reserved = 1
health_check_interval = 30
generation_timeout_secs = 300
sync_interval = 60
step_timeout = 1800
silence_threshold = 30

[notifications]
enabled = true
on_agent_start = true
on_agent_complete = true
on_agent_needs_input = true
on_pr_created = true
on_investigation_created = true
sound = false

[queue]
auto_assign = true
priority_order = [
    "INV",
    "FIX",
    "TASK",
    "FEAT",
    "SPIKE",
]
poll_interval_ms = 1000

[paths]
tickets = ".tickets"
projects = "."
state = ".tickets/operator"

[ui]
refresh_rate_ms = 250
completed_history_hours = 24
summary_max_length = 40

[ui.panel_names]
queue = "TODO QUEUE"
agents = "DOING"
awaiting = "AWAITING"
completed = "DONE"

[launch]
confirm_autonomous = true
confirm_paired = true
launch_delay_ms = 2000

[templates]
preset = "devops_kanban"
collection = []

[api]
pr_check_interval_secs = 60
rate_limit_check_interval_secs = 300
rate_limit_warning_threshold = 0.20000000298023224

[logging]
level = "info"
to_file = true

[tmux]
config_generated = false

[llm_tools]
detected = []
providers = []
detection_complete = false

[backstage]
enabled = true
port = 7007
auto_start = false
subpath = "backstage"
branding_subpath = "branding"

```

## Configuration Files

Configuration is loaded in this order (later sources override earlier):

1. **Built-in defaults** - Embedded in the binary
2. **Project config** - `.tickets/operator/config.toml`
3. **User config** - `~/.config/operator/config.toml`
4. **CLI flag** - `--config <path>`
5. **Environment variables** - `OPERATOR_*` prefix with `__` separator

### Environment Variable Override

Any configuration option can be overridden via environment variables.

**Format**: `OPERATOR_<SECTION>__<FIELD>`

**Examples**:
- `OPERATOR_AGENTS__MAX_PARALLEL=2`
- `OPERATOR_LOGGING__LEVEL=debug`
- `OPERATOR_BACKSTAGE__PORT=8080`


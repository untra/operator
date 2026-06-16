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
| `[llm_tools]` | LLM CLI tool detection and providers |

## `[agents]`

Agent lifecycle, parallelism, and health monitoring

| Field | Type | Default | Description |
| --- | --- | --- | --- |
| `max_parallel` * | `integer` | 5 |  |
| `cores_reserved` * | `integer` | 1 |  |
| `max_agents_per_repo` | `integer` | - | Maximum concurrent agents per project/repo (default: 1). Requires `git.use_worktrees` = true when > 1 to avoid conflicts. |
| `health_check_interval` * | `integer` | 30 |  |
| `generation_timeout_secs` | `integer` | 300 | Timeout in seconds for each agent generation (default: 300 = 5 min) |
| `sync_interval` | `integer` | 60 | Interval in seconds between ticket-session syncs (default: 60) |
| `step_timeout` | `integer` | 1800 | Maximum seconds a step can run before timing out (default: 1800 = 30 min) |
| `silence_threshold` | `integer` | 30 | Seconds of tmux silence before considering agent awaiting input (default: 30) |

## `[notifications]`

macOS notification preferences

| Field | Type | Default | Description |
| --- | --- | --- | --- |
| `enabled` * | `boolean` | true | Global enabled flag for all notifications |
| `os` | → `OsNotificationConfig` | - | OS notification configuration |
| `webhook` | `any` | - | Single webhook configuration (for simple setups) |
| `webhooks` | `array`[→ `WebhookConfig`] | - | Multiple webhook configurations |

## `[queue]`

Queue processing and ticket assignment

| Field | Type | Default | Description |
| --- | --- | --- | --- |
| `auto_assign` * | `boolean` | true |  |
| `priority_order` * | `array`[`string`] | ["INV", "FIX", "TASK", "FEAT", "SPIKE"] |  |
| `poll_interval_ms` * | `integer` | 1000 |  |

## `[paths]`

Directory paths for tickets, projects, and state

| Field | Type | Default | Description |
| --- | --- | --- | --- |
| `tickets` * | `string` | .tickets |  |
| `projects` * | `string` | . |  |
| `state` * | `string` | .tickets/operator |  |
| `worktrees` | `string` | - | Base directory for per-ticket worktrees (default: ~/.operator/worktrees) |

## `[ui]`

Terminal UI appearance and behavior

| Field | Type | Default | Description |
| --- | --- | --- | --- |
| `refresh_rate_ms` * | `integer` | 250 |  |
| `completed_history_hours` * | `integer` | 24 |  |
| `summary_max_length` * | `integer` | 40 |  |
| `panel_names` | → `PanelNamesConfig` | - |  |

## `[launch]`

Agent launch behavior and confirmations

| Field | Type | Default | Description |
| --- | --- | --- | --- |
| `confirm_autonomous` * | `boolean` | true |  |
| `confirm_paired` * | `boolean` | true |  |
| `launch_delay_ms` * | `integer` | 2000 |  |
| `docker` | → `DockerConfig` | - | Docker execution configuration |
| `yolo` | → `YoloConfig` | - | YOLO (auto-accept) mode configuration |

## `[templates]`

Issue type collections and presets

| Field | Type | Default | Description |
| --- | --- | --- | --- |
| `preset` | → `CollectionPreset` | - | Named preset for issue type collection Options: simple, `dev_kanban`, `devops_kanban`, custom |
| `collection` | `array`[`string`] | - | Custom issuetype collection (only used when preset = custom) List of issue type keys: TASK, FEAT, FIX, SPIKE, INV |
| `active_collection` | `string` \| `null` | - | Active collection name (overrides preset if set) Can be a builtin preset name or a user-defined collection |
| `collections_fetch_enabled` | `boolean` | - | Enable fetching hosted issuetype collections during setup. When disabled, only the embedded (offline) collections are offered. |
| `collections_manifest_url` | `string` \| `null` | - | URL of the hosted collection index manifest, fetched during setup. Points at a `CollectionIndex` JSON document listing available collections. |
| `collections_fetch_timeout_secs` | `integer` | - | Timeout in seconds for hosted collection fetch HTTP requests. |

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

## `[llm_tools]`

LLM CLI tool detection and providers

| Field | Type | Default | Description |
| --- | --- | --- | --- |
| `detected` | `array`[→ `DetectedTool`] | - | Detected CLI tools (populated on first startup) |
| `providers` | `array`[→ `LlmProvider`] | - | Available {tool, model} pairs for launching tickets Built from detected tools + their model aliases |
| `detection_complete` | `boolean` | - | Whether detection has been completed |
| `default_tool` | `string` \| `null` | - | User's preferred default LLM tool (e.g., "claude") |
| `default_model` | `string` \| `null` | - | User's preferred default model alias (e.g., "opus") |
| `skill_directory_overrides` | `object` | - | Per-tool overrides for skill directories (keyed by `tool_name`) |

## Example Configuration

```toml
projects = []
delegators = []
model_servers = []

[agents]
max_parallel = 5
cores_reserved = 1
max_agents_per_repo = 1
health_check_interval = 30
generation_timeout_secs = 300
sync_interval = 60
step_timeout = 1800
silence_threshold = 30

[notifications]
enabled = true
webhooks = []
on_agent_start = true
on_agent_complete = true
on_agent_needs_input = true
on_pr_created = true
on_investigation_created = true
sound = false

[notifications.os]
enabled = true
sound = false
events = []

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
worktrees = "/Users/samuelvolin/.operator/worktrees"

[ui]
refresh_rate_ms = 250
completed_history_hours = 24
summary_max_length = 40

[ui.panel_names]
status = "STATUS"
queue = "TODO QUEUE"
in_progress = "IN PROGRESS"
completed = "DONE"

[launch]
confirm_autonomous = true
confirm_paired = true
launch_delay_ms = 2000

[launch.docker]
enabled = false
image = ""
extra_args = []
mount_path = "/workspace"
env_vars = []

[launch.yolo]
enabled = false

[templates]
preset = "dev_kanban"
collection = []
collections_fetch_enabled = true
collections_manifest_url = "https://operator.untra.io/collections/index.json"
collections_fetch_timeout_secs = 5

[api]
pr_check_interval_secs = 60
rate_limit_check_interval_secs = 300
rate_limit_warning_threshold = 0.2

[logging]
level = "info"
to_file = true

[tmux]
config_generated = false

[sessions]
wrapper = "tmux"

[sessions.tmux]
config_generated = false
socket_name = "operator"

[sessions.vscode]
webhook_port = 7009
connect_timeout_ms = 5000

[sessions.cmux]
binary_path = "/Applications/cmux.app/Contents/Resources/bin/cmux"
require_in_cmux = true
placement = "auto"

[sessions.zellij]
require_in_zellij = true

[llm_tools]
detected = []
providers = []
detection_complete = false

[llm_tools.skill_directory_overrides]

[rest_api]
enabled = true
host = "127.0.0.1"
port = 7008
cors_origins = []

[git]
branch_format = "{type}/{ticket_id}"
use_worktrees = false

[git.github]
enabled = false
token_env = ""

[git.gitlab]
enabled = false
token_env = ""

[kanban.jira]

[kanban.linear]

[kanban.github]

[version_check]
enabled = true
url = "https://operator.untra.io/VERSION"
timeout_secs = 3

[relay]
auto_inject_mcp = false

[mcp]
http_enabled = true
stdio_advertised = true
expose_ticket_write_tools = false
external_servers = []

[acp]
stdio_advertised = true
max_concurrent_sessions = 8

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
- `OPERATOR_TMUX__ENABLED=true`


---
title: "Configuration Schema"
layout: doc
---

<!-- AUTO-GENERATED FROM docs/schemas/config.json - DO NOT EDIT MANUALLY -->
<!-- Regenerate with: cargo run -- docs -->

# Configuration Schema

JSON Schema for the Operator configuration file (`config.toml`).

## Schema Information

- **$schema**: `https://json-schema.org/draft/2020-12/schema`
- **title**: `Config`

## Required Fields

- `agents`
- `notifications`
- `queue`
- `paths`
- `ui`
- `launch`
- `templates`

## Properties

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `projects` | `array` | No | List of projects operator can assign work to |
| `agents` | → `AgentsConfig` | Yes |  |
| `notifications` | → `NotificationsConfig` | Yes |  |
| `queue` | → `QueueConfig` | Yes |  |
| `paths` | → `PathsConfig` | Yes |  |
| `ui` | → `UiConfig` | Yes |  |
| `launch` | → `LaunchConfig` | Yes |  |
| `templates` | → `TemplatesConfig` | Yes |  |
| `api` | → `ApiConfig` | No |  |
| `logging` | → `LoggingConfig` | No |  |
| `tmux` | → `TmuxConfig` | No |  |
| `llm_tools` | → `LlmToolsConfig` | No |  |
| `backstage` | → `BackstageConfig` | No |  |
| `rest_api` | → `RestApiConfig` | No |  |

## Type Definitions

### AgentsConfig

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `max_parallel` | `integer` | Yes |  |
| `cores_reserved` | `integer` | Yes |  |
| `health_check_interval` | `integer` | Yes |  |
| `generation_timeout_secs` | `integer` | No | Timeout in seconds for each agent generation (default: 300 = 5 min) |
| `sync_interval` | `integer` | No | Interval in seconds between ticket-session syncs (default: 60) |
| `step_timeout` | `integer` | No | Maximum seconds a step can run before timing out (default: 1800 = 30 min) |
| `silence_threshold` | `integer` | No | Seconds of tmux silence before considering agent awaiting input (default: 30) |

### NotificationsConfig

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `enabled` | `boolean` | Yes |  |
| `on_agent_start` | `boolean` | Yes |  |
| `on_agent_complete` | `boolean` | Yes |  |
| `on_agent_needs_input` | `boolean` | Yes |  |
| `on_pr_created` | `boolean` | Yes |  |
| `on_investigation_created` | `boolean` | Yes |  |
| `sound` | `boolean` | Yes |  |

### QueueConfig

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `auto_assign` | `boolean` | Yes |  |
| `priority_order` | `array` | Yes |  |
| `poll_interval_ms` | `integer` | Yes |  |

### PathsConfig

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `tickets` | `string` | Yes |  |
| `projects` | `string` | Yes |  |
| `state` | `string` | Yes |  |

### UiConfig

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `refresh_rate_ms` | `integer` | Yes |  |
| `completed_history_hours` | `integer` | Yes |  |
| `summary_max_length` | `integer` | Yes |  |
| `panel_names` | → `PanelNamesConfig` | No |  |

### PanelNamesConfig

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `queue` | `string` | No |  |
| `agents` | `string` | No |  |
| `awaiting` | `string` | No |  |
| `completed` | `string` | No |  |

### LaunchConfig

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `confirm_autonomous` | `boolean` | Yes |  |
| `confirm_paired` | `boolean` | Yes |  |
| `launch_delay_ms` | `integer` | Yes |  |
| `docker` | → `DockerConfig` | No | Docker execution configuration |
| `yolo` | → `YoloConfig` | No | YOLO (auto-accept) mode configuration |

### DockerConfig

Docker execution configuration for running agents in containers

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `enabled` | `boolean` | No | Whether docker mode option is available in launch dialog |
| `image` | `string` | No | Docker image to use (required if enabled) |
| `extra_args` | `array` | No | Additional docker run arguments |
| `mount_path` | `string` | No | Container mount path for the project (default: /workspace) |
| `env_vars` | `array` | No | Environment variables to pass through to the container |

### YoloConfig

YOLO (auto-accept) mode configuration for fully autonomous execution

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `enabled` | `boolean` | No | Whether YOLO mode option is available in launch dialog |

### TemplatesConfig

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `preset` | → `CollectionPreset` | No | Named preset for issue type collection Options: simple, dev_kanban, devops_kanban, custom |
| `collection` | `array` | No | Custom issuetype collection (only used when preset = custom) List of issue type keys: TASK, FEAT, FIX, SPIKE, INV |
| `active_collection` | `string` \| `null` | No | Active collection name (overrides preset if set) Can be a builtin preset name or a user-defined collection |

### CollectionPreset

Predefined issue type collections

**Allowed Values:**

- `simple` - Simple tasks only: TASK
- `dev_kanban` - Developer kanban: TASK, FEAT, FIX
- `devops_kanban` - DevOps kanban: TASK, SPIKE, INV, FEAT, FIX
- `custom` - Custom collection (use the collection field)

### ApiConfig

API integrations configuration

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `pr_check_interval_secs` | `integer` | No | Interval in seconds between PR status checks (default: 60) |
| `rate_limit_check_interval_secs` | `integer` | No | Interval in seconds between rate limit checks (default: 300) |
| `rate_limit_warning_threshold` | `number` | No | Show warning when rate limit remaining is below this percentage (default: 0.2) |

### LoggingConfig

Logging configuration

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `level` | `string` | No | Log level filter (trace, debug, info, warn, error) |
| `to_file` | `boolean` | No | Whether to log to file in TUI mode (false = stderr for debugging) |

### TmuxConfig

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `config_generated` | `boolean` | No | Whether custom tmux config has been generated |

### LlmToolsConfig

LLM CLI tools configuration

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `detected` | `array` | No | Detected CLI tools (populated on first startup) |
| `providers` | `array` | No | Available {tool, model} pairs for launching tickets Built from detected tools + their model aliases |
| `detection_complete` | `boolean` | No | Whether detection has been completed |

### DetectedTool

A detected CLI tool (e.g., claude binary)

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `name` | `string` | Yes | Tool name (e.g., "claude") |
| `path` | `string` | Yes | Path to the binary |
| `version` | `string` | Yes | Version string |
| `model_aliases` | `array` | Yes | Available model aliases (e.g., ["opus", "sonnet", "haiku"]) |
| `command_template` | `string` | No | Command template with {{model}}, {{session_id}}, {{prompt_file}} placeholders |
| `capabilities` | → `ToolCapabilities` | No | Tool capabilities |
| `yolo_flags` | `array` | No | CLI flags for YOLO (auto-accept) mode |

### ToolCapabilities

Tool capabilities

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `supports_sessions` | `boolean` | No | Whether the tool supports session continuity via UUID |
| `supports_headless` | `boolean` | No | Whether the tool can run in headless/non-interactive mode |

### LlmProvider

A {tool, model} pair that can be selected when launching tickets

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `tool` | `string` | Yes | CLI tool name (e.g., "claude") |
| `model` | `string` | Yes | Model alias or name (e.g., "opus", "sonnet") |
| `display_name` | `string` \| `null` | No | Optional display name for UI (e.g., "Claude Opus") |

### BackstageConfig

Backstage integration configuration

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `enabled` | `boolean` | No | Whether Backstage integration is enabled |
| `port` | `integer` | No | Port for the Backstage server |
| `auto_start` | `boolean` | No | Auto-start Backstage server when TUI launches |
| `subpath` | `string` | No | Subdirectory within state_path for Backstage installation |
| `branding_subpath` | `string` | No | Subdirectory within backstage path for branding customization |
| `release_url` | `string` | No | Base URL for downloading backstage-server binary |
| `local_binary_path` | `string` \| `null` | No | Optional local path to backstage-server binary If set, this is used instead of downloading from release_url |
| `branding` | → `BrandingConfig` | No | Branding and theming configuration |

### BrandingConfig

Branding configuration for Backstage portal

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `app_title` | `string` | No | App title shown in header |
| `org_name` | `string` | No | Organization name |
| `logo_path` | `string` \| `null` | No | Path to logo SVG (relative to branding path) |
| `colors` | → `ThemeColors` | No | Theme colors (uses Operator defaults if not set) |

### ThemeColors

Theme color configuration for Backstage
Default colors match Operator's tmux theme

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `primary` | `string` | No | Primary/accent color (default: salmon #cc6c55) |
| `secondary` | `string` | No | Secondary color (default: dark teal #114145) |
| `accent` | `string` | No | Accent/highlight color (default: cream #f4dbb7) |
| `warning` | `string` | No | Warning/error color (default: coral #d46048) |
| `muted` | `string` | No | Muted text color (default: darker salmon #8a4a3a) |

### RestApiConfig

REST API server configuration

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `enabled` | `boolean` | No | Whether the REST API is enabled |
| `port` | `integer` | No | Port for the REST API server |
| `cors_origins` | `array` | No | CORS allowed origins (empty = allow all) |


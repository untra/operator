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
| `sessions` | → `SessionsConfig` | No | Session wrapper configuration (tmux, vscode, or cmux) |
| `llm_tools` | → `LlmToolsConfig` | No |  |
| `backstage` | → `BackstageConfig` | No |  |
| `rest_api` | → `RestApiConfig` | No |  |
| `git` | → `GitConfig` | No |  |
| `kanban` | → `KanbanConfig` | No | Kanban provider configuration for syncing issues from Jira, Linear, etc. |
| `version_check` | → `VersionCheckConfig` | No | Version check configuration for automatic update notifications |
| `delegators` | `array` | No | Agent delegator configurations for autonomous ticket launching |

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

Notifications configuration with support for multiple integrations.

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `enabled` | `boolean` | Yes | Global enabled flag for all notifications |
| `os` | → `OsNotificationConfig` | No | OS notification configuration |
| `webhook` | object | No | Single webhook configuration (for simple setups) |
| `webhooks` | `array` | No | Multiple webhook configurations |

### OsNotificationConfig

OS notification configuration.

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `enabled` | `boolean` | No | Whether OS notifications are enabled |
| `sound` | `boolean` | No | Play sound with notifications |
| `events` | `array` | No | Events to send (empty = all events) Possible values: agent.started, agent.completed, agent.failed, agent.awaiting_input, agent.session_lost, pr.created, pr.merged, pr.closed, pr.ready_to_merge, pr.changes_requested, ticket.returned, investigation.created |

### WebhookConfig

Webhook notification configuration.

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `name` | `string` \| `null` | No | Optional name for this webhook (for logging) |
| `enabled` | `boolean` | No | Whether this webhook is enabled |
| `url` | `string` | No | Webhook URL |
| `auth_type` | `string` \| `null` | No | Authentication type: "bearer" or "basic" |
| `token_env` | `string` \| `null` | No | Environment variable containing the bearer token |
| `username` | `string` \| `null` | No | Username for basic auth |
| `password_env` | `string` \| `null` | No | Environment variable containing the password for basic auth |
| `events` | `array` \| `null` | No | Events to send (empty = all events) |

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
| `worktrees` | `string` | No | Base directory for per-ticket worktrees (default: ~/.operator/worktrees) |

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

### SessionsConfig

Session wrapper configuration

Controls how operator creates and manages terminal sessions for agents.
Three modes are supported:
- tmux: Standalone tmux sessions (default)
- vscode: VS Code integrated terminal (requires extension)
- cmux: macOS terminal multiplexer (requires running inside cmux)

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `wrapper` | → `SessionWrapperType` | No | Which session wrapper to use |
| `tmux` | → `SessionsTmuxConfig` | No | Tmux-specific configuration |
| `vscode` | → `SessionsVSCodeConfig` | No | VS Code-specific configuration |
| `cmux` | → `SessionsCmuxConfig` | No | cmux-specific configuration |

### SessionWrapperType

Session wrapper type for terminal session management

**Allowed Values:**

- `tmux` - Standalone tmux sessions (default)
- `vscode` - VS Code integrated terminal (via extension webhook)
- `cmux` - cmux macOS terminal multiplexer

### SessionsTmuxConfig

Tmux-specific session configuration

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `config_generated` | `boolean` | No | Whether custom tmux config has been generated |
| `socket_name` | `string` | No | Socket name for session isolation |

### SessionsVSCodeConfig

VS Code extension session configuration

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `webhook_port` | `integer` | No | Port for extension webhook server |
| `connect_timeout_ms` | `integer` | No | Connection timeout in milliseconds |

### SessionsCmuxConfig

cmux macOS terminal multiplexer session configuration

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `binary_path` | `string` | No | Path to the cmux binary |
| `require_in_cmux` | `boolean` | No | Require running inside cmux (CMUX_WORKSPACE_ID env var present) |
| `placement` | → `CmuxPlacementPolicy` | No | Where to place new agent sessions: "auto", "workspace", or "window" |

### CmuxPlacementPolicy

Placement policy for cmux sessions: where to create new agent terminals

**Allowed Values:**

- `auto` - Automatically choose: 0-1 windows → new workspace, >1 windows → new window
- `workspace` - Always create a new workspace in the active window
- `window` - Always create a new window for each ticket

### LlmToolsConfig

LLM CLI tools configuration

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `detected` | `array` | No | Detected CLI tools (populated on first startup) |
| `providers` | `array` | No | Available {tool, model} pairs for launching tickets Built from detected tools + their model aliases |
| `detection_complete` | `boolean` | No | Whether detection has been completed |
| `skill_directory_overrides` | `object` | No | Per-tool overrides for skill directories (keyed by tool_name) |

### DetectedTool

A detected CLI tool (e.g., claude binary)

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `name` | `string` | Yes | Tool name (e.g., "claude") |
| `path` | `string` | Yes | Path to the binary |
| `version` | `string` | Yes | Version string |
| `min_version` | `string` \| `null` | No | Minimum required version for Operator compatibility |
| `version_ok` | `boolean` | No | Whether the installed version meets the minimum requirement |
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

A {tool, model} pair that can be selected when launching tickets.
Includes optional variant fields adopted from vibe-kanban's profile system.

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `tool` | `string` | Yes | CLI tool name (e.g., "claude", "codex", "gemini") |
| `model` | `string` | Yes | Model alias or name (e.g., "opus", "sonnet", "gpt-4.1") |
| `display_name` | `string` \| `null` | No | Optional display name for UI (e.g., "Claude Opus", "Codex High") |
| `flags` | `array` | No | Additional CLI flags for this provider (e.g., ["--dangerously-skip-permissions"]) |
| `env` | `object` | No | Environment variables to set when launching |
| `approvals` | `boolean` | No | Whether this provider requires approval gates |
| `plan_only` | `boolean` | No | Whether to run in plan-only mode |
| `reasoning_effort` | `string` \| `null` | No | Reasoning effort level (Codex: "low", "medium", "high") |
| `sandbox` | `string` \| `null` | No | Sandbox mode (Codex: "danger-full-access", "workspace-write") |

### SkillDirectoriesOverride

Per-tool skill directory overrides

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `global` | `array` | No | Additional global skill directories |
| `project` | `array` | No | Additional project-relative skill directories |

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

### GitConfig

Git provider configuration for PR/MR operations

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `provider` | object | No | Active provider (auto-detected from remote URL if not specified) |
| `github` | → `GitHubConfig` | No | GitHub-specific configuration |
| `gitlab` | → `GitLabConfig` | No | GitLab-specific configuration (planned) |
| `branch_format` | `string` | No | Branch naming format (e.g., "{type}/{ticket_id}-{slug}") |
| `use_worktrees` | `boolean` | No | Whether to use git worktrees for per-ticket isolation (default: false) When false, tickets work directly in the project directory with branches |

### GitProviderConfig

Git provider selection

**Allowed Values:**

- `github` - GitHub (github.com)
- `gitlab` - GitLab (gitlab.com or self-hosted)
- `bitbucket` - Bitbucket (bitbucket.org)
- `azuredevops` - Azure DevOps (dev.azure.com)

### GitHubConfig

GitHub-specific configuration

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `enabled` | `boolean` | No | Whether GitHub integration is enabled |
| `token_env` | `string` | No | Environment variable containing the GitHub token (default: GITHUB_TOKEN) |

### GitLabConfig

GitLab-specific configuration (planned)

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `enabled` | `boolean` | No | Whether GitLab integration is enabled |
| `token_env` | `string` | No | Environment variable containing the GitLab token (default: GITLAB_TOKEN) |
| `host` | `string` \| `null` | No | GitLab host (default: gitlab.com, can be self-hosted) |

### KanbanConfig

Kanban provider configuration for syncing issues from external systems

Providers are keyed by domain/workspace:
- Jira: keyed by domain (e.g., "foobar.atlassian.net")
- Linear: keyed by workspace slug (e.g., "myworkspace")

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `jira` | `object` | No | Jira Cloud instances keyed by domain (e.g., "foobar.atlassian.net") |
| `linear` | `object` | No | Linear instances keyed by workspace slug |

### JiraConfig

Jira Cloud provider configuration

The domain is specified as the HashMap key in KanbanConfig.jira

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `enabled` | `boolean` | No | Whether this provider is enabled |
| `api_key_env` | `string` | No | Environment variable name containing the API key (default: OPERATOR_JIRA_API_KEY) |
| `email` | `string` | Yes | Atlassian account email for authentication |
| `projects` | `object` | No | Per-project sync configuration |

### ProjectSyncConfig

Per-project/team sync configuration for a kanban provider

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `sync_user_id` | `string` | No | User ID to sync issues for (provider-specific format) - Jira: accountId (e.g., "5e3f7acd9876543210abcdef") - Linear: user ID (e.g., "abc12345-6789-0abc-def0-123456789abc") |
| `sync_statuses` | `array` | No | Workflow statuses to sync (empty = default/first status only) |
| `collection_name` | `string` | No | IssueTypeCollection name this project maps to |
| `type_mappings` | `object` | No | Optional explicit mapping overrides: external issue type name → operator issue type key When empty, convention-based auto-matching is used (Bug→FIX, Story→FEAT, etc.) |

### LinearConfig

Linear provider configuration

The workspace slug is specified as the HashMap key in KanbanConfig.linear

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `enabled` | `boolean` | No | Whether this provider is enabled |
| `api_key_env` | `string` | No | Environment variable name containing the API key (default: OPERATOR_LINEAR_API_KEY) |
| `projects` | `object` | No | Per-team sync configuration |

### VersionCheckConfig

Version check configuration for automatic update notifications

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `enabled` | `boolean` | No | Enable automatic version checking on startup |
| `url` | `string` \| `null` | No | URL to fetch latest version from (optional, can be removed) |
| `timeout_secs` | `integer` | No | Timeout in seconds for version check HTTP request |

### Delegator

Agent delegator configuration for autonomous ticket launching

A delegator is a named {tool, model} pairing with optional launch configuration
that can be used to launch agents for tickets.

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `name` | `string` | Yes | Unique name for this delegator (e.g., "claude-opus-auto") |
| `llm_tool` | `string` | Yes | LLM tool name (must match a detected tool, e.g., "claude", "codex") |
| `model` | `string` | Yes | Model alias (e.g., "opus", "sonnet", "gpt-4o") |
| `display_name` | `string` \| `null` | No | Optional display name for UI |
| `model_properties` | `object` | No | Arbitrary model properties (e.g., reasoning_effort, sandbox) |
| `launch_config` | object | No | Optional launch configuration |

### DelegatorLaunchConfig

Launch configuration for a delegator

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `yolo` | `boolean` | No | Run in YOLO (auto-accept) mode |
| `permission_mode` | `string` \| `null` | No | Permission mode override |
| `flags` | `array` | No | Additional CLI flags |


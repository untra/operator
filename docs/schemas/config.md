---
title: "Configuration Schema"
layout: doc
---

<!-- AUTO-GENERATED FROM docs/schemas/config.json - DO NOT EDIT MANUALLY -->
<!-- Regenerate with: cargo run -- docs -->

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
| `profile` | → `ProfileIdentity` | No | Active configuration profile (id and name). |
| `projects` | `array` | No | List of projects operator can assign work to |
| `agents` | → `AgentsConfig` | Yes | Agent lifecycle, parallelism, and health monitoring. |
| `notifications` | → `NotificationsConfig` | Yes | Notification integrations. |
| `queue` | → `QueueConfig` | Yes | Queue ordering and assignment. |
| `paths` | → `PathsConfig` | Yes | Ticket, project, state, and worktree directories. |
| `ui` | → `UiConfig` | Yes | Terminal UI appearance and behavior. |
| `launch` | → `LaunchConfig` | Yes | Launch confirmations and execution defaults. |
| `templates` | → `TemplatesConfig` | Yes | Issue type collections and presets. |
| `api` | → `ApiConfig` | No | Pull-request and rate-limit polling. |
| `logging` | → `LoggingConfig` | No | Log level and output. |
| `telemetry` | → `TelemetryConfig` | No | OTLP ingest, session window, and export. |
| `tmux` | → `TmuxConfig` | No | Tmux integration configuration. |
| `sessions` | → `SessionsConfig` | No | Session wrapper configuration (tmux, vscode, or cmux) |
| `llm_tools` | → `LlmToolsConfig` | No | Detected LLM CLIs and provider preferences. |
| `rest_api` | → `RestApiConfig` | No | Embedded REST API server. |
| `git` | → `GitConfig` | No | Git provider, identity, and worktree settings. |
| `kanban` | → `KanbanConfig` | No | Kanban provider configuration for syncing issues from Jira, Linear, etc. |
| `version_check` | → `VersionCheckConfig` | No | Version check configuration for automatic update notifications |
| `delegators` | `array` | No | Agent delegator configurations for autonomous ticket launching |
| `model_servers` | `array` | No | User-declared model servers (ollama, lmstudio, any OpenAI-compat host). Implicit builtin servers exist for each `llm_tool`'s vendor API and do not need declaration. |
| `hosts` | `array` | No | Remote machines agents can be launched on over SSH, referenced by name from `DelegatorLaunchConfig.host`. |
| `targets` | `array` | No | Named execution targets (docker/coder/ssh/local) referenced by `DelegatorLaunchConfig.target`. |
| `governance` | `array` | No | Named launch policies referenced by `Delegator.governance`. |
| `relay` | → `RelayConfig` | No | Relay MCP injection configuration |
| `mcp` | → `McpConfig` | No | Model Context Protocol (MCP) server configuration |
| `acp` | → `AcpConfig` | No | Agent Client Protocol (ACP) agent configuration |
| `native_llm` | → `NativeLlmConfig` | No | In-daemon LLM calls (judge). Accepted but inert until the `native-llm` build feature ships; a configured judge falls back to the deterministic rule. |

## Type Definitions

### ProfileIdentity

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `id` | `string` | Yes | Stable profile id. Nil until the profile is registered. |
| `name` | `string` | Yes | Profile name. The default primary profile is `legacy`. |

### AgentsConfig

Agent lifecycle, parallelism, and health monitoring.

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `max_parallel` | `integer` | Yes | Upper bound on concurrent agents (default: 5). The effective cap is also limited by CPU cores minus `cores_reserved`. |
| `cores_reserved` | `integer` | Yes | CPU cores kept free when computing the effective agent cap (default: 1). |
| `max_agents_per_repo` | `integer` | No | Maximum concurrent agents per project/repo (default: 1). Requires `git.use_worktrees` = true when > 1 to avoid conflicts. |
| `health_check_interval` | `integer` | Yes | Seconds between agent health checks (default: 30). |
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
| `events` | `array` | No | Events to send (empty = all events) Possible values: agent.started, agent.completed, agent.failed, `agent.awaiting_input`, `agent.session_lost`, pr.created, pr.merged, pr.closed, `pr.ready_to_merge`, `pr.changes_requested`, ticket.returned, investigation.created |

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

Queue ordering and assignment.

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `auto_assign` | `boolean` | Yes | Assign the next ticket when an agent slot frees (default: true). |
| `priority_order` | `array` | Yes | Issue-type keys, highest priority first (default: INV, FIX, TASK, FEAT, SPIKE). |
| `poll_interval_ms` | `integer` | Yes | How often the queue is polled, in milliseconds (default: 1000). |

### PathsConfig

Directories for tickets, projects, state, and worktrees.

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `tickets` | `string` | Yes | Ticket root, relative to the working directory or absolute (default: `.tickets`). |
| `projects` | `string` | Yes | Projects root scanned for agent marker files (default: `.`). |
| `state` | `string` | Yes | Operator state directory (default: `.tickets/operator`). |
| `worktrees` | `string` | No | Base directory for per-ticket worktrees (default: ~/.operator/worktrees) |

### UiConfig

Terminal UI appearance and behavior.

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `refresh_rate_ms` | `integer` | Yes | TUI redraw interval in milliseconds (default: 250). |
| `completed_history_hours` | `integer` | Yes | Hours of completed tickets shown on the dashboard (default: 24). |
| `summary_max_length` | `integer` | Yes | Maximum characters shown for a ticket summary (default: 40). |
| `panel_names` | → `PanelNamesConfig` | No | Custom titles for the dashboard panels. |

### PanelNamesConfig

Custom titles for the dashboard panels.

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `status` | `string` | No | Status panel title (default: `STATUS`). |
| `queue` | `string` | No | Queue panel title (default: `TODO QUEUE`). |
| `in_progress` | `string` | No | In-progress panel title (default: `IN PROGRESS`). The key `agents` is accepted as an alias. |
| `completed` | `string` | No | Completed panel title (default: `DONE`). |

### LaunchConfig

Agent launch behavior and confirmations.

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `confirm_autonomous` | `boolean` | Yes | Ask before launching an autonomous ticket (default: true). |
| `confirm_paired` | `boolean` | Yes | Ask before launching a paired ticket (default: true). |
| `launch_delay_ms` | `integer` | Yes | Milliseconds to wait after confirmation before the agent starts (default: 2000). |
| `target` | `string` \| `null` | No | Default named execution target. Per-launch and per-delegator choices take precedence. |
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

Issue type collections and presets.

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `preset` | → `CollectionPreset` | No | Named preset for issue type collection Options: simple, `dev_kanban`, `devops_kanban`, custom |
| `collection` | `array` | No | Custom issuetype collection (only used when preset = custom) List of issue type keys: TASK, FEAT, FIX, SPIKE, INV |
| `active_collection` | `string` \| `null` | No | Active collection name (overrides preset if set) Can be a builtin preset name or a user-defined collection |
| `collections_fetch_enabled` | `boolean` | No | Enable fetching hosted issuetype collections during setup. When disabled, only the embedded (offline) collections are offered. |
| `collections_manifest_url` | `string` \| `null` | No | URL of the hosted collection index manifest, fetched during setup. Points at a `CollectionIndex` JSON document listing available collections. |
| `collections_fetch_timeout_secs` | `integer` | No | Timeout in seconds for hosted collection fetch HTTP requests. |

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
| `format` | → `LogFormat` | No | Line format for console and file logs (default: text).  `json` emits one JSON object per line with `trace_id` and `span_id`. Override with `OPERATOR_LOGGING__FORMAT=json`. |

### LogFormat

Console and file log line format.

**Allowed Values:**

- `text` - Human-readable lines.
- `json` - One JSON object per line, with `trace_id`/`span_id` when a trace is active.

### TelemetryConfig

OTLP telemetry: ingest from agents, a bounded per-session window, and export upstream.

Off by default. Operator never configures a destination of its own.

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `enabled` | `boolean` | No | Accept OTLP from agent sessions on `/otlp/v1/*` and inject agent OTLP env |
| `enable_redaction` | `boolean` | No | Hash personal data and drop content and unapproved signal names, while secrets are protected in both modes. |
| `ingest` | → `TelemetryIngestConfig` | No | Inbound OTLP limits and agent environment injection. |
| `export` | → `TelemetryExportConfig` | No | Upstream OTLP export. |
| `window` | → `TelemetryWindowConfig` | No | Bounded in-memory session window. |

### TelemetryIngestConfig

Limits incoming OTLP requests and controls agent environment injection.

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `max_body_bytes` | `integer` | No | Maximum request body size, enforced both compressed and decompressed |
| `queue_capacity` | `integer` | No | Bounded ingest queue; a full queue answers 503 with Retry-After |
| `inject_agent_env` | `boolean` | No | Export OTLP endpoint, token and trace context into launched agent sessions |

### TelemetryExportConfig

Describes the user-configured upstream OTLP destination.

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `endpoint` | `string` \| `null` | No | Upstream OTLP base URL; falls back to the standard `OTEL_EXPORTER_OTLP_*` variables |
| `protocol` | object | No | OTLP encoding. Default is `http/protobuf`. |
| `headers_env` | `string` \| `null` | No | Name of an environment variable holding OTLP headers (`key=value,...`) |
| `compression` | object | No | Request compression. Default is `gzip`. |
| `timeout_secs` | `integer` \| `null` | No | Upstream export timeout in seconds (default: 10). |
| `self_telemetry` | `boolean` | No | Export Operator's own traces, logs and metrics alongside forwarded agent telemetry |

### OtlpProtocol

Selects the OTLP HTTP payload encoding.

**Allowed Values:**

- `http/protobuf` - OTLP/HTTP protobuf (`http/protobuf`).
- `http/json` - OTLP/HTTP JSON (`http/json`).

### OtlpCompression

Selects compression for outgoing OTLP requests.

**Allowed Values:**

- `gzip` - gzip request compression.
- `none` - Send the OTLP body uncompressed.

### TelemetryWindowConfig

Bounds session retention in the in-memory telemetry window.

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `max_sessions` | `integer` | No | Sessions retained; the least recently updated is evicted first |
| `records_per_session` | `integer` | No | Records retained per session; the oldest are evicted first |
| `grace_secs` | `integer` | No | Seconds an ended session stays readable before eviction |

### TmuxConfig

Tmux integration configuration.

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `config_generated` | `boolean` | No | Whether custom tmux config has been generated |

### SessionsConfig

Session wrapper configuration

Controls how operator creates and manages terminal sessions for agents.
Four modes are supported:
- tmux: Standalone tmux sessions (default)
- vscode: VS Code integrated terminal (requires extension)
- cmux: macOS terminal multiplexer (requires running inside cmux)
- zellij: Zellij terminal workspace manager

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `wrapper` | → `SessionWrapperType` | No | Which session wrapper to use |
| `tmux` | → `SessionsTmuxConfig` | No | Tmux-specific configuration |
| `vscode` | → `SessionsVSCodeConfig` | No | VS Code-specific configuration |
| `cmux` | → `SessionsCmuxConfig` | No | cmux-specific configuration |
| `zellij` | → `SessionsZellijConfig` | No | Zellij-specific configuration |
| `connect` | → `SessionsConnectConfig` | No | How the UI connects you to a launched agent's session |

### SessionWrapperType

Session wrapper type for terminal session management

**Allowed Values:**

- `tmux` - Standalone tmux sessions (default)
- `vscode` - VS Code integrated terminal (via extension webhook)
- `cmux` - cmux macOS terminal multiplexer
- `zellij` - Zellij terminal workspace manager

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
| `require_in_cmux` | `boolean` | No | Require running inside cmux (`CMUX_WORKSPACE_ID` env var present) |
| `placement` | → `CmuxPlacementPolicy` | No | Where to place new agent sessions: "auto", "workspace", or "window" |

### CmuxPlacementPolicy

Placement policy for cmux sessions: where to create new agent terminals

**Allowed Values:**

- `auto` - Automatically choose: 0-1 windows → new workspace, >1 windows → new window
- `workspace` - Always create a new workspace in the active window
- `window` - Always create a new window for each ticket

### SessionsZellijConfig

Zellij terminal workspace manager session configuration

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `require_in_zellij` | `boolean` | No | Require running inside Zellij (ZELLIJ env var present) |

### SessionsConnectConfig

Connect preferences; the offered interfaces are derived from what is configured and installed

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `preferred` | object | No | Primary interface for connect links; unset follows the agent's session wrapper |
| `terminal_command` | `string` \| `null` | No | Command that opens a native terminal running `{command}` (e.g. `open -na Ghostty --args -e {command}`); unset disables server-side terminal opening |

### ConnectInterface

Interface used to open a launched agent's session

**Allowed Values:**

- `vscode` - VS Code, via the Operator extension
- `cursor` - Cursor, via the Operator extension
- `zed` - Zed (opens the folder only)
- `coder` - Coder's VS Code extension
- `cmux` - cmux workspace focus
- `tmux` - tmux attach
- `zellij` - zellij attach
- `ssh` - Plain SSH into the remote session
- `terminal` - Native terminal running the attach or resume command

### LlmToolsConfig

LLM CLI tools configuration

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `detected` | `array` | No | Detected CLI tools (populated on first startup) |
| `providers` | `array` | No | Available {tool, model} pairs for launching tickets Built from detected tools + their model aliases |
| `detection_complete` | `boolean` | No | Whether detection has been completed |
| `default_tool` | `string` \| `null` | No | User's preferred default LLM tool (e.g., "claude") |
| `default_model` | `string` \| `null` | No | User's preferred default model alias (e.g., "opus") |
| `skill_directory_overrides` | `object` | No | Per-tool overrides for skill directories (keyed by `tool_name`) |

### DetectedTool

A detected CLI tool (e.g., claude binary)

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `name` | `string` | Yes | Tool name (e.g., "claude") |
| `path` | `string` | Yes | Path to the binary |
| `version` | `string` | Yes | Version string |
| `min_version` | `string` \| `null` | No | Minimum required version for Operator compatibility |
| `version_ok` | `boolean` | No | Whether the installed version meets the minimum requirement |
| `model_aliases` | `array` | No | Available model aliases (e.g., ["opus", "sonnet", "haiku"]) |
| `command_template` | `string` | No | Command template with {{model}}, {{`session_id`}}, {{`prompt_file`}} placeholders |
| `capabilities` | → `ToolCapabilities` | No | Tool capabilities |
| `yolo_flags` | `array` | No | CLI flags for YOLO (auto-accept) mode |
| `health_ok` | `boolean` | No | Whether the tool passed its health check at detection on startup |

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

### RestApiConfig

REST API server configuration

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `enabled` | `boolean` | No | Whether the REST API is enabled |
| `host` | `string` | No | Address the REST API binds to. Defaults to `127.0.0.1` (local only) so the server is not reachable from other hosts. Set to `0.0.0.0` to expose it on all interfaces. |
| `port` | `integer` | No | Port for the REST API server |
| `cors_origins` | `array` | No | CORS allowed origins. Empty means **same-origin only** |
| `public_url` | `string` \| `null` | No | Externally reachable base URL (e.g. `https://operator.example.com`). Defaults to request host. |
| `shutdown_drain_seconds` | `integer` | No | Maximum time to wait for active agents before shutdown cleanup begins. |
| `shutdown_cleanup_seconds` | `integer` | No | Maximum time reserved for final callbacks and persistent cleanup. |

### GitConfig

Git provider configuration for PR/MR operations

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `identity` | object | No | Default commit identity for delegated work. |
| `gitea` | → `GiteaConfig` | No | Gitea forge settings for pull requests. |
| `forgejo` | → `ForgejoConfig` | No | Forgejo forge settings for pull requests. |
| `provider` | object | No | Active provider (auto-detected from remote URL if not specified) |
| `github` | → `GitHubConfig` | No | GitHub-specific configuration |
| `gitlab` | → `GitLabConfig` | No | GitLab-specific configuration |
| `branch_format` | `string` | No | Branch naming format (e.g., "{type}/{ticket_id}-{slug}") |
| `use_worktrees` | `boolean` | No | Whether to use git worktrees for per-ticket isolation (default: false) When false, tickets work directly in the project directory with branches |

### GitIdentityConfig

Commit identity template for delegated work.

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `name` | `string` | Yes | Author and committer name; may use `{ticket_id}`, `{project}` and `{ticket_type}` |
| `email` | `string` | Yes | Author and committer email; may use `{ticket_id}`, `{project}` and `{ticket_type}` |

### GiteaConfig

Gitea forge settings for pull requests.

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `enabled` | `boolean` | No | Whether Gitea integration is enabled (default: false). |
| `token_env` | `string` | No | Environment variable containing the Gitea token (default: `GITEA_TOKEN`). |
| `host` | `string` \| `null` | No | HTTPS host or base URL; defaults to gitea.com. |
| `wip_prefix` | `string` | No | Prefix applied to draft pull request titles (default: `WIP: `). |

### ForgejoConfig

Forgejo forge settings for pull requests.

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `enabled` | `boolean` | No | Whether Forgejo integration is enabled (default: false). |
| `token_env` | `string` | No | Environment variable containing the Forgejo token (default: `FORGEJO_TOKEN`). |
| `host` | `string` \| `null` | No | HTTPS host or base URL; defaults to codeberg.org. |
| `wip_prefix` | `string` | No | Prefix applied to draft pull request titles (default: `WIP: `). |

### GitProviderConfig

Git provider selection

**Allowed Values:**

- `github` - GitHub (github.com)
- `gitlab` - GitLab (gitlab.com or self-hosted)
- `bitbucket` - Bitbucket (bitbucket.org)
- `azuredevops` - Azure DevOps (dev.azure.com)
- `forgejo` - Forgejo (e.g. codeberg.org or self-hosted)
- `gitea` - Gitea (gitea.com or self-hosted)

### GitHubConfig

GitHub-specific configuration

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `enabled` | `boolean` | No | Whether GitHub integration is enabled |
| `token_env` | `string` | No | Environment variable containing the GitHub token (default: `GITHUB_TOKEN`) |

### GitLabConfig

GitLab-specific configuration

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `enabled` | `boolean` | No | Whether GitLab integration is enabled |
| `token_env` | `string` | No | Environment variable containing the GitLab token (default: `GITLAB_TOKEN`) |
| `host` | `string` \| `null` | No | GitLab host (default: gitlab.com, can be self-hosted) |

### KanbanConfig

Kanban provider configuration for syncing issues from external systems
Providers are keyed by domain/workspace:

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `gitea` | `object` | No | Gitea repository project boards, keyed by instance name. Import-only. |
| `forgejo` | `object` | No | Reserved Forgejo project boards; enabling requires an upstream board API. |
| `jira` | `object` | No | Jira Cloud instances keyed by domain (e.g., "foobar.atlassian.net") |
| `linear` | `object` | No | Linear instances keyed by workspace slug |
| `github` | `object` | No | GitHub Projects v2 instances keyed by owner login (user or org)  NOTE: This is the *kanban* GitHub integration (Projects v2), distinct from `GitHubConfig` which is the *git provider* used for PRs and branches. The two use different env vars and different scopes - see `docs/getting-started/kanban/github.md` for the full disambiguation. |
| `openspec` | `object` | No | `OpenSpec` roots keyed by a free-form instance name (e.g., a repo alias). Experimental, pull-only: each active change under `<root_path>/changes/` acts as a kanban "project" whose issues are the tasks.md task groups. |

### GiteaProjectsConfig

Repository project board configuration; credentials are environment variable names.

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `enabled` | `boolean` | No | Whether this instance is synced (default: false). |
| `base_url` | `string` | Yes | Instance base URL, including any deployment path prefix. |
| `api_key_env` | `string` | No | Environment variable containing the API token (default: `OPERATOR_GITEA_TOKEN`). |
| `repositories` | `array` | No | Repositories to discover, in owner/name form. |
| `projects` | `object` | No | Keys are absolute project URLs returned by discovery. |

### ProjectSyncConfig

Per-project/team sync configuration for a kanban provider

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `sync_user_id` | `string` | No | User ID to sync issues for (provider-specific format) - Jira: accountId (e.g., "5e3f7acd9876543210abcdef") - Linear: user ID (e.g., "abc12345-6789-0abc-def0-123456789abc") - GitHub Projects: numeric GitHub `databaseId` (e.g., "12345678") |
| `status_mapping` | → `KanbanStatusMapping` | No | Mapping of operator todo/doing/done to external board columns |
| `collection_name` | `string` \| `null` | No | Optional `IssueTypeCollection` name this project maps to. Not required for kanban onboarding or sync. |
| `type_mappings` | `object` | No | Explicit mapping: kanban issue type ID → operator issue type key (e.g., TASK, FEAT, FIX). Multiple kanban types can map to the same operator template. |
| `bidirectional` | `boolean` | No | When true, operator pushes status changes and activity logs back to this kanban project. Ticket state changes (todo→doing, doing→done) and step completions with delegator info are reflected upstream. Default: false. |
| `ticket_project` | `string` \| `null` | No | Operator project name stamped on tickets created from this source. Defaults to the external project key when unset. |
| `scope` | object | No | Sync one upstream container into a campaign instead of the whole project (Premium). Scoped syncs pull every member regardless of assignee. |

### KanbanStatusMapping

Explicit mapping from operator's strict todo/doing/done states to the
external board's column/status names.

Drives bidirectional sync: issues are pulled from the `todo` column,
pushed to `doing` when a ticket is claimed, to `done` when completed, and
back to `todo` when requeued. Unset fields fall back per-transition
(`doing` → "In Progress", `done` → "Done"); requeue only pushes when
`todo` is explicitly mapped.

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `todo` | `string` \| `null` | No | External column for operator "todo" (queued work; also the pull source) |
| `doing` | `string` \| `null` | No | External column for operator "doing" (claimed/launched tickets) |
| `done` | `string` \| `null` | No | External column for operator "done" (completed tickets) |

### SyncScope

An upstream container synced as one campaign.

**Allowed Values:**


### ForgejoProjectsConfig

Repository project board configuration; credentials are environment variable names.

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `enabled` | `boolean` | No | Whether this instance is synced (default: false). |
| `base_url` | `string` | Yes | Instance base URL, including any deployment path prefix. |
| `api_key_env` | `string` | No | Environment variable containing the API token (default: `OPERATOR_FORGEJO_TOKEN`). |
| `repositories` | `array` | No | Repositories to discover, in owner/name form. |
| `projects` | `object` | No | Keys are absolute project URLs returned by discovery. |

### JiraConfig

Jira Cloud provider configuration

The domain is specified as the `HashMap` key in KanbanConfig.jira

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `enabled` | `boolean` | No | Whether this provider is enabled |
| `api_key_env` | `string` | No | Environment variable name containing the API key (default: `OPERATOR_JIRA_API_KEY`) |
| `email` | `string` | No | Atlassian account email for authentication |
| `projects` | `object` | No | Per-project sync configuration |

### LinearConfig

Linear provider configuration

The workspace slug is specified as the `HashMap` key in KanbanConfig.linear

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `enabled` | `boolean` | No | Whether this provider is enabled |
| `api_key_env` | `string` | No | Environment variable name containing the API key (default: `OPERATOR_LINEAR_API_KEY`) |
| `projects` | `object` | No | Per-team sync configuration |

### GithubProjectsConfig

GitHub Projects v2 (kanban) provider configuration

The owner login (user or org) is specified as the `HashMap` key in
`KanbanConfig.github`. Project keys inside `projects` are `GraphQL` node
IDs (e.g., `PVT_kwDOABcdefg`) - opaque, stable identifiers used directly
by every GitHub Projects v2 mutation without needing a lookup.

**Distinct from `GitHubConfig`** (the git provider used for PR/branch
operations). They live in different parts of the config tree, use
different env vars (`OPERATOR_GITHUB_TOKEN` vs `GITHUB_TOKEN`), and
require different OAuth scopes (`project` vs `repo`). See
`docs/getting-started/kanban/github.md` for the full rationale.

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `enabled` | `boolean` | No | Whether this provider is enabled |
| `api_key_env` | `string` | No | Environment variable name containing the GitHub token (default: `OPERATOR_GITHUB_TOKEN`). The token must have `project` (or `read:project`) scope, NOT just `repo` - see the disambiguation guide in the kanban github docs. |
| `projects` | `object` | No | Per-project sync configuration. Keys are `GraphQL` project node IDs. |

### OpenspecConfig

`OpenSpec` provider configuration (experimental, pull-only)

The instance name is the `HashMap` key in `KanbanConfig.openspec`. There
are no credentials - the provider reads local markdown under `root_path`.

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `enabled` | `boolean` | No | Whether this provider is enabled |
| `root_path` | `string` | No | Directory containing the `OpenSpec` `changes/` tree (typically `<repo>/openspec`) |
| `project` | `string` \| `null` | No | Operator project stamped on imported tickets (defaults to the change id) |

### VersionCheckConfig

Version check configuration for automatic update notifications

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `enabled` | `boolean` | No | Enable automatic version checking on startup |
| `url` | `string` \| `null` | No | URL to fetch latest version from (optional, can be removed) |
| `timeout_secs` | `integer` | No | Timeout in seconds for version check HTTP request |

### Delegator

Agent delegator configuration for autonomous ticket launching

A delegator is a named {tool, model} pairing with optional launch configuration.

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `git` | object | No | Optional Git identity, HTTPS credential reference, and runtime settings. |
| `name` | `string` | Yes | Unique name for this delegator (e.g., "claude-opus-auto") |
| `llm_tool` | `string` | Yes | LLM tool name (must match a detected tool, e.g., "claude", "codex") |
| `model` | `string` | Yes | Model alias (e.g., "opus", "sonnet", "gpt-4o") |
| `display_name` | `string` \| `null` | No | Optional display name for UI |
| `model_properties` | `object` | No | Arbitrary model properties (e.g., `reasoning_effort`, sandbox) |
| `launch_config` | object | No | Optional launch configuration |
| `model_server` | `string` \| `null` | No | Name of a declared `ModelServer` (from `Config.model_servers`). `None` means use the `llm_tool`'s implicit vendor default (claude → anthropic-api, codex → openai-api, gemini → google-api). |
| `governance` | `string` \| `null` | No | Name of a `[[governance]]` policy enforced on every launch by this delegator. |
| `remote_agent` | object | No | Declarative reference to a remote, named agent on another platform (e.g. an AGNT agent or an `OpenAI` Assistant; see [`crate::config::AgentProfile`]).  Export-only: Operator has no runtime client for those platforms, so a delegator carrying this CANNOT be launched locally - resolution errors out (see `delegator_resolution`). It is stored, listed, serialized into an `AgentProfile`, and - for `platform == "agnt"` - surfaced in the `--format agnt` workflow export as a native AGNT `agnt-agent` node, whose `agentId` is this reference's `id` (AGNT identifies agents by UUID, so the `id` must be the agent's UUID, not its display name). `None` = ordinary, locally launchable delegator. |
| `x_agnt` | object | No | Opaque AGNT-namespaced extension fields, preserved verbatim across an `AgentProfile` round-trip so re-export is lossless (e.g. `memory`, `assignedWorkflows`, `creditLimit`). Operator never interprets this. |
| `x_openai` | object | No | Opaque OpenAI-namespaced extension fields, preserved verbatim across an `AgentProfile` round-trip (e.g. `instructions`, `tools`, `tool_resources`, `metadata`, thread refs). Mirror of [`Self::x_agnt`]; never interpreted. |
| `unmapped_core` | object | No | Opaque carry for `AgentProfile` shared-core fields Operator cannot model first-class (`system_prompt` / `skills` / `mcp_servers` / `tools`) so an import→export round-trip is lossless. Distinct from `x_agnt`: these are shared-core fields, not AGNT-specific, so folding them into `x_agnt` would corrupt that namespace. Operator never interprets this. |

### GitExecutionConfig

Git settings owned by a named delegator.

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `identity` | object | No | Commit identity; falls back to the global `git.identity` |
| `credentials` | object | No | HTTPS credential for the ticket's repository |
| `settings` | `array` | No | Extra `git config` entries; `credential.*`, `url.*`, `http.*`, `include*`, `user.name`, `user.email`, `core.askpass` and `core.sshCommand` are rejected |

### GitCredentialConfig

Supplied HTTPS credential, bound to a repository; contains no secret value.

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `repository_url` | `string` | Yes | HTTPS repository URL the credential is bound to, with no embedded credentials, query or fragment |
| `username` | `string` | Yes | HTTPS username sent with the token |
| `token_env` | `string` | Yes | Environment variable holding the token, read at launch |

### GitConfigEntry

One `git config` entry applied to delegated work.

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `key` | `string` | Yes | Dotted config key (e.g. "commit.gpgsign"); identity and authentication keys are rejected |
| `value` | `string` | Yes | Config value |

### DelegatorLaunchConfig

Launch configuration for a delegator

Controls how the delegator launches agents. Optional fields use tri-state
semantics: `None` = inherit from global config, `Some(true/false)` = override.

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `yolo` | `boolean` | No | Run in YOLO (auto-accept) mode |
| `permission_mode` | `string` \| `null` | No | Permission mode override |
| `flags` | `array` | No | Additional CLI flags |
| `use_worktrees` | `boolean` \| `null` | No | Override global `git.use_worktrees` per-delegator (None = use global setting) |
| `create_branch` | `boolean` \| `null` | No | Whether to create a git branch for the ticket (None = default behavior) |
| `docker` | `boolean` \| `null` | No | DEPRECATED: prefer `target`. Run in docker container (None = fall back to `launch.docker.enabled`, then local). |
| `prompt_prefix` | `string` \| `null` | No | Prompt text to prepend before the generated step prompt |
| `prompt_suffix` | `string` \| `null` | No | Prompt text to append after the generated step prompt |
| `operator_relay` | `boolean` \| `null` | No | Override global relay auto-inject MCP setting per-delegator (None = use global setting) |
| `host` | `string` \| `null` | No | DEPRECATED: prefer `target`. Name of a declared `RemoteHost` (from `Config.hosts`) to launch the agent CLI on over SSH. `None` = local. |
| `target` | `string` \| `null` | No | Name of an execution target: an explicit `[[targets]]` entry, the synthesized `local`/`docker` targets, or a `[[hosts]]` name. Supersedes `docker` and `host`. |

### RemoteAgentRef

A declarative reference to a remote, named agent hosted by another platform.

`platform` is the hosting service (`"agnt"`, `"openai"`) - deliberately
distinct from the core `provider`/`llm_tool` (the model or coding CLI). These
agents are API/memory-native and live on the remote side; Operator has no
runtime client for them, so a delegator carrying one is **export-only** and
cannot be launched locally (see the guard in `delegator_resolution`).

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `platform` | `string` | Yes | Hosting platform (e.g. `"agnt"`, `"openai"`). |
| `id` | `string` | Yes | Platform-native agent identifier (e.g. an AGNT agent UUID, an `OpenAI` `asst_…` id). |

### ModelServer

A named host that serves models via an inference API.

Model servers are orthogonal to `llm_tools`: a delegator pairs an agentic CLI
(`llm_tool`, e.g. claude/codex/gemini) with a model-serving endpoint
(`model_server`, e.g. ollama-local, openai-api, a custom vllm host).

Implicit builtin servers are returned by [`implicit_model_server_for_tool`] for shipped tools.

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `name` | `string` | Yes | Unique name (e.g., "ollama-local", "vllm-gpu1") |
| `kind` | `string` | Yes | Kind: "ollama", "openrouter", "openai-compat", "anthropic-api", "openai-api", "google-api", "lmstudio" |
| `base_url` | `string` \| `null` | No | Base URL of the inference endpoint (e.g., `http://localhost:11434`). `None` for implicit vendor servers means use the SDK default. |
| `api_key_env` | `string` \| `null` | No | Name of an env var providing the API key (e.g., `OLLAMA_API_KEY`) |
| `extra_env` | `object` | No | Additional environment variables set when spawning agents that use this server |
| `display_name` | `string` \| `null` | No | Optional display name for UI |

### RemoteHost

A named remote machine that agent CLI processes can be launched on over SSH.

Distinct from [`ModelServer`] (where model *inference* lives) and from
[`RemoteAgentRef`] (an export-only agent owned by another platform): a
`RemoteHost` is where the agent *CLI process* runs. Referenced by name from
[`DelegatorLaunchConfig::host`].

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `name` | `string` | Yes | Unique name referenced by `DelegatorLaunchConfig.host` (e.g., "gpu-vm") |
| `ssh_alias` | `string` | Yes | SSH destination, resolved via the user's `~/.ssh/config` |
| `workdir` | `string` | Yes | Absolute path to the project root on the remote host |
| `display_name` | `string` \| `null` | No | Optional display name for UI |
| `ssh_config_path` | `string` \| `null` | No | SSH config fragment passed with `-F` (used by provisioned coder aliases) |

### TargetDef

A named execution target agents can be launched on.

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `name` | `string` | Yes | Unique name, referenced by `DelegatorLaunchConfig.target`. `local` and `docker` are reserved for synthesized targets. |
| `display_name` | `string` \| `null` | No | Human-readable name for UI surfaces |

**Allowed Values:**


### CoderConfig

Coder workspace target: lifecycle + alias provisioning around the shared
SSH remote-launch path. There is no `enabled` field - presence in
`[[targets]]` is the enablement.

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `template` | `string` | Yes | Coder template child workspaces are created from (an allowlist - never per-ticket input) |
| `url_env` | `string` | No | Env var NAME holding the Coder deployment URL |
| `token_env` | `string` | No | Env var NAME holding the Coder session token. The variable is stripped from every agent's spawn environment on all target kinds. |
| `name_prefix` | `string` | No | Workspace name prefix for deterministic per-ticket naming |
| `workdir` | `string` \| `null` | No | Project root inside the workspace (None = /home/coder/{project}) |
| `stop_on_complete` | `boolean` | No | Stop the workspace when the ticket completes (never delete) |
| `create_timeout_secs` | `integer` | No | Bound on workspace create + agent-ready wait |
| `callback_url` | `string` \| `null` | No | Control-plane-reachable `OPERATOR_API_URL` override for detached multi-step (empty/None = reverse tunnel default) |
| `parameters` | `object` | No | Passthrough `--parameter` template parameters for `coder create`. Values may use `{{ticket_id}}`, `{{project}}`, `{{step}}` and `{{delegator}}` |
| `preset` | `string` \| `null` | No | Template preset passed to `coder create --preset` |
| `remote_callback_port` | `integer` \| `null` | No | Workspace-side port the reverse tunnel binds (None = the local API port) |
| `forward_env` | `array` | No | Env var NAMES resolved on the control plane and shipped to the agent (e.g. LLM provider credentials); a missing one fails the launch |
| `delete_on_complete` | `boolean` | No | Delete the workspace when the ticket completes (wins over `stop_on_complete`) |
| `delete_on_merge` | `boolean` | No | Delete the workspace when the ticket's PR merges |

### SshTarget

SSH target payload. Name and display name live on `TargetDef`; this is the
connection shape (`RemoteHost` minus identity).

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `ssh_alias` | `string` | Yes | Host alias resolved via the user's `~/.ssh/config` (or `ssh_config_path`) |
| `workdir` | `string` | Yes | Absolute project root on the remote machine |
| `ssh_config_path` | `string` \| `null` | No | SSH config fragment passed with `-F` (used by provisioned coder aliases) |
| `remote_callback_port` | `integer` \| `null` | No | Remote port the reverse tunnel binds for callbacks (None = the local API port) |

### GovernancePolicy

A named launch policy. Every constraint is optional; an absent one is unconstrained.

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `name` | `string` | Yes | Unique name, referenced by `Delegator.governance`. |
| `allowed_delegators` | `array` \| `null` | No | Delegator names allowed to launch. Absent = any. |
| `allowed_llm_tools` | `array` \| `null` | No | LLM tool names allowed to launch (e.g. `claude`). Absent = any. |
| `allowed_model_servers` | `array` \| `null` | No | Model server names allowed (declared or implicit, e.g. `anthropic-api`). Absent = any. |
| `allowed_targets` | `array` \| `null` | No | Execution target names allowed (e.g. `local`, `docker`, a `[[targets]]` name). Absent = any. |
| `max_parallel` | `integer` \| `null` | No | Maximum concurrently running agents under this policy. |
| `max_launches` | `integer` \| `null` | No | Maximum total launches under this policy (counted per campaign run). |
| `max_retries` | `integer` \| `null` | No | Maximum relaunches of a failed ticket (counted per campaign run). |
| `max_runtime_minutes` | `integer` \| `null` | No | Maximum wall-clock minutes per agent. |
| `budget_usd` | `number` \| `null` | No | Spend ceiling in USD. Reserved: accepted and reported, not yet enforced. |
| `require_confirmation` | `boolean` | No | Require operator confirmation before each launch. |

### RelayConfig

Relay MCP injection configuration

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `auto_inject_mcp` | `boolean` | No | When true, automatically inject the relay MCP server for all delegators. When false (default), relay injection is opt-in per delegator. |

### McpConfig

Model Context Protocol (MCP) server configuration

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `http_enabled` | `boolean` | No | Whether to mount MCP HTTP/SSE endpoints on the REST API server. Toggling requires an API restart (no hot-swap of the axum router). |
| `stdio_advertised` | `boolean` | No | Whether the descriptor endpoint advertises the `operator mcp` stdio command. Set to false on multi-tenant/remote deployments where clients shouldn't spawn local subprocesses. |
| `expose_ticket_write_tools` | `boolean` | No | Whether to expose ticket-mutating tools (claim, complete, return-to-queue, create) over MCP. Defaults to `false` because any MCP client can call them. |
| `external_servers` | `array` | No | External MCP servers to inject into spawned agent sessions. Each entry produces a separate `--mcp-config` file alongside the relay config when launching Claude Code agents. |

### ExternalMcpServer

An external MCP server to inject into spawned agent sessions.

Values in `command`, `args`, and `env` support `${VAR}` interpolation,
expanded at spawn time from the operator process environment.

When `discover_from` is set, operator reads an MCP server spec from that
JSON sidecar file at spawn time. The sidecar must contain a top-level
`mcpServer` object with `command`, `args`, and `env` fields. If the file
is absent and `command` is empty, the server is silently skipped.

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `name` | `string` | Yes | Server name used as the key in the `mcpServers` JSON object (e.g., "kanbots"). Must be unique across all external servers. |
| `command` | `string` | No | Command to execute. Supports `${VAR}` interpolation. |
| `args` | `array` | No | Command arguments. Each element supports `${VAR}` interpolation. |
| `env` | `object` | No | Environment variables passed to the MCP server process. Values support `${VAR}` interpolation. |
| `enabled` | `boolean` | No | Whether this server is enabled. Allows disabling without removing config. |
| `discover_from` | `string` \| `null` | No | Path to a JSON sidecar discovery file. Relative paths resolve from the project directory. The sidecar must contain `{ "mcpServer": { ... } }`. When the file exists, its `mcpServer` spec is used verbatim (overriding `command`/`args`/`env`). When absent and `command` is empty, the server is silently skipped. |

### AcpConfig

Agent Client Protocol (ACP) agent configuration.

Operator runs as an ACP agent over stdio when editors (Zed, `JetBrains`,
Emacs `agent-shell`, Kiro, etc.) spawn `operator acp`. Each ACP session
maps to an in-progress ACP ticket and a delegator subprocess.

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `stdio_advertised` | `boolean` | No | Whether the dashboard advertises the `operator acp` stdio entrypoint (and editor-config snippet actions). Set to false on machines that shouldn't be used as ACP agents. |
| `default_delegator` | `string` \| `null` | No | Name of the delegator (from `[[delegators]]`) to use for ACP prompts. If unset or not found, falls back to the operator's default delegator resolution. |
| `max_concurrent_sessions` | `integer` | No | Maximum number of concurrent ACP sessions. New `session/new` requests beyond this limit are rejected with a JSON-RPC error. |

### NativeLlmConfig

In-daemon LLM calls (built with the `native-llm` feature). Distinct from
delegators: these are single typed API calls, not agent CLI sessions.

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `judge` | object | No | Model that picks the winner of `multi_model` (`voting_mode = single_judge`) and `multi_prompt` (`selection_strategy = model_choice`) steps. Unset keeps the deterministic first/longest rule. |

### JudgeConfig

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `model_server` | `string` | Yes | Name of a declared or implicit model server (e.g. "anthropic-api") |
| `model` | `string` | Yes | Full API model id (e.g. "claude-sonnet-5"), not a CLI alias like "sonnet" |
| `timeout_secs` | `integer` | No | Seconds before the judge is abandoned and the deterministic rule applies |


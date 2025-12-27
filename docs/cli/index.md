---
title: "CLI Reference"
layout: doc
---

<!-- AUTO-GENERATED FROM src/main.rs, src/env_vars.rs - DO NOT EDIT MANUALLY -->
<!-- Regenerate with: cargo run -- docs -->

# CLI Reference

Operator provides both a TUI dashboard and CLI commands for queue management.

## Global Options

| Option | Description |
| --- | --- |
| `-c, --config` | Config file path |
| `-d, --debug` | Enable debug logging |

## Commands

When run without a command, Operator launches the interactive TUI dashboard.

### `queue`

Show queue status

| Argument/Option | Description |
| --- | --- |
| `-a, --all` | Show all tickets, not just summary |

### `launch`

Launch agent for next available ticket

| Argument/Option | Description |
| --- | --- |
| `<TICKET>` | Specific ticket to launch (optional) |
| `-y, --yes` | Skip confirmation prompt |

### `agents`

List active agents

| Argument/Option | Description |
| --- | --- |
| `-v, --verbose` | Show detailed agent info |

### `pause`

Pause queue processing

No additional arguments.

### `resume`

Resume queue processing

No additional arguments.

### `stalled`

Show stalled agents awaiting input

No additional arguments.

### `alert`

Create investigation from external alert

| Argument/Option | Description |
| --- | --- |
| `--source` | Alert source (e.g., pagerduty, datadog) |
| `--message` | Alert message |
| `--severity` | Severity (S0, S1, S2) (default: S1) |
| `--project` | Affected project (optional) |

### `create`

Create a new ticket from template

| Argument/Option | Description |
| --- | --- |
| `-t, --template` | Template type (feature, fix, spike, investigation) |
| `-p, --project` | Target project |

### `docs`

Generate documentation from source-of-truth files

| Argument/Option | Description |
| --- | --- |
| `-o, --output` | Output directory (default: docs/) |
| `-g, --only` | Only generate specific docs (taxonomy, issuetype, metadata) |

### `api`

Start the REST API server for issue type management

| Argument/Option | Description |
| --- | --- |
| `-p, --port` | Port to listen on (default: 7008) |

### `setup`

Initialize operator workspace (non-interactive by default)

| Argument/Option | Description |
| --- | --- |
| `-i, --interactive` | Launch TUI setup wizard instead of non-interactive setup |
| `-C, --collection` | Collection preset: simple, dev-kanban, devops-kanban (default: simple) |
| `--backstage` | Enable backstage configuration |
| `-f, --force` | Overwrite existing files |

## Environment Variables

All configuration can be overridden via environment variables using the `OPERATOR_` prefix with `__` as the separator for nested config paths.

### Quick Reference

| Variable | Description | Default |
| --- | --- | --- |
| `OPERATOR_API__ANTHROPIC_API_KEY` | Anthropic API key for rate limit monitoring and AI provider status | - |
| `OPERATOR_API__GITHUB_TOKEN` | GitHub personal access token for PR/issue tracking integration | - |
| `OPERATOR_AGENTS__MAX_PARALLEL` | Maximum number of agents that can run in parallel | 4 |
| `OPERATOR_AGENTS__CORES_RESERVED` | Number of CPU cores to reserve (not used by agents) | 2 |
| `OPERATOR_AGENTS__STALE_MINUTES` | Minutes of inactivity before an agent is considered stale | 30 |
| `OPERATOR_AGENTS__HEALTH_CHECK_INTERVAL_SECS` | Interval in seconds between agent health checks | 60 |
| `OPERATOR_AGENTS__COMPLETION_DETECTION_INTERVAL_SECS` | Interval in seconds between completion detection checks | 5 |
| `OPERATOR_AGENTS__SESSION_DIR` | Directory for storing agent session data | .claude/sessions |
| `OPERATOR_AGENTS__ENABLE_NOTIFICATIONS` | Enable macOS notifications for agent events | true |
| `OPERATOR_QUEUE__AUTO_ASSIGN` | Automatically assign tickets to available agents | true |
| `OPERATOR_QUEUE__POLL_INTERVAL_SECS` | Interval in seconds between queue polling cycles | 5 |
| `OPERATOR_QUEUE__PRIORITY_ORDER` | Comma-separated list of ticket types in priority order | INV,FIX,FEAT,SPIKE |
| `OPERATOR_NOTIFICATIONS__ENABLED` | Enable the notification system | true |
| `OPERATOR_NOTIFICATIONS__ON_LAUNCH` | Send notification when an agent launches | true |
| `OPERATOR_NOTIFICATIONS__ON_COMPLETE` | Send notification when an agent completes | true |
| `OPERATOR_NOTIFICATIONS__ON_STALL` | Send notification when an agent stalls | true |
| `OPERATOR_NOTIFICATIONS__ON_ERROR` | Send notification on agent errors | true |
| `OPERATOR_NOTIFICATIONS__SOUND` | Play sound with notifications | true |
| `OPERATOR_PATHS__TICKETS` | Directory containing ticket files | .tickets |
| `OPERATOR_PATHS__PROJECTS` | Root directory for project discovery | . |
| `OPERATOR_PATHS__STATE` | Directory for persistent operator state | .tickets/operator |
| `OPERATOR_UI__REFRESH_RATE_MS` | UI refresh rate in milliseconds | 250 |
| `OPERATOR_UI__SUMMARY_MAX_LENGTH` | Maximum length of ticket summaries in the UI | 60 |
| `OPERATOR_LAUNCH__MODE` | Agent launch mode (tmux or direct) | tmux |
| `OPERATOR_LAUNCH__CONFIRM` | Require confirmation before launching agents | true |
| `OPERATOR_TMUX__SESSION_PREFIX` | Prefix for tmux session names | operator |
| `OPERATOR_BACKSTAGE__PORT` | Port for the Backstage web server | 3000 |
| `OPERATOR_BACKSTAGE__AUTO_START` | Automatically start Backstage server with TUI | false |
| `OPERATOR_LLM_TOOLS__ENABLED` | Enable LLM tool allowlist/denylist functionality | true |
| `OPERATOR_LLM_TOOLS__ALLOWED` | Comma-separated list of allowed LLM tools (empty = all allowed) |  |
| `OPERATOR_LLM_TOOLS__DENIED` | Comma-separated list of denied LLM tools |  |
| `OPERATOR_LOGGING__LEVEL` | Log level (trace, debug, info, warn, error) | info |
| `OPERATOR_LOGGING__TO_FILE` | Write logs to file in addition to stderr | true |

### Authentication

| Variable | Description | Default |
| --- | --- | --- |
| `OPERATOR_API__ANTHROPIC_API_KEY` | Anthropic API key for rate limit monitoring and AI provider status | - |
| `OPERATOR_API__GITHUB_TOKEN` | GitHub personal access token for PR/issue tracking integration | - |

### Agents

| Variable | Description | Default |
| --- | --- | --- |
| `OPERATOR_AGENTS__MAX_PARALLEL` | Maximum number of agents that can run in parallel | 4 |
| `OPERATOR_AGENTS__CORES_RESERVED` | Number of CPU cores to reserve (not used by agents) | 2 |
| `OPERATOR_AGENTS__STALE_MINUTES` | Minutes of inactivity before an agent is considered stale | 30 |
| `OPERATOR_AGENTS__HEALTH_CHECK_INTERVAL_SECS` | Interval in seconds between agent health checks | 60 |
| `OPERATOR_AGENTS__COMPLETION_DETECTION_INTERVAL_SECS` | Interval in seconds between completion detection checks | 5 |
| `OPERATOR_AGENTS__SESSION_DIR` | Directory for storing agent session data | .claude/sessions |
| `OPERATOR_AGENTS__ENABLE_NOTIFICATIONS` | Enable macOS notifications for agent events | true |

### Queue

| Variable | Description | Default |
| --- | --- | --- |
| `OPERATOR_QUEUE__AUTO_ASSIGN` | Automatically assign tickets to available agents | true |
| `OPERATOR_QUEUE__POLL_INTERVAL_SECS` | Interval in seconds between queue polling cycles | 5 |
| `OPERATOR_QUEUE__PRIORITY_ORDER` | Comma-separated list of ticket types in priority order | INV,FIX,FEAT,SPIKE |

### Notifications

| Variable | Description | Default |
| --- | --- | --- |
| `OPERATOR_NOTIFICATIONS__ENABLED` | Enable the notification system | true |
| `OPERATOR_NOTIFICATIONS__ON_LAUNCH` | Send notification when an agent launches | true |
| `OPERATOR_NOTIFICATIONS__ON_COMPLETE` | Send notification when an agent completes | true |
| `OPERATOR_NOTIFICATIONS__ON_STALL` | Send notification when an agent stalls | true |
| `OPERATOR_NOTIFICATIONS__ON_ERROR` | Send notification on agent errors | true |
| `OPERATOR_NOTIFICATIONS__SOUND` | Play sound with notifications | true |

### Paths

| Variable | Description | Default |
| --- | --- | --- |
| `OPERATOR_PATHS__TICKETS` | Directory containing ticket files | .tickets |
| `OPERATOR_PATHS__PROJECTS` | Root directory for project discovery | . |
| `OPERATOR_PATHS__STATE` | Directory for persistent operator state | .tickets/operator |

### UI

| Variable | Description | Default |
| --- | --- | --- |
| `OPERATOR_UI__REFRESH_RATE_MS` | UI refresh rate in milliseconds | 250 |
| `OPERATOR_UI__SUMMARY_MAX_LENGTH` | Maximum length of ticket summaries in the UI | 60 |

### Launch

| Variable | Description | Default |
| --- | --- | --- |
| `OPERATOR_LAUNCH__MODE` | Agent launch mode (tmux or direct) | tmux |
| `OPERATOR_LAUNCH__CONFIRM` | Require confirmation before launching agents | true |

### Tmux

| Variable | Description | Default |
| --- | --- | --- |
| `OPERATOR_TMUX__SESSION_PREFIX` | Prefix for tmux session names | operator |

### Backstage

| Variable | Description | Default |
| --- | --- | --- |
| `OPERATOR_BACKSTAGE__PORT` | Port for the Backstage web server | 3000 |
| `OPERATOR_BACKSTAGE__AUTO_START` | Automatically start Backstage server with TUI | false |

### LLM Tools

| Variable | Description | Default |
| --- | --- | --- |
| `OPERATOR_LLM_TOOLS__ENABLED` | Enable LLM tool allowlist/denylist functionality | true |
| `OPERATOR_LLM_TOOLS__ALLOWED` | Comma-separated list of allowed LLM tools (empty = all allowed) |  |
| `OPERATOR_LLM_TOOLS__DENIED` | Comma-separated list of denied LLM tools |  |

### Logging

| Variable | Description | Default |
| --- | --- | --- |
| `OPERATOR_LOGGING__LEVEL` | Log level (trace, debug, info, warn, error) | info |
| `OPERATOR_LOGGING__TO_FILE` | Write logs to file in addition to stderr | true |


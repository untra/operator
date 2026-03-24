![Operator! logo](docs/assets/img/operator_logo.svg)

# Operator!
[![GitHub Tag](https://img.shields.io/github/v/tag/untra/operator)](https://github.com/untra/operator/releases) [![codecov](https://codecov.io/gh/untra/operator/branch/main/graph/badge.svg)](https://codecov.io/gh/untra/operator) [![VS Code Marketplace Installs](https://img.shields.io/visual-studio-marketplace/i/untra.operator-terminals?label=VS%20Code%20Installs)](https://marketplace.visualstudio.com/items?itemName=untra.operator-terminals)

**Session** [![tmux](https://img.shields.io/badge/tmux-1BB91F?logo=tmux&logoColor=white)](https://operator.untra.io/getting-started/sessions/tmux/) [![cmux](https://img.shields.io/badge/cmux-333333)](https://operator.untra.io/getting-started/sessions/cmux/) [![Zellij](https://img.shields.io/badge/Zellij-E8590C)](https://operator.untra.io/getting-started/sessions/zellij/) **|** **LLM Tool** [![Claude](https://img.shields.io/badge/Claude-D97757?logo=claude&logoColor=white)](https://operator.untra.io/getting-started/agents/claude/) [![Codex](https://img.shields.io/badge/Codex-000000?logo=openai&logoColor=white)](https://operator.untra.io/getting-started/agents/codex/) [![Gemini CLI](https://img.shields.io/badge/Gemini_CLI-8E75B2?logo=googlegemini&logoColor=white)](https://operator.untra.io/getting-started/agents/gemini-cli/) **|** **Kanban Provider** [![Jira](https://img.shields.io/badge/Jira-0052CC?logo=jira&logoColor=white)](https://operator.untra.io/getting-started/kanban/jira/) [![Linear](https://img.shields.io/badge/Linear-5E6AD2?logo=linear&logoColor=white)](https://operator.untra.io/getting-started/kanban/linear/) **|** **Git Version Control** [![GitHub](https://img.shields.io/badge/GitHub-181717?logo=github&logoColor=white)](https://operator.untra.io/getting-started/git/github/)

An orchestration tool for [**AI-assisted**](https://operator.untra.io/getting-started/agents/) [_kanban-shaped_](https://operator.untra.io/getting-started/kanban/) [git-versioned](https://operator.untra.io/getting-started/git/) software development.

<a href="https://marketplace.visualstudio.com/items?itemName=untra.operator-terminals" target="_blank" class="button">Install <b>Operator! Terminals</b> extension from Visual Studio Code Marketplace</a>

**Operator** is for you if:

- you do work assigned from tickets on a kanban board, such as [_Jira Cloud_](https://operator.untra.io/getting-started/kanban/jira/) or [_Linear_](https://operator.untra.io/getting-started/kanban/linear/)
- you use LLM assisted coding agent tools to accomplish work, such as [_Claude Code_](https://operator.untra.io/getting-started/agents/claude/), [_OpenAI Codex_](https://operator.untra.io/getting-started/agents/codex/), or [_Gemini CLI_](https://operator.untra.io/getting-started/agents/gemini-cli/)
- your work is version controlled with a git repository provider like [_GitHub_](https://operator.untra.io/getting-started/git/github/) or [_GitLab_](https://operator.untra.io/getting-started/git/gitlab/)

- you are drowning in the AI software development soup.

and you are ready to start seriously automating your work.

## Overview

`operator` is a TUI (terminal user interface) application that uses session wrappers ([tmux](https://operator.untra.io/getting-started/sessions/tmux/), [cmux](https://operator.untra.io/getting-started/sessions/cmux/), or [Zellij](https://operator.untra.io/getting-started/sessions/zellij/)) to manage multiple AI coding agents across multi-project workspaces of many codebases. It is designed to be ticket-first, launching LLM coding agents keyed off from markdown stories from a ticketing provider. It provides:

- **Queue Management**: ticket queue with priority-based work assignment, launchable from a dashboard
- **Agent Orchestration**: Launch, monitor, pause/resume LLM coding agents against kanban shaped work tickets, and track the ticket progress as it goes through your defined work implementation steps
- **Notifications**: macOS and linux notifications for agent events, keeping you the human in the loop. 
- **Dashboard**: Real-time view of queue, active agents, completed work, and waiting instances seeking feedback or human review

Operator is designed to facilitate work from markdown tickets, tackling tasks across multiple code repositories by semi-autonomous agents. Operator should be started from the root of your collective work projects repository (eg, `~/Documents`), so that it may start feature or fix work in the right part of the codebase.

When started for the first time, Operator will setup configuration to consume and process work tickets, and identify local projects by scanning for LLM tool marker files (`CLAUDE.md`, `CODEX.md`, `GEMINI.md`) and git repositories.

Operator comes with a separate web component, unneeded to do work but purpose built to give you a developer portal to expand their workflows and administrate Operator with ease.

Operator starts and runs it's own REST API, which can be reached by outside clients, including by the `opr8r` wrapper client. This is included to communicate with Operator api hosts outside of where it's hosted.

## Usage

```bash
# Launch dashboard
operator

# Quick commands (without entering TUI)
operator queue              # Show queue status
operator launch <ticket>    # Launch agent for ticket (with confirmation)
operator create             # Create a new work ticket
operator agents             # List active agents
operator pause              # Pause queue processing
operator resume             # Resume queue processing
operator stalled            # Show stalled agents awaiting input
operator alert              # Create investigation from external alert
operator docs               # Generate documentation from source-of-truth files
operator api                # Start the REST API server
operator setup              # Initialize operator workspace
```

## Installation

Download the latest release for your platform:

```bash
# macOS Apple Silicon
curl -L https://github.com/untra/operator/releases/latest/download/operator-macos-arm64 -o operator
chmod +x operator
sudo mv operator /usr/local/bin/

# Linux x86_64
curl -L https://github.com/untra/operator/releases/latest/download/operator-linux-x86_64 -o operator
chmod +x operator
sudo mv operator /usr/local/bin/

# Linux ARM64
curl -L https://github.com/untra/operator/releases/latest/download/operator-linux-arm64 -o operator
chmod +x operator
sudo mv operator /usr/local/bin/
```

Or build from source:

```bash
git clone https://github.com/untra/operator.git
cd operator
cargo build --release
sudo cp target/release/operator /usr/local/bin/
```

## Configuration

Workspace configuration lives in `.tickets/operator/config.toml` (created by `operator setup`). An optional global override can be placed at `~/.config/operator/config.toml`.

```toml
[agents]
max_parallel = 5          # Maximum concurrent agents
cores_reserved = 1        # Cores to keep free (actual max = cores - reserved)
health_check_interval = 30

[notifications]
enabled = true

[notifications.os]
enabled = true
sound = false
events = []               # Empty = all events

[queue]
auto_assign = true        # Automatically assign work when agents free
priority_order = ["INV", "FIX", "FEAT", "SPIKE"]
poll_interval_ms = 1000

[paths]
tickets = ".tickets"      # Relative to cwd
projects = "."            # cwd is projects root
state = ".tickets/operator"

[ui]
refresh_rate_ms = 250
completed_history_hours = 24
summary_max_length = 40

[launch]
confirm_autonomous = true
confirm_paired = true
launch_delay_ms = 2000

[sessions]
wrapper = "tmux"          # "tmux", "cmux", or "zellij"
```

## Ticket Priority

Work is assigned in priority order (not strict FIFO):

1. **INV** (Investigation) - Failures need immediate attention
2. **FIX** - Bug fixes and follow-up work
3. **FEAT** - New features
4. **SPIKE** - Research (requires human pairing)

Within each priority level, tickets are processed FIFO by timestamp.

## Agent Types

| Type | Mode | Parallelism | Human Required |
|------|------|-------------|----------------|
| Investigation | Paired | Single | Yes (urgent) |
| Spike | Paired | Single | Yes |
| Fix | Autonomous | Parallel OK | No (launch confirm only) |
| Feature | Autonomous | Parallel OK | No (launch confirm only) |

## Dashboard Layout

```
┌─────────────────────────────────────────────────────────────┐
│ operator v0.1.0                    ▶ RUNNING   5/7 agents  │
├─────────────┬─────────────┬─────────────┬───────────────────┤
│ QUEUE (12)  │ RUNNING (5) │ AWAITING (1)│ COMPLETED (8)     │
├─────────────┼─────────────┼─────────────┼───────────────────┤
│ INV-003 ‼️  │ backend     │ SPIKE-015   │ ✓ FEAT-040 12:30  │
│ FIX-089     │  FEAT-042   │  "what auth │ ✓ FIX-088  12:15  │
│ FIX-090     │  ██████░░   │   pattern?" │ ✓ FEAT-041 11:45  │
│ FEAT-043    │ frontend    │             │ ✓ FIX-087  11:30  │
│ FEAT-044    │  FIX-091    │ [R]espond   │                   │
│ FEAT-045    │  ████░░░░   │             │                   │
│             │ api         │             │                   │
│             │  FEAT-046   │             │                   │
│             │  ██░░░░░░   │             │                   │
│             │ admin       │             │                   │
│             │  FEAT-047   │             │                   │
│             │  █████████  │             │                   │
│             │ infra       │             │                   │
│             │  FIX-092    │             │                   │
│             │  ███░░░░░   │             │                   │
├─────────────┴─────────────┴─────────────┴───────────────────┤
│ [Q]ueue [L]aunch [P]ause [R]esume [A]gents [N]otifs [?]Help│
└─────────────────────────────────────────────────────────────┘
```

## Keyboard Shortcuts

See [Keyboard Shortcuts Reference](https://operator.untra.io/shortcuts/) for the full list, auto-generated from [`src/ui/keybindings.rs`](src/ui/keybindings.rs). Press `?` in the TUI to view shortcuts in-app.

## Integration Points

### LLM Agent Launch

Agents are launched in terminal sessions with the appropriate project folder and an initial prompt derived from the ticket.

### Third-Party Integrations

Investigations can be triggered externally:
- Webhook endpoint for alerting systems
- File watch for alert drop files
- CLI for manual urgent tickets

```bash
# Create urgent investigation from external alert
operator alert --source pagerduty --message "500 errors in backend" --severity S1
```

## LLM CLI Tool Integration

Operator launches LLM agents via CLI tools in terminal sessions. Each tool is configured via a JSON definition in `src/llm/tools/`.

### Supported Tools

| Tool | Detection | Models | Session Flag |
|------|-----------|--------|--------------|
| `claude` | `claude --version` | opus, sonnet, haiku | `--session-id` |
| `codex` | `codex --version` | gpt-4o, o1, o3 | `--resume` |
| `gemini` | `gemini --version` | pro, flash, ultra | `--resume` |

### How Operator Calls LLM Tools

Each tool has a JSON config in `src/llm/tools/` that defines argument mappings and a command template. Operator constructs the launch command from this config:

- **Prompt file**: Prompts are written to `.tickets/operator/prompts/<uuid>.txt` to avoid shell escaping issues with multiline prompts
- **Session ID**: A UUID v4 is generated per launch, enabling session resumption
- **Model aliases**: Operator uses short aliases (e.g., "opus", "sonnet") that resolve to latest model versions

### Adding Support for New LLM Tools

Create a new JSON tool config following the schema in `src/llm/tools/tool_config.schema.json`. The config defines:

- Tool binary name and version detection command
- Model aliases and argument mappings
- Command template with placeholder variables
- Capability flags (sessions, headless, permission modes)

**Requirements for the LLM tool:**
- Must be installable as a CLI binary
- Must accept prompt via flag (not just stdin)
- Must support model selection
- Should support session/conversation ID for continuity
- Should run interactively in a terminal (for session wrapper integration)

## Development

```bash
# Run in development
cargo run

# Run tests
cargo test

# Build release
cargo build --release
```

## Documentation

Reference documentation is auto-generated from source-of-truth files to minimize maintenance.

### Available References

| Generator | Source | Output |
|-----------|--------|--------|
| taxonomy | `src/backstage/taxonomy.toml` | `docs/backstage/taxonomy.md` |
| issuetype-schema | `src/schemas/issuetype_schema.json` | `docs/schemas/issuetype.md` |
| metadata-schema | `src/schemas/ticket_metadata.schema.json` | `docs/schemas/metadata.md` |
| shortcuts | `src/ui/keybindings.rs` | `docs/shortcuts/index.md` |
| cli | `src/main.rs`, `src/env_vars.rs` | `docs/cli/index.md` |
| config | `src/config.rs` | `docs/configuration/index.md` |
| OpenAPI | `src/rest/` (utoipa annotations) | `docs/schemas/openapi.json` |
| llm-tools | `src/llm/tools/tool_config.schema.json` | `docs/llm-tools/index.md` |
| startup | `src/startup/mod.rs` | `docs/startup/index.md` |
| config-schema | `docs/schemas/config.json` | `docs/schemas/config.md` |
| state-schema | `docs/schemas/state.json` | `docs/schemas/state.md` |
| schema-index | `docs/schemas/` | `docs/schemas/index.md` |
| jira-api | `docs/schemas/jira-api.json` | `docs/getting-started/kanban/jira-api.md` |

### Viewing Documentation

```bash
# Serve docs locally with Jekyll
cd docs && bundle install && bundle exec jekyll serve
# Visit http://localhost:4000

# View OpenAPI spec with Swagger UI
# After starting Jekyll, visit http://localhost:4000/schemas/api/
```

### Regenerating Documentation

```bash
# Regenerate all auto-generated docs
cargo run -- docs

# Regenerate specific docs
cargo run -- docs --only openapi
cargo run -- docs --only config

# Available generators: taxonomy, issuetype-schema, metadata-schema, shortcuts,
# cli, config, OpenAPI, llm-tools, startup, config-schema, state-schema,
# schema-index, jira-api
```

![Operator! logo](docs/assets/img/operator_logo.svg)

# Operator!

[![codecov](https://codecov.io/gh/untra/operator/branch/main/graph/badge.svg)](https://codecov.io/gh/untra/operator)

A multi-agent orchestration dashboard for [**AI-assisted**](https://operator.untra.io/getting-started/agents/) [_kanban-shaped_](https://operator.untra.io/getting-started/kanban/) [git-versioned](https://operator.untra.io/getting-started/git/) software development.

<a href="https://marketplace.visualstudio.com/items?itemName=untra.operator-terminals" target="_blank" class="button">Install <b>Operator! Terminals</b> extension from Visual Studio Code Marketplace</a>

Operator is for you if: 

- you do work assigned from tickets on a kanban board , such as [_Jira Cloud_](https://operator.untra.io/getting-started/kanban/jira/) or [_Linear_](https://operator.untra.io/getting-started/kanban/linear/)
- you use llm assisted coding agent tools to accomplish work, such as [_Claude Code_](https://operator.untra.io/getting-started/agents/claude/), [_OpenAI Codex_](https://operator.untra.io/getting-started/agents/codex/)
- your work is version controlled with git repository provider (like [_Github_](https://operator.untra.io/getting-started/git/github/))

- you are drowning in the AI software development soup.

and you're ready to start seriously automating your work.

## Overview

`operator` is a TUI (terminal user interface) application that uses [Tmux](https://github.com/tmux/tmux/wiki) to manages multiple Claude Code agents across multi-project workspaces of many codebases. It is designed to be ticket-first, starting claude code keyed off from markdown stories from a ticketing provider. It provides:

- **Queue Management**: ticket queue with priority-based work assignment, launchable from a dashboard
- **Agent Orchestration**: Launch, monitor, pause/resume Claude Desktop agents against kanban shaped work tickets, and track the ticket progress as it goes through your defined work implementation steps
- **Notifications**: macOS and linux notifications for agent events, keeping you the human in the loop. 
- **Dashboard**: Real-time view of queue, active agents, completed work, and waiting instances seeking feedback or human review

Operator is designed to facilitate work from markdown tickets, tackling tasks across multiple code repositories by semi-autonomous agents. Operator should be started from the root of your collective work projects repository (eg, `~/Documents`), so that it may start feature or fix work in the right part of the codebase.

When started for the first time, Operator will setup configuration to consume and process work tickets, and identify local projects with `claude.md files` to setup.

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
```

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      operator TUI                           │
├─────────────────────────────────────────────────────────────┤
│  Queue Manager    │  Agent Manager   │  Notification Svc   │
│  - Watch .tickets │  - Launch agents │  - macOS notifs     │
│  - Priority sort  │  - Track status  │  - Configurable     │
│  - Work assign    │  - Pause/resume  │  - Event hooks      │
├─────────────────────────────────────────────────────────────┤
│                    State Store (.operator/)                 │
│  - Agent sessions │  - Queue state   │  - Config           │
└─────────────────────────────────────────────────────────────┘
         │                    │                    │
         ▼                    ▼                    ▼
   Claude Desktop       .tickets/            Third-party
      Windows            queue/              integrations
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

Configuration lives in `~/.config/operator/config.toml` or `./config/default.toml`:

```toml
[agents]
max_parallel = 5          # Maximum concurrent agents
cores_reserved = 1        # Cores to keep free (max = cores - reserved)

[notifications]
enabled = true
on_agent_start = true
on_agent_complete = true
on_agent_needs_input = true
on_pr_created = true
sound = false

[queue]
auto_assign = true        # Automatically assign work when agents free
priority_order = ["INV", "FIX", "FEAT", "SPIKE"]

[paths]
tickets = "../.tickets"
projects = ".."
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

| Key | Action |
|-----|--------|
| `q` | Quit |
| `Q` | Focus queue panel |
| `L` | Launch next ticket (with confirmation) |
| `l` | Launch specific ticket |
| `P` | Pause all processing |
| `R` | Resume processing |
| `A` | Focus agents panel |
| `a` | View agent details |
| `N` | Toggle notifications |
| `?` | Help |
| `↑/↓` | Navigate lists |
| `Enter` | Select/expand |
| `Esc` | Back/cancel |

## Integration Points

### Claude Desktop

Agents are launched by opening Claude Desktop with the appropriate project folder and an initial prompt pointing to the ticket.

### Third-Party Integrations

Investigations can be triggered externally:
- Webhook endpoint for alerting systems
- File watch for alert drop files
- CLI for manual urgent tickets

```bash
# Create urgent investigation from external alert
operator alert --source pagerduty --message "500 errors in backend" --severity S1
```

## LLM CLI Tool Requirements

Operator launches LLM agents via CLI tools in tmux sessions. To be compatible with Operator, an LLM CLI tool must support the following capabilities:

### Required CLI Flags

| Flag | Purpose | Example |
|------|---------|---------|
| `-p` or `--prompt` | Accept an initial prompt/instruction | `-p "implement feature X"` |
| `--model` | Specify which model to use | `--model opus` |
| `--session-id` | UUID for session continuity/resumption | `--session-id 550e8400-...` |

### How Operator Calls LLM Tools

Operator constructs commands in this format:

```bash
<tool> --model <model> -p "$(cat <prompt_file>)" --session-id <uuid>
```

**Details:**
- **Prompt file**: Prompts are written to `.tickets/operator/prompts/<uuid>.txt` to avoid shell escaping issues with multiline prompts
- **Session ID**: A UUID v4 is generated per launch, enabling session resumption
- **Model aliases**: Operator uses short aliases (e.g., "opus", "sonnet") that resolve to latest model versions

### Currently Supported Tools

| Tool | Detection | Models |
|------|-----------|--------|
| `claude` | `which claude` + `claude --version` | opus, sonnet, haiku |

### Adding Support for New LLM Tools

To add support for a new LLM CLI tool (e.g., OpenAI Codex, Google Gemini):

1. Create a detector in `src/llm/<tool>.rs` that:
   - Checks if the binary exists (`which <tool>`)
   - Gets version (`<tool> --version`)
   - Returns available model aliases

2. Register the detector in `src/llm/detection.rs`

3. Update the launcher in `src/agents/launcher.rs` to handle the tool's specific CLI syntax

**Requirements for the LLM tool:**
- Must be installable as a CLI binary
- Must accept prompt via flag (not just stdin)
- Must support model selection
- Should support session/conversation ID for continuity
- Should run interactively in a terminal (for tmux integration)

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

| Reference | Location | Source |
|-----------|----------|--------|
| CLI Commands | `docs/cli/` | clap definitions in `src/main.rs` |
| Configuration | `docs/configuration/` | `src/config.rs` via schemars |
| Keyboard Shortcuts | `docs/shortcuts/` | `src/ui/keybindings.rs` |
| REST API (OpenAPI) | `docs/schemas/openapi.json` | utoipa annotations in `src/rest/` |
| Issue Type Schema | `docs/schemas/issuetype.md` | `src/schemas/issuetype_schema.json` |
| Ticket Metadata Schema | `docs/schemas/metadata.md` | `src/schemas/ticket_metadata.schema.json` |
| Project Taxonomy | `docs/backstage/taxonomy.md` | `src/backstage/taxonomy.toml` |

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
```

## TODO

* [ ] `--setup jira` option for jira sync for workspace setup
* [ ] `--setup linear` option for linear kanban provider
* [ ] `--setup gitlab` option for gitlab repository provider
* [ ] `--sync https://foobar.atlassian.net --project ABC` option for jira ticket sync

![Operator! logo](./img/operator_logo.svg)

# Operator!

Multi-agent orchestration dashboard for AI assisted development workflows.

## Overview

`operator` is a TUI (terminal user interface) application that manages multiple Claude Code agents across multi-project codebases. It provides:

- **Queue Management**: FIFO ticket queue with priority-based work assignment
- **Agent Orchestration**: Launch, monitor, pause/resume Claude Desktop agents
- **Notifications**: macOS notifications for agent events
- **Dashboard**: Real-time view of queue, active agents, and completed work

Operator is designed to facilitate ticket-based work across multiple code repositories by semi-autonomous agents. It should be started from the root of your collective work projects repository (eg, `~/Documents`).

When started for the first time, Operator will setup configuration to consume and process work tickets, and identify local projects with `claude.md files` to setup.

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

```bash
cd operator
# TBD: package manager installation
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

## Development

```bash
# Run in development
cargo run

# Run tests
cargo test

# Build release
cargo build --release
```

## TODO

* [ ] `--setup jira` option for jira sync for workspace setup
* [ ] `--setup notion` option for notion sync for workspace setup
* [ ] `--sync https://foobar.atlassian.net --project ABC` option for jira ticket sync

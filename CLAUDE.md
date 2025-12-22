# CLAUDE.md - operator

## Project Overview

`operator` is a Rust TUI application for orchestrating Claude Code agents across multi-project codebases. It manages ticket queues, launches agents, tracks progress, and provides notifications.

## Tech Stack

- **Language**: Rust
- **TUI Framework**: ratatui (with crossterm backend)
- **Async Runtime**: tokio
- **Notifications**: mac-notification-sys (macOS)
- **Config**: config crate (TOML)
- **File Watching**: notify crate

## Quick Reference

```bash
cargo fmt           # Format code
cargo clippy        # Lint
cargo test          # Run tests
cargo run           # Run TUI
cargo run -- queue  # CLI: show queue
cargo run -- launch # CLI: launch next ticket
```

## Architecture

```
src/
├── main.rs           # Entry point, CLI parsing
├── app.rs            # Application state and event loop
├── ui/               # TUI rendering
│   ├── mod.rs
│   ├── dashboard.rs  # Main dashboard layout
│   ├── queue.rs      # Queue panel
│   ├── agents.rs     # Agents panel
│   └── dialogs.rs    # Confirmation dialogs
├── queue/            # Queue management
│   ├── mod.rs
│   ├── ticket.rs     # Ticket parsing
│   ├── watcher.rs    # File system watcher
│   └── assigner.rs   # Work assignment logic
├── agents/           # Agent lifecycle
│   ├── mod.rs
│   ├── launcher.rs   # Claude Desktop integration
│   ├── tracker.rs    # Agent state tracking
│   └── session.rs    # Session persistence
├── notifications/    # Notification system
│   ├── mod.rs
│   └── macos.rs      # macOS notifications
├── config.rs         # Configuration management
└── state.rs          # Persistent state store
```

## Key Concepts

### Ticket Priority Order
1. INV (Investigation) - Failures, highest priority
2. FIX - Bug fixes
3. FEAT - Features
4. SPIKE - Research (requires pairing)

### Agent Modes
- **Autonomous** (FEAT, FIX): Launch and forget, monitor progress
- **Paired** (SPIKE, INV): Require human interaction, track "awaiting input"

### Parallelism Rules
- Max agents = min(configured_max, cpu_cores - reserved_cores)
- Autonomous agents can run in parallel across non-intersecting projects
- Paired agents run one at a time per operator attention
- Same project = sequential (to avoid conflicts)

## State Management

Operator state persists in `.operator/`:
```
.operator/
├── state.json        # Current queue/agent state
├── sessions/         # Agent session logs
│   └── {agent-id}.json
└── history.json      # Completed work log
```

## Ticket Workflow

1. **Watch**: Monitor `.tickets/queue/` for new tickets
2. **Sort**: Order by priority, then FIFO timestamp
3. **Assign**: When agent slot available, select next ticket
4. **Confirm**: Prompt operator for launch confirmation
5. **Launch**: Open Claude Desktop with project + ticket prompt
6. **Track**: Monitor agent progress, watch for completion
7. **Complete**: Move ticket, notify, update stats

## Claude Desktop Integration

Launch command (macOS):
```bash
open -a "Claude" --args --project "/path/to/project"
```

Initial prompt injected via:
- Clipboard + paste simulation, OR
- Project-specific `.claude/initial-prompt.md`, OR
- AppleScript automation

## Notifications

macOS notifications via `mac-notification-sys`:
```rust
Notification::new()
    .title("Agent Complete")
    .subtitle("backend")
    .message("FEAT-042: Add pagination")
    .send()?;
```

## Project Discovery

On startup, operator scans the configured projects directory for subdirectories containing a `CLAUDE.md` file. These are presented as available projects when creating tickets.

## Ticket Workflow

### Before Starting Work

1. Check `.tickets/queue/` for tickets matching your project
2. Look for `*-{project}-*.md` files, take the oldest (FIFO)
3. Move claimed ticket to `.tickets/in-progress/`
4. Create feature branch: `git checkout -b {branch-from-ticket}`

### Completing Work

1. Run `cargo fmt && cargo clippy && cargo test`
2. Commit with message: `{type}({project}): {summary}\n\nTicket: {ID}`
3. Create PR
4. Move ticket to `.tickets/completed/`

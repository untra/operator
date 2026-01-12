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

## Development Standards

### Mandatory Before Committing

All changes MUST pass these checks before committing:

```bash
cargo fmt                      # Format code
cargo clippy -- -D warnings    # Lint (warnings are errors)
cargo test                     # Run all tests
```

If any of these fail, fix the issues before proceeding. Do NOT use `#[allow(...)]` attributes to silence warnings unless there's a documented reason (e.g., code used only in tests).

### Subproject Validation

When changes touch subprojects, those must also pass validation:

**vscode-extension** (TypeScript/npm):
```bash
cd vscode-extension && npm run lint && npm run compile
```

**backstage-server** (TypeScript/Bun):
```bash
cd backstage-server && bun run lint && bun run typecheck && bun test
```

### Test-Driven Development (TDD)

This project follows TDD practices:

1. **Write tests first** - Before implementing a feature or fix, write a failing test that defines the expected behavior
2. **Run the test** - Verify it fails for the right reason
3. **Implement the minimum code** - Write just enough code to make the test pass
4. **Refactor** - Clean up while keeping tests green
5. **Repeat** - Add more tests to cover edge cases

Example workflow:
```bash
# 1. Write a new test in the appropriate module
# 2. Run tests to see it fail
cargo test test_new_feature -- --nocapture

# 3. Implement the feature
# 4. Run tests to see it pass
cargo test

# 5. Run full validation before committing
cargo fmt && cargo clippy -- -D warnings && cargo test
```

### Test Organization

- Unit tests go in the same file as the code, in a `#[cfg(test)] mod tests` block
- Integration tests go in `tests/` directory
- Use descriptive test names: `test_<function>_<scenario>_<expected_behavior>`

## Quick Reference

```bash
cargo fmt                      # Format code
cargo clippy -- -D warnings    # Lint (warnings as errors)
cargo test                     # Run all tests
cargo test <name>              # Run specific test
cargo run                      # Run TUI
cargo run -- queue             # CLI: show queue
cargo run -- launch            # CLI: launch next ticket
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

1. Run full validation: `cargo fmt && cargo clippy -- -D warnings && cargo test`
2. Ensure all tests pass and no clippy warnings
3. Commit with message: `{type}({project}): {summary}\n\nTicket: {ID}\n`

## Auto-Documentation System

Operator uses a schema-driven, code-derived documentation strategy to reduce maintenance burden. Documentation is auto-generated from structured source-of-truth files, ensuring docs stay in sync with code.

### Source-of-Truth Files

| File | Generates | Purpose |
|------|-----------|---------|
| `src/backstage/taxonomy.toml` | `docs/backstage/taxonomy.md` | 25 project Kinds across 5 tiers |
| `src/schemas/issuetype_schema.json` | `docs/schemas/issuetype.md` | Issue type structure (key, mode, fields, steps) |
| `src/schemas/ticket_metadata.schema.json` | `docs/schemas/metadata.md` | Ticket YAML frontmatter format |
| `src/ui/keybindings.rs` | `docs/shortcuts/index.md` | Keyboard shortcuts by context |
| `src/main.rs` + `src/env_vars.rs` | `docs/cli/index.md` | CLI commands and env vars |
| `src/config.rs` | `docs/configuration/index.md` | Config structure (via schemars) |
| `src/rest/` | `docs/schemas/openapi.json` | REST API spec (via utoipa) |

### Regenerating Documentation

```bash
# Generate all documentation
cargo run -- docs

# Generate specific docs only
cargo run -- docs --only taxonomy
cargo run -- docs --only openapi
cargo run -- docs --only config

# Available generators: taxonomy, issuetype, metadata, shortcuts, cli, config, openapi
```

### Auto-Generated File Headers

All generated files include a header warning:

```markdown
<!-- AUTO-GENERATED FROM {source} - DO NOT EDIT MANUALLY -->
<!-- Regenerate with: cargo run -- docs -->
```

### Adding New Generators

1. Create a struct implementing `DocGenerator` trait in `src/docs_gen/`
2. Implement `name()`, `source()`, `output_path()`, and `generate()`
3. Register in `src/docs_gen/mod.rs` `generate_all()` function
4. Add to CLI match in `src/main.rs` `cmd_docs()`

# CLAUDE.md - operator

## Project Overview

`operator` is a Rust TUI application for orchestrating Claude Code agents across multi-project codebases. It manages ticket queues, launches agents, tracks progress, and provides notifications.

## Tech Stack

- **Language**: Rust (core), TypeScript (`ui/`, `webcomponents/`, `vscode-extension/`)
- **TUI**: ratatui (crossterm backend); tokio async runtime
- **Server surfaces**: axum REST API + utoipa OpenAPI (`src/rest/`), MCP server (`src/mcp/`), ACP (`src/acp/`)
- **Web**: Vite/React SPA in `ui/`, shared `webcomponents/` package, thin VS Code extension webview
- **Notifications**: mac-notification-sys (macOS), notify-rust (Linux), webhooks
- **Config**: config crate (TOML); **File Watching**: notify crate

## Code Style
Aim for functional software development with a focus on stateless, single responsibility focus.
Minimize use of comments; they should be terse and used judiciously, ideally one sentence tops.
Data types come from rust; typescript and docs binds are generated from low-level rust types annotated with comments that embed as descriptions into configuration and reference files.
Favor falsey defaults ; lets aim not to enforce `default=true` or some other javascript-truthy default value.

## Plans & Specs Location

Write superpowers plans to `superpowers/plans/` and design specs to `superpowers/specs/` (repo root, not hosted).

## Development Standards

1. Ask, don't assume. If something is unclear, ask before writing into a corner. Never make silent assumptions about intent, architecture or requirements.
2. Consider the simplest solution first. First attempt the simplest approach that could work, then consider abstractions and flexibility with regards to similar implementations.
3. Avoid changing unrelated code. If a file or function is not part of the current task, do not modify it.
4. Flag uncertainty explicitly. If you are not confident about an approach or technical detail, say so before proceeding.
5. I am open to ideas on better approaches or strategies. Speak up and suggest a better course if I am off-course from a better solution.

### Mandatory Before Committing

All changes MUST pass these checks before committing. Run them with `make check`, which mirrors the CI `lint-test` job exactly:

```bash
make check
# equivalent to the exact CI commands:
cargo fmt --all -- --check                                   # Format check
cargo clippy --locked --all-targets --all-features -- -D warnings  # Lint (warnings are errors)
cargo test --locked                                          # Run all tests
```

> The `--locked --all-targets --all-features` flags matter: plain
> `cargo clippy` misses test-target and feature-gated lints (e.g. a dependency
> deprecation that only surfaces under `--all-targets`), which is how a clippy
> failure can pass locally yet break CI. Always use the full command above.

Install the pre-push hook once per clone so the fast lint gate (fmt + clippy,
no tests) runs automatically before every push; the full `make check` remains
the expectation before opening a PR:

```bash
make install-hooks   # sets core.hooksPath=.githooks
```

If any of these fail, fix the issues before proceeding. Do NOT use `#[allow(...)]` attributes to silence warnings unless there's a documented reason (e.g., code used only in tests).

### Subproject Validation

When changes touch subprojects, those must also pass validation:

**opr8r** (Rust/cargo):
```bash
cargo run 
```

**vscode-extension** (TypeScript/npm):
```bash
cd vscode-extension && npm run lint && npm run compile
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
make check
```

### Test Organization

- Unit tests go in the same file as the code, in a `#[cfg(test)] mod tests` block
- Integration tests go in `tests/` directory
- Use descriptive test names: `test_<function>_<scenario>_<expected_behavior>`

## Quick Reference

```bash
make check                     # Full CI-parity gate (fmt + clippy + test)
make install-hooks             # Install the lint-only pre-push hook (once per clone)
cargo fmt                      # Format code
cargo clippy --locked --all-targets --all-features -- -D warnings  # Lint (CI parity)
cargo test                     # Run all tests
cargo test <name>              # Run specific test
cargo run                      # Run TUI
cargo run -- queue             # CLI: show queue
cargo run -- launch            # CLI: launch next ticket
cargo run -- api               # REST API + embedded web UI
cargo run -- mcp               # MCP server (stdio)
cargo run -- docs              # Regenerate auto-generated docs
```

Full command list: `docs/cli/` (auto-generated).

## Architecture

Grouped map of `src/` (not exhaustive — `ls src/` for the full list):

```
src/
├── main.rs, lib.rs    # Entry + CLI parsing; lib/bin split (src/rest compiles
│                      #   in the lib and must not reference bin-only src/ui)
├── app/               # TUI application state and event loop
├── ui/                # Ratatui rendering: dashboard, panels, dialogs, keybindings
├── queue/             # Ticket parsing, creation, file watcher
├── agents/            # Agent lifecycle: launcher/ (tmux, zellij, cmux, coder,
│                      #   remote), monitor, activity/idle detection, hooks
├── config.rs, config/ # TOML config: agent profiles, kanban, llm tools,
│                      #   sessions, targets, git, notifications
├── state.rs           # Persistent state store
├── rest/              # REST API + embedded web UI hosting (axum, utoipa)
├── mcp/, acp/         # MCP server; Agent Client Protocol
├── api/, services/    # Kanban/GitHub/PR clients and sync services
├── llm/, permissions/ # LLM tool detection + runtime configs; per-tool
│                      #   permission translation (claude, codex, gemini)
├── issuetypes/, templates/, collections/  # Issue type schema, registry,
│                      #   shipped collections
├── taxonomy/, schemas/, docs_gen/  # Source-of-truth data + docs generators
├── workflow_gen/      # Workflow export (Claude .js, AGNT)
├── notifications/     # OS notifications (macOS/Linux) + webhook integrations
└── git/, steps/, relay/, integrations/, startup/, editors.rs, projects.rs, …
```

Sibling subprojects: `ui/` (SPA), `webcomponents/` (shared JS, built ahead of
`ui/` and `docs/`), `vscode-extension/`, `opr8r/`, `docs/` (hosted Jekyll
site), `collections/community/`.

## Key Concepts

### Ticket Priority
Queue order is `queue.priority_order` (default INV > FIX > TASK > FEAT > SPIKE), then FIFO.
Issue types are schema-driven (collections + customm templates), not a fixed set.

### Agent Modes
Execution mode is declared per issue type (`mode` in the issuetype schema):
- **Autonomous** (e.g. FEAT, FIX, TASK): launch and monitor progress
- **Paired** (e.g. SPIKE, INV): require human interaction, track "awaiting input"

### Parallelism Rules
- Effective max agents = max(1, min(`agents.max_parallel`, cpu_cores − `agents.cores_reserved`))
- Same repo is sequential unless `git.use_worktrees = true`, which allows up to
  `agents.max_agents_per_repo` agents in per-ticket worktrees
- Paired agents run one at a time per operator attention

## State Management

Persistent state lives under `paths.state` (default `.tickets/operator/`);
`state.json` holds queue/agent state — schema documented at `/schemas/state/`.
Per-ticket worktrees default to `~/.operator/worktrees`.

## Ticket Workflow

1. **Watch**: Monitor `.tickets/queue/` for new tickets
2. **Sort**: Order by priority, then FIFO timestamp
3. **Assign**: When agent slot available, select next ticket
4. **Confirm**: Prompt operator for launch confirmation
5. **Launch**: Run the agent CLI in the configured session target with the interpolated prompt
6. **Track**: Monitor agent progress, watch for completion
7. **Complete**: Move ticket, notify, update stats

## Agent Launching

`src/agents/launcher/` builds the agent CLI command (claude, codex, gemini) with a prompt interpolated from the ticket + issuetype steps,
then runs it in the configured session target:

- Terminal multiplexers: tmux, zellij, cmux
- Editors: VS Code, Cursor, Zed (`src/editors.rs`)
- Remote: SSH hosts and Coder workspaces (`[[targets]]` config)

Per-ticket git worktrees are prepared by `launcher/worktree_setup.rs`.

## Notifications

Dispatched through `src/notifications/service.rs` to the enabled integrations:
OS notifications (mac-notification-sys on macOS, notify-rust on Linux) and
webhooks.

## Project Discovery

On startup, operator scans the configured projects directory for subdirectories containing an agent marker file (`CLAUDE.md`, `GEMINI.md`, `CODEX.md`).
These are presented as available projects when creating tickets.

## Working a Ticket

### Before Starting Work

1. Check `.tickets/queue/` for tickets matching your project
2. Look for `*-{project}-*.md` files, take the oldest (FIFO)
3. Move claimed ticket to `.tickets/in-progress/`
4. Create feature branch: `git checkout -b {branch-from-ticket}`

### Completing Work

1. Run full validation: `make check` (CI-parity fmt + clippy + test)
2. Ensure all tests pass and no clippy warnings
3. Commit with message: `{type}({project}): {summary}\n\nTicket: {ID}\n`

## Auto-Documentation System

Operator uses a schema-driven, code-derived documentation strategy to reduce maintenance burden. Documentation is auto-generated from structured source-of-truth files, ensuring docs stay in sync with code.

### Source-of-Truth Files

| File | Generates | Purpose |
|------|-----------|---------|
| `src/taxonomy/taxonomy.toml` | `docs/taxonomy/index.md` | 25 project Kinds across 5 tiers |
| `src/schemas/issuetype_schema.json` | `docs/schemas/issuetype.md` | Issue type structure (key, mode, fields, steps) |
| `src/schemas/ticket_metadata.schema.json` | `docs/schemas/metadata.md` | Ticket YAML frontmatter format |
| `src/ui/keybindings.rs` | `docs/shortcuts/index.md` | Keyboard shortcuts by context |
| `src/main.rs` + `src/env_vars.rs` | `docs/cli/index.md` | CLI commands and env vars |
| `src/config.rs` | `docs/configuration/index.md` | Config structure (via schemars) |
| `src/rest/` | `docs/schemas/openapi.json` | REST API spec (via utoipa) |
| `src/docs_gen/llms.rs` + `docs/*/index.md` | `docs/llms.txt` | llms.txt site map for LLMs (no front matter; served verbatim) |
| `src/collections/*/collection.json` + `collections/community/*/` | `docs/collections/` | Hosted collection bundle: `index.json`, per-collection manifests, issuetype schemas, templates, icons |
| the same collection sources | `docs/collections/search.json` | Machine-readable collection catalog (also feeds the `/workflows/` page) |
| the same collection sources | `docs/workflows/` | The workflow catalog page + one page per collection |

### Regenerating Documentation

```bash
# Generate all documentation
cargo run -- docs

# Generate specific docs only
cargo run -- docs --only taxonomy
cargo run -- docs --only openapi
cargo run -- docs --only config

# `--only` accepts any key from docs_gen::all_generators(); an unknown key
# prints the full list. `llm-tools` is opt-in only: it is excluded from a full
# run because docs/llm-tools/index.md is currently maintained by hand.
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
3. Register it in `src/docs_gen/mod.rs` `all_generators()` — that one list drives
   the full run, the `--only` filter, and the CLI help text, so there is nothing
   to add in `src/main.rs`

## Design & UI Consistency

Operator presents one brand (terracotta + cornflower + cream over a green
scale) across **four rendering surfaces**. Keep them consistent by following the
rule that fits each surface — they are deliberately *not* all styled the same
way. Full details and swatches live in `docs/design-system/` (`/design-system/`).

**Brand source of truth:** `docs/assets/css/tokens.css` — the only place the
brand hex values + dark-mode overrides are declared. Both web surfaces consume
it; never re-declare a brand color elsewhere.

| Surface | Where | Rule |
|---------|-------|------|
| Docs site (Jekyll) | `docs/assets/css/main.css` | Links `tokens.css` (via `_includes/head.html`); style components with `var(--...)`, never raw hex. |
| Embedded SPA (Vite/React) | `ui/src/index.css` + `*.module.css` | Imports `tokens.css`; layers app-only semantic tokens (`--surface`, `--border`, `--danger`, …) on top. Components reference semantic tokens, not raw hex. |
| Ratatui TUI | `src/ui/*.rs` | Terminal can't render hex — match a **semantic role to ANSI** (danger→Red, success→Green, warning→Yellow, focus→Cyan). Reuse `color_for_key`/`glyph_for_key` from `src/templates/mod.rs`; don't re-hardcode issuetype/priority colors. |
| VS Code webview | `vscode-extension/webview-ui/` | **Defer to the VS Code host theme**: style with raw `var(--vscode-*)` custom properties (`styles/webview.css` + `components/primitives/`). Apply brand only as accents via the `--op-*` variables; never override the user's editor theme wholesale. No MUI/CSS-in-JS — enforced by `tests/ui_packaging.rs`. |

When adding or changing UI: change a brand color in `tokens.css` (web surfaces
follow automatically); reference semantic tokens in new web CSS; map a role to
ANSI in the TUI; and leave the webview deferring to the editor theme.

**Icons.** Every SVG icon follows the Operator icon standard — a single
monochrome `<path>` on a 24×24 canvas with no `fill`/`stroke`/`width`/`height`,
so it tints from `currentColor` and sizes to its container on all four
surfaces. Governed directories: `icons/`, `docs/assets/icons/`,
`ui/public/icons/`, and each collection's `icon.svg`. Enforced by
`cargo test --test svg_icon_standard`; the rules and rationale are in
`docs/design-system/`.

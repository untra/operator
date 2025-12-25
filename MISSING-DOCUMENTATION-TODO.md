# Missing Documentation TODO

This document tracks documentation gaps in the operator project. The docs site is published at [operator.untra.io](https://operator.untra.io).

---

## Auto-Documentation Architecture

### Design Principle: Single Source of Truth

The codebase uses patterns for self-documenting structures (e.g., `TemplateType` enum with `.description()`, `.display_name()` methods). This pattern extends to:

| Documentation | Source of Truth | Generator |
|---------------|-----------------|-----------|
| Keyboard Shortcuts | `src/ui/keybindings.rs` (registry) | `ShortcutsDocGenerator` |
| CLI Reference | `src/main.rs` (clap metadata) | `CliDocGenerator` |
| Configuration | `src/config.rs` (schemars schema) | `ConfigDocGenerator` |
| Taxonomy | `src/backstage/taxonomy.toml` | `TaxonomyDocGenerator` (exists) |
| Issue Type Schema | `src/templates/issuetype_schema.json` | `IssuetypeSchemaDocGenerator` (exists) |
| Ticket Metadata | `src/templates/ticket_metadata.schema.json` | `MetadataSchemaDocGenerator` (exists) |

**Command**: `cargo run -- docs` generates all documentation

---

## Priority 1: Auto-Generated Reference Docs (Existing Generators)

These generators exist in `src/docs_gen/` but output directories don't exist yet.

### Taxonomy Documentation
- [ ] Create `docs/backstage/` directory
- [ ] Generate `docs/backstage/taxonomy.md` (24-Kind taxonomy reference)
  - **Source**: `src/backstage/taxonomy.toml` (583 lines)
  - **Generator**: `TaxonomyDocGenerator` in `src/docs_gen/taxonomy.rs`
  - **Content**: Summary tables, tier sections, file patterns, Backstage type mappings

### Schema Documentation
- [ ] Create `docs/schemas/` directory
- [ ] Generate `docs/schemas/issuetype.md` (Issue type JSON schema)
  - **Source**: `src/templates/issuetype_schema.json` (366 lines)
  - **Generator**: `IssuetypeSchemaDocGenerator` in `src/docs_gen/issuetype.rs`
- [ ] Generate `docs/schemas/metadata.md` (Ticket metadata schema)
  - **Source**: `src/templates/ticket_metadata.schema.json`
  - **Generator**: `MetadataSchemaDocGenerator` in `src/docs_gen/metadata.rs`

### Navigation Updates
- [ ] Add navigation entries to `docs/_data/navigation.yml`:
  ```yaml
  - title: Backstage
    url: /backstage/
  - title: Schemas
    url: /schemas/
  ```

---

## Priority 2: Keyboard Shortcuts (NEW Generator Required)

**Current Problem**: Shortcuts defined in `handle_key()` match statements, help text maintained separately in `HelpDialog`.

### Architecture: Declarative Registry

Create `src/ui/keybindings.rs`:

```rust
pub struct Shortcut {
    pub key: KeyCode,
    pub alt_key: Option<KeyCode>,  // e.g., 'L' and 'l'
    pub description: &'static str,
    pub category: &'static str,    // "Navigation", "Actions", "Dialogs"
    pub context: &'static str,     // "global", "preview", "launch-dialog"
}

static SHORTCUTS: &[Shortcut] = &[
    Shortcut { key: KeyCode::Char('q'), alt_key: None, description: "Quit", category: "General", context: "global" },
    Shortcut { key: KeyCode::Char('W'), alt_key: Some(KeyCode::Char('w')), description: "Toggle Backstage server", category: "Actions", context: "global" },
    // ...
];
```

### Implementation Tasks
- [ ] Create `src/ui/keybindings.rs` with `Shortcut` registry
- [ ] Modify `src/ui/mod.rs` - Add `pub mod keybindings;`
- [ ] Modify `src/ui/dialogs.rs` - `HelpDialog` generates from registry
- [ ] Create `src/docs_gen/shortcuts.rs` - `ShortcutsDocGenerator`
- [ ] Modify `src/docs_gen/mod.rs` - Add shortcuts generator
- [ ] Create `docs/shortcuts/` directory
- [ ] Add navigation entry for shortcuts

### Shortcuts to Include

| Key | Action | Category | Context |
|-----|--------|----------|---------|
| `q` | Quit | General | global |
| `Tab` | Switch panels | Navigation | global |
| `j/k` | Navigate within panel | Navigation | global |
| `Enter` | Select/confirm | Actions | global |
| `Esc` | Cancel/close | Actions | global |
| `L` | Launch ticket | Actions | global |
| `P` | Pause queue | Actions | global |
| `R` | Resume queue | Actions | global |
| `C` | Create ticket | Actions | global |
| `J` | Projects menu | Dialogs | global |
| `?` | Help dialog | Dialogs | global |
| `W` | Toggle Backstage server & open browser | Actions | global |
| `S` | Manual sync (rate limits + ticket-session) | Actions | global |
| `V` | Show session preview | Actions | global |
| `Q` | Focus queue panel | Navigation | global |
| `A` | Focus agents panel | Navigation | global |
| `g` | Scroll to top | Navigation | preview |
| `G` | Scroll to bottom | Navigation | preview |
| `PageUp/Down` | Page scroll | Navigation | preview |

---

## Priority 3: CLI Reference (NEW Generator Required)

**Current State**: Clap definitions in `src/main.rs` already contain all metadata via `#[command]` and `#[arg]` attributes.

### Architecture: Clap Introspection

Create `src/docs_gen/cli.rs`:

```rust
pub struct CliDocGenerator;

impl DocGenerator for CliDocGenerator {
    fn name(&self) -> &'static str { "cli" }
    fn source(&self) -> &'static str { "src/main.rs (clap definitions)" }
    fn output_path(&self) -> &'static str { "cli/index.md" }

    fn generate(&self) -> Result<String> {
        let cmd = Cli::command();  // Get clap Command
        // Iterate subcommands, extract help text, format as markdown
    }
}
```

### Implementation Tasks
- [ ] Create `src/docs_gen/cli.rs` - `CliDocGenerator` using clap introspection
- [ ] Modify `src/docs_gen/mod.rs` - Add cli generator
- [ ] Modify `src/main.rs` - Include cli in `cmd_docs()`
- [ ] Create `docs/cli/` directory
- [ ] Add navigation entry

### Commands to Document (auto-extracted from clap)
| Command | Description |
|---------|-------------|
| `operator` | Launch TUI (default) |
| `operator queue` | List tickets in queue |
| `operator launch` | Launch next ticket |
| `operator agents` | List active agents |
| `operator pause` | Pause queue processing |
| `operator resume` | Resume queue processing |
| `operator stalled` | Show stalled tickets |
| `operator alert` | Send test notification |
| `operator create` | Create new ticket |
| `operator docs` | Generate documentation |
| `operator docs --output <path>` | Custom output directory |
| `operator docs --only <generator>` | Generate specific docs |

---

## Priority 4: Configuration Documentation (NEW Generator Required)

**Current State**: `src/config.rs` has 15+ structs with doc comments and `#[serde(default)]`.

### Architecture: schemars + JsonSchema

1. Add dependency: `schemars = "0.8"`
2. Derive `JsonSchema` on config structs
3. Create `ConfigDocGenerator`

```rust
pub struct ConfigDocGenerator;

impl DocGenerator for ConfigDocGenerator {
    fn generate(&self) -> Result<String> {
        let schema = schema_for!(Config);
        // Convert JSON Schema to markdown tables
    }
}
```

### Implementation Tasks
- [ ] Modify `Cargo.toml` - Add `schemars = "0.8"`
- [ ] Modify `src/config.rs` - Add `#[derive(JsonSchema)]` to structs
- [ ] Create `src/docs_gen/config.rs` - `ConfigDocGenerator`
- [ ] Modify `src/docs_gen/mod.rs` - Add config generator
- [ ] Create `docs/configuration/` directory
- [ ] Add navigation entry

### Config Sections (auto-documented from structs)
| Section | Description | Source Struct |
|---------|-------------|---------------|
| `agents` | Max parallel, health checks, timeouts | `AgentsConfig` |
| `notifications` | macOS notifications settings | `NotificationsConfig` |
| `queue` | Auto-assign, priority order | `QueueConfig` |
| `paths` | Tickets, projects, state paths | `PathsConfig` |
| `ui` | Refresh rate, panel names | `UIConfig` |
| `launch` | Prompt injection, shell command | `LaunchConfig` |
| `templates` | Preset, collection | `TemplatesConfig` |
| `backstage` | Port, auto_start, subpath | `BackstageConfig` |
| `tmux` | Config generation flag | `TmuxConfig` |
| `llm_tools` | Detected tools, providers | `LlmToolsConfig` |

### Environment Variables (manual section)
- [ ] Document API provider environment variables
  - `ANTHROPIC_API_KEY`
  - `GITHUB_TOKEN`
  - Linear/Jira integration vars

---

## Priority 5: Backstage User Guide (MANUAL)

This requires manual writing - not auto-generated.

### Tasks
- [ ] Create `docs/backstage/index.md` (manual write)
  - What is Backstage and why it's integrated
  - How `[W]` keybinding works (toggle server, open browser)
  - The 24-Kind taxonomy system explained for users
  - Scaffold location: `.tickets/operator/backstage/`
  - How to customize branding (`branding/` subdirectory)
  - Prerequisites (Bun installation)

### Reference Files
- `src/backstage/server.rs` - Server lifecycle
- `src/backstage/scaffold.rs` - Scaffold generation
- `src/backstage/branding.rs` - Branding defaults
- `BACKSTAGE-IMPLEMENTATION-PLAN.md` - Full technical spec

---

## Implementation Phases

### Phase 1: Run Existing Generators

```bash
# Create directories
mkdir -p docs/backstage docs/schemas

# Generate reference documentation
cargo run -- docs

# Verify output
ls docs/backstage/taxonomy.md
ls docs/schemas/issuetype.md
ls docs/schemas/metadata.md
```

### Phase 2: Keyboard Shortcuts Registry

| File | Action |
|------|--------|
| `src/ui/keybindings.rs` | CREATE - Shortcut registry with metadata |
| `src/ui/mod.rs` | MODIFY - Add `pub mod keybindings;` |
| `src/ui/dialogs.rs` | MODIFY - HelpDialog generates from registry |
| `src/docs_gen/shortcuts.rs` | CREATE - ShortcutsDocGenerator |
| `src/docs_gen/mod.rs` | MODIFY - Add shortcuts generator |

### Phase 3: CLI Documentation Generator

| File | Action |
|------|--------|
| `src/docs_gen/cli.rs` | CREATE - CliDocGenerator using clap introspection |
| `src/docs_gen/mod.rs` | MODIFY - Add cli generator |
| `src/main.rs` | MODIFY - Include cli in `cmd_docs()` |

### Phase 4: Configuration Documentation

| File | Action |
|------|--------|
| `Cargo.toml` | MODIFY - Add `schemars = "0.8"` |
| `src/config.rs` | MODIFY - Add `#[derive(JsonSchema)]` to structs |
| `src/docs_gen/config.rs` | CREATE - ConfigDocGenerator |
| `src/docs_gen/mod.rs` | MODIFY - Add config generator |

### Phase 5: Manual Documentation

- Write `docs/backstage/index.md` (user guide)
- Update `docs/_data/navigation.yml` with all new sections

---

## Directory Structure (After Implementation)

```
docs/
├── index.md                    # ✅ Exists
├── kanban/index.md             # ✅ Exists
├── issue-types/index.md        # ✅ Exists
├── tickets/index.md            # ✅ Exists
├── agents/index.md             # ✅ Exists
├── llm-tools/index.md          # ✅ Exists
├── tmux/index.md               # ✅ Exists
├── shortcuts/index.md          # 🔧 Auto-generated (Phase 2)
├── cli/index.md                # 🔧 Auto-generated (Phase 3)
├── configuration/index.md      # 🔧 Auto-generated (Phase 4)
├── backstage/
│   ├── index.md                # ✍️ Manual (Phase 5)
│   └── taxonomy.md             # 🔧 Auto-generated (Phase 1)
└── schemas/
    ├── issuetype.md            # 🔧 Auto-generated (Phase 1)
    └── metadata.md             # 🔧 Auto-generated (Phase 1)
```

---

## Navigation Update (`_data/navigation.yml`)

```yaml
docs:
  - title: Kanban
    url: /kanban/
  - title: Issue Types
    url: /issue-types/
  - title: LLM Tools
    url: /llm-tools/
  - title: Tickets
    url: /tickets/
  - title: Agents
    url: /agents/
  - title: Tmux
    url: /tmux/
  # NEW ENTRIES:
  - title: Shortcuts
    url: /shortcuts/
  - title: CLI
    url: /cli/
  - title: Configuration
    url: /configuration/
  - title: Backstage
    url: /backstage/
  - title: Schemas
    url: /schemas/
```

---

## Summary: All Generators After Implementation

| Generator | Output | Source | Status |
|-----------|--------|--------|--------|
| TaxonomyDocGenerator | `docs/backstage/taxonomy.md` | `taxonomy.toml` | ✅ Exists |
| IssuetypeSchemaDocGenerator | `docs/schemas/issuetype.md` | `issuetype_schema.json` | ✅ Exists |
| MetadataSchemaDocGenerator | `docs/schemas/metadata.md` | `ticket_metadata.schema.json` | ✅ Exists |
| ShortcutsDocGenerator | `docs/shortcuts/index.md` | `src/ui/keybindings.rs` | 🔧 To Create |
| CliDocGenerator | `docs/cli/index.md` | `src/main.rs` (clap) | 🔧 To Create |
| ConfigDocGenerator | `docs/configuration/index.md` | `src/config.rs` (schemars) | 🔧 To Create |

---

## Notes

- Generated docs include `<!-- AUTO-GENERATED - DO NOT EDIT MANUALLY -->` headers
- The `docs_gen` module is comprehensive (15 tests, ~1,200 lines)
- All generators implement the `DocGenerator` trait
- Markdown utilities in `src/docs_gen/markdown.rs` for consistent formatting

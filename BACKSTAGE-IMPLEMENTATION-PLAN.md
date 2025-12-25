# Backstage Deployer - Implementation Plan

A local-first, minimal Backstage deployment system integrated into operator with an expandable project taxonomy.

> **Status Update (2024-12)**: **100% Complete** - All milestones fully implemented.
> Milestone 6 (TUI Integration) was completed with `[W]` keybinding wired in `app.rs:713-717`.
> The backstage module contains ~6,600 lines of code with 103 tests.

---

## Overview

- **Location**: `.tickets/operator/backstage/` (Bun-based scaffold)
- **TUI Command**: `[W]eb` to launch/toggle Backstage server
- **Assessment**: `[J]` project command creates ASSESS ticket to analyze project and generate `catalog-info.yaml`
- **Taxonomy**: 24-Kind system with single source of truth in `src/backstage/taxonomy.toml`

---

## Execution Status

| Milestone | Focus | Status | Lines | Tests |
|-----------|-------|--------|-------|-------|
| M1 | Taxonomy Source of Truth | ✅ COMPLETE | 594 | 23 |
| M2 | Auto-documentation Generator | ✅ COMPLETE | 259 | 2 |
| M3 | Issue Type Framework | ✅ COMPLETE | — | 45 |
| M4 | Project Analysis Engine | ✅ COMPLETE | 812 | 9 |
| M5 | Server Lifecycle | ✅ COMPLETE | 820 | 34 |
| M6 | TUI Integration | ✅ COMPLETE | — | — |
| M7 | Backstage Scaffold | ✅ COMPLETE | 852 | 18 |
| M8 | Documentation | ✅ COMPLETE | — | — |

---

## Implementation Notes

### Milestone 1: Taxonomy System ✅

**Files**: `src/backstage/taxonomy.toml`, `src/backstage/taxonomy.rs`

The taxonomy defines 24 project Kinds across 4 Tiers:

| Tier | Range | Description | Kinds |
|------|-------|-------------|-------|
| Foundation | 1-4 | Infrastructure and platform foundations | infrastructure, identity-access, config-policy, monorepo-meta |
| Standards | 5-10 | Shared components and specifications | design-system, software-library, proto-sdk, blueprint, security-tooling, compliance-audit |
| Engines | 11-16 | Core business logic and services | ml-model, data-etl, microservice, api-gateway, ui-frontend, internal-tool |
| Ecosystem | 17-24 | Supporting tools and utilities | build-tool, e2e-test, docs-site, playbook, reference-example, cli-devtool, experiment-sandbox, archival-fork |

**Key Features**:
- Compile-time validated via `include_str!("taxonomy.toml")`
- `Lazy<Taxonomy>` singleton for efficient access
- File pattern detection: `detect_kind()`, `matching_kinds()`
- Navigation metadata: `icon`, `display_order`, `sidebar_label`
- 23 comprehensive tests validating integrity

### Milestone 2: Documentation Generator ✅

**Files**: `src/docs_gen/taxonomy.rs`, `src/docs_gen/markdown.rs`

Generates markdown documentation from taxonomy:
- Summary tables with all 24 Kinds
- Tier-specific sections with descriptions
- File pattern reference with glob syntax
- Backstage type mappings

### Milestone 3: Issue Types ✅

**Files**: `src/templates/assess.json`, `sync.json`, `init.json`

Three operator-specific issue types for Backstage workflows:

| Type | Mode | Glyph | Workflow |
|------|------|-------|----------|
| ASSESS | Autonomous | `~` | analyze → generate |
| SYNC | Autonomous | `@` | scan → validate → update |
| INIT | Paired | `%` | scaffold → configure → verify |

**Also implemented**:
- `src/templates/issuetype_schema.json` - JSON schema for validation
- `src/templates/project_analysis.schema.json` - ASSESS output schema
- `Operator` and `BackstageFull` collection presets in `src/issuetypes/collection.rs`

### Milestone 4: Project Analyzer ✅

**Files**: `src/backstage/analyzer.rs`

Complete type system for project analysis (812 lines):

```rust
ProjectAnalysis {
    project_name, project_path, detected_files,
    kind_assessment: KindAssessment { detected_kind, confidence, alternatives },
    languages: Vec<LanguageDetection>,
    frameworks: Vec<FrameworkDetection>,
    databases: Vec<DatabaseDetection>,
    docker: Option<DockerDetection>,
    ports: Vec<PortDetection>,
    tests: Vec<TestFrameworkDetection>,
    file_stats: FileStats,
}
```

Detection categories:
- **Frameworks**: Web, ORM, Testing, Build, Logging, Serialization, CLI, Async, API, UI
- **Databases**: Relational, Document, KeyValue, Graph, TimeSeries, MessageQueue, Search, Cache
- **Ports**: HTTP, HTTPS, gRPC, Database, Redis, RabbitMQ, WebSocket, Metrics, Debug
- **Evidence**: FileExists, FilePattern, ContentMatch, ConfigKey, Dependency, Import, Extension

### Milestone 5: Server Lifecycle ✅

**Files**: `src/backstage/server.rs`

Bun server process management (820 lines, 34 tests):

```rust
// Trait-based architecture for testing
pub trait BunClient: Send + Sync {
    fn check_available(&self) -> Result<BunVersion, BackstageError>;
    fn check_dependencies(&self, path: &Path) -> Result<bool, BackstageError>;
    fn install_dependencies(&self, path: &Path) -> Result<(), BackstageError>;
    fn start_server(&self, path: &Path, port: u16) -> Result<u32, BackstageError>;
    fn is_process_running(&self, pid: u32) -> bool;
}

// Implementations
pub struct SystemBunClient;  // Real Bun process management
pub struct MockBunClient;    // For unit testing

// Lifecycle manager
pub struct BackstageServer<C: BunClient> {
    pub fn start(&mut self) -> Result<(), BackstageError>;
    pub fn stop(&mut self) -> Result<(), BackstageError>;
    pub fn toggle(&mut self) -> Result<(), BackstageError>;
    pub fn open_browser(&self) -> Result<(), BackstageError>;
    pub fn status(&self) -> &ServerStatus;
}
```

### Milestone 7: Scaffold Generator ✅

**Files**: `src/backstage/scaffold.rs`, `src/backstage/branding.rs`

Generates complete Backstage structure (852 lines, 18 tests):

```
.tickets/operator/backstage/
├── package.json              # Bun/Node workspace
├── app-config.yaml           # Guest auth + 24 taxonomy kinds
├── app-config.production.yaml
├── Dockerfile
├── docker-compose.yaml
├── README.md
├── packages/
│   ├── app/package.json
│   └── backend/package.json
└── branding/logo.svg
```

**Generators** (implementing `ScaffoldFile` trait):
- `PackageJsonGenerator`, `AppConfigGenerator`, `AppConfigProductionGenerator`
- `DockerfileGenerator`, `DockerComposeGenerator`, `ReadmeGenerator`
- `AppPackageGenerator`, `BackendPackageGenerator`, `LogoGenerator`

**Features**:
- Idempotent: skips existing files unless `force: true`
- All 24 taxonomy kinds in catalog rules
- Conditional Docker file generation

---

## Completed Work

### Milestone 6: TUI Integration ✅

The BackstageServer is fully wired to the TUI.

**Implementation details** (`src/app.rs`):
- Line 58: `backstage_server: BackstageServer` field in App struct
- Lines 172-173: Server initialization with `BackstageServer::with_system_client()`
- Line 287: Status refresh via `backstage_server.refresh_status()`
- Line 713: `[W]` keybinding calls `backstage_server.toggle()`
- Lines 716-717: Browser opened when server starts

---

## Future Expansion

### Current Architecture (Catalog-Only)

```
┌─────────────┐     ┌─────────────┐
│   TUI       │     │  Backstage  │
│ (ratatui)   │     │  (React)    │
└──────┬──────┘     └──────┬──────┘
       │                   │
       ▼                   ▼
  .operator/          catalog/
  .tickets/           (static YAML)
```

Backstage reads static `catalog-info.yaml` files. No live connection to TUI state.

### Option A: Minimal (Current Plan)

Complete M6 TUI integration. Backstage serves as project catalog viewer.
- **Pros**: Fastest path, lowest complexity
- **Cons**: Limited to static catalog data

### Option B: API Bridge (Future Milestone)

Add HTTP API (axum) that Backstage can query:

```rust
// New: src/api/server.rs
Router::new()
    .route("/api/tickets", get(list_tickets))
    .route("/api/agents", get(list_agents))
    .route("/api/issuetypes", get(list_issuetypes))
    .route("/ws/events", any(websocket_handler))
```

```yaml
# app-config.yaml proxy
proxy:
  '/operator-api':
    target: http://localhost:7008
```

- **Pros**: Real-time updates, shared state
- **Cons**: Significant new development

### Option C: Custom Backstage Plugins (Future)

Build TypeScript plugins for rich visualization:
- `@operator/plugin-queue` - Ticket queue dashboard
- `@operator/plugin-agents` - Agent monitoring
- `@operator/plugin-workflows` - Step visualization

---

## File Summary

### Backstage Module (`src/backstage/`)

| File | Lines | Tests | Purpose |
|------|-------|-------|---------|
| `taxonomy.toml` | 583 | — | 24 Kinds, 4 Tiers definition |
| `taxonomy.rs` | 594 | 23 | Taxonomy loading and queries |
| `analyzer.rs` | 812 | 9 | Project analysis types |
| `server.rs` | 820 | 34 | Bun server lifecycle |
| `scaffold.rs` | 852 | 18 | Backstage scaffold generator |
| `branding.rs` | 119 | 4 | Default branding assets |
| `mod.rs` | 22 | — | Module exports |
| **Total** | **3,802** | **88** | |

### Issue Types (`src/templates/`)

| File | Lines | Purpose |
|------|-------|---------|
| `assess.json` | 93 | ASSESS workflow definition |
| `assess.md` | 19 | ASSESS markdown template |
| `sync.json` | 102 | SYNC workflow definition |
| `init.json` | 106 | INIT workflow definition |
| `issuetype_schema.json` | 366 | Issue type JSON schema |
| `project_analysis.schema.json` | 503 | ASSESS output schema |

### Documentation Generator (`src/docs_gen/`)

| File | Lines | Tests | Purpose |
|------|-------|-------|---------|
| `taxonomy.rs` | 259 | 2 | Taxonomy markdown generator |
| `markdown.rs` | 148 | 5 | Markdown utilities |
| `issuetype.rs` | 297 | 2 | Issue type docs generator |
| `metadata.rs` | 377 | 5 | Metadata schema docs |
| `mod.rs` | 100 | 1 | DocGenerator trait |

---

## The 24-Kind Taxonomy

### Tier: Foundation (1-4)
| ID | Key | Name | Description | Stakeholder | Output |
|----|-----|------|-------------|-------------|--------|
| 1 | `infrastructure` | Infrastructure (IaC) | Cloud resources and network | Platform/DevOps | Cloud Environment |
| 2 | `identity-access` | Identity & Access (IAM) | Service accounts, secrets, RBAC | SDET/SecOps | Permissions/Tokens |
| 3 | `config-policy` | Config & Policy | Feature flags, environment manifests | Platform/DevOps | Runtime Behavior |
| 4 | `monorepo-meta` | Monorepo / Meta | Orchestration, root standards | Architect/Lead | Project Standards |

### Tier: Standards (5-10)
| ID | Key | Name | Description | Stakeholder | Output |
|----|-----|------|-------------|-------------|--------|
| 5 | `design-system` | Design Systems | UI components, brand tokens | Product/UX | Component Libraries |
| 6 | `software-library` | Software Libraries | Reusable internal packages | Engineering | Versioned Packages |
| 7 | `proto-sdk` | Proto / SDK | API contracts, client libraries | Engineering | Contract Libraries |
| 8 | `blueprint` | Blueprints | Scaffolding templates | Architect/Lead | Project Templates |
| 9 | `security-tooling` | Security Tooling | Scanners, audit scripts | SDET/SecOps | Security Reports |
| 10 | `compliance-audit` | Compliance / Audit | Evidence, regulatory reports | SDET/SecOps | Compliance Proofs |

### Tier: Engines (11-16)
| ID | Key | Name | Description | Stakeholder | Output |
|----|-----|------|-------------|-------------|--------|
| 11 | `ml-model` | ML / Models | Training scripts, model artifacts | Data/ML | Model Artifacts |
| 12 | `data-etl` | Data / ETL | Data transformation, SQL models | Data/ML | Clean Datasets |
| 13 | `microservice` | Microservices | Backend business logic | Engineering | Running Binaries |
| 14 | `api-gateway` | APIs / Gateways | Entry points, routing | Engineering | Network Endpoints |
| 15 | `ui-frontend` | UIs / Frontends | Web/mobile apps | Engineering | Web/Mobile Assets |
| 16 | `internal-tool` | Internal Tooling | Private internal apps | Engineering | Operational Apps |

### Tier: Ecosystem (17-24)
| ID | Key | Name | Description | Stakeholder | Output |
|----|-----|------|-------------|-------------|--------|
| 17 | `build-tool` | Build Tools | CI/CD actions, build logic | Platform/DevOps | Automated Pipelines |
| 18 | `e2e-test` | E2E Test Suites | Integration/smoke tests | SDET/SecOps | Quality Reports |
| 19 | `docs-site` | Docs Sites | Documentation, tutorials | Product/UX | Static Support Sites |
| 20 | `playbook` | Internal Playbooks | Incident response, runbooks | Platform/DevOps | Operational Guides |
| 21 | `reference-example` | Reference / Example | Best-practice implementations | Architect/Lead | Educational Code |
| 22 | `cli-devtool` | CLIs / Developer Tools | Productivity scripts | Platform/DevOps | Developer UX Tools |
| 23 | `experiment-sandbox` | Experiment / Sandbox | POCs, R&D spikes | Engineering | Discardable Code |
| 24 | `archival-fork` | Archival / Forks | Legacy code, 3rd party forks | SDET/SecOps | Historical/Vendor Code |

---

## Validation Commands

```bash
# Full validation (run before every commit)
cargo fmt && cargo clippy -- -D warnings && cargo test

# Backstage-specific tests
cargo test backstage

# Taxonomy tests only
cargo test taxonomy

# Regenerate documentation
cargo run --bin docgen
```

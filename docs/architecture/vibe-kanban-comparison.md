---
layout: page
title: Vibe-Kanban Comparison
parent: Architecture
nav_order: 1
published: false
---

# Operator vs Vibe-Kanban Architecture Comparison

This document compares [vibe-kanban](https://github.com/BloopAI/vibe-kanban) and Operator's architectural approaches to LLM agent orchestration.

## Overview

| Aspect | Vibe-Kanban | Operator |
|--------|-------------|----------|
| **Focus** | Multi-executor task management with container isolation | TUI-driven ticket workflow with tmux sessions |
| **State Storage** | Database-backed (SQLite/PostgreSQL) | File-based JSON |
| **Execution Host** | Container service (Docker) | tmux sessions (macOS/Linux) |
| **Primary UI** | Web dashboard | Terminal TUI (ratatui) |

---

## Configuration Systems

### Vibe-Kanban: Three-Layer

```
1. Built-in defaults (default_profiles.json)
2. User overrides (~/.vibe-kanban/profiles.json)
3. Runtime overrides (env vars, CLI args)
```

Profiles use hierarchical identifiers: `CLAUDE_CODE.PLAN`, `CODEX.HIGH`.

### Operator: Four-Layer

```
1. Built-in defaults (embedded in binary)
2. Project config (.tickets/operator/config.toml)
3. User config (~/.config/operator/config.toml)
4. Environment variables (OPERATOR_ prefix)
```

Providers are flat structures with optional variant fields.

---

## Profile/Variant Model

### Vibe-Kanban: Executor Variants

Each executor (CLAUDE_CODE, CODEX, CURSOR_AGENT) has named variants:

```json
{
  "executors": {
    "CLAUDE_CODE": {
      "DEFAULT": { "dangerously_skip_permissions": true },
      "PLAN": { "plan": true },
      "OPUS": { "model": "opus" },
      "APPROVALS": { "approvals": true }
    }
  }
}
```

Variant resolution: `ExecutorProfileId(executor, variant)` → HashMap lookup → fallback to DEFAULT.

### Operator: Extended LlmProvider

Flat structure with optional variant fields per provider:

```rust
pub struct LlmProvider {
    pub tool: String,              // "claude", "codex", "gemini"
    pub model: String,             // "opus", "gpt-4.1"
    pub display_name: Option<String>,

    // Variant fields (all optional)
    pub flags: Vec<String>,
    pub env: HashMap<String, String>,
    pub approvals: bool,
    pub plan_only: bool,
    pub reasoning_effort: Option<String>,  // Codex
    pub sandbox: Option<String>,           // Codex
}
```

---

## Task Execution Lifecycle

### Vibe-Kanban Hierarchy

```
Task
└── Workspace (isolated Git worktree + branch)
    └── Session (conversation context)
        └── ExecutionProcess (individual execution step)
```

**Key features:**
- Workspaces get isolated Git worktrees
- Sessions maintain conversation continuity via internal agent IDs
- ExecutionProcess tracks setup, agent run, cleanup, dev servers

### Operator Hierarchy

```
Ticket
└── Agent (tmux session)
    └── Step (plan, build, code, test, deploy)
        └── Session UUID (Claude session ID per step)
```

**Key features:**
- Steps defined declaratively in JSON templates
- Session UUIDs stored in ticket YAML frontmatter
- Review gates per step (Plan, Visual, PR)

---

## Isolation Model

### Vibe-Kanban: Containerized Worktrees

- Each task gets isolated Git worktree
- Branch pattern: `{prefix}/{short_uuid}-{sanitized_title}`
- Before/after commit states tracked per repository
- Container spawns executor via `StandardCodingAgentExecutor::spawn()`

### Operator: Per-Ticket Worktrees

- WorktreeManager creates worktrees at `~/.operator/worktrees/{project}/{ticket_id}`
- Branch pattern: `{ticket_type}/{ticket_id}-{sanitized_summary}`
- ProjectRepo.setup_script runs before agent launch
- ProjectRepo.cleanup_script runs after PR merge
- Global locking prevents race conditions during creation

---

## State Management

### Vibe-Kanban

- **Database-backed**: SQLite or PostgreSQL
- Workspace, Session, ExecutionProcess as database models
- Exit signal monitoring via ExecutorExitSignal
- Auto-commit modifications on completion

### Operator

- **File-based JSON**: `.tickets/operator/state.json`
- AgentState tracks tmux session, content hash, last activity
- SessionMonitor polls tmux every 30s
- Silence detection for "awaiting input" status

---

## Human-in-the-Loop

### Vibe-Kanban

Approval controlled via variant selection:

| Variant | Behavior |
|---------|----------|
| DEFAULT | Auto-approve all |
| APPROVALS | Require approval for modifications |
| PLAN | Present plan before execution |

### Operator

Mode + review gates:

| Mode | Ticket Types | Behavior |
|------|--------------|----------|
| Autonomous | FEAT, FIX, TASK | Auto-proceed between steps |
| Paired | SPIKE, INV | Pause for human interaction |

Review gates per step:
- **Plan**: Operator approval before proceeding
- **Visual**: Browser-based visual check
- **PR**: GitHub PR review gate

---

## Multi-LLM Support

### Vibe-Kanban

First-class executors with per-executor configuration:

| Executor | Variants |
|----------|----------|
| CLAUDE_CODE | DEFAULT, PLAN, OPUS, APPROVALS |
| CODEX | DEFAULT, HIGH, APPROVALS, MAX |
| CURSOR_AGENT | DEFAULT, SONNET_4_5, GPT_5, GROK |

### Operator

Detection-based with extended LlmProvider:

```toml
[[llm_tools.providers]]
tool = "claude"
model = "opus"
display_name = "Claude Opus"
flags = ["--dangerously-skip-permissions"]

[[llm_tools.providers]]
tool = "codex"
model = "gpt-4.1"
display_name = "Codex High"
sandbox = "danger-full-access"
reasoning_effort = "high"
```

---

## Key Differences Summary

| Feature | Vibe-Kanban | Operator |
|---------|-------------|----------|
| Variant structure | Nested HashMap | Flat LlmProvider |
| Database | SQLite/PostgreSQL | JSON files |
| Execution | Container service | tmux sessions |
| Worktree base | Per-workspace | Per-ticket |
| Session continuity | Database model | Frontmatter UUIDs |
| Notifications | N/A | macOS native |
| UI | Web dashboard | Terminal TUI |
| Config format | JSON | TOML |

---

## Patterns Adopted from Vibe-Kanban

1. **Per-task Git worktrees** for parallel development isolation
2. **Setup/cleanup scripts** per repository
3. **Variant fields** on providers (flags, env, approvals, plan_only)
4. **Branch naming convention** with type prefix
5. **Multi-LLM support** (Claude, Codex, Gemini)

---

## References

- [vibe-kanban Configuration and Profiles](https://deepwiki.com/BloopAI/vibe-kanban/3.2-configuration-and-profiles)
- [vibe-kanban Task Attempts and Execution Lifecycle](https://deepwiki.com/BloopAI/vibe-kanban/2.3-task-attempts-and-execution-lifecycle)

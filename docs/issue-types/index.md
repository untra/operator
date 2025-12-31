---
title: Issue Types
description: "Learn about INV, FIX, FEAT, SPIKE, and TASK issue types, custom definitions, and Jira imports."
layout: doc
---

<span class="operator-brand">Operator!</span> supports five built-in issue types, organized into collections for different workflows. You can also define custom issue types and import from external kanban systems.

## Built-in Issue Types

### INV - Investigation

**Priority:** 1 (highest)
**Mode:** Paired

Investigation tickets are for diagnosing failures, understanding bugs, or exploring issues. They require human interaction and are worked on with operator pairing.

```
INV-001-project-investigate-login-failure.md
```

### FIX - Bug Fix

**Priority:** 2
**Mode:** Autonomous

Bug fixes are addressed after investigations. Agents can work autonomously once the problem is understood.

```
FIX-042-project-fix-null-pointer-exception.md
```

### FEAT - Feature

**Priority:** 3
**Mode:** Autonomous

New features are implemented after critical bugs are addressed. Agents can work autonomously with clear requirements.

```
FEAT-123-project-add-dark-mode.md
```

### SPIKE - Research

**Priority:** 4
**Mode:** Paired

Spikes are for research, exploration, and proof-of-concept work. They require human interaction and discussion.

```
SPIKE-007-project-evaluate-new-framework.md
```

### TASK - General Task

**Priority:** 5 (lowest)
**Mode:** Autonomous

Tasks are general work items that don't fit other categories. Used in simple workflows or as a catch-all.

```
TASK-099-project-update-dependencies.md
```

## Collections

Collections are named groupings of issue types that define which types are available and their priority order. This allows teams to customize their workflow.

### Built-in Presets

| Preset | Issue Types | Priority Order |
|--------|-------------|----------------|
| `simple` | TASK | TASK |
| `dev_kanban` | TASK, FEAT, FIX | FIX, FEAT, TASK |
| `devops_kanban` | TASK, SPIKE, INV, FEAT, FIX | INV, FIX, FEAT, SPIKE, TASK |

The default collection is `devops_kanban`.

### Using Collections

Collections can be activated via configuration:

```toml
# config.toml
[templates]
active_collection = "dev_kanban"
```

Or create a custom collection in `.tickets/operator/issuetypes/collections.toml`:

```toml
[collections.agile]
name = "agile"
description = "Agile development workflow"
types = ["STORY", "BUG", "TASK", "SPIKE"]
priority_order = ["BUG", "STORY", "TASK", "SPIKE"]
```

## Custom Issue Types

Define custom issue types in `.tickets/operator/issuetypes/`:

```
.tickets/operator/issuetypes/
  STORY.json           # User-defined type
  BUG.json             # User-defined type
  collections.toml     # Collection definitions
  imports/             # Imported types from Jira
```

### Issue Type Schema

```json
{
  "key": "STORY",
  "name": "User Story",
  "description": "A user-facing feature from the user's perspective",
  "mode": "autonomous",
  "glyph": "S",
  "color": "cyan",
  "project_required": true,
  "fields": [
    {"name": "id", "type": "string", "required": true, "auto": "id"},
    {"name": "summary", "type": "string", "required": true, "default": ""},
    {"name": "acceptance_criteria", "type": "string", "required": false}
  ],
  "steps": [
    {
      "name": "plan",
      "outputs": ["acceptance_criteria"],
      "prompt": "Create a plan for this user story",
      "allowed_tools": ["Read", "Grep", "Glob"]
    },
    {
      "name": "implement",
      "outputs": ["code"],
      "prompt": "Implement the user story",
      "allowed_tools": ["*"]
    }
  ]
}
```

## Importing from Kanban Systems

Import issue types from Jira to use their type definitions locally.

### Environment Variables

```bash
# Jira
export OPERATOR_JIRA_DOMAIN=your-domain.atlassian.net
export OPERATOR_JIRA_EMAIL=you@example.com
export OPERATOR_JIRA_TOKEN=your-api-token
```

### Imported Type Structure

Imported types have fields from the external system but use a single default step:

```json
{
  "key": "STORY",
  "name": "Story",
  "description": "Imported from Jira",
  "mode": "autonomous",
  "glyph": "S",
  "fields": [
    {"name": "id", "type": "string", "required": true, "auto": "id"},
    {"name": "summary", "type": "string", "required": true, "default": ""}
  ],
  "steps": [
    {"name": "execute", "outputs": [], "prompt": "Execute this task", "allowed_tools": ["*"]}
  ],
  "source": {"import": {"provider": "jira", "project": "MYPROJECT"}},
  "external_id": "10001"
}
```

Imported types are stored in:
```
.tickets/operator/issuetypes/imports/{provider}/{project}/
  Story.json
  Bug.json
```

## Agent Modes

### Autonomous Mode (FEAT, FIX, TASK)

- Launch and monitor progress
- Minimal human intervention
- Can run multiple agents in parallel

### Paired Mode (SPIKE, INV)

- Requires active human participation
- Tracks "awaiting input" states
- One paired agent at a time per operator

## Step Permissions

Each step in an issue type can define permissions that control the LLM agent's access to tools, directories, and MCP servers. Permissions are provider-agnostic and translated to the appropriate format for Claude, Gemini, or Codex at runtime.

### Permission Fields

Steps can include these optional permission fields:

```json
{
  "name": "build",
  "outputs": ["code"],
  "prompt": "Implement the feature...",
  "allowed_tools": ["Read", "Write", "Edit", "Bash"],
  "permissions": {
    "tools": {
      "allow": [
        { "tool": "Bash", "pattern": "cargo:*" },
        { "tool": "Bash", "pattern": "npm:*" }
      ],
      "deny": [
        { "tool": "Bash", "pattern": "rm -rf:*" },
        { "tool": "Bash", "pattern": "sudo:*" }
      ]
    },
    "directories": {
      "allow": ["../shared-libs/"],
      "deny": ["./.env", "./secrets/"]
    },
    "mcp_servers": {
      "enable": ["memory"],
      "disable": ["filesystem"]
    }
  },
  "cli_args": {
    "claude": ["--output-format", "json"],
    "gemini": ["--sandbox", "docker"],
    "codex": ["--approval-policy", "on-failure"]
  }
}
```

### Permission Composition

Step permissions are **additive** with project-level permissions:

1. Project permissions are loaded from `.operator/permissions.json`
2. Step permissions are added to project permissions
3. Both allow and deny lists are concatenated
4. Custom flags: step values override project values for the same key

### Provider Translation

Permissions are automatically translated to each provider's format:

| Provider | Config Format | Tool Syntax |
|----------|--------------|-------------|
| Claude | CLI flags | `--allowedTools "Bash(cargo:*)"` |
| Gemini | `.gemini/settings.json` | `"ShellTool(cargo:*)"` |
| Codex | `.codex/config.toml` | `[tools.exec].allow_patterns` |

### Session Config Persistence

All generated configs are stored for auditing at:
```
.tickets/operator/sessions/{ticket-id}/
  claude-audit.txt
  settings.json        # Gemini config
  config.toml          # Codex config
  launch-command.txt   # Full command used
```

## Validation

When loading collections, <span class="operator-brand">Operator!</span> validates that all referenced issue types exist. Missing types are logged as warnings and skipped:

```
WARN: Collection 'agile' references unknown type 'STORY', skipping
```

This allows collections to reference types that may not yet be defined, enabling gradual adoption.

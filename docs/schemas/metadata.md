---
title: "Ticket Metadata Schema"
layout: doc
---

<!-- AUTO-GENERATED FROM src/schemas/ticket_metadata.schema.json - DO NOT EDIT MANUALLY -->
<!-- Regenerate with: cargo run -- docs -->

# Ticket Metadata Schema

Schema for operator-tracked ticket metadata in YAML frontmatter. This schema documents the structure of ticket files used by the operator TUI.

## Schema Information

- **$schema**: `http://json-schema.org/draft-07/schema#`
- **$id**: `https://operator.untra.io/schemas/ticket_metadata.schema.json`
- **Additional Properties**: Allowed

## Required Fields

- `id`
- `status`

## Properties

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `id` | `string` | Yes | Kanban ticket ID (e.g., FEAT-1234). Also used for tmux session name derivation. |
| `status` | `string` | Yes | Operator workflow status |
| `step` | `string` | No | Current workflow step name (e.g., plan, build, code, test, deploy) |
| `priority` | `string` | No | Ticket priority level |
| `project` | `string` | No | Target project name (subdirectory in projects root) |
| `branch` | `string` | No | Git branch name for this ticket (auto-generated from type and summary) |
| `worktree_path` | `string` | No | Filesystem path to the git worktree for this ticket (per-ticket isolation) |
| `external_id` | `string` | No | External issue ID from the kanban provider (e.g., PROJ-123 for Jira, ENG-456 for Linear) |
| `external_url` | `string` (uri) | No | Full URL to the issue in the external provider's web UI |
| `external_provider` | `string` | No | Provider name for the external issue (e.g., jira, linear) |
| `step_delegators` | `object` | No | Step name to delegator name mapping. Populated when a step launches; used for bidirectional kanban activity logs. |
| `sessions` | `object` | No | Step name to LLM session UUID mapping. Each step gets its own session ID for continuity. |
| `llm_task` | `object` | No | LLM task metadata for delegate mode integration |

### id

- **Description**: Kanban ticket ID (e.g., FEAT-1234). Also used for tmux session name derivation.
- **Type**: `string`
- **Pattern**: `^[A-Z]+-\d+$`
- **Examples**: `FEAT-1234`, `FIX-5678`, `SPIKE-0001`, `INV-0042`, `TASK-9999`

### status

- **Description**: Operator workflow status
- **Type**: `string`
- **Default**: `"queued"`
- **Allowed Values**: `queued`, `running`, `awaiting`, `completed`

### step

- **Description**: Current workflow step name (e.g., plan, build, code, test, deploy)
- **Type**: `string`
- **Examples**: `plan`, `build`, `code`, `test`, `deploy`, `explore`, `summarize`

### priority

- **Description**: Ticket priority level
- **Type**: `string`
- **Default**: `"P2-medium"`
- **Allowed Values**: `P0-critical`, `P1-high`, `P2-medium`, `P3-low`

### project

- **Description**: Target project name (subdirectory in projects root)
- **Type**: `string`
- **Examples**: `gamesvc`, `operator`, `www`, `iac`

### branch

- **Description**: Git branch name for this ticket (auto-generated from type and summary)
- **Type**: `string`
- **Examples**: `feature/FEAT-1234-add-user-auth`, `fix/FIX-5678-login-timeout`

### worktree_path

- **Description**: Filesystem path to the git worktree for this ticket (per-ticket isolation)
- **Type**: `string`
- **Examples**: `/Users/dev/worktrees/op-FEAT-1234`

### external_id

- **Description**: External issue ID from the kanban provider (e.g., PROJ-123 for Jira, ENG-456 for Linear)
- **Type**: `string`
- **Examples**: `PROJ-123`, `ENG-456`

### external_url

- **Description**: Full URL to the issue in the external provider's web UI
- **Type**: `string` (uri)
- **Format**: `uri`
- **Examples**: `https://example.atlassian.net/browse/PROJ-123`

### external_provider

- **Description**: Provider name for the external issue (e.g., jira, linear)
- **Type**: `string`
- **Examples**: `jira`, `linear`

### step_delegators

- **Description**: Step name to delegator name mapping. Populated when a step launches; used for bidirectional kanban activity logs.
- **Type**: `object`
- **Examples**: `{"plan":"claude","build":"claude-code"}`

**Additional Properties**: `string` (Delegator name used for this step)

### sessions

- **Description**: Step name to LLM session UUID mapping. Each step gets its own session ID for continuity.
- **Type**: `object`
- **Examples**: `{"plan":"550e8400-e29b-41d4-a716-446655440000","build":"6ba7b810-9dad-11d1-80b4-00c04fd430c8"}`

**Additional Properties**: `string` (uuid) (UUID for the LLM session at this step)

### llm_task

- **Description**: LLM task metadata for delegate mode integration
- **Type**: `object`

**Object** Nested Properties:

| Property | Type | Description |
| --- | --- | --- |
| `id` | `string` (uuid) | LLM task ID (e.g., Claude delegate mode task UUID) |
| `status` | `string` | LLM task status |
| `blocked_by` | `array` | List of task IDs that must resolve before this task can proceed |

## Definitions

### Definition: id_derivations

The ticket ID is used to derive other identifiers:

| Property | Description |
| --- | --- |
| `tmux_session_name` | Derived as: op-{id} (e.g., op-FEAT-1234) |
| `git_branch` | Derived as: {key.lowercase}/{id}-{summary-slug} (e.g., feat/FEAT-1234-add-auth) |

## Examples

Complete ticket metadata examples:

### Example 1

```yaml
{
  "id": "FEAT-1234",
  "status": "running",
  "step": "build",
  "priority": "P2-medium",
  "project": "gamesvc",
  "branch": "feature/FEAT-1234-add-user-auth",
  "worktree_path": "/Users/dev/worktrees/op-FEAT-1234",
  "external_id": "PROJ-123",
  "external_url": "https://example.atlassian.net/browse/PROJ-123",
  "external_provider": "jira",
  "sessions": {
    "plan": "550e8400-e29b-41d4-a716-446655440000",
    "build": "6ba7b810-9dad-11d1-80b4-00c04fd430c8"
  },
  "step_delegators": {
    "plan": "claude",
    "build": "claude-code"
  }
}
```

### Example 2

```yaml
{
  "id": "SPIKE-0042",
  "status": "completed",
  "step": "summarize",
  "priority": "P2-medium",
  "project": "operator",
  "sessions": {
    "explore": "abc12345-6789-0abc-def0-123456789abc"
  },
  "llm_task": {
    "id": "def67890-1234-5678-9abc-def012345678",
    "status": "resolved",
    "blocked_by": []
  }
}
```


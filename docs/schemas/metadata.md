---
title: "Ticket Metadata Schema"
layout: doc
---

<!-- AUTO-GENERATED FROM src/templates/ticket_metadata.schema.json - DO NOT EDIT MANUALLY -->
<!-- Regenerate with: cargo run -- docs -->

# Ticket Metadata Schema

Schema for operator-tracked ticket metadata in YAML frontmatter. This schema documents the structure of ticket files used by the operator TUI.

## Schema Information

- **$schema**: `http://json-schema.org/draft-07/schema#`
- **$id**: `https://gbqr.us/operator/ticket-metadata.schema.json`
- **Additional Properties**: Allowed

## Required Fields

- `id`
- `status`

## Properties

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `branch` | `string` | No | Git branch name for this ticket (auto-generated from type and summary) |
| `completedDatetime` | `string` (date-time) | No | ISO 8601 datetime when the ticket was completed |
| `created` | `string` (date) | No | Creation date in YYYY-MM-DD format (legacy, prefer createdDatetime) |
| `createdDatetime` | `string` (date-time) | No | ISO 8601 datetime when the ticket was created |
| `id` | `string` | Yes | Kanban ticket ID (e.g., FEAT-1234). Also used for tmux session name derivation. |
| `llm_task` | `object` | No | LLM task metadata for delegate mode integration |
| `priority` | `string` | No | Ticket priority level |
| `project` | `string` | No | Target project name (subdirectory in projects root) |
| `sessions` | `object` | No | Step name to LLM session UUID mapping. Each step gets its own session ID for continuity. |
| `startedDatetime` | `string` (date-time) | No | ISO 8601 datetime when work began on the ticket (moved to running status) |
| `status` | `string` | Yes | Operator workflow status |
| `step` | `string` | No | Current workflow step name (e.g., plan, build, code, test, deploy) |

### branch

- **Description**: Git branch name for this ticket (auto-generated from type and summary)
- **Type**: `string`
- **Examples**: `feature/FEAT-1234-add-user-auth`, `fix/FIX-5678-login-timeout`

### completedDatetime

- **Description**: ISO 8601 datetime when the ticket was completed
- **Type**: `string` (date-time)
- **Format**: `date-time`
- **Examples**: `2024-12-25T17:30:00Z`

### created

- **Description**: Creation date in YYYY-MM-DD format (legacy, prefer createdDatetime)
- **Type**: `string` (date)
- **Format**: `date`

### createdDatetime

- **Description**: ISO 8601 datetime when the ticket was created
- **Type**: `string` (date-time)
- **Format**: `date-time`
- **Examples**: `2024-12-25T14:30:00Z`, `2024-12-25T09:15:00-05:00`

### id

- **Description**: Kanban ticket ID (e.g., FEAT-1234). Also used for tmux session name derivation.
- **Type**: `string`
- **Pattern**: `^[A-Z]+-\d+$`
- **Examples**: `FEAT-1234`, `FIX-5678`, `SPIKE-0001`, `INV-0042`, `TASK-9999`

### llm_task

- **Description**: LLM task metadata for delegate mode integration
- **Type**: `object`

**Object** Nested Properties:

| Property | Type | Description |
| --- | --- | --- |
| `blocked_by` | `array` | List of task IDs that must resolve before this task can proceed |
| `id` | `string` (uuid) | LLM task ID (e.g., Claude delegate mode task UUID) |
| `status` | `string` | LLM task status |

### priority

- **Description**: Ticket priority level
- **Type**: `string`
- **Default**: `"P2-medium"`
- **Allowed Values**: `P0-critical`, `P1-high`, `P2-medium`, `P3-low`

### project

- **Description**: Target project name (subdirectory in projects root)
- **Type**: `string`
- **Examples**: `gamesvc`, `operator`, `www`, `iac`

### sessions

- **Description**: Step name to LLM session UUID mapping. Each step gets its own session ID for continuity.
- **Type**: `object`
- **Examples**: `{"build":"6ba7b810-9dad-11d1-80b4-00c04fd430c8","plan":"550e8400-e29b-41d4-a716-446655440000"}`

**Additional Properties**: `string` (uuid) (UUID for the LLM session at this step)

### startedDatetime

- **Description**: ISO 8601 datetime when work began on the ticket (moved to running status)
- **Type**: `string` (date-time)
- **Format**: `date-time`
- **Examples**: `2024-12-25T15:00:00Z`

### status

- **Description**: Operator workflow status
- **Type**: `string`
- **Default**: `"queued"`
- **Allowed Values**: `queued`, `running`, `awaiting`, `completed`

### step

- **Description**: Current workflow step name (e.g., plan, build, code, test, deploy)
- **Type**: `string`
- **Examples**: `plan`, `build`, `code`, `test`, `deploy`, `explore`, `summarize`

## Definitions

### Definition: id_derivations

The ticket ID is used to derive other identifiers:

| Property | Description |
| --- | --- |
| `git_branch` | Derived as: {branch_prefix}/{id}-{summary-slug} (e.g., feature/FEAT-1234-add-auth) |
| `tmux_session_name` | Derived as: op-{id} (e.g., op-FEAT-1234) |

## Examples

Complete ticket metadata examples:

### Example 1

```yaml
{
  "branch": "feature/FEAT-1234-add-user-auth",
  "createdDatetime": "2024-12-25T14:30:00Z",
  "id": "FEAT-1234",
  "priority": "P2-medium",
  "project": "gamesvc",
  "sessions": {
    "build": "6ba7b810-9dad-11d1-80b4-00c04fd430c8",
    "plan": "550e8400-e29b-41d4-a716-446655440000"
  },
  "startedDatetime": "2024-12-25T15:00:00Z",
  "status": "running",
  "step": "build"
}
```

### Example 2

```yaml
{
  "completedDatetime": "2024-12-25T12:30:00Z",
  "createdDatetime": "2024-12-25T09:00:00Z",
  "id": "SPIKE-0042",
  "llm_task": {
    "blocked_by": [],
    "id": "def67890-1234-5678-9abc-def012345678",
    "status": "resolved"
  },
  "priority": "P2-medium",
  "project": "operator",
  "sessions": {
    "explore": "abc12345-6789-0abc-def0-123456789abc"
  },
  "startedDatetime": "2024-12-25T10:00:00Z",
  "status": "completed",
  "step": "summarize"
}
```


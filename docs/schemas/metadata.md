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
| `id` | `string` | Yes | Kanban ticket ID (e.g., FEAT-1234). Also used for tmux session name derivation. |
| `status` | `string` | Yes | Operator workflow status |
| `step` | `string` | No | Current workflow step name (e.g., plan, build, code, test, deploy) |
| `priority` | `string` | No | Ticket priority level |
| `project` | `string` | No | Target project name (subdirectory in projects root) |
| `created` | `string` (date) | No | Creation date in YYYY-MM-DD format (legacy, prefer createdDatetime) |
| `createdDatetime` | `string` (date-time) | No | ISO 8601 datetime when the ticket was created |
| `startedDatetime` | `string` (date-time) | No | ISO 8601 datetime when work began on the ticket (moved to running status) |
| `completedDatetime` | `string` (date-time) | No | ISO 8601 datetime when the ticket was completed |
| `branch` | `string` | No | Git branch name for this ticket (auto-generated from type and summary) |
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

### created

- **Description**: Creation date in YYYY-MM-DD format (legacy, prefer createdDatetime)
- **Type**: `string` (date)
- **Format**: `date`

### createdDatetime

- **Description**: ISO 8601 datetime when the ticket was created
- **Type**: `string` (date-time)
- **Format**: `date-time`
- **Examples**: `2024-12-25T14:30:00Z`, `2024-12-25T09:15:00-05:00`

### startedDatetime

- **Description**: ISO 8601 datetime when work began on the ticket (moved to running status)
- **Type**: `string` (date-time)
- **Format**: `date-time`
- **Examples**: `2024-12-25T15:00:00Z`

### completedDatetime

- **Description**: ISO 8601 datetime when the ticket was completed
- **Type**: `string` (date-time)
- **Format**: `date-time`
- **Examples**: `2024-12-25T17:30:00Z`

### branch

- **Description**: Git branch name for this ticket (auto-generated from type and summary)
- **Type**: `string`
- **Examples**: `feature/FEAT-1234-add-user-auth`, `fix/FIX-5678-login-timeout`

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
  "createdDatetime": "2024-12-25T14:30:00Z",
  "startedDatetime": "2024-12-25T15:00:00Z",
  "branch": "feature/FEAT-1234-add-user-auth",
  "sessions": {
    "plan": "550e8400-e29b-41d4-a716-446655440000",
    "build": "6ba7b810-9dad-11d1-80b4-00c04fd430c8"
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
  "createdDatetime": "2024-12-25T09:00:00Z",
  "startedDatetime": "2024-12-25T10:00:00Z",
  "completedDatetime": "2024-12-25T12:30:00Z",
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


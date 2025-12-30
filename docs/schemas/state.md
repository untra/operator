---
title: "Application State Schema"
layout: doc
---

<!-- AUTO-GENERATED FROM docs/schemas/state.json - DO NOT EDIT MANUALLY -->
<!-- Regenerate with: cargo run -- docs -->

# Application State Schema

JSON Schema for the Operator runtime state file (`state.json`).

This file tracks the current state of agents, completed tickets, and system status.

## Schema Information

- **$schema**: `https://json-schema.org/draft/2020-12/schema`
- **title**: `State`

## Required Fields

- `paused`
- `agents`
- `completed`

## Properties

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `paused` | `boolean` | Yes | Whether agent processing is paused |
| `agents` | `array` | Yes | Currently active agents |
| `completed` | `array` | Yes | Recently completed tickets |

## Type Definitions

### AgentState

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `id` | `string` | Yes |  |
| `ticket_id` | `string` | Yes |  |
| `ticket_type` | `string` | Yes |  |
| `project` | `string` | Yes |  |
| `status` | `string` | Yes |  |
| `started_at` | `string` | Yes |  |
| `last_activity` | `string` | Yes |  |
| `last_message` | `string` \| `null` | No |  |
| `paired` | `boolean` | Yes |  |
| `session_name` | `string` \| `null` | No | The tmux session name for this agent (for recovery) |
| `content_hash` | `string` \| `null` | No | Hash of the last captured pane content (for change detection) |
| `current_step` | `string` \| `null` | No | Current step in the ticket workflow (e.g., "plan", "implement", "test") |
| `step_started_at` | `string` \| `null` | No | When the current step started (for timeout detection) |
| `last_content_change` | `string` \| `null` | No | Last time content changed in the session (for hung detection) |
| `pr_url` | `string` \| `null` | No | PR URL if created during "pr" step |
| `pr_number` | `integer` \| `null` | No | PR number for GitHub API tracking |
| `github_repo` | `string` \| `null` | No | GitHub repo in format "owner/repo" |
| `pr_status` | `string` \| `null` | No | Last known PR status ("open", "approved", "changes_requested", "merged", "closed") |
| `completed_steps` | `array` | No | Completed steps for this ticket |
| `llm_tool` | `string` \| `null` | No | LLM tool used (e.g., "claude", "gemini", "codex") |
| `launch_mode` | `string` \| `null` | No | Launch mode: "default", "yolo", "docker", "docker-yolo" |

### CompletedTicket

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `ticket_id` | `string` | Yes |  |
| `ticket_type` | `string` | Yes |  |
| `project` | `string` | Yes |  |
| `summary` | `string` | Yes |  |
| `completed_at` | `string` | Yes |  |
| `pr_url` | `string` \| `null` | No |  |
| `output_tickets` | `array` | Yes |  |


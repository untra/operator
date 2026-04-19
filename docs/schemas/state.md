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
| `project_llm_stats` | `object` | No | Per-project LLM usage statistics |
| `project_collection_prefs` | `object` | No | Per-project issue type collection preferences (`project_name` -> `collection_name`) |
| `multi_agent_groups` | `array` | No | Active multi-agent step groups (`multi_model`, `multi_prompt`, `matrixed`) |

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
| `session_name` | `string` \| `null` | No | The terminal session name for this agent (for recovery) |
| `session_wrapper` | `string` \| `null` | No | Which session wrapper manages this agent: "tmux", "vscode", or "cmux" (None = legacy tmux) |
| `session_window_ref` | `string` \| `null` | No | Session window reference ID (top-level grouping: cmux window, tmux session, etc.) |
| `session_context_ref` | `string` \| `null` | No | Session context reference ID (mid-level: cmux workspace, tmux window, etc.) |
| `session_pane_ref` | `string` \| `null` | No | Session pane reference ID (leaf-level: cmux surface, tmux pane, etc.) |
| `content_hash` | `string` \| `null` | No | Hash of the last captured pane content (for change detection) |
| `current_step` | `string` \| `null` | No | Current step in the ticket workflow (e.g., "plan", "implement", "test") |
| `step_started_at` | `string` \| `null` | No | When the current step started (for timeout detection) |
| `last_content_change` | `string` \| `null` | No | Last time content changed in the session (for hung detection) |
| `pr_url` | `string` \| `null` | No | PR URL if created during "pr" step |
| `pr_number` | `integer` \| `null` | No | PR number for GitHub API tracking |
| `github_repo` | `string` \| `null` | No | GitHub repo in format "owner/repo" |
| `pr_status` | `string` \| `null` | No | Last known PR status ("open", "approved", "`changes_requested`", "merged", "closed") |
| `completed_steps` | `array` | No | Completed steps for this ticket |
| `llm_tool` | `string` \| `null` | No | LLM tool used (e.g., "claude", "gemini", "codex") |
| `llm_model` | `string` \| `null` | No | LLM model alias (e.g., "opus", "sonnet", "gpt-4o") |
| `launch_mode` | `string` \| `null` | No | Launch mode: "default", "yolo", "docker", "docker-yolo" |
| `review_state` | `string` \| `null` | No | Review state for `awaiting_input` agents Values: "`pending_plan`", "`pending_visual`", "`pending_pr_creation`", "`pending_pr_merge`" |
| `dev_server_pid` | `integer` \| `null` | No | Server process ID for visual review cleanup (if applicable) |
| `worktree_path` | `string` \| `null` | No | Path to the git worktree for this ticket (per-ticket isolation) |

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

### ProjectLlmStats

LLM usage statistics for a project

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `project` | `string` | Yes | Project name |
| `preferred_tool` | `string` \| `null` | No | Preferred LLM tool for this project (user override) |
| `preferred_model` | `string` \| `null` | No | Preferred model for this project (user override) |
| `tool_usage` | `object` | No | Usage history per LLM tool |
| `updated_at` | `string` | Yes | Last updated timestamp |

### LlmToolUsage

Usage statistics for a specific LLM tool

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `tool` | `string` | Yes | Tool name (e.g., "claude", "gemini") |
| `ticket_count` | `integer` | No | Total number of tickets processed |
| `success_count` | `integer` | No | Number of successful completions |
| `failure_count` | `integer` | No | Number of failures/abandonments |
| `total_time_secs` | `integer` | No | Total time spent (in seconds) |
| `last_used` | `string` | Yes | Last used timestamp |
| `model_usage` | `object` | No | Per-model breakdown |

### LlmModelUsage

Usage statistics for a specific model

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `model` | `string` | Yes | Model name/alias |
| `ticket_count` | `integer` | No | Number of tickets |
| `success_count` | `integer` | No | Success count |
| `failure_count` | `integer` | No | Failure count |
| `total_time_secs` | `integer` | No | Total time (seconds) |

### MultiAgentGroup

Tracks a group of agents working on a single multi-agent step

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `group_id` | `string` | Yes | Unique group identifier |
| `ticket_id` | `string` | Yes | Ticket this group belongs to |
| `step_name` | `string` | Yes | Step name being executed |
| `step_type` | `string` | Yes | Step type (`multi_model`, `multi_prompt`, `matrixed`) |
| `agent_ids` | `array` | Yes | Agent IDs in this group (populated as sub-agents launch) |
| `phase` | → `MultiAgentPhase` | Yes | Current execution phase |
| `individual_outputs` | `object` | No | Collected outputs from completed sub-agents, keyed by `variant_key` (delegator name for `multi_model`, index for `multi_prompt`, `{delegator}:{prompt_idx}` for `matrixed`). |
| `aggregated_output` | object | No | Final aggregated output (set when phase = Complete) |
| `expected_total` | `integer` | No | Total sub-agents expected (`agent_ids.len() + pending_launches.len()`). |
| `pending_launches` | `array` | No | Sub-agents that still need launching (waiting for a free slot). |
| `agent_variant_keys` | `object` | No | Maps launched `agent_id` to the `variant_key` used as the output key. |

### MultiAgentPhase

Execution phase for a multi-agent group

### PendingSubAgent

A sub-agent that has been planned but not yet launched (slot queue).

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `delegator_name` | `string` | Yes | Delegator (from `config.delegators`) this sub-agent should use. |
| `prompt` | `string` | Yes | Fully-rendered prompt text for this sub-agent. |
| `variant_key` | `string` | Yes | Key under which this sub-agent's output is recorded (see `individual_outputs`). |


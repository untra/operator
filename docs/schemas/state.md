---
title: "Application State Schema"
layout: doc
---

<!-- AUTO-GENERATED FROM docs/schemas/state.json - DO NOT EDIT MANUALLY -->
<!-- Regenerate with: cargo run -- docs -->

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
| `campaign_runs` | `object` | No | Campaign id → runner bookkeeping |

## Type Definitions

### AgentState

One agent working a ticket, persisted in the state file.

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `git_context` | object | No | Non-secret Git configuration captured at launch. |
| `id` | `string` | Yes | Agent ID |
| `ticket_id` | `string` | Yes | Ticket the agent works (e.g. "FEAT-7598") |
| `ticket_type` | `string` | Yes | Ticket type: FEAT, FIX, INV, SPIKE |
| `project` | `string` | Yes | Project the ticket belongs to |
| `status` | `string` | Yes | "running", "`awaiting_input`", "completing", "orphaned", or "failed" after an interrupted local shutdown |
| `started_at` | `string` | Yes | When the agent launched |
| `last_activity` | `string` | Yes | Last time the agent showed activity |
| `last_message` | `string` \| `null` | No | Latest status message from the agent |
| `paired` | `boolean` | Yes | Whether the operator works alongside the agent (true for SPIKE and INV) |
| `session_name` | `string` \| `null` | No | The terminal session name for this agent (for recovery) |
| `session_wrapper` | `string` \| `null` | No | Which session wrapper manages this agent: "tmux", "vscode", or "cmux" (None = legacy tmux) |
| `session_window_ref` | `string` \| `null` | No | Session window reference ID (top-level grouping: cmux window, tmux session, etc.) |
| `session_context_ref` | `string` \| `null` | No | Session context reference ID (mid-level: cmux workspace, tmux window, etc.) |
| `session_pane_ref` | `string` \| `null` | No | Session pane reference ID (leaf-level: cmux surface, tmux pane, etc.) |
| `content_hash` | `string` \| `null` | No | Hash of the last captured pane content (for change detection) |
| `current_step` | `string` \| `null` | No | Current step in the ticket workflow (e.g., "plan", "implement", "test") |
| `step_started_at` | `string` \| `null` | No | When the current step started (for timeout detection) |
| `last_content_change` | `string` \| `null` | No | Last time content changed in the session (for hung detection) |
| `pr_url` | `string` \| `null` | No | PR/MR URL if created during the "pr" step |
| `pr_number` | `integer` \| `null` | No | Code review request (PR/MR) number |
| `repo` | `string` \| `null` | No | Repository in "owner/repo" format on the configured git provider |
| `pr_status` | `string` \| `null` | No | Last known PR/MR status ("open", "approved", "`changes_requested`", "merged", "closed") |
| `completed_steps` | `array` | No | Completed steps for this ticket |
| `llm_tool` | `string` \| `null` | No | LLM tool used (e.g., "claude", "gemini", "codex") |
| `llm_model` | `string` \| `null` | No | LLM model alias (e.g., "opus", "sonnet", "gpt-4o") |
| `launch_mode` | `string` \| `null` | No | Launch mode: `default|yolo|docker[-yolo]|coder[-yolo]|ssh[-yolo]` |
| `review_state` | `string` \| `null` | No | Review state for `awaiting_input` agents Values: "`pending_plan`", "`pending_visual`", "`pending_proof`", "`pending_pr_creation`", "`pending_pr_merge`" |
| `dev_server_pid` | `integer` \| `null` | No | Server process ID for visual review cleanup (if applicable) |
| `worktree_path` | `string` \| `null` | No | Path to the git worktree for this ticket (per-ticket isolation) |
| `remote_host` | `string` \| `null` | No | Name of the `RemoteHost` this agent's CLI runs on over SSH (None = local) |
| `step_launch_context` | object | No | Launch context fixed at launch time; `complete_step` reads it back to build subsequent step commands with the same delegator/tool/model. |
| `target_name` | `string` \| `null` | No | Name of the resolved execution target this agent launched on. |
| `workspace_url` | `string` \| `null` | No | Coder dashboard URL of the workspace this agent runs in |
| `shutdown_recovery` | object | No | Shutdown recovery strategy. |

### GitExecutionConfig

Git settings owned by a named delegator.

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `identity` | object | No | Commit identity; falls back to the global `git.identity` |
| `credentials` | object | No | HTTPS credential for the ticket's repository |
| `settings` | `array` | No | Extra `git config` entries; `credential.*`, `url.*`, `http.*`, `include*`, `user.name`, `user.email`, `core.askpass` and `core.sshCommand` are rejected |

### GitIdentityConfig

Commit identity template for delegated work.

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `name` | `string` | Yes | Author and committer name; may use `{ticket_id}`, `{project}` and `{ticket_type}` |
| `email` | `string` | Yes | Author and committer email; may use `{ticket_id}`, `{project}` and `{ticket_type}` |

### GitCredentialConfig

Supplied HTTPS credential, bound to a repository; contains no secret value.

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `repository_url` | `string` | Yes | HTTPS repository URL the credential is bound to, with no embedded credentials, query or fragment |
| `username` | `string` | Yes | HTTPS username sent with the token |
| `token_env` | `string` | Yes | Environment variable holding the token, read at launch |

### GitConfigEntry

One `git config` entry applied to delegated work.

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `key` | `string` | Yes | Dotted config key (e.g. "commit.gpgsign"); identity and authentication keys are rejected |
| `value` | `string` | Yes | Config value |

### StepLaunchContext

Launch context fixed at launch time, persisted with the agent record, and
read back by `complete_step` to build subsequent step commands.

The persisted context is the baseline for a ticket's whole chain; per-step
`agent` overrides from the step schema apply on top for that step only.

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `delegator` | `string` \| `null` | No | Delegator name used at launch (None = ad-hoc provider/model) |
| `tool` | `string` | Yes | Resolved LLM tool (e.g. "claude") |
| `model` | `string` | Yes | Resolved model alias (e.g. "sonnet") |
| `yolo` | `boolean` | Yes | YOLO (auto-accept) mode |
| `session_id` | `string` \| `null` | No | Session UUID of the step launched with this context (informational; each transition mints a fresh UUID for the next step) |
| `opr8r` | `string` | Yes | opr8r invocation for the launch environment ("opr8r" inside a container where the image ships it on PATH, an absolute path locally). Steps exec inside the same environment, so the value holds chain-wide. |
| `operator_relay` | `boolean` \| `null` | No | Relay MCP injection override from the delegator launch config |
| `extra_flags` | `array` | No | Extra CLI flags from the delegator launch config |

### ShutdownRecovery

How startup reconciles an agent that was active when operator shut down.

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
| `judge_attempt` | object | No | The in-flight LLM judge call (set when phase = Voting). Its verdict arrives as a side file keyed by `attempt_id`, never through `State`. |

### MultiAgentPhase

Execution phase for a multi-agent group

### PendingSubAgent

A sub-agent that has been planned but not yet launched (slot queue).

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `delegator_name` | `string` | Yes | Delegator (from `config.delegators`) this sub-agent should use. |
| `prompt` | `string` | Yes | Fully-rendered prompt text for this sub-agent. |
| `variant_key` | `string` | Yes | Key under which this sub-agent's output is recorded (see `individual_outputs`). |

### JudgeAttempt

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `attempt_id` | `string` | Yes |  |
| `started_at` | `string` | Yes |  |
| `timeout_secs` | `integer` | Yes | Judge timeout copied at start, so a config edit can't strand the attempt |

### CampaignRun

Runner bookkeeping for one campaign.

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `launches` | `object` | No | Ticket id → launches made by the runner |
| `started_at` | `string` \| `null` | No |  |
| `last_error` | `string` \| `null` | No |  |
| `pause_reason` | `string` \| `null` | No | Why the run paused, if it did |


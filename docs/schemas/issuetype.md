---
title: "Issue Type Schema"
layout: doc
---

<!-- AUTO-GENERATED FROM src/templates/schema.rs (TemplateSchema) - DO NOT EDIT MANUALLY -->
<!-- Regenerate with: cargo run -- docs -->

# Issue Type Schema

Schema definition for an issuetype template

## Schema Information

- **$schema**: `https://json-schema.org/draft/2020-12/schema`
- **$id**: `N/A`

## Required Fields

- `key`
- `name`
- `description`
- `mode`
- `glyph`
- `fields`
- `steps`

## Properties

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `key` | `string` | Yes | Unique issuetype key (e.g., FEAT, FIX, SPIKE, INV, TASK) |
| `name` | `string` | Yes | Display name of the template type |
| `description` | `string` | Yes | Brief description of when to use this template |
| `mode` | → `ExecutionMode` | Yes | Whether this issuetype runs autonomously or requires human pairing |
| `glyph` | `string` | Yes | Glyph character displayed in UI for this issuetype |
| `color` | `string` \| `null` | No | Optional color for glyph display in TUI |
| `project_required` | `boolean` | No | Whether a project must be specified for this issuetype |
| `fields` | `array` | Yes | Field definitions for this template |
| `steps` | `array` | Yes | Lifecycle steps for completing this ticket type |
| `prompt` | `string` \| `null` | No | Optional prompt for work launching (interpolated with handlebars) |
| `agent_prompt` | `string` \| `null` | No | Prompt for generating this issue type's operator agent via `claude -p` |
| `agent` | `string` \| `null` | No | Default delegator name for this issuetype (overridden by step.agent) |

### key

- **Description**: Unique issuetype key (e.g., FEAT, FIX, SPIKE, INV, TASK)
- **Type**: `string`

### name

- **Description**: Display name of the template type
- **Type**: `string`

### description

- **Description**: Brief description of when to use this template
- **Type**: `string`

### mode

- **Description**: Whether this issuetype runs autonomously or requires human pairing
- **Type**: → `ExecutionMode`

### glyph

- **Description**: Glyph character displayed in UI for this issuetype
- **Type**: `string`

### color

- **Description**: Optional color for glyph display in TUI
- **Type**: `string` \| `null`
- **Default**: `null`

### project_required

- **Description**: Whether a project must be specified for this issuetype
- **Type**: `boolean`
- **Default**: `true`

### fields

- **Description**: Field definitions for this template
- **Type**: `array`

### steps

- **Description**: Lifecycle steps for completing this ticket type
- **Type**: `array`

### prompt

- **Description**: Optional prompt for work launching (interpolated with handlebars)
- **Type**: `string` \| `null`
- **Default**: `null`

### agent_prompt

- **Description**: Prompt for generating this issue type's operator agent via `claude -p`
- **Type**: `string` \| `null`
- **Default**: `null`

### agent

- **Description**: Default delegator name for this issuetype (overridden by step.agent)
- **Type**: `string` \| `null`
- **Default**: `null`

## Definitions

### Definition: ExecutionMode

Execution mode for an issuetype

### Definition: FieldSchema

Schema definition for a single field in a template

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `name` | `string` | Yes | Field identifier (matches handlebar variable name) |
| `description` | `string` | Yes | Help text for the field |
| `type` | → `FieldType` | Yes | Type of the field |
| `required` | `boolean` | No | Whether this field must be filled |
| `default` | `string` \| `null` | No | Default value if any |
| `auto` | object | No | Auto-generation strategy for this field |
| `options` | `array` | No | Options for enum fields |
| `placeholder` | `string` \| `null` | No | Placeholder text shown in template |
| `max_length` | `integer` \| `null` | No | Maximum length for string fields |
| `display_order` | `integer` \| `null` | No | Display order in form (lower = first) |
| `user_editable` | `boolean` | No | Whether the user can edit this field (false for auto-generated) |

### Definition: FieldType

Types of fields supported in template schemas

### Definition: AutoGenStrategy

Auto-generation strategies for fields

### Definition: StepSchema

Schema definition for a lifecycle step

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `name` | `string` | Yes | Step identifier (lowercase) |
| `display_name` | `string` \| `null` | No | Human-readable step name |
| `type` | → `StepTypeTag` | No | Step type discriminator (defaults to "task" for backward compatibility) |
| `outputs` | `array` | Yes | Types of outputs this step produces |
| `prompt` | `string` | Yes | Initial prompt template for the Claude agent |
| `review_type` | → `ReviewType` | No | Type of review required for this step (none, plan, visual, pr) |
| `visual_config` | object | No | Configuration for visual review (required when `review_type` is "visual") |
| `on_reject` | object | No | What to do if step output is rejected |
| `next_step` | `string` \| `null` | No | Name of the next step (None for final step) |
| `allowed_tools` | `array` | No | Claude Code tools allowed in this step |
| `agent` | `string` \| `null` | No | Optional agent (delegator) name for this step (overrides ticket's default agent) |
| `permissions` | object | No | Provider-agnostic permissions for this step |
| `cli_args` | object | No | Arbitrary CLI arguments per provider |
| `permission_mode` | → `PermissionMode` | No | Preferred LLM permission mode for this step |
| `jsonSchema` | object | No | Inline JSON schema for structured output (Claude-specific) |
| `jsonSchemaFile` | `string` \| `null` | No | Path to JSON schema file for structured output (Claude-specific) |
| `artifact_patterns` | `array` | No | File glob patterns in the worktree that signal this step is complete |
| `classifier_config` | object | No | Configuration for classifier steps (required when type=classifier) |
| `rag_config` | object | No | Configuration for RAG steps (required when type=rag) |
| `delegator_config` | object | No | Configuration for delegator steps (required when type=delegator) |
| `mcp_config` | object | No | Configuration for MCP steps (required when type=mcp) |
| `multi_model_config` | object | No | Configuration for multi-model steps (required when `type=multi_model`) |
| `multi_prompt_config` | object | No | Configuration for multi-prompt steps (required when `type=multi_prompt`) |
| `matrixed_config` | object | No | Configuration for matrixed steps (required when type=matrixed) |

### Definition: StepTypeTag

Discriminator tag for step types

### Definition: StepOutput

Types of outputs a step can produce

### Definition: ReviewType

Type of review required for a step

### Definition: VisualReviewConfig

Configuration for visual review steps

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `url` | `string` | Yes | URL to open for visual check (supports handlebars templates) |
| `startup_command` | `string` \| `null` | No | Optional startup command (e.g., dev server) to run before opening browser |
| `startup_timeout_secs` | `integer` \| `null` | No | Timeout in seconds for server startup (default: 30) |

### Definition: OnReject

Action to take when a step is rejected

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `goto_step` | `string` | Yes | Step name to return to on rejection |
| `prompt` | `string` | Yes | Prompt to use when restarting after rejection |

### Definition: StepPermissions

Complete permission set for a step (as defined in issuetype schema)

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `tools` | → `ToolPermissions` | No | Tool-level allow/deny lists |
| `directories` | → `DirectoryPermissions` | No | Directory-level allow/deny lists |
| `mcp_servers` | → `McpServerPermissions` | No | MCP server enable/disable configuration |
| `custom_flags` | → `CustomFlags` | No | Per-provider custom configuration flags |

### Definition: ToolPermissions

Tool-level permissions (allow/deny lists)

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `allow` | `array` | No | Tools/patterns to allow |
| `deny` | `array` | No | Tools/patterns to deny |

### Definition: ToolPattern

Provider-agnostic tool pattern

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `tool` | `string` | Yes | Tool name: Read, Write, Edit, Bash, Glob, Grep, `WebFetch`, etc. |
| `pattern` | `string` \| `null` | No | Optional pattern for tool arguments (e.g., "cargo test:*" for Bash) |

### Definition: DirectoryPermissions

Directory-level permissions

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `allow` | `array` | No | Additional directories to allow access to (glob patterns) |
| `deny` | `array` | No | Directories to deny access to (glob patterns) |

### Definition: McpServerPermissions

MCP server permissions (server-level enable/disable only)

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `enable` | `array` | No | MCP servers to enable for this step |
| `disable` | `array` | No | MCP servers to disable for this step |

### Definition: CustomFlags

Per-provider custom configuration flags

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `claude` | `object` | No | Claude-specific configuration flags |
| `gemini` | `object` | No | Gemini-specific configuration flags |
| `codex` | `object` | No | Codex-specific configuration flags |

### Definition: ProviderCliArgs

Arbitrary CLI arguments per provider

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `claude` | `array` | No | CLI arguments for Claude |
| `gemini` | `array` | No | CLI arguments for Gemini |
| `codex` | `array` | No | CLI arguments for Codex |

### Definition: PermissionMode

Permission mode for LLM interaction

### Definition: ClassifierConfig

Configuration for classifier steps that return structured typed output

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `output_type` | → `ClassifierOutputType` | Yes | What type of answer the classifier returns |
| `options` | `array` \| `null` | No | For enum type: the allowed options |
| `max_length` | `integer` \| `null` | No | For `short_string`: max character length (default 255) |
| `agent` | `string` \| `null` | No | Agent/delegator to use (overrides issuetype default) |

### Definition: ClassifierOutputType

Output types for classifier steps

### Definition: RagConfig

Configuration for RAG (retrieval-augmented generation) steps

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `sources` | `array` | Yes | Context sources to retrieve before running the prompt |
| `max_context_tokens` | `integer` \| `null` | No | Maximum tokens of context to inject (default: 50000) |
| `agent` | `string` \| `null` | No | Agent/delegator to use |
| `allowed_tools` | `array` | No | Tools allowed for the agent |

### Definition: RagSource

A source of context for RAG steps

### Definition: DelegatorStepConfig

Configuration for delegator steps that run with a specific model+flavor

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `delegator` | `string` | Yes | Named delegator reference (from config.delegators) |
| `prompt_flavor` | `string` \| `null` | No | Additional prompt flavor text prepended to the step prompt |
| `allowed_tools` | `array` | No | Tools allowed |
| `permissions` | object | No | Permissions |

### Definition: McpStepConfig

Configuration for MCP steps that require specific MCP tools

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `required_tools` | `array` | Yes | MCP tools that MUST be available (step fails if missing) |
| `optional_tools` | `array` | No | MCP tools that SHOULD be available (warning if missing) |
| `agent` | `string` \| `null` | No | Agent/delegator to use |
| `allowed_tools` | `array` | No | Tools allowed (in addition to MCP tools) |

### Definition: McpToolRef

Reference to a specific MCP server tool

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `server` | `string` | Yes | MCP server name |
| `tool` | `string` \| `null` | No | Specific tool name (None = all tools from this server) |

### Definition: MultiModelConfig

Configuration for multi-model delegation steps (fan-out + vote)

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `delegators` | `array` | Yes | Named delegator references (from config.delegators), minimum 2 |
| `voting_strategy` | → `VotingStrategy` | Yes | How to aggregate/select the final answer |
| `share_answers` | `boolean` | No | Whether to share all answers with all models in the voting round |
| `voting_prompt` | `string` \| `null` | No | Prompt for the voting round (Handlebars, receives {{ answers }} array) |
| `voting_mode` | → `VotingMode` | No | How the voting round executes |

### Definition: VotingStrategy

Voting strategy for multi-model steps

### Definition: VotingMode

How the voting round is executed in multi-model steps

### Definition: MultiPromptConfig

Configuration for multi-prompt interrogation steps (N variations, select best)

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `prompt_variations` | `array` | Yes | Prompt variations (Handlebars templates), minimum 2 |
| `selection_strategy` | → `SelectionStrategy` | Yes | How to select the best result |
| `agent` | `string` \| `null` | No | Agent/delegator to use for all variations |
| `selection_prompt` | `string` \| `null` | No | Prompt for the selection/review round |

### Definition: SelectionStrategy

Selection strategy for multi-prompt steps

### Definition: MatrixedConfig

Configuration for matrixed work output steps (N x M delegators x prompts)

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `delegators` | `array` | Yes | Named delegator references (N), minimum 2 |
| `prompt_variations` | `array` | Yes | Prompt variations (M) — Handlebars templates, minimum 2 |
| `output_format` | → `MatrixedOutputFormat` | Yes | How to organize/present the N x M output |
| `aggregation_prompt` | `string` \| `null` | No | Optional aggregation prompt (receives the full matrix of results) |

### Definition: MatrixedOutputFormat

Output format for matrixed steps


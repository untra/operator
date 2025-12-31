---
title: "Issue Type Schema"
layout: doc
---

<!-- AUTO-GENERATED FROM src/templates/issuetype_schema.json - DO NOT EDIT MANUALLY -->
<!-- Regenerate with: cargo run -- docs -->

# Issue Type Schema

Schema for validating operator issuetype template configurations

## Schema Information

- **$schema**: `http://json-schema.org/draft-07/schema#`
- **$id**: `https://gbqr.us/operator/issuetype-template.schema.json`

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
| `name` | `string` | Yes | Human-readable name of the issuetype, eg. bug, feature, task, chore, spike, etc. |
| `description` | `string` | Yes | Description of what this issuetype is for |
| `mode` | `string` | Yes | Whether this issuetype work runs autonomously or requires human pairing |
| `glyph` | `string` | Yes | Icon/glyph character displayed in the UI for this issuetype (e.g., '*', '#', '!', '?', '>') |
| `color` | `string` | No | Optional color for the glyph in TUI display |
| `project_required` | `boolean` | No | Whether a project must be specified for this issuetype |
| `fields` | `array` | Yes | Field definitions for the ticket form |
| `steps` | `array` | Yes | Lifecycle steps for completing this ticket type |
| `prompt` | `string` | No | Issue prompt to apply to work creation. |
| `agent_creation_prompt` | `string` | No | Optional prompt for generating an operator agent for this issuetype via 'claude -p' for this prompt. Should instruct Claude to output ONLY the agent system prompt. If omitted, no operator agent will be generated for this issuetype. |

### key

- **Description**: Unique issuetype key (e.g., FEAT, FIX, SPIKE, INV, TASK)
- **Type**: `string`
- **Pattern**: `^[A-Z]+$`
- **Examples**: `FEAT`, `FIX`, `SPIKE`, `INV`, `TASK`

### name

- **Description**: Human-readable name of the issuetype, eg. bug, feature, task, chore, spike, etc.
- **Type**: `string`

### description

- **Description**: Description of what this issuetype is for
- **Type**: `string`

### mode

- **Description**: Whether this issuetype work runs autonomously or requires human pairing
- **Type**: `string`
- **Default**: `"paired"`
- **Allowed Values**: `autonomous`, `paired`

### glyph

- **Description**: Icon/glyph character displayed in the UI for this issuetype (e.g., '*', '#', '!', '?', '>')
- **Type**: `string`
- **Examples**: `*`, `#`, `!`, `?`, `>`

### color

- **Description**: Optional color for the glyph in TUI display
- **Type**: `string`
- **Allowed Values**: `blue`, `cyan`, `green`, `yellow`, `magenta`, `red`

### project_required

- **Description**: Whether a project must be specified for this issuetype
- **Type**: `boolean`
- **Default**: `true`

### fields

- **Description**: Field definitions for the ticket form
- **Type**: `array`

### steps

- **Description**: Lifecycle steps for completing this ticket type
- **Type**: `array`

### prompt

- **Description**: Issue prompt to apply to work creation.
- **Type**: `string`

### agent_creation_prompt

- **Description**: Optional prompt for generating an operator agent for this issuetype via 'claude -p' for this prompt. Should instruct Claude to output ONLY the agent system prompt. If omitted, no operator agent will be generated for this issuetype.
- **Type**: `string`

## Definitions

### Definition: field

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `name` | `string` | Yes | Field identifier (lowercase with underscores) |
| `description` | `string` | Yes | Human-readable description of the field |
| `type` | `string` | Yes | Field data type |
| `required` | `boolean` | No | Whether this field must be filled |
| `default` | `string` \| `boolean` \| `integer` \| `null` | No | Default value for the field. Required if field is required (except for 'id' field) |
| `min` | `integer` | No | Minimum value for integer fields |
| `max` | `integer` | No | Maximum value for integer fields |
| `auto` | `string` | No | Auto-generation strategy for this field |
| `options` | `array` | No | Available options for enum fields |
| `placeholder` | `string` | No | Placeholder text shown in empty field |
| `max_length` | `integer` | No | Maximum character length for string/text fields |
| `display_order` | `integer` | No | Order in which to display this field in the form |
| `user_editable` | `boolean` | No | Whether the user can edit this field (false for auto-generated) |

### Definition: step

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `name` | `string` | Yes | Step identifier (lowercase) |
| `display_name` | `string` | No | Human-readable step name |
| `outputs` | `array` | Yes | Types of outputs this step produces |
| `prompt` | `string` | Yes | Initial prompt template for the Claude agent at this step |
| `allowed_tools` | `array` | Yes | Claude Code tools allowed in this step (e.g., 'Read', 'Write', 'Bash') |
| `review_type` | `string` | No | Type of review required: none (auto-proceed), plan (approve plan), visual (browser check), pr (GitHub PR review) |
| `visual_config` | `object` | No | Configuration for visual review (required when review_type is 'visual') |
| `on_reject` | `object` | No | What to do if step output is rejected |
| `next_step` | `string` \| `null` | No | Name of the next step (null for final step) |
| `permissions` | → `stepPermissions` | No | Provider-agnostic permissions for this step, merged additively with project settings |
| `cli_args` | → `providerCliArgs` | No | Arbitrary CLI arguments per provider |
| `permission_mode` | `string` | No | Preferred LLM permission mode for this step. Only applies to providers that support it (e.g., Claude). No-op for unsupported providers. |
| `jsonSchema` | `object` | No | Inline JSON schema for structured output. Claude-specific: sets --json-schema flag. Takes precedence over jsonSchemaFile if both are defined. |
| `jsonSchemaFile` | `string` | No | Path to a local JSON schema file for structured output, relative to the project root. Claude-specific: sets --json-schema flag. |

### Definition: stepPermissions

Provider-agnostic permissions for a step

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `tools` | `object` | No | Tool-level allow/deny lists |
| `directories` | `object` | No | Directory-level allow/deny lists |
| `mcp_servers` | `object` | No | MCP server enable/disable configuration |
| `custom_flags` | `object` | No | Per-provider custom configuration flags |

### Definition: toolPattern

Provider-agnostic tool permission pattern

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `tool` | `string` | Yes | Tool name: Read, Write, Edit, Bash, Glob, Grep, WebFetch, etc. |
| `pattern` | `string` | No | Optional pattern for tool arguments (e.g., 'cargo test:*' for Bash) |

### Definition: providerCliArgs

Arbitrary CLI arguments per provider

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `claude` | `array` | No | CLI arguments for Claude |
| `gemini` | `array` | No | CLI arguments for Gemini |
| `codex` | `array` | No | CLI arguments for Codex |


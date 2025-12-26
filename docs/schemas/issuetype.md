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
| `agent_prompt` | `string` | No | Prompt for generating an operator agent for this issuetype via 'claude -p'. Should instruct Claude to output ONLY the agent system prompt. If omitted, no operator agent will be generated for this issuetype. |
| `branch_prefix` | `string` | No | Git branch prefix for this issuetype (e.g., 'feature', 'fix') |
| `color` | `string` | No | Optional color for the glyph in TUI display |
| `description` | `string` | Yes | Description of what this issuetype is for |
| `fields` | `array` | Yes | Field definitions for the ticket form |
| `glyph` | `string` | Yes | Icon/glyph character displayed in the UI for this issuetype (e.g., '*', '#', '!', '?', '>') |
| `key` | `string` | Yes | Unique issuetype key (e.g., FEAT, FIX, SPIKE, INV, TASK) |
| `mode` | `string` | Yes | Whether this issuetype runs autonomously or requires human pairing |
| `name` | `string` | Yes | Human-readable name of the issuetype |
| `project_required` | `boolean` | No | Whether a project must be specified for this issuetype |
| `steps` | `array` | Yes | Lifecycle steps for completing this ticket type |

### agent_prompt

- **Description**: Prompt for generating an operator agent for this issuetype via 'claude -p'. Should instruct Claude to output ONLY the agent system prompt. If omitted, no operator agent will be generated for this issuetype.
- **Type**: `string`

### branch_prefix

- **Description**: Git branch prefix for this issuetype (e.g., 'feature', 'fix')
- **Type**: `string`
- **Default**: `"task"`

### color

- **Description**: Optional color for the glyph in TUI display
- **Type**: `string`
- **Allowed Values**: `blue`, `cyan`, `green`, `yellow`, `magenta`, `red`

### description

- **Description**: Description of what this issuetype is for
- **Type**: `string`

### fields

- **Description**: Field definitions for the ticket form
- **Type**: `array`

### glyph

- **Description**: Icon/glyph character displayed in the UI for this issuetype (e.g., '*', '#', '!', '?', '>')
- **Type**: `string`
- **Examples**: `*`, `#`, `!`, `?`, `>`

### key

- **Description**: Unique issuetype key (e.g., FEAT, FIX, SPIKE, INV, TASK)
- **Type**: `string`
- **Pattern**: `^[A-Z]+$`
- **Examples**: `FEAT`, `FIX`, `SPIKE`, `INV`, `TASK`

### mode

- **Description**: Whether this issuetype runs autonomously or requires human pairing
- **Type**: `string`
- **Allowed Values**: `autonomous`, `paired`

### name

- **Description**: Human-readable name of the issuetype
- **Type**: `string`

### project_required

- **Description**: Whether a project must be specified for this issuetype
- **Type**: `boolean`
- **Default**: `true`

### steps

- **Description**: Lifecycle steps for completing this ticket type
- **Type**: `array`

## Definitions

### Definition: field

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `auto` | `string` | No | Auto-generation strategy for this field |
| `default` | `string` \| `boolean` \| `null` | No | Default value for the field. Required if field is required (except for 'id' field) |
| `description` | `string` | Yes | Human-readable description of the field |
| `display_order` | `integer` | No | Order in which to display this field in the form |
| `max_length` | `integer` | No | Maximum character length for string/text fields |
| `name` | `string` | Yes | Field identifier (lowercase with underscores) |
| `options` | `array` | No | Available options for enum fields |
| `placeholder` | `string` | No | Placeholder text shown in empty field |
| `required` | `boolean` | No | Whether this field must be filled |
| `type` | `string` | Yes | Field data type |
| `user_editable` | `boolean` | No | Whether the user can edit this field (false for auto-generated) |

### Definition: providerCliArgs

Arbitrary CLI arguments per provider

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `claude` | `array` | No | CLI arguments for Claude |
| `codex` | `array` | No | CLI arguments for Codex |
| `gemini` | `array` | No | CLI arguments for Gemini |

### Definition: step

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `allowed_tools` | `array` | Yes | Claude Code tools allowed in this step (e.g., 'Read', 'Write', 'Bash') |
| `cli_args` | → `providerCliArgs` | No | Arbitrary CLI arguments per provider |
| `display_name` | `string` | No | Human-readable step name |
| `jsonSchema` | `object` | No | Inline JSON schema for structured output. Claude-specific: sets --json-schema flag. Takes precedence over jsonSchemaFile if both are defined. |
| `jsonSchemaFile` | `string` | No | Path to a local JSON schema file for structured output, relative to the project root. Claude-specific: sets --json-schema flag. |
| `name` | `string` | Yes | Step identifier (lowercase) |
| `next_step` | `string` \| `null` | No | Name of the next step (null for final step) |
| `on_reject` | `object` | No | What to do if step output is rejected |
| `outputs` | `array` | Yes | Types of outputs this step produces |
| `permission_mode` | `string` | No | Preferred LLM permission mode for this step. Only applies to providers that support it (e.g., Claude). No-op for unsupported providers. |
| `permissions` | → `stepPermissions` | No | Provider-agnostic permissions for this step, merged additively with project settings |
| `prompt` | `string` | Yes | Initial prompt template for the Claude agent at this step |
| `requires_review` | `boolean` | No | Whether this step requires human review before proceeding |

### Definition: stepPermissions

Provider-agnostic permissions for a step

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `custom_flags` | `object` | No | Per-provider custom configuration flags |
| `directories` | `object` | No | Directory-level allow/deny lists |
| `mcp_servers` | `object` | No | MCP server enable/disable configuration |
| `tools` | `object` | No | Tool-level allow/deny lists |

### Definition: toolPattern

Provider-agnostic tool permission pattern

| Property | Type | Required | Description |
| --- | --- | --- | --- |
| `pattern` | `string` | No | Optional pattern for tool arguments (e.g., 'cargo test:*' for Bash) |
| `tool` | `string` | Yes | Tool name: Read, Write, Edit, Bash, Glob, Grep, WebFetch, etc. |


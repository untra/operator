---
title: "Schema Reference"
layout: doc
---

<!-- AUTO-GENERATED FROM docs/schemas/ - DO NOT EDIT MANUALLY -->
<!-- Regenerate with: cargo run -- docs -->

# Schema Reference

This section documents all JSON schemas and type definitions used by Operator.

## Documentation

Human-readable documentation for each schema:

| Schema | Description |
| --- | --- |
| [Configuration](config/) | Structure of `config.toml` - agents, paths, UI, notifications, and integrations |
| [Application State](state/) | Runtime state file (`state.json`) - active agents, completed tickets, system status |
| [Issue Type](issuetype/) | Issue type template format - fields, steps, permissions, and workflows |
| [Ticket Metadata](metadata/) | Ticket YAML frontmatter - status, priority, sessions, and LLM task tracking |
| [REST API](api/) | Interactive OpenAPI documentation with Swagger UI |

## Raw JSON Schemas

Machine-readable JSON Schema files for validation and code generation:

| File | Format | Description |
| --- | --- | --- |
| [config.json](config.json) | JSON Schema | Configuration file schema (generated via schemars) |
| [state.json](state.json) | JSON Schema | Runtime state file schema (generated via schemars) |
| [openapi.json](openapi.json) | OpenAPI 3.0 | REST API specification (generated via utoipa) |

## TypeScript Types

TypeScript type definitions are available for frontend integration:

- [TypeScript API Documentation](/typescript/) - Generated via TypeDoc
- Source: `shared/types.ts` (generated via ts-rs)

## Regenerating Schemas

Schemas are auto-generated from source code. To regenerate:

```bash
# Generate JSON schemas and TypeScript types
cargo run --bin generate_types

# Generate documentation pages
cargo run -- docs

# Generate TypeScript API docs
npm run docs:typescript
```

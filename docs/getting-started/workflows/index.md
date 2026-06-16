---
title: "Workflow Formats"
description: "Render an Operator ticket + issue type into a workflow another LLM tool or model can run."
layout: doc
---

# Workflow Formats

Operator is a kanban-shaped orchestrator: each **ticket** carries the work, and
its **issue type** carries the *shape* of the work — an ordered set of steps
(tasks, classifiers, delegators, fan-outs, pipelines, human review gates). A
**workflow export** renders that `ticket + issue type` pair into a concrete
orchestration format another tool or model can execute.

This is **export-only and lossy-by-design**: Operator emits the format; it does
not parse one back. Shapes a target can't represent natively (human review
gates, fan-out, RAG/MCP) are flattened deterministically and annotated, so the
same input always produces the same output.

## Formats

| Format | Artifact | Status | Docs |
|---|---|---|---|
| Claude Workflow | `.js` (Claude Code dynamic workflow) | GA | [Claude Workflow](./claude/) |
| AGNT Workflow | `.json` (AGNT.gg graph) | Alpha | [AGNT Workflow](./agnt/) |

The authoritative, machine-readable list is the
[`GET /api/v1/workflow-formats`](https://operator.untra.io/schemas/openapi.json)
endpoint, derived from the same source of truth that backs this page.

## How to export

CLI:

```bash
operator workflow export FEAT-1234              # default: claude (.js)
operator workflow export FEAT-1234 --format agnt
```

REST (the web UI and VS Code use the same shared code path):

```bash
# Concrete ticket -> workflow
curl -X POST "http://localhost:7008/api/v1/tickets/FEAT-1234/workflow-export?format=claude"

# Issue type alone -> preview (placeholder values, no ticket required)
curl "http://localhost:7008/api/v1/issuetypes/FEAT/workflow-preview?format=agnt"

# Discover the available formats
curl "http://localhost:7008/api/v1/workflow-formats"
```

In the TUI, web UI, and VS Code, the **Workflows** section lists the formats and
links to preview/export.

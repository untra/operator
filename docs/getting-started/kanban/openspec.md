---
title: "OpenSpec"
description: "Import OpenSpec spec-driven change tasks as Operator tickets."
layout: doc
---

# OpenSpec

<span class="badge alpha">Experimental</span>

Operator can import work from [**OpenSpec**](https://github.com/Fission-AI/OpenSpec), the spec-driven development (SDD) framework for AI coding assistants. OpenSpec keeps proposed changes as plain-markdown bundles in your repository; Operator turns their task checklists into queued tickets.

> **Experimental.** This provider is pull-only and file-based. Operator never edits your OpenSpec files — checking off completed tasks in `tasks.md` remains yours (or your agent's) to do.

## How the mapping works

OpenSpec stores each proposed change at `openspec/changes/<change-id>/` with a `proposal.md`, an implementation checklist in `tasks.md`, and optional `design.md` + spec deltas. Operator maps that structure onto its kanban model:

| OpenSpec | Operator |
|----------|----------|
| Active change (`changes/<id>/`) | A kanban "project" (the change id is the project key) |
| `## 1. Group` heading in `tasks.md` | One ticket per task group |
| Checklist items under the group | The ticket's task list (embedded in the body) |
| All items checked | Group counts as `done` and is skipped on import |

Each imported ticket carries `external_provider: openspec` and `external_id: <change-id>#<group-number>` in its frontmatter, so re-running an import skips everything already in your queue — imports are idempotent.

The ticket body includes the change id, the proposal's **Why** section, the group's checklist verbatim, and a pointer to the change directory so agents read the full spec (proposal, design, deltas) before starting.

## Configuration

Add an OpenSpec root to `operator.toml`:

```toml
[kanban.openspec.myrepo]
enabled = true
root_path = "/path/to/your-repo/openspec"  # the dir containing changes/
project = "yourproject"                     # operator project for imported tickets
```

| Setting | Default | Description |
|---------|---------|-------------|
| `enabled` | `false` | Whether this OpenSpec root is active |
| `root_path` | — | Directory containing the OpenSpec `changes/` tree |
| `project` | change id | Operator project stamped on imported tickets |

No credentials are needed — OpenSpec is local markdown.

## Importing

```bash
# Import one change's open task groups as tickets
operator import openspec add-dark-mode

# Import every active (non-archived) change under all configured roots
operator import openspec

# Sync all configured kanban providers, OpenSpec included
operator import
```

Fully-checked task groups are skipped; unchecked or partially-checked groups become `TASK` tickets (flagged `needs_issuetype_mapping` so you can retype them if desired). Re-running any of these commands only creates tickets for groups not already imported.

## Limitations

- **Pull-only.** Ticket completion is not written back to `tasks.md` checkboxes, and Operator cannot create OpenSpec changes.
- **Group granularity.** One ticket per `## N.` task group — individual checklist items are not split into their own tickets.
- **No dependency ordering.** Tickets are queued FIFO in group order; Operator's same-project sequencing keeps them from running concurrently, but there is no hard blocking between groups.
- `design.md` and spec deltas are referenced by path, not ingested.

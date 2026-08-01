---
title: "JR Orchestration"
layout: doc
section: workflows
---

<!-- AUTO-GENERATED FROM the jr_orchestration collection manifest - DO NOT EDIT MANUALLY -->
<!-- Regenerate with: cargo run -- docs -->

Feature/task orchestration with coder, reviewer, architect, and rebase work units.

| | |
|---|---|
| **Tier** | community |
| **Author** | [snapwich](https://github.com/snapwich/jr) |
| **License** | MIT |
| **Version** | 1.0.0 |
| **Created** | 2026-06-16 |
| **Updated** | 2026-07-01 |
| **Loop shape** | `feature_task_review_graph` |
| **Review gates** | `code_review`, `architect_review`, `human_pr_review` |
| **Stops when** | feature PR ready for human review; review changes requested; blocked dependency documented; review escalated to human after repeated changes (operator stops; no auto-handoff) |
| **Manifest** | [`collection.json`](/collections/jr_orchestration/collection.json) |

## Issue types

| Key | Name | Mode | Steps |
|---|---|---|---|
| `JRPLAN` | JR Plan | paired | 4 |
| `JRFEAT` | JR Feature | paired | 4 |
| `JRTASK` | JR Task | autonomous | 5 |
| `JRREV` | JR Review | paired | 4 |
| `JRREBASE` | JR Rebase | autonomous | 4 |

## Workflows

Select an issue type to see the Operator workflow it defines. This is the same graph the Operator app draws, rendered from the same published JSON.

<div class="workflow-explorer">
  <operator-workflow-explorer base="/collections/jr_orchestration/"></operator-workflow-explorer>
</div>

## Install

Operator reads the hosted catalog on startup, so this collection appears in the setup picker. To pin it explicitly:

```toml
# config.toml
[templates]
active_collection = "jr_orchestration"
```

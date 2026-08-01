---
title: "Dev Kanban"
layout: doc
section: workflows
---

<!-- AUTO-GENERATED FROM the dev_kanban collection manifest - DO NOT EDIT MANUALLY -->
<!-- Regenerate with: cargo run -- docs -->

Developer kanban with TASK, FEAT, FIX

| | |
|---|---|
| **Tier** | official |
| **Author** | [Operator!](https://github.com/untra/operator) |
| **License** | MIT |
| **Version** | 1.0.0 |
| **Created** | 2026-01-08 |
| **Updated** | 2026-06-16 |
| **Loop shape** | `single_pass` |
| **Review gates** | `test_suite` |
| **Stops when** | tests_green |
| **Manifest** | [`collection.json`](/collections/dev_kanban/collection.json) |

## Issue types

| Key | Name | Mode | Steps |
|---|---|---|---|
| `TASK` | Task | autonomous | 1 |
| `FEAT` | Feature | autonomous | 5 |
| `FIX` | Fix | autonomous | 5 |

## Workflows

Select an issue type to see the Operator workflow it defines. This is the same graph the Operator app draws, rendered from the same published JSON.

<div class="workflow-explorer">
  <operator-workflow-explorer base="/collections/dev_kanban/"></operator-workflow-explorer>
</div>

## Install

Operator reads the hosted catalog on startup, so this collection appears in the setup picker. To pin it explicitly:

```toml
# config.toml
[templates]
active_collection = "dev_kanban"
```

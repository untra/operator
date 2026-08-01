---
title: "Simple"
layout: doc
section: workflows
---

<!-- AUTO-GENERATED FROM the simple collection manifest - DO NOT EDIT MANUALLY -->
<!-- Regenerate with: cargo run -- docs -->

Simple workflow with TASK only

| | |
|---|---|
| **Tier** | official |
| **Author** | [Operator!](https://github.com/untra/operator) |
| **License** | MIT |
| **Version** | 1.0.0 |
| **Created** | 2026-01-08 |
| **Updated** | 2026-07-01 |
| **Loop shape** | `single_pass` |
| **Stops when** | task_complete |
| **Manifest** | [`collection.json`](/collections/simple/collection.json) |

## Issue types

| Key | Name | Mode | Steps |
|---|---|---|---|
| `TASK` | Task | autonomous | 1 |

## Workflows

Select an issue type to see the Operator workflow it defines. This is the same graph the Operator app draws, rendered from the same published JSON.

<div class="workflow-explorer">
  <operator-workflow-explorer base="/collections/simple/"></operator-workflow-explorer>
</div>

## Install

Operator reads the hosted catalog on startup, so this collection appears in the setup picker. To pin it explicitly:

```toml
# config.toml
[templates]
active_collection = "simple"
```

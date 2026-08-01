---
title: "Coder"
layout: doc
section: workflows
---

<!-- AUTO-GENERATED FROM the coder collection manifest - DO NOT EDIT MANUALLY -->
<!-- Regenerate with: cargo run -- docs -->

Linear-synced engineering flow: Feature, Improvement, and Bug work delegated to coding agents.

| | |
|---|---|
| **Tier** | community |
| **Author** | [untra](https://github.com/untra/operator) |
| **License** | MIT |
| **Version** | 1.0.0 |
| **Created** | 2026-08-01 |
| **Updated** | 2026-08-01 |
| **Loop shape** | `kanban_synced_single_pass` |
| **Review gates** | `plan_review`, `test_suite`, `pr_review` |
| **Stops when** | tests_green; pr_created |
| **Manifest** | [`collection.json`](/collections/coder/collection.json) |

## Issue types

| Key | Name | Mode | Steps |
|---|---|---|---|
| `FEATURE` | Feature | autonomous | 5 |
| `IMPROVEMENT` | Improvement | autonomous | 4 |
| `BUG` | Bug | autonomous | 5 |

## Workflows

Select an issue type to see the Operator workflow it defines. This is the same graph the Operator app draws, rendered from the same published JSON.

<div class="workflow-explorer">
  <operator-workflow-explorer base="/collections/coder/"></operator-workflow-explorer>
</div>

## Install

Operator reads the hosted catalog on startup, so this collection appears in the setup picker. To pin it explicitly:

```toml
# config.toml
[templates]
active_collection = "coder"
```

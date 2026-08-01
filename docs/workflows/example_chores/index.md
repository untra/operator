---
title: "Example Chores"
layout: doc
section: workflows
---

<!-- AUTO-GENERATED FROM the example_chores collection manifest - DO NOT EDIT MANUALLY -->
<!-- Regenerate with: cargo run -- docs -->

Minimal example community collection demonstrating the shareable format.

| | |
|---|---|
| **Tier** | community |
| **Author** | [untra](https://github.com/untra/operator) |
| **License** | MIT |
| **Version** | 0.1.0 |
| **Created** | 2026-07-01 |
| **Updated** | 2026-07-01 |
| **Loop shape** | `single_pass` |
| **Review gates** | `test_suite` |
| **Stops when** | tests_green |
| **Manifest** | [`collection.json`](/collections/example_chores/collection.json) |

## Issue types

| Key | Name | Mode | Steps |
|---|---|---|---|
| `CHORE` | Chore | autonomous | 2 |

## Workflows

Select an issue type to see the Operator workflow it defines. This is the same graph the Operator app draws, rendered from the same published JSON.

<div class="workflow-explorer">
  <operator-workflow-explorer base="/collections/example_chores/"></operator-workflow-explorer>
</div>

## Install

Operator reads the hosted catalog on startup, so this collection appears in the setup picker. To pin it explicitly:

```toml
# config.toml
[templates]
active_collection = "example_chores"
```

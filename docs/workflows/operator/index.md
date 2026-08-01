---
title: "Operator"
layout: doc
section: workflows
---

<!-- AUTO-GENERATED FROM the operator collection manifest - DO NOT EDIT MANUALLY -->
<!-- Regenerate with: cargo run -- docs -->

Operator automation tasks: ASSESS, SYNC, INIT

| | |
|---|---|
| **Tier** | official |
| **Author** | [Operator!](https://github.com/untra/operator) |
| **License** | MIT |
| **Version** | 1.0.0 |
| **Created** | 2026-01-08 |
| **Updated** | 2026-07-01 |
| **Loop shape** | `single_pass` |
| **Review gates** | `human` |
| **Stops when** | setup_artifacts_written |
| **Manifest** | [`collection.json`](/collections/operator/collection.json) |

## Issue types

| Key | Name | Mode | Steps |
|---|---|---|---|
| `ASSESS` | Project Assessment | autonomous | 2 |
| `SYNC` | Catalog Sync | autonomous | 3 |
| `INIT` | Workspace Init | paired | 3 |
| `AGENT_SETUP` | Agent Setup | paired | 3 |
| `PROJECT_INIT` | Project Initialization | autonomous | 2 |

## Workflows

Select an issue type to see the Operator workflow it defines. This is the same graph the Operator app draws, rendered from the same published JSON.

<div class="workflow-explorer">
  <operator-workflow-explorer base="/collections/operator/"></operator-workflow-explorer>
</div>

## Install

Operator reads the hosted catalog on startup, so this collection appears in the setup picker. To pin it explicitly:

```toml
# config.toml
[templates]
active_collection = "operator"
```

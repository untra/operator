---
title: "Elves Overnight"
layout: doc
section: workflows
---

<!-- AUTO-GENERATED FROM the elves_overnight collection manifest - DO NOT EDIT MANUALLY -->
<!-- Regenerate with: cargo run -- docs -->

Long-running staged batch workflow with durable memory, validation, PR review, and reporting.

| | |
|---|---|
| **Tier** | community |
| **Author** | [Aigora](https://github.com/aigorahub/elves) |
| **License** | MIT |
| **Version** | 1.0.0 |
| **Created** | 2026-06-16 |
| **Updated** | 2026-07-01 |
| **Loop shape** | `staged_long_running_batch_loop` |
| **Review gates** | `stage_review`, `batch_validation`, `fresh_review`, `judge_verdict`, `human_land_gate` |
| **Stops when** | batch complete and checkpointed; validation cannot be repaired safely; PR has unresolved requested changes; time/risk budget exhausted |
| **Manifest** | [`collection.json`](/collections/elves_overnight/collection.json) |

## Issue types

| Key | Name | Mode | Steps |
|---|---|---|---|
| `ELVSTAGE` | Elves Stage | paired | 4 |
| `ELVBATCH` | Elves Batch | autonomous | 9 |
| `LANDPR` | Land Pull Request | paired | 6 |
| `ELVRPT` | Elves Report | paired | 3 |

## Workflows

Select an issue type to see the Operator workflow it defines. This is the same graph the Operator app draws, rendered from the same published JSON.

<div class="workflow-explorer">
  <operator-workflow-explorer base="/collections/elves_overnight/"></operator-workflow-explorer>
</div>

## Install

Operator reads the hosted catalog on startup, so this collection appears in the setup picker. To pin it explicitly:

```toml
# config.toml
[templates]
active_collection = "elves_overnight"
```

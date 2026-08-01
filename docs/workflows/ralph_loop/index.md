---
title: "Ralph Loop"
layout: doc
section: workflows
---

<!-- AUTO-GENERATED FROM the ralph_loop collection manifest - DO NOT EDIT MANUALLY -->
<!-- Regenerate with: cargo run -- docs -->

PRD-to-story loop for completing one right-sized story per fresh agent context.

| | |
|---|---|
| **Tier** | community |
| **Author** | [snarktank](https://github.com/snarktank/ralph) |
| **License** | MIT |
| **Version** | 1.0.0 |
| **Created** | 2026-06-16 |
| **Updated** | 2026-07-01 |
| **Loop shape** | `fresh_context_story_loop` |
| **Review gates** | `plan_review`, `test_suite`, `story_completion_check` |
| **Stops when** | all stories have passes=true; blocked story documented; quality gates fail repeatedly; max_iterations reached (advisory; outer story loop is operator-queue-driven) |
| **Manifest** | [`collection.json`](/collections/ralph_loop/collection.json) |

## Issue types

| Key | Name | Mode | Steps |
|---|---|---|---|
| `PRD` | Product Requirements Document | paired | 4 |
| `STORY` | Ralph Story | autonomous | 5 |
| `RLOOP` | Ralph Loop Coordinator | paired | 4 |

## Workflows

Select an issue type to see the Operator workflow it defines. This is the same graph the Operator app draws, rendered from the same published JSON.

<div class="workflow-explorer">
  <operator-workflow-explorer base="/collections/ralph_loop/"></operator-workflow-explorer>
</div>

## Install

Operator reads the hosted catalog on startup, so this collection appears in the setup picker. To pin it explicitly:

```toml
# config.toml
[templates]
active_collection = "ralph_loop"
```

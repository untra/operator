---
title: Artifact Detection
description: "How Operator uses file artifacts as positive signals for step completion."
layout: doc
---

<span class="operator-brand">Operator!</span> can detect step completion by checking whether expected output files exist in the agent's worktree. This provides a **positive signal** that supplements idle detection.

## Overview

When an LLM agent goes idle, <span class="operator-brand">Operator!</span> needs to determine whether the agent completed its step successfully or is stuck. Idle detection alone is a "negative signal" -- it only tells you the agent stopped working, not whether it finished.

Artifact detection adds a positive signal: if a step declares expected output files and those files exist in the worktree when the agent goes idle, <span class="operator-brand">Operator!</span> treats the step as **completed**.

## How It Works

```
Agent goes idle (hook signal, pattern match)
         |
         v
   Has artifact_patterns?
      /         \
    Yes          No
     |            |
     v            v
 Check files   MovedToAwaiting
     |
     v
 All patterns match?
    /         \
  Yes          No
   |            |
   v            v
StepCompleted  MovedToAwaiting
```

### Detection Flow

1. The health check detects the agent is idle (via hook signals or content pattern matching)
2. If the agent's current step has `artifact_patterns`, <span class="operator-brand">Operator!</span> checks whether all patterns match at least one file in the agent's worktree
3. If all artifacts are found, the step is marked **StepCompleted** and the ticket advances to the next step
4. If any artifact is missing, the agent is marked **MovedToAwaiting** (stuck, needs human attention)

### Supplements, Does Not Replace

Artifact detection supplements idle detection. An agent must first be detected as idle before artifacts are checked. If the agent is still actively working, artifacts are not evaluated -- even if the expected files happen to exist already.

Steps without `artifact_patterns` behave exactly as before: idle detection triggers MovedToAwaiting.

## Configuration

Add `artifact_patterns` to any step in your issue type definition:

```json
{
  "name": "plan",
  "display_name": "Planning",
  "outputs": ["plan"],
  "prompt": "Create a plan in .tickets/plans/{{ id }}.md ...",
  "allowed_tools": ["Read", "Glob", "Grep", "Write"],
  "review_type": "plan",
  "artifact_patterns": [".tickets/plans/{{ id }}.md"],
  "next_step": "build"
}
```

Each entry is a file glob pattern resolved relative to the agent's worktree root. **All** patterns must match at least one file for the step to be considered complete.

### Pattern Syntax

Patterns use standard glob syntax:

| Pattern | Matches |
|---------|---------|
| `plan.md` | Exact file in worktree root |
| `*.rs` | All `.rs` files in worktree root |
| `src/**/*.rs` | All `.rs` files recursively under `src/` |
| `.tickets/plans/*.md` | Any markdown file in `.tickets/plans/` |

### Handlebars Templates

Patterns support handlebars variable interpolation from ticket fields. For example, `{{ id }}` is replaced with the ticket ID before the glob is evaluated.

### Empty Patterns

If `artifact_patterns` is omitted or set to an empty array, no artifact checking is performed. The step falls back to standard idle detection behavior.

## Caching

Artifact checks use a **2-second cache TTL**. Within that window, repeated health checks for the same agent reuse the previous artifact status without rescanning the filesystem.

## Example

The built-in FEAT issue type uses artifact detection on the `plan` step:

```json
{
  "name": "plan",
  "display_name": "Planning",
  "outputs": ["plan"],
  "prompt": "Create a plan in .tickets/plans/{{ id }}.md ...",
  "allowed_tools": ["Read", "Glob", "Grep", "Write"],
  "review_type": "plan",
  "artifact_patterns": [".tickets/plans/{{ id }}.md"],
  "next_step": "build"
}
```

When the planning agent finishes and goes idle, <span class="operator-brand">Operator!</span> checks for the plan file. If found, the step completes and the plan review flow begins. If the file is missing, the operator is notified that the agent may be stuck.

## Schema Reference

The `artifact_patterns` field is defined in the [Issue Type Schema](/schemas/issuetype/) under the step definition:

```json
{
  "artifact_patterns": {
    "type": "array",
    "items": { "type": "string" },
    "description": "File glob patterns in the worktree that signal this step is complete"
  }
}
```

See [Issue Types](/issue-types/) for more on configuring steps.

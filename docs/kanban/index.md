---
title: Kanban Workflow
description: "Understand the kanban-style workflow for managing tickets through queue, in-progress, and completed stages."
layout: doc
---

<span class="operator-brand">Operator!</span> uses a kanban-style workflow to manage tickets through their lifecycle.

## Ticket Lifecycle

Tickets flow through these stages:

```
.tickets/queue/     -> Work waiting to be picked up
.tickets/in-progress/ -> Currently being worked on
.tickets/completed/   -> Finished work
```

## Workflow Steps

### 1. Queue

New tickets are created in `.tickets/queue/`. They are sorted by:
1. **Priority** - INV > FIX > FEAT > SPIKE
2. **Timestamp** - FIFO within same priority

### 2. Assignment

When an agent slot is available, <span class="operator-brand">Operator!</span>:
1. Selects the next ticket by priority
2. Prompts for launch confirmation
3. Moves ticket to `in-progress/`

### 3. In Progress

While work is in progress:
- Agent status is tracked
- Progress notifications are sent
- <span class="operator-brand">Operator!</span> monitors for completion or awaiting input

### 4. Completion

When work finishes:
- Ticket moves to `completed/`
- Notification is sent
- Agent slot is freed for next ticket

## Parallelism Rules

<span class="operator-brand">Operator!</span> enforces these rules for concurrent work:

- **Max agents** = min(configured_max, cpu_cores - reserved_cores)
- **Autonomous agents** (FEAT, FIX) can run in parallel on different projects
- **Paired agents** (SPIKE, INV) run one at a time per operator
- **Same project** = sequential execution to avoid conflicts

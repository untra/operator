---
title: "Kanban"
description: "What a kanban board is, and how Operator uses one to run agents."
layout: doc
---

Kanban is a way of managing work by making it visible. A **board** holds columns; each **column** is a state of work; each **card** is one piece of work. Cards move left to right:

```
| To Do          | In Progress     | Done            |
|----------------|-----------------|-----------------|
| waiting work   | active work     | finished work   |
```

Two rules do most of the work:

1. **Pull, don't push.** Nobody is handed work - whoever has capacity pulls the next card.
2. **Limit work in progress.** Few cards in flight at once means work finishes instead of piling up half-done.

That's it. The board *is* the status report.

## How Operator uses kanban

In <span class="operator-brand">Operator!</span>, the cards are [tickets](/getting-started/tickets/) and the workers are AI agents. Operator holds three internal states - **todo**, **doing**, **done** - and enforces both rules: agents pull the next ticket when a slot frees up, and parallelism limits cap work in progress.

That board is a [kanban provider](/getting-started/kanban/) built-in, where each card is a markdown file under `.tickets/` and the column is the directory it sits in.

You can run on it alone, or connect an external provider like Jira, Linear or
GitHub Projects. Those sync *into* the Operator board: their issues arrive as
tickets here, and Operator maps its three states back onto your board's columns
as agents work.

## Card order

Both the web board and the agent launcher order work the same way, so what you
see at the top of the TODO column is what runs next:

1. **Issue type**, in the order set by `queue.priority_order` (default
   `INV > FIX > TASK > FEAT > SPIKE`). A type the list does not mention sorts
   after every type it does.
2. **The ticket's own `priority:`** field - `P0-critical`, `P1-high`,
   `P2-medium` (the default) or `P3-low`.
3. **Oldest first**, by the timestamp in the filename.

The DONE column ignores all of that and shows the most recently completed first.

## Filtering the board

The board at `/queue` shows every ticket in every project, which gets long fast.
The filter bar above it narrows the view without touching any ticket:

- **Search** matches a ticket's ID or summary. Several words all have to match.
- **Project**, **Type** and **Priority** are drawn from the tickets actually on
  the board, so a collection's custom issue types appear as soon as one exists.
  Picking several options in one facet widens it; picking across facets narrows.

Filters are yours alone - they live in your browser, not in the config or on the
server - and they survive a reload. The counter reads `N of M tickets` while any
filter is on, and **Clear filters** appears only when there is something to
clear.

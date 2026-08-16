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

1. **Pull, don't push.** Nobody is handed work — whoever has capacity pulls the next card.
2. **Limit work in progress.** Few cards in flight at once means work finishes instead of piling up half-done.

That's it. The board *is* the status report.

## How Operator uses kanban

In <span class="operator-brand">Operator!</span>, the cards are [tickets](/getting-started/tickets/) and the workers are AI agents. Operator holds three internal states — **todo**, **doing**, **done** — and enforces both rules: agents pull the next ticket when a slot frees up, and parallelism limits cap work in progress.

You can run entirely from local tickets, or sync the board with an external [kanban provider](/getting-started/kanban/) like Jira, Linear, or GitHub Projects — Operator maps its three states onto your board's columns and moves cards as agents work.

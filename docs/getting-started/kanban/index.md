---
title: "Supported Kanban Providers"
description: "Kanban and issue tracking integrations for Operator."
layout: doc
---

# Supported Kanban Providers

Operator integrates with popular issue tracking systems to manage work items for AI agents.

## Available Integrations

| Provider | Status | Notes |
|----------|--------|-------|
| [Jira Cloud](/getting-started/kanban/jira/) | Supported | Full API integration |
| [Linear](/getting-started/kanban/linear/) | Supported | Full API integration |
| [GitHub Projects](/getting-started/kanban/github/) | Supported | Projects v2 GraphQL integration |
| [OpenSpec](/getting-started/kanban/openspec/) | Experimental | Local spec-driven changes; pull-only |

## How It Works

Operator syncs tickets from your kanban provider:

1. **Pull**: Fetches issues from configured boards/projects
2. **Queue**: Orders tickets by priority and type
3. **Assign**: Dispatches tickets to available agents
4. **Update**: Pushes status changes back to your provider

## The Ticket Lifecycle

Whether tickets come from a provider or from `.tickets/`, Operator moves them
through the same three directories:

```
.tickets/queue/       -> Work waiting to be picked up
.tickets/in-progress/ -> Currently being worked on
.tickets/completed/   -> Finished work
```

**Queue.** New tickets land in `.tickets/queue/` and are ordered by their issue
type's position in the active collection, then FIFO by timestamp within the same
type. The ordering is a property of the collection, not a hard-coded table — see
[Workflows](/workflows/).

**Assignment.** When an agent slot frees up, Operator selects the next ticket,
prompts for launch confirmation, and moves it to `in-progress/`.

**In progress.** Agent status is tracked, progress notifications are sent, and
Operator watches for completion or for the agent awaiting input.

**Completion.** The ticket moves to `completed/`, a notification is sent, and the
slot is freed for the next ticket.

## Parallelism Rules

Operator bounds concurrent work so agents do not collide:

- **Max agents** = min(configured_max, cpu_cores - reserved_cores)
- **Autonomous agents** can run in parallel across different projects
- **Paired agents** run one at a time — they need your attention
- **Same project** is sequential, to avoid conflicting edits

Whether an issue type is autonomous or paired is declared by its `mode`. See
[Supported Coding Agents](/getting-started/agents/) for what each mode means in
practice.

## Column Mapping (todo / doing / done)

Operator is strict about its three internal states — **todo**, **doing**,
**done** — because they represent the work actually inflight at operator's
level. External boards have flexible columns, so each synced project declares
a `status_mapping` linking the two:

```toml
[kanban.<provider>."<workspace>".projects.<KEY>.status_mapping]
todo = "To Do"          # pulled into the queue; requeue pushes back here
doing = "In Progress"   # pushed when a ticket is launched/claimed
done = "Done"           # pushed when a ticket completes
```

With `bidirectional = true`, a synced ticket moves on the external board as
operator works it: launch → `doing`, complete → `done`, return-to-queue →
`todo`. The board's real column names are discoverable via the
`/api/v1/kanban/statuses` endpoints, and the VS Code config panel offers them
as dropdowns. See the per-provider guides for details.

## Choosing a Provider

Both Jira Cloud and Linear are fully supported kanban providers:

- **Jira Cloud**: Best for teams already using Atlassian products, with rich workflow customization
- **Linear**: Best for teams wanting a modern, fast issue tracker with streamlined workflows

## Local Tickets

Operator also supports local-only tickets in `.tickets/queue/` for projects without external issue tracking. See [Tickets](/getting-started/tickets/) for details.

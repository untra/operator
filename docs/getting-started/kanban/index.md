---
title: "Supported Kanban Providers"
description: "Kanban and issue tracking integrations for Operator."
layout: doc
---

<span class="operator-brand">Operator!</span> is itself a kanban board, available by default.

Issues are defined as markdown tickets on the Operator board, launching tickets with delegated agents allows the work to be done according to the workflows and standards you define.

## Available Integrations

Statuses follow the [feature maturity](/maturity/) scale.

| Provider | Status | Notes |
|----------|--------|-------|
| [Operator](/getting-started/kanban/operator/) | GA | Built in; the `.tickets/` markdown is the board |
| [Jira Cloud](/getting-started/kanban/jira/) | Beta | Full API integration |
| [Linear](/getting-started/kanban/linear/) | Beta | Full API integration |
| [GitHub Projects](/getting-started/kanban/github/) | Beta | Projects v2 GraphQL integration |
| [OpenSpec](/getting-started/kanban/openspec/) | Alpha | Local spec-driven changes; pull-only |

## How It Works

The Operator board always exists. Connecting an external provider adds a sync
loop on top of it:

1. **Pull**: Fetches issues from configured boards/projects
2. **Queue**: Writes them as tickets on the Operator board, ordered by type then FIFO
3. **Assign**: Dispatches tickets to available agents
4. **Update**: Pushes column changes back to the originating provider

The Operator board is only ever the destination of a sync, never a source.

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
type. The ordering is a property of the collection, not a hard-coded table - see
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
- **Paired agents** run one at a time - they need your attention
- **Same project** is sequential, to avoid conflicting edits

Whether an issue type is autonomous or paired is declared by its `mode`. See
[Supported Coding Agents](/getting-started/agents/) for what each mode means in
practice.

## Column Mapping (todo / doing / done)

Operator is strict about its three internal states - **todo**, **doing**,
**done** - because they represent the work actually inflight at operator's
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

- **Operator**: The default. Best when the work starts here - no setup, no
  credentials, tickets versioned alongside your code. Start here and add an
  external provider only when work has to be visible to people outside Operator.
- **Jira Cloud**: Best for teams already using Atlassian products, with rich workflow customization
- **Linear**: Best for teams wanting a modern, fast issue tracker with streamlined workflows
- **GitHub Projects**: Best when your work already lives in GitHub issues and Projects v2 boards
- **OpenSpec**: Best for local, spec-driven change tracking without an external tracker (pull-only)

## The Operator Board

Tickets in `.tickets/` are not a fallback for projects without an issue tracker -
they are the board every provider syncs into. See
[Operator](/getting-started/kanban/operator/) for how to work it, and
[Tickets](/getting-started/tickets/) for the ticket format.

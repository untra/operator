---
title: "Operator"
description: "The built-in kanban board: your markdown tickets, the columns agents pull from."
layout: doc
---

Operator is its own kanban provider, and the one it encourages you to start with.
There is nothing to connect and no credential to store: the board is the `.tickets/` directory on the server, and every other provider syncs *into* it.

## The board is the directory

A ticket is a markdown file. Which column it appears in is which directory it lives in:

```
.tickets/queue/        -> TODO QUEUE     work waiting to be pulled
.tickets/in-progress/  -> IN PROGRESS    an agent (or you) is on it
.tickets/completed/    -> DONE           finished work
```

## Why it is the default

Operator's job is to run structured, agent-dispatched work. That needs a queue it fully controls.

The Operator workflow graph is attached to each issue type. The built-in board is that queue - external providers are a way to *feed* it, not a replacement for it.

- **No setup.** It is active the moment Operator starts.
- **No credentials.** Nothing to rotate, nothing to leak.
- **Works offline.** No API to be rate-limited by.
- **Your format.** Ticket frontmatter is [documented](/schemas/metadata/) and yours to extend.

## Working the board

From the web UI's Queue page:

- **Add a card.** *+ New ticket* opens a form - issue type, project, summary. The ticket is written to `.tickets/queue/` through the same endpoint the CLI and MCP use, so it is identical to one created anywhere else.
- **Launch a card.** Click a card in TODO QUEUE or IN PROGRESS to open its detail panel: the ticket, its issue type's workflow graph, and a launch form where you pick the delegator, session wrapper and execution target.
- **Inspect a finished card.** DONE cards open read-only - the detail and the workflow it ran, without launch controls.

From the terminal:

```bash
operator create              # new ticket
operator queue               # show the board
operator launch              # launch the next ticket
```

## Relationship to external providers

Connecting [Jira](/getting-started/kanban/jira/), [Linear](/getting-started/kanban/linear/), [GitHub Projects](/getting-started/kanban/github/) or
another Kanban provider does not replace this board. Their issues are pulled in as markdown tickets here, and with `bidirectional = true`
each column change is pushed back to the originating board.

The Operator board is never a sync *source*. It is the destination, so it takes no `[kanban.operator]` config section and does not appear in `operator sync`.

## Configuration

None. The only related setting is where the tickets live:

```toml
[paths]
tickets = ".tickets"
```

See [Tickets](/getting-started/tickets/) for the ticket format and [Kanban](/getting-started/concepts/kanban/) for the concepts behind the columns.

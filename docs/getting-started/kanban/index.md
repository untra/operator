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

## How It Works

Operator syncs tickets from your kanban provider:

1. **Pull**: Fetches issues from configured boards/projects
2. **Queue**: Orders tickets by priority and type
3. **Assign**: Dispatches tickets to available agents
4. **Update**: Pushes status changes back to your provider

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

Operator also supports local-only tickets in `.tickets/queue/` for projects without external issue tracking. See [Tickets](/tickets/) for details.

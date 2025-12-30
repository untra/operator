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

## How It Works

Operator syncs tickets from your kanban provider:

1. **Pull**: Fetches issues from configured boards/projects
2. **Queue**: Orders tickets by priority and type
3. **Assign**: Dispatches tickets to available agents
4. **Update**: Pushes status changes back to your provider

## Choosing a Provider

Jira Cloud is the currently supported kanban provider. Additional providers may be added in future releases.

## Local Tickets

Operator also supports local-only tickets in `.tickets/queue/` for projects without external issue tracking. See [Tickets](/tickets/) for details.

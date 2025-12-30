---
title: "Supported Notification Integrations"
description: "Notification providers for Operator events."
layout: doc
---

# Supported Notification Integrations

Operator can notify you when important events occur, such as agent completion, failures, or tickets awaiting review.

## Available Integrations

| Provider | Status | Notes |
|----------|--------|-------|
| [Webhooks](/getting-started/notifications/webhooks/) | Supported | HTTP callbacks for any service |
| [Operating System](/getting-started/notifications/os/) | Supported | Native macOS notifications |

## How Notifications Work

When configurable events occur, Operator checks all enabled notification providers and dispatches alerts:

1. **Event triggers** - Agent completes, fails, or needs input
2. **Provider check** - Each enabled provider receives the event
3. **Dispatch** - Notifications sent to all configured endpoints

## Events

The following events can trigger notifications:

| Event | Description |
|-------|-------------|
| `agent.started` | Agent started working on a ticket |
| `agent.completed` | Agent finished work on a ticket |
| `agent.failed` | Agent encountered an error |
| `agent.awaiting_input` | Agent needs human input (SPIKE/INV) |
| `agent.session_lost` | Agent's tmux session terminated unexpectedly |
| `pr.created` | Pull request created |
| `pr.merged` | Pull request merged |
| `pr.closed` | Pull request closed without merge |
| `pr.ready_to_merge` | Pull request approved and ready to merge |
| `pr.changes_requested` | Pull request has changes requested |
| `ticket.returned` | Ticket returned to queue |
| `investigation.created` | Investigation ticket created from alert |

## Configuration

Enable notifications in your config:

```toml
# ~/.config/operator/config.toml

[notifications]
enabled = true

# OS notifications (native system notifications)
[notifications.os]
enabled = true
sound = false
events = []  # Empty = all events

# Webhook (optional)
[notifications.webhook]
enabled = true
url = "https://your-service.com/webhook"
events = ["agent.completed", "agent.failed"]
```

See individual provider pages for provider-specific configuration.

---
title: "Webhooks"
description: "Configure webhook notifications for Operator events."
layout: doc
---

# Webhook Notifications

Send HTTP POST requests to external services when Operator events occur.

## Configuration

See the full [webhook notification configuration reference](/configuration/#notifications-webhook).

Add webhook notifications to your Operator configuration:

```toml
# ~/.config/operator/config.toml

[notifications.webhook]
enabled = true
url = "https://your-service.com/webhook"
```

## Authentication

### Bearer Token

```toml
[notifications.webhook]
enabled = true
url = "https://api.example.com/webhook"
auth_type = "bearer"
token_env = "WEBHOOK_TOKEN"
```

### Basic Auth

```toml
[notifications.webhook]
enabled = true
url = "https://api.example.com/webhook"
auth_type = "basic"
username = "operator"
password_env = "WEBHOOK_PASSWORD"
```

## Payload Format

Webhook requests are sent as JSON with the following structure:

```json
{
  "event": "agent.completed",
  "timestamp": "2024-01-15T10:30:00Z",
  "data": {
    "ticket_id": "PROJ-123",
    "agent": "claude",
    "project": "backend",
    "duration_seconds": 342,
    "result": "success"
  }
}
```

## Multiple Webhooks

Configure multiple webhook endpoints:

```toml
[[notifications.webhooks]]
name = "slack"
url = "https://hooks.slack.com/services/xxx"
events = ["agent.completed", "agent.failed"]

[[notifications.webhooks]]
name = "pagerduty"
url = "https://events.pagerduty.com/v2/enqueue"
events = ["agent.failed"]
auth_type = "bearer"
token_env = "PAGERDUTY_TOKEN"
```

## Integration Examples

### Slack

Use Slack's incoming webhooks:

```toml
[notifications.webhook]
enabled = true
url = "https://hooks.slack.com/services/T00000000/B00000000/XXXXXXXX"
```

### Discord

Use Discord's webhook URL:

```toml
[notifications.webhook]
enabled = true
url = "https://discord.com/api/webhooks/xxx/yyy"
```

## Troubleshooting

### Webhook not firing

Check that events are enabled:

```toml
[notifications]
enabled = true
events = ["agent.completed"]  # Must include the event type
```

### Authentication errors

Verify your token is set:

```bash
echo $WEBHOOK_TOKEN
```

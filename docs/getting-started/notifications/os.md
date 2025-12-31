---
title: "Operating System"
description: "Configure native OS notifications for Operator events."
layout: doc
---

# Operating System Notifications

Display native system notifications when Operator events occur.

## Platform Support

| Platform | Status | Method |
|----------|--------|--------|
| macOS | Supported | `mac-notification-sys` |
| Linux | Supported | `notify-rust` |
| Windows | Planned | Toast notifications |

## Configuration

See the full [OS notification configuration reference](/configuration/#notifications-os).

Enable OS notifications in your Operator configuration:

```toml
# ~/.config/operator/config.toml

[notifications.os]
enabled = true
```

## macOS Setup

On macOS, Operator uses the native notification system. No additional setup required.

### Notification Permissions

If notifications don't appear, check System Settings:

1. Open **System Settings** > **Notifications**
2. Find **Operator** in the app list
3. Ensure notifications are allowed

### Sound

Enable notification sounds:

```toml
[notifications.os]
enabled = true
sound = true  # Play system notification sound
```

## Notification Content

OS notifications display:

- **Title**: Event type (e.g., "Agent Completed")
- **Subtitle**: Project name
- **Body**: Ticket ID and summary

Example notification:

```
Agent Completed
backend
PROJ-123: Add user authentication
```

## Event Filtering

Only receive notifications for specific events:

```toml
[notifications.os]
enabled = true
events = ["agent.failed", "agent.awaiting_input"]
```

## Do Not Disturb

OS notifications respect system Do Not Disturb settings. Notifications will be queued and displayed when DND is disabled.

## Troubleshooting

### Notifications not appearing (macOS)

1. Check notification permissions in System Settings
2. Ensure Operator is not in Focus mode exclusions
3. Try running Operator from Terminal (not background)

### No sound

Verify sound is enabled:

```toml
[notifications.os]
sound = true
```

And check that system volume is not muted.

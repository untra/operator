---
title: "Linear"
description: "Configure Linear integration with Operator."
layout: doc
---

# Linear

Connect Operator to Linear for modern issue tracking and project management.

## Prerequisites

- Linear workspace with team access
- API key for authentication

## Create API Key

1. Go to [Linear Settings](https://linear.app/settings/account/api)
2. Under "Personal API keys", click "Create key"
3. Name it "Operator" and copy the key

## Configuration

Set the required environment variable:

```bash
export OPERATOR_LINEAR_API_KEY="lin_api_xxxxxxxxxxxxx"
```

Add Linear to your Operator configuration (team ID is the key):

```toml
# ~/.config/operator/config.toml

[kanban.linear."team-uuid-here"]
enabled = true
api_key_env = "OPERATOR_LINEAR_API_KEY"  # default

[kanban.linear."team-uuid-here".projects.default]
sync_user_id = "your-linear-user-id"
collection_name = "dev_kanban"
```

### Finding Your Team ID

Your team ID is a UUID visible in Linear URLs when viewing team settings, or via the API:

```bash
curl -H "Authorization: $OPERATOR_LINEAR_API_KEY" \
     -H "Content-Type: application/json" \
     https://api.linear.app/graphql \
     -d '{"query": "{ teams { nodes { id name } } }"}'
```

### Multiple Teams

You can configure multiple Linear teams:

```toml
[kanban.linear."uuid-engineering-team"]
enabled = true
api_key_env = "OPERATOR_LINEAR_API_KEY"

[kanban.linear."uuid-platform-team"]
enabled = true
api_key_env = "OPERATOR_LINEAR_API_KEY"
```

## Issue Mapping

Operator maps Linear labels to ticket types:

| Linear Label | Operator Type |
|--------------|---------------|
| bug | FIX |
| feature | FEAT |
| improvement | FEAT |
| spike | SPIKE |

## Syncing Issues

Pull issues from Linear:

```bash
operator sync
```

## Per-Team Configuration

Configure sync settings for each team:

```toml
[kanban.linear."team-uuid-here".projects.default]
sync_user_id = "user-uuid-here"           # Your Linear user ID
sync_statuses = ["Todo", "In Progress"]   # Statuses to sync (empty = default only)
collection_name = "dev_kanban"            # IssueTypeCollection to use
```

## Troubleshooting

### Authentication errors

Verify your API key:

```bash
curl -H "Authorization: $OPERATOR_LINEAR_API_KEY" \
     -H "Content-Type: application/json" \
     https://api.linear.app/graphql \
     -d '{"query": "{ viewer { id name } }"}'
```

### Missing issues

Check that the user ID and team ID are correct, and that the issues are assigned to the configured user.

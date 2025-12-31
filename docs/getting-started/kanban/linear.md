---
title: "Linear"
description: "Configure Linear integration with Operator."
layout: doc
published: false
---

# Linear

Connect Operator to Linear for modern issue tracking.

## Prerequisites

- Linear workspace
- API key for authentication

## Create API Key

1. Go to Linear Settings > API
2. Click "Create key"
3. Name it "Operator" and copy the key

## Configuration

Add Linear to your Operator configuration:

```toml
# ~/.config/operator/config.toml

[kanban.linear]
enabled = true
api_key_env = "LINEAR_API_KEY"
team_key = "ENG"  # Your team identifier
```

Set your API key:

```bash
export LINEAR_API_KEY="lin_api_xxxxx"
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

## Workflow States

Configure which states to sync:

```toml
[kanban.linear]
states = ["Todo", "In Progress", "Backlog"]
```

## Filtering

Limit sync to specific projects or labels:

```toml
[kanban.linear]
project = "Backend"
labels = ["priority-high", "priority-medium"]
```

## Troubleshooting

### Authentication errors

Test your API key:

```bash
curl -H "Authorization: $LINEAR_API_KEY" \
     https://api.linear.app/graphql \
     -d '{"query": "{ viewer { id } }"}'
```

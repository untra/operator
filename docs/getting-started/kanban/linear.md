---
title: "Linear"
description: "Configure Linear integration with Operator."
layout: doc
---

Connect Operator to [**Linear**](https://linear.app/features) for modern issue tracking and project management.

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

## Relationships

| Linear | Ticket frontmatter |
|--------|--------------------|
| Issue blocked by another (`blocks` relation) | `depends_on: ["linear:ENG-3"]` |
| Sub-issues | `depends_on` - the parent waits for its sub-issues |
| Project | `external_parent: linear:<project-id>` |
| Parent issue (when not in a project) | `external_parent: linear:ENG-1` |

Relationships are pull-only: Operator never writes links or parents back to Linear. On every sync, the `linear:` entries in a ticket's `depends_on` are replaced with what Linear reports now; ticket ids you added yourself are kept. A blocker that isn't synced stays in `depends_on` as `linear:<key>` - the ticket waits on it until it is synced and completed, or the entry is removed. See [Dependencies](/getting-started/tickets/#dependencies-and-campaigns).

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
collection_name = "dev_kanban"            # IssueTypeCollection to use

[kanban.linear."team-uuid-here".projects.default.status_mapping]
todo = "Todo"           # Workflow state pulled into operator's queue (and requeue target)
doing = "In Progress"   # State pushed when a ticket is launched/claimed
done = "Done"           # State pushed when a ticket completes
```

### Column Mapping (todo / doing / done)

`status_mapping` declares which Linear workflow state corresponds to each of
operator's strict todo/doing/done states. Issues are pulled from the `todo`
(and `doing`) states; with `bidirectional = true`, ticket transitions are
pushed back to the mapped states (requeue > `todo` only fires when `todo` is
mapped; unmapped `doing`/`done` fall back to `"In Progress"`/`"Done"`).

Discover the team's real state names via `POST /api/v1/kanban/statuses`
(onboarding) or `GET /api/v1/kanban/linear/TEAM/statuses` (configured team).

> **Migrating from `sync_statuses`:** the old list is no longer read (the key
> is silently ignored). Re-express it as the `status_mapping` table above.

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

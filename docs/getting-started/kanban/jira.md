---
title: "Jira Cloud"
description: "Configure Jira Cloud integration with Operator."
layout: doc
---

Connect Operator to [**Jira Cloud**](https://www.atlassian.com/software/jira) for issue tracking and project management.

## Prerequisites

- Jira Cloud account (not Jira Server/Data Center)
- Project with appropriate permissions
- API token for authentication

## Create API Token

1. Go to [Atlassian Account Settings](https://id.atlassian.com/manage/api-tokens)
2. Click "Create API token"
3. Name it "Operator" and copy the token

## Configuration

Set the required environment variables:

```bash
export OPERATOR_JIRA_DOMAIN="your-org.atlassian.net"
export OPERATOR_JIRA_EMAIL="your-email@example.com"
export OPERATOR_JIRA_API_KEY="your-api-token"
```

Add Jira to your Operator configuration (domain is the key):

```toml
# ~/.config/operator/config.toml

[kanban.jira."your-org.atlassian.net"]
enabled = true
email = "your-email@example.com"
api_key_env = "OPERATOR_JIRA_API_KEY"  # default

[kanban.jira."your-org.atlassian.net".projects.PROJ]
sync_user_id = "your-jira-account-id"
collection_name = "dev_kanban"
```

### Multiple Jira Workspaces

You can configure multiple Jira workspaces with custom environment variable names:

```toml
[kanban.jira."work.atlassian.net"]
enabled = true
email = "work@company.com"
api_key_env = "OPERATOR_JIRA_WORK_API_KEY"

[kanban.jira."personal.atlassian.net"]
enabled = true
email = "personal@example.com"
api_key_env = "OPERATOR_JIRA_PERSONAL_API_KEY"
```

## Issue Mapping

Operator maps Jira issue types to ticket types:

| Jira Type | Operator Type |
|-----------|---------------|
| Bug | FIX |
| Story | FEAT |
| Task | FEAT |
| Spike | SPIKE |

## Syncing Issues

Pull issues from Jira:

```bash
operator sync
```

## Per-Project Configuration

Configure sync settings for each project:

```toml
[kanban.jira."your-org.atlassian.net".projects.PROJ]
sync_user_id = "5e3f7acd9876543210abcdef"  # Your Jira accountId
collection_name = "dev_kanban"               # IssueTypeCollection to use

[kanban.jira."your-org.atlassian.net".projects.PROJ.status_mapping]
todo = "To Do"          # Column pulled into operator's queue (and requeue target)
doing = "In Progress"   # Column pushed when a ticket is launched/claimed
done = "Done"           # Column pushed when a ticket completes
```

### Column Mapping (todo / doing / done)

Operator is strict about its three internal states — todo, doing, done — while
Jira boards have arbitrary columns. `status_mapping` declares which Jira status
corresponds to each operator state:

- Issues are **pulled** from the `todo` (and `doing`) columns into the queue.
- With `bidirectional = true`, launching a ticket moves the Jira issue to
  `doing`, completing it moves it to `done`, and returning it to the queue
  moves it back to `todo` (requeue only pushes when `todo` is mapped).
- Unmapped `doing`/`done` fall back to `"In Progress"`/`"Done"` on push.

Discover the board's real column names via
`POST /api/v1/kanban/statuses` (onboarding) or
`GET /api/v1/kanban/jira/PROJ/statuses` (configured project) — the VS Code
config panel uses these to populate the mapping dropdowns.

> **Migrating from `sync_statuses`:** the old
> `sync_statuses = ["To Do", "In Progress"]` list is no longer read (the key is
> silently ignored). Re-express it as the explicit `status_mapping` table above.

## Troubleshooting

### Authentication errors

Verify your credentials:

```bash
curl -u email:token https://your-org.atlassian.net/rest/api/3/myself
```

### Missing issues

Check your JQL query and permissions in Jira.

## API Reference

The full set of Jira REST calls Operator makes is documented in the generated
[Jira API Reference](/getting-started/kanban/jira-api/).

---
title: "Gitea Projects"
description: "Import assigned issues from Gitea repository project boards."
layout: doc
---

Gitea Projects is an **Alpha, import-only** kanban provider. It discovers repository boards and columns and imports assigned issues into Operator tickets. Organization and personal boards, pull requests, and upstream writes are excluded.

## Connect

Use the web setup wizard or VS Code's **Connect Kanban Provider** flow and select **Gitea Projects**. Enter the instance URL, repository (`owner/name`), and token; select a board, sync user, and source column. The TUI's configure action opens this guide for TOML setup.

Create a personal access token at your instance's `/user/settings/applications`. Select `read:user`, `read:repository`, and `read:issue`; the account must be able to read the repository, its project boards, issues, and assignees. Public-only tokens cannot import private repositories. Administrative or write scopes are not required. See [Gitea token scopes](https://docs.gitea.com/development/oauth2-provider/#scopes).

Set `OPERATOR_GITEA_TOKEN` for persistence. Interactive onboarding retains the token in an instance-bound session store; restart requires the environment variable. Use separate variable names for multiple instances. TOML contains only the variable name.

```toml
[kanban.gitea.work]
enabled = true
base_url = "https://git.example.com"
api_key_env = "OPERATOR_GITEA_TOKEN"
repositories = ["team/repository"]

[kanban.gitea.work.projects."https://git.example.com/team/repository/projects/1"]
sync_user_id = "42"
bidirectional = false
ticket_project = "repository"

[kanban.gitea.work.projects."https://git.example.com/team/repository/projects/1".status_mapping]
todo = "To Do"
```

Project keys are absolute URLs returned by discovery, including any instance path prefix. User IDs are numeric account IDs, returned during onboarding. Without a mapped source column, imports use the default column, then the first ordered column. Column names must be distinct and nonempty. Repository labels supply issue-type mappings.

## Compatibility and failures

The server must expose repository project, column, and column-issue GET endpoints under `/api/v1/repos/{owner}/{repo}/projects`. Operator probes these capabilities during onboarding; older Gitea versions may lack them. The contract follows the [published Gitea project API](https://docs.gitea.com/api/operations/repo-list-projects/).

401 means an invalid/expired token. 403 means missing scopes or account permissions. 404 can mean an inaccessible repository or board, or an unavailable project API; check the instance's `/swagger.v1.json`. Rate limiting, network errors, and incomplete pagination fail the import without creating tickets from a partial result. Deleted or renamed mapped columns require updating configuration.

Board and issue identities include the instance and repository. Repeated imports and cards appearing on multiple selected boards do not create duplicate tickets. Bidirectional configuration and mutation calls are rejected; moving or completing an Operator ticket does not move its upstream card.

This token is distinct from `GITEA_TOKEN`, used by the [Git provider](/getting-started/git/gitea/) for PR operations. Forgejo boards have a [separate compatibility status](/getting-started/kanban/forgejo/).

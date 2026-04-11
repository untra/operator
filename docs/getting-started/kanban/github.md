---
title: "GitHub Projects"
description: "Configure GitHub Projects v2 integration with Operator."
layout: doc
---

# GitHub Projects

Connect Operator to [**GitHub Projects v2**](https://docs.github.com/en/issues/planning-and-tracking-with-projects/learning-about-projects/about-projects) for issue tracking and project management.

> **⚠ Token Disambiguation — read this first**
>
> GitHub Projects uses a **separate** API token from Operator's git provider (the one that creates pull requests). Even if you've already set `GITHUB_TOKEN` for PR workflows, you'll need a *second* token in `OPERATOR_GITHUB_TOKEN` with the `project` (or `read:project`) scope. The two **can** be the same physical PAT minted with both scopes — but they must be exposed via two different environment variables so Operator can route them correctly.
>
> | Operator subsystem            | Env var                  | Required scopes                                  | Configured at                      |
> |-------------------------------|--------------------------|--------------------------------------------------|------------------------------------|
> | Git provider (PRs, branches)  | `GITHUB_TOKEN`           | `repo` (or fine-grained Contents + PRs)          | `[git.github]`                     |
> | Kanban provider (Projects v2) | `OPERATOR_GITHUB_TOKEN`  | `project` or `read:project` (or fine-grained Projects) | `[kanban.github."<owner>"]` |
>
> Operator deliberately **does not** fall back from `OPERATOR_GITHUB_TOKEN` to `GITHUB_TOKEN`. Silently using a repo-scoped token would produce confusing 403s deep in the sync loop. If only `GITHUB_TOKEN` is set, the kanban provider stays inactive.

## Prerequisites

- A GitHub account with access to at least one Project v2 (user-owned or org-owned)
- A Personal Access Token (PAT) — classic or fine-grained — with the `project` scope, or a GitHub App installation token with `organization_projects: write`
- Operator installed and running

## Create a Token

You have two options. **Fine-grained PATs are recommended** because they're scoped to specific orgs/repos and have built-in expiration.

### Option A — Classic Personal Access Token (simpler)

1. Go to [github.com/settings/tokens](https://github.com/settings/tokens)
2. Click **Generate new token (classic)**
3. Name it something like *"Operator Kanban (read+write)"*
4. Select scopes:
   - `project` (full read + write to Projects v2) — **or** `read:project` (read-only)
   - Optionally `read:org` if you need to enumerate org projects
5. Click **Generate token**, then copy the `ghp_...` value

### Option B — Fine-Grained Personal Access Token (recommended)

1. Go to [github.com/settings/personal-access-tokens](https://github.com/settings/personal-access-tokens)
2. Click **Generate new token**
3. **Resource owner**: select the user or org that owns the projects you want to sync
4. **Repository access**: select the repos whose issues should appear as project items (use *Public Repositories* for read-only org-wide access, or *Selected repositories* for tighter scoping)
5. **Permissions**:
   - **Organization → Projects**: Read-and-write (or Read-only)
   - **Repository → Issues**: Read (so issue content is fetched alongside project items)
   - **Repository → Contents**: Read (only if you also want body/labels)
6. Click **Generate token**, then copy the `github_pat_...` value

## Configuration

### 1. Export the token

```bash
# Kanban projects token (this guide)
export OPERATOR_GITHUB_TOKEN="ghp_xxxxxxxxxxxxxxxx"

# Optional: separate token for git/PR operations (NOT this guide)
export GITHUB_TOKEN="ghp_yyyyyyyyyyyyyyyy"
```

### 2. Add a kanban section to `~/.config/operator/config.toml`

```toml
[kanban.github."my-org"]
enabled = true
api_key_env = "OPERATOR_GITHUB_TOKEN"  # default

[kanban.github."my-org".projects.PVT_kwDOABcdefg]
sync_user_id = "12345678"        # numeric GitHub `databaseId`
sync_statuses = ["In Progress", "Todo"]
collection_name = "dev_kanban"
```

The hashmap key under `[kanban.github."<owner>"]` is the GitHub owner login (user or org). Project keys inside `projects` are **GraphQL node IDs** (e.g. `PVT_kwDOABcdefg`) — not project numbers — because every Projects v2 mutation needs the node ID and storing it directly avoids an extra lookup per call.

### 3. Multiple Owners with Different Tokens

You can scope distinct tokens per owner via `api_key_env`:

```toml
[kanban.github."my-personal-account"]
enabled = true
api_key_env = "OPERATOR_GITHUB_TOKEN"          # personal PAT

[kanban.github."my-employer-org"]
enabled = true
api_key_env = "OPERATOR_GITHUB_WORK_TOKEN"     # work fine-grained PAT
```

Then set both env vars:

```bash
export OPERATOR_GITHUB_TOKEN="ghp_personal..."
export OPERATOR_GITHUB_WORK_TOKEN="github_pat_work..."
```

## Finding Your Project Node ID

Easiest path is via `gh`:

```bash
gh api graphql -f query='
query {
  viewer {
    projectsV2(first: 10) {
      nodes { id number title owner { ... on Organization { login } ... on User { login } } }
    }
  }
}
'
```

For org-owned projects:

```bash
gh api graphql -f query='
query($login: String!) {
  organization(login: $login) {
    projectsV2(first: 20) {
      nodes { id number title }
    }
  }
}
' -F login=my-org
```

The `id` field is what you put in `[kanban.github."<owner>".projects.<id>]`.

If you'd rather skip this step, use the **VS Code extension** or **Operator TUI** onboarding flow — both will list your projects after validating your token and write the config for you.

## Finding Your `sync_user_id`

`sync_user_id` is your GitHub user's numeric `databaseId` (NOT your login string). The validation step in onboarding fetches this for you, but you can also get it manually:

```bash
gh api user --jq .id
# 12345678
```

Or via GraphQL:

```bash
gh api graphql -f query='query { viewer { databaseId login } }'
```

## Issue Mapping

Operator's GitHub Projects provider exposes issue types via two paths, in order of preference:

1. **Org-level Issue Types** (recommended where available) — the new first-class GitHub feature. See [docs.github.com/en/issues/tracking-your-work-with-issues/configuring-issues/managing-issue-types-in-an-organization](https://docs.github.com/en/issues/tracking-your-work-with-issues/configuring-issues/managing-issue-types-in-an-organization). If your org has issue types configured, the provider exposes them directly.
2. **Repo labels (fallback)** — when issue types aren't available (user-owned projects or orgs without the feature), the provider aggregates labels from all repos linked through project items.

Configure mappings via `type_mappings` in your `ProjectSyncConfig`:

| GitHub source                  | Operator type |
|--------------------------------|---------------|
| `bug` (label) / `Bug` (issue type)            | `FIX`         |
| `feature` (label) / `Feature` (issue type)    | `FEAT`        |
| `enhancement` (label)                         | `FEAT`        |
| `spike` (label) / `Spike` (issue type)        | `SPIKE`       |

Operator's `kanban_issuetype_service` syncs the available types into a local catalog at `.tickets/operator/kanban/github/<project_id>/issuetypes.json` after onboarding completes.

## Per-Project Configuration

```toml
[kanban.github."my-org".projects.PVT_kwDOABcdefg]
sync_user_id = "12345678"                  # your numeric GitHub databaseId
sync_statuses = ["In Progress", "Todo"]    # Status field option names to sync
collection_name = "dev_kanban"             # IssueTypeCollection to use

[kanban.github."my-org".projects.PVT_kwDOABcdefg.type_mappings]
"L_bug"     = "FIX"
"L_feature" = "FEAT"
"L_spike"   = "SPIKE"
```

The keys in `type_mappings` are the GraphQL label IDs (or issue type IDs) returned by `get_issue_types()` — they're persisted in the local issue type catalog after the first sync, and you can find them with:

```bash
cat .tickets/operator/kanban/github/PVT_kwDOABcdefg/issuetypes.json
```

## Syncing Issues

Pull issues from GitHub Projects:

```bash
operator sync
```

The provider client-side filters by your `sync_user_id` (project items don't support server-side assignee filtering in the GraphQL API), so very large projects may pull a few extra pages before applying the filter. Status filtering uses the `Status` single-select field's option names — make sure the values in `sync_statuses` exactly match the names defined in your project (case-insensitive).

### What gets synced

- **Real issues** linked to the project
- **Pull requests** linked to the project
- **Draft issues** (project-only items, no underlying repo issue)

The `key` field on the synced ticket follows these formats:

| Item type     | Key format                |
|---------------|---------------------------|
| Issue         | `octocat/hello#42`        |
| Pull request  | `octocat/hello!42`        |
| Draft issue   | `draft:PVTI_lAHO_xxxxxxx` |

## Creating New Issues

For v1, the GitHub Projects provider creates **draft issues only** via the `addProjectV2DraftIssue` mutation. Draft issues live inside the project (not in any repo) and can be promoted to real issues later from the GitHub UI.

If you need real repo issues, create them through GitHub's normal flows — they'll appear in operator after the next sync if they're added to a project the operator is configured for.

## Troubleshooting

### "Token authenticated but lacks 'project' scope"

This is the disambiguation guard rail firing. It means the token reached GitHub's API successfully but doesn't have the `project` scope — most likely you accidentally pasted your `GITHUB_TOKEN` (which is repo-scoped for PR workflows). Re-mint a token with the `project` (or `read:project`) scope and re-run onboarding.

If you're using a fine-grained PAT and you're sure it has Projects permissions, double-check the **Resource owner** matches the org/user whose projects you're trying to sync — fine-grained PATs are scoped per resource owner.

### Authentication errors

Verify your token reaches the API:

```bash
curl -H "Authorization: bearer $OPERATOR_GITHUB_TOKEN" \
     -H "User-Agent: operator" \
     https://api.github.com/graphql \
     -d '{"query":"{ viewer { login databaseId } }"}'
```

For classic PATs, also check the response headers — they include `x-oauth-scopes`:

```bash
curl -i -H "Authorization: bearer $OPERATOR_GITHUB_TOKEN" \
     https://api.github.com/user 2>&1 | grep -i x-oauth-scopes
# x-oauth-scopes: project, read:org, repo
```

If `project` (or `read:project`) is missing, that's your problem.

### Missing issues

- Confirm `sync_user_id` is the numeric `databaseId`, **not** your login. `gh api user --jq .id` returns the right value.
- Confirm the issue is actually assigned to that user. Operator filters client-side after fetching, so unassigned items are dropped silently.
- Confirm the issue's Status field value appears in `sync_statuses`. Match is case-insensitive but must otherwise be exact.
- For huge projects (>500 items), check the operator logs for pagination warnings.

### "No GitHub Projects v2 found for this token"

Either your token genuinely has no project access, or the projects you expected to see aren't visible to the authenticated user. For org projects, you may need `read:org` scope (classic) or *Members → Read* permission (fine-grained) so the org enumeration works.

## See Also

- [Jira Cloud setup](./jira.md)
- [Linear setup](./linear.md)
- [Kanban workflow overview](../../kanban/index.md)

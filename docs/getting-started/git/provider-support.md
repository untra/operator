---
title: "Provider Support"
description: "Architecture guide for adding new Git provider integrations."
layout: doc
---

This guide explains how Operator integrates with Git hosting providers, and
how to add support for a new one.

## Support Tiers

Statuses follow the [feature maturity](/maturity/) scale. 

| Provider | CLI | Tier | Operations |
|----------|-----|------|------------|
| [GitHub](/getting-started/git/github/) | `gh` | Beta | Full - read, create, list, read comments |
| [GitLab](/getting-started/git/gitlab/) | `glab` | Alpha | Full - read, create, list, read comments |
| Bitbucket | `bb` | Proto | Detection only |
| Azure DevOps | `az` | Proto | Detection only |
| Forgejo | `fj` | Proto | Detection only |
| [Gitea](/getting-started/git/gitea/) | `tea api` | Alpha | Read, create, list, comments, reviews |

"Detection only" means Operator recognizes the provider from its remote URL and can report which CLI to install, but has no operational `PrService`
implementation yet - creating, reading, or listing code review requests (including reading their comments) for that provider isn't wired up.

Delegator Git identity and supplied HTTPS credentials are provider-independent, including for detect-only providers. PR authentication is operational for GitHub, GitLab, and Gitea. Forgejo CLI integration remains deferred.

## Architecture

```text
PrWorkflow / PrMonitorService
            |
      PrServiceRouter (configuration + captured delegator context)
       /             |               \
GitHubService   GitLabService    GiteaService
      |               |               |
    GhCli           GlabCli          TeaCli
      |               |               |
     gh              glab          tea api
```

Gitea uses Tea's JSON API interface and per-invocation private login configuration. Reads retry transient failures; PR creation reconciles an ambiguous response before returning an error and never blindly repeats the POST.


`GitHubService` and `GitLabService` both wrap their CLI with exponential
backoff retry (via `backon`) and implement the same trait, so every layer
above the retry services is provider-agnostic. Bitbucket, Azure DevOps,
Forgejo, and Gitea are detected (`GitProvider::from_remote_url`) but have no
service/CLI-wrapper pair yet, so `pr_service_for` returns an
`UnsupportedProviderError` for them.

### The `PrService` trait

`src/api/pr_service.rs` defines the provider-agnostic contract every service
implements:

```rust
#[async_trait]
pub trait PrService: Send + Sync {
    /// Get the provider name (e.g., "github", "gitlab")
    fn provider_name(&self) -> &str;

    /// Check if the service is available and authenticated
    async fn check_available(&self) -> Result<bool>;

    /// Get the authenticated user
    async fn get_authenticated_user(&self) -> Result<String>;

    /// Get PR/MR information
    async fn get_pr(&self, repo_info: &RepoInfo, pr_number: i64) -> Result<PullRequestInfo>;

    /// Check if PR/MR is ready to merge (approved + checks pass)
    async fn is_ready_to_merge(&self, repo_info: &RepoInfo, pr_number: i64) -> Result<bool>;

    /// Get the review state of a PR/MR
    async fn get_review_state(&self, repo_info: &RepoInfo, pr_number: i64)
        -> Result<PrReviewState>;

    /// Create a new PR/MR
    async fn create_pr(
        &self,
        repo_info: &RepoInfo,
        request: &CreatePrRequest,
        cwd: &Path,
    ) -> Result<PullRequestInfo, CreatePrError>;

    /// List PRs/MRs for a branch
    async fn list_prs_for_branch(
        &self,
        repo_info: &RepoInfo,
        branch: &str,
    ) -> Result<Vec<PullRequestInfo>>;

    /// Get all comments on a PR/MR
    async fn get_all_comments(
        &self,
        repo_info: &RepoInfo,
        pr_number: i64,
    ) -> Result<Vec<UnifiedPrComment>>;

    /// Open PR/MR in browser
    async fn open_in_browser(&self, repo_info: &RepoInfo, pr_number: i64) -> Result<()>;

    /// Get comments since a given time
    async fn get_comments_since(
        &self,
        repo_info: &RepoInfo,
        pr_number: i64,
        since: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<UnifiedPrComment>>;

    /// Find an existing PR for a branch
    async fn find_pr_for_branch(
        &self,
        repo_info: &RepoInfo,
        branch: &str,
    ) -> Result<Option<PullRequestInfo>>;
}
```

`PrServiceRouter` also implements `PrService`: it holds a resolver
(`pr_service_for` by default) and dispatches every per-repo call to the
operational service for `repo_info.provider`. `provider_name()`,
`check_available()`, and `get_authenticated_user()` take no `RepoInfo`, so the
router falls back to GitHub for those and reports its own name as `"auto"`.

## Terminology

Providers name the same underlying concepts differently. Operator's code and
UI default to the neutral term "code review request (PR/MR)"; provider brand
names appear only when talking about that specific provider.

| Concept | GitHub | GitLab | Bitbucket | Azure DevOps | Forgejo | Gitea |
|---------|--------|--------|-----------|--------------|---------|-------|
| Code Review Request | Pull Request | Merge Request | Pull Request | Pull Request | Pull Request | Pull Request |
| CI Status | Checks | Pipelines | Build status | Checks | Checks | Checks |
| CI Automation | Actions | CI/CD | Pipelines | Azure Pipelines | Actions | Actions |
| Approval | Review | Approval | Approval | Approval | Review | Review |

GitHub and GitLab are operational today; Bitbucket, Azure DevOps, Forgejo,
and Gitea terminology above is provided for reference ahead of their
`PrService` implementations.

## Provider Detection

`src/types/pr.rs` detects the provider straight from a repo's remote URL -
there's no separate "detect" step to configure:

- `GitProvider::from_remote_url(remote_url: &str) -> Option<GitProvider>` -
  matches on hostname substrings (`github.com`, `gitlab.` / `gitlab.com`,
  `bitbucket.org`, `dev.azure.com` / `visualstudio.com`, `codeberg.org`,
  `gitea.com`).
- `RepoInfo::from_remote_url(remote_url: &str) -> Result<RepoInfo, RepoInfoError>` -
  calls the above, then parses `owner`/`repo_name` out of the URL with a
  provider-specific regex.

GitLab subgroups are supported: `https://gitlab.com/group/subgroup/repo.git`
parses to `owner = "group/subgroup"`, `repo_name = "repo"` (the owner regex
for GitLab captures everything up to the final path segment, unlike the other
providers' single-segment owner).

## Configuration

Git provider configuration lives in `src/config/git_config.rs`:

- `GitConfig` - top-level `[git]` table: `provider` (`Option<GitProviderConfig>`,
  auto-detected from the remote when unset), `github` (`GitHubConfig`), `gitlab`
  (`GitLabConfig`), `branch_format` (default `"{type}/{ticket_id}"`), and
  `use_worktrees` (default `false`).
- `GitProviderConfig` - the explicit-override enum (`GitHub`, `GitLab`,
  `Bitbucket`, `AzureDevOps`, `Forgejo`, `Gitea`), serialized `#[serde(rename_all
  = "lowercase")]`; `From<GitProviderConfig> for GitProvider` converts it into
  the runtime enum used by `pr_service_for`.
- `GitHubConfig` - `enabled` (default `true`), `token_env` (default
  `"GITHUB_TOKEN"`).
- `GitLabConfig` - `enabled` (default `false`), `token_env` (default
  `"GITLAB_TOKEN"`), `host` (`Option<String>`, for self-hosted instances).

```toml
[git]
provider = "gitlab"            # optional; auto-detected from the remote if omitted
branch_format = "{type}/{ticket_id}"
use_worktrees = false

[git.github]
enabled = true
token_env = "GITHUB_TOKEN"

[git.gitlab]
enabled = true
token_env = "GITLAB_TOKEN"
host = "gitlab.example.com"    # self-hosted instances only
```

## Code-Review Gating

Once a code review request exists, `PrMonitorService`
(`src/services/pr_monitor.rs`) polls it every 60 seconds through the same
`PrService` (routed via `PrServiceRouter`), watching for merge, close,
approval, changes-requested, and ready-to-merge/ready-for-review transitions.

An agent working a ticket carries a `review_state` marker
(`src/state.rs`) while it waits on a human:

- `pending_pr_creation` - the agent finished and Operator is opening the PR/MR.
- `pending_pr_merge` - the PR/MR is open and awaiting merge.

Both surface in the in-progress panel (`src/ui/in_progress_panel.rs`) with
their own icon and status text. A human resolves the gate either from the
TUI's agents panel (`y` to approve, `x` to reject - see
`src/ui/keybindings.rs`) or via REST (`POST /api/v1/agents/{agent_id}/approve`
/ `.../reject`, `src/rest/routes/agents.rs`), which write a review signal file
the agent picks back up. See [Supported Coding Agents](/getting-started/agents/)
for the agent-side half of this flow.

## Adding a New Provider

Checklist for taking a provider from detect-only to fully operational (this
is what GitHub and GitLab already went through):

1. **Enum variants** - add the provider to both `GitProvider`
   (`src/types/pr.rs`, `#[serde(rename_all = "lowercase")]`) and
   `GitProviderConfig` (`src/config/git_config.rs`), plus the
   `From<GitProviderConfig> for GitProvider` arm.
2. **Catalog entry** - add a row in `src/integrations/catalog.rs`'s
   `all_integrations()` under `Vertical::Git`. `tests/vertical_parity.rs`
   enforces the tier rules: `Alpha`+ needs a `docs_path` pointing at a real
   docs page; `Beta`+ additionally needs `readme_badge: true` (and a matching
   README badge).
3. **CLI detection row** - add a `CliSpec` to `CLI_SPECS` in
   `src/api/cli_detection.rs` so `detect_all_clis`/`detect_for` can probe the
   new CLI.
4. **CLI wrapper + retry service + `PrService` impl** - a `src/api/<provider>_cli.rs`
   wrapper (mirroring `GhCli`/`GlabCli`), a `src/api/<provider>_service.rs`
   retry wrapper around it (mirroring `GitHubService`/`GitLabService`), and a
   `PrService` impl for that service in `src/api/pr_service.rs`.
5. **Router wiring** - add the new service to the `match` in `pr_service_for`
   (`src/api/pr_service.rs`), replacing its `UnsupportedProviderError` arm.
6. **Onboarding metadata** - once operational, add a `ProviderMeta` entry (and
   a `meta_for` arm) in `src/app/git_onboarding.rs` so the TUI's onboarding
   flow can walk a user through CLI install / token setup for it.
7. **Live test row** - add a `ProviderCase` to `PROVIDER_CASES` in
   `tests/gitprovider_integration.rs` (a template comment there shows the
   shape) plus a `#[tokio::test]` calling `run_provider_case` for it.
8. **Docs page + nav** - a `docs/getting-started/git/<provider>/index.md`
   page, added to `navigation.yml`.

Forgejo (CLI `fj`, see
[codeberg.org/forgejo-contrib/forgejo-cli](https://codeberg.org/forgejo-contrib/forgejo-cli))
and Gitea (CLI `tea`, see
[gitea.com/gitea/tea](https://gitea.com/gitea/tea)) are the most likely next
operational candidates - both are currently detect-only (`Proto`).

## Legacy Note

`src/api/providers/repo/` is a separate, experimental REST-polling module
(its own `RepoProvider` trait, currently only a `GitHubProvider` impl). It
predates the `PrService` stack, is not part of the provider contract
described above, and is slated for consolidation into `PrService`. Don't
extend it for new provider support - follow the checklist above instead.

## Testing

`tests/gitprovider_integration.rs` drives the live `PrService` stack
(`gh`/`glab`) against real, external test repositories. It's strictly
read-only - no create/write operations against any provider. Table-shape
invariants (env-var name uniqueness, slug/provider matching) run under plain
`cargo test`, no setup required:

```bash
cargo test --test gitprovider_integration provider_case_table
```

The live per-provider tests are opt-in and gated on:

- `OPERATOR_GITPROVIDER_TEST_ENABLED=true` - required to run any test in the
  file.
- `OPERATOR_GITPROVIDER_TEST_REPO_GITHUB` - full remote URL of a GitHub test
  repo (e.g. `https://github.com/owner/repo`).
- `OPERATOR_GITPROVIDER_TEST_PR_GITHUB` - number of an open GitHub PR on that
  repo with **at least one comment**.
- `OPERATOR_GITPROVIDER_TEST_REPO_GITLAB` - full remote URL of a GitLab test
  repo.
- `OPERATOR_GITPROVIDER_TEST_PR_GITLAB` - number of an open GitLab MR on that
  repo with **at least one comment**.

Each row also needs its CLI (`gh`, `glab`) installed and authenticated; a row
skips itself (rather than failing the suite) if its env var is unset or its
CLI isn't available:

```bash
OPERATOR_GITPROVIDER_TEST_ENABLED=true \
    OPERATOR_GITPROVIDER_TEST_REPO_GITHUB=https://github.com/owner/repo \
    OPERATOR_GITPROVIDER_TEST_PR_GITHUB=1 \
    OPERATOR_GITPROVIDER_TEST_REPO_GITLAB=https://gitlab.com/owner/repo \
    OPERATOR_GITPROVIDER_TEST_PR_GITLAB=1 \
    cargo test --test gitprovider_integration -- --nocapture
```

The comment requirement matters: `get_all_comments` is asserted non-empty, so
the test repo's designated PR/MR needs at least one existing comment (general
or inline) before the run.

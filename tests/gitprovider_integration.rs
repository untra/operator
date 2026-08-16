//! Live integration tests for the `PrService` stack (GitHub, GitLab, ...)
//!
//! These tests drive the real provider CLIs (`gh`, `glab`) through
//! `pr_service_for`/`PrService` against designated, real, external test
//! repositories. They are strictly read-only: no create/write operations
//! against any provider are performed anywhere in this file.
//!
//! ## Environment Variables
//!
//! - `OPERATOR_GITPROVIDER_TEST_ENABLED=true`: Required to run any test in
//!   this file.
//! - `OPERATOR_GITPROVIDER_TEST_REPO_GITHUB`: Full remote URL of a GitHub
//!   test repo (e.g. `https://github.com/owner/repo`).
//! - `OPERATOR_GITPROVIDER_TEST_PR_GITHUB`: Number of an open GitHub PR on
//!   that repo with at least one comment.
//! - `OPERATOR_GITPROVIDER_TEST_REPO_GITLAB`: Full remote URL of a GitLab
//!   test repo.
//! - `OPERATOR_GITPROVIDER_TEST_PR_GITLAB`: Number of an open GitLab MR on
//!   that repo with at least one comment.
//!
//! Each row additionally needs its CLI (`gh`, `glab`) installed and
//! authenticated. A row skips itself (with `eprintln!`) rather than failing
//! the suite when its repo env var is unset or its CLI is unavailable.
//!
//! ## Running Tests
//!
//! ```bash
//! # Both rows
//! OPERATOR_GITPROVIDER_TEST_ENABLED=true \
//!     OPERATOR_GITPROVIDER_TEST_REPO_GITHUB=https://github.com/owner/repo \
//!     OPERATOR_GITPROVIDER_TEST_PR_GITHUB=1 \
//!     OPERATOR_GITPROVIDER_TEST_REPO_GITLAB=https://gitlab.com/owner/repo \
//!     OPERATOR_GITPROVIDER_TEST_PR_GITLAB=1 \
//!     cargo test --test gitprovider_integration -- --nocapture
//!
//! # Table-invariant unit test only (no env vars, no CLIs needed)
//! cargo test --test gitprovider_integration provider_case_table
//! ```

use operator::api::pr_service::pr_service_for;
use operator::api::PrService;
use operator::types::pr::{GitProvider, PrState, RepoInfo};
use std::collections::HashSet;
use std::env;
use std::sync::Arc;

// ─── Configuration Helpers ───────────────────────────────────────────────────

/// Check if git-provider tests are enabled
fn gitprovider_tests_enabled() -> bool {
    env::var("OPERATOR_GITPROVIDER_TEST_ENABLED")
        .map(|v| v == "true" || v == "1")
        .unwrap_or(false)
}

/// Macro to skip test if git-provider tests are not configured
macro_rules! skip_if_not_configured {
    () => {
        if !gitprovider_tests_enabled() {
            eprintln!("Skipping test: OPERATOR_GITPROVIDER_TEST_ENABLED not set to true");
            return;
        }
    };
}

// ─── Provider Table ──────────────────────────────────────────────────────────

/// One row of the live provider test table. Adding a new operational
/// provider (bitbucket, azure, forgejo, gitea) is just adding a row here
/// plus a `#[tokio::test]` that calls `run_provider_case` for it -- see the
/// template comment below `PROVIDER_CASES`.
struct ProviderCase {
    /// Matches `GitProvider::slug()` for this row's provider.
    slug: &'static str,
    /// The CLI binary this row's `PrService` shells out to.
    cli: &'static str,
    /// Env var holding the full remote URL of the live test repo.
    repo_env: &'static str,
    /// Env var holding the PR/MR number to exercise (must have >=1 comment).
    pr_env: &'static str,
    /// Builds the `PrService` under test for this row.
    service: fn() -> Arc<dyn PrService>,
}

const PROVIDER_CASES: &[ProviderCase] = &[
    ProviderCase {
        slug: "github",
        cli: "gh",
        repo_env: "OPERATOR_GITPROVIDER_TEST_REPO_GITHUB",
        pr_env: "OPERATOR_GITPROVIDER_TEST_PR_GITHUB",
        service: || {
            pr_service_for(GitProvider::GitHub, &operator::config::GitConfig::default())
                .expect("github is operational")
        },
    },
    ProviderCase {
        slug: "gitlab",
        cli: "glab",
        repo_env: "OPERATOR_GITPROVIDER_TEST_REPO_GITLAB",
        pr_env: "OPERATOR_GITPROVIDER_TEST_PR_GITLAB",
        service: || {
            pr_service_for(GitProvider::GitLab, &operator::config::GitConfig::default())
                .expect("gitlab is operational")
        },
    },
    ProviderCase {
        slug: "gitea",
        cli: "tea",
        repo_env: "OPERATOR_GITPROVIDER_TEST_REPO_GITEA",
        pr_env: "OPERATOR_GITPROVIDER_TEST_PR_GITEA",
        service: || {
            let mut config = operator::config::GiteaConfig::default();
            if let Ok(remote) = env::var("OPERATOR_GITPROVIDER_TEST_REPO_GITEA") {
                let url = url::Url::parse(&remote).expect("Gitea test remote must be HTTPS");
                config.host = Some(url.origin().ascii_serialization());
            }
            Arc::new(operator::api::gitea_service::GiteaService::new(config))
        },
    },
    // Template for a future row (documentation only, NOT dead code -- copy
    // this into a real `ProviderCase` once the provider gets an operational
    // `PrService` impl in `src/api/pr_service.rs::pr_service_for`):
    //
    // ProviderCase {
    //     slug: "bitbucket",       // GitProvider::Bitbucket.slug()
    //     cli: "bb" or whatever CLI backs BitbucketService,
    //     repo_env: "OPERATOR_GITPROVIDER_TEST_REPO_BITBUCKET",
    //     pr_env: "OPERATOR_GITPROVIDER_TEST_PR_BITBUCKET",
    //     service: || pr_service_for(GitProvider::Bitbucket).expect("bitbucket is operational"),
    // },
    //
    // Azure DevOps, Forgejo, and Gitea rows follow the same shape with
    // `GitProvider::AzureDevOps` / `Forgejo` / `Gitea` and their own
    // `OPERATOR_GITPROVIDER_TEST_{REPO,PR}_{AZUREDEVOPS,FORGEJO,GITEA}` pair.
];

/// Look up a row by slug (stable against reordering `PROVIDER_CASES`).
fn case_by_slug(slug: &str) -> &'static ProviderCase {
    PROVIDER_CASES
        .iter()
        .find(|case| case.slug == slug)
        .unwrap_or_else(|| panic!("no ProviderCase for slug {slug}"))
}

// ─── Shared Row Runner ────────────────────────────────────────────────────────

/// Drive one row of `PROVIDER_CASES` through the live `PrService` stack.
/// Skips (with `eprintln!`) rather than fails when the row isn't configured
/// or its CLI isn't installed/authed -- a missing row must not fail the
/// suite.
async fn run_provider_case(case: &ProviderCase) {
    skip_if_not_configured!();

    let Ok(repo_url) = env::var(case.repo_env) else {
        eprintln!("Skipping {} row: {} not set", case.slug, case.repo_env);
        return;
    };

    let service = (case.service)();

    // 1. check_available
    match service.check_available().await {
        Ok(true) => {}
        Ok(false) => {
            eprintln!(
                "Skipping {} row: {} CLI not installed or not authenticated",
                case.slug, case.cli
            );
            return;
        }
        Err(e) => {
            eprintln!("Skipping {} row: check_available errored: {e}", case.slug);
            return;
        }
    }

    // 2. get_authenticated_user
    let user = service
        .get_authenticated_user()
        .await
        .unwrap_or_else(|e| panic!("{}: get_authenticated_user failed: {e}", case.slug));
    assert!(
        !user.is_empty(),
        "{}: authenticated user name should not be empty",
        case.slug
    );
    eprintln!("{}: authenticated as {user}", case.slug);

    // 3. RepoInfo::from_remote_url
    let mut git = operator::config::GitConfig::default();
    if case.slug == "gitea" {
        git.gitea.host = Some(
            url::Url::parse(&repo_url)
                .unwrap()
                .origin()
                .ascii_serialization(),
        );
    }
    let hosts = operator::types::pr::ProviderHosts::from_config(&git).unwrap();
    let repo_info = RepoInfo::from_remote_url_with_hosts(&repo_url, &hosts)
        .unwrap_or_else(|e| panic!("{}: failed to parse repo url {repo_url}: {e}", case.slug));
    assert_eq!(
        repo_info.provider.slug(),
        case.slug,
        "{}: parsed provider should match row slug",
        case.slug
    );

    let Ok(pr_number_str) = env::var(case.pr_env) else {
        eprintln!(
            "Skipping remaining {} assertions: {} not set",
            case.slug, case.pr_env
        );
        return;
    };
    let pr_number: i64 = pr_number_str
        .parse()
        .unwrap_or_else(|e| panic!("{}: {} must be an integer: {e}", case.slug, case.pr_env));

    // 4. get_pr
    let pr = service
        .get_pr(&repo_info, pr_number)
        .await
        .unwrap_or_else(|e| panic!("{}: get_pr({pr_number}) failed: {e}", case.slug));
    assert!(
        matches!(pr.state, PrState::Open | PrState::Merged | PrState::Closed),
        "{}: PR state should be a valid variant",
        case.slug
    );
    eprintln!(
        "{}: PR #{pr_number} state={:?} title={:?}",
        case.slug, pr.state, pr.title
    );

    // 5. get_review_state -- external review state drifts, any variant is fine
    match service.get_review_state(&repo_info, pr_number).await {
        Ok(state) => eprintln!("{}: review state = {state:?}", case.slug),
        Err(e) => panic!("{}: get_review_state failed: {e}", case.slug),
    }

    // 6. get_all_comments
    let comments = service
        .get_all_comments(&repo_info, pr_number)
        .await
        .unwrap_or_else(|e| panic!("{}: get_all_comments failed: {e}", case.slug));
    eprintln!("{}: {} comments found", case.slug, comments.len());
    assert!(
        !comments.is_empty(),
        "{}: {} promises PR #{pr_number} has >=1 comment, found none",
        case.slug,
        case.pr_env
    );

    // 7. find_pr_for_branch -- `PullRequestInfo` doesn't expose the source
    // branch, so it isn't cheaply obtainable from `get_pr`'s result above.
    // Per the tolerant-tests rule, skip this assertion rather than guess.
    eprintln!(
        "{}: skipping find_pr_for_branch assertion -- PullRequestInfo has no branch field",
        case.slug
    );
}

// ─── Live Tests (one per row) ─────────────────────────────────────────────────

#[tokio::test]
async fn test_github_provider_live() {
    run_provider_case(case_by_slug("github")).await;
}

#[tokio::test]
async fn test_gitlab_provider_live() {
    run_provider_case(case_by_slug("gitlab")).await;
}

// ─── CLI Flag Contract (no credentials, no network) ──────────────────────────

/// Every long flag an argv builder emits must appear in the installed CLI's
/// own `--help`. This needs the binary but no auth and no repository, so it
/// runs anywhere the CLI is installed.
///
/// This is the check that catches a flag the binary does not accept -- the
/// class of bug that made `gh pr create --json` fail on flag parse while
/// every unit test still passed.
#[test]
fn provider_cli_flags_are_accepted_by_the_installed_binary() {
    use operator::api::argv::ProviderCommand;
    use operator::types::pr::CreatePrRequest;

    let request = CreatePrRequest {
        title: "Contract check".into(),
        body: Some("Body".into()),
        head_branch: "optest/source".into(),
        base_branch: "main".into(),
        draft: Some(true),
    };

    let cases: Vec<ProviderCommand> = vec![
        operator::api::GhCli::create_pr_argv(
            &RepoInfo::new(GitProvider::GitHub, "owner", "repo"),
            &request,
        ),
        operator::api::GlabCli::create_pr_argv(
            &RepoInfo::new(GitProvider::GitLab, "owner", "repo"),
            &request,
        ),
    ];

    let mut checked = 0;
    for cmd in cases {
        let help = match std::process::Command::new(cmd.program)
            .args(cmd.subcommand())
            .arg("--help")
            .output()
        {
            Ok(out) if out.status.success() => {
                format!(
                    "{}{}",
                    String::from_utf8_lossy(&out.stdout),
                    String::from_utf8_lossy(&out.stderr)
                )
            }
            _ => {
                eprintln!(
                    "Skipping {} {}: CLI not installed",
                    cmd.program,
                    cmd.subcommand().join(" ")
                );
                continue;
            }
        };
        checked += 1;
        for flag in cmd.long_flags() {
            assert!(
                help.contains(flag),
                "`{} {}` does not accept `{flag}` -- it is absent from that \
                 subcommand's --help",
                cmd.program,
                cmd.subcommand().join(" ")
            );
        }
    }
    eprintln!("verified argv flags against {checked} installed CLI(s)");
}

// ─── Table Invariants (plain `cargo test`, no gating) ────────────────────────

#[test]
fn provider_case_table_env_names_unique_and_slugs_match() {
    let mut repo_envs = HashSet::new();
    let mut pr_envs = HashSet::new();

    for case in PROVIDER_CASES {
        assert!(
            repo_envs.insert(case.repo_env),
            "duplicate repo_env in PROVIDER_CASES: {}",
            case.repo_env
        );
        assert!(
            pr_envs.insert(case.pr_env),
            "duplicate pr_env in PROVIDER_CASES: {}",
            case.pr_env
        );

        assert!(
            GitProvider::ALL.iter().any(|p| p.slug() == case.slug),
            "{}: no GitProvider matches this row's slug",
            case.slug
        );

        let service = (case.service)();
        assert_eq!(
            service.provider_name(),
            case.slug,
            "{}: PrService::provider_name() should match the row's slug",
            case.slug
        );
    }
}

#[tokio::test]
async fn gitea_provider_live() {
    run_provider_case(PROVIDER_CASES.iter().find(|c| c.slug == "gitea").unwrap()).await;
}

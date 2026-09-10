//! Live integration tests for the `PrService` stack (GitHub, GitLab, Gitea)
//!
//! These tests drive the real provider CLIs (`gh`, `glab`, `tea`) through
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
//! - `OPERATOR_GITPROVIDER_TEST_REPO_GITEA`: Full remote URL of a Gitea test
//!   repo. Gitea is self-hosted, so its origin also registers the Gitea host.
//! - `OPERATOR_GITPROVIDER_TEST_PR_GITEA`: Number of an open Gitea PR on that
//!   repo with at least one comment.
//!
//! Each row additionally needs its CLI (`gh`, `glab`, `tea`) installed and
//! authenticated. A row skips itself (with `eprintln!`) rather than failing
//! the suite when its repo env var is unset or its CLI is unavailable.
//!
//! A row env var counts as unset when it is missing **or blank**: CI wires
//! these from repository variables that export an empty string when
//! undefined, and a blank value must skip the row rather than be parsed.
//!
//! ## Running Tests
//!
//! ```bash
//! # Every row
//! OPERATOR_GITPROVIDER_TEST_ENABLED=true \
//!     OPERATOR_GITPROVIDER_TEST_REPO_GITHUB=https://github.com/owner/repo \
//!     OPERATOR_GITPROVIDER_TEST_PR_GITHUB=1 \
//!     OPERATOR_GITPROVIDER_TEST_REPO_GITLAB=https://gitlab.com/owner/repo \
//!     OPERATOR_GITPROVIDER_TEST_PR_GITLAB=1 \
//!     OPERATOR_GITPROVIDER_TEST_REPO_GITEA=https://gitea.example/owner/repo \
//!     OPERATOR_GITPROVIDER_TEST_PR_GITEA=1 \
//!     cargo test --test gitprovider_integration -- --nocapture
//!
//! # Table-invariant unit test only (no env vars, no CLIs needed)
//! cargo test --test gitprovider_integration provider_case_table
//! ```

use operator::api::pr_service::pr_service_for;
use operator::api::PrService;
use operator::config::GitConfig;
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

/// A set-but-blank value reads as unset -- see the blank-value note in the
/// module docs.
fn non_blank(value: String) -> Option<String> {
    (!value.trim().is_empty()).then_some(value)
}

/// The value of `var`, or `None` when it is unset or blank.
fn configured(var: &str) -> Option<String> {
    env::var(var).ok().and_then(non_blank)
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
/// provider (bitbucket, azure, forgejo) is just adding a row here
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
    /// Registers the test repo's origin as this provider's host. `None` for
    /// providers resolved by well-known domain; `Some` for self-hosted ones,
    /// which are unresolvable until their host is configured.
    apply_host: Option<fn(&mut GitConfig, String)>,
    /// Builds the `PrService` under test for this row.
    service: fn(&GitConfig) -> Arc<dyn PrService>,
}

const PROVIDER_CASES: &[ProviderCase] = &[
    ProviderCase {
        slug: "github",
        cli: "gh",
        repo_env: "OPERATOR_GITPROVIDER_TEST_REPO_GITHUB",
        pr_env: "OPERATOR_GITPROVIDER_TEST_PR_GITHUB",
        apply_host: None,
        service: |git| pr_service_for(GitProvider::GitHub, git).expect("github is operational"),
    },
    ProviderCase {
        slug: "gitlab",
        cli: "glab",
        repo_env: "OPERATOR_GITPROVIDER_TEST_REPO_GITLAB",
        pr_env: "OPERATOR_GITPROVIDER_TEST_PR_GITLAB",
        apply_host: None,
        service: |git| pr_service_for(GitProvider::GitLab, git).expect("gitlab is operational"),
    },
    ProviderCase {
        slug: "gitea",
        cli: "tea",
        repo_env: "OPERATOR_GITPROVIDER_TEST_REPO_GITEA",
        pr_env: "OPERATOR_GITPROVIDER_TEST_PR_GITEA",
        apply_host: Some(|git, origin| git.gitea.host = Some(origin)),
        service: |git| pr_service_for(GitProvider::Gitea, git).expect("gitea is operational"),
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
    //     apply_host: None,
    //     service: |git| pr_service_for(GitProvider::Bitbucket, git).expect("operational"),
    // },
    //
    // Azure DevOps and Forgejo follow the same shape once `pr_service_for`
    // stops returning `UnsupportedProviderError` for them. A self-hosted row
    // also needs `apply_host`, as the `gitea` row above does.
];

/// Look up a row by slug (stable against reordering `PROVIDER_CASES`).
fn case_by_slug(slug: &str) -> &'static ProviderCase {
    PROVIDER_CASES
        .iter()
        .find(|case| case.slug == slug)
        .unwrap_or_else(|| panic!("no ProviderCase for slug {slug}"))
}

// ─── Shared Row Runner ────────────────────────────────────────────────────────

/// The `GitConfig` a row's service and URL parsing both resolve against.
/// Self-hosted rows register the live repo's own origin as their host.
fn git_config_for(case: &ProviderCase, repo_url: &str) -> GitConfig {
    let mut git = GitConfig::default();
    if let Some(apply_host) = case.apply_host {
        let origin = url::Url::parse(repo_url)
            .unwrap_or_else(|e| {
                panic!(
                    "{}: {} must be an absolute URL: {e}",
                    case.slug, case.repo_env
                )
            })
            .origin()
            .ascii_serialization();
        apply_host(&mut git, origin);
    }
    git
}

/// Drive one row of `PROVIDER_CASES` through the live `PrService` stack.
/// Skips (with `eprintln!`) rather than fails when the row isn't configured
/// or its CLI isn't installed/authed -- a missing row must not fail the
/// suite.
async fn run_provider_case(case: &ProviderCase) {
    skip_if_not_configured!();

    let Some(repo_url) = configured(case.repo_env) else {
        eprintln!("Skipping {} row: {} not set", case.slug, case.repo_env);
        return;
    };

    let git = git_config_for(case, &repo_url);
    let service = (case.service)(&git);

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
    let hosts = operator::types::pr::ProviderHosts::from_config(&git).unwrap();
    let repo_info = RepoInfo::from_remote_url_with_hosts(&repo_url, &hosts)
        .unwrap_or_else(|e| panic!("{}: failed to parse repo url {repo_url}: {e}", case.slug));
    assert_eq!(
        repo_info.provider.slug(),
        case.slug,
        "{}: parsed provider should match row slug",
        case.slug
    );

    let Some(pr_number_str) = configured(case.pr_env) else {
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

#[tokio::test]
async fn test_gitea_provider_live() {
    run_provider_case(case_by_slug("gitea")).await;
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

/// CI wires every row with `${{ vars.X || '' }}`, which exports an empty
/// string when the variable is undefined -- a blank value must read as unset.
#[test]
fn blank_env_values_are_treated_as_unset() {
    for blank in ["", "   ", "\n"] {
        assert_eq!(
            non_blank(blank.to_string()),
            None,
            "blank value {blank:?} should read as unset"
        );
    }
    assert_eq!(
        non_blank("https://github.com/owner/repo".to_string()).as_deref(),
        Some("https://github.com/owner/repo")
    );
}

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

        let service = (case.service)(&GitConfig::default());
        assert_eq!(
            service.provider_name(),
            case.slug,
            "{}: PrService::provider_name() should match the row's slug",
            case.slug
        );
    }
}

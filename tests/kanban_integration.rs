//! Integration tests for Kanban providers (Jira, Linear, and GitHub Projects)
//!
//! These tests require real API credentials and test workspaces.
//! They are skipped when credentials are not available.
//!
//! ## Environment Variables Required
//!
//! ### For Jira:
//! - `OPERATOR_JIRA_DOMAIN`: Jira instance domain (e.g., "company.atlassian.net")
//! - `OPERATOR_JIRA_EMAIL`: Account email
//! - `OPERATOR_JIRA_API_KEY`: API token
//! - `OPERATOR_JIRA_TEST_PROJECT`: Test project key (e.g., "TEST")
//!
//! ### For Linear:
//! - `OPERATOR_LINEAR_API_KEY`: Linear API key
//! - `OPERATOR_LINEAR_TEST_TEAM`: Test team ID (UUID)
//!
//! ### For GitHub Projects:
//! - `OPERATOR_GITHUB_TOKEN`: PAT with `project` (or `read:project`) scope.
//!   MUST be distinct from `GITHUB_TOKEN` used for PR workflows — the kanban
//!   provider deliberately does not fall back. See
//!   `docs/getting-started/kanban/github.md`.
//! - `OPERATOR_GITHUB_TEST_PROJECT`: `ProjectV2` `GraphQL` node ID
//!   (starts with `PVT_`). Fetch via:
//!   `gh api graphql -f query='query { viewer { projectsV2(first: 20) { nodes { id number title } } } }'`.
//!   The project must have a Status single-select field with at least one
//!   terminal option (Done/Complete/Closed/Resolved) — default GitHub
//!   project templates satisfy this.
//!
//! ## Running Tests
//!
//! ```bash
//! # All integration tests
//! cargo test --test kanban_integration
//!
//! # Jira tests only
//! cargo test --test kanban_integration jira_tests -- --nocapture --test-threads=1
//!
//! # Linear tests only
//! cargo test --test kanban_integration linear_tests -- --nocapture --test-threads=1
//!
//! # GitHub tests only
//! cargo test --test kanban_integration github_tests -- --nocapture --test-threads=1
//! ```
//!
//! ## A note on GitHub test drafts
//!
//! `GithubProjectsProvider::create_issue` (v1) produces draft issues via
//! `AddProjectV2DraftIssueInput`. The provider does not expose item
//! deletion, so test drafts are moved to a terminal status (Done) for
//! cleanup but remain in the project. Periodically filter the test project
//! by the `[OPTEST]` prefix and archive manually, or point
//! `OPERATOR_GITHUB_TEST_PROJECT` at a dedicated throwaway project.

use operator::api::providers::kanban::{
    CreateIssueRequest, GithubProjectsProvider, JiraProvider, KanbanProvider, LinearProvider,
    UpdateStatusRequest,
};
use std::env;
use tokio::sync::OnceCell;

// Cached credential validation results
static JIRA_CREDENTIALS_VALID: OnceCell<bool> = OnceCell::const_new();
static LINEAR_CREDENTIALS_VALID: OnceCell<bool> = OnceCell::const_new();
static GITHUB_CREDENTIALS_VALID: OnceCell<bool> = OnceCell::const_new();

// ─── Configuration Helpers ───────────────────────────────────────────────────

/// Check if Jira credentials are configured (non-empty env vars)
fn jira_configured() -> bool {
    env::var("OPERATOR_JIRA_DOMAIN")
        .map(|s| !s.is_empty())
        .unwrap_or(false)
        && env::var("OPERATOR_JIRA_EMAIL")
            .map(|s| !s.is_empty())
            .unwrap_or(false)
        && env::var("OPERATOR_JIRA_API_KEY")
            .map(|s| !s.is_empty())
            .unwrap_or(false)
        && env::var("OPERATOR_JIRA_TEST_PROJECT")
            .map(|s| !s.is_empty())
            .unwrap_or(false)
}

/// Check if Linear credentials are configured (non-empty env vars)
fn linear_configured() -> bool {
    env::var("OPERATOR_LINEAR_API_KEY")
        .map(|s| !s.is_empty())
        .unwrap_or(false)
        && env::var("OPERATOR_LINEAR_TEST_TEAM")
            .map(|s| !s.is_empty())
            .unwrap_or(false)
}

/// Check if GitHub Projects credentials are configured (non-empty env vars).
///
/// Only `OPERATOR_GITHUB_TOKEN` is consulted — the provider deliberately does
/// NOT fall back to `GITHUB_TOKEN` (which is reserved for the git/PR provider
/// and typically lacks the `project` scope). See
/// `src/api/providers/kanban/github_projects.rs` module docs.
fn github_configured() -> bool {
    env::var("OPERATOR_GITHUB_TOKEN")
        .map(|s| !s.is_empty())
        .unwrap_or(false)
        && env::var("OPERATOR_GITHUB_TEST_PROJECT")
            .map(|s| !s.is_empty())
            .unwrap_or(false)
}

/// Get the Jira test project key
fn jira_test_project() -> String {
    env::var("OPERATOR_JIRA_TEST_PROJECT").expect("OPERATOR_JIRA_TEST_PROJECT required")
}

/// Get the Linear test team ID
fn linear_test_team() -> String {
    env::var("OPERATOR_LINEAR_TEST_TEAM").expect("OPERATOR_LINEAR_TEST_TEAM required")
}

/// Get the GitHub test project node ID (e.g. `PVT_kwDOABC123`)
fn github_test_project() -> String {
    env::var("OPERATOR_GITHUB_TEST_PROJECT").expect("OPERATOR_GITHUB_TEST_PROJECT required")
}

/// Generate a unique test issue title with [OPTEST] prefix
fn test_issue_title(suffix: &str) -> String {
    let uuid = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    format!("[OPTEST] {suffix} - {uuid}")
}

/// Find a terminal status (Done, Complete, Completed, Closed, Resolved) from available statuses.
/// Returns the first matching status name (case-insensitive).
fn find_terminal_status(statuses: &[String]) -> Option<String> {
    const TERMINAL_PATTERNS: &[&str] = &["done", "complete", "completed", "closed", "resolved"];
    for pattern in TERMINAL_PATTERNS {
        if let Some(status) = statuses.iter().find(|s| s.eq_ignore_ascii_case(pattern)) {
            return Some(status.clone());
        }
    }
    None
}

/// Validate Jira credentials by testing the connection.
/// Result is cached for the duration of the test run.
async fn jira_credentials_valid() -> bool {
    if !jira_configured() {
        return false;
    }

    *JIRA_CREDENTIALS_VALID
        .get_or_init(|| async {
            match JiraProvider::from_env() {
                Ok(provider) => match provider.test_connection().await {
                    Ok(valid) => {
                        if !valid {
                            eprintln!(
                                "Jira credentials validation failed: connection test returned false"
                            );
                        }
                        valid
                    }
                    Err(e) => {
                        eprintln!("Jira credentials validation failed: {e}");
                        false
                    }
                },
                Err(e) => {
                    eprintln!("Jira provider initialization failed: {e}");
                    false
                }
            }
        })
        .await
}

/// Validate Linear credentials by testing the connection.
/// Result is cached for the duration of the test run.
async fn linear_credentials_valid() -> bool {
    if !linear_configured() {
        return false;
    }

    *LINEAR_CREDENTIALS_VALID
        .get_or_init(|| async {
            match LinearProvider::from_env() {
                Ok(provider) => match provider.test_connection().await {
                    Ok(valid) => {
                        if !valid {
                            eprintln!(
                                "Linear credentials validation failed: connection test returned false"
                            );
                        }
                        valid
                    }
                    Err(e) => {
                        eprintln!("Linear credentials validation failed: {e}");
                        false
                    }
                },
                Err(e) => {
                    eprintln!("Linear provider initialization failed: {e}");
                    false
                }
            }
        })
        .await
}

/// Validate GitHub Projects credentials by testing the connection.
/// Result is cached for the duration of the test run.
async fn github_credentials_valid() -> bool {
    if !github_configured() {
        return false;
    }

    *GITHUB_CREDENTIALS_VALID
        .get_or_init(|| async {
            match GithubProjectsProvider::from_env() {
                Ok(provider) => match provider.test_connection().await {
                    Ok(valid) => {
                        if !valid {
                            eprintln!(
                                "GitHub credentials validation failed: connection test returned false"
                            );
                        }
                        valid
                    }
                    Err(e) => {
                        eprintln!("GitHub credentials validation failed: {e}");
                        false
                    }
                },
                Err(e) => {
                    eprintln!("GitHub provider initialization failed: {e}");
                    false
                }
            }
        })
        .await
}

/// Macro to skip test if provider is not configured or credentials are invalid
macro_rules! skip_if_not_configured {
    ($configured:expr, $valid:expr, $provider:expr) => {
        if !$configured {
            eprintln!("Skipping test: {} credentials not configured", $provider);
            return;
        }
        if !$valid.await {
            eprintln!(
                "Skipping test: {} credentials invalid or expired",
                $provider
            );
            return;
        }
    };
}

// ─── Jira Integration Tests ──────────────────────────────────────────────────

mod jira_tests {
    use super::*;

    fn get_provider() -> JiraProvider {
        JiraProvider::from_env().expect("Jira provider should be configured")
    }

    #[tokio::test]
    async fn test_connection() {
        skip_if_not_configured!(jira_configured(), jira_credentials_valid(), "Jira");
        let provider = get_provider();

        let result = provider.test_connection().await;
        assert!(result.is_ok(), "Connection test failed: {result:?}");
        assert!(result.unwrap(), "Connection should be valid");
    }

    #[tokio::test]
    async fn test_list_projects() {
        skip_if_not_configured!(jira_configured(), jira_credentials_valid(), "Jira");
        let provider = get_provider();

        let projects = provider
            .list_projects()
            .await
            .expect("Should list projects");
        assert!(!projects.is_empty(), "Should have at least one project");

        // Verify test project exists
        let test_project = jira_test_project();
        assert!(
            projects.iter().any(|p| p.key == test_project),
            "Test project {} should exist in {:?}",
            test_project,
            projects.iter().map(|p| &p.key).collect::<Vec<_>>()
        );
    }

    #[tokio::test]
    async fn test_list_users() {
        skip_if_not_configured!(jira_configured(), jira_credentials_valid(), "Jira");
        let provider = get_provider();
        let project = jira_test_project();

        let users = provider
            .list_users(&project)
            .await
            .expect("Should list users");
        assert!(!users.is_empty(), "Should have assignable users");

        // Verify users have required fields
        for user in &users {
            assert!(!user.id.is_empty(), "User should have ID");
            assert!(!user.name.is_empty(), "User should have name");
        }
    }

    #[tokio::test]
    async fn test_list_statuses() {
        skip_if_not_configured!(jira_configured(), jira_credentials_valid(), "Jira");
        let provider = get_provider();
        let project = jira_test_project();

        let statuses = provider
            .list_statuses(&project)
            .await
            .expect("Should list statuses");
        assert!(!statuses.is_empty(), "Should have workflow statuses");

        eprintln!("Available Jira statuses: {statuses:?}");
    }

    #[tokio::test]
    async fn test_get_issue_types() {
        skip_if_not_configured!(jira_configured(), jira_credentials_valid(), "Jira");
        let provider = get_provider();
        let project = jira_test_project();

        let types = provider
            .get_issue_types(&project)
            .await
            .expect("Should get issue types");
        assert!(!types.is_empty(), "Should have issue types");

        eprintln!(
            "Available Jira issue types: {:?}",
            types.iter().map(|t| &t.name).collect::<Vec<_>>()
        );
    }

    #[tokio::test]
    async fn test_list_issues() {
        skip_if_not_configured!(jira_configured(), jira_credentials_valid(), "Jira");
        let provider = get_provider();
        let project = jira_test_project();

        // Get a user to filter by
        let users = provider
            .list_users(&project)
            .await
            .expect("Should list users");
        if users.is_empty() {
            eprintln!("No assignable users, skipping list_issues test");
            return;
        }

        let user_id = &users[0].id;

        // List issues (may be empty, that's OK)
        let issues = provider
            .list_issues(&project, user_id, &[])
            .await
            .expect("Should list issues");

        eprintln!("Found {} issues for user {}", issues.len(), users[0].name);

        // Verify issue structure if any exist
        for issue in &issues {
            assert!(!issue.id.is_empty(), "Issue should have ID");
            assert!(!issue.key.is_empty(), "Issue should have key");
            assert!(!issue.summary.is_empty(), "Issue should have summary");
        }
    }

    #[tokio::test]
    async fn test_create_issue() {
        skip_if_not_configured!(jira_configured(), jira_credentials_valid(), "Jira");
        let provider = get_provider();
        let project = jira_test_project();

        let request = CreateIssueRequest {
            summary: test_issue_title("Jira Create Test"),
            description: Some("Created by integration test - safe to delete".to_string()),
            assignee_id: None,
            status: None,
            priority: None,
        };

        let response = provider
            .create_issue(&project, request)
            .await
            .expect("Should create issue");

        eprintln!("Created Jira issue: {}", response.issue.key);

        assert!(
            response.issue.key.starts_with(&project),
            "Issue key should start with project"
        );
        assert!(
            response.issue.summary.contains("[OPTEST]"),
            "Issue should have OPTEST prefix"
        );
    }

    #[tokio::test]
    async fn test_update_issue_status() {
        skip_if_not_configured!(jira_configured(), jira_credentials_valid(), "Jira");
        let provider = get_provider();
        let project = jira_test_project();

        // First create an issue
        let request = CreateIssueRequest {
            summary: test_issue_title("Jira Status Test"),
            description: Some("Testing status transition".to_string()),
            assignee_id: None,
            status: None,
            priority: None,
        };

        let created = provider
            .create_issue(&project, request)
            .await
            .expect("Should create issue");

        eprintln!(
            "Created issue {} in status: {}",
            created.issue.key, created.issue.status
        );

        // Get available statuses
        let statuses = provider
            .list_statuses(&project)
            .await
            .expect("Should get statuses");

        // Find a different status to transition to
        let target_status = statuses
            .iter()
            .find(|s| *s != &created.issue.status)
            .cloned();

        if let Some(target) = target_status {
            eprintln!("Transitioning to: {target}");

            let update_request = UpdateStatusRequest {
                status: target.clone(),
            };

            // Note: This may fail if the transition isn't valid from current state
            match provider
                .update_issue_status(&created.issue.key, update_request)
                .await
            {
                Ok(updated) => {
                    eprintln!("New status: {}", updated.status);
                    // Status may not match exactly due to workflow rules
                }
                Err(e) => {
                    eprintln!("Transition failed (may be expected): {e}");
                }
            }
        } else {
            eprintln!("No alternative status available for transition test");
        }
    }
}

// ─── Linear Integration Tests ────────────────────────────────────────────────

mod linear_tests {
    use super::*;

    fn get_provider() -> LinearProvider {
        LinearProvider::from_env().expect("Linear provider should be configured")
    }

    #[tokio::test]
    async fn test_connection() {
        skip_if_not_configured!(linear_configured(), linear_credentials_valid(), "Linear");
        let provider = get_provider();

        let result = provider.test_connection().await;
        assert!(result.is_ok(), "Connection test failed: {result:?}");
        assert!(result.unwrap(), "Connection should be valid");
    }

    #[tokio::test]
    async fn test_list_projects() {
        skip_if_not_configured!(linear_configured(), linear_credentials_valid(), "Linear");
        let provider = get_provider();

        let teams = provider.list_projects().await.expect("Should list teams");
        assert!(!teams.is_empty(), "Should have at least one team");

        // Verify test team exists
        let test_team = linear_test_team();
        assert!(
            teams.iter().any(|t| t.id == test_team),
            "Test team {} should exist in {:?}",
            test_team,
            teams.iter().map(|t| (&t.id, &t.name)).collect::<Vec<_>>()
        );
    }

    #[tokio::test]
    async fn test_list_users() {
        skip_if_not_configured!(linear_configured(), linear_credentials_valid(), "Linear");
        let provider = get_provider();
        let team = linear_test_team();

        let users = provider
            .list_users(&team)
            .await
            .expect("Should list team members");
        assert!(!users.is_empty(), "Team should have members");

        for user in &users {
            assert!(!user.id.is_empty(), "User should have ID");
            assert!(!user.name.is_empty(), "User should have name");
        }

        eprintln!("Found {} Linear team members", users.len());
    }

    #[tokio::test]
    async fn test_list_statuses() {
        skip_if_not_configured!(linear_configured(), linear_credentials_valid(), "Linear");
        let provider = get_provider();
        let team = linear_test_team();

        let statuses = provider
            .list_statuses(&team)
            .await
            .expect("Should list statuses");
        assert!(!statuses.is_empty(), "Should have workflow states");

        eprintln!("Available Linear statuses: {statuses:?}");
    }

    #[tokio::test]
    async fn test_get_issue_types() {
        skip_if_not_configured!(linear_configured(), linear_credentials_valid(), "Linear");
        let provider = get_provider();
        let team = linear_test_team();

        // Linear uses labels as issue types
        let types = provider
            .get_issue_types(&team)
            .await
            .expect("Should get labels");

        eprintln!(
            "Available Linear labels: {:?}",
            types.iter().map(|t| &t.name).collect::<Vec<_>>()
        );
    }

    #[tokio::test]
    async fn test_list_issues() {
        skip_if_not_configured!(linear_configured(), linear_credentials_valid(), "Linear");
        let provider = get_provider();
        let team = linear_test_team();

        let users = provider.list_users(&team).await.expect("Should list users");
        if users.is_empty() {
            eprintln!("No team members, skipping list_issues test");
            return;
        }

        let user_id = &users[0].id;
        let issues = provider
            .list_issues(&team, user_id, &[])
            .await
            .expect("Should list issues");

        eprintln!("Found {} issues for user {}", issues.len(), users[0].name);

        for issue in &issues {
            assert!(!issue.id.is_empty());
            assert!(!issue.key.is_empty());
        }
    }

    #[tokio::test]
    async fn test_create_issue() {
        skip_if_not_configured!(linear_configured(), linear_credentials_valid(), "Linear");
        let provider = get_provider();
        let team = linear_test_team();

        let request = CreateIssueRequest {
            summary: test_issue_title("Linear Create Test"),
            description: Some("Created by integration test - safe to delete".to_string()),
            assignee_id: None,
            status: None,
            priority: None,
        };

        let response = provider
            .create_issue(&team, request)
            .await
            .expect("Should create issue");

        eprintln!("Created Linear issue: {}", response.issue.key);

        assert!(
            response.issue.summary.contains("[OPTEST]"),
            "Issue should have OPTEST prefix"
        );

        // ─── Cleanup: Move issue to terminal status ────────────────────────────────
        let statuses = provider
            .list_statuses(&team)
            .await
            .expect("Should get statuses for cleanup");

        if let Some(done_status) = find_terminal_status(&statuses) {
            if !response.issue.status.eq_ignore_ascii_case(&done_status) {
                eprintln!(
                    "Cleanup: Moving issue {} to {}",
                    response.issue.key, done_status
                );
                let _ = provider
                    .update_issue_status(
                        &response.issue.key,
                        UpdateStatusRequest {
                            status: done_status,
                        },
                    )
                    .await;
            }
        }
    }

    #[tokio::test]
    async fn test_update_issue_status() {
        skip_if_not_configured!(linear_configured(), linear_credentials_valid(), "Linear");
        let provider = get_provider();
        let team = linear_test_team();

        // First create an issue
        let request = CreateIssueRequest {
            summary: test_issue_title("Linear Status Test"),
            description: Some("Testing status transition".to_string()),
            assignee_id: None,
            status: None,
            priority: None,
        };

        let created = provider
            .create_issue(&team, request)
            .await
            .expect("Should create issue");

        eprintln!(
            "Created issue {} in status: {}",
            created.issue.key, created.issue.status
        );

        // Get available statuses
        let statuses = provider
            .list_statuses(&team)
            .await
            .expect("Should get statuses");

        // Find terminal status for later cleanup
        let terminal_status = find_terminal_status(&statuses);

        // Find a different status to transition to (not terminal, to test intermediate transition)
        let target_status = statuses
            .iter()
            .find(|s| {
                *s != &created.issue.status
                    && terminal_status
                        .as_ref()
                        .is_none_or(|t| !s.eq_ignore_ascii_case(t))
            })
            .cloned();

        let mut current_status = created.issue.status.clone();

        if let Some(target) = target_status {
            eprintln!("Transitioning to: {target}");

            let update_request = UpdateStatusRequest {
                status: target.clone(),
            };

            let updated = provider
                .update_issue_status(&created.issue.key, update_request)
                .await
                .expect("Should update status");

            eprintln!("New status: {}", updated.status);
            assert_eq!(
                updated.status.to_lowercase(),
                target.to_lowercase(),
                "Status should be updated"
            );
            current_status = updated.status;
        } else {
            eprintln!("No alternative status available for transition test");
        }

        // ─── Cleanup: Move issue to terminal status (Done) ─────────────────────────
        if let Some(done_status) = terminal_status {
            // Only transition if not already in terminal status
            if current_status.eq_ignore_ascii_case(&done_status) {
                eprintln!("Issue already in terminal status: {current_status}");
            } else {
                eprintln!("Cleanup: Transitioning issue to terminal status: {done_status}");

                let done_request = UpdateStatusRequest {
                    status: done_status.clone(),
                };

                match provider
                    .update_issue_status(&created.issue.key, done_request)
                    .await
                {
                    Ok(final_issue) => {
                        eprintln!(
                            "Issue {} moved to terminal status: {}",
                            final_issue.key, final_issue.status
                        );
                        assert_eq!(
                            final_issue.status.to_lowercase(),
                            done_status.to_lowercase(),
                            "Issue should be in terminal status"
                        );
                    }
                    Err(e) => {
                        eprintln!(
                            "Warning: Could not move issue to terminal status '{done_status}': {e}"
                        );
                        // Don't fail the test - cleanup is best-effort
                    }
                }
            }
        } else {
            eprintln!("Warning: No terminal status found in available statuses: {statuses:?}");
        }
    }
}

// ─── GitHub Projects Integration Tests ───────────────────────────────────────

mod github_tests {
    use super::*;

    fn get_provider() -> GithubProjectsProvider {
        GithubProjectsProvider::from_env().expect("GitHub provider should be configured")
    }

    #[tokio::test]
    async fn test_connection() {
        skip_if_not_configured!(github_configured(), github_credentials_valid(), "GitHub");
        let provider = get_provider();

        let result = provider.test_connection().await;
        assert!(result.is_ok(), "Connection test failed: {result:?}");
        assert!(result.unwrap(), "Connection should be valid");
    }

    #[tokio::test]
    async fn test_list_projects() {
        skip_if_not_configured!(github_configured(), github_credentials_valid(), "GitHub");
        let provider = get_provider();

        let projects = provider
            .list_projects()
            .await
            .expect("Should list projects");
        assert!(!projects.is_empty(), "Should have at least one project");

        // GitHub populates both id and key with the ProjectV2 node_id
        // (src/api/providers/kanban/github_projects.rs list_projects).
        let test_project = github_test_project();
        assert!(
            projects.iter().any(|p| p.key == test_project),
            "Test project {} should exist in {:?}",
            test_project,
            projects
                .iter()
                .map(|p| (&p.key, &p.name))
                .collect::<Vec<_>>()
        );
    }

    #[tokio::test]
    async fn test_list_users() {
        skip_if_not_configured!(github_configured(), github_credentials_valid(), "GitHub");
        let provider = get_provider();
        let project = github_test_project();

        // GitHub derives users from assignees on existing project items
        // (list_users scans items). A fresh test project with only draft
        // issues may legitimately return an empty list — unlike Jira/Linear
        // where team members/assignable users are a separate endpoint.
        let users = provider
            .list_users(&project)
            .await
            .expect("Should list users (may be empty for fresh project)");

        eprintln!("Found {} GitHub project assignees", users.len());

        for user in &users {
            assert!(!user.id.is_empty(), "User should have ID");
            assert!(!user.name.is_empty(), "User should have name");
        }
    }

    #[tokio::test]
    async fn test_list_statuses() {
        skip_if_not_configured!(github_configured(), github_credentials_valid(), "GitHub");
        let provider = get_provider();
        let project = github_test_project();

        let statuses = provider
            .list_statuses(&project)
            .await
            .expect("Should list statuses");

        // A configured Status field is a hard prerequisite — the create/
        // update_status tests below depend on it. Fail loudly if missing.
        assert!(
            !statuses.is_empty(),
            "Test project must have a Status single-select field with at \
             least one option. See docs/getting-started/kanban/github.md."
        );

        eprintln!("Available GitHub statuses: {statuses:?}");
    }

    #[tokio::test]
    async fn test_get_issue_types() {
        skip_if_not_configured!(github_configured(), github_credentials_valid(), "GitHub");
        let provider = get_provider();
        let project = github_test_project();

        // May be empty: only orgs with issue types enabled return a
        // non-empty list, otherwise the provider falls back to aggregated
        // labels from linked repos, which may also be empty.
        let types = provider
            .get_issue_types(&project)
            .await
            .expect("Should get issue types / labels");

        eprintln!(
            "Available GitHub issue types/labels: {:?}",
            types.iter().map(|t| &t.name).collect::<Vec<_>>()
        );
    }

    #[tokio::test]
    async fn test_list_issues() {
        skip_if_not_configured!(github_configured(), github_credentials_valid(), "GitHub");
        let provider = get_provider();
        let project = github_test_project();

        let users = provider
            .list_users(&project)
            .await
            .expect("Should list users");
        if users.is_empty() {
            eprintln!("No project assignees, skipping list_issues test");
            return;
        }

        let user_id = &users[0].id;
        let issues = provider
            .list_issues(&project, user_id, &[])
            .await
            .expect("Should list issues");

        eprintln!("Found {} issues for user {}", issues.len(), users[0].name);

        for issue in &issues {
            assert!(!issue.id.is_empty(), "Issue should have ID");
            assert!(!issue.key.is_empty(), "Issue should have key");
        }
    }

    #[tokio::test]
    async fn test_create_issue() {
        skip_if_not_configured!(github_configured(), github_credentials_valid(), "GitHub");
        let provider = get_provider();
        let project = github_test_project();

        let request = CreateIssueRequest {
            summary: test_issue_title("GitHub Create Test"),
            description: Some("Created by integration test - safe to archive".to_string()),
            assignee_id: None,
            status: None,
            priority: None,
        };

        let response = provider
            .create_issue(&project, request)
            .await
            .expect("Should create draft issue");

        eprintln!("Created GitHub issue: {}", response.issue.key);

        // v1 creates draft issues only (AddProjectV2DraftIssueInput).
        // The resulting key is formatted `draft:{project_item_id}`.
        assert!(
            response.issue.key.starts_with("draft:"),
            "v1 should create draft issues with 'draft:' key prefix, got: {}",
            response.issue.key
        );
        assert!(
            response.issue.summary.contains("[OPTEST]"),
            "Issue should have OPTEST prefix"
        );

        // ─── Cleanup: Move draft to terminal status ────────────────────────────────
        // The provider exposes no deletion API; we move to Done so the test
        // project remains visually sane. Drafts still accumulate — see file
        // doc comment.
        let statuses = provider
            .list_statuses(&project)
            .await
            .expect("Should get statuses for cleanup");

        if let Some(done_status) = find_terminal_status(&statuses) {
            eprintln!(
                "Cleanup: Moving draft {} to {}",
                response.issue.key, done_status
            );
            let _ = provider
                .update_issue_status(
                    &response.issue.key,
                    UpdateStatusRequest {
                        status: done_status,
                    },
                )
                .await;
        }
    }

    #[tokio::test]
    async fn test_update_issue_status() {
        skip_if_not_configured!(github_configured(), github_credentials_valid(), "GitHub");
        let provider = get_provider();
        let project = github_test_project();

        // First create a draft issue. create_issue populates the item_lookup
        // cache directly, so we don't need a list_issues call before the
        // update (github_projects.rs create_issue).
        let request = CreateIssueRequest {
            summary: test_issue_title("GitHub Status Test"),
            description: Some("Testing status transition".to_string()),
            assignee_id: None,
            status: None,
            priority: None,
        };

        let created = provider
            .create_issue(&project, request)
            .await
            .expect("Should create draft issue");

        // GitHub create_issue returns status="" — the draft does not yet
        // have a Status field value assigned. That's fine for this test.
        eprintln!(
            "Created draft {} (initial status: {:?})",
            created.issue.key, created.issue.status
        );

        // Get available statuses.
        let statuses = provider
            .list_statuses(&project)
            .await
            .expect("Should get statuses");

        // Find terminal status for later cleanup.
        let terminal_status = find_terminal_status(&statuses);

        // Find a non-terminal status to transition to first.
        let target_status = statuses
            .iter()
            .find(|s| {
                terminal_status
                    .as_ref()
                    .is_none_or(|t| !s.eq_ignore_ascii_case(t))
            })
            .cloned();

        if let Some(target) = target_status {
            eprintln!("Transitioning to: {target}");

            let update_request = UpdateStatusRequest {
                status: target.clone(),
            };

            // update_issue_status returns a minimal ExternalIssue — only
            // id/key/status are populated (github_projects.rs:1404-1415),
            // so we only assert on status here.
            let updated = provider
                .update_issue_status(&created.issue.key, update_request)
                .await
                .expect("Should update status");

            eprintln!("New status: {}", updated.status);
            assert_eq!(
                updated.status.to_lowercase(),
                target.to_lowercase(),
                "Status should be updated"
            );
        } else {
            eprintln!("No non-terminal status available for transition test");
        }

        // ─── Cleanup: Move draft to terminal status (Done) ─────────────────────────
        if let Some(done_status) = terminal_status {
            eprintln!("Cleanup: Transitioning draft to terminal status: {done_status}");

            let done_request = UpdateStatusRequest {
                status: done_status.clone(),
            };

            match provider
                .update_issue_status(&created.issue.key, done_request)
                .await
            {
                Ok(final_issue) => {
                    eprintln!(
                        "Draft {} moved to terminal status: {}",
                        final_issue.key, final_issue.status
                    );
                    assert_eq!(
                        final_issue.status.to_lowercase(),
                        done_status.to_lowercase(),
                        "Draft should be in terminal status"
                    );
                }
                Err(e) => {
                    eprintln!(
                        "Warning: Could not move draft to terminal status '{done_status}': {e}"
                    );
                    // Don't fail the test - cleanup is best-effort
                }
            }
        } else {
            eprintln!("Warning: No terminal status found in available statuses: {statuses:?}");
        }
    }
}

// ─── Cross-Provider Tests ────────────────────────────────────────────────────

#[tokio::test]
async fn test_provider_interface_consistency() {
    // This test verifies all three providers implement the same interface
    let jira_ok = jira_configured() && jira_credentials_valid().await;
    let linear_ok = linear_configured() && linear_credentials_valid().await;
    let github_ok = github_configured() && github_credentials_valid().await;

    if !jira_ok && !linear_ok && !github_ok {
        eprintln!("Skipping: No providers configured or credentials invalid");
        return;
    }

    if jira_ok {
        let provider = JiraProvider::from_env().unwrap();
        assert_eq!(provider.name(), "jira");
        assert!(provider.is_configured());
        eprintln!("Jira provider: configured and ready");
    }

    if linear_ok {
        let provider = LinearProvider::from_env().unwrap();
        assert_eq!(provider.name(), "linear");
        assert!(provider.is_configured());
        eprintln!("Linear provider: configured and ready");
    }

    if github_ok {
        let provider = GithubProjectsProvider::from_env().unwrap();
        assert_eq!(provider.name(), "github");
        assert!(provider.is_configured());
        eprintln!("GitHub provider: configured and ready");
    }
}

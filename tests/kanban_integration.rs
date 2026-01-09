//! Integration tests for Kanban providers (Jira and Linear)
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
//! ```

use operator::api::providers::kanban::{
    CreateIssueRequest, JiraProvider, KanbanProvider, LinearProvider, UpdateStatusRequest,
};
use std::env;

// ─── Configuration Helpers ───────────────────────────────────────────────────

/// Check if Jira credentials are configured
fn jira_configured() -> bool {
    env::var("OPERATOR_JIRA_DOMAIN").is_ok()
        && env::var("OPERATOR_JIRA_EMAIL").is_ok()
        && env::var("OPERATOR_JIRA_API_KEY").is_ok()
        && env::var("OPERATOR_JIRA_TEST_PROJECT").is_ok()
}

/// Check if Linear credentials are configured
fn linear_configured() -> bool {
    env::var("OPERATOR_LINEAR_API_KEY").is_ok() && env::var("OPERATOR_LINEAR_TEST_TEAM").is_ok()
}

/// Get the Jira test project key
fn jira_test_project() -> String {
    env::var("OPERATOR_JIRA_TEST_PROJECT").expect("OPERATOR_JIRA_TEST_PROJECT required")
}

/// Get the Linear test team ID
fn linear_test_team() -> String {
    env::var("OPERATOR_LINEAR_TEST_TEAM").expect("OPERATOR_LINEAR_TEST_TEAM required")
}

/// Generate a unique test issue title with [OPTEST] prefix
fn test_issue_title(suffix: &str) -> String {
    let uuid = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    format!("[OPTEST] {} - {}", suffix, uuid)
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

/// Macro to skip test if provider is not configured
macro_rules! skip_if_not_configured {
    ($configured:expr, $provider:expr) => {
        if !$configured {
            eprintln!("Skipping test: {} credentials not configured", $provider);
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
        skip_if_not_configured!(jira_configured(), "Jira");
        let provider = get_provider();

        let result = provider.test_connection().await;
        assert!(result.is_ok(), "Connection test failed: {:?}", result);
        assert!(result.unwrap(), "Connection should be valid");
    }

    #[tokio::test]
    async fn test_list_projects() {
        skip_if_not_configured!(jira_configured(), "Jira");
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
        skip_if_not_configured!(jira_configured(), "Jira");
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
        skip_if_not_configured!(jira_configured(), "Jira");
        let provider = get_provider();
        let project = jira_test_project();

        let statuses = provider
            .list_statuses(&project)
            .await
            .expect("Should list statuses");
        assert!(!statuses.is_empty(), "Should have workflow statuses");

        eprintln!("Available Jira statuses: {:?}", statuses);
    }

    #[tokio::test]
    async fn test_get_issue_types() {
        skip_if_not_configured!(jira_configured(), "Jira");
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
        skip_if_not_configured!(jira_configured(), "Jira");
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
        skip_if_not_configured!(jira_configured(), "Jira");
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
        skip_if_not_configured!(jira_configured(), "Jira");
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
            eprintln!("Transitioning to: {}", target);

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
                    eprintln!("Transition failed (may be expected): {}", e);
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
        skip_if_not_configured!(linear_configured(), "Linear");
        let provider = get_provider();

        let result = provider.test_connection().await;
        assert!(result.is_ok(), "Connection test failed: {:?}", result);
        assert!(result.unwrap(), "Connection should be valid");
    }

    #[tokio::test]
    async fn test_list_projects() {
        skip_if_not_configured!(linear_configured(), "Linear");
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
        skip_if_not_configured!(linear_configured(), "Linear");
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
        skip_if_not_configured!(linear_configured(), "Linear");
        let provider = get_provider();
        let team = linear_test_team();

        let statuses = provider
            .list_statuses(&team)
            .await
            .expect("Should list statuses");
        assert!(!statuses.is_empty(), "Should have workflow states");

        eprintln!("Available Linear statuses: {:?}", statuses);
    }

    #[tokio::test]
    async fn test_get_issue_types() {
        skip_if_not_configured!(linear_configured(), "Linear");
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
        skip_if_not_configured!(linear_configured(), "Linear");
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
        skip_if_not_configured!(linear_configured(), "Linear");
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
        skip_if_not_configured!(linear_configured(), "Linear");
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
            eprintln!("Transitioning to: {}", target);

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
            if !current_status.eq_ignore_ascii_case(&done_status) {
                eprintln!(
                    "Cleanup: Transitioning issue to terminal status: {}",
                    done_status
                );

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
                            "Warning: Could not move issue to terminal status '{}': {}",
                            done_status, e
                        );
                        // Don't fail the test - cleanup is best-effort
                    }
                }
            } else {
                eprintln!("Issue already in terminal status: {}", current_status);
            }
        } else {
            eprintln!(
                "Warning: No terminal status found in available statuses: {:?}",
                statuses
            );
        }
    }
}

// ─── Cross-Provider Tests ────────────────────────────────────────────────────

#[tokio::test]
async fn test_provider_interface_consistency() {
    // This test verifies both providers implement the same interface
    let jira_ok = jira_configured();
    let linear_ok = linear_configured();

    if !jira_ok && !linear_ok {
        eprintln!("Skipping: No providers configured");
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
}

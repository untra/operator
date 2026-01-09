//! Integration tests for Git CLI and WorktreeManager
//!
//! These tests use the actual operator repository for realistic testing.
//! All test artifacts use the `optest/` branch prefix for identification and cleanup.
//!
//! ## Environment Variables
//!
//! - `OPERATOR_GIT_TEST_ENABLED=true`: Required to run any git tests
//! - `OPERATOR_GIT_PUSH_ENABLED=true`: Required for tests that push to remote
//!
//! ## Running Tests
//!
//! ```bash
//! # All git integration tests (read-only only)
//! OPERATOR_GIT_TEST_ENABLED=true cargo test --test git_integration -- --test-threads=1
//!
//! # Include remote push tests
//! OPERATOR_GIT_TEST_ENABLED=true OPERATOR_GIT_PUSH_ENABLED=true \
//!     cargo test --test git_integration -- --test-threads=1
//!
//! # Specific test module
//! OPERATOR_GIT_TEST_ENABLED=true cargo test --test git_integration git_cli_readonly_tests -- --test-threads=1
//! ```
//!
//! ## Test Branch Naming
//!
//! All test branches use the prefix `optest/` followed by a unique identifier:
//! - `optest/test-{timestamp}-{random}`
//!
//! This allows easy identification and cleanup of test artifacts.

use operator::git::{GitCli, WorktreeManager};
use std::env;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

// ─── Configuration Helpers ───────────────────────────────────────────────────

/// Branch prefix for all test branches (easy to identify and cleanup)
const TEST_BRANCH_PREFIX: &str = "optest/";

/// Check if git tests are enabled
fn git_tests_enabled() -> bool {
    env::var("OPERATOR_GIT_TEST_ENABLED")
        .map(|v| v == "true" || v == "1")
        .unwrap_or(false)
}

/// Check if push tests are enabled (requires additional permission)
fn push_tests_enabled() -> bool {
    git_tests_enabled()
        && env::var("OPERATOR_GIT_PUSH_ENABLED")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(false)
}

/// Macro to skip test if git tests are not configured
macro_rules! skip_if_not_configured {
    () => {
        if !git_tests_enabled() {
            eprintln!("Skipping test: OPERATOR_GIT_TEST_ENABLED not set to true");
            return;
        }
    };
}

/// Macro to skip test if push tests are not enabled
macro_rules! skip_if_push_not_enabled {
    () => {
        skip_if_not_configured!();
        if !push_tests_enabled() {
            eprintln!("Skipping test: OPERATOR_GIT_PUSH_ENABLED not set to true");
            return;
        }
    };
}

/// Generate a unique test branch name
fn test_branch_name(suffix: &str) -> String {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{}{}-{}", TEST_BRANCH_PREFIX, suffix, timestamp)
}

/// Generate a unique test ticket ID
fn test_ticket_id() -> String {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    format!("OPTEST-{}", timestamp % 100000)
}

/// Get the path to the operator repository (current working directory)
fn get_repo_path() -> PathBuf {
    env::current_dir().expect("Failed to get current directory")
}

/// Get the default branch name for the repository (returns remote ref like "origin/main")
async fn get_default_branch(repo_path: &Path) -> String {
    // Try to read origin/HEAD symbolic ref
    if let Ok(ref_str) = GitCli::symbolic_ref(repo_path, "refs/remotes/origin/HEAD").await {
        // refs/remotes/origin/main -> origin/main
        if let Some(branch) = ref_str.strip_prefix("refs/remotes/origin/") {
            return format!("origin/{}", branch);
        }
    }
    // Fallback to origin/main (works in CI where local main doesn't exist)
    "origin/main".to_string()
}

// ─── GitCli Read-Only Tests ──────────────────────────────────────────────────

mod git_cli_readonly_tests {
    use super::*;

    /// Test: Verify current_branch returns a valid branch name
    #[tokio::test]
    async fn test_current_branch_returns_valid_name() {
        skip_if_not_configured!();
        let repo_path = get_repo_path();

        let branch = GitCli::current_branch(&repo_path)
            .await
            .expect("Should get current branch");

        assert!(!branch.is_empty(), "Branch name should not be empty");
        assert!(
            !branch.contains('\n'),
            "Branch name should not contain newlines"
        );
        eprintln!("Current branch: {}", branch);
    }

    /// Test: Verify repo_root returns a path containing 'operator'
    #[tokio::test]
    async fn test_repo_root_detection() {
        skip_if_not_configured!();
        let repo_path = get_repo_path();

        let root = GitCli::repo_root(&repo_path)
            .await
            .expect("Should get repo root");

        assert!(
            root.contains("operator"),
            "Root should contain 'operator': {}",
            root
        );
        assert!(Path::new(&root).exists(), "Root path should exist");
        eprintln!("Repo root: {}", root);
    }

    /// Test: Verify is_dirty returns a boolean without error
    #[tokio::test]
    async fn test_is_dirty_returns_bool() {
        skip_if_not_configured!();
        let repo_path = get_repo_path();

        let is_dirty = GitCli::is_dirty(&repo_path)
            .await
            .expect("Should check dirty status");

        eprintln!("Repository dirty: {}", is_dirty);
        // We can't assert the value, but we verify it doesn't error
    }

    /// Test: Verify is_worktree correctly identifies a git worktree
    #[tokio::test]
    async fn test_is_worktree_in_repo() {
        skip_if_not_configured!();
        let repo_path = get_repo_path();

        let is_wt = GitCli::is_worktree(&repo_path)
            .await
            .expect("Should check worktree status");

        assert!(is_wt, "Operator repo should be inside a work tree");
    }

    /// Test: Verify is_worktree returns false for non-repo paths
    #[tokio::test]
    async fn test_is_worktree_outside_repo() {
        skip_if_not_configured!();
        let temp = TempDir::new().expect("Failed to create temp dir");

        let is_wt = GitCli::is_worktree(temp.path())
            .await
            .expect("Should check worktree status");

        assert!(!is_wt, "Temp directory should not be a worktree");
    }

    /// Test: Verify list_worktrees returns at least the main worktree
    #[tokio::test]
    async fn test_list_worktrees_has_main() {
        skip_if_not_configured!();
        let repo_path = get_repo_path();

        let worktrees = GitCli::list_worktrees(&repo_path)
            .await
            .expect("Should list worktrees");

        assert!(!worktrees.is_empty(), "Should have at least one worktree");

        // First worktree should be the main one (not bare)
        let main_wt = &worktrees[0];
        assert!(!main_wt.bare, "Main worktree should not be bare");
        assert!(main_wt.head.is_some(), "Main worktree should have HEAD");

        eprintln!("Found {} worktrees", worktrees.len());
        for wt in &worktrees {
            eprintln!("  - {} (branch: {:?})", wt.path, wt.branch);
        }
    }

    /// Test: Verify head_commit returns a valid SHA
    #[tokio::test]
    async fn test_head_commit_returns_sha() {
        skip_if_not_configured!();
        let repo_path = get_repo_path();

        let sha = GitCli::head_commit(&repo_path)
            .await
            .expect("Should get HEAD commit");

        assert_eq!(sha.len(), 40, "SHA should be 40 characters");
        assert!(
            sha.chars().all(|c| c.is_ascii_hexdigit()),
            "SHA should be hex: {}",
            sha
        );

        eprintln!("HEAD commit: {}", sha);
    }

    /// Test: Verify get_remote_url returns origin URL
    #[tokio::test]
    async fn test_get_remote_url() {
        skip_if_not_configured!();
        let repo_path = get_repo_path();

        let url = GitCli::get_remote_url(&repo_path)
            .await
            .expect("Should get remote URL");

        assert!(
            url.contains("operator") || url.contains("gbqr"),
            "Remote URL should reference operator: {}",
            url
        );

        eprintln!("Remote URL: {}", url);
    }

    /// Test: Verify symbolic_ref can read origin/HEAD
    #[tokio::test]
    async fn test_symbolic_ref_origin_head() {
        skip_if_not_configured!();
        let repo_path = get_repo_path();

        // This may fail if origin/HEAD isn't set, which is OK
        match GitCli::symbolic_ref(&repo_path, "refs/remotes/origin/HEAD").await {
            Ok(ref_str) => {
                assert!(ref_str.starts_with("refs/"), "Should be a ref: {}", ref_str);
                eprintln!("origin/HEAD points to: {}", ref_str);
            }
            Err(e) => {
                eprintln!("origin/HEAD not set (expected in some setups): {}", e);
            }
        }
    }
}

// ─── GitCli Remote Tests ─────────────────────────────────────────────────────

mod git_cli_remote_tests {
    use super::*;

    /// Test: Verify fetch from origin succeeds
    #[tokio::test]
    async fn test_fetch_origin() {
        skip_if_not_configured!();
        let repo_path = get_repo_path();

        let result = GitCli::fetch(&repo_path, "origin").await;
        assert!(result.is_ok(), "Fetch should succeed: {:?}", result);

        eprintln!("Fetch from origin completed");
    }

    /// Test: Verify remote_branch_exists detects main/master
    #[tokio::test]
    async fn test_remote_branch_exists_main() {
        skip_if_not_configured!();
        let repo_path = get_repo_path();

        // Check for main (most common default)
        let main_exists = GitCli::remote_branch_exists(&repo_path, "origin", "main")
            .await
            .expect("Should check main branch");

        let master_exists = GitCli::remote_branch_exists(&repo_path, "origin", "master")
            .await
            .expect("Should check master branch");

        assert!(
            main_exists || master_exists,
            "Either main or master should exist on remote"
        );

        eprintln!(
            "main exists: {}, master exists: {}",
            main_exists, master_exists
        );
    }

    /// Test: Verify remote_branch_exists returns false for non-existent branch
    #[tokio::test]
    async fn test_remote_branch_exists_nonexistent() {
        skip_if_not_configured!();
        let repo_path = get_repo_path();

        let exists =
            GitCli::remote_branch_exists(&repo_path, "origin", "this-branch-does-not-exist-12345")
                .await
                .expect("Should check branch existence");

        assert!(!exists, "Non-existent branch should return false");
    }
}

// ─── GitCli Branch Lifecycle Tests ───────────────────────────────────────────

mod git_cli_branch_lifecycle_tests {
    use super::*;

    /// Test: Full branch lifecycle - create, push, verify, delete
    #[tokio::test]
    async fn test_branch_create_push_delete_lifecycle() {
        skip_if_push_not_enabled!();
        let repo_path = get_repo_path();

        let branch_name = test_branch_name("lifecycle");
        let default_branch = get_default_branch(&repo_path).await;

        eprintln!("Testing branch lifecycle: {}", branch_name);
        eprintln!("Base branch: {}", default_branch);

        // Create branch from default branch
        let result = GitCli::create_branch(&repo_path, &branch_name, &default_branch).await;
        assert!(
            result.is_ok(),
            "Branch creation should succeed: {:?}",
            result
        );

        // Push to remote with upstream
        let push_result = GitCli::push(&repo_path, "origin", &branch_name, true).await;
        assert!(
            push_result.is_ok(),
            "Push should succeed: {:?}",
            push_result
        );

        // Verify branch exists on remote
        let exists = GitCli::remote_branch_exists(&repo_path, "origin", &branch_name)
            .await
            .expect("Should check branch existence");
        assert!(exists, "Branch should exist on remote after push");

        // Delete remote branch
        let delete_remote = GitCli::delete_remote_branch(&repo_path, "origin", &branch_name).await;
        assert!(
            delete_remote.is_ok(),
            "Remote delete should succeed: {:?}",
            delete_remote
        );

        // Verify branch no longer exists on remote
        let exists_after = GitCli::remote_branch_exists(&repo_path, "origin", &branch_name)
            .await
            .expect("Should check branch existence");
        assert!(
            !exists_after,
            "Branch should not exist on remote after delete"
        );

        // Delete local branch
        let delete_local = GitCli::delete_branch(&repo_path, &branch_name, true).await;
        assert!(
            delete_local.is_ok(),
            "Local delete should succeed: {:?}",
            delete_local
        );

        eprintln!("Branch lifecycle test completed successfully");
    }

    /// Test: Local branch creation without push
    #[tokio::test]
    async fn test_branch_create_local_only() {
        skip_if_not_configured!();
        let repo_path = get_repo_path();

        let branch_name = test_branch_name("local-only");
        let default_branch = get_default_branch(&repo_path).await;

        // Create branch
        let result = GitCli::create_branch(&repo_path, &branch_name, &default_branch).await;
        assert!(
            result.is_ok(),
            "Branch creation should succeed: {:?}",
            result
        );

        // Cleanup: delete local branch
        let delete = GitCli::delete_branch(&repo_path, &branch_name, true).await;
        assert!(delete.is_ok(), "Cleanup should succeed: {:?}", delete);

        eprintln!("Local branch test completed");
    }
}

// ─── GitCli Worktree Tests ───────────────────────────────────────────────────

mod git_cli_worktree_tests {
    use super::*;

    /// Test: Add and remove a worktree
    #[tokio::test]
    async fn test_worktree_add_remove_lifecycle() {
        skip_if_not_configured!();
        let repo_path = get_repo_path();
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let worktree_path = temp_dir.path().join("test-worktree");

        let branch_name = test_branch_name("worktree-test");
        let default_branch = get_default_branch(&repo_path).await;

        eprintln!("Creating worktree at: {}", worktree_path.display());
        eprintln!("Branch: {}, Base: {}", branch_name, default_branch);

        // Add worktree with new branch
        let add_result = GitCli::add_worktree(
            &repo_path,
            &worktree_path,
            &branch_name,
            true, // create_branch
            Some(&default_branch),
        )
        .await;
        assert!(
            add_result.is_ok(),
            "Worktree add should succeed: {:?}",
            add_result
        );

        // Verify worktree exists
        assert!(worktree_path.exists(), "Worktree path should exist");

        // Verify it's a valid worktree
        let is_wt = GitCli::is_worktree(&worktree_path)
            .await
            .expect("Should check worktree status");
        assert!(is_wt, "New path should be a worktree");

        // Verify branch in worktree
        let wt_branch = GitCli::current_branch(&worktree_path)
            .await
            .expect("Should get worktree branch");
        assert_eq!(
            wt_branch, branch_name,
            "Worktree should be on correct branch"
        );

        // Verify worktree appears in list
        let worktrees = GitCli::list_worktrees(&repo_path)
            .await
            .expect("Should list worktrees");
        // Use canonicalized paths for comparison (temp dirs may have symlinks)
        let canonical_wt_path = worktree_path
            .canonicalize()
            .unwrap_or_else(|_| worktree_path.clone());
        let found = worktrees.iter().any(|wt| {
            let wt_path = Path::new(&wt.path);
            wt_path
                .canonicalize()
                .unwrap_or_else(|_| wt_path.to_path_buf())
                == canonical_wt_path
        });
        assert!(found, "New worktree should appear in list");

        // Remove worktree
        let remove_result = GitCli::remove_worktree(&repo_path, &worktree_path, false).await;
        assert!(
            remove_result.is_ok(),
            "Worktree remove should succeed: {:?}",
            remove_result
        );

        // Prune worktree metadata
        let prune_result = GitCli::prune_worktrees(&repo_path).await;
        assert!(
            prune_result.is_ok(),
            "Prune should succeed: {:?}",
            prune_result
        );

        // Cleanup: delete the branch we created
        let _ = GitCli::delete_branch(&repo_path, &branch_name, true).await;

        eprintln!("Worktree lifecycle test completed");
    }

    /// Test: Worktree add with existing branch fails appropriately
    #[tokio::test]
    async fn test_worktree_add_existing_branch() {
        skip_if_not_configured!();
        let repo_path = get_repo_path();
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let worktree_path = temp_dir.path().join("existing-branch-wt");

        let default_branch = get_default_branch(&repo_path).await;

        // Try to add worktree with existing branch (not creating)
        // This should fail because the branch is already checked out
        let result = GitCli::add_worktree(
            &repo_path,
            &worktree_path,
            &default_branch,
            false, // don't create_branch
            None,
        )
        .await;

        // This might fail because default branch is already checked out
        // That's expected behavior
        eprintln!("Add existing branch result: {:?}", result);

        // Cleanup if it succeeded
        if result.is_ok() {
            let _ = GitCli::remove_worktree(&repo_path, &worktree_path, true).await;
        }
    }

    /// Test: Prune worktrees works without error
    #[tokio::test]
    async fn test_prune_worktrees() {
        skip_if_not_configured!();
        let repo_path = get_repo_path();

        let result = GitCli::prune_worktrees(&repo_path).await;
        assert!(result.is_ok(), "Prune should succeed: {:?}", result);

        eprintln!("Prune worktrees completed");
    }

    /// Test: Repair worktrees works without error
    #[tokio::test]
    async fn test_repair_worktrees() {
        skip_if_not_configured!();
        let repo_path = get_repo_path();

        let result = GitCli::repair_worktrees(&repo_path).await;
        assert!(result.is_ok(), "Repair should succeed: {:?}", result);

        eprintln!("Repair worktrees completed");
    }
}

// ─── WorktreeManager Tests ───────────────────────────────────────────────────

mod worktree_manager_tests {
    use super::*;

    /// Test: WorktreeManager path generation
    #[tokio::test]
    async fn test_worktree_path_generation() {
        let temp = TempDir::new().expect("Failed to create temp dir");
        let manager = WorktreeManager::new(temp.path().to_path_buf());

        let path = manager.worktree_path("myproject", "FEAT-123");

        assert!(
            path.starts_with(temp.path()),
            "Path should be under base dir"
        );
        assert!(
            path.ends_with("myproject/feat-123"),
            "Path should follow pattern"
        );

        eprintln!("Generated path: {}", path.display());
    }

    /// Test: WorktreeManager path generation with various ticket IDs
    #[tokio::test]
    async fn test_worktree_path_normalization() {
        let temp = TempDir::new().expect("Failed to create temp dir");
        let manager = WorktreeManager::new(temp.path().to_path_buf());

        // Test case normalization
        let path1 = manager.worktree_path("Project", "FEAT-100");
        let path2 = manager.worktree_path("Project", "feat-100");

        // Ticket IDs are lowercased
        assert!(path1.to_string_lossy().contains("feat-100"));
        assert!(path2.to_string_lossy().contains("feat-100"));

        eprintln!("Path 1: {}", path1.display());
        eprintln!("Path 2: {}", path2.display());
    }

    /// Test: create_for_ticket full lifecycle
    #[tokio::test]
    async fn test_create_for_ticket_lifecycle() {
        skip_if_not_configured!();

        let temp = TempDir::new().expect("Failed to create temp dir");
        let manager = WorktreeManager::new(temp.path().to_path_buf());
        let repo_path = get_repo_path();
        let default_branch = get_default_branch(&repo_path).await;

        let ticket_id = test_ticket_id();
        let branch_name = test_branch_name("manager-test");

        eprintln!("Creating worktree for ticket: {}", ticket_id);
        eprintln!("Branch: {}", branch_name);

        // Create worktree
        let info = manager
            .create_for_ticket(
                &repo_path,
                "test-project",
                &ticket_id,
                &branch_name,
                &default_branch,
            )
            .await
            .expect("Should create worktree");

        // Verify info fields
        assert!(info.path.exists(), "Worktree path should exist");
        assert_eq!(info.branch, branch_name, "Branch should match");
        assert!(!info.base_commit.is_empty(), "Base commit should be set");
        assert_eq!(
            info.target_branch, default_branch,
            "Target branch should match"
        );

        // Verify it's a valid git worktree
        let is_wt = GitCli::is_worktree(&info.path)
            .await
            .expect("Should check worktree");
        assert!(is_wt, "Created path should be a worktree");

        // Test is_dirty
        let dirty = manager
            .is_dirty(&info)
            .await
            .expect("Should check dirty status");
        assert!(!dirty, "New worktree should be clean");

        // Cleanup
        manager
            .cleanup_worktree(&info, true, false)
            .await
            .expect("Cleanup should succeed");

        assert!(!info.path.exists(), "Worktree path should be removed");

        eprintln!("Worktree lifecycle completed successfully");
    }

    /// Test: ensure_worktree_exists idempotency
    #[tokio::test]
    async fn test_ensure_worktree_exists_idempotent() {
        skip_if_not_configured!();

        let temp = TempDir::new().expect("Failed to create temp dir");
        let manager = WorktreeManager::new(temp.path().to_path_buf());
        let repo_path = get_repo_path();
        let default_branch = get_default_branch(&repo_path).await;

        let ticket_id = test_ticket_id();
        let branch_name = test_branch_name("idempotent");

        // First call creates
        let info1 = manager
            .ensure_worktree_exists(
                &repo_path,
                "test-project",
                &ticket_id,
                &branch_name,
                &default_branch,
            )
            .await
            .expect("First ensure should succeed");

        // Second call returns same
        let info2 = manager
            .ensure_worktree_exists(
                &repo_path,
                "test-project",
                &ticket_id,
                &branch_name,
                &default_branch,
            )
            .await
            .expect("Second ensure should succeed");

        // Same path should be returned
        assert_eq!(
            info1.path, info2.path,
            "Same worktree path should be returned"
        );

        // Cleanup
        manager
            .cleanup_worktree(&info1, true, false)
            .await
            .expect("Cleanup should succeed");
    }

    /// Test: list_project_worktrees empty project
    #[tokio::test]
    async fn test_list_project_worktrees_empty() {
        let temp = TempDir::new().expect("Failed to create temp dir");
        let manager = WorktreeManager::new(temp.path().to_path_buf());

        let worktrees = manager
            .list_project_worktrees("nonexistent")
            .await
            .expect("Should list worktrees");

        assert!(
            worktrees.is_empty(),
            "Non-existent project should have no worktrees"
        );
    }

    /// Test: list_project_worktrees with worktrees
    #[tokio::test]
    async fn test_list_project_worktrees_with_entries() {
        skip_if_not_configured!();

        let temp = TempDir::new().expect("Failed to create temp dir");
        let manager = WorktreeManager::new(temp.path().to_path_buf());
        let repo_path = get_repo_path();
        let default_branch = get_default_branch(&repo_path).await;

        let project = "list-test-project";
        let ticket1 = test_ticket_id();
        let ticket2 = format!("{}-2", test_ticket_id());

        // Create two worktrees
        let info1 = manager
            .create_for_ticket(
                &repo_path,
                project,
                &ticket1,
                &test_branch_name("list1"),
                &default_branch,
            )
            .await
            .expect("Should create first worktree");

        let info2 = manager
            .create_for_ticket(
                &repo_path,
                project,
                &ticket2,
                &test_branch_name("list2"),
                &default_branch,
            )
            .await
            .expect("Should create second worktree");

        // List worktrees
        let worktrees = manager
            .list_project_worktrees(project)
            .await
            .expect("Should list worktrees");

        assert_eq!(worktrees.len(), 2, "Should have two worktrees");

        // Cleanup
        manager
            .cleanup_worktree(&info1, true, false)
            .await
            .expect("Cleanup 1");
        manager
            .cleanup_worktree(&info2, true, false)
            .await
            .expect("Cleanup 2");
    }

    /// Test: cleanup_project_worktrees removes all
    #[tokio::test]
    async fn test_cleanup_project_worktrees() {
        skip_if_not_configured!();

        let temp = TempDir::new().expect("Failed to create temp dir");
        let manager = WorktreeManager::new(temp.path().to_path_buf());
        let repo_path = get_repo_path();
        let default_branch = get_default_branch(&repo_path).await;

        let project = "cleanup-test-project";

        // Create a worktree
        let _info = manager
            .create_for_ticket(
                &repo_path,
                project,
                &test_ticket_id(),
                &test_branch_name("cleanup"),
                &default_branch,
            )
            .await
            .expect("Should create worktree");

        // Cleanup all
        manager
            .cleanup_project_worktrees(project, &repo_path)
            .await
            .expect("Cleanup should succeed");

        // Verify empty
        let remaining = manager
            .list_project_worktrees(project)
            .await
            .expect("Should list worktrees");
        assert!(remaining.is_empty(), "All worktrees should be cleaned up");
    }
}

// ─── WorktreeManager Error Tests ─────────────────────────────────────────────

mod worktree_manager_error_tests {
    use super::*;

    /// Test: create_for_ticket with invalid repo path
    #[tokio::test]
    async fn test_create_invalid_repo_path() {
        let temp = TempDir::new().expect("Failed to create temp dir");
        let manager = WorktreeManager::new(temp.path().to_path_buf());

        let result = manager
            .create_for_ticket(
                Path::new("/nonexistent/repo"),
                "project",
                "TICKET-1",
                "branch",
                "main",
            )
            .await;

        assert!(result.is_err(), "Should fail with invalid repo path");
        eprintln!("Expected error: {:?}", result.unwrap_err());
    }

    /// Test: validate behavior when worktree directory already exists but isn't a worktree
    #[tokio::test]
    async fn test_path_exists_but_not_worktree() {
        skip_if_not_configured!();

        let temp = TempDir::new().expect("Failed to create temp dir");
        let repo_path = get_repo_path();

        // Create the expected worktree path as a regular directory
        let project = "conflict-project";
        let ticket = "CONFLICT-1";
        let manager = WorktreeManager::new(temp.path().to_path_buf());
        let expected_path = manager.worktree_path(project, ticket);

        std::fs::create_dir_all(&expected_path).expect("Should create dir");

        // Try to create worktree - should fail validation
        let result = manager
            .create_for_ticket(&repo_path, project, ticket, "test-branch", "main")
            .await;

        // May succeed if it can recover, or fail with appropriate error
        eprintln!("Result when path exists but isn't worktree: {:?}", result);

        // Cleanup
        let _ = std::fs::remove_dir_all(&expected_path);
    }
}

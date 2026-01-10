//! Worktree setup for per-ticket isolation
//!
//! Creates git worktrees for tickets before launching agents,
//! enabling parallel development without branch conflicts.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use tracing::{debug, info, warn};

use crate::config::Config;
use crate::git::{GitCli, WorktreeInfo, WorktreeManager};
use crate::queue::Ticket;

/// Sanitize a string for use in branch names
fn sanitize_branch_name(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}

/// Generate a branch name for a ticket
///
/// Pattern: {ticket_type}/{ticket_id}
/// Example: feat/feat-1234
pub fn branch_name_for_ticket(ticket: &Ticket) -> String {
    format!(
        "{}/{}",
        ticket.ticket_type.to_lowercase(),
        sanitize_branch_name(&ticket.id)
    )
}

/// Setup a worktree for a ticket if the project is a git repository
///
/// Returns the working directory path to use (worktree if created, or project path otherwise).
///
/// # Arguments
/// * `config` - Operator configuration
/// * `ticket` - The ticket to create a worktree for (will be mutated to set worktree_path and branch)
/// * `project_path` - Path to the project directory
///
/// # Returns
/// * `Ok(PathBuf)` - The path to use as working directory (worktree or project)
/// * `Err` - If worktree creation fails
pub async fn setup_worktree_for_ticket(
    config: &Config,
    ticket: &mut Ticket,
    project_path: &Path,
) -> Result<PathBuf> {
    // Check if already has a worktree (e.g., on relaunch)
    if let Some(ref existing) = ticket.worktree_path {
        let existing_path = PathBuf::from(existing);
        if existing_path.exists() {
            debug!(
                worktree = %existing,
                "Ticket already has a worktree, reusing"
            );
            return Ok(existing_path);
        }
    }

    // Check if project is a git repository
    let git_dir = project_path.join(".git");
    if !git_dir.exists() {
        debug!(
            project = %project_path.display(),
            "Project is not a git repository, skipping worktree setup"
        );
        return Ok(project_path.to_path_buf());
    }

    // Generate branch name
    let branch_name = branch_name_for_ticket(ticket);

    // Determine target branch (use "main" or "master" by default)
    let target_branch = detect_default_branch(project_path)
        .await
        .unwrap_or_else(|| "main".to_string());

    info!(
        project = %ticket.project,
        ticket_id = %ticket.id,
        branch = %branch_name,
        target = %target_branch,
        "Setting up worktree for ticket"
    );

    // Create worktree manager
    let worktree_manager = WorktreeManager::new(config.worktrees_path());

    // Create or get existing worktree
    let worktree_info = worktree_manager
        .ensure_worktree_exists(
            project_path,
            &ticket.project,
            &ticket.id,
            &branch_name,
            &target_branch,
        )
        .await
        .context("Failed to create worktree for ticket")?;

    // Update ticket with worktree info
    ticket
        .set_worktree_path(&worktree_info.path.to_string_lossy())
        .context("Failed to update ticket worktree_path")?;
    ticket
        .set_branch(&worktree_info.branch)
        .context("Failed to update ticket branch")?;

    info!(
        worktree = %worktree_info.path.display(),
        branch = %worktree_info.branch,
        "Worktree ready for ticket"
    );

    Ok(worktree_info.path)
}

/// Detect the default branch for a repository
async fn detect_default_branch(repo_path: &Path) -> Option<String> {
    // Try to get the HEAD reference
    match GitCli::symbolic_ref(repo_path, "refs/remotes/origin/HEAD").await {
        Ok(head_ref) => {
            // Extract branch name from refs/remotes/origin/main
            let branch = head_ref
                .trim()
                .strip_prefix("refs/remotes/origin/")
                .map(|s| s.to_string());
            if branch.is_some() {
                return branch;
            }
        }
        Err(e) => {
            debug!(
                "Could not determine default branch from symbolic ref: {}",
                e
            );
        }
    }

    // Fall back to checking if main or master exists
    for branch in &["main", "master"] {
        let ref_path = repo_path.join(".git/refs/heads").join(branch);
        if ref_path.exists() {
            return Some(branch.to_string());
        }
    }

    // Check packed-refs for main/master
    let packed_refs = repo_path.join(".git/packed-refs");
    if packed_refs.exists() {
        if let Ok(contents) = std::fs::read_to_string(&packed_refs) {
            for branch in &["main", "master"] {
                if contents.contains(&format!("refs/heads/{}", branch)) {
                    return Some(branch.to_string());
                }
            }
        }
    }

    None
}

/// Cleanup worktree for a completed ticket
///
/// # Arguments
/// * `config` - Operator configuration
/// * `worktree_path` - Path to the worktree to clean up
/// * `repo_path` - Path to the main repository
/// * `cleanup_script` - Optional cleanup script to run before removal
/// * `prune_branch` - Whether to delete the branch
/// * `delete_remote_branch` - Whether to delete the remote branch too
#[allow(dead_code)] // Will be used in sync.rs for PR merge cleanup
pub async fn cleanup_ticket_worktree(
    config: &Config,
    worktree_path: &Path,
    repo_path: &Path,
    cleanup_script: Option<&str>,
    prune_branch: bool,
    delete_remote_branch: bool,
) -> Result<()> {
    use tokio::process::Command;

    // Run cleanup script if provided
    if let Some(script) = cleanup_script {
        info!(script = %script, "Running cleanup script before worktree removal");
        let output = Command::new("sh")
            .arg("-c")
            .arg(script)
            .current_dir(worktree_path)
            .output()
            .await
            .context("Failed to run cleanup script")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!(stderr = %stderr, "Cleanup script failed, continuing with worktree removal");
        }
    }

    // Get current branch in worktree
    let branch = GitCli::current_branch(worktree_path)
        .await
        .unwrap_or_else(|_| "unknown".to_string());

    let worktree_info = WorktreeInfo {
        path: worktree_path.to_path_buf(),
        branch,
        base_commit: String::new(),
        repo_path: repo_path.to_path_buf(),
        target_branch: String::new(),
    };

    // Create worktree manager and cleanup
    let worktree_manager = WorktreeManager::new(config.worktrees_path());
    worktree_manager
        .cleanup_worktree(&worktree_info, prune_branch, delete_remote_branch)
        .await
        .context("Failed to cleanup worktree")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_ticket(id: &str, ticket_type: &str) -> crate::queue::Ticket {
        crate::queue::Ticket {
            id: id.to_string(),
            ticket_type: ticket_type.to_string(),
            filename: "test.md".to_string(),
            filepath: "/tmp/test.md".to_string(),
            timestamp: "20241225-1200".to_string(),
            project: "test-project".to_string(),
            summary: "Test".to_string(),
            priority: "P2-medium".to_string(),
            status: "queued".to_string(),
            step: String::new(),
            content: String::new(),
            sessions: std::collections::HashMap::new(),
            llm_task: crate::queue::LlmTask::default(),
            worktree_path: None,
            branch: None,
            external_id: None,
            external_url: None,
            external_provider: None,
        }
    }

    // ========================================
    // sanitize_branch_name() tests
    // ========================================

    #[test]
    fn test_sanitize_branch_name() {
        assert_eq!(sanitize_branch_name("FEAT-1234"), "feat-1234");
        assert_eq!(sanitize_branch_name("Fix Bug #42"), "fix-bug--42");
        assert_eq!(sanitize_branch_name("hello_world"), "hello_world");
    }

    #[test]
    fn test_sanitize_branch_name_spaces_become_hyphens() {
        assert_eq!(sanitize_branch_name("add user auth"), "add-user-auth");
        assert_eq!(sanitize_branch_name("  leading spaces"), "leading-spaces");
        assert_eq!(sanitize_branch_name("trailing spaces  "), "trailing-spaces");
    }

    #[test]
    fn test_sanitize_branch_name_special_chars_become_hyphens() {
        // Special chars become hyphens, trailing hyphens are trimmed
        assert_eq!(sanitize_branch_name("feat@123!"), "feat-123");
        assert_eq!(sanitize_branch_name("test$%^&*()"), "test");
        assert_eq!(sanitize_branch_name("fix/slash/test"), "fix-slash-test");
    }

    #[test]
    fn test_sanitize_branch_name_unicode_alphanumeric_preserved() {
        // is_alphanumeric() includes Unicode letters, so they're preserved
        // Japanese characters are alphanumeric in Unicode
        assert_eq!(sanitize_branch_name("feat-日本語"), "feat-日本語");
        // Emoji is not alphanumeric, so it becomes hyphen (and gets trimmed)
        assert_eq!(sanitize_branch_name("test-🎉"), "test");
        // Accented letters are alphanumeric
        assert_eq!(sanitize_branch_name("café-feature"), "café-feature");
    }

    #[test]
    fn test_sanitize_branch_name_preserves_underscores() {
        assert_eq!(sanitize_branch_name("feature_flag"), "feature_flag");
        assert_eq!(sanitize_branch_name("test_case_123"), "test_case_123");
    }

    #[test]
    fn test_sanitize_branch_name_trims_leading_trailing_hyphens() {
        assert_eq!(sanitize_branch_name("-leading"), "leading");
        assert_eq!(sanitize_branch_name("trailing-"), "trailing");
        assert_eq!(sanitize_branch_name("--both--"), "both");
    }

    // ========================================
    // branch_name_for_ticket() tests
    // ========================================

    #[test]
    fn test_branch_name_for_ticket() {
        let ticket = make_ticket("FEAT-1234", "FEAT");
        assert_eq!(branch_name_for_ticket(&ticket), "feat/feat-1234");
    }

    #[test]
    fn test_branch_name_for_fix_ticket() {
        let ticket = make_ticket("FIX-5678", "FIX");
        assert_eq!(branch_name_for_ticket(&ticket), "fix/fix-5678");
    }

    #[test]
    fn test_branch_name_for_spike_ticket() {
        let ticket = make_ticket("SPIKE-001", "SPIKE");
        assert_eq!(branch_name_for_ticket(&ticket), "spike/spike-001");
    }

    #[test]
    fn test_branch_name_for_inv_ticket() {
        let ticket = make_ticket("INV-critical-bug", "INV");
        assert_eq!(branch_name_for_ticket(&ticket), "inv/inv-critical-bug");
    }

    #[test]
    fn test_branch_name_for_task_ticket() {
        let ticket = make_ticket("TASK-cleanup", "TASK");
        assert_eq!(branch_name_for_ticket(&ticket), "task/task-cleanup");
    }

    #[test]
    fn test_branch_name_with_special_chars_in_id() {
        let ticket = make_ticket("FEAT-123.1", "FEAT");
        assert_eq!(branch_name_for_ticket(&ticket), "feat/feat-123-1");
    }
}

//! Git worktree manager for isolated ticket development.
//!
//! Follows vibe-kanban patterns:
//! - Per-ticket worktrees for parallel development
//! - Global locking to prevent race conditions during creation
//! - Comprehensive cleanup on completion

use crate::git::cli::GitCli;
use anyhow::{anyhow, Context, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use tokio::sync::Mutex;
use tracing::{debug, info, instrument, warn};

// Global locks for worktree creation (prevent race conditions)
lazy_static::lazy_static! {
    static ref WORKTREE_CREATION_LOCKS: Mutex<HashMap<PathBuf, Arc<Mutex<()>>>> =
        Mutex::new(HashMap::new());
}

/// Get or create a lock for a specific path
async fn get_path_lock(path: &Path) -> Arc<Mutex<()>> {
    let mut locks = WORKTREE_CREATION_LOCKS.lock().await;
    locks
        .entry(path.to_path_buf())
        .or_insert_with(|| Arc::new(Mutex::new(())))
        .clone()
}

/// Information about a created worktree
#[derive(Debug, Clone)]
pub struct WorktreeInfo {
    /// Path to the worktree directory
    pub path: PathBuf,
    /// Branch name in the worktree
    pub branch: String,
    /// Base commit SHA (merge-base with target branch)
    pub base_commit: String,
    /// Path to the main repository
    pub repo_path: PathBuf,
    /// Target branch this was created from (e.g., "main")
    pub target_branch: String,
}

/// Manages git worktrees for ticket development
pub struct WorktreeManager {
    /// Base directory for all worktrees (e.g., ~/.operator/worktrees/)
    base_worktree_dir: PathBuf,
}

impl WorktreeManager {
    /// Create a new worktree manager
    pub fn new(base_worktree_dir: PathBuf) -> Self {
        Self { base_worktree_dir }
    }

    /// Get the worktree path for a ticket
    pub fn worktree_path(&self, project_name: &str, ticket_id: &str) -> PathBuf {
        self.base_worktree_dir
            .join(project_name)
            .join(ticket_id.to_lowercase())
    }

    /// Create a worktree for a ticket
    ///
    /// # Arguments
    /// * `repo_path` - Path to the main repository
    /// * `project_name` - Project name (for organizing worktrees)
    /// * `ticket_id` - Ticket ID (used in branch name and path)
    /// * `branch_name` - Branch name to create
    /// * `target_branch` - Target branch to base off (e.g., "main")
    #[instrument(skip(self), fields(project = %project_name, ticket = %ticket_id))]
    pub async fn create_for_ticket(
        &self,
        repo_path: &Path,
        project_name: &str,
        ticket_id: &str,
        branch_name: &str,
        target_branch: &str,
    ) -> Result<WorktreeInfo> {
        let worktree_path = self.worktree_path(project_name, ticket_id);

        // Acquire lock for this worktree path
        let lock = get_path_lock(&worktree_path).await;
        let _guard = lock.lock().await;

        info!(?worktree_path, %branch_name, %target_branch, "Creating worktree for ticket");

        // Check if worktree already exists
        if worktree_path.exists() {
            debug!("Worktree already exists, validating");
            return self
                .validate_existing_worktree(&worktree_path, branch_name, repo_path, target_branch)
                .await;
        }

        // Ensure parent directory exists
        if let Some(parent) = worktree_path.parent() {
            fs::create_dir_all(parent)
                .await
                .context("Failed to create worktree parent directory")?;
        }

        // Fetch latest from remote to ensure target branch is up to date
        if let Err(e) = GitCli::fetch(repo_path, "origin").await {
            warn!("Failed to fetch from origin: {}", e);
        }

        // Get the base commit for the target branch
        let base_ref = format!("origin/{}", target_branch);
        let base_commit = GitCli::head_commit(repo_path)
            .await
            .context("Failed to get HEAD commit")?;

        // Create the worktree with a new branch
        GitCli::add_worktree(
            repo_path,
            &worktree_path,
            branch_name,
            true,
            Some(&base_ref),
        )
        .await
        .context("Failed to create worktree")?;

        info!("Worktree created successfully");

        Ok(WorktreeInfo {
            path: worktree_path,
            branch: branch_name.to_string(),
            base_commit,
            repo_path: repo_path.to_path_buf(),
            target_branch: target_branch.to_string(),
        })
    }

    /// Validate an existing worktree and return its info
    async fn validate_existing_worktree(
        &self,
        worktree_path: &Path,
        expected_branch: &str,
        repo_path: &Path,
        target_branch: &str,
    ) -> Result<WorktreeInfo> {
        // Verify it's a valid git worktree
        if !GitCli::is_worktree(worktree_path).await? {
            return Err(anyhow!(
                "Path exists but is not a valid git worktree: {}",
                worktree_path.display()
            ));
        }

        // Get current branch
        let current_branch = GitCli::current_branch(worktree_path).await?;

        // Warn if branch doesn't match but continue
        if current_branch != expected_branch {
            warn!(
                "Existing worktree is on branch '{}', expected '{}'",
                current_branch, expected_branch
            );
        }

        // Get base commit
        let base_commit = GitCli::head_commit(worktree_path).await?;

        Ok(WorktreeInfo {
            path: worktree_path.to_path_buf(),
            branch: current_branch,
            base_commit,
            repo_path: repo_path.to_path_buf(),
            target_branch: target_branch.to_string(),
        })
    }

    /// Ensure a worktree exists for a ticket, creating if necessary
    #[instrument(skip(self), fields(project = %project_name, ticket = %ticket_id))]
    pub async fn ensure_worktree_exists(
        &self,
        repo_path: &Path,
        project_name: &str,
        ticket_id: &str,
        branch_name: &str,
        target_branch: &str,
    ) -> Result<WorktreeInfo> {
        let worktree_path = self.worktree_path(project_name, ticket_id);

        if worktree_path.exists() {
            debug!("Worktree exists, validating");
            self.validate_existing_worktree(&worktree_path, branch_name, repo_path, target_branch)
                .await
        } else {
            self.create_for_ticket(
                repo_path,
                project_name,
                ticket_id,
                branch_name,
                target_branch,
            )
            .await
        }
    }

    /// Cleanup and remove a worktree
    ///
    /// # Arguments
    /// * `worktree` - Worktree info to cleanup
    /// * `prune_branch` - Whether to delete the branch after removal
    /// * `delete_remote_branch` - Whether to also delete from remote
    #[instrument(skip(self, worktree), fields(path = %worktree.path.display()))]
    pub async fn cleanup_worktree(
        &self,
        worktree: &WorktreeInfo,
        prune_branch: bool,
        delete_remote_branch: bool,
    ) -> Result<()> {
        let lock = get_path_lock(&worktree.path).await;
        let _guard = lock.lock().await;

        info!("Cleaning up worktree");

        // Remove the worktree directory
        if worktree.path.exists() {
            // First try git worktree remove
            if let Err(e) =
                GitCli::remove_worktree(&worktree.repo_path, &worktree.path, false).await
            {
                warn!("git worktree remove failed, trying force: {}", e);

                // Try force remove
                if let Err(e) =
                    GitCli::remove_worktree(&worktree.repo_path, &worktree.path, true).await
                {
                    warn!("git worktree remove --force failed: {}", e);

                    // Last resort: remove directory manually
                    if let Err(e) = fs::remove_dir_all(&worktree.path).await {
                        warn!("Failed to remove worktree directory: {}", e);
                    }
                }
            }
        }

        // Prune worktree metadata
        if let Err(e) = GitCli::prune_worktrees(&worktree.repo_path).await {
            warn!("Failed to prune worktrees: {}", e);
        }

        // Delete the branch if requested
        if prune_branch {
            // Delete local branch
            if let Err(e) = GitCli::delete_branch(&worktree.repo_path, &worktree.branch, true).await
            {
                warn!("Failed to delete local branch '{}': {}", worktree.branch, e);
            }

            // Delete remote branch if requested
            if delete_remote_branch {
                if let Err(e) =
                    GitCli::delete_remote_branch(&worktree.repo_path, "origin", &worktree.branch)
                        .await
                {
                    warn!(
                        "Failed to delete remote branch '{}': {}",
                        worktree.branch, e
                    );
                }
            }
        }

        info!("Worktree cleanup complete");
        Ok(())
    }

    /// Check if a worktree has uncommitted changes
    pub async fn is_dirty(&self, worktree: &WorktreeInfo) -> Result<bool> {
        GitCli::is_dirty(&worktree.path).await
    }

    /// Push worktree changes to remote
    #[instrument(skip(self, worktree), fields(path = %worktree.path.display()))]
    pub async fn push_changes(&self, worktree: &WorktreeInfo, set_upstream: bool) -> Result<()> {
        GitCli::push(&worktree.path, "origin", &worktree.branch, set_upstream).await
    }

    /// List all managed worktrees for a project
    pub async fn list_project_worktrees(&self, project_name: &str) -> Result<Vec<PathBuf>> {
        let project_dir = self.base_worktree_dir.join(project_name);

        if !project_dir.exists() {
            return Ok(Vec::new());
        }

        let mut entries = fs::read_dir(&project_dir).await?;
        let mut worktrees = Vec::new();

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_dir() {
                // Verify it's still a valid worktree
                if GitCli::is_worktree(&path).await.unwrap_or(false) {
                    worktrees.push(path);
                }
            }
        }

        Ok(worktrees)
    }

    /// Cleanup all worktrees for a project
    #[instrument(skip(self), fields(project = %project_name))]
    pub async fn cleanup_project_worktrees(
        &self,
        project_name: &str,
        repo_path: &Path,
    ) -> Result<()> {
        let worktrees = self.list_project_worktrees(project_name).await?;

        for path in worktrees {
            info!(?path, "Cleaning up project worktree");

            // Get branch name from worktree
            let branch = match GitCli::current_branch(&path).await {
                Ok(b) => b,
                Err(_) => continue,
            };

            let worktree = WorktreeInfo {
                path: path.clone(),
                branch,
                base_commit: String::new(),
                repo_path: repo_path.to_path_buf(),
                target_branch: String::new(),
            };

            if let Err(e) = self.cleanup_worktree(&worktree, false, false).await {
                warn!("Failed to cleanup worktree {:?}: {}", path, e);
            }
        }

        // Remove the project directory if empty
        let project_dir = self.base_worktree_dir.join(project_name);
        if project_dir.exists() {
            if let Ok(mut entries) = fs::read_dir(&project_dir).await {
                if entries.next_entry().await?.is_none() {
                    let _ = fs::remove_dir(&project_dir).await;
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_worktree_path() {
        let temp = TempDir::new().unwrap();
        let manager = WorktreeManager::new(temp.path().to_path_buf());

        let path = manager.worktree_path("myproject", "FEAT-123");
        assert!(path.ends_with("myproject/feat-123"));
    }

    #[tokio::test]
    async fn test_list_project_worktrees_empty() {
        let temp = TempDir::new().unwrap();
        let manager = WorktreeManager::new(temp.path().to_path_buf());

        let worktrees = manager.list_project_worktrees("nonexistent").await.unwrap();
        assert!(worktrees.is_empty());
    }
}

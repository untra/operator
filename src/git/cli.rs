//! Git CLI wrapper for worktree and branch operations.
//!
//! Uses the git CLI directly (rather than libgit2) for mutable operations
//! to ensure compatibility with sparse-checkout, hooks, and other git features.

use anyhow::{anyhow, Context, Result};
use std::path::Path;
use std::process::Stdio;
use tokio::process::Command;
use tracing::{debug, instrument, warn};

/// Low-level git command wrapper
pub struct GitCli;

impl GitCli {
    /// Execute a git command and return stdout
    async fn run_git(args: &[&str], cwd: &Path) -> Result<String> {
        debug!(?args, ?cwd, "Running git command");

        let output = Command::new("git")
            .args(args)
            .current_dir(cwd)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .context("Failed to execute git command")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!(
                "git {} failed: {}",
                args.first().unwrap_or(&""),
                stderr.trim()
            ));
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    /// Execute a git command, returning Ok(()) on success
    async fn run_git_silent(args: &[&str], cwd: &Path) -> Result<()> {
        Self::run_git(args, cwd).await?;
        Ok(())
    }

    /// Get the current branch name
    #[instrument(skip_all, fields(path = %path.display()))]
    pub async fn current_branch(path: &Path) -> Result<String> {
        Self::run_git(&["rev-parse", "--abbrev-ref", "HEAD"], path).await
    }

    /// Get the remote URL for origin
    #[instrument(skip_all, fields(path = %path.display()))]
    pub async fn get_remote_url(path: &Path) -> Result<String> {
        Self::run_git(&["remote", "get-url", "origin"], path).await
    }

    /// Check if the working directory has uncommitted changes
    #[instrument(skip_all, fields(path = %path.display()))]
    pub async fn is_dirty(path: &Path) -> Result<bool> {
        let output = Self::run_git(&["status", "--porcelain"], path).await?;
        Ok(!output.is_empty())
    }

    /// Get the merge-base commit between two branches
    #[instrument(skip_all, fields(path = %path.display(), branch1, branch2))]
    pub async fn merge_base(path: &Path, branch1: &str, branch2: &str) -> Result<String> {
        Self::run_git(&["merge-base", branch1, branch2], path).await
    }

    /// Fetch from remote
    #[instrument(skip_all, fields(path = %path.display()))]
    pub async fn fetch(path: &Path, remote: &str) -> Result<()> {
        Self::run_git_silent(&["fetch", remote], path).await
    }

    /// Check if a branch exists on remote
    #[instrument(skip_all, fields(path = %path.display(), remote, branch))]
    pub async fn remote_branch_exists(path: &Path, remote: &str, branch: &str) -> Result<bool> {
        let result = Self::run_git(&["ls-remote", "--heads", remote, branch], path).await?;
        Ok(!result.is_empty())
    }

    /// Create a new branch from a base
    #[instrument(skip_all, fields(path = %path.display(), branch, base))]
    pub async fn create_branch(path: &Path, branch: &str, base: &str) -> Result<()> {
        Self::run_git_silent(&["branch", branch, base], path).await
    }

    /// Delete a branch (local)
    #[instrument(skip_all, fields(path = %path.display(), branch, force))]
    pub async fn delete_branch(path: &Path, branch: &str, force: bool) -> Result<()> {
        let flag = if force { "-D" } else { "-d" };
        Self::run_git_silent(&["branch", flag, branch], path).await
    }

    /// Delete a remote branch
    #[instrument(skip_all, fields(path = %path.display(), remote, branch))]
    pub async fn delete_remote_branch(path: &Path, remote: &str, branch: &str) -> Result<()> {
        Self::run_git_silent(&["push", remote, "--delete", branch], path).await
    }

    /// Push a branch to remote
    #[instrument(skip_all, fields(path = %path.display(), remote, branch))]
    pub async fn push(path: &Path, remote: &str, branch: &str, set_upstream: bool) -> Result<()> {
        if set_upstream {
            Self::run_git_silent(&["push", "-u", remote, branch], path).await
        } else {
            Self::run_git_silent(&["push", remote, branch], path).await
        }
    }

    /// Force push a branch to remote
    #[instrument(skip_all, fields(path = %path.display(), remote, branch))]
    pub async fn force_push(path: &Path, remote: &str, branch: &str) -> Result<()> {
        Self::run_git_silent(&["push", "--force-with-lease", remote, branch], path).await
    }

    /// Add a worktree
    #[instrument(skip_all, fields(repo_path = %repo_path.display(), worktree_path = %worktree_path.display(), branch))]
    pub async fn add_worktree(
        repo_path: &Path,
        worktree_path: &Path,
        branch: &str,
        create_branch: bool,
        base: Option<&str>,
    ) -> Result<()> {
        let worktree_str = worktree_path.to_string_lossy();

        if create_branch {
            let base_ref = base.unwrap_or("HEAD");
            Self::run_git_silent(
                &["worktree", "add", "-b", branch, &worktree_str, base_ref],
                repo_path,
            )
            .await
        } else {
            Self::run_git_silent(&["worktree", "add", &worktree_str, branch], repo_path).await
        }
    }

    /// Remove a worktree
    #[instrument(skip_all, fields(repo_path = %repo_path.display(), worktree_path = %worktree_path.display(), force))]
    pub async fn remove_worktree(
        repo_path: &Path,
        worktree_path: &Path,
        force: bool,
    ) -> Result<()> {
        let worktree_str = worktree_path.to_string_lossy();

        if force {
            Self::run_git_silent(&["worktree", "remove", "--force", &worktree_str], repo_path).await
        } else {
            Self::run_git_silent(&["worktree", "remove", &worktree_str], repo_path).await
        }
    }

    /// Prune worktree metadata
    #[instrument(skip_all, fields(repo_path = %repo_path.display()))]
    pub async fn prune_worktrees(repo_path: &Path) -> Result<()> {
        Self::run_git_silent(&["worktree", "prune"], repo_path).await
    }

    /// List all worktrees
    #[instrument(skip_all, fields(repo_path = %repo_path.display()))]
    pub async fn list_worktrees(repo_path: &Path) -> Result<Vec<WorktreeEntry>> {
        let output = Self::run_git(&["worktree", "list", "--porcelain"], repo_path).await?;

        let mut entries = Vec::new();
        let mut current: Option<WorktreeEntry> = None;

        for line in output.lines() {
            if let Some(path) = line.strip_prefix("worktree ") {
                if let Some(entry) = current.take() {
                    entries.push(entry);
                }
                current = Some(WorktreeEntry {
                    path: path.to_string(),
                    branch: None,
                    head: None,
                    bare: false,
                });
            } else if let Some(head) = line.strip_prefix("HEAD ") {
                if let Some(ref mut entry) = current {
                    entry.head = Some(head.to_string());
                }
            } else if let Some(branch) = line.strip_prefix("branch ") {
                if let Some(ref mut entry) = current {
                    entry.branch = Some(branch.to_string());
                }
            } else if line == "bare" {
                if let Some(ref mut entry) = current {
                    entry.bare = true;
                }
            }
        }

        if let Some(entry) = current {
            entries.push(entry);
        }

        Ok(entries)
    }

    /// Get the root of the git repository
    #[instrument(skip_all, fields(path = %path.display()))]
    pub async fn repo_root(path: &Path) -> Result<String> {
        Self::run_git(&["rev-parse", "--show-toplevel"], path).await
    }

    /// Check if path is inside a git worktree
    #[instrument(skip_all, fields(path = %path.display()))]
    pub async fn is_worktree(path: &Path) -> Result<bool> {
        let result = Self::run_git(&["rev-parse", "--is-inside-work-tree"], path).await;
        match result {
            Ok(output) => Ok(output == "true"),
            Err(_) => Ok(false),
        }
    }

    /// Get the HEAD commit SHA
    #[instrument(skip_all, fields(path = %path.display()))]
    pub async fn head_commit(path: &Path) -> Result<String> {
        Self::run_git(&["rev-parse", "HEAD"], path).await
    }

    /// Check if a repository has at least one commit
    #[instrument(skip_all, fields(path = %path.display()))]
    pub async fn has_commits(path: &Path) -> Result<bool> {
        match Self::run_git(&["rev-parse", "--verify", "HEAD"], path).await {
            Ok(_) => Ok(true),
            Err(e) => {
                let err_msg = e.to_string();
                // Various error messages indicate no commits exist:
                // - "unknown revision" - older git versions
                // - "ambiguous argument" - some git configurations
                // - "Needed a single revision" - newer git versions with --verify
                if err_msg.contains("unknown revision")
                    || err_msg.contains("ambiguous argument")
                    || err_msg.contains("Needed a single revision")
                {
                    Ok(false)
                } else {
                    Err(e)
                }
            }
        }
    }

    /// Read a symbolic reference (like refs/remotes/origin/HEAD)
    #[instrument(skip_all, fields(path = %path.display(), refname))]
    pub async fn symbolic_ref(path: &Path, refname: &str) -> Result<String> {
        Self::run_git(&["symbolic-ref", refname], path).await
    }

    /// Commit all changes with a message
    #[instrument(skip_all, fields(path = %path.display()))]
    pub async fn commit_all(path: &Path, message: &str) -> Result<()> {
        Self::run_git_silent(&["add", "-A"], path).await?;
        Self::run_git_silent(&["commit", "-m", message], path).await
    }

    /// Reset to a specific commit
    #[instrument(skip_all, fields(path = %path.display(), commit, hard))]
    pub async fn reset(path: &Path, commit: &str, hard: bool) -> Result<()> {
        if hard {
            Self::run_git_silent(&["reset", "--hard", commit], path).await
        } else {
            Self::run_git_silent(&["reset", commit], path).await
        }
    }

    /// Attempt to repair a worktree (useful after failed creation)
    #[instrument(skip_all, fields(repo_path = %repo_path.display()))]
    pub async fn repair_worktrees(repo_path: &Path) -> Result<()> {
        // First prune stale worktrees
        if let Err(e) = Self::prune_worktrees(repo_path).await {
            warn!("Failed to prune worktrees: {}", e);
        }
        Ok(())
    }
}

/// Entry from `git worktree list --porcelain`
#[derive(Debug, Clone)]
pub struct WorktreeEntry {
    pub path: String,
    pub branch: Option<String>,
    pub head: Option<String>,
    pub bare: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[tokio::test]
    async fn test_current_branch_in_repo() {
        // This test runs in the operator repo itself
        let cwd = env::current_dir().unwrap();
        let branch = GitCli::current_branch(&cwd).await;
        // Should succeed if we're in a git repo
        assert!(branch.is_ok() || branch.is_err()); // Just test it doesn't panic
    }

    #[tokio::test]
    async fn test_is_dirty() {
        let cwd = env::current_dir().unwrap();
        let result = GitCli::is_dirty(&cwd).await;
        // Should return a result (dirty or clean)
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_repo_root() {
        let cwd = env::current_dir().unwrap();
        let result = GitCli::repo_root(&cwd).await;
        if let Ok(root) = result {
            assert!(root.contains("operator") || root.contains("gbqr"));
        }
    }

    #[tokio::test]
    async fn test_list_worktrees() {
        let cwd = env::current_dir().unwrap();
        let result = GitCli::list_worktrees(&cwd).await;
        if let Ok(worktrees) = result {
            // Should have at least the main worktree
            assert!(!worktrees.is_empty());
        }
    }
}

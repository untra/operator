//! Unified Project type - combines git-focus (vibe-kanban) + AI-focus (Operator)
//!
//! This type bridges the gap between:
//! - vibe-kanban's git-centric project model (repos, worktrees, dev scripts)
//! - Operator's AI-centric project model (CLAUDE.md, taxonomy, analysis)

use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use ts_rs::TS;
use uuid::Uuid;

/// Unified Project - combines git-focus (vibe-kanban) + AI-focus (Operator)
///
/// A Project represents a codebase that can be worked on by AI agents.
/// It may contain multiple git repositories and has configuration for
/// both git operations and AI agent execution.
#[derive(Debug, Clone, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export)]
pub struct Project {
    /// Unique identifier
    pub id: Uuid,

    /// Human-readable project name
    pub name: String,

    /// Filesystem path to the project root
    #[ts(type = "string")]
    pub path: PathBuf,

    // ─────────────────────────────────────────────────────────────────────
    // Git integration (from vibe-kanban)
    // ─────────────────────────────────────────────────────────────────────
    /// Associated git repositories (can be multiple for monorepos)
    #[serde(default)]
    pub repos: Vec<ProjectRepo>,

    /// Default branch for merging (e.g., "main", "master")
    #[serde(default)]
    pub default_branch: Option<String>,

    // ─────────────────────────────────────────────────────────────────────
    // AI configuration (from Operator)
    // ─────────────────────────────────────────────────────────────────────
    /// Path to the AI context file (CLAUDE.md, GEMINI.md, etc.)
    #[serde(default)]
    #[ts(type = "string | null")]
    pub ai_context_path: Option<PathBuf>,

    /// Backstage taxonomy kind (tier 1-5)
    #[serde(default)]
    pub kind: Option<String>,

    /// Tags for categorization
    #[serde(default)]
    pub tags: Vec<String>,

    // ─────────────────────────────────────────────────────────────────────
    // Execution configuration (both systems)
    // ─────────────────────────────────────────────────────────────────────
    /// Development server startup command
    #[serde(default)]
    pub dev_script: Option<String>,

    /// Working directory for dev script execution
    #[serde(default)]
    #[ts(type = "string | null")]
    pub dev_script_working_dir: Option<PathBuf>,

    /// Default working directory for AI agents
    #[serde(default)]
    #[ts(type = "string | null")]
    pub default_agent_working_dir: Option<PathBuf>,

    /// Setup script to run before agent execution
    #[serde(default)]
    pub setup_script: Option<String>,

    /// Cleanup script to run after agent execution
    #[serde(default)]
    pub cleanup_script: Option<String>,

    // ─────────────────────────────────────────────────────────────────────
    // Remote integration (from vibe-kanban)
    // ─────────────────────────────────────────────────────────────────────
    /// Link to shared organizational project
    #[serde(default)]
    pub remote_project_id: Option<Uuid>,

    // ─────────────────────────────────────────────────────────────────────
    // Timestamps
    // ─────────────────────────────────────────────────────────────────────
    #[ts(type = "string")]
    pub created_at: DateTime<Utc>,

    #[ts(type = "string")]
    pub updated_at: DateTime<Utc>,
}

impl Project {
    /// Create a new project with minimal required fields
    pub fn new(name: String, path: PathBuf) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            path,
            repos: Vec::new(),
            default_branch: None,
            ai_context_path: None,
            kind: None,
            tags: Vec::new(),
            dev_script: None,
            dev_script_working_dir: None,
            default_agent_working_dir: None,
            setup_script: None,
            cleanup_script: None,
            remote_project_id: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Create from an existing directory with AI context file
    pub fn from_discovery(name: String, path: PathBuf, ai_context_path: Option<PathBuf>) -> Self {
        let mut project = Self::new(name, path.clone());
        project.ai_context_path = ai_context_path;

        // If the path is a git repo, add it as a repo
        if path.join(".git").exists() {
            project.repos.push(ProjectRepo::new(path));
        }

        project
    }
}

/// A git repository associated with a project
///
/// Projects can have multiple repos (monorepo pattern) or a single repo.
/// Each repo can have its own setup/cleanup scripts for worktree operations.
#[derive(Debug, Clone, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export)]
pub struct ProjectRepo {
    /// Unique identifier for this repo association
    pub id: Uuid,

    /// Absolute path to the repository root
    #[ts(type = "string")]
    pub path: PathBuf,

    /// Whether this is the primary repo for the project
    #[serde(default)]
    pub is_primary: bool,

    /// Setup script to run when creating worktrees for this repo
    #[serde(default)]
    pub setup_script: Option<String>,

    /// Cleanup script to run when deleting worktrees
    #[serde(default)]
    pub cleanup_script: Option<String>,

    /// Files to copy into worktrees (e.g., .env files)
    #[serde(default)]
    pub copy_files: Vec<String>,
}

impl ProjectRepo {
    /// Create a new repo association
    pub fn new(path: PathBuf) -> Self {
        Self {
            id: Uuid::new_v4(),
            path,
            is_primary: true,
            setup_script: None,
            cleanup_script: None,
            copy_files: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_new() {
        let project = Project::new("test-project".to_string(), PathBuf::from("/tmp/test"));

        assert_eq!(project.name, "test-project");
        assert_eq!(project.path, PathBuf::from("/tmp/test"));
        assert!(project.repos.is_empty());
        assert!(project.ai_context_path.is_none());
    }

    #[test]
    fn test_project_from_discovery_with_git() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().to_path_buf();

        // Create .git directory
        std::fs::create_dir(path.join(".git")).unwrap();

        let project = Project::from_discovery(
            "discovered".to_string(),
            path.clone(),
            Some(path.join("CLAUDE.md")),
        );

        assert_eq!(project.name, "discovered");
        assert_eq!(project.repos.len(), 1);
        assert_eq!(project.repos[0].path, path);
        assert!(project.repos[0].is_primary);
    }

    #[test]
    fn test_project_from_discovery_without_git() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().to_path_buf();

        let project = Project::from_discovery("no-git".to_string(), path, None);

        assert!(project.repos.is_empty());
        assert!(project.ai_context_path.is_none());
    }

    #[test]
    fn test_project_repo_new() {
        let repo = ProjectRepo::new(PathBuf::from("/tmp/repo"));

        assert_eq!(repo.path, PathBuf::from("/tmp/repo"));
        assert!(repo.is_primary);
        assert!(repo.setup_script.is_none());
        assert!(repo.copy_files.is_empty());
    }
}

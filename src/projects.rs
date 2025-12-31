//! Project discovery by scanning for LLM tool marker files and git repositories.
//!
//! Supports two discovery modes:
//! - **Marker-based**: Scan for CLAUDE.md, GEMINI.md, CODEX.md files
//! - **Git-based**: Scan for .git directories and extract repo info

#![allow(dead_code)] // Git discovery types for future integration

use crate::types::pr::GitHubRepoInfo;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use ts_rs::TS;

/// Marker files for each LLM tool
pub const TOOL_MARKERS: &[(&str, &str)] = &[
    ("claude", "CLAUDE.md"),
    ("gemini", "GEMINI.md"),
    ("codex", "CODEX.md"),
];

/// A discovered project with git and LLM tool information
#[derive(Debug, Clone, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export)]
pub struct DiscoveredProject {
    /// Project name (directory name)
    pub name: String,
    /// Absolute path to project root
    #[ts(type = "string")]
    pub path: PathBuf,
    /// LLM tools available (from marker files)
    pub llm_tools: Vec<String>,
    /// Git repository info (if .git exists)
    pub git_info: Option<GitRepoInfo>,
}

/// Git repository information
#[derive(Debug, Clone, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export)]
pub struct GitRepoInfo {
    /// Remote origin URL
    pub remote_url: Option<String>,
    /// Parsed GitHub info (if GitHub remote)
    pub github_info: Option<GitHubRepoInfo>,
    /// Default branch name
    pub default_branch: String,
    /// Whether repo has uncommitted changes
    pub is_dirty: bool,
}

/// Discover projects by scanning for .git directories
///
/// Returns a list of discovered projects with git repo info and LLM tool availability.
/// Projects are included if they have either a .git directory or LLM marker files.
pub fn discover_projects_with_git(projects_path: &Path) -> Vec<DiscoveredProject> {
    let mut projects = Vec::new();

    if let Ok(entries) = fs::read_dir(projects_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();

                // Skip hidden directories
                if name.starts_with('.') {
                    continue;
                }

                // Check for .git directory
                let git_info = if path.join(".git").exists() {
                    Some(extract_git_info(&path))
                } else {
                    None
                };

                // Check for LLM marker files
                let llm_tools: Vec<String> = TOOL_MARKERS
                    .iter()
                    .filter(|(_, marker)| path.join(marker).exists())
                    .map(|(tool, _)| tool.to_string())
                    .collect();

                // Include if has git OR has LLM markers
                if git_info.is_some() || !llm_tools.is_empty() {
                    projects.push(DiscoveredProject {
                        name,
                        path: path.clone(),
                        llm_tools,
                        git_info,
                    });
                }
            }
        }
    }

    projects.sort_by(|a, b| a.name.cmp(&b.name));
    projects
}

/// Extract git repository information from a project path
fn extract_git_info(path: &Path) -> GitRepoInfo {
    // Get remote origin URL
    let remote_url = Command::new("git")
        .args(["remote", "get-url", "origin"])
        .current_dir(path)
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                Some(String::from_utf8_lossy(&o.stdout).trim().to_string())
            } else {
                None
            }
        });

    // Parse GitHub info from remote URL
    let github_info = remote_url
        .as_ref()
        .and_then(|url| GitHubRepoInfo::from_remote_url(url).ok());

    // Detect default branch
    let default_branch = detect_default_branch(path);

    // Check for uncommitted changes
    let is_dirty = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(path)
        .output()
        .map(|o| o.status.success() && !o.stdout.is_empty())
        .unwrap_or(false);

    GitRepoInfo {
        remote_url,
        github_info,
        default_branch,
        is_dirty,
    }
}

/// Detect the default branch for a repository
fn detect_default_branch(path: &Path) -> String {
    // Try to read origin/HEAD symbolic ref
    if let Ok(output) = Command::new("git")
        .args(["symbolic-ref", "refs/remotes/origin/HEAD"])
        .current_dir(path)
        .output()
    {
        if output.status.success() {
            let full_ref = String::from_utf8_lossy(&output.stdout);
            if let Some(branch) = full_ref.trim().strip_prefix("refs/remotes/origin/") {
                return branch.to_string();
            }
        }
    }

    // Fallback: check common default branch names
    for branch in &["main", "master", "develop"] {
        let check = Command::new("git")
            .args(["rev-parse", "--verify", &format!("origin/{}", branch)])
            .current_dir(path)
            .output();
        if check.map(|o| o.status.success()).unwrap_or(false) {
            return branch.to_string();
        }
    }

    // Final fallback
    "main".to_string()
}

/// Discover projects by tool, scanning for tool-specific marker files
///
/// Returns a map of tool_name → list of project names.
/// A project can appear under multiple tools if it has multiple marker files.
pub fn discover_projects_by_tool(projects_path: &Path) -> HashMap<String, Vec<String>> {
    let mut projects_by_tool: HashMap<String, Vec<String>> = HashMap::new();

    if let Ok(entries) = fs::read_dir(projects_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let dir_name = path.file_name().map(|n| n.to_string_lossy().to_string());

                if let Some(name) = dir_name {
                    // Check each tool marker
                    for (tool, marker) in TOOL_MARKERS {
                        if path.join(marker).exists() {
                            projects_by_tool
                                .entry(tool.to_string())
                                .or_default()
                                .push(name.clone());
                        }
                    }
                }
            }
        }
    }

    // Sort each tool's project list
    for projects in projects_by_tool.values_mut() {
        projects.sort();
    }

    projects_by_tool
}

/// Discover projects by scanning for subdirectories with any LLM tool marker file
///
/// Returns a sorted list of directory names that contain at least one marker file.
pub fn discover_projects(projects_path: &Path) -> Vec<String> {
    let by_tool = discover_projects_by_tool(projects_path);
    let mut all: Vec<String> = by_tool.values().flatten().cloned().collect();
    all.sort();
    all.dedup();
    all
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::tempdir;

    #[test]
    fn test_discover_projects_finds_claude_md() {
        let temp = tempdir().unwrap();

        // Create project with CLAUDE.md
        let project_a = temp.path().join("project-a");
        fs::create_dir(&project_a).unwrap();
        File::create(project_a.join("CLAUDE.md")).unwrap();

        // Create project without any marker file
        let project_b = temp.path().join("project-b");
        fs::create_dir(&project_b).unwrap();

        // Create another project with CLAUDE.md
        let project_c = temp.path().join("project-c");
        fs::create_dir(&project_c).unwrap();
        File::create(project_c.join("CLAUDE.md")).unwrap();

        let projects = discover_projects(temp.path());

        assert_eq!(projects.len(), 2);
        assert!(projects.contains(&"project-a".to_string()));
        assert!(projects.contains(&"project-c".to_string()));
        assert!(!projects.contains(&"project-b".to_string()));
    }

    #[test]
    fn test_discover_projects_returns_sorted() {
        let temp = tempdir().unwrap();

        for name in &["zebra", "apple", "mango"] {
            let project = temp.path().join(name);
            fs::create_dir(&project).unwrap();
            File::create(project.join("CLAUDE.md")).unwrap();
        }

        let projects = discover_projects(temp.path());

        assert_eq!(projects, vec!["apple", "mango", "zebra"]);
    }

    #[test]
    fn test_discover_projects_empty_dir() {
        let temp = tempdir().unwrap();
        let projects = discover_projects(temp.path());
        assert!(projects.is_empty());
    }

    #[test]
    fn test_discover_projects_nonexistent_path() {
        let projects = discover_projects(Path::new("/nonexistent/path"));
        assert!(projects.is_empty());
    }

    #[test]
    fn test_discover_projects_by_tool() {
        let temp = tempdir().unwrap();

        // Create project with CLAUDE.md only
        let project_a = temp.path().join("project-a");
        fs::create_dir(&project_a).unwrap();
        File::create(project_a.join("CLAUDE.md")).unwrap();

        // Create project with GEMINI.md only
        let project_b = temp.path().join("project-b");
        fs::create_dir(&project_b).unwrap();
        File::create(project_b.join("GEMINI.md")).unwrap();

        // Create project with both CLAUDE.md and GEMINI.md
        let project_c = temp.path().join("project-c");
        fs::create_dir(&project_c).unwrap();
        File::create(project_c.join("CLAUDE.md")).unwrap();
        File::create(project_c.join("GEMINI.md")).unwrap();

        // Create project with CODEX.md only
        let project_d = temp.path().join("project-d");
        fs::create_dir(&project_d).unwrap();
        File::create(project_d.join("CODEX.md")).unwrap();

        let by_tool = discover_projects_by_tool(temp.path());

        // Claude should have project-a and project-c
        let claude_projects = by_tool.get("claude").unwrap();
        assert_eq!(claude_projects.len(), 2);
        assert!(claude_projects.contains(&"project-a".to_string()));
        assert!(claude_projects.contains(&"project-c".to_string()));

        // Gemini should have project-b and project-c
        let gemini_projects = by_tool.get("gemini").unwrap();
        assert_eq!(gemini_projects.len(), 2);
        assert!(gemini_projects.contains(&"project-b".to_string()));
        assert!(gemini_projects.contains(&"project-c".to_string()));

        // Codex should have project-d only
        let codex_projects = by_tool.get("codex").unwrap();
        assert_eq!(codex_projects.len(), 1);
        assert!(codex_projects.contains(&"project-d".to_string()));
    }

    #[test]
    fn test_discover_projects_by_tool_sorted() {
        let temp = tempdir().unwrap();

        for name in &["zebra", "apple", "mango"] {
            let project = temp.path().join(name);
            fs::create_dir(&project).unwrap();
            File::create(project.join("CLAUDE.md")).unwrap();
        }

        let by_tool = discover_projects_by_tool(temp.path());
        let claude_projects = by_tool.get("claude").unwrap();

        assert_eq!(claude_projects, &vec!["apple", "mango", "zebra"]);
    }

    #[test]
    fn test_discover_projects_deduplicates() {
        let temp = tempdir().unwrap();

        // Create project with both CLAUDE.md and GEMINI.md
        let project_a = temp.path().join("project-a");
        fs::create_dir(&project_a).unwrap();
        File::create(project_a.join("CLAUDE.md")).unwrap();
        File::create(project_a.join("GEMINI.md")).unwrap();

        // discover_projects should return each project only once
        let projects = discover_projects(temp.path());
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0], "project-a");
    }

    #[test]
    fn test_tool_markers_contains_expected_tools() {
        let tool_names: Vec<&str> = TOOL_MARKERS.iter().map(|(t, _)| *t).collect();
        assert!(tool_names.contains(&"claude"));
        assert!(tool_names.contains(&"gemini"));
        assert!(tool_names.contains(&"codex"));
    }

    #[test]
    fn test_tool_markers_has_correct_filenames() {
        let markers: std::collections::HashMap<&str, &str> = TOOL_MARKERS.iter().cloned().collect();
        assert_eq!(markers.get("claude"), Some(&"CLAUDE.md"));
        assert_eq!(markers.get("gemini"), Some(&"GEMINI.md"));
        assert_eq!(markers.get("codex"), Some(&"CODEX.md"));
    }

    #[test]
    fn test_discover_projects_ignores_files() {
        let temp = tempdir().unwrap();

        // Create a file (not directory) named like a project
        File::create(temp.path().join("not-a-dir")).unwrap();

        // Create actual project directory
        let project = temp.path().join("real-project");
        fs::create_dir(&project).unwrap();
        File::create(project.join("CLAUDE.md")).unwrap();

        let projects = discover_projects(temp.path());
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0], "real-project");
    }

    #[test]
    fn test_discover_projects_by_tool_empty_for_missing_tool() {
        let temp = tempdir().unwrap();

        // Only create CLAUDE.md projects
        let project = temp.path().join("project-a");
        fs::create_dir(&project).unwrap();
        File::create(project.join("CLAUDE.md")).unwrap();

        let by_tool = discover_projects_by_tool(temp.path());

        // claude should have projects
        assert!(by_tool.contains_key("claude"));
        // gemini and codex should be None (not even empty Vec)
        assert!(!by_tool.contains_key("gemini"));
        assert!(!by_tool.contains_key("codex"));
    }

    // Git discovery tests

    #[test]
    fn test_discover_projects_with_git_finds_git_repos() {
        let temp = tempdir().unwrap();

        // Create project with .git directory
        let project = temp.path().join("my-project");
        fs::create_dir(&project).unwrap();
        fs::create_dir(project.join(".git")).unwrap();

        let projects = discover_projects_with_git(temp.path());
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].name, "my-project");
        assert!(projects[0].git_info.is_some());
    }

    #[test]
    fn test_discover_projects_with_git_includes_both_git_and_marker() {
        let temp = tempdir().unwrap();

        // Git-only project
        let git_proj = temp.path().join("git-only");
        fs::create_dir(&git_proj).unwrap();
        fs::create_dir(git_proj.join(".git")).unwrap();

        // Marker-only project
        let marker_proj = temp.path().join("marker-only");
        fs::create_dir(&marker_proj).unwrap();
        File::create(marker_proj.join("CLAUDE.md")).unwrap();

        // Both git and marker
        let both_proj = temp.path().join("both");
        fs::create_dir(&both_proj).unwrap();
        fs::create_dir(both_proj.join(".git")).unwrap();
        File::create(both_proj.join("CLAUDE.md")).unwrap();

        let projects = discover_projects_with_git(temp.path());
        assert_eq!(projects.len(), 3);

        // Find each project
        let git_only = projects.iter().find(|p| p.name == "git-only").unwrap();
        assert!(git_only.git_info.is_some());
        assert!(git_only.llm_tools.is_empty());

        let marker_only = projects.iter().find(|p| p.name == "marker-only").unwrap();
        assert!(marker_only.git_info.is_none());
        assert!(marker_only.llm_tools.contains(&"claude".to_string()));

        let both = projects.iter().find(|p| p.name == "both").unwrap();
        assert!(both.git_info.is_some());
        assert!(both.llm_tools.contains(&"claude".to_string()));
    }

    #[test]
    fn test_discover_projects_with_git_skips_hidden_dirs() {
        let temp = tempdir().unwrap();

        // Create hidden directory with .git
        let hidden = temp.path().join(".hidden-project");
        fs::create_dir(&hidden).unwrap();
        fs::create_dir(hidden.join(".git")).unwrap();

        // Create normal project
        let project = temp.path().join("normal-project");
        fs::create_dir(&project).unwrap();
        fs::create_dir(project.join(".git")).unwrap();

        let projects = discover_projects_with_git(temp.path());
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].name, "normal-project");
    }

    #[test]
    fn test_discover_projects_with_git_sorted() {
        let temp = tempdir().unwrap();

        for name in &["zebra", "apple", "mango"] {
            let project = temp.path().join(name);
            fs::create_dir(&project).unwrap();
            fs::create_dir(project.join(".git")).unwrap();
        }

        let projects = discover_projects_with_git(temp.path());
        let names: Vec<&str> = projects.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(names, vec!["apple", "mango", "zebra"]);
    }

    #[test]
    fn test_discover_projects_with_git_empty_dir() {
        let temp = tempdir().unwrap();
        let projects = discover_projects_with_git(temp.path());
        assert!(projects.is_empty());
    }

    #[test]
    fn test_discover_projects_with_git_nonexistent_path() {
        let projects = discover_projects_with_git(Path::new("/nonexistent/path"));
        assert!(projects.is_empty());
    }

    #[test]
    fn test_git_repo_info_default_branch_fallback() {
        let temp = tempdir().unwrap();

        // Create project with .git but no remotes (bare minimum)
        let project = temp.path().join("new-repo");
        fs::create_dir(&project).unwrap();

        // Initialize a real git repo
        let _ = std::process::Command::new("git")
            .args(["init"])
            .current_dir(&project)
            .output();

        let projects = discover_projects_with_git(temp.path());
        assert_eq!(projects.len(), 1);

        let git_info = projects[0].git_info.as_ref().unwrap();
        // Should have a default branch (fallback to "main")
        assert!(!git_info.default_branch.is_empty());
    }

    #[test]
    fn test_discovered_project_has_correct_path() {
        let temp = tempdir().unwrap();

        let project = temp.path().join("my-project");
        fs::create_dir(&project).unwrap();
        fs::create_dir(project.join(".git")).unwrap();

        let projects = discover_projects_with_git(temp.path());
        assert_eq!(projects[0].path, project);
    }

    #[test]
    fn test_discovered_project_multiple_llm_tools() {
        let temp = tempdir().unwrap();

        let project = temp.path().join("multi-tool");
        fs::create_dir(&project).unwrap();
        File::create(project.join("CLAUDE.md")).unwrap();
        File::create(project.join("GEMINI.md")).unwrap();
        File::create(project.join("CODEX.md")).unwrap();

        let projects = discover_projects_with_git(temp.path());
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].llm_tools.len(), 3);
        assert!(projects[0].llm_tools.contains(&"claude".to_string()));
        assert!(projects[0].llm_tools.contains(&"gemini".to_string()));
        assert!(projects[0].llm_tools.contains(&"codex".to_string()));
    }
}

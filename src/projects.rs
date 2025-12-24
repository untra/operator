//! Project discovery by scanning for LLM tool marker files (CLAUDE.md, GEMINI.md, CODEX.md)

use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Marker files for each LLM tool
pub const TOOL_MARKERS: &[(&str, &str)] = &[
    ("claude", "CLAUDE.md"),
    ("gemini", "GEMINI.md"),
    ("codex", "CODEX.md"),
];

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
}

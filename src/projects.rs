//! Project discovery by scanning for CLAUDE.md files

use std::fs;
use std::path::Path;

/// Discover projects by scanning for subdirectories with CLAUDE.md files
///
/// Returns a sorted list of directory names that contain a CLAUDE.md file.
pub fn discover_projects(projects_path: &Path) -> Vec<String> {
    let mut projects = Vec::new();

    if let Ok(entries) = fs::read_dir(projects_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let claude_md = path.join("CLAUDE.md");
                if claude_md.exists() {
                    if let Some(name) = path.file_name() {
                        projects.push(name.to_string_lossy().to_string());
                    }
                }
            }
        }
    }

    projects.sort();
    projects
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

        // Create project without CLAUDE.md
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
}

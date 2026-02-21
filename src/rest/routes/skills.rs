//! Skills discovery endpoint.
//!
//! Scans tool skill directories for .md files and returns them tagged by tool.

use axum::{extract::State, Json};

use crate::llm::tool_config::load_all_tool_configs;
use crate::rest::dto::{SkillEntry, SkillsResponse};
use crate::rest::state::ApiState;

/// List all discovered skills across LLM tools
#[utoipa::path(
    get,
    path = "/api/v1/skills",
    tag = "Skills",
    responses(
        (status = 200, description = "List of discovered skill files", body = SkillsResponse)
    )
)]
pub async fn list(State(state): State<ApiState>) -> Json<SkillsResponse> {
    let config = state.config.clone();
    let tool_configs = load_all_tool_configs();
    let mut skills = Vec::new();

    for tc in &tool_configs {
        let skill_dirs = match &tc.skill_directories {
            Some(dirs) => dirs.clone(),
            None => continue,
        };

        // Check for per-tool overrides in config
        let overrides = config
            .llm_tools
            .skill_directory_overrides
            .get(&tc.tool_name);

        // Scan global directories
        let mut global_dirs = skill_dirs.global.clone();
        if let Some(ov) = overrides {
            global_dirs.extend(ov.global.iter().cloned());
        }
        for dir in &global_dirs {
            let expanded = expand_tilde(dir);
            scan_directory(&expanded, &tc.tool_name, "global", &mut skills);
        }

        // Scan project directories (relative to cwd)
        let mut project_dirs = skill_dirs.project.clone();
        if let Some(ov) = overrides {
            project_dirs.extend(ov.project.iter().cloned());
        }
        let cwd = std::env::current_dir().unwrap_or_default();
        for dir in &project_dirs {
            let full_path = cwd.join(dir);
            // Handle single-file entries (e.g., "AGENTS.md")
            if full_path.is_file() {
                if let Some(filename) = full_path.file_name() {
                    skills.push(SkillEntry {
                        tool_name: tc.tool_name.clone(),
                        filename: filename.to_string_lossy().to_string(),
                        file_path: full_path.to_string_lossy().to_string(),
                        scope: "project".to_string(),
                    });
                }
            } else {
                scan_directory(
                    &full_path.to_string_lossy(),
                    &tc.tool_name,
                    "project",
                    &mut skills,
                );
            }
        }
    }

    let total = skills.len();
    Json(SkillsResponse { skills, total })
}

/// Expand ~ to home directory
fn expand_tilde(path: &str) -> String {
    if path.starts_with("~/") {
        if let Some(home) = dirs::home_dir() {
            return format!("{}{}", home.display(), &path[1..]);
        }
    }
    path.to_string()
}

/// Scan a directory for .md files and add them to the skills list
fn scan_directory(dir: &str, tool_name: &str, scope: &str, skills: &mut Vec<SkillEntry>) {
    let path = std::path::Path::new(dir);
    if !path.is_dir() {
        return;
    }

    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let entry_path = entry.path();
            if entry_path.is_file() {
                if let Some(ext) = entry_path.extension() {
                    if ext == "md" {
                        if let Some(filename) = entry_path.file_name() {
                            skills.push(SkillEntry {
                                tool_name: tool_name.to_string(),
                                filename: filename.to_string_lossy().to_string(),
                                file_path: entry_path.to_string_lossy().to_string(),
                                scope: scope.to_string(),
                            });
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_tilde() {
        let expanded = expand_tilde("~/test");
        assert!(!expanded.starts_with("~/"));
        assert!(expanded.ends_with("/test"));
    }

    #[test]
    fn test_expand_tilde_no_tilde() {
        assert_eq!(expand_tilde("/usr/local/bin"), "/usr/local/bin");
    }

    #[test]
    fn test_scan_nonexistent_directory() {
        let mut skills = Vec::new();
        scan_directory("/nonexistent/path", "test", "global", &mut skills);
        assert!(skills.is_empty());
    }

    #[tokio::test]
    async fn test_skills_endpoint() {
        use crate::config::Config;
        use std::path::PathBuf;

        let config = Config::default();
        let state = ApiState::new(config, PathBuf::from("/tmp/test"));

        let resp = list(State(state)).await;
        // Should return without error (skills may be empty)
        assert!(resp.total == resp.skills.len());
    }
}

//! Skill deployment to agent-native command paths
//!
//! Deploys operator workflow skills (e.g., step transition) to each agent's
//! native command directory so they appear as first-class slash commands.
//!
//! Path mapping per agent:
//! - Claude: `.claude/commands/operator/{name}.md`
//! - Gemini: `.gemini/commands/operator/{name}.toml`
//! - Codex:  `.codex/skills/operator-{name}/SKILL.md`

use std::fs;
use std::path::Path;

use anyhow::Result;
use tracing::warn;

/// Built-in step skill content, embedded at compile time.
const STEP_SKILL: &str = include_str!("../../skills/step.md");

/// Built-in skills: (name, content)
const BUILT_IN_SKILLS: &[(&str, &str)] = &[("step", STEP_SKILL)];

/// Deploy operator skills to agent-native command paths.
///
/// `working_dir` is where skills are written (worktree path OR project path).
/// `project_path` is the original project root (used to find custom overrides).
/// `tools` lists which agent tools to deploy for (e.g., `["claude", "gemini", "codex"]`).
///
/// Deployment is non-fatal: errors are logged as warnings and collected, but the
/// function returns `Ok(())` as long as at least one skill deploys successfully
/// (or there's nothing to deploy).
pub fn deploy_skills(working_dir: &Path, project_path: &Path, tools: &[&str]) -> Result<()> {
    for (name, builtin_content) in BUILT_IN_SKILLS {
        let content = resolve_skill_content(name, builtin_content, project_path);
        let description = extract_description(&content).unwrap_or_default();

        for tool in tools {
            let result = match *tool {
                "claude" => deploy_claude_skill(working_dir, name, &content),
                "gemini" => deploy_gemini_skill(working_dir, name, &description, &content),
                "codex" => deploy_codex_skill(working_dir, name, &content),
                other => {
                    warn!(
                        tool = other,
                        skill = name,
                        "Unknown tool, skipping skill deployment"
                    );
                    continue;
                }
            };

            if let Err(e) = result {
                warn!(
                    tool = *tool,
                    skill = name,
                    error = %e,
                    "Failed to deploy skill (non-fatal)"
                );
            }
        }
    }

    Ok(())
}

/// Resolve skill content: use project override if present, otherwise built-in.
fn resolve_skill_content(name: &str, builtin: &str, project_path: &Path) -> String {
    let override_path = project_path
        .join(".operator")
        .join("skills")
        .join(format!("{name}.md"));

    if override_path.exists() {
        match fs::read_to_string(&override_path) {
            Ok(content) => return content,
            Err(e) => {
                warn!(
                    path = %override_path.display(),
                    error = %e,
                    "Failed to read skill override, using built-in"
                );
            }
        }
    }

    builtin.to_string()
}

/// Deploy a skill as a Claude command (markdown).
fn deploy_claude_skill(working_dir: &Path, name: &str, content: &str) -> Result<()> {
    let dir = working_dir
        .join(".claude")
        .join("commands")
        .join("operator");
    fs::create_dir_all(&dir)?;
    let file = dir.join(format!("{name}.md"));
    fs::write(&file, content)?;
    Ok(())
}

/// Deploy a skill as a Gemini command (TOML).
fn deploy_gemini_skill(
    working_dir: &Path,
    name: &str,
    description: &str,
    content: &str,
) -> Result<()> {
    let dir = working_dir
        .join(".gemini")
        .join("commands")
        .join("operator");
    fs::create_dir_all(&dir)?;
    let toml_content = skill_to_gemini_toml(description, content);
    let file = dir.join(format!("{name}.toml"));
    fs::write(&file, toml_content)?;
    Ok(())
}

/// Deploy a skill as a Codex skill (directory with SKILL.md).
fn deploy_codex_skill(working_dir: &Path, name: &str, content: &str) -> Result<()> {
    let dir = working_dir
        .join(".codex")
        .join("skills")
        .join(format!("operator-{name}"));
    fs::create_dir_all(&dir)?;
    let file = dir.join("SKILL.md");
    fs::write(&file, content)?;
    Ok(())
}

/// Strip YAML frontmatter from skill content, returning just the body.
fn strip_frontmatter(content: &str) -> &str {
    if let Some(rest) = content.strip_prefix("---") {
        if let Some(end) = rest.find("---") {
            let after = &rest[end + 3..];
            return after.trim_start_matches('\n');
        }
    }
    content
}

/// Convert skill content to Gemini TOML command format.
fn skill_to_gemini_toml(description: &str, skill_content: &str) -> String {
    let body = strip_frontmatter(skill_content);
    let escaped = body.replace('\\', "\\\\").replace("\"\"\"", "\\\"\\\"\\\"");
    format!(
        "description = \"{}\"\n\nprompt = \"\"\"\n{}\n\"\"\"\n",
        description.replace('"', "\\\""),
        escaped
    )
}

/// Extract the description field from YAML frontmatter.
fn extract_description(content: &str) -> Option<String> {
    if let Some(rest) = content.strip_prefix("---") {
        if let Some(end) = rest.find("---") {
            let frontmatter = &rest[..end];
            for line in frontmatter.lines() {
                if let Some(desc) = line.strip_prefix("description:") {
                    return Some(desc.trim().to_string());
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_strip_frontmatter_removes_yaml() {
        let content = "---\nname: test\ndescription: A test\n---\n# Body\nHello";
        let result = strip_frontmatter(content);
        assert_eq!(result, "# Body\nHello");
    }

    #[test]
    fn test_strip_frontmatter_no_frontmatter() {
        let content = "# Just a heading\nSome text";
        let result = strip_frontmatter(content);
        assert_eq!(result, content);
    }

    #[test]
    fn test_strip_frontmatter_unclosed() {
        let content = "---\nname: test\n# No closing delimiter\nSome text";
        let result = strip_frontmatter(content);
        assert_eq!(result, content);
    }

    #[test]
    fn test_skill_to_gemini_toml_basic() {
        let content =
            "---\nname: test\ndescription: A test skill\n---\n# Instructions\nDo the thing.";
        let result = skill_to_gemini_toml("A test skill", content);
        assert!(result.starts_with("description = \"A test skill\""));
        assert!(result.contains("prompt = \"\"\""));
        assert!(result.contains("# Instructions\nDo the thing."));
    }

    #[test]
    fn test_skill_to_gemini_toml_escapes_special_chars() {
        let content = "---\nname: test\n---\nUse a backslash \\ and triple quotes \"\"\" here.";
        let result = skill_to_gemini_toml("test", content);
        assert!(result.contains("\\\\"));
        assert!(result.contains("\\\"\\\"\\\""));
    }

    #[test]
    fn test_deploy_claude_skill_creates_file() {
        let tmp = TempDir::new().unwrap();
        let content = "---\nname: step\n---\n# Step\nDo things.";
        deploy_claude_skill(tmp.path(), "step", content).unwrap();

        let expected = tmp.path().join(".claude/commands/operator/step.md");
        assert!(expected.exists());
        assert_eq!(fs::read_to_string(expected).unwrap(), content);
    }

    #[test]
    fn test_deploy_gemini_skill_creates_toml() {
        let tmp = TempDir::new().unwrap();
        let content =
            "---\nname: step\ndescription: Signal step completion\n---\n# Step\nDo things.";
        deploy_gemini_skill(tmp.path(), "step", "Signal step completion", content).unwrap();

        let expected = tmp.path().join(".gemini/commands/operator/step.toml");
        assert!(expected.exists());
        let toml = fs::read_to_string(expected).unwrap();
        assert!(toml.starts_with("description = \"Signal step completion\""));
        assert!(toml.contains("# Step\nDo things."));
    }

    #[test]
    fn test_deploy_codex_skill_creates_dir() {
        let tmp = TempDir::new().unwrap();
        let content = "---\nname: step\n---\n# Step\nDo things.";
        deploy_codex_skill(tmp.path(), "step", content).unwrap();

        let expected = tmp.path().join(".codex/skills/operator-step/SKILL.md");
        assert!(expected.exists());
        assert_eq!(fs::read_to_string(expected).unwrap(), content);
    }

    #[test]
    fn test_resolve_uses_builtin_by_default() {
        let tmp = TempDir::new().unwrap();
        let builtin = "---\nname: step\n---\nBuilt-in content";
        let result = resolve_skill_content("step", builtin, tmp.path());
        assert_eq!(result, builtin);
    }

    #[test]
    fn test_resolve_uses_project_override() {
        let tmp = TempDir::new().unwrap();
        let override_dir = tmp.path().join(".operator").join("skills");
        fs::create_dir_all(&override_dir).unwrap();
        let override_content = "---\nname: step\n---\nCustom override content";
        fs::write(override_dir.join("step.md"), override_content).unwrap();

        let builtin = "---\nname: step\n---\nBuilt-in content";
        let result = resolve_skill_content("step", builtin, tmp.path());
        assert_eq!(result, override_content);
    }

    #[test]
    fn test_deploy_skills_all_tools() {
        let tmp = TempDir::new().unwrap();
        deploy_skills(tmp.path(), tmp.path(), &["claude", "gemini", "codex"]).unwrap();

        assert!(tmp
            .path()
            .join(".claude/commands/operator/step.md")
            .exists());
        assert!(tmp
            .path()
            .join(".gemini/commands/operator/step.toml")
            .exists());
        assert!(tmp
            .path()
            .join(".codex/skills/operator-step/SKILL.md")
            .exists());
    }

    #[test]
    fn test_deploy_skills_unknown_tool_nonfatal() {
        let tmp = TempDir::new().unwrap();
        let result = deploy_skills(tmp.path(), tmp.path(), &["unknown_tool"]);
        assert!(result.is_ok());
    }
}

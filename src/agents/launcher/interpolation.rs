//! Handlebars-based prompt interpolation engine
//!
//! Provides template variable substitution for agent prompts, supporting:
//! - Ticket frontmatter metadata
//! - Step information (step_count, step_names)
//! - Project context (project, cwd)
//! - Template files (acceptance_criteria, definition_of_done, definition_of_ready)

use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use handlebars::Handlebars;
use serde_json::{json, Value};

use crate::config::Config;
use crate::queue::Ticket;
use crate::templates::{schema::TemplateSchema, TemplateType};

/// Standard operator output instructions appended to all prompts.
/// This instructs agents to output a status block for progress tracking.
const OPERATOR_OUTPUT_INSTRUCTIONS: &str = r#"
---

## Status Reporting

When you complete your work or reach a stopping point, output a status block in this exact format:

```
---OPERATOR_STATUS---
status: complete | in_progress | blocked | failed
exit_signal: true | false
confidence: 0-100
files_modified: <count>
tests_status: passing | failing | skipped | not_run
error_count: <count>
tasks_completed: <count>
tasks_remaining: <count>
summary: <brief description of work done this iteration>
recommendation: <suggested next action or empty if done>
blockers: <comma-separated list if blocked, otherwise empty>
---END_OPERATOR_STATUS---
```

**Required fields:** status, exit_signal
**Set exit_signal: true** when your work on this step is complete
**Set exit_signal: false** if more work remains to be done
"#;

/// Handlebars-based prompt interpolator
pub struct PromptInterpolator {
    handlebars: Handlebars<'static>,
}

impl Default for PromptInterpolator {
    fn default() -> Self {
        Self::new()
    }
}

impl PromptInterpolator {
    /// Create a new prompt interpolator
    pub fn new() -> Self {
        let mut handlebars = Handlebars::new();
        // Don't escape HTML entities - prompts are plain text
        handlebars.set_strict_mode(false);
        Self { handlebars }
    }

    /// Build the full context for interpolation
    ///
    /// Includes ticket metadata, step info, project context, and template files.
    pub fn build_context(
        &self,
        config: &Config,
        ticket: &Ticket,
        project_path: &str,
    ) -> Result<Value> {
        self.build_context_with_previous(config, ticket, project_path, None, None)
    }

    /// Build context with previous step's output for piping
    ///
    /// Used when transitioning between steps to pass context forward.
    pub fn build_context_with_previous(
        &self,
        config: &Config,
        ticket: &Ticket,
        project_path: &str,
        previous_summary: Option<&str>,
        previous_recommendation: Option<&str>,
    ) -> Result<Value> {
        // Load the template schema for step information
        let schema = TemplateType::from_key(&ticket.ticket_type)
            .and_then(|tt| TemplateSchema::from_json(tt.schema()).ok());

        // Build step information
        let (step_count, step_names) = if let Some(ref schema) = schema {
            let count = schema.steps.len();
            let names: Vec<String> = schema.steps.iter().map(|s| s.name.clone()).collect();
            (count, names.join(", "))
        } else {
            (0, String::new())
        };

        // Load template files from operator templates directory
        let templates_dir = config.tickets_path().join("operator").join("templates");
        let acceptance_criteria =
            load_template_file(&templates_dir.join("ACCEPTANCE_CRITERIA.md")).unwrap_or_default();
        let definition_of_done =
            load_template_file(&templates_dir.join("DEFINITION_OF_DONE.md")).unwrap_or_default();
        let definition_of_ready =
            load_template_file(&templates_dir.join("DEFINITION_OF_READY.md")).unwrap_or_default();

        // Build the context object
        let mut context = json!({
            // Ticket metadata
            "id": ticket.id,
            "ticket_type": ticket.ticket_type,
            "summary": ticket.summary,
            "priority": ticket.priority,
            "status": ticket.status,
            "step": ticket.step,
            "content": ticket.content,
            "filename": ticket.filename,
            "filepath": ticket.filepath,
            "timestamp": ticket.timestamp,

            // Project context
            "project": ticket.project,
            "cwd": project_path,

            // Step information
            "step_count": step_count,
            "step_names": step_names,

            // Template files (pre-loaded)
            "acceptance_criteria": acceptance_criteria,
            "definition_of_done": definition_of_done,
            "definition_of_ready": definition_of_ready,

            // Ticket path for reference
            "ticket_path": format!("../.tickets/in-progress/{}", ticket.filename),

            // Operator output instructions for status reporting
            "operator_output_instructions": OPERATOR_OUTPUT_INSTRUCTIONS,

            // Previous step context (for multi-step piping)
            "previous_summary": previous_summary.unwrap_or(""),
            "previous_recommendation": previous_recommendation.unwrap_or(""),
        });

        // Add branch name if available
        if let Value::Object(ref mut map) = context {
            map.insert("branch".to_string(), json!(ticket.branch_name()));
        }

        Ok(context)
    }

    /// Render a template string with the given context
    pub fn render(&self, template: &str, context: &Value) -> Result<String> {
        self.handlebars
            .render_template(template, context)
            .context("Failed to render template")
    }

    /// Build and render a combined prompt for launching an agent
    ///
    /// Combines issuetype prompt, step prompt, ticket contents, and operator output instructions.
    /// Order: issuetype.prompt -> step.prompt -> ticket contents -> previous context -> output instructions
    pub fn build_launch_prompt(
        &self,
        config: &Config,
        ticket: &Ticket,
        project_path: &str,
    ) -> Result<String> {
        self.build_launch_prompt_with_context(config, ticket, project_path, None, None)
    }

    /// Build prompt with previous step context for multi-step piping
    pub fn build_launch_prompt_with_context(
        &self,
        config: &Config,
        ticket: &Ticket,
        project_path: &str,
        previous_summary: Option<&str>,
        previous_recommendation: Option<&str>,
    ) -> Result<String> {
        // Build the context for interpolation
        let context = self.build_context_with_previous(
            config,
            ticket,
            project_path,
            previous_summary,
            previous_recommendation,
        )?;

        // Load the template schema
        let schema = TemplateType::from_key(&ticket.ticket_type)
            .and_then(|tt| TemplateSchema::from_json(tt.schema()).ok());

        let mut parts = Vec::new();

        // 1. Add issuetype prompt (if exists)
        if let Some(ref schema) = schema {
            if let Some(ref prompt) = schema.prompt {
                let rendered = self.render(prompt, &context)?;
                if !rendered.trim().is_empty() {
                    parts.push(rendered);
                }
            }
        }

        // 2. Add step prompt (if exists)
        if let Some(ref schema) = schema {
            let step_name = if ticket.step.is_empty() {
                schema.steps.first().map(|s| s.name.as_str())
            } else {
                Some(ticket.step.as_str())
            };

            if let Some(step_name) = step_name {
                if let Some(step) = schema.get_step(step_name) {
                    let rendered = self.render(&step.prompt, &context)?;
                    if !rendered.trim().is_empty() {
                        parts.push(rendered);
                    }
                }
            }
        }

        // 3. Add ticket contents
        let ticket_path = config
            .tickets_path()
            .join("in-progress")
            .join(&ticket.filename);
        if ticket_path.exists() {
            if let Ok(contents) = fs::read_to_string(&ticket_path) {
                if !contents.trim().is_empty() {
                    parts.push(format!("## Ticket Contents\n\n{}", contents));
                }
            }
        }

        // 4. Add previous step context if available
        if let Some(summary) = previous_summary {
            if !summary.is_empty() {
                let mut context_section = String::from("## Previous Step Context\n\n");
                context_section.push_str(&format!("**Summary:** {}\n", summary));
                if let Some(rec) = previous_recommendation {
                    if !rec.is_empty() {
                        context_section.push_str(&format!("**Recommendation:** {}\n", rec));
                    }
                }
                parts.push(context_section);
            }
        }

        // 5. Always append operator output instructions
        parts.push(OPERATOR_OUTPUT_INSTRUCTIONS.trim().to_string());

        // Join with separators
        Ok(parts.join("\n\n---\n\n"))
    }
}

/// Load a template file, returning an empty string if it doesn't exist
fn load_template_file(path: &Path) -> Result<String> {
    if path.exists() {
        fs::read_to_string(path).context("Failed to read template file")
    } else {
        Ok(String::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_test_ticket() -> Ticket {
        Ticket {
            filename: "20241225-1200-FEAT-test-test.md".to_string(),
            filepath: "/tmp/tickets/queue/20241225-1200-FEAT-test-test.md".to_string(),
            timestamp: "20241225-1200".to_string(),
            ticket_type: "FEAT".to_string(),
            project: "test-project".to_string(),
            id: "FEAT-1234".to_string(),
            summary: "Add new feature".to_string(),
            priority: "P2-medium".to_string(),
            status: "in-progress".to_string(),
            step: "plan".to_string(),
            content: "# Feature Description\n\nThis is the feature content.".to_string(),
            sessions: std::collections::HashMap::new(),
            llm_task: crate::queue::LlmTask::default(),
            worktree_path: None,
            branch: None,
            external_id: None,
            external_url: None,
            external_provider: None,
        }
    }

    #[test]
    fn test_render_simple_template() {
        let interpolator = PromptInterpolator::new();
        let context = json!({
            "id": "FEAT-123",
            "project": "myproject"
        });

        let result = interpolator
            .render("Working on {{ id }} in {{ project }}", &context)
            .unwrap();
        assert_eq!(result, "Working on FEAT-123 in myproject");
    }

    #[test]
    fn test_render_missing_variable() {
        let interpolator = PromptInterpolator::new();
        let context = json!({
            "id": "FEAT-123"
        });

        // Missing variables should render as empty (strict mode disabled)
        let result = interpolator
            .render("ID: {{ id }}, Missing: {{ missing }}", &context)
            .unwrap();
        assert_eq!(result, "ID: FEAT-123, Missing: ");
    }

    #[test]
    fn test_build_context_includes_ticket_fields() {
        let temp_dir = TempDir::new().unwrap();
        let config = crate::config::Config {
            paths: crate::config::PathsConfig {
                tickets: temp_dir.path().to_string_lossy().to_string(),
                projects: temp_dir.path().to_string_lossy().to_string(),
                state: temp_dir.path().to_string_lossy().to_string(),
                worktrees: temp_dir
                    .path()
                    .join("worktrees")
                    .to_string_lossy()
                    .to_string(),
            },
            ..Default::default()
        };

        let ticket = make_test_ticket();
        let interpolator = PromptInterpolator::new();
        let context = interpolator
            .build_context(&config, &ticket, "/path/to/project")
            .unwrap();

        assert_eq!(context["id"], "FEAT-1234");
        assert_eq!(context["project"], "test-project");
        assert_eq!(context["cwd"], "/path/to/project");
        assert_eq!(context["summary"], "Add new feature");
        assert_eq!(context["step"], "plan");
    }

    #[test]
    fn test_build_context_step_info() {
        let temp_dir = TempDir::new().unwrap();
        let config = crate::config::Config {
            paths: crate::config::PathsConfig {
                tickets: temp_dir.path().to_string_lossy().to_string(),
                projects: temp_dir.path().to_string_lossy().to_string(),
                state: temp_dir.path().to_string_lossy().to_string(),
                worktrees: temp_dir
                    .path()
                    .join("worktrees")
                    .to_string_lossy()
                    .to_string(),
            },
            ..Default::default()
        };

        let ticket = make_test_ticket();
        let interpolator = PromptInterpolator::new();
        let context = interpolator
            .build_context(&config, &ticket, "/path/to/project")
            .unwrap();

        // FEAT template has 5 steps
        assert_eq!(context["step_count"], 5);
        assert!(context["step_names"].as_str().unwrap().contains("plan"));
    }

    #[test]
    fn test_load_template_file_nonexistent() {
        let result = load_template_file(Path::new("/nonexistent/file.md")).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_load_template_file_exists() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.md");
        fs::write(&file_path, "Test content").unwrap();

        let result = load_template_file(&file_path).unwrap();
        assert_eq!(result, "Test content");
    }

    #[test]
    fn test_build_context_includes_operator_output_instructions() {
        let temp_dir = TempDir::new().unwrap();
        let config = crate::config::Config {
            paths: crate::config::PathsConfig {
                tickets: temp_dir.path().to_string_lossy().to_string(),
                projects: temp_dir.path().to_string_lossy().to_string(),
                state: temp_dir.path().to_string_lossy().to_string(),
                worktrees: temp_dir
                    .path()
                    .join("worktrees")
                    .to_string_lossy()
                    .to_string(),
            },
            ..Default::default()
        };

        let ticket = make_test_ticket();
        let interpolator = PromptInterpolator::new();
        let context = interpolator
            .build_context(&config, &ticket, "/path/to/project")
            .unwrap();

        // Check operator_output_instructions is present
        let instructions = context["operator_output_instructions"].as_str().unwrap();
        assert!(instructions.contains("---OPERATOR_STATUS---"));
        assert!(instructions.contains("exit_signal"));
    }

    #[test]
    fn test_build_context_with_previous_step() {
        let temp_dir = TempDir::new().unwrap();
        let config = crate::config::Config {
            paths: crate::config::PathsConfig {
                tickets: temp_dir.path().to_string_lossy().to_string(),
                projects: temp_dir.path().to_string_lossy().to_string(),
                state: temp_dir.path().to_string_lossy().to_string(),
                worktrees: temp_dir
                    .path()
                    .join("worktrees")
                    .to_string_lossy()
                    .to_string(),
            },
            ..Default::default()
        };

        let ticket = make_test_ticket();
        let interpolator = PromptInterpolator::new();
        let context = interpolator
            .build_context_with_previous(
                &config,
                &ticket,
                "/path/to/project",
                Some("Implemented auth"),
                Some("Ready for tests"),
            )
            .unwrap();

        assert_eq!(context["previous_summary"], "Implemented auth");
        assert_eq!(context["previous_recommendation"], "Ready for tests");
    }

    #[test]
    fn test_render_operator_output_template() {
        let interpolator = PromptInterpolator::new();
        let context = json!({
            "operator_output_instructions": OPERATOR_OUTPUT_INSTRUCTIONS
        });

        let template = "Do the work.\n\n{{operator_output_instructions}}";
        let result = interpolator.render(template, &context).unwrap();

        assert!(result.contains("Do the work"));
        assert!(result.contains("---OPERATOR_STATUS---"));
    }
}

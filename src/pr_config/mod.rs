#![allow(dead_code)]

//! PR configuration module for project-specific pull request workflows

use anyhow::{Context, Result};
use handlebars::Handlebars;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::queue::Ticket;

/// PR configuration loaded from a project's `.operator/pr-config.toml`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrConfig {
    /// Branch naming pattern, e.g., "{type}/{id}-{slug}"
    /// Available variables: {type}, {id}, {slug}, {project}
    #[serde(default = "default_branch_pattern")]
    pub branch_pattern: String,

    /// PR title format, e.g., "{type}({project}): {summary}"
    /// Available variables: {type}, {id}, {slug}, {project}, {summary}
    #[serde(default = "default_title_format")]
    pub title_format: String,

    /// PR body template (handlebars template)
    /// If not set, uses body_template_file or a default template
    #[serde(default)]
    pub body_template: Option<String>,

    /// Path to body template file (relative to project root)
    #[serde(default)]
    pub body_template_file: Option<String>,

    /// Required checks before PR can be considered ready
    #[serde(default)]
    pub required_checks: Vec<String>,

    /// Merge strategy: "squash", "merge", "rebase", or "none" (manual)
    #[serde(default = "default_merge_strategy")]
    pub merge_strategy: String,

    /// Base branch for PRs (default: "main")
    #[serde(default = "default_base_branch")]
    pub base_branch: String,

    /// GitHub repository in format "owner/repo"
    #[serde(default)]
    pub github_repo: Option<String>,

    /// Labels to add to PRs
    #[serde(default)]
    pub labels: Vec<String>,

    /// Reviewers to request (usernames without @)
    #[serde(default)]
    pub reviewers: Vec<String>,

    /// Team reviewers to request (format: "org/team-name")
    #[serde(default)]
    pub team_reviewers: Vec<String>,

    /// Whether to auto-merge when all checks pass and approved
    #[serde(default)]
    pub auto_merge: bool,

    /// Draft PR by default
    #[serde(default)]
    pub draft_by_default: bool,
}

fn default_branch_pattern() -> String {
    "{type}/{id}-{slug}".to_string()
}

fn default_title_format() -> String {
    "{type}({project}): {summary}".to_string()
}

fn default_merge_strategy() -> String {
    "squash".to_string()
}

fn default_base_branch() -> String {
    "main".to_string()
}

impl Default for PrConfig {
    fn default() -> Self {
        Self {
            branch_pattern: default_branch_pattern(),
            title_format: default_title_format(),
            body_template: None,
            body_template_file: None,
            required_checks: Vec::new(),
            merge_strategy: default_merge_strategy(),
            base_branch: default_base_branch(),
            github_repo: None,
            labels: Vec::new(),
            reviewers: Vec::new(),
            team_reviewers: Vec::new(),
            auto_merge: false,
            draft_by_default: false,
        }
    }
}

impl PrConfig {
    /// Load PR config from a project directory
    /// Looks for `.operator/pr-config.toml` in the project root
    pub fn load_from_project(project_path: &Path) -> Result<Option<Self>> {
        let config_path = project_path.join(".operator").join("pr-config.toml");

        if !config_path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&config_path).context(format!(
            "Failed to read PR config from {}",
            config_path.display()
        ))?;

        let config: PrConfig = toml::from_str(&content).context(format!(
            "Failed to parse PR config from {}",
            config_path.display()
        ))?;

        Ok(Some(config))
    }

    /// Load PR config or return defaults if not found
    pub fn load_or_default(project_path: &Path) -> Self {
        Self::load_from_project(project_path)
            .ok()
            .flatten()
            .unwrap_or_default()
    }

    /// Generate branch name from ticket data
    pub fn generate_branch_name(&self, ticket: &Ticket) -> String {
        let type_lower = ticket.ticket_type.to_lowercase();
        let type_prefix = match ticket.ticket_type.as_str() {
            "FEAT" => "feature",
            "FIX" => "fix",
            "SPIKE" => "spike",
            "INV" => "investigation",
            "TASK" => "task",
            _ => &type_lower,
        };

        let slug = generate_slug(&ticket.summary);

        self.branch_pattern
            .replace("{type}", type_prefix)
            .replace("{id}", &ticket.id)
            .replace("{slug}", &slug)
            .replace("{project}", &ticket.project)
    }

    /// Generate PR title from ticket data
    pub fn generate_title(&self, ticket: &Ticket) -> String {
        let type_lower = ticket.ticket_type.to_lowercase();
        let slug = generate_slug(&ticket.summary);

        self.title_format
            .replace("{type}", &type_lower)
            .replace("{TYPE}", &ticket.ticket_type)
            .replace("{id}", &ticket.id)
            .replace("{slug}", &slug)
            .replace("{project}", &ticket.project)
            .replace("{summary}", &ticket.summary)
    }

    /// Generate PR body from ticket data using template
    pub fn generate_body(&self, ticket: &Ticket, project_path: &Path) -> Result<String> {
        // Determine which template to use
        let template = if let Some(ref inline) = self.body_template {
            inline.clone()
        } else if let Some(ref file) = self.body_template_file {
            let template_path = project_path.join(file);
            fs::read_to_string(&template_path).context(format!(
                "Failed to read body template from {}",
                template_path.display()
            ))?
        } else {
            default_body_template()
        };

        // Render with handlebars
        let mut hbs = Handlebars::new();
        hbs.set_strict_mode(false); // Allow missing variables

        let mut data = serde_json::Map::new();
        data.insert("id".to_string(), serde_json::json!(ticket.id));
        data.insert(
            "ticket_type".to_string(),
            serde_json::json!(ticket.ticket_type),
        );
        data.insert("project".to_string(), serde_json::json!(ticket.project));
        data.insert("summary".to_string(), serde_json::json!(ticket.summary));
        data.insert("priority".to_string(), serde_json::json!(ticket.priority));
        data.insert("step".to_string(), serde_json::json!(ticket.step));

        // Extract description from ticket content (after frontmatter)
        let description = extract_description(&ticket.content);
        data.insert("description".to_string(), serde_json::json!(description));

        // Extract context section if present
        let context = extract_section(&ticket.content, "Context");
        data.insert("context".to_string(), serde_json::json!(context));

        // Labels and reviewers
        data.insert("labels".to_string(), serde_json::json!(self.labels));
        data.insert("reviewers".to_string(), serde_json::json!(self.reviewers));

        let body = hbs
            .render_template(&template, &serde_json::Value::Object(data))
            .context("Failed to render PR body template")?;

        Ok(body)
    }

    /// Get the gh pr create command arguments for this config
    pub fn gh_create_args(&self, ticket: &Ticket, project_path: &Path) -> Result<Vec<String>> {
        let mut args = vec![
            "pr".to_string(),
            "create".to_string(),
            "--title".to_string(),
            self.generate_title(ticket),
            "--body".to_string(),
            self.generate_body(ticket, project_path)?,
            "--base".to_string(),
            self.base_branch.clone(),
        ];

        // Add labels
        for label in &self.labels {
            args.push("--label".to_string());
            args.push(label.clone());
        }

        // Add reviewers
        for reviewer in &self.reviewers {
            args.push("--reviewer".to_string());
            args.push(reviewer.clone());
        }

        // Draft flag
        if self.draft_by_default {
            args.push("--draft".to_string());
        }

        Ok(args)
    }
}

/// Generate a URL-safe slug from text
fn generate_slug(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .take(5)
        .collect::<Vec<_>>()
        .join("-")
}

/// Extract description from ticket content (after frontmatter, before sections)
fn extract_description(content: &str) -> String {
    // Skip frontmatter
    let body = if content.trim_start().starts_with("---") {
        if let Some(pos) = content.find("\n---\n") {
            &content[pos + 5..]
        } else if let Some(pos) = content.find("\n---") {
            &content[pos + 4..]
        } else {
            content
        }
    } else {
        content
    };

    // Get content until first ## heading
    if let Some(pos) = body.find("\n##") {
        body[..pos].trim().to_string()
    } else {
        body.trim().to_string()
    }
}

/// Extract a specific section from ticket content
fn extract_section(content: &str, section_name: &str) -> String {
    let header = format!("## {}", section_name);
    if let Some(start) = content.find(&header) {
        let after_header = &content[start + header.len()..];
        // Find end of section (next ## or end of content)
        let end = after_header.find("\n##").unwrap_or(after_header.len());
        after_header[..end].trim().to_string()
    } else {
        String::new()
    }
}

/// Default PR body template
fn default_body_template() -> String {
    r#"## Summary

{{ summary }}

{{#if description}}
## Description

{{ description }}
{{/if}}

{{#if context}}
## Context

{{ context }}
{{/if}}

## Ticket

- **ID**: {{ id }}
- **Type**: {{ ticket_type }}
- **Project**: {{ project }}
- **Priority**: {{ priority }}

---
*Automated PR by Operator*
"#
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_slug() {
        assert_eq!(
            generate_slug("Add leaderboard pagination"),
            "add-leaderboard-pagination"
        );
        assert_eq!(generate_slug("Fix bug #123"), "fix-bug-123");
        assert_eq!(
            generate_slug("A very long feature name that should be truncated"),
            "a-very-long-feature-name"
        );
    }

    #[test]
    fn test_default_config() {
        let config = PrConfig::default();
        assert_eq!(config.branch_pattern, "{type}/{id}-{slug}");
        assert_eq!(config.title_format, "{type}({project}): {summary}");
        assert_eq!(config.base_branch, "main");
        assert_eq!(config.merge_strategy, "squash");
    }

    #[test]
    fn test_extract_description() {
        let content = r#"---
id: FEAT-123
status: queued
---

This is the main description of the ticket.
It can span multiple lines.

## Context

Some context here.
"#;
        let desc = extract_description(content);
        assert!(desc.contains("This is the main description"));
        assert!(!desc.contains("Context"));
    }
}

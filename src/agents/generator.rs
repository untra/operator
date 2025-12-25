#![allow(dead_code)]

//! Agent and project ticket creators for operator-managed projects
//!
//! Creates TASK tickets for generating Claude Code agent files in a project's
//! `.claude/agents/` directory, and ASSESS tickets for Backstage catalog
//! assessment. These tickets can then be launched via the normal operator workflow.

use anyhow::{Context, Result};
use chrono::Local;
use std::fs;
use std::path::Path;

use crate::config::Config;
use crate::templates::schema::TemplateSchema;
use crate::templates::TemplateType;

/// Standard tools available to operator agents
pub const AGENT_TOOLS: &str = "Bash, Glob, Grep, Read, WebFetch, TodoWrite, WebSearch, BashOutput, KillShell, AskUserQuestion, Skill, SlashCommand, Edit, Write, NotebookEdit";

/// Result of creating agent tickets
#[derive(Debug, Clone)]
pub struct AgentTicketResult {
    /// Ticket IDs that were created
    pub created: Vec<String>,
    /// Agent keys that were skipped (already exist)
    pub skipped: Vec<String>,
    /// Errors encountered (key, error message)
    pub errors: Vec<(String, String)>,
}

/// Creates TASK tickets for missing operator agents
pub struct AgentTicketCreator;

impl AgentTicketCreator {
    /// Create TASK tickets for missing operator agents in a project
    pub fn create_agent_tickets(
        project_path: &Path,
        project_name: &str,
        config: &Config,
    ) -> Result<AgentTicketResult> {
        let agents_dir = project_path.join(".claude").join("agents");

        // Ensure queue directory exists
        let queue_dir = config.tickets_path().join("queue");
        fs::create_dir_all(&queue_dir).context("Failed to create queue directory")?;

        let mut result = AgentTicketResult {
            created: Vec::new(),
            skipped: Vec::new(),
            errors: Vec::new(),
        };

        for template_type in TemplateType::all() {
            // Skip Task - it doesn't get an operator agent
            if matches!(template_type, TemplateType::Task) {
                continue;
            }

            // Parse the template schema
            let schema = match TemplateSchema::from_json(template_type.schema()) {
                Ok(s) => s,
                Err(e) => {
                    result
                        .errors
                        .push((template_type.as_str().to_string(), e.to_string()));
                    continue;
                }
            };

            // Check if this template has an agent_prompt
            let Some(agent_prompt) = &schema.agent_prompt else {
                continue;
            };

            // Check if agent file already exists - skip if so
            let agent_filename = format!("{}-operator.md", schema.key.to_lowercase());
            if agents_dir.join(&agent_filename).exists() {
                result.skipped.push(schema.key.clone());
                continue;
            }

            // Create TASK ticket
            match Self::create_ticket(
                &queue_dir,
                project_name,
                &schema.key,
                &schema.name,
                agent_prompt,
            ) {
                Ok(ticket_id) => result.created.push(ticket_id),
                Err(e) => result.errors.push((schema.key.clone(), e.to_string())),
            }
        }

        Ok(result)
    }

    fn create_ticket(
        queue_dir: &Path,
        project_name: &str,
        key: &str,
        name: &str,
        agent_prompt: &str,
    ) -> Result<String> {
        let now = Local::now();
        let timestamp = now.format("%Y%m%d-%H%M").to_string();
        let date = now.format("%Y-%m-%d").to_string();

        // Unique ticket ID includes project and key
        // Format: TASK-{project}-{KEY}-{timestamp}
        let id = format!("TASK-{}-{}-{}", project_name, key, timestamp);

        // Filename: YYYYMMDD-HHMM-TASK-project-KEY-agent.md
        let filename = format!("{}-TASK-{}-{}-agent.md", timestamp, project_name, key);

        let key_lower = key.to_lowercase();

        // Build ticket content in standard markdown format
        let content = format!(
            r#"# {id}: Create {project_name} {name} operator agent

**ID**: {id}
**Project**: {project_name}
**Priority**: P2-medium
**Status**: queued
**Created**: {date}

## Summary

Create the {key_lower}-operator agent for {project_name} project.

## Context

{agent_prompt}

## Acceptance Criteria

- [ ] Agent file created at `.claude/agents/{key_lower}-operator.md`
- [ ] Agent has proper frontmatter (name, description, tools)
- [ ] Agent prompt follows project patterns from CLAUDE.md
"#,
            id = id,
            project_name = project_name,
            name = name,
            date = date,
            key_lower = key_lower,
            agent_prompt = agent_prompt,
        );

        let ticket_path = queue_dir.join(&filename);
        fs::write(&ticket_path, content).context(format!("Failed to write ticket {}", filename))?;

        Ok(id)
    }
}

/// Result of creating an assessment ticket
#[derive(Debug, Clone)]
pub struct AssessTicketResult {
    /// Ticket ID that was created
    pub ticket_id: String,
    /// Project that was assessed
    pub project: String,
}

/// Creates ASSESS tickets for Backstage catalog assessment
pub struct AssessTicketCreator;

impl AssessTicketCreator {
    /// Create an ASSESS ticket for a project
    pub fn create_assess_ticket(
        project_path: &Path,
        project_name: &str,
        config: &Config,
    ) -> Result<AssessTicketResult> {
        // Ensure queue directory exists
        let queue_dir = config.tickets_path().join("queue");
        fs::create_dir_all(&queue_dir).context("Failed to create queue directory")?;

        let now = Local::now();
        let timestamp = now.format("%Y%m%d-%H%M").to_string();
        let datetime = now.format("%Y-%m-%d %H:%M").to_string();

        // Unique ticket ID
        let id = format!("ASSESS-{}-{}", project_name, timestamp);

        // Filename: YYYYMMDD-HHMM-ASSESS-project.md
        let filename = format!("{}-ASSESS-{}.md", timestamp, project_name);

        // Check if catalog-info.yaml already exists
        let catalog_exists = project_path.join("catalog-info.yaml").exists();
        let action = if catalog_exists { "Update" } else { "Generate" };

        // Build ticket content using the ASSESS template format
        let content = format!(
            r#"---
id: {id}
step: analyze
project: {project}
status: queued
created: {datetime}
---

# Assessment: {action} catalog-info.yaml for {project}

## Project
{project}
"#,
            id = id,
            project = project_name,
            datetime = datetime,
            action = action,
        );

        let ticket_path = queue_dir.join(&filename);
        fs::write(&ticket_path, content).context(format!("Failed to write ticket {}", filename))?;

        Ok(AssessTicketResult {
            ticket_id: id,
            project: project_name.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_agent_tools_list() {
        assert!(AGENT_TOOLS.contains("Read"));
        assert!(AGENT_TOOLS.contains("Write"));
        assert!(AGENT_TOOLS.contains("Bash"));
    }

    #[test]
    fn test_ticket_creation() {
        let temp_dir = TempDir::new().unwrap();
        let queue_dir = temp_dir.path().join("queue");
        fs::create_dir_all(&queue_dir).unwrap();

        let result = AgentTicketCreator::create_ticket(
            &queue_dir,
            "myproject",
            "FEAT",
            "Feature",
            "Test prompt for feature agent",
        );

        assert!(result.is_ok());
        let ticket_id = result.unwrap();
        assert!(ticket_id.starts_with("TASK-"));

        // Check that a file was created
        let files: Vec<_> = fs::read_dir(&queue_dir).unwrap().collect();
        assert_eq!(files.len(), 1);

        // Verify content
        let file_path = files[0].as_ref().unwrap().path();
        let content = fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("myproject"));
        assert!(content.contains("feat-operator"));
        assert!(content.contains("Test prompt for feature agent"));
    }
}

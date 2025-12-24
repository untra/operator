#![allow(dead_code)]
#![allow(unused_variables)]

//! Step session creation for launching Claude agents per step

use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use uuid::Uuid;

use crate::config::Config;
use crate::pr_config::PrConfig;
use crate::queue::Ticket;
use crate::steps::StepManager;
use crate::templates::schema::StepSchema;

/// Creates step-specific Claude sessions
pub struct StepSession {
    config: Config,
    step_manager: StepManager,
}

impl StepSession {
    /// Create a new step session helper
    pub fn new(config: &Config) -> Self {
        Self {
            config: config.clone(),
            step_manager: StepManager::new(config),
        }
    }

    /// Generate the Claude command arguments for a specific step
    /// Returns arguments for: claude -p <prompt> --session-id <uuid>
    /// The prompt includes the initial message after a `---` separator
    pub fn generate_claude_args(
        &self,
        ticket: &mut Ticket,
        step: &StepSchema,
        pr_config: Option<&PrConfig>,
        project_path: &str,
    ) -> Result<(String, String)> {
        let base_prompt = self.generate_prompt(ticket, step, pr_config)?;
        let session_uuid = self.generate_session_uuid(ticket, step)?;
        let initial_message = self.generate_initial_message(ticket, step);

        // Combine prompt and message with separator
        let full_prompt = format!("{}\n---\n{}", base_prompt, initial_message);

        Ok((full_prompt, session_uuid))
    }

    /// Generate the agent system prompt for a step
    fn generate_prompt(
        &self,
        ticket: &Ticket,
        step: &StepSchema,
        pr_config: Option<&PrConfig>,
    ) -> Result<String> {
        let step_prompt = self
            .step_manager
            .get_step_prompt(ticket, pr_config)
            .context("Failed to get step prompt")?;

        // Build a comprehensive prompt with context
        let allowed_tools = step.allowed_tools.join(", ");

        let prompt = format!(
            r#"You are working on step "{}" ({}) for ticket {}.

## Ticket Details
- **ID**: {}
- **Type**: {}
- **Project**: {}
- **Summary**: {}
- **Priority**: {}

## Step Instructions
{}

## Allowed Tools
You may use these tools for this step: {}

## Guidelines
- Focus only on completing this specific step
- Do not proceed to the next step; signal completion when done
- If you encounter blockers, document them clearly
- Follow existing project patterns and conventions

When you have completed this step, clearly indicate "STEP COMPLETE" in your final message."#,
            step.name,
            step.display_name(),
            ticket.id,
            ticket.id,
            ticket.ticket_type,
            ticket.project,
            ticket.summary,
            ticket.priority,
            step_prompt,
            allowed_tools,
        );

        Ok(prompt)
    }

    /// Generate a unique session UUID for this step and store it in the ticket
    fn generate_session_uuid(&self, ticket: &mut Ticket, step: &StepSchema) -> Result<String> {
        let session_uuid = Uuid::new_v4().to_string();

        // Store the session UUID in the ticket frontmatter
        ticket.set_session_id(&step.name, &session_uuid)?;

        Ok(session_uuid)
    }

    /// Generate the initial message to send to the agent
    fn generate_initial_message(&self, ticket: &Ticket, step: &StepSchema) -> String {
        format!(
            "Begin working on the \"{}\" step for ticket {}. Read the ticket file and start working on the task.",
            step.display_name(),
            ticket.id
        )
    }

    /// Get the current step or first step if not set
    pub fn get_effective_step(&self, ticket: &Ticket) -> Option<StepSchema> {
        if ticket.step.is_empty() {
            self.step_manager.first_step(&ticket.ticket_type)
        } else {
            self.step_manager.current_step(ticket)
        }
    }

    /// Generate the tmux session command
    /// Returns the full command to launch Claude in a tmux session
    pub fn generate_tmux_command(
        &self,
        ticket: &mut Ticket,
        step: &StepSchema,
        pr_config: Option<&PrConfig>,
        project_path: &str,
    ) -> Result<String> {
        let (prompt, session_uuid) =
            self.generate_claude_args(ticket, step, pr_config, project_path)?;

        // Write prompt to file (avoids newline issues with tmux send-keys)
        let prompt_file = self.write_prompt_file(&session_uuid, &prompt)?;

        // Get the configured LLM tool
        let tool = self
            .config
            .llm_tools
            .detected
            .first()
            .context("No LLM tool detected. Install claude, gemini, or codex CLI.")?;

        // Get model from first available provider (default to "sonnet" if none)
        let model = self
            .config
            .llm_tools
            .providers
            .first()
            .map(|p| p.model.as_str())
            .unwrap_or("sonnet");

        // Build model flag
        let model_flag = format!("--model {} ", model);

        // Build command from template
        let llm_cmd = tool
            .command_template
            .replace("{{model_flag}}", &model_flag)
            .replace("{{model}}", model)
            .replace("{{session_id}}", &session_uuid)
            .replace("{{prompt_file}}", &prompt_file.display().to_string());

        let command = format!("cd '{}' && {}", project_path, llm_cmd);

        Ok(command)
    }

    /// Write a prompt to a file and return the path
    /// Prompts are stored in .tickets/operator/prompts/{session_uuid}.txt
    fn write_prompt_file(&self, session_uuid: &str, prompt: &str) -> Result<PathBuf> {
        let prompts_dir = self.config.tickets_path().join("operator/prompts");
        fs::create_dir_all(&prompts_dir).context("Failed to create prompts directory")?;

        let prompt_file = prompts_dir.join(format!("{}.txt", session_uuid));
        fs::write(&prompt_file, prompt).context("Failed to write prompt file")?;

        Ok(prompt_file)
    }

    /// Get step progress info for display
    pub fn get_progress_info(&self, ticket: &Ticket) -> StepProgressInfo {
        let (current_idx, total, step_names) = self.step_manager.get_progress(ticket);
        let current_step = self.step_manager.current_step(ticket);

        StepProgressInfo {
            current_index: current_idx,
            total_steps: total,
            step_names,
            current_step_name: current_step.as_ref().map(|s| s.name.clone()),
            current_step_display: current_step.as_ref().map(|s| s.display_name().to_string()),
            requires_review: current_step
                .as_ref()
                .map(|s| s.requires_review)
                .unwrap_or(false),
            is_final: current_step
                .as_ref()
                .map(|s| s.next_step.is_none())
                .unwrap_or(true),
        }
    }
}

/// Step progress information for display
#[derive(Debug, Clone)]
pub struct StepProgressInfo {
    pub current_index: usize,
    pub total_steps: usize,
    pub step_names: Vec<String>,
    pub current_step_name: Option<String>,
    pub current_step_display: Option<String>,
    pub requires_review: bool,
    pub is_final: bool,
}

impl StepProgressInfo {
    /// Format as a progress string like "[plan] > implement > test > pr"
    pub fn format_progress(&self) -> String {
        self.step_names
            .iter()
            .enumerate()
            .map(|(i, name)| {
                if i == self.current_index {
                    format!("[{}]", name)
                } else {
                    name.clone()
                }
            })
            .collect::<Vec<_>>()
            .join(" > ")
    }

    /// Format as percentage complete
    pub fn percentage_complete(&self) -> u8 {
        if self.total_steps == 0 {
            100
        } else {
            ((self.current_index as f32 / self.total_steps as f32) * 100.0) as u8
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_ticket() -> Ticket {
        Ticket {
            filename: "20241221-1430-FEAT-gamesvc-test.md".to_string(),
            filepath: "/test/path".to_string(),
            timestamp: "20241221-1430".to_string(),
            ticket_type: "FEAT".to_string(),
            project: "gamesvc".to_string(),
            id: "FEAT-1234".to_string(),
            summary: "Test feature".to_string(),
            priority: "P2-medium".to_string(),
            status: "queued".to_string(),
            step: "plan".to_string(),
            content: "Test content".to_string(),
            sessions: std::collections::HashMap::new(),
        }
    }

    fn make_test_step() -> StepSchema {
        StepSchema {
            name: "plan".to_string(),
            display_name: Some("Planning".to_string()),
            outputs: vec![],
            prompt: "Create a plan".to_string(),
            allowed_tools: vec!["Read".to_string(), "Glob".to_string()],
            requires_review: false,
            on_reject: None,
            next_step: Some("implement".to_string()),
        }
    }

    #[test]
    fn test_generate_session_uuid() {
        let config = Config::default();
        let session = StepSession::new(&config);

        // Create a temp file for the ticket
        let temp_dir = tempfile::tempdir().unwrap();
        let ticket_path = temp_dir.path().join("20241221-1430-FEAT-gamesvc-test.md");
        let content = r#"---
id: FEAT-1234
status: queued
step: plan
---

# Feature: Test feature
"#;
        std::fs::write(&ticket_path, content).unwrap();

        let mut ticket = crate::queue::Ticket::from_file(&ticket_path).unwrap();
        let step = make_test_step();

        let uuid = session.generate_session_uuid(&mut ticket, &step).unwrap();

        // UUID should be valid format (36 chars with hyphens)
        assert_eq!(uuid.len(), 36);
        assert!(uuid.contains('-'));

        // Session ID should be stored in ticket
        assert_eq!(ticket.get_session_id("plan").unwrap(), &uuid);
    }

    #[test]
    fn test_progress_info() {
        let config = Config::default();
        let session = StepSession::new(&config);
        let ticket = make_test_ticket();

        let info = session.get_progress_info(&ticket);
        assert_eq!(info.current_index, 0);
        assert!(info.total_steps > 0);
        assert_eq!(info.current_step_name, Some("plan".to_string()));
    }
}

//! Prompt generation and file handling for agent launches

use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use uuid::Uuid;

use crate::config::Config;
use crate::queue::Ticket;
use crate::templates::{schema::TemplateSchema, TemplateType};

/// Generate the initial prompt for a ticket based on its type
pub fn generate_prompt(config: &Config, ticket: &Ticket) -> String {
    let ticket_path = config
        .tickets_path()
        .join("in-progress")
        .join(&ticket.filename);

    match ticket.ticket_type.as_str() {
        "FEAT" | "FIX" => {
            format!(
                r#"I'm starting work on ticket {}-{}.

Please read the ticket at: {}

Then:
1. Create a feature branch: `git checkout -b {}`
2. Implement the requirements from the ticket
3. Run all validation steps (tests, linting)
4. Create a single, focused commit with message format:
   ```
   {}({}): <summary>

   <description>

   Ticket: {}
   ```
5. Create a pull request
6. Move the ticket to completed: `mv {} {}`

Let me know when you've read the ticket and are ready to begin."#,
                ticket.ticket_type,
                ticket.id,
                ticket_path.display(),
                ticket.branch_name(),
                ticket.ticket_type.to_lowercase(),
                ticket.project,
                ticket.id,
                ticket_path.display(),
                config
                    .tickets_path()
                    .join("completed")
                    .join(&ticket.filename)
                    .display(),
            )
        }
        "SPIKE" => {
            format!(
                r#"Starting spike session for {}.

Please read the spike ticket at: {}

This is a paired research session. I'll be here to:
- Answer questions about the codebase
- Discuss findings with you
- Help you explore and investigate
- Document our discoveries in the ticket

The output of this spike will be new feature/fix tickets based on what we learn.

Let me know when you've read the ticket and what you'd like to explore first."#,
                ticket.id,
                ticket_path.display(),
            )
        }
        "INV" => {
            format!(
                r#"URGENT: Investigation needed for {}.

Please read the investigation ticket at: {}

This is a priority incident. Let's:
1. Understand the observed failure
2. Gather evidence (logs, errors, metrics)
3. Form and test hypotheses
4. Identify root cause
5. Recommend immediate mitigation
6. Generate fix tickets

I'm here to help investigate. What information do you have about the incident so far?"#,
                ticket.id,
                ticket_path.display(),
            )
        }
        "TASK" => {
            format!(
                r#"Starting task: {}

Please read the task ticket at: {}

Follow the instructions in the ticket's Context section to complete this task.
When done, move the ticket to completed."#,
                ticket.id,
                ticket_path.display()
            )
        }
        _ => {
            format!(
                "Starting work on ticket: {}\n\nPlease read: {}",
                ticket.id,
                ticket_path.display()
            )
        }
    }
}

/// Get the agent_prompt from a template if it exists
pub fn get_agent_prompt(ticket_type: &str) -> Option<String> {
    TemplateType::from_key(ticket_type)
        .and_then(|tt| TemplateSchema::from_json(tt.schema()).ok())
        .and_then(|schema| schema.agent_prompt)
}

/// Get the top-level prompt from a template if it exists
pub fn get_template_prompt(ticket_type: &str) -> Option<String> {
    TemplateType::from_key(ticket_type)
        .and_then(|tt| TemplateSchema::from_json(tt.schema()).ok())
        .and_then(|schema| schema.prompt)
}

/// Generate a UUID for the claude --session-id flag
pub fn generate_session_uuid() -> String {
    Uuid::new_v4().to_string()
}

/// Write a prompt to a file and return the path
/// Prompts are stored in .tickets/operator/prompts/{session_uuid}.txt
pub fn write_prompt_file(config: &Config, session_uuid: &str, prompt: &str) -> Result<PathBuf> {
    let prompts_dir = config.tickets_path().join("operator/prompts");
    fs::create_dir_all(&prompts_dir).context("Failed to create prompts directory")?;

    let prompt_file = prompts_dir.join(format!("{}.txt", session_uuid));
    fs::write(&prompt_file, prompt).context("Failed to write prompt file")?;

    Ok(prompt_file)
}

/// Escape a string for safe use in shell command
pub fn shell_escape(s: &str) -> String {
    // Use single quotes and escape any single quotes within
    let escaped = s.replace('\'', "'\"'\"'");
    format!("'{}'", escaped)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shell_escape_simple() {
        assert_eq!(shell_escape("hello"), "'hello'");
    }

    #[test]
    fn test_shell_escape_with_quotes() {
        assert_eq!(shell_escape("it's"), "'it'\"'\"'s'");
    }

    #[test]
    fn test_shell_escape_multiline() {
        let input = "line1\nline2";
        let escaped = shell_escape(input);
        assert!(escaped.starts_with('\''));
        assert!(escaped.ends_with('\''));
        assert!(escaped.contains('\n'));
    }

    #[test]
    fn test_generate_session_uuid_is_valid() {
        let uuid_str = generate_session_uuid();

        // Should be a valid UUID format (36 chars with hyphens)
        assert_eq!(uuid_str.len(), 36);
        assert!(uuid_str.contains('-'));

        // Should parse as a valid UUID
        let parsed = Uuid::parse_str(&uuid_str);
        assert!(parsed.is_ok());
    }

    #[test]
    fn test_generate_session_uuid_is_unique() {
        let uuid1 = generate_session_uuid();
        let uuid2 = generate_session_uuid();
        let uuid3 = generate_session_uuid();

        // Each UUID should be unique
        assert_ne!(uuid1, uuid2);
        assert_ne!(uuid2, uuid3);
        assert_ne!(uuid1, uuid3);
    }
}

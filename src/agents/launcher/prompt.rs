//! Prompt generation and file handling for agent launches

use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use uuid::Uuid;

use crate::config::Config;
use crate::queue::Ticket;
use crate::templates::{schema::TemplateSchema, TemplateType};

/// Environment variables injected into operator-spawned agent command scripts
/// for branding (status line, pane title, UI deep-links).
#[derive(Debug, Clone, Default)]
pub struct OperatorEnvVars {
    pub agent_id: String,
    pub ticket_id: String,
    pub project: String,
    pub step: String,
    pub ui_url: String,
    pub ui_port: u16,
}

impl OperatorEnvVars {
    /// Render shell `export` lines for all operator env vars.
    pub fn to_export_block(&self) -> String {
        format!(
            "export OPERATOR_AGENT_ID={}\nexport OPERATOR_TICKET_ID={}\nexport OPERATOR_PROJECT={}\nexport OPERATOR_STEP={}\nexport OPERATOR_UI_URL={}\nexport OPERATOR_UI_PORT={}\n",
            shell_escape(&self.agent_id),
            shell_escape(&self.ticket_id),
            shell_escape(&self.project),
            shell_escape(&self.step),
            shell_escape(&self.ui_url),
            self.ui_port,
        )
    }

    /// Render an OSC 2 escape sequence to set the terminal pane title.
    pub fn to_pane_title_line(&self) -> String {
        format!(
            "printf '\\033]2;[OPR8R] %s | %s\\033\\\\' {} {}\n",
            shell_escape(&self.ticket_id),
            shell_escape(&self.project),
        )
    }
}

/// Generate the initial prompt for a ticket based on its type
pub fn generate_prompt(config: &Config, ticket: &Ticket) -> String {
    let ticket_path = config
        .tickets_path()
        .join("in-progress")
        .join(&ticket.filename);

    match ticket.ticket_type.as_str() {
        "FEAT" | "FIX" => {
            format!(
                r"I'm starting work on ticket {}-{}.

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

Let me know when you've read the ticket and are ready to begin.",
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
                r"Starting spike session for {}.

Please read the spike ticket at: {}

This is a paired research session. I'll be here to:
- Answer questions about the codebase
- Discuss findings with you
- Help you explore and investigate
- Document our discoveries in the ticket

The output of this spike will be new feature/fix tickets based on what we learn.

Let me know when you've read the ticket and what you'd like to explore first.",
                ticket.id,
                ticket_path.display(),
            )
        }
        "INV" => {
            format!(
                r"URGENT: Investigation needed for {}.

Please read the investigation ticket at: {}

This is a priority incident. Let's:
1. Understand the observed failure
2. Gather evidence (logs, errors, metrics)
3. Form and test hypotheses
4. Identify root cause
5. Recommend immediate mitigation
6. Generate fix tickets

I'm here to help investigate. What information do you have about the incident so far?",
                ticket.id,
                ticket_path.display(),
            )
        }
        "TASK" => {
            format!(
                r"Starting task: {}

Please read the task ticket at: {}

Follow the instructions in the ticket's Context section to complete this task.
When done, move the ticket to completed.",
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

/// Get the `agent_prompt` from a template if it exists
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
/// Prompts are stored in .`tickets/operator/prompts/{session_uuid}.txt`
pub fn write_prompt_file(config: &Config, session_uuid: &str, prompt: &str) -> Result<PathBuf> {
    let prompts_dir = config.tickets_path().join("operator/prompts");
    fs::create_dir_all(&prompts_dir).context("Failed to create prompts directory")?;

    let prompt_file = prompts_dir.join(format!("{session_uuid}.txt"));
    fs::write(&prompt_file, prompt).context("Failed to write prompt file")?;

    Ok(prompt_file)
}

/// Write a shell command to an executable script file and return the path
/// Commands are stored in .`tickets/operator/commands/{session_uuid}.sh`
///
/// This solves issues with long commands and special characters when using tmux send-keys.
/// Instead of pasting complex commands directly, we write them to a script and execute that.
pub fn write_command_file(
    config: &Config,
    session_uuid: &str,
    project_path: &str,
    llm_command: &str,
    operator_env: Option<&OperatorEnvVars>,
) -> Result<PathBuf> {
    let commands_dir = config.tickets_path().join("operator/commands");
    fs::create_dir_all(&commands_dir).context("Failed to create commands directory")?;

    let command_file = commands_dir.join(format!("{session_uuid}.sh"));

    // Build script content with shebang, optional env vars, cd, and exec
    let env_block = operator_env
        .map(OperatorEnvVars::to_export_block)
        .unwrap_or_default();

    let pane_title = operator_env
        .map(OperatorEnvVars::to_pane_title_line)
        .unwrap_or_default();

    let script_content = format!(
        "#!/bin/bash\n{env_block}{pane_title}cd {}\nexec {}\n",
        shell_escape(project_path),
        llm_command
    );

    fs::write(&command_file, &script_content).context("Failed to write command file")?;

    // Make the file executable on Unix systems
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let permissions = fs::Permissions::from_mode(0o755);
        fs::set_permissions(&command_file, permissions)
            .context("Failed to set command file permissions")?;
    }

    Ok(command_file)
}

/// Escape a string for safe use in shell command
pub fn shell_escape(s: &str) -> String {
    // Use single quotes and escape any single quotes within
    let escaped = s.replace('\'', "'\"'\"'");
    format!("'{escaped}'")
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

    fn make_test_config_with_tickets_path(tickets_path: &std::path::Path) -> Config {
        use crate::config::PathsConfig;

        Config {
            paths: PathsConfig {
                tickets: tickets_path.to_string_lossy().to_string(),
                projects: tickets_path.parent().unwrap().to_string_lossy().to_string(),
                state: tickets_path.join("operator").to_string_lossy().to_string(),
                worktrees: tickets_path
                    .join("operator/worktrees")
                    .to_string_lossy()
                    .to_string(),
            },
            ..Default::default()
        }
    }

    #[test]
    fn test_write_command_file_creates_file_with_correct_content() {
        use tempfile::tempdir;

        // Create a temp directory to act as our tickets path
        let temp_dir = tempdir().unwrap();
        let config = make_test_config_with_tickets_path(temp_dir.path());

        let session_uuid = "test-uuid-1234";
        let project_path = "/path/to/project";
        let llm_command = "claude --session-id abc123 --print-prompt-path /tmp/prompt.txt";

        let result = write_command_file(&config, session_uuid, project_path, llm_command, None);
        assert!(result.is_ok());

        let command_file = result.unwrap();
        assert!(command_file.exists());
        assert_eq!(command_file.file_name().unwrap(), "test-uuid-1234.sh");

        let content = std::fs::read_to_string(&command_file).unwrap();
        assert!(content.starts_with("#!/bin/bash\n"));
        assert!(content.contains("cd '/path/to/project'"));
        assert!(content.contains("exec claude --session-id abc123"));
    }

    #[test]
    fn test_write_command_file_handles_spaces_in_path() {
        use tempfile::tempdir;

        let temp_dir = tempdir().unwrap();
        let config = make_test_config_with_tickets_path(temp_dir.path());

        let session_uuid = "test-uuid-spaces";
        let project_path = "/path/with spaces/to/project";
        let llm_command = "claude --arg value";

        let result = write_command_file(&config, session_uuid, project_path, llm_command, None);
        assert!(result.is_ok());

        let content = std::fs::read_to_string(result.unwrap()).unwrap();
        // Path with spaces should be properly escaped with single quotes
        assert!(content.contains("cd '/path/with spaces/to/project'"));
    }

    #[test]
    fn test_write_command_file_handles_special_chars_in_path() {
        use tempfile::tempdir;

        let temp_dir = tempdir().unwrap();
        let config = make_test_config_with_tickets_path(temp_dir.path());

        let session_uuid = "test-uuid-special";
        let project_path = "/path/with'quotes/and$dollar";
        let llm_command = "claude --arg value";

        let result = write_command_file(&config, session_uuid, project_path, llm_command, None);
        assert!(result.is_ok());

        let content = std::fs::read_to_string(result.unwrap()).unwrap();
        // Single quotes in path should be escaped properly
        assert!(content.contains("cd '/path/with'\"'\"'quotes/and$dollar'"));
    }

    #[test]
    fn test_write_command_file_preserves_complex_commands() {
        use tempfile::tempdir;

        let temp_dir = tempdir().unwrap();
        let config = make_test_config_with_tickets_path(temp_dir.path());

        let session_uuid = "test-uuid-complex";
        let project_path = "/project";
        let llm_command = r#"claude --session-id abc --print-prompt-path /tmp/file.txt --add-dir "/dir with spaces" --model sonnet"#;

        let result = write_command_file(&config, session_uuid, project_path, llm_command, None);
        assert!(result.is_ok());

        let content = std::fs::read_to_string(result.unwrap()).unwrap();
        // The full command should be preserved exactly
        assert!(content.contains(llm_command));
    }

    #[cfg(unix)]
    #[test]
    fn test_write_command_file_is_executable() {
        use std::os::unix::fs::PermissionsExt;
        use tempfile::tempdir;

        let temp_dir = tempdir().unwrap();
        let config = make_test_config_with_tickets_path(temp_dir.path());

        let session_uuid = "test-uuid-executable";
        let project_path = "/project";
        let llm_command = "claude --arg value";

        let result = write_command_file(&config, session_uuid, project_path, llm_command, None);
        assert!(result.is_ok());

        let command_file = result.unwrap();
        let metadata = std::fs::metadata(&command_file).unwrap();
        let permissions = metadata.permissions();

        // Check that the file is executable (0o755 = rwxr-xr-x)
        assert_eq!(permissions.mode() & 0o777, 0o755);
    }

    #[test]
    fn test_operator_env_vars_to_export_block() {
        let env = OperatorEnvVars {
            agent_id: "abc-123".to_string(),
            ticket_id: "FEAT-042".to_string(),
            project: "gamesvc".to_string(),
            step: "implement".to_string(),
            ui_url: "http://localhost:7007/#/agent/abc-123".to_string(),
            ui_port: 7007,
        };
        let block = env.to_export_block();
        assert!(block.contains("export OPERATOR_AGENT_ID='abc-123'"));
        assert!(block.contains("export OPERATOR_TICKET_ID='FEAT-042'"));
        assert!(block.contains("export OPERATOR_PROJECT='gamesvc'"));
        assert!(block.contains("export OPERATOR_STEP='implement'"));
        assert!(block.contains("export OPERATOR_UI_URL='http://localhost:7007/#/agent/abc-123'"));
        assert!(block.contains("export OPERATOR_UI_PORT=7007"));
    }

    #[test]
    fn test_operator_env_vars_to_pane_title_line() {
        let env = OperatorEnvVars {
            agent_id: "abc-123".to_string(),
            ticket_id: "FEAT-042".to_string(),
            project: "gamesvc".to_string(),
            step: "implement".to_string(),
            ui_url: "http://localhost:7007/#/agent/abc-123".to_string(),
            ui_port: 7007,
        };
        let line = env.to_pane_title_line();
        assert!(line.contains("\\033]2;"));
        assert!(line.contains("FEAT-042"));
        assert!(line.contains("gamesvc"));
    }

    #[test]
    fn test_write_command_file_with_operator_env() {
        use tempfile::tempdir;

        let temp_dir = tempdir().unwrap();
        let config = make_test_config_with_tickets_path(temp_dir.path());

        let env = OperatorEnvVars {
            agent_id: "test-agent-id".to_string(),
            ticket_id: "FEAT-001".to_string(),
            project: "myproject".to_string(),
            step: "plan".to_string(),
            ui_url: "http://localhost:7007/#/agent/test-agent-id".to_string(),
            ui_port: 7007,
        };

        let result = write_command_file(
            &config,
            "test-uuid-env",
            "/path/to/project",
            "claude --session-id abc123",
            Some(&env),
        );
        assert!(result.is_ok());

        let content = std::fs::read_to_string(result.unwrap()).unwrap();
        assert!(content.contains("export OPERATOR_AGENT_ID='test-agent-id'"));
        assert!(content.contains("export OPERATOR_TICKET_ID='FEAT-001'"));
        assert!(content.contains("export OPERATOR_PROJECT='myproject'"));
        assert!(content.contains("\\033]2;"));
        assert!(content.contains("cd '/path/to/project'"));
        assert!(content.contains("exec claude --session-id abc123"));
    }

    #[test]
    fn test_write_command_file_without_operator_env_unchanged() {
        use tempfile::tempdir;

        let temp_dir = tempdir().unwrap();
        let config = make_test_config_with_tickets_path(temp_dir.path());

        let result = write_command_file(
            &config,
            "test-uuid-noenv",
            "/path/to/project",
            "claude --session-id abc123",
            None,
        );
        assert!(result.is_ok());

        let content = std::fs::read_to_string(result.unwrap()).unwrap();
        assert!(!content.contains("OPERATOR_"));
        assert!(!content.contains("\\033]2;"));
        assert!(content.starts_with("#!/bin/bash\ncd"));
    }
}

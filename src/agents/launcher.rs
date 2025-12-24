#![allow(dead_code)]

use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result};
use uuid::Uuid;

use super::tmux::{sanitize_session_name, SystemTmuxClient, TmuxClient, TmuxError};
use crate::config::Config;
use crate::notifications;
use crate::queue::{Queue, Ticket};
use crate::state::State;
use crate::templates::{schema::TemplateSchema, TemplateType};

/// Session name prefix for operator-managed tmux sessions
pub const SESSION_PREFIX: &str = "op-";

/// Minimum required tmux version
pub const MIN_TMUX_VERSION: (u32, u32) = (2, 1);

pub struct Launcher {
    config: Config,
    tmux: Arc<dyn TmuxClient>,
}

impl Launcher {
    /// Create a new Launcher with the system tmux client
    ///
    /// Uses custom tmux config if it has been generated and exists.
    pub fn new(config: &Config) -> Result<Self> {
        // Use custom tmux config if it exists
        let tmux: Arc<dyn TmuxClient> = if config.tmux.config_generated {
            let config_path = config.tmux_config_path();
            if config_path.exists() {
                Arc::new(SystemTmuxClient::with_config(config_path))
            } else {
                Arc::new(SystemTmuxClient::new())
            }
        } else {
            Arc::new(SystemTmuxClient::new())
        };

        Ok(Self {
            config: config.clone(),
            tmux,
        })
    }

    /// Create a new Launcher with a custom tmux client (for testing)
    pub fn with_tmux_client(config: &Config, tmux: Arc<dyn TmuxClient>) -> Result<Self> {
        Ok(Self {
            config: config.clone(),
            tmux,
        })
    }

    /// Check if tmux is available and meets minimum version requirements
    pub fn check_tmux(&self) -> Result<(), TmuxError> {
        let version = self.tmux.check_available()?;

        if !version.meets_minimum(MIN_TMUX_VERSION.0, MIN_TMUX_VERSION.1) {
            return Err(TmuxError::VersionTooOld(
                version.raw,
                format!("{}.{}", MIN_TMUX_VERSION.0, MIN_TMUX_VERSION.1),
            ));
        }

        tracing::info!(
            version = %version.raw,
            "tmux is available"
        );

        Ok(())
    }

    /// Launch a Claude agent in a tmux session for the given ticket
    pub async fn launch(&self, ticket: &Ticket) -> Result<String> {
        // Move ticket to in-progress
        let queue = Queue::new(&self.config)?;
        queue.claim_ticket(ticket)?;

        // Get project path
        let project_path = self.get_project_path(ticket)?;

        // Generate the initial prompt for the agent
        let initial_prompt = self.generate_prompt(ticket);

        // Launch in tmux session
        let session_name = self.launch_in_tmux(ticket, &project_path, &initial_prompt)?;

        // Update state
        let mut state = State::load(&self.config)?;
        let agent_id = state.add_agent(
            ticket.id.clone(),
            ticket.ticket_type.clone(),
            ticket.project.clone(),
            ticket.is_paired(),
        )?;

        // Store session name in state for later recovery
        state.update_agent_session(&agent_id, &session_name)?;

        // Set the current step in state
        if !ticket.step.is_empty() {
            state.update_agent_step(&agent_id, &ticket.step)?;
        }

        // Send notification
        if self.config.notifications.enabled && self.config.notifications.on_agent_start {
            notifications::send(
                "Agent Started",
                &format!(
                    "{} - {} (tmux: {})",
                    ticket.project, ticket.ticket_type, session_name
                ),
                &ticket.summary,
                self.config.notifications.sound,
            )?;
        }

        Ok(agent_id)
    }

    fn get_project_path(&self, ticket: &Ticket) -> Result<String> {
        let projects_root = self.config.projects_path();

        let project_path = if ticket.project == "global" {
            // Global tickets use the root directory
            projects_root
        } else {
            projects_root.join(&ticket.project)
        };

        if !project_path.exists() {
            anyhow::bail!("Project path does not exist: {:?}", project_path);
        }

        Ok(project_path.to_string_lossy().to_string())
    }

    fn generate_prompt(&self, ticket: &Ticket) -> String {
        let ticket_path = self
            .config
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
                    self.config
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

    /// Launch Claude in a tmux session
    fn launch_in_tmux(
        &self,
        ticket: &Ticket,
        project_path: &str,
        initial_prompt: &str,
    ) -> Result<String> {
        // Create session name from ticket ID (sanitize for tmux)
        let session_name = format!("{}{}", SESSION_PREFIX, sanitize_session_name(&ticket.id));

        // Check if session already exists
        match self.tmux.session_exists(&session_name) {
            Ok(true) => {
                anyhow::bail!(
                    "Tmux session '{}' already exists. Attach with: tmux attach -t {}",
                    session_name,
                    session_name
                );
            }
            Err(TmuxError::NotInstalled) => {
                anyhow::bail!(
                    "tmux is not installed. Please install tmux to use operator.\n\
                     On macOS: brew install tmux\n\
                     On Ubuntu/Debian: sudo apt install tmux"
                );
            }
            Err(e) => {
                tracing::warn!(error = %e, "Error checking for existing session, proceeding anyway");
            }
            Ok(false) => {} // Good, session doesn't exist
        }

        // Create new tmux session in project directory
        self.tmux
            .create_session(&session_name, project_path)
            .map_err(|e| match e {
                TmuxError::NotInstalled => anyhow::anyhow!(
                    "tmux is not installed. Please install tmux to use operator.\n\
                     On macOS: brew install tmux\n\
                     On Ubuntu/Debian: sudo apt install tmux"
                ),
                TmuxError::SessionExists(_) => anyhow::anyhow!(
                    "Tmux session '{}' already exists. Attach with: tmux attach -t {}",
                    session_name,
                    session_name
                ),
                _ => anyhow::anyhow!("Failed to create tmux session '{}': {}", session_name, e),
            })?;

        // Wait for the shell to initialize before sending keys
        // Without this delay, send_keys may run before the shell is ready
        std::thread::sleep(std::time::Duration::from_millis(500));

        // Set up silence monitoring for awaiting input detection
        if let Err(e) = self
            .tmux
            .set_monitor_silence(&session_name, self.config.agents.silence_threshold as u32)
        {
            tracing::warn!(
                session = %session_name,
                error = %e,
                "Failed to set monitor-silence, awaiting detection may not work"
            );
        }

        // Generate a UUID for the Claude session-id
        let session_uuid = generate_session_uuid();

        // Get the step name (use "initial" if not set)
        let step_name = if ticket.step.is_empty() {
            "initial".to_string()
        } else {
            ticket.step.clone()
        };

        // Store the session UUID in the ticket file (now in in-progress)
        let ticket_in_progress_path = self
            .config
            .tickets_path()
            .join("in-progress")
            .join(&ticket.filename);
        if ticket_in_progress_path.exists() {
            if let Ok(mut updated_ticket) = Ticket::from_file(&ticket_in_progress_path) {
                if let Err(e) = updated_ticket.set_session_id(&step_name, &session_uuid) {
                    tracing::warn!(
                        error = %e,
                        ticket = %ticket.id,
                        step = %step_name,
                        "Failed to store session UUID in ticket"
                    );
                }
            }
        }

        // Get the model from first available provider (default to "sonnet" if none)
        let model = self
            .get_default_model()
            .unwrap_or_else(|| "sonnet".to_string());

        // Build the full prompt based on whether template has agent_prompt
        let full_prompt = if let Some(agent_prompt) = get_agent_prompt(&ticket.ticket_type) {
            // Templates with agent_prompt (FEAT, FIX, INV, SPIKE)
            let ticket_path = format!(".tickets/in-progress/{}", ticket.filename);
            let message = format!(
                "use the {} agent to implement the ticket at {}",
                ticket.ticket_type.to_lowercase(),
                ticket_path
            );
            // Combine prompt and message with separator
            format!("{}\n---\n{}", agent_prompt, message)
        } else {
            // Templates without agent_prompt (TASK) - use original detailed prompt
            initial_prompt.to_string()
        };

        // Write prompt to file (avoids newline issues with tmux send-keys)
        let prompt_file = write_prompt_file(&self.config, &session_uuid, &full_prompt)?;

        // Build command using the detected tool's template
        let llm_cmd = self.build_llm_command(&model, &session_uuid, &prompt_file)?;

        // Send the LLM command to the session
        if let Err(e) = self.tmux.send_keys(&session_name, &llm_cmd, true) {
            // Clean up the session if we couldn't send the command
            let _ = self.tmux.kill_session(&session_name);
            anyhow::bail!("Failed to start LLM agent in tmux session: {}", e);
        }

        tracing::info!(
            session = %session_name,
            session_uuid = %session_uuid,
            project = %ticket.project,
            ticket = %ticket.id,
            step = %step_name,
            "Launched agent in tmux session"
        );

        Ok(session_name)
    }

    /// List all operator tmux sessions
    pub fn list_sessions(&self) -> Result<Vec<String>> {
        match self.tmux.list_sessions(Some(SESSION_PREFIX)) {
            Ok(sessions) => Ok(sessions.into_iter().map(|s| s.name).collect()),
            Err(TmuxError::NotInstalled) => {
                tracing::warn!("tmux not installed, returning empty session list");
                Ok(Vec::new())
            }
            Err(e) => {
                tracing::warn!(error = %e, "Failed to list tmux sessions");
                Ok(Vec::new())
            }
        }
    }

    /// Kill a specific operator tmux session
    pub fn kill_session(&self, session_name: &str) -> Result<()> {
        self.tmux
            .kill_session(session_name)
            .context("Failed to kill tmux session")?;
        Ok(())
    }

    /// Capture the current content of a session's pane
    pub fn capture_session_content(&self, session_name: &str) -> Result<String> {
        self.tmux
            .capture_pane(session_name, false)
            .context("Failed to capture pane content")
    }

    /// Check if a session is still alive
    pub fn session_alive(&self, session_name: &str) -> bool {
        matches!(self.tmux.session_exists(session_name), Ok(true))
    }

    /// Attach to a tmux session (returns the command to run)
    pub fn attach_command(session_name: &str) -> String {
        format!("tmux attach -t {}", session_name)
    }

    /// Get the model for the first available provider
    fn get_default_model(&self) -> Option<String> {
        self.config
            .llm_tools
            .providers
            .first()
            .map(|p| p.model.clone())
    }

    /// Get the detected tool for a given provider
    fn get_detected_tool(&self, tool_name: &str) -> Option<&crate::config::DetectedTool> {
        self.config
            .llm_tools
            .detected
            .iter()
            .find(|t| t.name == tool_name)
    }

    /// Build the LLM command using the tool's command template
    fn build_llm_command(
        &self,
        model: &str,
        session_id: &str,
        prompt_file: &std::path::Path,
    ) -> Result<String> {
        // Find the first detected tool (default to claude for backwards compatibility)
        let tool = self.config.llm_tools.detected.first().ok_or_else(|| {
            anyhow::anyhow!("No LLM tool detected. Install claude, gemini, or codex CLI.")
        })?;

        // Build model flag based on tool's arg_mapping
        // For now, use a simple pattern - could be enhanced to use arg_mapping from config
        let model_flag = format!("--model {} ", model);

        // Build command from template
        let cmd = tool
            .command_template
            .replace("{{model_flag}}", &model_flag)
            .replace("{{model}}", model)
            .replace("{{session_id}}", session_id)
            .replace("{{prompt_file}}", &prompt_file.display().to_string());

        Ok(cmd)
    }
}

/// Get the agent_prompt from a template if it exists
fn get_agent_prompt(ticket_type: &str) -> Option<String> {
    TemplateType::from_key(ticket_type)
        .and_then(|tt| TemplateSchema::from_json(tt.schema()).ok())
        .and_then(|schema| schema.agent_prompt)
}

/// Generate a UUID for the claude --session-id flag
fn generate_session_uuid() -> String {
    Uuid::new_v4().to_string()
}

/// Write a prompt to a file and return the path
/// Prompts are stored in .tickets/operator/prompts/{session_uuid}.txt
fn write_prompt_file(config: &Config, session_uuid: &str, prompt: &str) -> Result<PathBuf> {
    let prompts_dir = config.tickets_path().join("operator/prompts");
    fs::create_dir_all(&prompts_dir).context("Failed to create prompts directory")?;

    let prompt_file = prompts_dir.join(format!("{}.txt", session_uuid));
    fs::write(&prompt_file, prompt).context("Failed to write prompt file")?;

    Ok(prompt_file)
}

/// Escape a string for safe use in shell command
fn shell_escape(s: &str) -> String {
    // Use single quotes and escape any single quotes within
    let escaped = s.replace('\'', "'\"'\"'");
    format!("'{}'", escaped)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agents::tmux::MockTmuxClient;
    use crate::config::PathsConfig;
    use tempfile::TempDir;
    use uuid::Uuid;

    fn make_test_config(temp_dir: &TempDir) -> Config {
        let projects_path = temp_dir.path().join("projects");
        let tickets_path = temp_dir.path().join("tickets");
        let state_path = temp_dir.path().join("state");
        std::fs::create_dir_all(&projects_path).unwrap();
        std::fs::create_dir_all(tickets_path.join("queue")).unwrap();
        std::fs::create_dir_all(tickets_path.join("in-progress")).unwrap();
        std::fs::create_dir_all(tickets_path.join("completed")).unwrap();
        std::fs::create_dir_all(&state_path).unwrap();

        // Create a test project
        let test_project = projects_path.join("test-project");
        std::fs::create_dir_all(&test_project).unwrap();
        std::fs::write(test_project.join("CLAUDE.md"), "# Test Project").unwrap();

        Config {
            paths: PathsConfig {
                tickets: tickets_path.to_string_lossy().to_string(),
                projects: projects_path.to_string_lossy().to_string(),
                state: state_path.to_string_lossy().to_string(),
            },
            projects: vec!["test-project".to_string()],
            ..Default::default()
        }
    }

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
    fn test_attach_command() {
        assert_eq!(
            Launcher::attach_command("op-TASK-123"),
            "tmux attach -t op-TASK-123"
        );
    }

    #[test]
    fn test_check_tmux_available() {
        let temp_dir = TempDir::new().unwrap();
        let config = make_test_config(&temp_dir);
        let mock = Arc::new(MockTmuxClient::new());

        let launcher = Launcher::with_tmux_client(&config, mock).unwrap();
        assert!(launcher.check_tmux().is_ok());
    }

    #[test]
    fn test_check_tmux_not_installed() {
        let temp_dir = TempDir::new().unwrap();
        let config = make_test_config(&temp_dir);
        let mock = Arc::new(MockTmuxClient::not_installed());

        let launcher = Launcher::with_tmux_client(&config, mock).unwrap();
        let result = launcher.check_tmux();
        assert!(matches!(result, Err(TmuxError::NotInstalled)));
    }

    #[test]
    fn test_check_tmux_version_too_old() {
        let temp_dir = TempDir::new().unwrap();
        let config = make_test_config(&temp_dir);
        let mock = MockTmuxClient::new();
        *mock.version.lock().unwrap() = Some(super::super::tmux::TmuxVersion {
            major: 1,
            minor: 9,
            raw: "tmux 1.9".to_string(),
        });

        let launcher = Launcher::with_tmux_client(&config, Arc::new(mock)).unwrap();
        let result = launcher.check_tmux();
        assert!(matches!(result, Err(TmuxError::VersionTooOld(_, _))));
    }

    #[test]
    fn test_list_sessions_empty() {
        let temp_dir = TempDir::new().unwrap();
        let config = make_test_config(&temp_dir);
        let mock = Arc::new(MockTmuxClient::new());

        let launcher = Launcher::with_tmux_client(&config, mock).unwrap();
        let sessions = launcher.list_sessions().unwrap();
        assert!(sessions.is_empty());
    }

    #[test]
    fn test_list_sessions_filters_prefix() {
        let temp_dir = TempDir::new().unwrap();
        let config = make_test_config(&temp_dir);
        let mock = Arc::new(MockTmuxClient::new());

        // Add some sessions
        mock.add_session("op-TASK-123", "/tmp");
        mock.add_session("op-FEAT-456", "/tmp");
        mock.add_session("other-session", "/tmp");

        let launcher = Launcher::with_tmux_client(&config, mock).unwrap();
        let sessions = launcher.list_sessions().unwrap();

        assert_eq!(sessions.len(), 2);
        assert!(sessions.iter().all(|s| s.starts_with("op-")));
    }

    #[test]
    fn test_list_sessions_tmux_not_installed() {
        let temp_dir = TempDir::new().unwrap();
        let config = make_test_config(&temp_dir);
        let mock = Arc::new(MockTmuxClient::not_installed());

        let launcher = Launcher::with_tmux_client(&config, mock).unwrap();
        let sessions = launcher.list_sessions().unwrap();

        // Should return empty list, not error
        assert!(sessions.is_empty());
    }

    #[test]
    fn test_session_alive() {
        let temp_dir = TempDir::new().unwrap();
        let config = make_test_config(&temp_dir);
        let mock = Arc::new(MockTmuxClient::new());

        mock.add_session("op-TASK-123", "/tmp");

        let launcher = Launcher::with_tmux_client(&config, mock).unwrap();

        assert!(launcher.session_alive("op-TASK-123"));
        assert!(!launcher.session_alive("op-TASK-456"));
    }

    #[test]
    fn test_kill_session() {
        let temp_dir = TempDir::new().unwrap();
        let config = make_test_config(&temp_dir);
        let mock = Arc::new(MockTmuxClient::new());

        mock.add_session("op-TASK-123", "/tmp");

        let launcher = Launcher::with_tmux_client(&config, mock.clone()).unwrap();

        assert!(launcher.session_alive("op-TASK-123"));
        launcher.kill_session("op-TASK-123").unwrap();
        assert!(!launcher.session_alive("op-TASK-123"));
    }

    #[test]
    fn test_kill_session_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let config = make_test_config(&temp_dir);
        let mock = Arc::new(MockTmuxClient::new());

        let launcher = Launcher::with_tmux_client(&config, mock).unwrap();
        let result = launcher.kill_session("nonexistent");

        assert!(result.is_err());
    }

    #[test]
    fn test_capture_session_content() {
        let temp_dir = TempDir::new().unwrap();
        let config = make_test_config(&temp_dir);
        let mock = Arc::new(MockTmuxClient::new());

        mock.add_session("op-TASK-123", "/tmp");
        mock.set_session_content("op-TASK-123", "Hello from Claude!");

        let launcher = Launcher::with_tmux_client(&config, mock).unwrap();
        let content = launcher.capture_session_content("op-TASK-123").unwrap();

        assert_eq!(content, "Hello from Claude!");
    }

    #[test]
    fn test_session_name_sanitization() {
        // Test that session names are properly sanitized
        assert_eq!(
            format!("{}{}", SESSION_PREFIX, sanitize_session_name("TASK-123")),
            "op-TASK-123"
        );
        assert_eq!(
            format!("{}{}", SESSION_PREFIX, sanitize_session_name("FEAT-123.1")),
            "op-FEAT-123-1"
        );
        assert_eq!(
            format!(
                "{}{}",
                SESSION_PREFIX,
                sanitize_session_name("INV:critical")
            ),
            "op-INV-critical"
        );
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

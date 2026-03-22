#![allow(dead_code)]

//! Agent switching for per-step agent overrides.
//!
//! When a step transition occurs and the next step specifies a different agent
//! (delegator), the current agent is gracefully exited using a 3-tier escalation
//! (`/exit` → `Ctrl+C` → `Ctrl+D`) and the new one is launched in the same terminal session.
//!
//! Supports tmux and cmux backends via the `TerminalOps` trait.

use std::sync::Arc;
use std::thread;
use std::time::Duration;

use anyhow::{bail, Result};

use crate::agents::cmux::CmuxClient;
use crate::agents::tmux::TmuxClient;
use crate::agents::zellij::ZellijClient;
use crate::config::{Config, Delegator};
use crate::templates::schema::StepSchema;

/// Synchronous terminal operations needed for agent switching.
///
/// Both `TmuxClient` and `CmuxClient` are adapted to this interface.
pub trait TerminalOps: Send + Sync {
    /// Send a command/text to the session. `press_enter` appends Enter key for tmux.
    fn send_text(&self, session: &str, text: &str, press_enter: bool) -> Result<()>;

    /// Capture the current screen content from a session.
    fn read_content(&self, session: &str) -> Result<String>;
}

/// Adapt `TmuxClient` to `TerminalOps`
struct TmuxOps(Arc<dyn TmuxClient>);

impl TerminalOps for TmuxOps {
    fn send_text(&self, session: &str, text: &str, press_enter: bool) -> Result<()> {
        self.0
            .send_keys(session, text, press_enter)
            .map_err(|e| anyhow::anyhow!("{e}"))
    }

    fn read_content(&self, session: &str) -> Result<String> {
        self.0
            .capture_pane(session, false)
            .map_err(|e| anyhow::anyhow!("{e}"))
    }
}

/// Adapt `CmuxClient` to `TerminalOps`.
/// For cmux, the `session` parameter is the workspace ref.
struct CmuxOps(Arc<dyn CmuxClient>);

impl TerminalOps for CmuxOps {
    fn send_text(&self, workspace_ref: &str, text: &str, press_enter: bool) -> Result<()> {
        let text_to_send = if press_enter {
            format!("{text}\n")
        } else {
            text.to_string()
        };
        self.0
            .send_text(workspace_ref, &text_to_send)
            .map_err(|e| anyhow::anyhow!("{e}"))
    }

    fn read_content(&self, workspace_ref: &str) -> Result<String> {
        self.0
            .read_screen(workspace_ref, false)
            .map_err(|e| anyhow::anyhow!("{e}"))
    }
}

/// Adapt `ZellijClient` to `TerminalOps`.
/// For zellij, the `session` parameter is the tab name.
struct ZellijOps(Arc<dyn ZellijClient>);

impl TerminalOps for ZellijOps {
    fn send_text(&self, tab_name: &str, text: &str, press_enter: bool) -> Result<()> {
        let text_to_send = if press_enter {
            format!("{text}\n")
        } else {
            text.to_string()
        };
        self.0
            .send_text(tab_name, &text_to_send)
            .map_err(|e| anyhow::anyhow!("{e}"))
    }

    fn read_content(&self, tab_name: &str) -> Result<String> {
        self.0
            .read_screen(tab_name)
            .map_err(|e| anyhow::anyhow!("{e}"))
    }
}

// Per-agent exit commands
const CLAUDE_EXIT_CMD: &str = "/exit";
const GEMINI_EXIT_CMD: &str = "/quit";
const CODEX_EXIT_CMD: &str = "\x03"; // Ctrl+C

// Shell prompt patterns indicating agent has exited
const SHELL_PROMPT_PATTERNS: &[&str] = &["$ ", "% ", "# ", "❯ "];

// Timing constants
const EXIT_POLL_INTERVAL: Duration = Duration::from_millis(200);
const EXIT_POLL_TIMEOUT: Duration = Duration::from_secs(3);
const SHELL_STABILIZE_WAIT: Duration = Duration::from_secs(2);
const AGENT_READY_TIMEOUT: Duration = Duration::from_secs(30);
const AGENT_READY_POLL: Duration = Duration::from_millis(500);

pub struct AgentSwitcher {
    ops: Box<dyn TerminalOps>,
}

impl AgentSwitcher {
    /// Create an `AgentSwitcher` using a tmux client
    pub fn new(tmux: Arc<dyn TmuxClient>) -> Self {
        Self {
            ops: Box::new(TmuxOps(tmux)),
        }
    }

    /// Create an `AgentSwitcher` using a cmux client
    pub fn with_cmux(cmux: Arc<dyn CmuxClient>) -> Self {
        Self {
            ops: Box::new(CmuxOps(cmux)),
        }
    }

    /// Create an `AgentSwitcher` using a zellij client
    pub fn with_zellij(zellij: Arc<dyn ZellijClient>) -> Self {
        Self {
            ops: Box::new(ZellijOps(zellij)),
        }
    }

    /// Create an `AgentSwitcher` with a custom `TerminalOps` implementation (for testing)
    pub fn with_ops(ops: Box<dyn TerminalOps>) -> Self {
        Self { ops }
    }

    /// Check if a step requires switching to a different agent.
    /// Returns the delegator to switch to, or None if no switch is needed.
    pub fn needs_switch(
        &self,
        current_tool: &str,
        current_model: &str,
        step: &StepSchema,
        config: &Config,
    ) -> Option<Delegator> {
        let agent_name = step.agent.as_ref()?;

        // Look up agent name in config.delegators
        let delegator = config.delegators.iter().find(|d| &d.name == agent_name)?;

        // Compare to current tool/model
        if delegator.llm_tool == current_tool && delegator.model == current_model {
            None
        } else {
            Some(delegator.clone())
        }
    }

    /// Switch from the current agent to a new one in the given tmux session.
    ///
    /// 1. Send exit command for current tool
    /// 2. Poll for shell prompt (3-tier escalation: exit → Ctrl+C → Ctrl+D)
    /// 3. Wait for shell stabilization
    /// 4. Launch new agent command
    /// 5. Poll for agent readiness
    pub fn switch_agent(&self, session: &str, current_tool: &str, new_command: &str) -> Result<()> {
        // Step 1: Send exit command for the current tool
        let exit_cmd = exit_command_for_tool(current_tool);
        self.ops
            .send_text(session, exit_cmd, !exit_cmd.starts_with('\x03'))?;

        // Step 2: Poll for shell prompt with escalation
        if !self.poll_for_shell(session, EXIT_POLL_TIMEOUT) {
            // Escalation tier 2: Ctrl+C
            self.ops.send_text(session, "\x03", false)?;
            if !self.poll_for_shell(session, EXIT_POLL_TIMEOUT) {
                // Escalation tier 3: Ctrl+D
                self.ops.send_text(session, "\x04", false)?;
                if !self.poll_for_shell(session, EXIT_POLL_TIMEOUT) {
                    bail!(
                        "Failed to exit agent '{current_tool}' in session '{session}' after 3-tier escalation"
                    );
                }
            }
        }

        // Step 3: Wait for shell to stabilize
        thread::sleep(SHELL_STABILIZE_WAIT);

        // Step 4: Launch the new agent
        self.ops.send_text(session, new_command, true)?;

        // Step 5: Poll for agent readiness
        if !self.poll_for_agent_ready(session, AGENT_READY_TIMEOUT) {
            bail!(
                "New agent did not become ready in session '{session}' within {AGENT_READY_TIMEOUT:?}"
            );
        }

        Ok(())
    }

    /// Poll for a shell prompt pattern in the session's pane content.
    /// Returns true if a shell prompt is detected within the timeout.
    fn poll_for_shell(&self, session: &str, timeout: Duration) -> bool {
        let start = std::time::Instant::now();

        while start.elapsed() < timeout {
            if let Ok(content) = self.ops.read_content(session) {
                let trimmed = content.trim_end();
                if let Some(last_line) = trimmed.lines().last() {
                    let last = last_line.trim();
                    for pattern in SHELL_PROMPT_PATTERNS {
                        if last.ends_with(pattern.trim()) {
                            return true;
                        }
                    }
                }
            }
            thread::sleep(EXIT_POLL_INTERVAL);
        }

        false
    }

    /// Poll for agent readiness by checking content stability.
    /// The agent is considered ready when two consecutive captures produce the same
    /// non-shell content (indicating the agent has loaded and is waiting for input).
    fn poll_for_agent_ready(&self, session: &str, timeout: Duration) -> bool {
        let start = std::time::Instant::now();
        let mut last_content: Option<String> = None;

        while start.elapsed() < timeout {
            if let Ok(content) = self.ops.read_content(session) {
                let trimmed = content.trim().to_string();

                // Skip if content looks like a shell prompt (agent hasn't started yet)
                let is_shell = trimmed.lines().last().is_none_or(|last| {
                    let l = last.trim();
                    SHELL_PROMPT_PATTERNS.iter().any(|p| l.ends_with(p.trim()))
                });

                if !is_shell && !trimmed.is_empty() {
                    if let Some(ref prev) = last_content {
                        if prev == &trimmed {
                            return true;
                        }
                    }
                    last_content = Some(trimmed);
                }
            }
            thread::sleep(AGENT_READY_POLL);
        }

        false
    }
}

/// Build the CLI command string for launching a delegator's agent.
pub fn build_agent_command(delegator: &Delegator) -> String {
    let mut cmd = delegator.llm_tool.clone();
    if !delegator.model.is_empty() {
        cmd.push_str(&format!(" --model {}", delegator.model));
    }
    if let Some(ref launch_config) = delegator.launch_config {
        for flag in &launch_config.flags {
            cmd.push_str(&format!(" {flag}"));
        }
    }
    cmd
}

/// Get the exit command for a given LLM tool name.
fn exit_command_for_tool(tool: &str) -> &'static str {
    match tool {
        "claude" => CLAUDE_EXIT_CMD,
        "gemini" => GEMINI_EXIT_CMD,
        "codex" => CODEX_EXIT_CMD,
        _ => CLAUDE_EXIT_CMD, // Default to /exit
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agents::tmux::MockTmuxClient;
    use crate::config::{Config, Delegator};
    use crate::templates::schema::{PermissionMode, ReviewType, StepSchema};
    use std::collections::HashMap;

    fn make_step(agent: Option<&str>) -> StepSchema {
        StepSchema {
            name: "build".to_string(),
            display_name: None,
            outputs: vec![],
            prompt: "Build it".to_string(),
            allowed_tools: vec![],
            review_type: ReviewType::None,
            visual_config: None,
            on_reject: None,
            next_step: None,
            permissions: None,
            cli_args: None,
            permission_mode: PermissionMode::Default,
            agent: agent.map(std::string::ToString::to_string),
            json_schema: None,
            json_schema_file: None,
            artifact_patterns: vec![],
        }
    }

    fn make_config_with_delegators(delegators: Vec<Delegator>) -> Config {
        Config {
            delegators,
            ..Default::default()
        }
    }

    fn make_delegator(name: &str, tool: &str, model: &str) -> Delegator {
        Delegator {
            name: name.to_string(),
            llm_tool: tool.to_string(),
            model: model.to_string(),
            display_name: None,
            model_properties: HashMap::default(),
            launch_config: None,
        }
    }

    // ─── exit_command_for_tool tests ─────────────────────────────────────────────

    #[test]
    fn test_exit_command_for_tool() {
        assert_eq!(exit_command_for_tool("claude"), "/exit");
        assert_eq!(exit_command_for_tool("gemini"), "/quit");
        assert_eq!(exit_command_for_tool("codex"), "\x03");
        assert_eq!(exit_command_for_tool("unknown"), "/exit"); // default
    }

    // ─── needs_switch tests ─────────────────────────────────────────────────────

    #[test]
    fn test_needs_switch_different_tool() {
        let tmux = Arc::new(MockTmuxClient::new());
        let switcher = AgentSwitcher::new(tmux);

        let step = make_step(Some("gemini-pro"));
        let config =
            make_config_with_delegators(vec![make_delegator("gemini-pro", "gemini", "pro")]);

        let result = switcher.needs_switch("claude", "opus", &step, &config);
        assert!(result.is_some());
        let delegator = result.unwrap();
        assert_eq!(delegator.llm_tool, "gemini");
        assert_eq!(delegator.model, "pro");
    }

    #[test]
    fn test_needs_switch_same_tool() {
        let tmux = Arc::new(MockTmuxClient::new());
        let switcher = AgentSwitcher::new(tmux);

        let step = make_step(Some("claude-opus"));
        let config =
            make_config_with_delegators(vec![make_delegator("claude-opus", "claude", "opus")]);

        let result = switcher.needs_switch("claude", "opus", &step, &config);
        assert!(result.is_none());
    }

    #[test]
    fn test_needs_switch_no_agent() {
        let tmux = Arc::new(MockTmuxClient::new());
        let switcher = AgentSwitcher::new(tmux);

        let step = make_step(None);
        let config = make_config_with_delegators(vec![]);

        let result = switcher.needs_switch("claude", "opus", &step, &config);
        assert!(result.is_none());
    }

    #[test]
    fn test_needs_switch_unknown_delegator() {
        let tmux = Arc::new(MockTmuxClient::new());
        let switcher = AgentSwitcher::new(tmux);

        let step = make_step(Some("nonexistent"));
        let config = make_config_with_delegators(vec![]);

        let result = switcher.needs_switch("claude", "opus", &step, &config);
        assert!(result.is_none());
    }

    #[test]
    fn test_needs_switch_same_tool_different_model() {
        let tmux = Arc::new(MockTmuxClient::new());
        let switcher = AgentSwitcher::new(tmux);

        let step = make_step(Some("claude-sonnet"));
        let config =
            make_config_with_delegators(vec![make_delegator("claude-sonnet", "claude", "sonnet")]);

        let result = switcher.needs_switch("claude", "opus", &step, &config);
        assert!(result.is_some());
        let delegator = result.unwrap();
        assert_eq!(delegator.llm_tool, "claude");
        assert_eq!(delegator.model, "sonnet");
    }

    // ─── switch_agent tests ─────────────────────────────────────────────────────

    #[test]
    fn test_switch_agent_graceful_exit() {
        let mock = Arc::new(MockTmuxClient::new());
        mock.add_session("op-test", "/tmp/project");

        // Set content to show shell prompt (agent exited immediately)
        mock.set_session_content("op-test", "user@host ~/project $ ");

        let switcher = AgentSwitcher::new(mock.clone());
        let _result = switcher.switch_agent("op-test", "claude", "gemini --model pro");

        // Should succeed (agent readiness poll will fail but that's ok for unit test)
        // The key assertions are about what was sent
        let keys = mock.get_session_keys_sent("op-test").unwrap();
        assert!(
            keys[0].contains("/exit"),
            "Should send /exit for claude, got: {keys:?}"
        );
    }

    #[test]
    fn test_switch_agent_sends_correct_exit_for_gemini() {
        let mock = Arc::new(MockTmuxClient::new());
        mock.add_session("op-test", "/tmp/project");
        mock.set_session_content("op-test", "user@host ~/project $ ");

        let switcher = AgentSwitcher::new(mock.clone());
        let _ = switcher.switch_agent("op-test", "gemini", "claude --model opus");

        let keys = mock.get_session_keys_sent("op-test").unwrap();
        assert!(
            keys[0].contains("/quit"),
            "Should send /quit for gemini, got: {keys:?}"
        );
    }

    #[test]
    fn test_switch_agent_sends_correct_exit_for_codex() {
        let mock = Arc::new(MockTmuxClient::new());
        mock.add_session("op-test", "/tmp/project");
        mock.set_session_content("op-test", "user@host ~/project $ ");

        let switcher = AgentSwitcher::new(mock.clone());
        let _ = switcher.switch_agent("op-test", "codex", "claude --model opus");

        let keys = mock.get_session_keys_sent("op-test").unwrap();
        // Codex exit is Ctrl+C (0x03), sent without Enter
        assert!(
            keys[0].contains('\x03'),
            "Should send Ctrl+C for codex, got: {keys:?}"
        );
    }
}

//! Launch and relaunch options for agent sessions

use crate::config::LlmProvider;

/// Launch options for starting an agent with specific provider and mode settings
#[derive(Debug, Clone, Default)]
pub struct LaunchOptions {
    /// LLM provider to use (if None, use default)
    pub provider: Option<LlmProvider>,
    /// Delegator name used for this launch (for state tracking and step-level switching)
    pub delegator_name: Option<String>,
    /// Additional CLI flags from delegator `launch_config`
    pub extra_flags: Vec<String>,
    /// Run in docker container
    pub docker_mode: bool,
    /// Run in YOLO (auto-accept) mode
    pub yolo_mode: bool,
    /// Override project path (if None, use ticket's project)
    pub project_override: Option<String>,
    /// Override global `git.use_worktrees` from delegator (None = use global config)
    pub use_worktrees_override: Option<bool>,
    /// Override branch creation from delegator (None = default behavior)
    pub create_branch_override: Option<bool>,
    /// Prompt text to prepend before the generated step prompt
    pub prompt_prefix: Option<String>,
    /// Prompt text to append after the generated step prompt
    pub prompt_suffix: Option<String>,
    /// Suffix appended to the generated session name to differentiate
    /// multiple sub-agents launched for the same ticket (multi-agent steps).
    /// When `None`, session name is the usual `{prefix}{sanitized-ticket-id}`.
    pub session_suffix: Option<String>,
}

impl LaunchOptions {
    /// Get the launch mode string for state tracking
    pub fn launch_mode_string(&self) -> String {
        match (self.docker_mode, self.yolo_mode) {
            (true, true) => "docker-yolo".to_string(),
            (true, false) => "docker".to_string(),
            (false, true) => "yolo".to_string(),
            (false, false) => "default".to_string(),
        }
    }
}

/// Options for relaunching an existing in-progress ticket
#[derive(Debug, Clone, Default)]
pub struct RelaunchOptions {
    /// Base launch options
    pub launch_options: LaunchOptions,
    /// Existing Claude session UUID to resume (optional)
    /// If provided, uses --resume flag with the existing prompt file
    pub resume_session_id: Option<String>,
    /// Feedback from previous attempt (what went wrong)
    pub retry_reason: Option<String>,
}

//! Shared test harness for launch integration tests across all session wrappers.
//!
//! This module provides `LaunchTestContext` and common helpers used by
//! `launch_integration.rs` (tmux), `launch_integration_zellij.rs`, and
//! `launch_integration_cmux.rs`.

#![allow(dead_code)]

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use serde::Deserialize;
use tempfile::TempDir;

// ─── Configuration ────────────────────────────────────────────────────────────

/// Which wrapper the test context should generate config for
#[derive(Clone, Copy)]
pub enum WrapperTestMode {
    Tmux,
    Zellij,
    Cmux,
}

/// Check if launch integration tests are enabled
pub fn launch_tests_enabled() -> bool {
    env::var("OPERATOR_LAUNCH_TEST_ENABLED")
        .map(|v| v == "true" || v == "1")
        .unwrap_or(false)
}

// ─── Test Data Structures ───────────────────────────────────────────────────

/// Captured invocation data from mock LLM
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct MockInvocation {
    pub timestamp: String,
    pub invocation_id: String,
    pub command: String,
    pub args: Vec<String>,
    pub session_id: String,
    pub model: String,
    pub prompt_file: String,
    pub prompt_content: String,
    pub config_flags: String,
    pub cwd: String,
}

// ─── Test Context ───────────────────────────────────────────────────────────

/// Test context holding temporary directories and providing helpers.
///
/// Generic across all wrappers — the `WrapperTestMode` controls which
/// `[sessions]` block is written to the config TOML.
pub struct LaunchTestContext {
    pub temp_dir: TempDir,
    pub output_dir: TempDir,
    pub config_path: PathBuf,
    pub tickets_path: PathBuf,
    pub projects_path: PathBuf,
    pub state_path: PathBuf,
    pub mock_llm_path: PathBuf,
    pub test_id: String,
}

impl LaunchTestContext {
    /// Create a new test context with isolated directories
    pub fn new(test_name: &str, mode: WrapperTestMode) -> Self {
        let test_id = format!(
            "{}-{}",
            test_name,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        );

        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let output_dir = TempDir::new().expect("Failed to create output dir");

        let tickets_path = temp_dir.path().join("tickets");
        let projects_path = temp_dir.path().join("projects");
        let state_path = temp_dir.path().join("state");

        // Create directory structure
        fs::create_dir_all(tickets_path.join("queue")).unwrap();
        fs::create_dir_all(tickets_path.join("in-progress")).unwrap();
        fs::create_dir_all(tickets_path.join("completed")).unwrap();
        fs::create_dir_all(tickets_path.join("operator/prompts")).unwrap();
        fs::create_dir_all(tickets_path.join("operator/sessions")).unwrap();
        fs::create_dir_all(tickets_path.join("operator/templates")).unwrap();
        fs::create_dir_all(&state_path).unwrap();
        fs::create_dir_all(temp_dir.path().join("worktrees")).unwrap();

        // Create mock project CLAUDE.md
        // Note: Project name must be lowercase alphanumeric (no hyphens) to match filename regex
        fs::create_dir_all(projects_path.join("testproject")).unwrap();
        fs::write(
            projects_path.join("testproject/CLAUDE.md"),
            "# Test Project\n\nThis is a test project for integration testing.",
        )
        .unwrap();

        // Get path to mock LLM script
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let mock_llm_source = PathBuf::from(manifest_dir).join("tests/fixtures/mock_llm.sh");
        let mock_llm_path = temp_dir.path().join("mock-claude");

        // Copy mock LLM script to temp dir
        fs::copy(&mock_llm_source, &mock_llm_path).expect("Failed to copy mock LLM script");

        // Make executable on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&mock_llm_path).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&mock_llm_path, perms).unwrap();
        }

        // Generate wrapper-specific sessions config
        let sessions_config = match mode {
            WrapperTestMode::Tmux => String::new(), // default is tmux, no [sessions] needed
            WrapperTestMode::Zellij => r#"
[sessions]
wrapper = "zellij"

[sessions.zellij]
require_in_zellij = false
"#
            .to_string(),
            WrapperTestMode::Cmux => format!(
                r#"
[sessions]
wrapper = "cmux"

[sessions.cmux]
binary_path = "{mock_llm}"
require_in_cmux = false
placement = "workspace"
"#,
                mock_llm = mock_llm_path.display(),
            ),
        };

        // Generate test config
        let config_path = temp_dir.path().join("config.toml");
        let config_content = format!(
            r#"
[paths]
tickets = "{tickets}"
projects = "{projects}"
state = "{state}"
worktrees = "{worktrees}"

[llm_tools]
detection_complete = true

[[llm_tools.detected]]
name = "claude"
path = "{mock_llm}"
version = "1.0.0-test"
version_ok = true
model_aliases = ["sonnet", "opus", "haiku"]
command_template = "{mock_llm} {{{{config_flags}}}}{{{{model_flag}}}}--session-id {{{{session_id}}}} --print-prompt-path {{{{prompt_file}}}}"
yolo_flags = ["--dangerously-skip-permissions"]

[llm_tools.detected.capabilities]
supports_sessions = true
supports_headless = true

[[llm_tools.providers]]
tool = "claude"
model = "sonnet"

[notifications]
enabled = false
on_agent_start = false
on_agent_complete = false
on_agent_needs_input = false
on_pr_created = false
on_investigation_created = false
sound = false

[notifications.os]
enabled = false
sound = false
events = []

[agents]
max_agents = 4
silence_threshold = 30
reserved_cores = 2

[launch]

[launch.docker]
image = ""
mount_path = "/workspace"
env_vars = []
extra_args = []

[tmux]
config_generated = false
{sessions_config}
"#,
            tickets = tickets_path.display(),
            projects = projects_path.display(),
            state = state_path.display(),
            worktrees = temp_dir.path().join("worktrees").display(),
            mock_llm = mock_llm_path.display(),
            sessions_config = sessions_config,
        );
        fs::write(&config_path, config_content).unwrap();

        Self {
            temp_dir,
            output_dir,
            config_path,
            tickets_path,
            projects_path,
            state_path,
            mock_llm_path,
            test_id,
        }
    }

    /// Create a new test context with a custom sessions TOML block.
    ///
    /// Use this when you need a wrapper type not covered by `WrapperTestMode`
    /// (e.g., vscode).
    pub fn new_with_sessions_toml(test_name: &str, sessions_toml: &str) -> Self {
        // Build with Tmux mode (doesn't matter, we'll override the sessions config)
        let ctx = Self::new(test_name, WrapperTestMode::Tmux);

        // Re-generate the config with the custom sessions block
        let config_content = format!(
            r#"
[paths]
tickets = "{tickets}"
projects = "{projects}"
state = "{state}"
worktrees = "{worktrees}"

[llm_tools]
detection_complete = true

[[llm_tools.detected]]
name = "claude"
path = "{mock_llm}"
version = "1.0.0-test"
version_ok = true
model_aliases = ["sonnet", "opus", "haiku"]
command_template = "{mock_llm} {{{{config_flags}}}}{{{{model_flag}}}}--session-id {{{{session_id}}}} --print-prompt-path {{{{prompt_file}}}}"
yolo_flags = ["--dangerously-skip-permissions"]

[llm_tools.detected.capabilities]
supports_sessions = true
supports_headless = true

[[llm_tools.providers]]
tool = "claude"
model = "sonnet"

[notifications]
enabled = false
on_agent_start = false
on_agent_complete = false
on_agent_needs_input = false
on_pr_created = false
on_investigation_created = false
sound = false

[notifications.os]
enabled = false
sound = false
events = []

[agents]
max_agents = 4
silence_threshold = 30
reserved_cores = 2

[launch]

[launch.docker]
image = ""
mount_path = "/workspace"
env_vars = []
extra_args = []

[tmux]
config_generated = false
{sessions_toml}
"#,
            tickets = ctx.tickets_path.display(),
            projects = ctx.projects_path.display(),
            state = ctx.state_path.display(),
            worktrees = ctx.temp_dir.path().join("worktrees").display(),
            mock_llm = ctx.mock_llm_path.display(),
            sessions_toml = sessions_toml,
        );
        fs::write(&ctx.config_path, config_content).unwrap();

        ctx
    }

    /// Create a test ticket in the queue
    /// Filename format: YYYYMMDD-HHMM-TYPE-PROJECT-description.md
    /// Note: Project name must be lowercase alphanumeric (no hyphens)
    pub fn create_ticket(&self, ticket_type: &str, ticket_id: &str, content: &str) -> PathBuf {
        let timestamp = chrono::Local::now().format("%Y%m%d-%H%M").to_string();
        // Use "testproject" (no hyphen) to match the project directory
        let description = ticket_id.to_lowercase().replace('-', "_");
        let filename = format!("{timestamp}-{ticket_type}-testproject-{description}.md");
        let path = self.tickets_path.join("queue").join(&filename);
        fs::write(&path, content).unwrap();
        path
    }

    /// Create a `DEFINITION_OF_DONE.md` template file
    pub fn create_definition_of_done(&self, content: &str) {
        let path = self
            .tickets_path
            .join("operator/templates/DEFINITION_OF_DONE.md");
        fs::write(path, content).unwrap();
    }

    /// Create `ACCEPTANCE_CRITERIA.md` template file
    pub fn create_acceptance_criteria(&self, content: &str) {
        let path = self
            .tickets_path
            .join("operator/templates/ACCEPTANCE_CRITERIA.md");
        fs::write(path, content).unwrap();
    }

    /// Get the operator binary path
    pub fn operator_bin(&self) -> PathBuf {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        // Try release first, then debug
        let release_path = PathBuf::from(manifest_dir).join("target/release/operator");
        if release_path.exists() {
            return release_path;
        }
        PathBuf::from(manifest_dir).join("target/debug/operator")
    }

    /// Run operator launch command
    pub fn run_launch(&self, args: &[&str]) -> std::process::Output {
        let mut cmd = Command::new(self.operator_bin());
        cmd.arg("--config")
            .arg(&self.config_path)
            .arg("launch")
            .arg("--yes"); // Skip confirmation

        for arg in args {
            cmd.arg(arg);
        }

        cmd.env("MOCK_LLM_OUTPUT_DIR", self.output_dir.path())
            .env("RUST_BACKTRACE", "1")
            .output()
            .expect("Failed to run operator")
    }

    /// Get all invocation files from mock LLM
    pub fn get_invocations(&self) -> Vec<MockInvocation> {
        let mut invocations = Vec::new();

        if let Ok(entries) = fs::read_dir(self.output_dir.path()) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map(|e| e == "json").unwrap_or(false) {
                    if let Ok(content) = fs::read_to_string(&path) {
                        if let Ok(inv) = serde_json::from_str(&content) {
                            invocations.push(inv);
                        }
                    }
                }
            }
        }

        invocations
    }

    /// Get the latest invocation
    pub fn get_latest_invocation(&self) -> Option<MockInvocation> {
        let mut invocations = self.get_invocations();
        invocations.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        invocations.into_iter().next()
    }

    /// Read prompt file content from the prompts directory
    pub fn read_prompt_files(&self) -> Vec<String> {
        let prompts_dir = self.tickets_path.join("operator/prompts");
        let mut contents = Vec::new();

        if let Ok(entries) = fs::read_dir(prompts_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map(|e| e == "txt").unwrap_or(false) {
                    if let Ok(content) = fs::read_to_string(&path) {
                        contents.push(content);
                    }
                }
            }
        }

        contents
    }

    /// Read command file content from the commands directory
    pub fn read_command_files(&self) -> Vec<(PathBuf, String)> {
        let commands_dir = self.tickets_path.join("operator/commands");
        let mut files = Vec::new();

        if let Ok(entries) = fs::read_dir(commands_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map(|e| e == "sh").unwrap_or(false) {
                    if let Ok(content) = fs::read_to_string(&path) {
                        files.push((path, content));
                    }
                }
            }
        }

        files
    }

    /// Check if ticket was moved to in-progress
    pub fn ticket_in_progress(&self) -> bool {
        let in_progress = self.tickets_path.join("in-progress");
        in_progress
            .read_dir()
            .map(|mut d| d.next().is_some())
            .unwrap_or(false)
    }
}

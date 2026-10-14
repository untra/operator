#![allow(dead_code)]
#![allow(unused_imports)]

//! `TMux` session management abstraction layer.
//!
//! Provides a trait-based abstraction over tmux operations to enable:
//! - Unit testing without real tmux
//! - Mocking session behavior
//! - Graceful handling when tmux is unavailable

use std::collections::HashMap;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::{Arc, Mutex};

use anyhow::{Context, Result};
use thiserror::Error;

/// Errors specific to tmux operations
#[derive(Error, Debug)]
pub enum TmuxError {
    #[error("tmux is not installed or not in PATH")]
    NotInstalled,

    #[error("tmux version {0} is below minimum required version {1}")]
    VersionTooOld(String, String),

    #[error("tmux server is not running")]
    ServerNotRunning,

    #[error("session '{0}' already exists")]
    SessionExists(String),

    #[error("session '{0}' not found")]
    SessionNotFound(String),

    #[error("failed to create session '{0}': {1}")]
    SessionCreationFailed(String, String),

    #[error("failed to send keys to session '{0}': {1}")]
    SendKeysFailed(String, String),

    #[error("failed to set buffer '{0}': {1}")]
    SetBufferFailed(String, String),

    #[error("failed to paste buffer '{0}' to session '{1}': {2}")]
    PasteBufferFailed(String, String, String),

    #[error("failed to delete buffer '{0}': {1}")]
    DeleteBufferFailed(String, String),

    #[error("tmux command failed: {0}")]
    CommandFailed(String),
}

/// Version information for tmux
#[derive(Debug, Clone, PartialEq)]
pub struct TmuxVersion {
    pub major: u32,
    pub minor: u32,
    pub raw: String,
}

impl TmuxVersion {
    /// Parse a version string like "tmux 3.4" or "tmux 3.3a"
    pub fn parse(version_str: &str) -> Option<Self> {
        // Format: "tmux X.Y" or "tmux X.Ya"
        let parts: Vec<&str> = version_str.split_whitespace().collect();
        if parts.len() < 2 {
            return None;
        }

        let version_part = parts[1];
        let numeric_part: String = version_part
            .chars()
            .take_while(|c| c.is_ascii_digit() || *c == '.')
            .collect();

        let mut version_nums = numeric_part.split('.');
        let major: u32 = version_nums.next()?.parse().ok()?;
        let minor: u32 = version_nums.next().unwrap_or("0").parse().unwrap_or(0);

        Some(Self {
            major,
            minor,
            raw: version_str.to_string(),
        })
    }

    /// Check if this version meets the minimum requirement
    pub fn meets_minimum(&self, min_major: u32, min_minor: u32) -> bool {
        self.major > min_major || (self.major == min_major && self.minor >= min_minor)
    }
}

/// Information about a tmux session
#[derive(Debug, Clone)]
pub struct TmuxSession {
    pub name: String,
    pub created: Option<String>,
    pub attached: bool,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

/// Trait abstracting tmux operations for testability
pub trait TmuxClient: Send + Sync {
    /// Check if tmux is available and return version info
    fn check_available(&self) -> Result<TmuxVersion, TmuxError>;

    /// Check if a session exists
    fn session_exists(&self, name: &str) -> Result<bool, TmuxError>;

    /// Create a new detached session
    fn create_session(&self, name: &str, working_dir: &str) -> Result<(), TmuxError>;

    /// Send keys to a session
    fn send_keys(&self, session: &str, keys: &str, press_enter: bool) -> Result<(), TmuxError>;

    /// Kill a session
    fn kill_session(&self, name: &str) -> Result<(), TmuxError>;

    /// List all sessions matching a prefix
    fn list_sessions(&self, prefix: Option<&str>) -> Result<Vec<TmuxSession>, TmuxError>;

    /// Capture pane content from a session
    fn capture_pane(&self, session: &str, with_escape_codes: bool) -> Result<String, TmuxError>;

    /// Set the size of a detached session's window
    fn set_window_size(&self, session: &str, width: u32, height: u32) -> Result<(), TmuxError>;

    /// Check if the tmux server is running
    fn server_running(&self) -> bool;

    /// Enable silence monitoring on a session window
    /// When the session is inactive for `seconds`, the silence flag will be set
    fn set_monitor_silence(&self, session: &str, seconds: u32) -> Result<(), TmuxError>;

    /// Check if the window has the silence flag set (inactive for monitor-silence period)
    fn check_silence_flag(&self, session: &str) -> Result<bool, TmuxError>;

    /// Reset the silence flag after handling (by briefly toggling monitor-silence)
    fn reset_silence_flag(&self, session: &str) -> Result<(), TmuxError>;

    /// Attach to a session (returns the command to execute for shell attach)
    /// This suspends the current terminal and attaches to the tmux session
    fn attach_session(&self, session: &str) -> Result<(), TmuxError>;

    /// Point the attached client at `target` (`session` or `session:window`)
    fn switch_client(&self, target: &str) -> Result<(), TmuxError>;

    /// Set a client-detached hook that runs a command when the client detaches
    fn set_client_detached_hook(&self, session: &str, command: &str) -> Result<(), TmuxError>;

    /// Clear the client-detached hook
    fn clear_client_detached_hook(&self, session: &str) -> Result<(), TmuxError>;

    /// Attach to session with detach hook that creates a signal file
    /// Returns the path to the signal file that will be created on detach
    fn attach_session_with_detach_signal(&self, session: &str) -> Result<String, TmuxError>;

    /// Set a named tmux buffer with the given content
    fn set_buffer(&self, buffer_name: &str, content: &str) -> Result<(), TmuxError>;

    /// Paste a named buffer to a session
    fn paste_buffer(&self, buffer_name: &str, session: &str) -> Result<(), TmuxError>;

    /// Delete a named buffer
    fn delete_buffer(&self, buffer_name: &str) -> Result<(), TmuxError>;

    /// Send a command to a session via buffer (bypasses send-keys length limit)
    /// Uses: set-buffer -> paste-buffer -> send Enter -> delete-buffer
    fn send_command_via_buffer(&self, session: &str, command: &str) -> Result<(), TmuxError>;

    /// Safe version of `send_keys` that automatically uses buffer for long commands
    /// Commands over `SEND_KEYS_THRESHOLD` bytes use the buffer method to avoid tmux limits
    fn send_keys_safe(&self, session: &str, keys: &str, press_enter: bool)
        -> Result<(), TmuxError>;
}

/// Real implementation using system tmux
pub struct SystemTmuxClient {
    /// Path to custom tmux config file (None = use default)
    config_path: Option<PathBuf>,
    /// Tmux server socket name (None = use default socket)
    /// Using a dedicated socket ensures our custom config is always used
    socket_name: Option<String>,
}

/// Default socket name for operator-managed tmux sessions
pub const OPERATOR_SOCKET: &str = "operator";

/// Threshold in bytes for switching from `send_keys` to buffer method
/// tmux 1.8 has ~2KB limit, tmux 1.9+ has ~16KB limit. Use conservative value.
const SEND_KEYS_THRESHOLD: usize = 2000;

impl SystemTmuxClient {
    /// Create a new client using default tmux config and socket
    pub fn new() -> Self {
        Self {
            config_path: None,
            socket_name: None,
        }
    }

    /// Create a new client using a custom tmux config file
    /// This also uses a dedicated socket ("operator") to ensure isolation
    pub fn with_config(config_path: PathBuf) -> Self {
        Self {
            config_path: Some(config_path),
            socket_name: Some(OPERATOR_SOCKET.to_string()),
        }
    }

    /// The client the rest of Operator uses: the generated config and socket when set up.
    pub fn for_config(config: &crate::config::Config) -> Self {
        let config_path = config.tmux_config_path();
        if config.tmux.config_generated && config_path.exists() {
            Self::with_config(config_path)
        } else {
            Self::new()
        }
    }

    fn run_tmux(&self, args: &[&str]) -> Result<Output, TmuxError> {
        let mut cmd = Command::new("tmux");

        // Use dedicated socket if configured (must come before -f)
        if let Some(ref socket) = self.socket_name {
            cmd.arg("-L").arg(socket);
        }

        // If custom config is set, add -f flag
        if let Some(ref config_path) = self.config_path {
            cmd.arg("-f").arg(config_path);
        }

        cmd.args(args).output().map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                TmuxError::NotInstalled
            } else {
                TmuxError::CommandFailed(e.to_string())
            }
        })
    }
}

impl Default for SystemTmuxClient {
    fn default() -> Self {
        Self::new()
    }
}

impl TmuxClient for SystemTmuxClient {
    fn check_available(&self) -> Result<TmuxVersion, TmuxError> {
        let output = self.run_tmux(&["-V"])?;

        if !output.status.success() {
            return Err(TmuxError::NotInstalled);
        }

        let version_str = String::from_utf8_lossy(&output.stdout);
        TmuxVersion::parse(version_str.trim()).ok_or_else(|| {
            TmuxError::CommandFailed(format!("Could not parse version: {version_str}"))
        })
    }

    fn session_exists(&self, name: &str) -> Result<bool, TmuxError> {
        // Use exact match with -t=
        let output = self.run_tmux(&["has-session", "-t", &format!("={name}")]);

        match output {
            Ok(out) => Ok(out.status.success()),
            Err(TmuxError::NotInstalled) => Err(TmuxError::NotInstalled),
            Err(_) => Ok(false), // Server not running or other error means no session
        }
    }

    fn create_session(&self, name: &str, working_dir: &str) -> Result<(), TmuxError> {
        // Check if already exists
        if self.session_exists(name)? {
            return Err(TmuxError::SessionExists(name.to_string()));
        }

        let output = self.run_tmux(&["new-session", "-d", "-s", name, "-c", working_dir])?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(TmuxError::SessionCreationFailed(
                name.to_string(),
                stderr.to_string(),
            ));
        }

        Ok(())
    }

    fn send_keys(&self, session: &str, keys: &str, press_enter: bool) -> Result<(), TmuxError> {
        let mut args = vec!["send-keys", "-t", session, keys];
        if press_enter {
            args.push("Enter");
        }

        let output = self.run_tmux(&args)?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(TmuxError::SendKeysFailed(
                session.to_string(),
                stderr.to_string(),
            ));
        }

        Ok(())
    }

    fn kill_session(&self, name: &str) -> Result<(), TmuxError> {
        let output = self.run_tmux(&["kill-session", "-t", name])?;

        if !output.status.success() {
            return Err(TmuxError::SessionNotFound(name.to_string()));
        }

        Ok(())
    }

    fn list_sessions(&self, prefix: Option<&str>) -> Result<Vec<TmuxSession>, TmuxError> {
        let output = self.run_tmux(&[
            "list-sessions",
            "-F",
            "#{session_name}\t#{session_created}\t#{session_attached}\t#{window_width}\t#{window_height}",
        ]);

        match output {
            Ok(out) if out.status.success() => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                let sessions: Vec<TmuxSession> = stdout
                    .lines()
                    .filter_map(|line| {
                        let parts: Vec<&str> = line.split('\t').collect();
                        if parts.is_empty() {
                            return None;
                        }

                        let name = parts[0].to_string();

                        // Filter by prefix if provided
                        if let Some(p) = prefix {
                            if !name.starts_with(p) {
                                return None;
                            }
                        }

                        Some(TmuxSession {
                            name,
                            created: parts.get(1).map(|s| (*s).to_string()),
                            attached: parts.get(2).is_some_and(|s| *s == "1"),
                            width: parts.get(3).and_then(|s| s.parse().ok()),
                            height: parts.get(4).and_then(|s| s.parse().ok()),
                        })
                    })
                    .collect();

                Ok(sessions)
            }
            Ok(_) => Ok(Vec::new()), // No sessions or server not running
            Err(TmuxError::NotInstalled) => Err(TmuxError::NotInstalled),
            Err(_) => Ok(Vec::new()),
        }
    }

    fn capture_pane(&self, session: &str, with_escape_codes: bool) -> Result<String, TmuxError> {
        let mut args = vec!["capture-pane", "-p", "-t", session];
        if with_escape_codes {
            args.push("-e"); // Preserve escape sequences (colors)
        }

        let output = self.run_tmux(&args)?;

        if !output.status.success() {
            return Err(TmuxError::SessionNotFound(session.to_string()));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    fn set_window_size(&self, session: &str, width: u32, height: u32) -> Result<(), TmuxError> {
        let size_arg = format!("{width}x{height}");
        let output = self.run_tmux(&[
            "resize-window",
            "-t",
            session,
            "-x",
            &width.to_string(),
            "-y",
            &height.to_string(),
        ])?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(TmuxError::CommandFailed(format!(
                "Failed to resize window to {size_arg}: {stderr}"
            )));
        }

        Ok(())
    }

    fn server_running(&self) -> bool {
        matches!(
            self.run_tmux(&["list-sessions"]),
            Ok(out) if out.status.success() || out.status.code() == Some(1)
        )
    }

    fn set_monitor_silence(&self, session: &str, seconds: u32) -> Result<(), TmuxError> {
        let output = self.run_tmux(&[
            "set-window-option",
            "-t",
            session,
            "monitor-silence",
            &seconds.to_string(),
        ])?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(TmuxError::CommandFailed(format!(
                "Failed to set monitor-silence on {session}: {stderr}"
            )));
        }

        Ok(())
    }

    fn check_silence_flag(&self, session: &str) -> Result<bool, TmuxError> {
        let output = self.run_tmux(&[
            "display-message",
            "-p",
            "-t",
            session,
            "#{window_silence_flag}",
        ])?;

        if !output.status.success() {
            return Err(TmuxError::SessionNotFound(session.to_string()));
        }

        let flag = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(flag == "1")
    }

    fn reset_silence_flag(&self, session: &str) -> Result<(), TmuxError> {
        // Reset by toggling monitor-silence off then back on
        // First get current value
        let output =
            self.run_tmux(&["display-message", "-p", "-t", session, "#{monitor-silence}"])?;

        let current_value = String::from_utf8_lossy(&output.stdout).trim().to_string();

        // Turn off briefly
        self.run_tmux(&["set-window-option", "-t", session, "monitor-silence", "0"])?;

        // Restore original value
        self.run_tmux(&[
            "set-window-option",
            "-t",
            session,
            "monitor-silence",
            &current_value,
        ])?;

        Ok(())
    }

    fn switch_client(&self, target: &str) -> Result<(), TmuxError> {
        let output = self.run_tmux(&["switch-client", "-t", target])?;
        if output.status.success() {
            Ok(())
        } else {
            Err(TmuxError::CommandFailed(
                String::from_utf8_lossy(&output.stderr).trim().to_string(),
            ))
        }
    }

    fn attach_session(&self, session: &str) -> Result<(), TmuxError> {
        // Use status() instead of output() for interactive execution.
        // This allows the user to interact with the tmux session directly.
        // The caller should handle terminal suspension/restoration.
        let mut cmd = Command::new("tmux");

        // Use dedicated socket if configured (must come before -f)
        if let Some(ref socket) = self.socket_name {
            cmd.arg("-L").arg(socket);
        }

        // If custom config is set, add -f flag
        if let Some(ref config_path) = self.config_path {
            cmd.arg("-f").arg(config_path);
        }

        let status = cmd
            .args(["attach-session", "-t", session])
            .status()
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    TmuxError::NotInstalled
                } else {
                    TmuxError::CommandFailed(e.to_string())
                }
            })?;

        if !status.success() {
            return Err(TmuxError::CommandFailed(format!(
                "tmux attach failed with exit code: {:?}",
                status.code()
            )));
        }

        Ok(())
    }

    fn set_client_detached_hook(&self, session: &str, command: &str) -> Result<(), TmuxError> {
        // set-hook -t {session} client-detached 'run-shell "{command}"'
        let hook_cmd = format!("run-shell \"{command}\"");
        let output = self.run_tmux(&["set-hook", "-t", session, "client-detached", &hook_cmd])?;

        if !output.status.success() {
            return Err(TmuxError::CommandFailed(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }
        Ok(())
    }

    fn clear_client_detached_hook(&self, session: &str) -> Result<(), TmuxError> {
        let output = self.run_tmux(&["set-hook", "-u", "-t", session, "client-detached"])?;

        if !output.status.success() {
            // Ignore errors when hook doesn't exist
            let stderr = String::from_utf8_lossy(&output.stderr);
            if !stderr.contains("not found") {
                return Err(TmuxError::CommandFailed(stderr.to_string()));
            }
        }
        Ok(())
    }

    fn attach_session_with_detach_signal(&self, session: &str) -> Result<String, TmuxError> {
        // Create signal file path
        let signal_file = format!("/tmp/operator-detach-{session}.signal");

        // Set hook to create signal file on detach
        let hook_cmd = format!("touch {signal_file}");
        self.set_client_detached_hook(session, &hook_cmd)?;

        // Attach to the session
        self.attach_session(session)?;

        // After detach, clear the hook
        let _ = self.clear_client_detached_hook(session);

        Ok(signal_file)
    }

    fn set_buffer(&self, buffer_name: &str, content: &str) -> Result<(), TmuxError> {
        use std::io::Write;
        use std::process::Stdio;

        // Use load-buffer with stdin ("-") to avoid CLI argument length limits (ARG_MAX).
        // tmux load-buffer reads from path, where "-" means stdin.
        // This allows setting buffers with content of any size.
        let mut cmd = Command::new("tmux");

        // Use dedicated socket if configured (must come before -f)
        if let Some(ref socket) = self.socket_name {
            cmd.arg("-L").arg(socket);
        }

        // If custom config is set, add -f flag
        if let Some(ref config_path) = self.config_path {
            cmd.arg("-f").arg(config_path);
        }

        cmd.args(["load-buffer", "-b", buffer_name, "-"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = cmd.spawn().map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                TmuxError::NotInstalled
            } else {
                TmuxError::CommandFailed(e.to_string())
            }
        })?;

        // Write content to stdin
        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(content.as_bytes())
                .map_err(|e| TmuxError::SetBufferFailed(buffer_name.to_string(), e.to_string()))?;
        }

        let output = child
            .wait_with_output()
            .map_err(|e| TmuxError::SetBufferFailed(buffer_name.to_string(), e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(TmuxError::SetBufferFailed(
                buffer_name.to_string(),
                stderr.to_string(),
            ));
        }

        Ok(())
    }

    fn paste_buffer(&self, buffer_name: &str, session: &str) -> Result<(), TmuxError> {
        let output = self.run_tmux(&["paste-buffer", "-b", buffer_name, "-t", session])?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(TmuxError::PasteBufferFailed(
                buffer_name.to_string(),
                session.to_string(),
                stderr.to_string(),
            ));
        }

        Ok(())
    }

    fn delete_buffer(&self, buffer_name: &str) -> Result<(), TmuxError> {
        let output = self.run_tmux(&["delete-buffer", "-b", buffer_name])?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(TmuxError::DeleteBufferFailed(
                buffer_name.to_string(),
                stderr.to_string(),
            ));
        }

        Ok(())
    }

    fn send_command_via_buffer(&self, session: &str, command: &str) -> Result<(), TmuxError> {
        // Use unique buffer name to avoid conflicts with concurrent launches
        let buffer_name = format!("op-cmd-{session}");

        // Set buffer with the command
        self.set_buffer(&buffer_name, command)?;

        // Paste buffer to session (cleanup buffer on failure)
        if let Err(e) = self.paste_buffer(&buffer_name, session) {
            let _ = self.delete_buffer(&buffer_name);
            return Err(e);
        }

        // Send Enter to execute the command (cleanup buffer on failure)
        if let Err(e) = self.send_keys(session, "", true) {
            let _ = self.delete_buffer(&buffer_name);
            return Err(e);
        }

        // Clean up the buffer
        let _ = self.delete_buffer(&buffer_name);

        Ok(())
    }

    fn send_keys_safe(
        &self,
        session: &str,
        keys: &str,
        press_enter: bool,
    ) -> Result<(), TmuxError> {
        if keys.len() > SEND_KEYS_THRESHOLD {
            // Long command: use buffer method
            if press_enter {
                self.send_command_via_buffer(session, keys)
            } else {
                // For non-enter case with long content, still use buffer but don't press enter
                let buffer_name = format!("op-cmd-{session}");
                self.set_buffer(&buffer_name, keys)?;

                if let Err(e) = self.paste_buffer(&buffer_name, session) {
                    let _ = self.delete_buffer(&buffer_name);
                    return Err(e);
                }

                let _ = self.delete_buffer(&buffer_name);
                Ok(())
            }
        } else {
            // Short command: use regular send_keys
            self.send_keys(session, keys, press_enter)
        }
    }
}

/// Mock implementation for testing
#[derive(Default)]
pub struct MockTmuxClient {
    /// Simulated sessions: name -> (`working_dir`, content, attached)
    sessions: Arc<Mutex<HashMap<String, MockSession>>>,
    /// Simulated buffers: `buffer_name` -> content
    buffers: Arc<Mutex<HashMap<String, String>>>,
    /// Whether tmux is "installed"
    pub installed: Arc<Mutex<bool>>,
    /// Version to report
    pub version: Arc<Mutex<Option<TmuxVersion>>>,
    /// Record of commands executed
    pub command_log: Arc<Mutex<Vec<MockCommand>>>,
    /// Whether to simulate server running
    pub server_running: Arc<Mutex<bool>>,
    /// Config path used (for test verification)
    pub config_path: Arc<Mutex<Option<PathBuf>>>,
    /// Socket name used (for test verification)
    pub socket_name: Arc<Mutex<Option<String>>>,
}

#[derive(Debug, Clone)]
pub struct MockSession {
    pub working_dir: String,
    pub content: String,
    pub attached: bool,
    pub width: u32,
    pub height: u32,
    pub keys_sent: Vec<String>,
    pub monitor_silence: u32,
    pub silence_flag: bool,
    pub client_detached_hook: Option<String>,
}

#[derive(Debug, Clone)]
pub struct MockCommand {
    pub operation: String,
    pub args: Vec<String>,
}

impl MockTmuxClient {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            buffers: Arc::new(Mutex::new(HashMap::new())),
            installed: Arc::new(Mutex::new(true)),
            version: Arc::new(Mutex::new(Some(TmuxVersion {
                major: 3,
                minor: 4,
                raw: "tmux 3.4".to_string(),
            }))),
            command_log: Arc::new(Mutex::new(Vec::new())),
            server_running: Arc::new(Mutex::new(true)),
            config_path: Arc::new(Mutex::new(None)),
            socket_name: Arc::new(Mutex::new(None)),
        }
    }

    /// Create a mock that uses a custom config path and dedicated socket
    pub fn with_config(config_path: PathBuf) -> Self {
        let mock = Self::new();
        *mock.config_path.lock().unwrap() = Some(config_path);
        *mock.socket_name.lock().unwrap() = Some(OPERATOR_SOCKET.to_string());
        mock
    }

    /// Create a mock that simulates tmux not being installed
    pub fn not_installed() -> Self {
        let mock = Self::new();
        *mock.installed.lock().unwrap() = false;
        mock
    }

    /// Add a pre-existing session
    pub fn add_session(&self, name: &str, working_dir: &str) {
        self.sessions.lock().unwrap().insert(
            name.to_string(),
            MockSession {
                working_dir: working_dir.to_string(),
                content: String::new(),
                attached: false,
                width: 80,
                height: 24,
                keys_sent: Vec::new(),
                monitor_silence: 0,
                silence_flag: false,
                client_detached_hook: None,
            },
        );
    }

    /// Set the silence flag for a session (simulates silence detection)
    pub fn set_silence_flag(&self, name: &str, flag: bool) {
        if let Some(session) = self.sessions.lock().unwrap().get_mut(name) {
            session.silence_flag = flag;
        }
    }

    /// Set content for a session (simulates agent output)
    pub fn set_session_content(&self, name: &str, content: &str) {
        if let Some(session) = self.sessions.lock().unwrap().get_mut(name) {
            session.content = content.to_string();
        }
    }

    /// Get the working directory for a session (for test assertions)
    pub fn get_session_working_dir(&self, name: &str) -> Option<String> {
        self.sessions
            .lock()
            .unwrap()
            .get(name)
            .map(|s| s.working_dir.clone())
    }

    /// Get the keys sent to a session (for test assertions)
    pub fn get_session_keys_sent(&self, name: &str) -> Option<Vec<String>> {
        self.sessions
            .lock()
            .unwrap()
            .get(name)
            .map(|s| s.keys_sent.clone())
    }

    /// Get the command log
    pub fn get_commands(&self) -> Vec<MockCommand> {
        self.command_log.lock().unwrap().clone()
    }

    fn log_command(&self, operation: &str, args: &[&str]) {
        self.command_log.lock().unwrap().push(MockCommand {
            operation: operation.to_string(),
            args: args.iter().map(|s| (*s).to_string()).collect(),
        });
    }
}

impl TmuxClient for MockTmuxClient {
    fn check_available(&self) -> Result<TmuxVersion, TmuxError> {
        self.log_command("check_available", &[]);

        if !*self.installed.lock().unwrap() {
            return Err(TmuxError::NotInstalled);
        }

        self.version
            .lock()
            .unwrap()
            .clone()
            .ok_or(TmuxError::NotInstalled)
    }

    fn session_exists(&self, name: &str) -> Result<bool, TmuxError> {
        self.log_command("session_exists", &[name]);

        if !*self.installed.lock().unwrap() {
            return Err(TmuxError::NotInstalled);
        }

        Ok(self.sessions.lock().unwrap().contains_key(name))
    }

    fn create_session(&self, name: &str, working_dir: &str) -> Result<(), TmuxError> {
        self.log_command("create_session", &[name, working_dir]);

        if !*self.installed.lock().unwrap() {
            return Err(TmuxError::NotInstalled);
        }

        let mut sessions = self.sessions.lock().unwrap();
        if sessions.contains_key(name) {
            return Err(TmuxError::SessionExists(name.to_string()));
        }

        sessions.insert(
            name.to_string(),
            MockSession {
                working_dir: working_dir.to_string(),
                content: String::new(),
                attached: false,
                width: 80,
                height: 24,
                keys_sent: Vec::new(),
                monitor_silence: 0,
                silence_flag: false,
                client_detached_hook: None,
            },
        );

        Ok(())
    }

    fn send_keys(&self, session: &str, keys: &str, press_enter: bool) -> Result<(), TmuxError> {
        self.log_command(
            "send_keys",
            &[session, keys, if press_enter { "Enter" } else { "" }],
        );

        if !*self.installed.lock().unwrap() {
            return Err(TmuxError::NotInstalled);
        }

        let mut sessions = self.sessions.lock().unwrap();
        if let Some(s) = sessions.get_mut(session) {
            let mut key_record = keys.to_string();
            if press_enter {
                key_record.push_str(" [Enter]");
            }
            s.keys_sent.push(key_record);
            Ok(())
        } else {
            Err(TmuxError::SessionNotFound(session.to_string()))
        }
    }

    fn kill_session(&self, name: &str) -> Result<(), TmuxError> {
        self.log_command("kill_session", &[name]);

        if !*self.installed.lock().unwrap() {
            return Err(TmuxError::NotInstalled);
        }

        let mut sessions = self.sessions.lock().unwrap();
        if sessions.remove(name).is_some() {
            Ok(())
        } else {
            Err(TmuxError::SessionNotFound(name.to_string()))
        }
    }

    fn list_sessions(&self, prefix: Option<&str>) -> Result<Vec<TmuxSession>, TmuxError> {
        self.log_command("list_sessions", &[prefix.unwrap_or("")]);

        if !*self.installed.lock().unwrap() {
            return Err(TmuxError::NotInstalled);
        }

        let sessions = self.sessions.lock().unwrap();
        let result: Vec<TmuxSession> = sessions
            .iter()
            .filter(|(name, _)| prefix.is_none_or(|p| name.starts_with(p)))
            .map(|(name, s)| TmuxSession {
                name: name.clone(),
                created: None,
                attached: s.attached,
                width: Some(s.width),
                height: Some(s.height),
            })
            .collect();

        Ok(result)
    }

    fn capture_pane(&self, session: &str, _with_escape_codes: bool) -> Result<String, TmuxError> {
        self.log_command("capture_pane", &[session]);

        if !*self.installed.lock().unwrap() {
            return Err(TmuxError::NotInstalled);
        }

        let sessions = self.sessions.lock().unwrap();
        if let Some(s) = sessions.get(session) {
            Ok(s.content.clone())
        } else {
            Err(TmuxError::SessionNotFound(session.to_string()))
        }
    }

    fn set_window_size(&self, session: &str, width: u32, height: u32) -> Result<(), TmuxError> {
        self.log_command(
            "set_window_size",
            &[session, &width.to_string(), &height.to_string()],
        );

        if !*self.installed.lock().unwrap() {
            return Err(TmuxError::NotInstalled);
        }

        let mut sessions = self.sessions.lock().unwrap();
        if let Some(s) = sessions.get_mut(session) {
            s.width = width;
            s.height = height;
            Ok(())
        } else {
            Err(TmuxError::SessionNotFound(session.to_string()))
        }
    }

    fn server_running(&self) -> bool {
        *self.server_running.lock().unwrap()
    }

    fn set_monitor_silence(&self, session: &str, seconds: u32) -> Result<(), TmuxError> {
        self.log_command("set_monitor_silence", &[session, &seconds.to_string()]);

        if !*self.installed.lock().unwrap() {
            return Err(TmuxError::NotInstalled);
        }

        let mut sessions = self.sessions.lock().unwrap();
        if let Some(s) = sessions.get_mut(session) {
            s.monitor_silence = seconds;
            Ok(())
        } else {
            Err(TmuxError::SessionNotFound(session.to_string()))
        }
    }

    fn check_silence_flag(&self, session: &str) -> Result<bool, TmuxError> {
        self.log_command("check_silence_flag", &[session]);

        if !*self.installed.lock().unwrap() {
            return Err(TmuxError::NotInstalled);
        }

        let sessions = self.sessions.lock().unwrap();
        if let Some(s) = sessions.get(session) {
            Ok(s.silence_flag)
        } else {
            Err(TmuxError::SessionNotFound(session.to_string()))
        }
    }

    fn reset_silence_flag(&self, session: &str) -> Result<(), TmuxError> {
        self.log_command("reset_silence_flag", &[session]);

        if !*self.installed.lock().unwrap() {
            return Err(TmuxError::NotInstalled);
        }

        let mut sessions = self.sessions.lock().unwrap();
        if let Some(s) = sessions.get_mut(session) {
            s.silence_flag = false;
            Ok(())
        } else {
            Err(TmuxError::SessionNotFound(session.to_string()))
        }
    }

    fn switch_client(&self, target: &str) -> Result<(), TmuxError> {
        self.log_command("switch_client", &[target]);

        if !*self.installed.lock().unwrap() {
            return Err(TmuxError::NotInstalled);
        }

        let session = target.split(':').next().unwrap_or(target);
        if self.sessions.lock().unwrap().contains_key(session) {
            Ok(())
        } else {
            Err(TmuxError::SessionNotFound(session.to_string()))
        }
    }

    fn attach_session(&self, session: &str) -> Result<(), TmuxError> {
        self.log_command("attach_session", &[session]);

        if !*self.installed.lock().unwrap() {
            return Err(TmuxError::NotInstalled);
        }

        let mut sessions = self.sessions.lock().unwrap();
        if let Some(s) = sessions.get_mut(session) {
            s.attached = true;
            Ok(())
        } else {
            Err(TmuxError::SessionNotFound(session.to_string()))
        }
    }

    fn set_client_detached_hook(&self, session: &str, command: &str) -> Result<(), TmuxError> {
        self.log_command("set_client_detached_hook", &[session, command]);

        if !*self.installed.lock().unwrap() {
            return Err(TmuxError::NotInstalled);
        }

        let mut sessions = self.sessions.lock().unwrap();
        if let Some(s) = sessions.get_mut(session) {
            s.client_detached_hook = Some(command.to_string());
            Ok(())
        } else {
            Err(TmuxError::SessionNotFound(session.to_string()))
        }
    }

    fn clear_client_detached_hook(&self, session: &str) -> Result<(), TmuxError> {
        self.log_command("clear_client_detached_hook", &[session]);

        if !*self.installed.lock().unwrap() {
            return Err(TmuxError::NotInstalled);
        }

        let mut sessions = self.sessions.lock().unwrap();
        if let Some(s) = sessions.get_mut(session) {
            s.client_detached_hook = None;
            Ok(())
        } else {
            Err(TmuxError::SessionNotFound(session.to_string()))
        }
    }

    fn attach_session_with_detach_signal(&self, session: &str) -> Result<String, TmuxError> {
        self.log_command("attach_session_with_detach_signal", &[session]);

        if !*self.installed.lock().unwrap() {
            return Err(TmuxError::NotInstalled);
        }

        let signal_file = format!("/tmp/operator-detach-{session}.signal");

        let mut sessions = self.sessions.lock().unwrap();
        if let Some(s) = sessions.get_mut(session) {
            s.client_detached_hook = Some(format!("touch {signal_file}"));
            s.attached = true;
            Ok(signal_file)
        } else {
            Err(TmuxError::SessionNotFound(session.to_string()))
        }
    }

    fn set_buffer(&self, buffer_name: &str, content: &str) -> Result<(), TmuxError> {
        self.log_command("set_buffer", &[buffer_name, content]);

        if !*self.installed.lock().unwrap() {
            return Err(TmuxError::NotInstalled);
        }

        self.buffers
            .lock()
            .unwrap()
            .insert(buffer_name.to_string(), content.to_string());
        Ok(())
    }

    fn paste_buffer(&self, buffer_name: &str, session: &str) -> Result<(), TmuxError> {
        self.log_command("paste_buffer", &[buffer_name, session]);

        if !*self.installed.lock().unwrap() {
            return Err(TmuxError::NotInstalled);
        }

        // Check if buffer exists
        let buffers = self.buffers.lock().unwrap();
        let content = buffers.get(buffer_name).ok_or_else(|| {
            TmuxError::PasteBufferFailed(
                buffer_name.to_string(),
                session.to_string(),
                "buffer not found".to_string(),
            )
        })?;
        let content = content.clone();
        drop(buffers);

        // Check if session exists and paste content
        let mut sessions = self.sessions.lock().unwrap();
        if let Some(s) = sessions.get_mut(session) {
            // In mock, pasting buffer is like sending keys without Enter
            s.keys_sent
                .push(format!("[buffer:{buffer_name}] {content}"));
            Ok(())
        } else {
            Err(TmuxError::SessionNotFound(session.to_string()))
        }
    }

    fn delete_buffer(&self, buffer_name: &str) -> Result<(), TmuxError> {
        self.log_command("delete_buffer", &[buffer_name]);

        if !*self.installed.lock().unwrap() {
            return Err(TmuxError::NotInstalled);
        }

        self.buffers.lock().unwrap().remove(buffer_name);
        Ok(())
    }

    fn send_command_via_buffer(&self, session: &str, command: &str) -> Result<(), TmuxError> {
        self.log_command("send_command_via_buffer", &[session, command]);

        if !*self.installed.lock().unwrap() {
            return Err(TmuxError::NotInstalled);
        }

        let buffer_name = format!("op-cmd-{session}");

        // Set buffer
        self.set_buffer(&buffer_name, command)?;

        // Paste buffer (cleanup on failure)
        if let Err(e) = self.paste_buffer(&buffer_name, session) {
            let _ = self.delete_buffer(&buffer_name);
            return Err(e);
        }

        // Send Enter
        if let Err(e) = self.send_keys(session, "", true) {
            let _ = self.delete_buffer(&buffer_name);
            return Err(e);
        }

        // Clean up buffer
        let _ = self.delete_buffer(&buffer_name);

        Ok(())
    }

    fn send_keys_safe(
        &self,
        session: &str,
        keys: &str,
        press_enter: bool,
    ) -> Result<(), TmuxError> {
        self.log_command(
            "send_keys_safe",
            &[session, keys, if press_enter { "Enter" } else { "" }],
        );

        if !*self.installed.lock().unwrap() {
            return Err(TmuxError::NotInstalled);
        }

        if keys.len() > SEND_KEYS_THRESHOLD {
            // Long command: use buffer method
            if press_enter {
                self.send_command_via_buffer(session, keys)
            } else {
                let buffer_name = format!("op-cmd-{session}");
                self.set_buffer(&buffer_name, keys)?;

                if let Err(e) = self.paste_buffer(&buffer_name, session) {
                    let _ = self.delete_buffer(&buffer_name);
                    return Err(e);
                }

                let _ = self.delete_buffer(&buffer_name);
                Ok(())
            }
        } else {
            // Short command: use regular send_keys
            self.send_keys(session, keys, press_enter)
        }
    }
}

/// Sanitize a string for use as a tmux session name
pub fn sanitize_session_name(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '-'
            }
        })
        .collect()
}

// ============================================================================
// SessionWrapper implementation for tmux
// ============================================================================

use crate::agents::terminal_wrapper::{SessionError, SessionInfo, SessionWrapper, WrapperType};
use async_trait::async_trait;

/// Wrapper around `TmuxClient` that implements `SessionWrapper` trait
///
/// This provides the `SessionWrapper` interface for tmux-based session management.
/// It wraps any `TmuxClient` implementation (System or Mock).
pub struct TmuxWrapper {
    client: Arc<dyn TmuxClient>,
}

impl TmuxWrapper {
    /// Create a new `TmuxWrapper` from a `TmuxClient`
    pub fn new(client: Arc<dyn TmuxClient>) -> Self {
        Self { client }
    }

    /// Create with default system tmux client
    pub fn system() -> Self {
        Self::new(Arc::new(SystemTmuxClient::new()))
    }

    /// Create with system tmux client using custom config
    pub fn with_config(config_path: PathBuf) -> Self {
        Self::new(Arc::new(SystemTmuxClient::with_config(config_path)))
    }

    /// Get the underlying `TmuxClient`
    pub fn client(&self) -> &dyn TmuxClient {
        self.client.as_ref()
    }
}

#[async_trait]
impl SessionWrapper for TmuxWrapper {
    fn wrapper_type(&self) -> WrapperType {
        WrapperType::Tmux
    }

    async fn check_available(&self) -> Result<(), SessionError> {
        self.client
            .check_available()
            .map(|_| ())
            .map_err(|e| SessionError::NotAvailable(e.to_string()))
    }

    async fn session_exists(&self, name: &str) -> Result<bool, SessionError> {
        self.client
            .session_exists(name)
            .map_err(|e| SessionError::CommandFailed(e.to_string()))
    }

    async fn create_session(&self, name: &str, working_dir: &str) -> Result<(), SessionError> {
        self.client
            .create_session(name, working_dir)
            .map_err(|e| match e {
                TmuxError::SessionExists(n) => SessionError::SessionExists(n),
                other => SessionError::CommandFailed(other.to_string()),
            })
    }

    async fn send_command(&self, session: &str, command: &str) -> Result<(), SessionError> {
        self.client
            .send_keys(session, command, true)
            .map_err(|e| match e {
                TmuxError::SessionNotFound(n) => SessionError::SessionNotFound(n),
                other => SessionError::CommandFailed(other.to_string()),
            })
    }

    async fn kill_session(&self, name: &str) -> Result<(), SessionError> {
        self.client.kill_session(name).map_err(|e| match e {
            TmuxError::SessionNotFound(n) => SessionError::SessionNotFound(n),
            other => SessionError::CommandFailed(other.to_string()),
        })
    }

    async fn list_sessions(&self, prefix: Option<&str>) -> Result<Vec<SessionInfo>, SessionError> {
        self.client
            .list_sessions(prefix)
            .map(|sessions| {
                sessions
                    .into_iter()
                    .map(|s| SessionInfo {
                        name: s.name,
                        created: s.created,
                        attached: s.attached,
                        wrapper_type: WrapperType::Tmux,
                    })
                    .collect()
            })
            .map_err(|e| SessionError::CommandFailed(e.to_string()))
    }

    async fn focus_session(&self, session: &str) -> Result<(), SessionError> {
        self.client.attach_session(session).map_err(|e| match e {
            TmuxError::SessionNotFound(n) => SessionError::SessionNotFound(n),
            other => SessionError::CommandFailed(other.to_string()),
        })
    }

    fn capture_content(&self, session: &str) -> Result<String, SessionError> {
        self.client
            .capture_pane(session, false)
            .map_err(|e| match e {
                TmuxError::SessionNotFound(n) => SessionError::SessionNotFound(n),
                other => SessionError::CommandFailed(other.to_string()),
            })
    }

    fn supports_content_capture(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_parse() {
        let v = TmuxVersion::parse("tmux 3.4").unwrap();
        assert_eq!(v.major, 3);
        assert_eq!(v.minor, 4);

        let v = TmuxVersion::parse("tmux 3.3a").unwrap();
        assert_eq!(v.major, 3);
        assert_eq!(v.minor, 3);

        let v = TmuxVersion::parse("tmux 2.9").unwrap();
        assert_eq!(v.major, 2);
        assert_eq!(v.minor, 9);
    }

    #[test]
    fn test_version_meets_minimum() {
        let v = TmuxVersion::parse("tmux 3.4").unwrap();
        assert!(v.meets_minimum(2, 0));
        assert!(v.meets_minimum(3, 0));
        assert!(v.meets_minimum(3, 4));
        assert!(!v.meets_minimum(3, 5));
        assert!(!v.meets_minimum(4, 0));
    }

    #[test]
    fn test_sanitize_session_name() {
        assert_eq!(sanitize_session_name("simple"), "simple");
        assert_eq!(sanitize_session_name("with-dash"), "with-dash");
        assert_eq!(sanitize_session_name("with_underscore"), "with_underscore");
        assert_eq!(sanitize_session_name("with.dot"), "with-dot");
        assert_eq!(sanitize_session_name("with:colon"), "with-colon");
        assert_eq!(sanitize_session_name("with space"), "with-space");
        assert_eq!(sanitize_session_name("FEAT-123.1"), "FEAT-123-1");
    }

    #[test]
    fn test_system_tmux_with_config_stores_path() {
        let config_path = PathBuf::from("/test/.tmux.conf");
        let client = SystemTmuxClient::with_config(config_path.clone());

        assert_eq!(client.config_path, Some(config_path));
    }

    #[test]
    fn test_default_tmux_client_has_no_config() {
        let client = SystemTmuxClient::new();
        assert!(client.config_path.is_none());
    }
}

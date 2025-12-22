#![allow(dead_code)]
#![allow(unused_imports)]

//! TMux session management abstraction layer.
//!
//! Provides a trait-based abstraction over tmux operations to enable:
//! - Unit testing without real tmux
//! - Mocking session behavior
//! - Graceful handling when tmux is unavailable

use std::collections::HashMap;
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
}

/// Real implementation using system tmux
pub struct SystemTmuxClient;

impl SystemTmuxClient {
    pub fn new() -> Self {
        Self
    }

    fn run_tmux(&self, args: &[&str]) -> Result<Output, TmuxError> {
        Command::new("tmux").args(args).output().map_err(|e| {
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
            TmuxError::CommandFailed(format!("Could not parse version: {}", version_str))
        })
    }

    fn session_exists(&self, name: &str) -> Result<bool, TmuxError> {
        // Use exact match with -t=
        let output = self.run_tmux(&["has-session", "-t", &format!("={}", name)]);

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
                            created: parts.get(1).map(|s| s.to_string()),
                            attached: parts.get(2).map(|s| *s == "1").unwrap_or(false),
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
        let size_arg = format!("{}x{}", width, height);
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
                "Failed to resize window to {}: {}",
                size_arg, stderr
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
                "Failed to set monitor-silence on {}: {}",
                session, stderr
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
}

/// Mock implementation for testing
#[derive(Default)]
pub struct MockTmuxClient {
    /// Simulated sessions: name -> (working_dir, content, attached)
    sessions: Arc<Mutex<HashMap<String, MockSession>>>,
    /// Whether tmux is "installed"
    pub installed: Arc<Mutex<bool>>,
    /// Version to report
    pub version: Arc<Mutex<Option<TmuxVersion>>>,
    /// Record of commands executed
    pub command_log: Arc<Mutex<Vec<MockCommand>>>,
    /// Whether to simulate server running
    pub server_running: Arc<Mutex<bool>>,
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
            installed: Arc::new(Mutex::new(true)),
            version: Arc::new(Mutex::new(Some(TmuxVersion {
                major: 3,
                minor: 4,
                raw: "tmux 3.4".to_string(),
            }))),
            command_log: Arc::new(Mutex::new(Vec::new())),
            server_running: Arc::new(Mutex::new(true)),
        }
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

    /// Get the command log
    pub fn get_commands(&self) -> Vec<MockCommand> {
        self.command_log.lock().unwrap().clone()
    }

    fn log_command(&self, operation: &str, args: &[&str]) {
        self.command_log.lock().unwrap().push(MockCommand {
            operation: operation.to_string(),
            args: args.iter().map(|s| s.to_string()).collect(),
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
            .filter(|(name, _)| prefix.map_or(true, |p| name.starts_with(p)))
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
    fn test_mock_client_basic() {
        let client = MockTmuxClient::new();

        // Check available
        let version = client.check_available().unwrap();
        assert_eq!(version.major, 3);

        // Create session
        client.create_session("test-session", "/tmp").unwrap();

        // Session should exist
        assert!(client.session_exists("test-session").unwrap());
        assert!(!client.session_exists("other-session").unwrap());

        // List sessions
        let sessions = client.list_sessions(None).unwrap();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].name, "test-session");

        // Send keys
        client
            .send_keys("test-session", "echo hello", true)
            .unwrap();

        // Kill session
        client.kill_session("test-session").unwrap();
        assert!(!client.session_exists("test-session").unwrap());
    }

    #[test]
    fn test_mock_client_not_installed() {
        let client = MockTmuxClient::not_installed();

        assert!(matches!(
            client.check_available(),
            Err(TmuxError::NotInstalled)
        ));

        assert!(matches!(
            client.create_session("test", "/tmp"),
            Err(TmuxError::NotInstalled)
        ));
    }

    #[test]
    fn test_mock_client_session_exists_error() {
        let client = MockTmuxClient::new();

        client.create_session("test", "/tmp").unwrap();

        assert!(matches!(
            client.create_session("test", "/tmp"),
            Err(TmuxError::SessionExists(_))
        ));
    }

    #[test]
    fn test_mock_client_session_not_found() {
        let client = MockTmuxClient::new();

        assert!(matches!(
            client.send_keys("nonexistent", "test", false),
            Err(TmuxError::SessionNotFound(_))
        ));

        assert!(matches!(
            client.kill_session("nonexistent"),
            Err(TmuxError::SessionNotFound(_))
        ));
    }

    #[test]
    fn test_mock_client_command_logging() {
        let client = MockTmuxClient::new();

        client.check_available().unwrap();
        client.create_session("test", "/tmp").unwrap();
        client.send_keys("test", "echo", true).unwrap();

        let commands = client.get_commands();
        assert_eq!(commands.len(), 3);
        assert_eq!(commands[0].operation, "check_available");
        assert_eq!(commands[1].operation, "create_session");
        assert_eq!(commands[2].operation, "send_keys");
    }

    #[test]
    fn test_mock_client_capture_pane() {
        let client = MockTmuxClient::new();

        client.create_session("test", "/tmp").unwrap();
        client.set_session_content("test", "Hello, World!");

        let content = client.capture_pane("test", false).unwrap();
        assert_eq!(content, "Hello, World!");
    }

    #[test]
    fn test_mock_client_list_with_prefix() {
        let client = MockTmuxClient::new();

        client.create_session("op-task-1", "/tmp").unwrap();
        client.create_session("op-task-2", "/tmp").unwrap();
        client.create_session("other-session", "/tmp").unwrap();

        let op_sessions = client.list_sessions(Some("op-")).unwrap();
        assert_eq!(op_sessions.len(), 2);

        let all_sessions = client.list_sessions(None).unwrap();
        assert_eq!(all_sessions.len(), 3);
    }
}

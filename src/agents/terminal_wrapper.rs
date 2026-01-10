//! Terminal session wrapper abstraction layer.
//!
//! Provides a trait-based abstraction over terminal session operations to enable:
//! - Multiple backend implementations (tmux, VS Code terminals)
//! - Composition with activity detectors (hooks, shell execution events)
//! - Testability via mock implementations

use std::collections::HashSet;
use std::sync::Arc;

use async_trait::async_trait;
use thiserror::Error;
use tokio::sync::RwLock;

/// Errors specific to session wrapper operations
#[derive(Error, Debug)]
pub enum SessionError {
    #[error("session wrapper is not available: {0}")]
    NotAvailable(String),

    #[error("session '{0}' already exists")]
    SessionExists(String),

    #[error("session '{0}' not found")]
    SessionNotFound(String),

    #[error("operation not supported: {0}")]
    NotSupported(String),

    #[error("command failed: {0}")]
    CommandFailed(String),

    #[error("connection failed: {0}")]
    ConnectionFailed(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Type of session wrapper
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WrapperType {
    /// Standalone tmux sessions
    Tmux,
    /// VS Code integrated terminal (via extension webhook)
    VSCode,
}

impl std::fmt::Display for WrapperType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WrapperType::Tmux => write!(f, "tmux"),
            WrapperType::VSCode => write!(f, "vscode"),
        }
    }
}

/// Information about a session
#[derive(Debug, Clone)]
pub struct SessionInfo {
    pub name: String,
    pub created: Option<String>,
    pub attached: bool,
    pub wrapper_type: WrapperType,
}

/// Core terminal session operations
///
/// This trait abstracts the terminal-level operations needed to manage
/// agent sessions. Implementations exist for tmux (default) and VS Code
/// (via extension webhook).
#[async_trait]
pub trait SessionWrapper: Send + Sync {
    /// Get the type of wrapper
    fn wrapper_type(&self) -> WrapperType;

    /// Check if the wrapper is available and operational
    async fn check_available(&self) -> Result<(), SessionError>;

    /// Check if a session exists
    async fn session_exists(&self, name: &str) -> Result<bool, SessionError>;

    /// Create a new session in the given working directory
    async fn create_session(&self, name: &str, working_dir: &str) -> Result<(), SessionError>;

    /// Send a command to the session
    async fn send_command(&self, session: &str, command: &str) -> Result<(), SessionError>;

    /// Kill/close a session
    async fn kill_session(&self, name: &str) -> Result<(), SessionError>;

    /// List all sessions matching an optional prefix
    async fn list_sessions(&self, prefix: Option<&str>) -> Result<Vec<SessionInfo>, SessionError>;

    /// Focus/attach to a session (platform-specific behavior)
    /// - tmux: attaches to session (takes over terminal)
    /// - vscode: shows the terminal panel
    async fn focus_session(&self, session: &str) -> Result<(), SessionError>;

    /// Optional: capture terminal content
    /// Default implementation returns NotSupported - only tmux provides this.
    /// VS Code terminals don't need content capture (user sees terminal directly).
    fn capture_content(&self, _session: &str) -> Result<String, SessionError> {
        Err(SessionError::NotSupported(
            "content capture not available for this wrapper".into(),
        ))
    }

    /// Check if this wrapper supports content capture
    fn supports_content_capture(&self) -> bool {
        false
    }
}

/// Configuration for activity detection
#[derive(Debug, Clone)]
pub struct ActivityConfig {
    /// Silence threshold in seconds (for tmux monitor-silence)
    pub silence_threshold_secs: u32,
    /// Tool name for pattern matching (e.g., "claude", "gemini")
    pub tool_name: Option<String>,
}

impl Default for ActivityConfig {
    fn default() -> Self {
        Self {
            silence_threshold_secs: 10,
            tool_name: None,
        }
    }
}

/// Activity detection - knows when a session is idle vs working
///
/// Multiple implementations exist:
/// - LlmHookDetector: Uses Claude/Gemini hooks (fastest, most reliable)
/// - VSCodeActivityDetector: Uses shell execution events via extension
/// - TmuxActivityDetector: Uses silence flags + content patterns (fallback)
pub trait ActivityDetector: Send + Sync {
    /// Check if session is idle (waiting for input)
    fn is_idle(&self, session_id: &str) -> Result<bool, SessionError>;

    /// Check if session has resumed activity (for awaiting_input -> running transition)
    fn has_resumed(&self, session_id: &str) -> Result<bool, SessionError>;

    /// Configure activity detection for a session
    fn configure(&self, session_id: &str, config: &ActivityConfig) -> Result<(), SessionError>;

    /// Clear any cached state for a session (e.g., on session end)
    fn clear(&self, session_id: &str) -> Result<(), SessionError>;
}

/// Composed session combining a terminal wrapper with an activity detector
///
/// This is the main type used by the launcher and monitor to manage agent sessions.
/// It combines:
/// - A `SessionWrapper` for terminal operations (tmux or VS Code)
/// - An `ActivityDetector` for idle/resume detection (hooks, shell events, or tmux silence)
pub struct ComposedSession<W: SessionWrapper, A: ActivityDetector> {
    /// Terminal wrapper implementation
    pub terminal: W,
    /// Activity detector implementation
    pub activity: A,
    /// Session UUID (for Claude --session-id)
    pub session_uuid: String,
}

impl<W: SessionWrapper, A: ActivityDetector> ComposedSession<W, A> {
    /// Create a new composed session
    pub fn new(terminal: W, activity: A, session_uuid: String) -> Self {
        Self {
            terminal,
            activity,
            session_uuid,
        }
    }

    /// Get the wrapper type
    pub fn wrapper_type(&self) -> WrapperType {
        self.terminal.wrapper_type()
    }

    /// Check if the session is idle
    pub fn is_idle(&self) -> Result<bool, SessionError> {
        self.activity.is_idle(&self.session_uuid)
    }

    /// Check if the session has resumed
    pub fn has_resumed(&self) -> Result<bool, SessionError> {
        self.activity.has_resumed(&self.session_uuid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wrapper_type_display() {
        assert_eq!(WrapperType::Tmux.to_string(), "tmux");
        assert_eq!(WrapperType::VSCode.to_string(), "vscode");
    }

    #[test]
    fn test_activity_config_default() {
        let config = ActivityConfig::default();
        assert_eq!(config.silence_threshold_secs, 10);
        assert!(config.tool_name.is_none());
    }

    #[test]
    fn test_session_error_display() {
        let err = SessionError::NotSupported("test".into());
        assert_eq!(err.to_string(), "operation not supported: test");

        let err = SessionError::SessionNotFound("test-session".into());
        assert_eq!(err.to_string(), "session 'test-session' not found");
    }
}

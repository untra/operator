#![allow(dead_code)]
#![allow(unused_imports)]

//! Zellij session management abstraction layer.
//!
//! Provides a trait-based abstraction over Zellij operations to enable:
//! - Unit testing without real Zellij
//! - Mocking session behavior
//! - Graceful handling when Zellij is unavailable
//!
//! Zellij is a terminal workspace manager. When operator runs inside a Zellij
//! session, it can create tabs for agent sessions within the current Zellij
//! instance. This module mirrors the CmuxClient/CmuxWrapper pattern.

use std::collections::HashMap;
use std::process::{Command, Output};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use thiserror::Error;

use crate::agents::terminal_wrapper::{SessionError, SessionInfo, SessionWrapper, WrapperType};

/// Errors specific to Zellij operations
#[derive(Error, Debug)]
pub enum ZellijError {
    #[error("zellij is not installed")]
    NotInstalled,

    #[error("not running inside zellij (ZELLIJ env var not set)")]
    NotInZellij,

    #[error("zellij command failed: {0}")]
    CommandFailed(String),

    #[error("tab '{0}' not found")]
    TabNotFound(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Trait abstracting Zellij operations for testability
pub trait ZellijClient: Send + Sync {
    /// Check if zellij is available (binary exists and can run)
    fn check_available(&self) -> Result<(), ZellijError>;

    /// Check if we're running inside zellij (ZELLIJ env var present)
    fn check_in_zellij(&self) -> Result<(), ZellijError>;

    /// List all tab names in the current session
    fn list_tab_names(&self) -> Result<Vec<String>, ZellijError>;

    /// Create a new tab with the given name and working directory
    fn create_tab(&self, name: &str, cwd: &str) -> Result<(), ZellijError>;

    /// Send text to a named tab (focuses the tab first)
    fn send_text(&self, tab_name: &str, text: &str) -> Result<(), ZellijError>;

    /// Read screen content from a named tab
    fn read_screen(&self, tab_name: &str) -> Result<String, ZellijError>;

    /// Focus a tab by name
    fn focus_tab(&self, tab_name: &str) -> Result<(), ZellijError>;

    /// Close a tab by name (focuses it first, then closes)
    fn close_tab(&self, tab_name: &str) -> Result<(), ZellijError>;
}

// ============================================================================
// SystemZellijClient — real CLI calls
// ============================================================================

/// Real implementation using the zellij binary.
///
/// Operates via `zellij action` commands inside the current Zellij session.
pub struct SystemZellijClient;

impl SystemZellijClient {
    /// Create a new system client
    pub fn new() -> Self {
        Self
    }

    fn run_zellij(args: &[&str]) -> Result<Output, ZellijError> {
        Command::new("zellij").args(args).output().map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                ZellijError::NotInstalled
            } else {
                ZellijError::Io(e)
            }
        })
    }

    fn run_zellij_success(args: &[&str]) -> Result<String, ZellijError> {
        let output = Self::run_zellij(args)?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ZellijError::CommandFailed(format!(
                "zellij {} failed: {}",
                args.first().unwrap_or(&""),
                stderr
            )));
        }
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }
}

impl Default for SystemZellijClient {
    fn default() -> Self {
        Self::new()
    }
}

impl ZellijClient for SystemZellijClient {
    fn check_available(&self) -> Result<(), ZellijError> {
        let output = Self::run_zellij(&["--version"])?;
        if !output.status.success() {
            return Err(ZellijError::NotInstalled);
        }
        Ok(())
    }

    fn check_in_zellij(&self) -> Result<(), ZellijError> {
        if std::env::var("ZELLIJ").is_err() {
            return Err(ZellijError::NotInZellij);
        }
        Ok(())
    }

    fn list_tab_names(&self) -> Result<Vec<String>, ZellijError> {
        let output = Self::run_zellij_success(&["action", "query-tab-names"])?;
        let names = output
            .lines()
            .filter(|l| !l.is_empty())
            .map(|l| l.trim().to_string())
            .collect();
        Ok(names)
    }

    fn create_tab(&self, name: &str, cwd: &str) -> Result<(), ZellijError> {
        Self::run_zellij_success(&["action", "new-tab", "--name", name, "--cwd", cwd])?;
        Ok(())
    }

    fn send_text(&self, tab_name: &str, text: &str) -> Result<(), ZellijError> {
        // Focus the tab first, then write characters
        self.focus_tab(tab_name)?;
        Self::run_zellij_success(&["action", "write-chars", "--", text])?;
        Ok(())
    }

    fn read_screen(&self, tab_name: &str) -> Result<String, ZellijError> {
        self.focus_tab(tab_name)?;
        let temp_path = format!("/tmp/operator-zellij-{tab_name}.txt");
        Self::run_zellij_success(&["action", "dump-screen", &temp_path])?;
        let content = std::fs::read_to_string(&temp_path).map_err(ZellijError::Io)?;
        Ok(content)
    }

    fn focus_tab(&self, tab_name: &str) -> Result<(), ZellijError> {
        Self::run_zellij_success(&["action", "go-to-tab-name", tab_name])?;
        Ok(())
    }

    fn close_tab(&self, tab_name: &str) -> Result<(), ZellijError> {
        self.focus_tab(tab_name)?;
        Self::run_zellij_success(&["action", "close-tab"])?;
        Ok(())
    }
}

// ============================================================================
// MockZellijClient — in-memory state for testing
// ============================================================================

/// Mock tab for testing
#[derive(Debug, Clone)]
struct MockTab {
    name: String,
    cwd: String,
    screen_content: String,
}

/// Inner mutable state for `MockZellijClient`
#[derive(Debug)]
struct MockState {
    available: bool,
    in_zellij: bool,
    tabs: Vec<MockTab>,
    sent_texts: Vec<(String, String)>, // (tab_name, text)
}

/// Mock implementation for testing
pub struct MockZellijClient {
    state: Mutex<MockState>,
}

impl MockZellijClient {
    /// Create a new mock client in a valid state (available, in zellij, no tabs)
    pub fn new() -> Self {
        Self {
            state: Mutex::new(MockState {
                available: true,
                in_zellij: true,
                tabs: vec![],
                sent_texts: vec![],
            }),
        }
    }

    /// Set whether zellij is available
    pub fn set_available(&self, available: bool) {
        if let Ok(mut state) = self.state.lock() {
            state.available = available;
        }
    }

    /// Set whether we're running inside zellij
    pub fn set_in_zellij(&self, in_zellij: bool) {
        if let Ok(mut state) = self.state.lock() {
            state.in_zellij = in_zellij;
        }
    }

    /// Set screen content for a tab
    pub fn set_screen_content(&self, tab_name: &str, content: &str) {
        if let Ok(mut state) = self.state.lock() {
            if let Some(tab) = state.tabs.iter_mut().find(|t| t.name == tab_name) {
                tab.screen_content = content.to_string();
            }
        }
    }

    /// Get all sent texts (`tab_name`, text) for assertions
    pub fn sent_texts(&self) -> Vec<(String, String)> {
        self.state
            .lock()
            .ok()
            .map(|s| s.sent_texts.clone())
            .unwrap_or_default()
    }
}

impl Default for MockZellijClient {
    fn default() -> Self {
        Self::new()
    }
}

impl ZellijClient for MockZellijClient {
    fn check_available(&self) -> Result<(), ZellijError> {
        let state = self
            .state
            .lock()
            .map_err(|e| ZellijError::CommandFailed(format!("lock poisoned: {e}")))?;
        if !state.available {
            return Err(ZellijError::NotInstalled);
        }
        Ok(())
    }

    fn check_in_zellij(&self) -> Result<(), ZellijError> {
        let state = self
            .state
            .lock()
            .map_err(|e| ZellijError::CommandFailed(format!("lock poisoned: {e}")))?;
        if !state.in_zellij {
            return Err(ZellijError::NotInZellij);
        }
        Ok(())
    }

    fn list_tab_names(&self) -> Result<Vec<String>, ZellijError> {
        let state = self
            .state
            .lock()
            .map_err(|e| ZellijError::CommandFailed(format!("lock poisoned: {e}")))?;
        Ok(state.tabs.iter().map(|t| t.name.clone()).collect())
    }

    fn create_tab(&self, name: &str, cwd: &str) -> Result<(), ZellijError> {
        let mut state = self
            .state
            .lock()
            .map_err(|e| ZellijError::CommandFailed(format!("lock poisoned: {e}")))?;
        state.tabs.push(MockTab {
            name: name.to_string(),
            cwd: cwd.to_string(),
            screen_content: String::new(),
        });
        Ok(())
    }

    fn send_text(&self, tab_name: &str, text: &str) -> Result<(), ZellijError> {
        let mut state = self
            .state
            .lock()
            .map_err(|e| ZellijError::CommandFailed(format!("lock poisoned: {e}")))?;

        if !state.tabs.iter().any(|t| t.name == tab_name) {
            return Err(ZellijError::TabNotFound(tab_name.to_string()));
        }

        state
            .sent_texts
            .push((tab_name.to_string(), text.to_string()));
        Ok(())
    }

    fn read_screen(&self, tab_name: &str) -> Result<String, ZellijError> {
        let state = self
            .state
            .lock()
            .map_err(|e| ZellijError::CommandFailed(format!("lock poisoned: {e}")))?;

        state
            .tabs
            .iter()
            .find(|t| t.name == tab_name)
            .map(|t| t.screen_content.clone())
            .ok_or_else(|| ZellijError::TabNotFound(tab_name.to_string()))
    }

    fn focus_tab(&self, tab_name: &str) -> Result<(), ZellijError> {
        let state = self
            .state
            .lock()
            .map_err(|e| ZellijError::CommandFailed(format!("lock poisoned: {e}")))?;

        if !state.tabs.iter().any(|t| t.name == tab_name) {
            return Err(ZellijError::TabNotFound(tab_name.to_string()));
        }
        Ok(())
    }

    fn close_tab(&self, tab_name: &str) -> Result<(), ZellijError> {
        let mut state = self
            .state
            .lock()
            .map_err(|e| ZellijError::CommandFailed(format!("lock poisoned: {e}")))?;

        let idx = state
            .tabs
            .iter()
            .position(|t| t.name == tab_name)
            .ok_or_else(|| ZellijError::TabNotFound(tab_name.to_string()))?;

        state.tabs.remove(idx);
        Ok(())
    }
}

// ============================================================================
// ZellijWrapper — SessionWrapper implementation for Zellij
// ============================================================================

/// Wrapper around `ZellijClient` that implements `SessionWrapper` trait.
///
/// This provides the `SessionWrapper` interface for Zellij-based session
/// management. Each "session" maps to a Zellij tab within the current
/// Zellij instance.
pub struct ZellijWrapper {
    client: Arc<dyn ZellijClient>,
    /// Map of session name -> tab name
    session_tabs: Mutex<HashMap<String, String>>,
}

impl ZellijWrapper {
    /// Create a new `ZellijWrapper` from a `ZellijClient`
    pub fn new(client: Arc<dyn ZellijClient>) -> Self {
        Self {
            client,
            session_tabs: Mutex::new(HashMap::new()),
        }
    }

    /// Create with the system Zellij client
    pub fn from_system() -> Self {
        let client = Arc::new(SystemZellijClient::new());
        Self::new(client)
    }

    /// Get the tab name for a session
    fn tab_name(&self, session_name: &str) -> Option<String> {
        self.session_tabs
            .lock()
            .ok()
            .and_then(|map| map.get(session_name).cloned())
    }
}

#[async_trait]
impl SessionWrapper for ZellijWrapper {
    fn wrapper_type(&self) -> WrapperType {
        WrapperType::Zellij
    }

    async fn check_available(&self) -> Result<(), SessionError> {
        self.client
            .check_available()
            .map_err(|e| SessionError::NotAvailable(e.to_string()))?;
        self.client
            .check_in_zellij()
            .map_err(|e| SessionError::NotAvailable(e.to_string()))?;
        Ok(())
    }

    async fn session_exists(&self, name: &str) -> Result<bool, SessionError> {
        Ok(self
            .session_tabs
            .lock()
            .map_err(|e| SessionError::CommandFailed(format!("lock poisoned: {e}")))?
            .contains_key(name))
    }

    async fn create_session(&self, name: &str, working_dir: &str) -> Result<(), SessionError> {
        // Check for duplicate
        if self.session_exists(name).await? {
            return Err(SessionError::SessionExists(name.to_string()));
        }

        // Create the tab
        self.client
            .create_tab(name, working_dir)
            .map_err(|e| SessionError::CommandFailed(e.to_string()))?;

        // Track the mapping (session name -> tab name, same value)
        if let Ok(mut map) = self.session_tabs.lock() {
            map.insert(name.to_string(), name.to_string());
        }

        Ok(())
    }

    async fn send_command(&self, session: &str, command: &str) -> Result<(), SessionError> {
        let tab = self
            .tab_name(session)
            .ok_or_else(|| SessionError::SessionNotFound(session.to_string()))?;

        self.client
            .send_text(&tab, command)
            .map_err(|e| SessionError::CommandFailed(e.to_string()))
    }

    async fn kill_session(&self, name: &str) -> Result<(), SessionError> {
        let tab = self
            .tab_name(name)
            .ok_or_else(|| SessionError::SessionNotFound(name.to_string()))?;

        self.client
            .close_tab(&tab)
            .map_err(|e| SessionError::CommandFailed(e.to_string()))?;

        // Clean up tracking
        if let Ok(mut map) = self.session_tabs.lock() {
            map.remove(name);
        }

        Ok(())
    }

    async fn list_sessions(&self, prefix: Option<&str>) -> Result<Vec<SessionInfo>, SessionError> {
        let tab_names = self
            .client
            .list_tab_names()
            .map_err(|e| SessionError::CommandFailed(e.to_string()))?;

        let sessions: Vec<SessionInfo> = tab_names
            .into_iter()
            .filter(|name| {
                if let Some(pfx) = prefix {
                    name.starts_with(pfx)
                } else {
                    true
                }
            })
            .map(|name| SessionInfo {
                name,
                created: None,
                attached: false,
                wrapper_type: WrapperType::Zellij,
            })
            .collect();

        Ok(sessions)
    }

    async fn focus_session(&self, session: &str) -> Result<(), SessionError> {
        let tab = self
            .tab_name(session)
            .ok_or_else(|| SessionError::SessionNotFound(session.to_string()))?;

        self.client
            .focus_tab(&tab)
            .map_err(|e| SessionError::CommandFailed(e.to_string()))
    }

    fn capture_content(&self, session: &str) -> Result<String, SessionError> {
        let tab = self
            .tab_name(session)
            .ok_or_else(|| SessionError::SessionNotFound(session.to_string()))?;

        self.client
            .read_screen(&tab)
            .map_err(|e| SessionError::CommandFailed(e.to_string()))
    }

    fn supports_content_capture(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // ZellijClient trait tests (via MockZellijClient)
    // ========================================================================

    #[test]
    fn test_zellij_check_available_succeeds() {
        let client = MockZellijClient::new();
        assert!(client.check_available().is_ok());
    }

    #[test]
    fn test_zellij_check_available_fails() {
        let client = MockZellijClient::new();
        client.set_available(false);
        let err = client.check_available().unwrap_err();
        assert!(matches!(err, ZellijError::NotInstalled));
    }

    #[test]
    fn test_zellij_check_in_zellij_fails() {
        let client = MockZellijClient::new();
        client.set_in_zellij(false);
        assert!(client.check_available().is_ok()); // available is still true
        let err = client.check_in_zellij().unwrap_err();
        assert!(matches!(err, ZellijError::NotInZellij));
    }

    #[test]
    fn test_zellij_create_tab() {
        let client = MockZellijClient::new();
        client.create_tab("test-tab", "/tmp/project").unwrap();
        let tabs = client.list_tab_names().unwrap();
        assert_eq!(tabs.len(), 1);
        assert_eq!(tabs[0], "test-tab");
    }

    #[test]
    fn test_zellij_send_text() {
        let client = MockZellijClient::new();
        client.create_tab("test-tab", "/tmp/project").unwrap();
        client.send_text("test-tab", "echo hello").unwrap();

        let texts = client.sent_texts();
        assert_eq!(texts.len(), 1);
        assert_eq!(texts[0], ("test-tab".to_string(), "echo hello".to_string()));
    }

    #[test]
    fn test_zellij_read_screen() {
        let client = MockZellijClient::new();
        client.create_tab("test-tab", "/tmp/project").unwrap();

        // Empty by default
        let content = client.read_screen("test-tab").unwrap();
        assert!(content.is_empty());

        // Set and read content
        client.set_screen_content("test-tab", "$ hello\nworld");
        let content = client.read_screen("test-tab").unwrap();
        assert_eq!(content, "$ hello\nworld");
    }

    #[test]
    fn test_zellij_close_tab() {
        let client = MockZellijClient::new();
        client.create_tab("test-tab", "/tmp/project").unwrap();
        assert_eq!(client.list_tab_names().unwrap().len(), 1);

        client.close_tab("test-tab").unwrap();
        assert_eq!(client.list_tab_names().unwrap().len(), 0);

        // Should not be found after close
        assert!(client.read_screen("test-tab").is_err());
    }

    #[test]
    fn test_zellij_list_tab_names() {
        let client = MockZellijClient::new();
        client.create_tab("tab-a", "/tmp/a").unwrap();
        client.create_tab("tab-b", "/tmp/b").unwrap();
        client.create_tab("tab-c", "/tmp/c").unwrap();

        let tabs = client.list_tab_names().unwrap();
        assert_eq!(tabs.len(), 3);
        assert!(tabs.contains(&"tab-a".to_string()));
        assert!(tabs.contains(&"tab-b".to_string()));
        assert!(tabs.contains(&"tab-c".to_string()));
    }

    #[test]
    fn test_zellij_send_text_tab_not_found() {
        let client = MockZellijClient::new();
        let err = client.send_text("nonexistent", "hello").unwrap_err();
        assert!(matches!(err, ZellijError::TabNotFound(_)));
    }

    #[test]
    fn test_zellij_focus_tab_not_found() {
        let client = MockZellijClient::new();
        let err = client.focus_tab("nonexistent").unwrap_err();
        assert!(matches!(err, ZellijError::TabNotFound(_)));
    }

    #[test]
    fn test_zellij_close_tab_not_found() {
        let client = MockZellijClient::new();
        let err = client.close_tab("nonexistent").unwrap_err();
        assert!(matches!(err, ZellijError::TabNotFound(_)));
    }

    // ========================================================================
    // ZellijWrapper SessionWrapper tests
    // ========================================================================

    #[tokio::test]
    async fn test_zellij_wrapper_type() {
        let wrapper = ZellijWrapper::new(Arc::new(MockZellijClient::new()));
        assert_eq!(wrapper.wrapper_type(), WrapperType::Zellij);
    }

    #[tokio::test]
    async fn test_zellij_wrapper_supports_content_capture() {
        let wrapper = ZellijWrapper::new(Arc::new(MockZellijClient::new()));
        assert!(wrapper.supports_content_capture());
    }

    #[tokio::test]
    async fn test_zellij_wrapper_check_available() {
        let wrapper = ZellijWrapper::new(Arc::new(MockZellijClient::new()));
        assert!(wrapper.check_available().await.is_ok());
    }

    #[tokio::test]
    async fn test_zellij_wrapper_check_available_not_installed() {
        let client = MockZellijClient::new();
        client.set_available(false);
        let wrapper = ZellijWrapper::new(Arc::new(client));
        let err = wrapper.check_available().await.unwrap_err();
        assert!(matches!(err, SessionError::NotAvailable(_)));
    }

    #[tokio::test]
    async fn test_zellij_wrapper_check_available_not_in_zellij() {
        let client = MockZellijClient::new();
        client.set_in_zellij(false);
        let wrapper = ZellijWrapper::new(Arc::new(client));
        let err = wrapper.check_available().await.unwrap_err();
        assert!(matches!(err, SessionError::NotAvailable(_)));
    }

    #[tokio::test]
    async fn test_zellij_wrapper_create_and_list() {
        let client = Arc::new(MockZellijClient::new());
        let wrapper = ZellijWrapper::new(client);

        // Create sessions
        wrapper
            .create_session("op-TASK-001", "/tmp/project-a")
            .await
            .unwrap();
        wrapper
            .create_session("op-FEAT-002", "/tmp/project-b")
            .await
            .unwrap();

        // Session exists
        assert!(wrapper.session_exists("op-TASK-001").await.unwrap());
        assert!(wrapper.session_exists("op-FEAT-002").await.unwrap());
        assert!(!wrapper.session_exists("nonexistent").await.unwrap());

        // List all via client tab names
        let all = wrapper.list_sessions(None).await.unwrap();
        assert_eq!(all.len(), 2);

        // List with prefix
        let tasks = wrapper.list_sessions(Some("op-TASK")).await.unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].name, "op-TASK-001");
        assert_eq!(tasks[0].wrapper_type, WrapperType::Zellij);
    }

    #[tokio::test]
    async fn test_zellij_wrapper_create_duplicate_rejected() {
        let wrapper = ZellijWrapper::new(Arc::new(MockZellijClient::new()));

        wrapper
            .create_session("op-TASK-001", "/tmp/project")
            .await
            .unwrap();
        let err = wrapper
            .create_session("op-TASK-001", "/tmp/project")
            .await
            .unwrap_err();
        assert!(matches!(err, SessionError::SessionExists(_)));
    }

    #[tokio::test]
    async fn test_zellij_wrapper_send_command() {
        let client = Arc::new(MockZellijClient::new());
        let wrapper = ZellijWrapper::new(client.clone());

        wrapper
            .create_session("op-TASK-001", "/tmp/project")
            .await
            .unwrap();
        wrapper
            .send_command("op-TASK-001", "echo hello")
            .await
            .unwrap();

        let texts = client.sent_texts();
        assert_eq!(texts.len(), 1);
        assert_eq!(texts[0].1, "echo hello");
    }

    #[tokio::test]
    async fn test_zellij_wrapper_send_command_not_found() {
        let wrapper = ZellijWrapper::new(Arc::new(MockZellijClient::new()));
        let err = wrapper
            .send_command("nonexistent", "echo hello")
            .await
            .unwrap_err();
        assert!(matches!(err, SessionError::SessionNotFound(_)));
    }

    #[tokio::test]
    async fn test_zellij_wrapper_kill_session() {
        let wrapper = ZellijWrapper::new(Arc::new(MockZellijClient::new()));

        wrapper
            .create_session("op-TASK-001", "/tmp/project")
            .await
            .unwrap();
        assert!(wrapper.session_exists("op-TASK-001").await.unwrap());

        wrapper.kill_session("op-TASK-001").await.unwrap();
        assert!(!wrapper.session_exists("op-TASK-001").await.unwrap());
    }

    #[tokio::test]
    async fn test_zellij_wrapper_kill_session_not_found() {
        let wrapper = ZellijWrapper::new(Arc::new(MockZellijClient::new()));
        let err = wrapper.kill_session("nonexistent").await.unwrap_err();
        assert!(matches!(err, SessionError::SessionNotFound(_)));
    }

    #[tokio::test]
    async fn test_zellij_wrapper_focus_session() {
        let wrapper = ZellijWrapper::new(Arc::new(MockZellijClient::new()));

        wrapper
            .create_session("op-TASK-001", "/tmp/project")
            .await
            .unwrap();
        assert!(wrapper.focus_session("op-TASK-001").await.is_ok());
    }

    #[tokio::test]
    async fn test_zellij_wrapper_focus_session_not_found() {
        let wrapper = ZellijWrapper::new(Arc::new(MockZellijClient::new()));
        let err = wrapper.focus_session("nonexistent").await.unwrap_err();
        assert!(matches!(err, SessionError::SessionNotFound(_)));
    }

    #[tokio::test]
    async fn test_zellij_wrapper_capture_content() {
        let client = Arc::new(MockZellijClient::new());
        let wrapper = ZellijWrapper::new(client.clone());

        wrapper
            .create_session("op-TASK-001", "/tmp/project")
            .await
            .unwrap();

        // Set content on the tab
        client.set_screen_content("op-TASK-001", "$ claude --prompt 'test'\n> ");

        let content = wrapper.capture_content("op-TASK-001").unwrap();
        assert!(content.contains("claude"));
    }

    #[tokio::test]
    async fn test_zellij_wrapper_capture_content_not_found() {
        let wrapper = ZellijWrapper::new(Arc::new(MockZellijClient::new()));
        let err = wrapper.capture_content("nonexistent").unwrap_err();
        assert!(matches!(err, SessionError::SessionNotFound(_)));
    }
}

#![allow(dead_code)]
#![allow(unused_imports)]

//! cmux session management abstraction layer.
//!
//! Provides a trait-based abstraction over cmux operations to enable:
//! - Unit testing without real cmux
//! - Mocking session behavior
//! - Graceful handling when cmux is unavailable
//!
//! cmux is a macOS terminal multiplexer that organizes work into windows
//! and workspaces. This module mirrors the TmuxClient/TmuxWrapper pattern.

use std::collections::HashMap;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use thiserror::Error;

use crate::agents::terminal_wrapper::{SessionError, SessionInfo, SessionWrapper, WrapperType};
use crate::config::{CmuxPlacementPolicy, SessionsCmuxConfig};

/// Errors specific to cmux operations
#[derive(Error, Debug)]
pub enum CmuxError {
    #[error("cmux is not installed at configured path: {0}")]
    NotInstalled(String),

    #[error("not running inside cmux (CMUX_WORKSPACE_ID not set)")]
    NotInCmux,

    #[error("cmux command failed: {0}")]
    CommandFailed(String),

    #[error("window '{0}' not found")]
    WindowNotFound(String),

    #[error("workspace '{0}' not found")]
    WorkspaceNotFound(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Information about a cmux window
#[derive(Debug, Clone)]
pub struct CmuxWindow {
    pub id: String,
    pub name: Option<String>,
}

/// Information about a cmux workspace
#[derive(Debug, Clone)]
pub struct CmuxWorkspace {
    pub id: String,
    pub name: Option<String>,
    pub window_id: String,
}

/// Trait abstracting cmux operations for testability
pub trait CmuxClient: Send + Sync {
    /// Check if cmux is available (binary exists and can run)
    fn check_available(&self) -> Result<(), CmuxError>;

    /// Check if we're running inside cmux (env vars present)
    fn check_in_cmux(&self) -> Result<(), CmuxError>;

    /// List all open windows
    fn list_windows(&self) -> Result<Vec<CmuxWindow>, CmuxError>;

    /// Get count of open windows
    fn window_count(&self) -> Result<usize, CmuxError>;

    /// Create a new workspace in a given window
    fn create_workspace(
        &self,
        window_ref: &str,
        working_dir: &str,
        name: Option<&str>,
    ) -> Result<String, CmuxError>;

    /// Create a new window
    fn create_window(&self, name: Option<&str>) -> Result<String, CmuxError>;

    /// Send text to a workspace
    fn send_text(&self, workspace_ref: &str, text: &str) -> Result<(), CmuxError>;

    /// Read screen content from a workspace
    fn read_screen(&self, workspace_ref: &str, scrollback: bool) -> Result<String, CmuxError>;

    /// Focus a workspace (bring to front)
    fn focus_workspace(&self, workspace_ref: &str) -> Result<(), CmuxError>;

    /// Focus a window (bring to front)
    fn focus_window(&self, window_ref: &str) -> Result<(), CmuxError>;

    /// Close a workspace
    fn close_workspace(&self, workspace_ref: &str) -> Result<(), CmuxError>;

    /// Get the active window ID
    fn active_window_id(&self) -> Result<String, CmuxError>;

    /// Rename a workspace
    fn rename_workspace(&self, workspace_ref: &str, name: &str) -> Result<(), CmuxError>;

    /// Rename a window
    fn rename_window(&self, window_ref: &str, name: &str) -> Result<(), CmuxError>;
}

// ============================================================================
// SystemCmuxClient — real CLI calls
// ============================================================================

/// Real implementation using the cmux binary
pub struct SystemCmuxClient {
    binary_path: String,
}

impl SystemCmuxClient {
    /// Create a new client with the given binary path
    pub fn new(binary_path: String) -> Self {
        Self { binary_path }
    }

    /// Create from config
    pub fn from_config(config: &SessionsCmuxConfig) -> Self {
        Self::new(config.binary_path.clone())
    }

    fn run_cmux(&self, args: &[&str]) -> Result<Output, CmuxError> {
        Command::new(&self.binary_path)
            .args(args)
            .output()
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    CmuxError::NotInstalled(self.binary_path.clone())
                } else {
                    CmuxError::Io(e)
                }
            })
    }

    fn run_cmux_success(&self, args: &[&str]) -> Result<String, CmuxError> {
        let output = self.run_cmux(args)?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(CmuxError::CommandFailed(format!(
                "cmux {} failed: {}",
                args.first().unwrap_or(&""),
                stderr
            )));
        }
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }
}

impl CmuxClient for SystemCmuxClient {
    fn check_available(&self) -> Result<(), CmuxError> {
        // Check binary exists and runs
        let output = self.run_cmux(&["--version"])?;
        if !output.status.success() {
            return Err(CmuxError::NotInstalled(self.binary_path.clone()));
        }
        Ok(())
    }

    fn check_in_cmux(&self) -> Result<(), CmuxError> {
        if std::env::var("CMUX_WORKSPACE_ID").is_err() {
            return Err(CmuxError::NotInCmux);
        }
        Ok(())
    }

    fn list_windows(&self) -> Result<Vec<CmuxWindow>, CmuxError> {
        let output = self.run_cmux_success(&["list-windows", "--json"])?;
        // Parse JSON output — each line is a window
        // For now, parse simple newline-delimited IDs
        let windows = output
            .lines()
            .filter(|l| !l.is_empty())
            .map(|line| CmuxWindow {
                id: line.trim().to_string(),
                name: None,
            })
            .collect();
        Ok(windows)
    }

    fn window_count(&self) -> Result<usize, CmuxError> {
        self.list_windows().map(|w| w.len())
    }

    fn create_workspace(
        &self,
        window_ref: &str,
        working_dir: &str,
        name: Option<&str>,
    ) -> Result<String, CmuxError> {
        let mut args = vec![
            "create-workspace",
            "--window",
            window_ref,
            "--cwd",
            working_dir,
        ];
        if let Some(n) = name {
            args.push("--name");
            args.push(n);
        }
        self.run_cmux_success(&args)
    }

    fn create_window(&self, name: Option<&str>) -> Result<String, CmuxError> {
        let mut args = vec!["create-window"];
        if let Some(n) = name {
            args.push("--name");
            args.push(n);
        }
        self.run_cmux_success(&args)
    }

    fn send_text(&self, workspace_ref: &str, text: &str) -> Result<(), CmuxError> {
        self.run_cmux_success(&["send-text", "--workspace", workspace_ref, text])?;
        Ok(())
    }

    fn read_screen(&self, workspace_ref: &str, scrollback: bool) -> Result<String, CmuxError> {
        let mut args = vec!["read-screen", "--workspace", workspace_ref];
        if scrollback {
            args.push("--scrollback");
        }
        self.run_cmux_success(&args)
    }

    fn focus_workspace(&self, workspace_ref: &str) -> Result<(), CmuxError> {
        self.run_cmux_success(&["focus-workspace", "--workspace", workspace_ref])?;
        Ok(())
    }

    fn focus_window(&self, window_ref: &str) -> Result<(), CmuxError> {
        self.run_cmux_success(&["focus-window", "--window", window_ref])?;
        Ok(())
    }

    fn close_workspace(&self, workspace_ref: &str) -> Result<(), CmuxError> {
        self.run_cmux_success(&["close-workspace", "--workspace", workspace_ref])?;
        Ok(())
    }

    fn active_window_id(&self) -> Result<String, CmuxError> {
        self.run_cmux_success(&["active-window-id"])
    }

    fn rename_workspace(&self, workspace_ref: &str, name: &str) -> Result<(), CmuxError> {
        self.run_cmux_success(&[
            "rename-workspace",
            "--workspace",
            workspace_ref,
            "--name",
            name,
        ])?;
        Ok(())
    }

    fn rename_window(&self, window_ref: &str, name: &str) -> Result<(), CmuxError> {
        self.run_cmux_success(&["rename-window", "--window", window_ref, "--name", name])?;
        Ok(())
    }
}

// ============================================================================
// MockCmuxClient — in-memory state for testing
// ============================================================================

/// Mock workspace for testing
#[derive(Debug, Clone)]
struct MockWorkspace {
    id: String,
    name: Option<String>,
    window_id: String,
    working_dir: String,
    screen_content: String,
    focused: bool,
}

/// Mock window for testing
#[derive(Debug, Clone)]
struct MockWindow {
    id: String,
    name: Option<String>,
    focused: bool,
}

/// Inner mutable state for `MockCmuxClient`
#[derive(Debug)]
struct MockState {
    available: bool,
    in_cmux: bool,
    windows: Vec<MockWindow>,
    workspaces: Vec<MockWorkspace>,
    next_workspace_id: u32,
    next_window_id: u32,
    active_window_id: String,
    sent_texts: Vec<(String, String)>, // (workspace_ref, text)
}

/// Mock implementation for testing
pub struct MockCmuxClient {
    state: Mutex<MockState>,
}

impl MockCmuxClient {
    /// Create a new mock client in a valid state (available, in cmux, 1 window)
    pub fn new() -> Self {
        Self {
            state: Mutex::new(MockState {
                available: true,
                in_cmux: true,
                windows: vec![MockWindow {
                    id: "win-1".to_string(),
                    name: Some("Main".to_string()),
                    focused: true,
                }],
                workspaces: vec![],
                next_workspace_id: 1,
                next_window_id: 2,
                active_window_id: "win-1".to_string(),
                sent_texts: vec![],
            }),
        }
    }

    /// Create a mock client that is not available
    pub fn unavailable() -> Self {
        let mock = Self::new();
        if let Ok(mut state) = mock.state.lock() {
            state.available = false;
        }
        mock
    }

    /// Create a mock client that is not running inside cmux
    pub fn not_in_cmux() -> Self {
        let mock = Self::new();
        if let Ok(mut state) = mock.state.lock() {
            state.in_cmux = false;
        }
        mock
    }

    /// Add additional windows to test multi-window scenarios
    pub fn add_window(&self, name: &str) {
        if let Ok(mut state) = self.state.lock() {
            let id = format!("win-{}", state.next_window_id);
            state.next_window_id += 1;
            state.windows.push(MockWindow {
                id,
                name: Some(name.to_string()),
                focused: false,
            });
        }
    }

    /// Set screen content for a workspace
    pub fn set_screen_content(&self, workspace_ref: &str, content: &str) {
        if let Ok(mut state) = self.state.lock() {
            if let Some(ws) = state
                .workspaces
                .iter_mut()
                .find(|ws| ws.id == workspace_ref)
            {
                ws.screen_content = content.to_string();
            }
        }
    }

    /// Get all sent texts (`workspace_ref`, text) for assertions
    pub fn sent_texts(&self) -> Vec<(String, String)> {
        self.state
            .lock()
            .ok()
            .map(|s| s.sent_texts.clone())
            .unwrap_or_default()
    }

    /// Get workspace by name for assertions
    pub fn find_workspace_by_name(&self, name: &str) -> Option<String> {
        self.state.lock().ok().and_then(|state| {
            state
                .workspaces
                .iter()
                .find(|ws| ws.name.as_deref() == Some(name))
                .map(|ws| ws.id.clone())
        })
    }
}

impl Default for MockCmuxClient {
    fn default() -> Self {
        Self::new()
    }
}

impl CmuxClient for MockCmuxClient {
    fn check_available(&self) -> Result<(), CmuxError> {
        let state = self
            .state
            .lock()
            .map_err(|e| CmuxError::CommandFailed(format!("lock poisoned: {e}")))?;
        if !state.available {
            return Err(CmuxError::NotInstalled("mock".to_string()));
        }
        Ok(())
    }

    fn check_in_cmux(&self) -> Result<(), CmuxError> {
        let state = self
            .state
            .lock()
            .map_err(|e| CmuxError::CommandFailed(format!("lock poisoned: {e}")))?;
        if !state.in_cmux {
            return Err(CmuxError::NotInCmux);
        }
        Ok(())
    }

    fn list_windows(&self) -> Result<Vec<CmuxWindow>, CmuxError> {
        let state = self
            .state
            .lock()
            .map_err(|e| CmuxError::CommandFailed(format!("lock poisoned: {e}")))?;
        Ok(state
            .windows
            .iter()
            .map(|w| CmuxWindow {
                id: w.id.clone(),
                name: w.name.clone(),
            })
            .collect())
    }

    fn window_count(&self) -> Result<usize, CmuxError> {
        let state = self
            .state
            .lock()
            .map_err(|e| CmuxError::CommandFailed(format!("lock poisoned: {e}")))?;
        Ok(state.windows.len())
    }

    fn create_workspace(
        &self,
        window_ref: &str,
        working_dir: &str,
        name: Option<&str>,
    ) -> Result<String, CmuxError> {
        let mut state = self
            .state
            .lock()
            .map_err(|e| CmuxError::CommandFailed(format!("lock poisoned: {e}")))?;

        // Verify window exists
        if !state.windows.iter().any(|w| w.id == window_ref) {
            return Err(CmuxError::WindowNotFound(window_ref.to_string()));
        }

        let id = format!("ws-{}", state.next_workspace_id);
        state.next_workspace_id += 1;
        state.workspaces.push(MockWorkspace {
            id: id.clone(),
            name: name.map(std::string::ToString::to_string),
            window_id: window_ref.to_string(),
            working_dir: working_dir.to_string(),
            screen_content: String::new(),
            focused: false,
        });
        Ok(id)
    }

    fn create_window(&self, name: Option<&str>) -> Result<String, CmuxError> {
        let mut state = self
            .state
            .lock()
            .map_err(|e| CmuxError::CommandFailed(format!("lock poisoned: {e}")))?;

        let id = format!("win-{}", state.next_window_id);
        state.next_window_id += 1;
        state.windows.push(MockWindow {
            id: id.clone(),
            name: name.map(std::string::ToString::to_string),
            focused: false,
        });
        Ok(id)
    }

    fn send_text(&self, workspace_ref: &str, text: &str) -> Result<(), CmuxError> {
        let mut state = self
            .state
            .lock()
            .map_err(|e| CmuxError::CommandFailed(format!("lock poisoned: {e}")))?;

        if !state.workspaces.iter().any(|ws| ws.id == workspace_ref) {
            return Err(CmuxError::WorkspaceNotFound(workspace_ref.to_string()));
        }

        state
            .sent_texts
            .push((workspace_ref.to_string(), text.to_string()));
        Ok(())
    }

    fn read_screen(&self, workspace_ref: &str, _scrollback: bool) -> Result<String, CmuxError> {
        let state = self
            .state
            .lock()
            .map_err(|e| CmuxError::CommandFailed(format!("lock poisoned: {e}")))?;

        state
            .workspaces
            .iter()
            .find(|ws| ws.id == workspace_ref)
            .map(|ws| ws.screen_content.clone())
            .ok_or_else(|| CmuxError::WorkspaceNotFound(workspace_ref.to_string()))
    }

    fn focus_workspace(&self, workspace_ref: &str) -> Result<(), CmuxError> {
        let mut state = self
            .state
            .lock()
            .map_err(|e| CmuxError::CommandFailed(format!("lock poisoned: {e}")))?;

        if !state.workspaces.iter().any(|ws| ws.id == workspace_ref) {
            return Err(CmuxError::WorkspaceNotFound(workspace_ref.to_string()));
        }

        // Unfocus all, focus target
        for ws in &mut state.workspaces {
            ws.focused = ws.id == workspace_ref;
        }
        Ok(())
    }

    fn focus_window(&self, window_ref: &str) -> Result<(), CmuxError> {
        let mut state = self
            .state
            .lock()
            .map_err(|e| CmuxError::CommandFailed(format!("lock poisoned: {e}")))?;

        if !state.windows.iter().any(|w| w.id == window_ref) {
            return Err(CmuxError::WindowNotFound(window_ref.to_string()));
        }

        for w in &mut state.windows {
            w.focused = w.id == window_ref;
        }
        Ok(())
    }

    fn close_workspace(&self, workspace_ref: &str) -> Result<(), CmuxError> {
        let mut state = self
            .state
            .lock()
            .map_err(|e| CmuxError::CommandFailed(format!("lock poisoned: {e}")))?;

        let idx = state
            .workspaces
            .iter()
            .position(|ws| ws.id == workspace_ref)
            .ok_or_else(|| CmuxError::WorkspaceNotFound(workspace_ref.to_string()))?;

        state.workspaces.remove(idx);
        Ok(())
    }

    fn active_window_id(&self) -> Result<String, CmuxError> {
        let state = self
            .state
            .lock()
            .map_err(|e| CmuxError::CommandFailed(format!("lock poisoned: {e}")))?;
        Ok(state.active_window_id.clone())
    }

    fn rename_workspace(&self, workspace_ref: &str, name: &str) -> Result<(), CmuxError> {
        let mut state = self
            .state
            .lock()
            .map_err(|e| CmuxError::CommandFailed(format!("lock poisoned: {e}")))?;

        let ws = state
            .workspaces
            .iter_mut()
            .find(|ws| ws.id == workspace_ref)
            .ok_or_else(|| CmuxError::WorkspaceNotFound(workspace_ref.to_string()))?;

        ws.name = Some(name.to_string());
        Ok(())
    }

    fn rename_window(&self, window_ref: &str, name: &str) -> Result<(), CmuxError> {
        let mut state = self
            .state
            .lock()
            .map_err(|e| CmuxError::CommandFailed(format!("lock poisoned: {e}")))?;

        let w = state
            .windows
            .iter_mut()
            .find(|w| w.id == window_ref)
            .ok_or_else(|| CmuxError::WindowNotFound(window_ref.to_string()))?;

        w.name = Some(name.to_string());
        Ok(())
    }
}

// ============================================================================
// CmuxWrapper — SessionWrapper implementation for cmux
// ============================================================================

/// Wrapper around `CmuxClient` that implements `SessionWrapper` trait
///
/// This provides the `SessionWrapper` interface for cmux-based session management.
/// It wraps any `CmuxClient` implementation (System or Mock) and applies the
/// configured placement policy when creating sessions.
pub struct CmuxWrapper {
    client: Arc<dyn CmuxClient>,
    placement: CmuxPlacementPolicy,
    /// Map of session name → workspace ref (for routing operations)
    session_workspaces: Mutex<HashMap<String, String>>,
    /// Map of session name → window ref
    session_windows: Mutex<HashMap<String, String>>,
}

impl CmuxWrapper {
    /// Create a new `CmuxWrapper` from a `CmuxClient` and config
    pub fn new(client: Arc<dyn CmuxClient>, config: &SessionsCmuxConfig) -> Self {
        Self {
            client,
            placement: config.placement,
            session_workspaces: Mutex::new(HashMap::new()),
            session_windows: Mutex::new(HashMap::new()),
        }
    }

    /// Create with system cmux client from config
    pub fn from_config(config: &SessionsCmuxConfig) -> Self {
        let client = Arc::new(SystemCmuxClient::from_config(config));
        Self::new(client, config)
    }

    /// Get the underlying `CmuxClient`
    pub fn client(&self) -> &dyn CmuxClient {
        self.client.as_ref()
    }

    /// Get the workspace ref for a session name
    pub fn workspace_ref(&self, session_name: &str) -> Option<String> {
        self.session_workspaces
            .lock()
            .ok()
            .and_then(|map| map.get(session_name).cloned())
    }

    /// Get the window ref for a session name
    pub fn window_ref(&self, session_name: &str) -> Option<String> {
        self.session_windows
            .lock()
            .ok()
            .and_then(|map| map.get(session_name).cloned())
    }

    /// Apply placement policy to determine where to create the session.
    /// Returns (`window_ref`, `created_new_window`)
    fn resolve_placement(&self) -> Result<(String, bool), CmuxError> {
        match self.placement {
            CmuxPlacementPolicy::Workspace => {
                // Always create in active window
                let window_id = self.client.active_window_id()?;
                Ok((window_id, false))
            }
            CmuxPlacementPolicy::Window => {
                // Always create a new window
                let window_id = self.client.create_window(None)?;
                Ok((window_id, true))
            }
            CmuxPlacementPolicy::Auto => {
                // 0-1 windows → workspace in active window; >1 → new window
                let count = self.client.window_count()?;
                if count <= 1 {
                    let window_id = self.client.active_window_id()?;
                    Ok((window_id, false))
                } else {
                    let window_id = self.client.create_window(None)?;
                    Ok((window_id, true))
                }
            }
        }
    }
}

#[async_trait]
impl SessionWrapper for CmuxWrapper {
    fn wrapper_type(&self) -> WrapperType {
        WrapperType::Cmux
    }

    async fn check_available(&self) -> Result<(), SessionError> {
        self.client
            .check_available()
            .map_err(|e| SessionError::NotAvailable(e.to_string()))?;
        self.client
            .check_in_cmux()
            .map_err(|e| SessionError::NotAvailable(e.to_string()))?;
        Ok(())
    }

    async fn session_exists(&self, name: &str) -> Result<bool, SessionError> {
        Ok(self
            .session_workspaces
            .lock()
            .map_err(|e| SessionError::CommandFailed(format!("lock poisoned: {e}")))?
            .contains_key(name))
    }

    async fn create_session(&self, name: &str, working_dir: &str) -> Result<(), SessionError> {
        // Check for duplicate
        if self.session_exists(name).await? {
            return Err(SessionError::SessionExists(name.to_string()));
        }

        // Resolve placement
        let (window_ref, _new_window) = self
            .resolve_placement()
            .map_err(|e| SessionError::CommandFailed(e.to_string()))?;

        // Create workspace in the target window
        let workspace_ref = self
            .client
            .create_workspace(&window_ref, working_dir, Some(name))
            .map_err(|e| SessionError::CommandFailed(e.to_string()))?;

        // Track the mapping
        if let Ok(mut map) = self.session_workspaces.lock() {
            map.insert(name.to_string(), workspace_ref);
        }
        if let Ok(mut map) = self.session_windows.lock() {
            map.insert(name.to_string(), window_ref);
        }

        Ok(())
    }

    async fn send_command(&self, session: &str, command: &str) -> Result<(), SessionError> {
        let workspace_ref = self
            .workspace_ref(session)
            .ok_or_else(|| SessionError::SessionNotFound(session.to_string()))?;

        self.client
            .send_text(&workspace_ref, command)
            .map_err(|e| SessionError::CommandFailed(e.to_string()))
    }

    async fn kill_session(&self, name: &str) -> Result<(), SessionError> {
        let workspace_ref = self
            .workspace_ref(name)
            .ok_or_else(|| SessionError::SessionNotFound(name.to_string()))?;

        self.client
            .close_workspace(&workspace_ref)
            .map_err(|e| SessionError::CommandFailed(e.to_string()))?;

        // Clean up tracking
        if let Ok(mut map) = self.session_workspaces.lock() {
            map.remove(name);
        }
        if let Ok(mut map) = self.session_windows.lock() {
            map.remove(name);
        }

        Ok(())
    }

    async fn list_sessions(&self, prefix: Option<&str>) -> Result<Vec<SessionInfo>, SessionError> {
        let map = self
            .session_workspaces
            .lock()
            .map_err(|e| SessionError::CommandFailed(format!("lock poisoned: {e}")))?;

        let sessions: Vec<SessionInfo> = map
            .keys()
            .filter(|name| {
                if let Some(pfx) = prefix {
                    name.starts_with(pfx)
                } else {
                    true
                }
            })
            .map(|name| SessionInfo {
                name: name.clone(),
                created: None,
                attached: false,
                wrapper_type: WrapperType::Cmux,
            })
            .collect();

        Ok(sessions)
    }

    async fn focus_session(&self, session: &str) -> Result<(), SessionError> {
        let workspace_ref = self
            .workspace_ref(session)
            .ok_or_else(|| SessionError::SessionNotFound(session.to_string()))?;

        // Focus the window first, then the workspace
        if let Some(window_ref) = self.window_ref(session) {
            let _ = self
                .client
                .focus_window(&window_ref)
                .map_err(|e| SessionError::CommandFailed(e.to_string()));
        }

        self.client
            .focus_workspace(&workspace_ref)
            .map_err(|e| SessionError::CommandFailed(e.to_string()))
    }

    fn capture_content(&self, session: &str) -> Result<String, SessionError> {
        let workspace_ref = self
            .workspace_ref(session)
            .ok_or_else(|| SessionError::SessionNotFound(session.to_string()))?;

        self.client
            .read_screen(&workspace_ref, true)
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
    // CmuxClient trait tests (via MockCmuxClient)
    // ========================================================================

    #[test]
    fn test_cmux_check_available_success() {
        let client = MockCmuxClient::new();
        assert!(client.check_available().is_ok());
    }

    #[test]
    fn test_cmux_check_available_not_installed() {
        let client = MockCmuxClient::unavailable();
        let err = client.check_available().unwrap_err();
        assert!(matches!(err, CmuxError::NotInstalled(_)));
    }

    #[test]
    fn test_cmux_check_available_not_in_cmux() {
        let client = MockCmuxClient::not_in_cmux();
        assert!(client.check_available().is_ok()); // available
        let err = client.check_in_cmux().unwrap_err();
        assert!(matches!(err, CmuxError::NotInCmux));
    }

    #[test]
    fn test_cmux_window_operations() {
        let client = MockCmuxClient::new();

        // Should start with 1 window
        assert_eq!(client.window_count().unwrap(), 1);

        // Create another
        let win_id = client.create_window(Some("Second")).unwrap();
        assert!(win_id.starts_with("win-"));
        assert_eq!(client.window_count().unwrap(), 2);

        // List them
        let windows = client.list_windows().unwrap();
        assert_eq!(windows.len(), 2);
    }

    #[test]
    fn test_cmux_workspace_lifecycle() {
        let client = MockCmuxClient::new();

        // Create workspace in the existing window
        let ws_id = client
            .create_workspace("win-1", "/tmp/test", Some("test-ws"))
            .unwrap();
        assert!(ws_id.starts_with("ws-"));

        // Send text
        assert!(client.send_text(&ws_id, "echo hello").is_ok());

        // Verify sent texts
        let texts = client.sent_texts();
        assert_eq!(texts.len(), 1);
        assert_eq!(texts[0], (ws_id.clone(), "echo hello".to_string()));

        // Read screen (empty by default)
        let content = client.read_screen(&ws_id, false).unwrap();
        assert!(content.is_empty());

        // Set and read screen content
        client.set_screen_content(&ws_id, "$ hello\nworld");
        let content = client.read_screen(&ws_id, false).unwrap();
        assert_eq!(content, "$ hello\nworld");

        // Focus workspace
        assert!(client.focus_workspace(&ws_id).is_ok());

        // Rename workspace
        assert!(client.rename_workspace(&ws_id, "renamed").is_ok());

        // Close workspace
        assert!(client.close_workspace(&ws_id).is_ok());

        // Should not be found after close
        assert!(client.read_screen(&ws_id, false).is_err());
    }

    #[test]
    fn test_cmux_workspace_in_nonexistent_window() {
        let client = MockCmuxClient::new();
        let err = client
            .create_workspace("win-999", "/tmp", None)
            .unwrap_err();
        assert!(matches!(err, CmuxError::WindowNotFound(_)));
    }

    #[test]
    fn test_cmux_operations_on_nonexistent_workspace() {
        let client = MockCmuxClient::new();
        assert!(client.send_text("ws-999", "test").is_err());
        assert!(client.read_screen("ws-999", false).is_err());
        assert!(client.focus_workspace("ws-999").is_err());
        assert!(client.close_workspace("ws-999").is_err());
        assert!(client.rename_workspace("ws-999", "x").is_err());
    }

    // ========================================================================
    // CmuxWrapper SessionWrapper tests
    // ========================================================================

    fn make_wrapper(client: MockCmuxClient, placement: CmuxPlacementPolicy) -> CmuxWrapper {
        let config = SessionsCmuxConfig {
            binary_path: "/mock/cmux".to_string(),
            require_in_cmux: true,
            placement,
        };
        CmuxWrapper::new(Arc::new(client), &config)
    }

    #[tokio::test]
    async fn test_cmux_wrapper_type() {
        let wrapper = make_wrapper(MockCmuxClient::new(), CmuxPlacementPolicy::Auto);
        assert_eq!(wrapper.wrapper_type(), WrapperType::Cmux);
    }

    #[tokio::test]
    async fn test_cmux_supports_content_capture_true() {
        let wrapper = make_wrapper(MockCmuxClient::new(), CmuxPlacementPolicy::Auto);
        assert!(wrapper.supports_content_capture());
    }

    #[tokio::test]
    async fn test_cmux_check_available_wrapper() {
        let wrapper = make_wrapper(MockCmuxClient::new(), CmuxPlacementPolicy::Auto);
        assert!(wrapper.check_available().await.is_ok());
    }

    #[tokio::test]
    async fn test_cmux_check_available_not_installed_wrapper() {
        let wrapper = make_wrapper(MockCmuxClient::unavailable(), CmuxPlacementPolicy::Auto);
        let err = wrapper.check_available().await.unwrap_err();
        assert!(matches!(err, SessionError::NotAvailable(_)));
    }

    #[tokio::test]
    async fn test_cmux_check_available_not_in_cmux_wrapper() {
        let wrapper = make_wrapper(MockCmuxClient::not_in_cmux(), CmuxPlacementPolicy::Auto);
        let err = wrapper.check_available().await.unwrap_err();
        assert!(matches!(err, SessionError::NotAvailable(_)));
    }

    #[tokio::test]
    async fn test_cmux_create_session_single_window_placement() {
        // Auto placement with 1 window → workspace in active window
        let wrapper = make_wrapper(MockCmuxClient::new(), CmuxPlacementPolicy::Auto);

        wrapper
            .create_session("op-TASK-001", "/tmp/project")
            .await
            .unwrap();

        assert!(wrapper.session_exists("op-TASK-001").await.unwrap());
        assert!(wrapper.workspace_ref("op-TASK-001").is_some());
        // Should use existing window, not create new one
        assert_eq!(wrapper.window_ref("op-TASK-001").unwrap(), "win-1");
    }

    #[tokio::test]
    async fn test_cmux_create_session_multi_window_placement() {
        // Auto placement with >1 windows → new window
        let client = MockCmuxClient::new();
        client.add_window("Second");
        let wrapper = make_wrapper(client, CmuxPlacementPolicy::Auto);

        wrapper
            .create_session("op-TASK-002", "/tmp/project")
            .await
            .unwrap();

        assert!(wrapper.session_exists("op-TASK-002").await.unwrap());
        // Should have created a new window (win-3, since win-2 was added by add_window)
        let win_ref = wrapper.window_ref("op-TASK-002").unwrap();
        assert_ne!(win_ref, "win-1"); // Not the original window
    }

    #[tokio::test]
    async fn test_cmux_create_session_workspace_policy() {
        // Workspace policy → always in active window
        let client = MockCmuxClient::new();
        client.add_window("Second");
        let wrapper = make_wrapper(client, CmuxPlacementPolicy::Workspace);

        wrapper
            .create_session("op-TASK-003", "/tmp/project")
            .await
            .unwrap();

        // Should use active window despite >1 windows
        assert_eq!(wrapper.window_ref("op-TASK-003").unwrap(), "win-1");
    }

    #[tokio::test]
    async fn test_cmux_create_session_window_policy() {
        // Window policy → always creates a new window
        let wrapper = make_wrapper(MockCmuxClient::new(), CmuxPlacementPolicy::Window);

        wrapper
            .create_session("op-TASK-004", "/tmp/project")
            .await
            .unwrap();

        // Should have created a new window even with only 1 window
        let win_ref = wrapper.window_ref("op-TASK-004").unwrap();
        assert_ne!(win_ref, "win-1");
    }

    #[tokio::test]
    async fn test_cmux_create_session_duplicate_rejected() {
        let wrapper = make_wrapper(MockCmuxClient::new(), CmuxPlacementPolicy::Auto);

        wrapper
            .create_session("op-TASK-005", "/tmp/project")
            .await
            .unwrap();
        let err = wrapper
            .create_session("op-TASK-005", "/tmp/project")
            .await
            .unwrap_err();
        assert!(matches!(err, SessionError::SessionExists(_)));
    }

    #[tokio::test]
    async fn test_cmux_send_command() {
        let client = MockCmuxClient::new();
        let client_arc: Arc<MockCmuxClient> = Arc::new(client);
        let config = SessionsCmuxConfig {
            binary_path: "/mock/cmux".to_string(),
            require_in_cmux: true,
            placement: CmuxPlacementPolicy::Auto,
        };
        let wrapper = CmuxWrapper::new(client_arc.clone(), &config);

        wrapper
            .create_session("op-TASK-006", "/tmp/project")
            .await
            .unwrap();
        wrapper
            .send_command("op-TASK-006", "echo hello")
            .await
            .unwrap();

        let texts = client_arc.sent_texts();
        assert_eq!(texts.len(), 1);
        assert_eq!(texts[0].1, "echo hello");
    }

    #[tokio::test]
    async fn test_cmux_send_command_not_found() {
        let wrapper = make_wrapper(MockCmuxClient::new(), CmuxPlacementPolicy::Auto);
        let err = wrapper
            .send_command("nonexistent", "echo hello")
            .await
            .unwrap_err();
        assert!(matches!(err, SessionError::SessionNotFound(_)));
    }

    #[tokio::test]
    async fn test_cmux_kill_session() {
        let wrapper = make_wrapper(MockCmuxClient::new(), CmuxPlacementPolicy::Auto);

        wrapper
            .create_session("op-TASK-007", "/tmp/project")
            .await
            .unwrap();
        assert!(wrapper.session_exists("op-TASK-007").await.unwrap());

        wrapper.kill_session("op-TASK-007").await.unwrap();
        assert!(!wrapper.session_exists("op-TASK-007").await.unwrap());
    }

    #[tokio::test]
    async fn test_cmux_kill_session_not_found() {
        let wrapper = make_wrapper(MockCmuxClient::new(), CmuxPlacementPolicy::Auto);
        let err = wrapper.kill_session("nonexistent").await.unwrap_err();
        assert!(matches!(err, SessionError::SessionNotFound(_)));
    }

    #[tokio::test]
    async fn test_cmux_focus_session() {
        let wrapper = make_wrapper(MockCmuxClient::new(), CmuxPlacementPolicy::Auto);

        wrapper
            .create_session("op-TASK-008", "/tmp/project")
            .await
            .unwrap();
        assert!(wrapper.focus_session("op-TASK-008").await.is_ok());
    }

    #[tokio::test]
    async fn test_cmux_focus_session_not_found() {
        let wrapper = make_wrapper(MockCmuxClient::new(), CmuxPlacementPolicy::Auto);
        let err = wrapper.focus_session("nonexistent").await.unwrap_err();
        assert!(matches!(err, SessionError::SessionNotFound(_)));
    }

    #[tokio::test]
    async fn test_cmux_capture_content() {
        let client = MockCmuxClient::new();
        let client_arc: Arc<MockCmuxClient> = Arc::new(client);
        let config = SessionsCmuxConfig {
            binary_path: "/mock/cmux".to_string(),
            require_in_cmux: true,
            placement: CmuxPlacementPolicy::Auto,
        };
        let wrapper = CmuxWrapper::new(client_arc.clone(), &config);

        wrapper
            .create_session("op-TASK-009", "/tmp/project")
            .await
            .unwrap();

        // Set content on the workspace
        let ws_ref = wrapper.workspace_ref("op-TASK-009").unwrap();
        client_arc.set_screen_content(&ws_ref, "$ claude --prompt 'test'\n> ");

        let content = wrapper.capture_content("op-TASK-009").unwrap();
        assert!(content.contains("claude"));
    }

    #[tokio::test]
    async fn test_cmux_list_sessions() {
        let wrapper = make_wrapper(MockCmuxClient::new(), CmuxPlacementPolicy::Auto);

        wrapper
            .create_session("op-TASK-010", "/tmp/a")
            .await
            .unwrap();
        wrapper
            .create_session("op-FEAT-011", "/tmp/b")
            .await
            .unwrap();

        // List all
        let all = wrapper.list_sessions(None).await.unwrap();
        assert_eq!(all.len(), 2);

        // List with prefix
        let tasks = wrapper.list_sessions(Some("op-TASK")).await.unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].name, "op-TASK-010");

        // Verify wrapper type
        assert_eq!(tasks[0].wrapper_type, WrapperType::Cmux);
    }
}

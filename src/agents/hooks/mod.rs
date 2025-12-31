//! Hook management for LLM CLI tools
//!
//! This module handles installing and monitoring hooks for tools that support them
//! (Claude and Gemini). Hooks provide faster, more accurate detection of when an
//! agent has finished responding and is waiting for input.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Errors that can occur during hook operations
#[derive(Debug, Error)]
pub enum HookError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Hook installation failed: {0}")]
    InstallFailed(String),
    #[error("Unsupported tool: {0}")]
    UnsupportedTool(String),
}

/// Signal file written by hooks when agent stops/completes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookSignal {
    /// Event type (e.g., "stop", "awaiting")
    pub event: String,
    /// Unix timestamp when signal was created
    pub timestamp: u64,
    /// Session ID of the agent
    pub session_id: String,
}

/// Manages hook installation and signal monitoring for LLM tools
#[derive(Debug, Clone)]
pub struct HookManager {
    /// Directory where signal files are written
    signal_dir: PathBuf,
}

impl Default for HookManager {
    fn default() -> Self {
        Self::new()
    }
}

impl HookManager {
    /// Create a new HookManager with default signal directory
    pub fn new() -> Self {
        Self {
            signal_dir: PathBuf::from("/tmp/operator-signals"),
        }
    }

    /// Create a new HookManager with a custom signal directory
    pub fn with_signal_dir(signal_dir: PathBuf) -> Self {
        Self { signal_dir }
    }

    /// Ensure the signal directory exists
    pub fn ensure_signal_dir(&self) -> Result<(), HookError> {
        if !self.signal_dir.exists() {
            fs::create_dir_all(&self.signal_dir)?;
        }
        Ok(())
    }

    /// Check if a hook signal exists for the given session
    pub fn check_hook_signal(&self, session_id: &str) -> Option<HookSignal> {
        let signal_path = self.signal_dir.join(format!("{}.signal", session_id));
        if signal_path.exists() {
            let content = fs::read_to_string(&signal_path).ok()?;
            serde_json::from_str(&content).ok()
        } else {
            None
        }
    }

    /// Clear the hook signal for a session (used when resuming)
    pub fn clear_signal(&self, session_id: &str) -> Result<(), HookError> {
        let signal_path = self.signal_dir.join(format!("{}.signal", session_id));
        if signal_path.exists() {
            fs::remove_file(&signal_path)?;
        }
        Ok(())
    }

    /// Get the path to a signal file for a session
    pub fn signal_path(&self, session_id: &str) -> PathBuf {
        self.signal_dir.join(format!("{}.signal", session_id))
    }

    /// Generate the hook script content for Claude's Stop hook
    pub fn generate_claude_hook_script(&self) -> String {
        let signal_dir = self.signal_dir.display();
        format!(
            r#"#!/bin/bash
# Operator hook for Claude Code - fires on Stop event
# Receives JSON via stdin with session_id, hook_event_name, etc.

INPUT=$(cat)
SESSION_ID=$(echo "$INPUT" | jq -r '.session_id')
TIMESTAMP=$(date +%s)

mkdir -p "{signal_dir}"
echo "{{"\"event\":\"stop\",\"timestamp\":$TIMESTAMP,\"session_id\":\"$SESSION_ID\"}}" \
  > "{signal_dir}/$SESSION_ID.signal"
"#,
            signal_dir = signal_dir
        )
    }

    /// Generate the hook script content for Gemini's AfterAgent hook
    pub fn generate_gemini_hook_script(&self) -> String {
        let signal_dir = self.signal_dir.display();
        format!(
            r#"#!/bin/bash
# Operator hook for Gemini CLI - fires on AfterAgent event
# Receives JSON via stdin with session_id, hook_event_name, etc.

INPUT=$(cat)
SESSION_ID=$(echo "$INPUT" | jq -r '.session_id')
TIMESTAMP=$(date +%s)

mkdir -p "{signal_dir}"
echo "{{"\"event\":\"stop\",\"timestamp\":$TIMESTAMP,\"session_id\":\"$SESSION_ID\"}}" \
  > "{signal_dir}/$SESSION_ID.signal"
"#,
            signal_dir = signal_dir
        )
    }

    /// Install hooks for a specific tool if supported
    pub fn install_hooks(&self, tool_name: &str, script_path: &str) -> Result<(), HookError> {
        // Ensure signal directory exists
        self.ensure_signal_dir()?;

        // Expand ~ in path
        let script_path = expand_tilde(script_path);

        // Create parent directory if needed
        if let Some(parent) = script_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Generate appropriate script based on tool
        let script_content = match tool_name {
            "claude" => self.generate_claude_hook_script(),
            "gemini" => self.generate_gemini_hook_script(),
            _ => return Err(HookError::UnsupportedTool(tool_name.to_string())),
        };

        // Write the script
        fs::write(&script_path, script_content)?;

        // Make it executable on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&script_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&script_path, perms)?;
        }

        tracing::info!(
            tool = tool_name,
            path = %script_path.display(),
            "Installed operator hook script"
        );

        Ok(())
    }

    /// Check if hooks are installed for a tool
    pub fn is_hook_installed(&self, script_path: &str) -> bool {
        expand_tilde(script_path).exists()
    }
}

/// Expand ~ to home directory in a path
fn expand_tilde(path: &str) -> PathBuf {
    if let Some(stripped) = path.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(stripped);
        }
    } else if path == "~" {
        if let Some(home) = dirs::home_dir() {
            return home;
        }
    }
    PathBuf::from(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_hook_manager_new() {
        let manager = HookManager::new();
        assert_eq!(manager.signal_dir, PathBuf::from("/tmp/operator-signals"));
    }

    #[test]
    fn test_check_hook_signal_not_exists() {
        let temp_dir = TempDir::new().unwrap();
        let manager = HookManager::with_signal_dir(temp_dir.path().to_path_buf());

        let signal = manager.check_hook_signal("nonexistent-session");
        assert!(signal.is_none());
    }

    #[test]
    fn test_check_hook_signal_exists() {
        let temp_dir = TempDir::new().unwrap();
        let manager = HookManager::with_signal_dir(temp_dir.path().to_path_buf());

        // Create a signal file
        let signal_content =
            r#"{"event":"stop","timestamp":1234567890,"session_id":"test-session"}"#;
        fs::write(temp_dir.path().join("test-session.signal"), signal_content).unwrap();

        let signal = manager.check_hook_signal("test-session");
        assert!(signal.is_some());
        let signal = signal.unwrap();
        assert_eq!(signal.event, "stop");
        assert_eq!(signal.session_id, "test-session");
        assert_eq!(signal.timestamp, 1234567890);
    }

    #[test]
    fn test_clear_signal() {
        let temp_dir = TempDir::new().unwrap();
        let manager = HookManager::with_signal_dir(temp_dir.path().to_path_buf());

        // Create a signal file
        let signal_path = temp_dir.path().join("test-session.signal");
        fs::write(&signal_path, "{}").unwrap();
        assert!(signal_path.exists());

        // Clear it
        manager.clear_signal("test-session").unwrap();
        assert!(!signal_path.exists());
    }

    #[test]
    fn test_clear_signal_not_exists() {
        let temp_dir = TempDir::new().unwrap();
        let manager = HookManager::with_signal_dir(temp_dir.path().to_path_buf());

        // Should not error if file doesn't exist
        let result = manager.clear_signal("nonexistent");
        assert!(result.is_ok());
    }

    #[test]
    fn test_generate_claude_hook_script() {
        let manager = HookManager::new();
        let script = manager.generate_claude_hook_script();

        assert!(script.contains("#!/bin/bash"));
        assert!(script.contains("Operator hook for Claude Code"));
        assert!(script.contains("jq -r '.session_id'"));
        assert!(script.contains("/tmp/operator-signals"));
    }

    #[test]
    fn test_generate_gemini_hook_script() {
        let manager = HookManager::new();
        let script = manager.generate_gemini_hook_script();

        assert!(script.contains("#!/bin/bash"));
        assert!(script.contains("Operator hook for Gemini CLI"));
        assert!(script.contains("jq -r '.session_id'"));
        assert!(script.contains("/tmp/operator-signals"));
    }

    #[test]
    fn test_install_hooks_claude() {
        let temp_dir = TempDir::new().unwrap();
        let manager = HookManager::with_signal_dir(temp_dir.path().to_path_buf());

        let script_path = temp_dir.path().join("hooks/operator-stop.sh");
        let result = manager.install_hooks("claude", script_path.to_str().unwrap());

        assert!(result.is_ok());
        assert!(script_path.exists());

        let content = fs::read_to_string(&script_path).unwrap();
        assert!(content.contains("Claude Code"));
    }

    #[test]
    fn test_install_hooks_unsupported() {
        let temp_dir = TempDir::new().unwrap();
        let manager = HookManager::with_signal_dir(temp_dir.path().to_path_buf());

        let script_path = temp_dir.path().join("hooks/operator-stop.sh");
        let result = manager.install_hooks("codex", script_path.to_str().unwrap());

        assert!(result.is_err());
        match result {
            Err(HookError::UnsupportedTool(tool)) => assert_eq!(tool, "codex"),
            _ => panic!("Expected UnsupportedTool error"),
        }
    }

    #[test]
    fn test_is_hook_installed() {
        let temp_dir = TempDir::new().unwrap();
        let manager = HookManager::with_signal_dir(temp_dir.path().to_path_buf());

        let script_path = temp_dir.path().join("test-script.sh");
        assert!(!manager.is_hook_installed(script_path.to_str().unwrap()));

        fs::write(&script_path, "#!/bin/bash").unwrap();
        assert!(manager.is_hook_installed(script_path.to_str().unwrap()));
    }
}

//! Activity detection implementations for different backends.
//!
//! Activity detectors determine when an agent session is idle (waiting for input)
//! or has resumed work. Multiple implementations exist for different detection strategies:
//!
//! - `LlmHookDetector`: Uses Claude/Gemini hooks - fastest and most reliable
//! - `TmuxActivityDetector`: Uses tmux silence flags and content patterns - fallback
//! - `MockActivityDetector`: For testing

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::agents::hooks::HookManager;
use crate::agents::idle_detector::IdleDetector;
use crate::agents::terminal_wrapper::{ActivityConfig, ActivityDetector, SessionError};
use crate::agents::tmux::TmuxClient;

/// Activity detector using LLM tool hooks (Claude, Gemini)
///
/// This is the preferred detector when the LLM tool supports hooks.
/// It uses signal files written by the tool's hook system.
pub struct LlmHookDetector {
    hook_manager: HookManager,
    /// Track the last seen event per session to detect transitions
    last_events: Mutex<HashMap<String, String>>,
}

impl LlmHookDetector {
    /// Create a new hook-based activity detector
    pub fn new(hook_manager: HookManager) -> Self {
        Self {
            hook_manager,
            last_events: Mutex::new(HashMap::new()),
        }
    }

    /// Create with default hook manager
    pub fn default_manager() -> Self {
        Self::new(HookManager::new())
    }
}

impl ActivityDetector for LlmHookDetector {
    fn is_idle(&self, session_id: &str) -> Result<bool, SessionError> {
        if let Some(signal) = self.hook_manager.check_hook_signal(session_id) {
            // Update last seen event
            if let Ok(mut events) = self.last_events.lock() {
                events.insert(session_id.to_string(), signal.event.clone());
            }
            Ok(signal.event == "stop")
        } else {
            // No signal - assume not idle (could be still running)
            Ok(false)
        }
    }

    fn has_resumed(&self, session_id: &str) -> Result<bool, SessionError> {
        if let Some(signal) = self.hook_manager.check_hook_signal(session_id) {
            // Check if event changed from "stop" to something else
            if let Ok(events) = self.last_events.lock() {
                if let Some(last) = events.get(session_id) {
                    if last == "stop" && signal.event != "stop" {
                        return Ok(true);
                    }
                }
            }
            // Also check for explicit "start" event
            Ok(signal.event == "start")
        } else {
            // Signal was cleared - could indicate resumed (signal consumed)
            if let Ok(events) = self.last_events.lock() {
                if events.get(session_id).map(|e| e == "stop").unwrap_or(false) {
                    // Was stopped, signal gone = likely resumed
                    return Ok(true);
                }
            }
            Ok(false)
        }
    }

    fn configure(&self, _session_id: &str, _config: &ActivityConfig) -> Result<(), SessionError> {
        // Hook-based detection doesn't need configuration
        Ok(())
    }

    fn clear(&self, session_id: &str) -> Result<(), SessionError> {
        // Clear the hook signal and our tracking
        let _ = self.hook_manager.clear_signal(session_id);
        if let Ok(mut events) = self.last_events.lock() {
            events.remove(session_id);
        }
        Ok(())
    }
}

/// Activity detector using tmux silence monitoring and content patterns
///
/// Used as a fallback when LLM hooks are not available.
/// Uses tmux's monitor-silence feature and pattern matching on captured content.
pub struct TmuxActivityDetector {
    tmux: Arc<dyn TmuxClient>,
    idle_detector: IdleDetector,
    /// Content hashes for change detection
    content_hashes: Mutex<HashMap<String, String>>,
}

impl TmuxActivityDetector {
    /// Create a new tmux-based activity detector
    pub fn new(tmux: Arc<dyn TmuxClient>, idle_detector: IdleDetector) -> Self {
        Self {
            tmux,
            idle_detector,
            content_hashes: Mutex::new(HashMap::new()),
        }
    }

    fn compute_hash(content: &str) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

impl ActivityDetector for TmuxActivityDetector {
    fn is_idle(&self, session_id: &str) -> Result<bool, SessionError> {
        // First check tmux silence flag
        let silence_flag = self
            .tmux
            .check_silence_flag(session_id)
            .map_err(|e| SessionError::CommandFailed(e.to_string()))?;

        if !silence_flag {
            return Ok(false); // Still active
        }

        // Silence flag set - check content patterns
        if let Ok(content) = self.tmux.capture_pane(session_id, false) {
            // Use idle detector patterns (default to "claude" if no tool specified)
            if self.idle_detector.is_idle("claude", &content) {
                return Ok(true);
            }
        }

        // Silence flag set but no idle pattern matched - still consider idle
        Ok(true)
    }

    fn has_resumed(&self, session_id: &str) -> Result<bool, SessionError> {
        // Check if content has changed since last check
        if let Ok(content) = self.tmux.capture_pane(session_id, false) {
            let new_hash = Self::compute_hash(&content);

            if let Ok(mut hashes) = self.content_hashes.lock() {
                if let Some(old_hash) = hashes.get(session_id) {
                    if old_hash != &new_hash {
                        // Content changed - resumed
                        hashes.insert(session_id.to_string(), new_hash);
                        return Ok(true);
                    }
                } else {
                    // First check - store hash
                    hashes.insert(session_id.to_string(), new_hash);
                }
            }
        }

        Ok(false)
    }

    fn configure(&self, session_id: &str, config: &ActivityConfig) -> Result<(), SessionError> {
        self.tmux
            .set_monitor_silence(session_id, config.silence_threshold_secs)
            .map_err(|e| SessionError::CommandFailed(e.to_string()))
    }

    fn clear(&self, session_id: &str) -> Result<(), SessionError> {
        // Clear content hash tracking
        if let Ok(mut hashes) = self.content_hashes.lock() {
            hashes.remove(session_id);
        }

        // Reset silence flag
        let _ = self.tmux.reset_silence_flag(session_id);

        Ok(())
    }
}

/// Mock activity detector for testing
pub struct MockActivityDetector {
    /// Map of session_id -> is_idle
    idle_states: Mutex<HashMap<String, bool>>,
    /// Map of session_id -> has_resumed
    resumed_states: Mutex<HashMap<String, bool>>,
}

impl Default for MockActivityDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl MockActivityDetector {
    pub fn new() -> Self {
        Self {
            idle_states: Mutex::new(HashMap::new()),
            resumed_states: Mutex::new(HashMap::new()),
        }
    }

    /// Set the idle state for a session
    pub fn set_idle(&self, session_id: &str, idle: bool) {
        if let Ok(mut states) = self.idle_states.lock() {
            states.insert(session_id.to_string(), idle);
        }
    }

    /// Set the resumed state for a session
    pub fn set_resumed(&self, session_id: &str, resumed: bool) {
        if let Ok(mut states) = self.resumed_states.lock() {
            states.insert(session_id.to_string(), resumed);
        }
    }
}

impl ActivityDetector for MockActivityDetector {
    fn is_idle(&self, session_id: &str) -> Result<bool, SessionError> {
        Ok(self
            .idle_states
            .lock()
            .ok()
            .and_then(|s| s.get(session_id).copied())
            .unwrap_or(false))
    }

    fn has_resumed(&self, session_id: &str) -> Result<bool, SessionError> {
        Ok(self
            .resumed_states
            .lock()
            .ok()
            .and_then(|s| s.get(session_id).copied())
            .unwrap_or(false))
    }

    fn configure(&self, _session_id: &str, _config: &ActivityConfig) -> Result<(), SessionError> {
        Ok(())
    }

    fn clear(&self, session_id: &str) -> Result<(), SessionError> {
        if let Ok(mut states) = self.idle_states.lock() {
            states.remove(session_id);
        }
        if let Ok(mut states) = self.resumed_states.lock() {
            states.remove(session_id);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_activity_detector_defaults_to_not_idle() {
        let detector = MockActivityDetector::new();
        assert!(!detector.is_idle("test-session").unwrap());
        assert!(!detector.has_resumed("test-session").unwrap());
    }

    #[test]
    fn test_mock_activity_detector_set_idle() {
        let detector = MockActivityDetector::new();
        detector.set_idle("test-session", true);
        assert!(detector.is_idle("test-session").unwrap());
    }

    #[test]
    fn test_mock_activity_detector_set_resumed() {
        let detector = MockActivityDetector::new();
        detector.set_resumed("test-session", true);
        assert!(detector.has_resumed("test-session").unwrap());
    }

    #[test]
    fn test_mock_activity_detector_clear() {
        let detector = MockActivityDetector::new();
        detector.set_idle("test-session", true);
        detector.set_resumed("test-session", true);

        detector.clear("test-session").unwrap();

        assert!(!detector.is_idle("test-session").unwrap());
        assert!(!detector.has_resumed("test-session").unwrap());
    }

    #[test]
    fn test_llm_hook_detector_no_signal() {
        // Create with a custom signal dir that doesn't exist
        let hook_manager = HookManager::with_signal_dir("/tmp/nonexistent-test-signals".into());
        let detector = LlmHookDetector::new(hook_manager);

        // No signal file = not idle
        assert!(!detector.is_idle("test-session").unwrap());
    }
}

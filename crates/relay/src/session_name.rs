//! Provider-agnostic session name source for the relay channel binary.
//!
//! Each LLM tool has its own convention for session naming. This trait abstracts
//! over those differences so the channel binary can support Claude, Codex, and others
//! with the same registration flow.

use async_trait::async_trait;

/// Provides the name for a relay peer registration and watches for name changes.
#[async_trait]
pub trait SessionNameSource: Send + Sync + 'static {
    /// Name to use at registration time. Returns `None` if not yet known.
    fn initial_name(&self) -> Option<String>;

    /// Watch for name changes and call `on_name` when a new name is detected.
    /// Returns after setting up the watcher (does not block indefinitely).
    async fn watch<F>(&self, on_name: F) -> anyhow::Result<()>
    where
        F: Fn(String) + Send + 'static;
}

// ── ExplicitSessionNameSource ─────────────────────────────────────────────────

/// A fixed name assigned by the caller (e.g., operator assigning a ticket ID).
/// Used for operator-managed agents where the name is known at launch time.
pub struct ExplicitSessionNameSource {
    name: String,
}

impl ExplicitSessionNameSource {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

#[async_trait]
impl SessionNameSource for ExplicitSessionNameSource {
    fn initial_name(&self) -> Option<String> {
        Some(self.name.clone())
    }

    async fn watch<F>(&self, _on_name: F) -> anyhow::Result<()>
    where
        F: Fn(String) + Send + 'static,
    {
        // Explicit names are fixed; no file to watch
        Ok(())
    }
}

// ── ClaudeSessionNameSource ───────────────────────────────────────────────────

/// Watches `~/.claude/sessions/{ppid}.json` for Claude Code's `/rename` command output.
///
/// When the user runs `/rename` in Claude Code, it writes `{"name": "..."}` to this file.
/// The watcher detects the change and calls `on_name` with the sanitized name, keeping
/// the hub registry in sync.
pub struct ClaudeSessionNameSource {
    session_file: std::path::PathBuf,
}

impl ClaudeSessionNameSource {
    /// Derive the session file path from the current process's parent PID.
    #[cfg(unix)]
    pub fn for_current_process() -> Self {
        let ppid = libc_ppid();
        let home = dirs::home_dir().unwrap_or_default();
        Self {
            session_file: home
                .join(".claude")
                .join("sessions")
                .join(format!("{ppid}.json")),
        }
    }

    pub fn from_path(session_file: std::path::PathBuf) -> Self {
        Self { session_file }
    }

    fn read_name(&self) -> Option<String> {
        let content = std::fs::read_to_string(&self.session_file).ok()?;
        let value: serde_json::Value = serde_json::from_str(&content).ok()?;
        let name = value.get("name")?.as_str()?;
        let sanitized = sanitize_session_name(name);
        if sanitized.is_empty() {
            None
        } else {
            Some(sanitized)
        }
    }
}

#[async_trait]
impl SessionNameSource for ClaudeSessionNameSource {
    fn initial_name(&self) -> Option<String> {
        self.read_name()
    }

    async fn watch<F>(&self, on_name: F) -> anyhow::Result<()>
    where
        F: Fn(String) + Send + 'static,
    {
        use notify::{Event, RecursiveMode, Watcher};
        use std::time::Duration;
        use tokio::sync::mpsc;

        let session_file = self.session_file.clone();
        let watch_dir = session_file
            .parent()
            .ok_or_else(|| anyhow::anyhow!("Session file has no parent directory"))?
            .to_path_buf();

        let (tx, mut rx) = mpsc::channel::<Event>(16);

        let mut watcher = notify::recommended_watcher(move |res: notify::Result<Event>| {
            if let Ok(event) = res {
                let _ = tx.blocking_send(event);
            }
        })?;

        if watch_dir.exists() {
            watcher.watch(&watch_dir, RecursiveMode::NonRecursive)?;
        }

        tokio::spawn(async move {
            let _watcher = watcher; // Keep alive for the duration of the spawn
            let mut debounce_pending = false;

            loop {
                tokio::select! {
                    Some(_event) = rx.recv() => {
                        debounce_pending = true;
                    }
                    () = tokio::time::sleep(Duration::from_millis(50)), if debounce_pending => {
                        debounce_pending = false;
                        if let Ok(content) = std::fs::read_to_string(&session_file) {
                            if let Ok(value) = serde_json::from_str::<serde_json::Value>(&content) {
                                if let Some(name) = value.get("name").and_then(|n| n.as_str()) {
                                    let sanitized = sanitize_session_name(name);
                                    if !sanitized.is_empty() {
                                        on_name(sanitized);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        });

        Ok(())
    }
}

/// Sanitize a session name: keep alphanumeric, `.`, `-`, `_`; truncate to 64 chars.
/// Matches claude-relay's `sanitizeSessionName` in `src/identity.ts`.
pub fn sanitize_session_name(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_alphanumeric() || *c == '.' || *c == '-' || *c == '_')
        .take(64)
        .collect()
}

// Platform shim for PPID
#[cfg(unix)]
fn libc_ppid() -> u32 {
    // Use std::process for current PID; PPID requires a syscall.
    // On Linux/macOS, read from /proc/self/status or use getppid() via libc.
    // Since we don't depend on the libc crate, read from /proc/self/status on Linux
    // and fall back to 0 on macOS (where the binary reads its own parent from the OS).
    #[cfg(target_os = "linux")]
    {
        if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
            for line in status.lines() {
                if let Some(rest) = line.strip_prefix("PPid:\t") {
                    if let Ok(ppid) = rest.trim().parse::<u32>() {
                        return ppid;
                    }
                }
            }
        }
        0
    }
    #[cfg(not(target_os = "linux"))]
    {
        // macOS: use sysctl or just return 0 as a safe fallback
        // The relay-channel binary is the primary user of ClaudeSessionNameSource;
        // operator itself uses ExplicitSessionNameSource.
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_keeps_valid_chars() {
        assert_eq!(
            sanitize_session_name("my-project.v2_ok"),
            "my-project.v2_ok"
        );
    }

    #[test]
    fn test_sanitize_strips_spaces_and_slashes() {
        assert_eq!(sanitize_session_name("my project/name"), "myprojectname");
    }

    #[test]
    fn test_sanitize_truncates_to_64() {
        let long = "a".repeat(100);
        assert_eq!(sanitize_session_name(&long).len(), 64);
    }

    #[test]
    fn test_sanitize_empty_input() {
        assert_eq!(sanitize_session_name(""), "");
    }

    #[test]
    fn test_explicit_source_initial_name() {
        let src = ExplicitSessionNameSource::new("FEAT-123");
        assert_eq!(src.initial_name(), Some("FEAT-123".to_string()));
    }

    #[tokio::test]
    async fn test_explicit_source_watch_is_noop() {
        let src = ExplicitSessionNameSource::new("FEAT-123");
        let result = src.watch(|_| {}).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_claude_source_reads_name_from_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("session.json");
        std::fs::write(&path, r#"{"name":"my-session"}"#).unwrap();
        let src = ClaudeSessionNameSource::from_path(path);
        assert_eq!(src.initial_name(), Some("my-session".to_string()));
    }

    #[test]
    fn test_claude_source_returns_none_for_missing_file() {
        let src = ClaudeSessionNameSource::from_path("/tmp/nonexistent_relay_session.json".into());
        assert_eq!(src.initial_name(), None);
    }

    #[test]
    fn test_claude_source_sanitizes_name() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("s.json");
        std::fs::write(&path, r#"{"name":"my project/v2"}"#).unwrap();
        let src = ClaudeSessionNameSource::from_path(path);
        assert_eq!(src.initial_name(), Some("myprojectv2".to_string()));
    }
}

//! ACP session registry — maps `SessionId` to operator tickets.
//!
//! When an editor calls `session/new`, [`SessionRegistry::create_or_attach`]
//! either attaches to an existing in-progress ACP ticket (if exactly one
//! matches the editor's cwd) or writes a fresh `ACP-{short}.md` into
//! `.tickets/in-progress/` and registers the session against it.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use agent_client_protocol::schema::SessionId;
use anyhow::{anyhow, Context, Result};
use tokio::sync::oneshot;

use crate::config::Config;

/// One live ACP session: an editor-spawned conversation backed by an
/// `ACP-*.md` ticket in the in-progress directory.
#[derive(Debug, Clone)]
pub struct AcpSession {
    /// Reserved for future ticket-completion bookkeeping (mark the ACP
    /// ticket done when the editor disconnects).
    #[allow(dead_code)]
    pub session_id: SessionId,
    /// Reserved for future ticket-update logic.
    #[allow(dead_code)]
    pub ticket_path: PathBuf,
    pub working_directory: PathBuf,
}

/// Thread-safe registry of live ACP sessions. Handler closures share this
/// across `Agent.builder()` registrations via `Arc`.
#[derive(Debug, Default, Clone)]
pub struct SessionRegistry {
    sessions: Arc<Mutex<HashMap<SessionId, AcpSession>>>,
    cancel_senders: Arc<Mutex<HashMap<SessionId, oneshot::Sender<()>>>>,
}

impl SessionRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Create or attach an ACP session for `cwd`.
    ///
    /// Behavior:
    /// 1. Reject if the registry already holds `config.acp.max_concurrent_sessions`.
    /// 2. Canonicalize `cwd` (falling back to the literal path on error).
    /// 3. If exactly one `ACP-*.md` ticket in `in-progress/` has matching
    ///    frontmatter `cwd`, attach to it (do not write a new ticket).
    /// 4. Otherwise write a fresh `ACP-{session-short}.md` to `in-progress/`.
    pub fn create_or_attach(&self, config: &Config, cwd: &Path) -> Result<SessionId> {
        {
            let active = self.sessions.lock().unwrap().len();
            if active >= config.acp.max_concurrent_sessions {
                return Err(anyhow!(
                    "ACP session limit reached: {active}/{}",
                    config.acp.max_concurrent_sessions
                ));
            }
        }

        let canonical_cwd = std::fs::canonicalize(cwd).unwrap_or_else(|_| cwd.to_path_buf());
        let in_progress = config.tickets_path().join("in-progress");
        let session_id = SessionId::from(uuid::Uuid::new_v4().to_string());

        let ticket_path = match find_matching_acp_ticket(&in_progress, &canonical_cwd) {
            Some(path) => path,
            None => write_new_acp_ticket(&in_progress, &session_id, &canonical_cwd)?,
        };

        let session = AcpSession {
            session_id: session_id.clone(),
            ticket_path,
            working_directory: canonical_cwd,
        };
        self.sessions
            .lock()
            .unwrap()
            .insert(session_id.clone(), session);
        Ok(session_id)
    }

    /// Number of live ACP sessions. Used by `AcpAgentServer::active_sessions`
    /// once the registry is shared with the dashboard.
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.sessions.lock().unwrap().len()
    }

    /// Reserved for the same future as `len`.
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.sessions.lock().unwrap().is_empty()
    }

    /// Return a clone of the session matching `id`, if any.
    pub fn get(&self, id: &SessionId) -> Option<AcpSession> {
        self.sessions.lock().unwrap().get(id).cloned()
    }

    /// Store a cancel sender for an in-flight prompt. The cancel notification
    /// handler calls [`take_cancel_sender`] to fire it.
    pub fn register_cancel_sender(&self, id: &SessionId, tx: oneshot::Sender<()>) {
        self.cancel_senders.lock().unwrap().insert(id.clone(), tx);
    }

    /// Remove and return the cancel sender for `id`, if one is registered.
    /// Returns `None` if the prompt already completed (sender was cleaned up)
    /// or if no prompt is in flight for this session.
    pub fn take_cancel_sender(&self, id: &SessionId) -> Option<oneshot::Sender<()>> {
        self.cancel_senders.lock().unwrap().remove(id)
    }
}

/// Scan `in_progress` for `ACP-*.md` files whose frontmatter `cwd` matches
/// `target`. Returns `Some(path)` iff exactly one matches; otherwise `None`.
fn find_matching_acp_ticket(in_progress: &Path, target: &Path) -> Option<PathBuf> {
    let entries = std::fs::read_dir(in_progress).ok()?;
    let matches: Vec<PathBuf> = entries
        .filter_map(std::result::Result::ok)
        .map(|e| e.path())
        .filter(|p| {
            p.extension().and_then(|e| e.to_str()) == Some("md")
                && p.file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n.starts_with("ACP-"))
        })
        .filter(|p| ticket_cwd_matches(p, target))
        .collect();
    if matches.len() == 1 {
        Some(matches.into_iter().next().unwrap())
    } else {
        None
    }
}

/// True iff the file at `path` has YAML frontmatter with a `cwd` field that
/// equals `target` after path comparison.
fn ticket_cwd_matches(path: &Path, target: &Path) -> bool {
    let Ok(content) = std::fs::read_to_string(path) else {
        return false;
    };
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return false;
    }
    let after_open = &trimmed[3..];
    let Some(end_idx) = after_open.find("\n---") else {
        return false;
    };
    let yaml_str = after_open[..end_idx].trim();
    let Ok(fm) = serde_yaml::from_str::<HashMap<String, serde_yaml::Value>>(yaml_str) else {
        return false;
    };
    fm.get("cwd")
        .and_then(serde_yaml::Value::as_str)
        .is_some_and(|s| Path::new(s) == target)
}

fn write_new_acp_ticket(in_progress: &Path, session_id: &SessionId, cwd: &Path) -> Result<PathBuf> {
    std::fs::create_dir_all(in_progress)
        .with_context(|| format!("create in-progress dir {}", in_progress.display()))?;
    let short = session_short(session_id);
    let filename = format!("ACP-{short}.md");
    let path = in_progress.join(filename);
    let now = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let cwd_str = cwd.display().to_string();
    let project = cwd.file_name().and_then(|n| n.to_str()).unwrap_or("global");
    let body = format!(
        "---\nid: ACP-{short}\nstatus: in-progress\nkind: acp\ncreated: {now}\nproject: {project}\ncwd: {cwd_str}\n---\n\n# ACP session from {cwd_str}\n"
    );
    std::fs::write(&path, body).with_context(|| format!("write ACP ticket {}", path.display()))?;
    Ok(path)
}

/// First 8 hex chars of the session UUID — short enough for a filename, long
/// enough that random collisions inside one in-progress dir are negligible.
fn session_short(session_id: &SessionId) -> String {
    session_id
        .0
        .chars()
        .filter(char::is_ascii_hexdigit)
        .take(8)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Once;

    static INIT: Once = Once::new();

    fn test_config(tickets_dir: &Path, max_sessions: usize) -> Config {
        INIT.call_once(|| {});
        let mut config = Config::default();
        config.paths.tickets = tickets_dir.to_string_lossy().into_owned();
        config.acp.max_concurrent_sessions = max_sessions;
        config
    }

    fn write_acp_ticket(in_progress: &Path, name: &str, cwd: &Path) {
        std::fs::create_dir_all(in_progress).unwrap();
        let body = format!(
            "---\nid: ACP-{name}\nstatus: in-progress\nkind: acp\ncreated: 2026-05-17\nproject: test\ncwd: {}\n---\n\n# pre-existing\n",
            cwd.display()
        );
        std::fs::write(in_progress.join(format!("ACP-{name}.md")), body).unwrap();
    }

    #[test]
    fn test_attaches_when_one_acp_ticket_matches_cwd() {
        let tickets = tempfile::TempDir::new().unwrap();
        let cwd = tempfile::TempDir::new().unwrap();
        let canon_cwd = std::fs::canonicalize(cwd.path()).unwrap();
        let in_progress = tickets.path().join("in-progress");
        write_acp_ticket(&in_progress, "abcd1234", &canon_cwd);

        let config = test_config(tickets.path(), 4);
        let registry = SessionRegistry::new();
        let session_id = registry.create_or_attach(&config, cwd.path()).unwrap();

        let session = registry
            .sessions
            .lock()
            .unwrap()
            .get(&session_id)
            .cloned()
            .expect("session must be registered");
        assert_eq!(session.ticket_path, in_progress.join("ACP-abcd1234.md"));
        assert_eq!(session.working_directory, canon_cwd);
        // No new file written: only the pre-seeded one exists
        let count = std::fs::read_dir(&in_progress).unwrap().count();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_creates_new_ticket_when_no_match() {
        let tickets = tempfile::TempDir::new().unwrap();
        let cwd = tempfile::TempDir::new().unwrap();
        let in_progress = tickets.path().join("in-progress");

        let config = test_config(tickets.path(), 4);
        let registry = SessionRegistry::new();
        let session_id = registry.create_or_attach(&config, cwd.path()).unwrap();

        let path = registry
            .sessions
            .lock()
            .unwrap()
            .get(&session_id)
            .unwrap()
            .ticket_path
            .clone();
        assert!(path.exists());
        assert!(path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .starts_with("ACP-"));
        assert_eq!(std::fs::read_dir(&in_progress).unwrap().count(), 1);
    }

    #[test]
    fn test_creates_new_when_multiple_acp_tickets_match() {
        let tickets = tempfile::TempDir::new().unwrap();
        let cwd = tempfile::TempDir::new().unwrap();
        let canon_cwd = std::fs::canonicalize(cwd.path()).unwrap();
        let in_progress = tickets.path().join("in-progress");
        write_acp_ticket(&in_progress, "aaaaaaaa", &canon_cwd);
        write_acp_ticket(&in_progress, "bbbbbbbb", &canon_cwd);

        let config = test_config(tickets.path(), 4);
        let registry = SessionRegistry::new();
        let session_id = registry.create_or_attach(&config, cwd.path()).unwrap();

        let path = registry
            .sessions
            .lock()
            .unwrap()
            .get(&session_id)
            .unwrap()
            .ticket_path
            .clone();
        // Should be a brand-new file, not one of the two pre-seeded ones
        assert!(path.exists());
        assert_ne!(path, in_progress.join("ACP-aaaaaaaa.md"));
        assert_ne!(path, in_progress.join("ACP-bbbbbbbb.md"));
        assert_eq!(std::fs::read_dir(&in_progress).unwrap().count(), 3);
    }

    #[test]
    fn test_rejects_when_max_concurrent_sessions_reached() {
        let tickets = tempfile::TempDir::new().unwrap();
        let cwd = tempfile::TempDir::new().unwrap();
        let config = test_config(tickets.path(), 1);
        let registry = SessionRegistry::new();
        registry.create_or_attach(&config, cwd.path()).unwrap();

        let err = registry
            .create_or_attach(&config, cwd.path())
            .expect_err("second session must be rejected");
        assert!(err.to_string().contains("session limit"));
    }

    #[test]
    fn test_session_short_is_8_hex_chars() {
        let id = SessionId::from("deadbeef-cafe-1234-5678-90abcdef0000".to_string());
        let short = session_short(&id);
        assert_eq!(short.len(), 8);
        assert!(short.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_register_and_take_cancel_sender() {
        let registry = SessionRegistry::new();
        let id = SessionId::from("test-session-1".to_string());
        let (tx, _rx) = oneshot::channel();
        registry.register_cancel_sender(&id, tx);
        assert!(
            registry.take_cancel_sender(&id).is_some(),
            "first take should return the sender"
        );
    }

    #[test]
    fn test_double_take_cancel_sender_returns_none() {
        let registry = SessionRegistry::new();
        let id = SessionId::from("test-session-2".to_string());
        let (tx, _rx) = oneshot::channel();
        registry.register_cancel_sender(&id, tx);
        registry.take_cancel_sender(&id);
        assert!(
            registry.take_cancel_sender(&id).is_none(),
            "second take should return None"
        );
    }

    #[test]
    fn test_take_cancel_sender_unknown_session_returns_none() {
        let registry = SessionRegistry::new();
        let id = SessionId::from("nonexistent".to_string());
        assert!(registry.take_cancel_sender(&id).is_none());
    }
}

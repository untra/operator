#![allow(dead_code)]

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Represents a persisted agent session for recovery and auditing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub ticket_id: String,
    pub ticket_type: String,
    pub project: String,
    pub started_at: DateTime<Utc>,
    pub events: Vec<SessionEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionEvent {
    pub timestamp: DateTime<Utc>,
    pub event_type: SessionEventType,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionEventType {
    Started,
    AwaitingInput,
    Resumed,
    PrCreated,
    TicketsCreated,
    Completed,
    Failed,
    Paused,
}

impl Session {
    pub fn new(id: String, ticket_id: String, ticket_type: String, project: String) -> Self {
        let now = Utc::now();
        Self {
            id,
            ticket_id,
            ticket_type,
            project,
            started_at: now,
            events: vec![SessionEvent {
                timestamp: now,
                event_type: SessionEventType::Started,
                message: None,
            }],
        }
    }

    pub fn load(sessions_dir: &Path, id: &str) -> Result<Option<Self>> {
        let path = sessions_dir.join(format!("{}.json", id));

        if !path.exists() {
            return Ok(None);
        }

        let contents = fs::read_to_string(&path)?;
        let session: Session = serde_json::from_str(&contents)?;
        Ok(Some(session))
    }

    pub fn save(&self, sessions_dir: &Path) -> Result<()> {
        fs::create_dir_all(sessions_dir)?;
        let path = sessions_dir.join(format!("{}.json", self.id));
        let contents = serde_json::to_string_pretty(self)?;
        fs::write(path, contents)?;
        Ok(())
    }

    pub fn add_event(&mut self, event_type: SessionEventType, message: Option<String>) {
        self.events.push(SessionEvent {
            timestamp: Utc::now(),
            event_type,
            message,
        });
    }

    pub fn last_event(&self) -> Option<&SessionEvent> {
        self.events.last()
    }

    pub fn is_awaiting_input(&self) -> bool {
        matches!(
            self.last_event().map(|e| &e.event_type),
            Some(SessionEventType::AwaitingInput)
        )
    }

    pub fn is_completed(&self) -> bool {
        matches!(
            self.last_event().map(|e| &e.event_type),
            Some(SessionEventType::Completed)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_test_session() -> Session {
        Session::new(
            "test-id".to_string(),
            "TICKET-001".to_string(),
            "feature".to_string(),
            "my-project".to_string(),
        )
    }

    #[test]
    fn test_session_new_initializes_with_started_event() {
        let session = make_test_session();

        assert_eq!(session.id, "test-id");
        assert_eq!(session.ticket_id, "TICKET-001");
        assert_eq!(session.ticket_type, "feature");
        assert_eq!(session.project, "my-project");
        assert_eq!(session.events.len(), 1);

        let first_event = &session.events[0];
        assert!(matches!(first_event.event_type, SessionEventType::Started));
        assert!(first_event.message.is_none());
    }

    #[test]
    fn test_add_event_appends_to_events() {
        let mut session = make_test_session();
        assert_eq!(session.events.len(), 1);

        session.add_event(SessionEventType::AwaitingInput, Some("waiting".to_string()));
        assert_eq!(session.events.len(), 2);

        let last = &session.events[1];
        assert!(matches!(last.event_type, SessionEventType::AwaitingInput));
        assert_eq!(last.message, Some("waiting".to_string()));
    }

    #[test]
    fn test_add_event_without_message() {
        let mut session = make_test_session();
        session.add_event(SessionEventType::Completed, None);

        let last = session.events.last().unwrap();
        assert!(matches!(last.event_type, SessionEventType::Completed));
        assert!(last.message.is_none());
    }

    #[test]
    fn test_last_event_returns_most_recent() {
        let mut session = make_test_session();

        // Initially returns Started event
        let last = session.last_event().unwrap();
        assert!(matches!(last.event_type, SessionEventType::Started));

        // After adding events, returns the newest
        session.add_event(SessionEventType::PrCreated, None);
        session.add_event(SessionEventType::Completed, None);

        let last = session.last_event().unwrap();
        assert!(matches!(last.event_type, SessionEventType::Completed));
    }

    #[test]
    fn test_is_awaiting_input_returns_true_when_last_event_is_awaiting() {
        let mut session = make_test_session();
        assert!(!session.is_awaiting_input());

        session.add_event(SessionEventType::AwaitingInput, None);
        assert!(session.is_awaiting_input());

        // After another event, should return false
        session.add_event(SessionEventType::Resumed, None);
        assert!(!session.is_awaiting_input());
    }

    #[test]
    fn test_is_completed_returns_true_when_last_event_is_completed() {
        let mut session = make_test_session();
        assert!(!session.is_completed());

        session.add_event(SessionEventType::Completed, None);
        assert!(session.is_completed());
    }

    #[test]
    fn test_save_and_load_roundtrip() {
        let temp_dir = TempDir::new().unwrap();
        let sessions_dir = temp_dir.path();

        let mut session = make_test_session();
        session.add_event(SessionEventType::PrCreated, Some("PR #42".to_string()));

        // Save
        session.save(sessions_dir).unwrap();

        // Load
        let loaded = Session::load(sessions_dir, "test-id").unwrap();
        assert!(loaded.is_some());

        let loaded = loaded.unwrap();
        assert_eq!(loaded.id, "test-id");
        assert_eq!(loaded.ticket_id, "TICKET-001");
        assert_eq!(loaded.events.len(), 2);
    }

    #[test]
    fn test_load_nonexistent_returns_none() {
        let temp_dir = TempDir::new().unwrap();
        let result = Session::load(temp_dir.path(), "nonexistent").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_session_event_types_coverage() {
        let mut session = make_test_session();

        // Test all event types
        session.add_event(SessionEventType::AwaitingInput, None);
        session.add_event(SessionEventType::Resumed, None);
        session.add_event(SessionEventType::PrCreated, None);
        session.add_event(SessionEventType::TicketsCreated, None);
        session.add_event(SessionEventType::Paused, None);
        session.add_event(SessionEventType::Failed, None);
        session.add_event(SessionEventType::Completed, None);

        assert_eq!(session.events.len(), 8);
        assert!(session.is_completed());
    }
}

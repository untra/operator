use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

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

    pub fn load(sessions_dir: &PathBuf, id: &str) -> Result<Option<Self>> {
        let path = sessions_dir.join(format!("{}.json", id));

        if !path.exists() {
            return Ok(None);
        }

        let contents = fs::read_to_string(&path)?;
        let session: Session = serde_json::from_str(&contents)?;
        Ok(Some(session))
    }

    pub fn save(&self, sessions_dir: &PathBuf) -> Result<()> {
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

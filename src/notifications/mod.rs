use anyhow::Result;

#[cfg(target_os = "macos")]
mod macos;

/// Send a notification
pub fn send(title: &str, subtitle: &str, message: &str, sound: bool) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        macos::send_notification(title, subtitle, message, sound)
    }

    #[cfg(not(target_os = "macos"))]
    {
        // Fall back to just logging on non-macOS systems
        tracing::info!("Notification: {} - {} - {}", title, subtitle, message);
        Ok(())
    }
}

/// Notification event types for the operator
#[derive(Debug, Clone)]
pub enum NotificationEvent {
    AgentStarted {
        project: String,
        ticket_type: String,
        ticket_id: String,
    },
    AgentComplete {
        project: String,
        ticket_type: String,
        ticket_id: String,
        pr_url: Option<String>,
    },
    AgentNeedsInput {
        project: String,
        ticket_type: String,
        ticket_id: String,
        question: String,
    },
    PrCreated {
        project: String,
        ticket_id: String,
        pr_url: String,
    },
    InvestigationCreated {
        source: String,
        severity: String,
        summary: String,
    },
}

impl NotificationEvent {
    pub fn send(&self, sound: bool) -> Result<()> {
        match self {
            NotificationEvent::AgentStarted {
                project,
                ticket_type,
                ticket_id,
            } => send(
                "Agent Started",
                &format!("{} - {}", project, ticket_type),
                &format!("Working on {}", ticket_id),
                sound,
            ),

            NotificationEvent::AgentComplete {
                project,
                ticket_type,
                ticket_id,
                pr_url,
            } => {
                let message = if let Some(url) = pr_url {
                    format!("{} complete - PR: {}", ticket_id, url)
                } else {
                    format!("{} complete", ticket_id)
                };
                send(
                    "Agent Complete",
                    &format!("{} - {}", project, ticket_type),
                    &message,
                    sound,
                )
            }

            NotificationEvent::AgentNeedsInput {
                project,
                ticket_type,
                ticket_id,
                question,
            } => send(
                "Agent Needs Input",
                &format!("{} - {} ({})", project, ticket_type, ticket_id),
                question,
                sound,
            ),

            NotificationEvent::PrCreated {
                project,
                ticket_id,
                pr_url,
            } => send(
                "PR Created",
                &format!("{} - {}", project, ticket_id),
                pr_url,
                sound,
            ),

            NotificationEvent::InvestigationCreated {
                source,
                severity,
                summary,
            } => send(
                "Investigation Created",
                &format!("[{}] from {}", severity, source),
                summary,
                sound,
            ),
        }
    }
}

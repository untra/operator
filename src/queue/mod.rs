#![allow(dead_code)]
#![allow(unused_imports)]

pub mod creator;
mod ticket;
mod watcher;

pub use creator::TicketCreator;
pub use ticket::Ticket;
pub use watcher::QueueWatcher;

use anyhow::{Context, Result};
use chrono::Utc;
use std::fs;
use std::path::PathBuf;

use crate::config::Config;

pub struct Queue {
    config: Config,
    queue_path: PathBuf,
    in_progress_path: PathBuf,
    completed_path: PathBuf,
    templates_path: PathBuf,
}

impl Queue {
    pub fn new(config: &Config) -> Result<Self> {
        let tickets_path = config.tickets_path();

        Ok(Self {
            config: config.clone(),
            queue_path: tickets_path.join("queue"),
            in_progress_path: tickets_path.join("in-progress"),
            completed_path: tickets_path.join("completed"),
            templates_path: tickets_path.join("templates"),
        })
    }

    /// List all tickets in queue, sorted by priority then FIFO
    pub fn list_by_priority(&self) -> Result<Vec<Ticket>> {
        let mut tickets = self.list_queue()?;

        tickets.sort_by(|a, b| {
            let priority_a = self.config.priority_index(&a.ticket_type);
            let priority_b = self.config.priority_index(&b.ticket_type);

            priority_a
                .cmp(&priority_b)
                .then_with(|| a.timestamp.cmp(&b.timestamp))
        });

        Ok(tickets)
    }

    /// List all tickets in queue (unsorted)
    pub fn list_queue(&self) -> Result<Vec<Ticket>> {
        self.list_directory(&self.queue_path)
    }

    /// List in-progress tickets
    pub fn list_in_progress(&self) -> Result<Vec<Ticket>> {
        self.list_directory(&self.in_progress_path)
    }

    /// List completed tickets
    pub fn list_completed(&self) -> Result<Vec<Ticket>> {
        self.list_directory(&self.completed_path)
    }

    fn list_directory(&self, path: &PathBuf) -> Result<Vec<Ticket>> {
        if !path.exists() {
            return Ok(Vec::new());
        }

        let mut tickets = Vec::new();

        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().is_some_and(|e| e == "md") {
                if let Ok(ticket) = Ticket::from_file(&path) {
                    tickets.push(ticket);
                }
            }
        }

        Ok(tickets)
    }

    /// Get the next ticket to work on (highest priority, oldest)
    pub fn next_ticket(&self) -> Result<Option<Ticket>> {
        let tickets = self.list_by_priority()?;
        Ok(tickets.into_iter().next())
    }

    /// Find a specific ticket by ID in any directory
    pub fn find_ticket(&self, id: &str) -> Result<Option<Ticket>> {
        // Search in queue first
        for ticket in self.list_queue()? {
            if ticket.id == id || ticket.filename.contains(id) {
                return Ok(Some(ticket));
            }
        }

        // Then in-progress
        for ticket in self.list_in_progress()? {
            if ticket.id == id || ticket.filename.contains(id) {
                return Ok(Some(ticket));
            }
        }

        Ok(None)
    }

    /// Find a specific ticket by ID in the in-progress directory only
    pub fn get_in_progress_ticket(&self, id: &str) -> Result<Option<Ticket>> {
        for ticket in self.list_in_progress()? {
            if ticket.id == id || ticket.filename.contains(id) {
                return Ok(Some(ticket));
            }
        }
        Ok(None)
    }

    /// Reload a ticket from disk (useful after external modifications)
    pub fn reload_ticket(&self, ticket: &Ticket) -> Result<Ticket> {
        let path = std::path::PathBuf::from(&ticket.filepath);
        Ticket::from_file(&path)
    }

    /// Move ticket from queue to in-progress
    pub fn claim_ticket(&self, ticket: &Ticket) -> Result<()> {
        let src = self.queue_path.join(&ticket.filename);
        let dst = self.in_progress_path.join(&ticket.filename);

        fs::rename(&src, &dst).context("Failed to move ticket to in-progress")?;

        Ok(())
    }

    /// Move ticket from in-progress to completed
    pub fn complete_ticket(&self, ticket: &Ticket) -> Result<()> {
        let src = self.in_progress_path.join(&ticket.filename);
        let dst = self.completed_path.join(&ticket.filename);

        fs::rename(&src, &dst).context("Failed to move ticket to completed")?;

        Ok(())
    }

    /// Create a new investigation ticket from an external alert
    pub fn create_investigation(
        &self,
        source: String,
        message: String,
        severity: String,
        project: Option<String>,
    ) -> Result<Ticket> {
        // Read template
        let template_path = self.templates_path.join("investigation.md");
        let template =
            fs::read_to_string(&template_path).context("Failed to read investigation template")?;

        // Generate ticket ID and filename
        let now = Utc::now();
        let timestamp = now.format("%Y%m%d-%H%M").to_string();
        let id = format!("{:04}", now.timestamp() % 10000);
        let project_str = project.as_deref().unwrap_or("global");
        let short_desc = slugify(&message, 30);
        let filename = format!("{}-INV-{}-{}.md", timestamp, project_str, short_desc);

        // Fill in template
        let content = template
            .replace("INV-XXXX", &format!("INV-{}", id))
            .replace(
                "[global|adminsvc|apisvc|gamesvc|g|hushsvc|uzersvc|outboundsvc|www|iac|proto|e2e]",
                project_str,
            )
            .replace("[S0-outage|S1-major|S2-minor]", &severity)
            .replace("YYYY-MM-DD", &now.format("%Y-%m-%d").to_string())
            .replace(
                "[alert|user-report|monitoring|deploy-failure|test-failure]",
                &source,
            )
            .replace("[One-line description of the observed failure]", &message);

        // Write ticket
        let ticket_path = self.queue_path.join(&filename);
        fs::write(&ticket_path, &content)?;

        Ticket::from_file(&ticket_path)
    }
}

fn slugify(s: &str, max_len: usize) -> String {
    let slug: String = s
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect();

    // Remove consecutive dashes and trim
    let mut result = String::new();
    let mut last_dash = false;

    for c in slug.chars() {
        if c == '-' {
            if !last_dash && !result.is_empty() {
                result.push(c);
                last_dash = true;
            }
        } else {
            result.push(c);
            last_dash = false;
        }
    }

    result.trim_matches('-').chars().take(max_len).collect()
}

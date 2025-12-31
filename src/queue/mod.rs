#![allow(dead_code)] // Active module - some queue methods reserved for future workflow features
#![allow(unused_imports)]

pub mod creator;
mod ticket;
mod watcher;

pub use creator::TicketCreator;
pub use ticket::{LlmTask, Ticket};
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

    /// Move ticket from in-progress back to queue
    pub fn return_to_queue(&self, ticket: &Ticket) -> Result<()> {
        let src = self.in_progress_path.join(&ticket.filename);
        let dst = self.queue_path.join(&ticket.filename);

        fs::rename(&src, &dst).context("Failed to move ticket back to queue")?;

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    // ─── Slugify Tests ───────────────────────────────────────────────────────────

    #[test]
    fn test_slugify_basic() {
        assert_eq!(slugify("Hello World", 100), "hello-world");
    }

    #[test]
    fn test_slugify_unicode() {
        // Rust's is_alphanumeric() includes unicode letters, so they're preserved
        // Only emoji (non-alphanumeric) become hyphens
        assert_eq!(slugify("hello世界", 100), "hello世界");
        assert_eq!(slugify("café", 100), "café");
        assert_eq!(slugify("test🚀rocket", 100), "test-rocket");
    }

    #[test]
    fn test_slugify_special_chars() {
        assert_eq!(slugify("foo@#$bar", 100), "foo-bar");
        assert_eq!(slugify("hello!world?test", 100), "hello-world-test");
    }

    #[test]
    fn test_slugify_consecutive_dashes() {
        assert_eq!(slugify("foo---bar", 100), "foo-bar");
        assert_eq!(slugify("a   b   c", 100), "a-b-c");
    }

    #[test]
    fn test_slugify_leading_trailing() {
        assert_eq!(slugify("--foo--", 100), "foo");
        assert_eq!(slugify("   hello   ", 100), "hello");
        assert_eq!(slugify("###test###", 100), "test");
    }

    #[test]
    fn test_slugify_max_length() {
        assert_eq!(slugify("hello-world-this-is-long", 10), "hello-worl");
        assert_eq!(slugify("abcdefghij", 5), "abcde");
    }

    #[test]
    fn test_slugify_empty() {
        assert_eq!(slugify("", 100), "");
    }

    #[test]
    fn test_slugify_only_special_chars() {
        assert_eq!(slugify("@#$%", 100), "");
        assert_eq!(slugify("   ", 100), "");
        assert_eq!(slugify("---", 100), "");
    }

    // ─── Queue Priority Tests ────────────────────────────────────────────────────

    fn create_test_ticket(
        dir: &std::path::Path,
        timestamp: &str,
        ticket_type: &str,
        project: &str,
    ) {
        let filename = format!("{}-{}-{}-summary.md", timestamp, ticket_type, project);
        let content = format!(
            "---\npriority: P2-medium\n---\n# {}: Test Summary\n\nDescription here.",
            ticket_type
        );
        fs::write(dir.join(&filename), content).unwrap();
    }

    fn test_config(temp_dir: &TempDir) -> Config {
        let tickets_path = temp_dir.path().to_path_buf();
        fs::create_dir_all(tickets_path.join("queue")).unwrap();
        fs::create_dir_all(tickets_path.join("in-progress")).unwrap();
        fs::create_dir_all(tickets_path.join("completed")).unwrap();
        fs::create_dir_all(tickets_path.join("templates")).unwrap();

        let mut config = Config::default();
        config.paths.tickets = tickets_path.to_string_lossy().to_string();
        config
    }

    #[test]
    fn test_list_by_priority_empty_queue() {
        let temp_dir = TempDir::new().unwrap();
        let config = test_config(&temp_dir);
        let queue = Queue::new(&config).unwrap();

        let tickets = queue.list_by_priority().unwrap();
        assert!(tickets.is_empty());
    }

    #[test]
    fn test_list_by_priority_single_ticket() {
        let temp_dir = TempDir::new().unwrap();
        let config = test_config(&temp_dir);
        let queue_dir = temp_dir.path().join("queue");

        create_test_ticket(&queue_dir, "20241231-1200", "FEAT", "test");

        let queue = Queue::new(&config).unwrap();
        let tickets = queue.list_by_priority().unwrap();

        assert_eq!(tickets.len(), 1);
        assert_eq!(tickets[0].ticket_type, "FEAT");
    }

    #[test]
    fn test_list_by_priority_fifo_on_tie() {
        let temp_dir = TempDir::new().unwrap();
        let config = test_config(&temp_dir);
        let queue_dir = temp_dir.path().join("queue");

        // Same type, different timestamps - older should come first
        create_test_ticket(&queue_dir, "20241231-1000", "FEAT", "older");
        create_test_ticket(&queue_dir, "20241231-1200", "FEAT", "newer");

        let queue = Queue::new(&config).unwrap();
        let tickets = queue.list_by_priority().unwrap();

        assert_eq!(tickets.len(), 2);
        assert_eq!(tickets[0].project, "older");
        assert_eq!(tickets[1].project, "newer");
    }

    #[test]
    fn test_list_by_priority_respects_order() {
        let temp_dir = TempDir::new().unwrap();
        let config = test_config(&temp_dir);
        let queue_dir = temp_dir.path().join("queue");

        // Create tickets in reverse priority order (timestamps same)
        create_test_ticket(&queue_dir, "20241231-1200", "SPIKE", "proj1");
        create_test_ticket(&queue_dir, "20241231-1201", "FEAT", "proj2");
        create_test_ticket(&queue_dir, "20241231-1202", "TASK", "proj3");
        create_test_ticket(&queue_dir, "20241231-1203", "FIX", "proj4");
        create_test_ticket(&queue_dir, "20241231-1204", "INV", "proj5");

        let queue = Queue::new(&config).unwrap();
        let tickets = queue.list_by_priority().unwrap();

        assert_eq!(tickets.len(), 5);
        // Should be sorted by priority: INV > FIX > TASK > FEAT > SPIKE
        assert_eq!(tickets[0].ticket_type, "INV");
        assert_eq!(tickets[1].ticket_type, "FIX");
        assert_eq!(tickets[2].ticket_type, "TASK");
        assert_eq!(tickets[3].ticket_type, "FEAT");
        assert_eq!(tickets[4].ticket_type, "SPIKE");
    }

    #[test]
    fn test_next_ticket_selection() {
        let temp_dir = TempDir::new().unwrap();
        let config = test_config(&temp_dir);
        let queue_dir = temp_dir.path().join("queue");

        // FIX should be selected over FEAT despite later timestamp
        create_test_ticket(&queue_dir, "20241231-1000", "FEAT", "low");
        create_test_ticket(&queue_dir, "20241231-1200", "FIX", "high");

        let queue = Queue::new(&config).unwrap();
        let next = queue.next_ticket().unwrap();

        assert!(next.is_some());
        assert_eq!(next.unwrap().ticket_type, "FIX");
    }

    #[test]
    fn test_next_ticket_empty_queue() {
        let temp_dir = TempDir::new().unwrap();
        let config = test_config(&temp_dir);
        let queue = Queue::new(&config).unwrap();

        let next = queue.next_ticket().unwrap();
        assert!(next.is_none());
    }
}

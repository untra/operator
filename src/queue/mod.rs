#![allow(dead_code)] // Active module - some queue methods reserved for future workflow features
#![allow(unused_imports)]

pub mod creator;
mod ticket;
mod watcher;

pub use creator::TicketCreator;
pub use ticket::{LlmTask, StepAdvanceResult, Ticket, TicketPriority, TicketStatus};
pub use watcher::QueueWatcher;

use anyhow::{Context, Result};
use chrono::Utc;
use std::fs;
use std::path::PathBuf;

use crate::config::Config;

/// One of operator's three ticket states, and the directory that holds it.
///
/// The board column a ticket appears in is a property of which directory it
/// lives in, so this is also the unit an external board transition is keyed on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TicketColumn {
    Queue,
    InProgress,
    Completed,
}

impl TicketColumn {
    pub fn dir_name(&self) -> &'static str {
        match self {
            TicketColumn::Queue => "queue",
            TicketColumn::InProgress => "in-progress",
            TicketColumn::Completed => "completed",
        }
    }
}

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

    /// List all tickets in queue: issuetype rank, then the ticket's own
    /// priority, then FIFO. The web board sorts on the same three keys.
    pub fn list_by_priority(&self) -> Result<Vec<Ticket>> {
        let mut tickets = self.list_queue()?;

        tickets.sort_by(|a, b| {
            let priority_a = self.config.priority_index(&a.ticket_type);
            let priority_b = self.config.priority_index(&b.ticket_type);

            priority_a
                .cmp(&priority_b)
                .then_with(|| a.priority_level().cmp(&b.priority_level()))
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

    /// Directory a ticket lives in for one of operator's three states.
    fn column_path(&self, column: TicketColumn) -> &std::path::Path {
        match column {
            TicketColumn::Queue => &self.queue_path,
            TicketColumn::InProgress => &self.in_progress_path,
            TicketColumn::Completed => &self.completed_path,
        }
    }

    /// Where this ticket's file actually is right now.
    ///
    /// `Ticket::filepath` is authoritative when it still resolves, but a ticket
    /// value can outlive the path it was read from (it may have been moved
    /// since, or synthesized). Falling back to a filename lookup across the
    /// three columns matches how tickets are located everywhere else.
    fn locate(&self, ticket: &Ticket) -> Option<PathBuf> {
        let recorded = PathBuf::from(&ticket.filepath);
        if recorded.is_file() {
            return Some(recorded);
        }
        [
            TicketColumn::Queue,
            TicketColumn::InProgress,
            TicketColumn::Completed,
        ]
        .into_iter()
        .map(|c| self.column_path(c).join(&ticket.filename))
        .find(|p| p.is_file())
    }

    /// Move a ticket into `column` from wherever it currently lives.
    ///
    /// Moving a ticket to the column it is already in is a no-op.
    pub fn move_ticket(&self, ticket: &Ticket, column: TicketColumn) -> Result<()> {
        let dst = self.column_path(column).join(&ticket.filename);
        let src = self
            .locate(ticket)
            .ok_or_else(|| anyhow::anyhow!("Ticket file not found for '{}'", ticket.filename))?;
        if src == dst {
            return Ok(());
        }
        fs::create_dir_all(self.column_path(column))
            .with_context(|| format!("Failed to create {} directory", column.dir_name()))?;
        fs::rename(&src, &dst)
            .with_context(|| format!("Failed to move ticket to {}", column.dir_name()))?;
        Ok(())
    }

    /// Move ticket from queue to in-progress
    pub fn claim_ticket(&self, ticket: &Ticket) -> Result<()> {
        self.move_ticket(ticket, TicketColumn::InProgress)
    }

    /// Move ticket from in-progress to completed
    pub fn complete_ticket(&self, ticket: &Ticket) -> Result<()> {
        self.move_ticket(ticket, TicketColumn::Completed)
    }

    /// Move ticket from in-progress back to queue
    pub fn return_to_queue(&self, ticket: &Ticket) -> Result<()> {
        self.move_ticket(ticket, TicketColumn::Queue)
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
        let filename = format!("{timestamp}-INV-{project_str}-{short_desc}.md");

        // Fill in template
        let content = template
            .replace("INV-XXXX", &format!("INV-{id}"))
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
        let filename = format!("{timestamp}-{ticket_type}-{project}-summary.md");
        let content = format!(
            "---\npriority: P2-medium\n---\n# {ticket_type}: Test Summary\n\nDescription here."
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

    /// Within one issuetype the ticket's own `priority:` outranks FIFO, so the
    /// launcher and the web board pick the same next ticket.
    #[test]
    fn test_list_by_priority_breaks_type_ties_by_ticket_priority() {
        let temp_dir = TempDir::new().unwrap();
        let config = test_config(&temp_dir);
        let queue_dir = temp_dir.path().join("queue");

        let write = |timestamp: &str, project: &str, priority: &str| {
            fs::write(
                queue_dir.join(format!("{timestamp}-FEAT-{project}-summary.md")),
                format!("---\npriority: {priority}\n---\n# FEAT: Test Summary\n"),
            )
            .unwrap();
        };
        write("20241231-1000", "older", "P3-low");
        write("20241231-1200", "newer", "P0-critical");

        let queue = Queue::new(&config).unwrap();
        let tickets = queue.list_by_priority().unwrap();

        assert_eq!(tickets[0].project, "newer");
        assert_eq!(tickets[1].project, "older");
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

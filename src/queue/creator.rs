//! Ticket creation logic

use anyhow::{Context, Result};
use chrono::Utc;
use handlebars::Handlebars;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use super::ids;
use crate::config::Config;
use crate::templates::schema::TemplateSchema;
use crate::templates::TemplateType;

/// Lock file guarding id allocation in the tickets directory.
const CREATION_LOCK_FILENAME: &str = ".create.lock";

/// Filename project segment for tickets that target no project.
const DEFAULT_PROJECT: &str = "global";

/// Held while allocating an id and writing a ticket. Released on drop.
pub struct CreationLock {
    _process: std::sync::MutexGuard<'static, ()>,
    _file: fs::File,
}

/// Creates new tickets from templates
pub struct TicketCreator {
    tickets_root: PathBuf,
    /// Active issuetype collection, stamped into created tickets' frontmatter
    collection: Option<String>,
}

impl TicketCreator {
    /// Create a new ticket creator
    pub fn new(config: &Config) -> Self {
        let tickets_path = config.tickets_path();
        Self {
            tickets_root: tickets_path,
            collection: config.templates.active_collection.clone(),
        }
    }

    fn queue_path(&self) -> PathBuf {
        self.tickets_root.join("queue")
    }

    /// Serializes id allocation and writes across single and batch creation,
    /// both within this process and against any other operator process
    /// (embedded TUI server, standalone `operator api`, CLI) sharing the
    /// tickets directory.
    pub fn creation_lock(tickets_root: &Path) -> Result<CreationLock> {
        static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
        let process = LOCK
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        fs::create_dir_all(tickets_root).context("Failed to create tickets directory")?;
        let file = fs::OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .open(tickets_root.join(CREATION_LOCK_FILENAME))
            .context("Failed to open ticket creation lock")?;
        file.lock().context("Failed to acquire ticket creation lock")?;
        Ok(CreationLock {
            _process: process,
            _file: file,
        })
    /// Serializes id allocation and writes across single and batch creation.
    pub fn creation_lock() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
        LOCK.lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// Ids of every ticket in queue, in-progress and completed.
    fn existing_ids(&self) -> Vec<String> {
        ["queue", "in-progress", "completed"]
            .iter()
            .filter_map(|dir| fs::read_dir(self.tickets_root.join(dir)).ok())
            .flatten()
            .filter_map(|entry| entry.ok().map(|e| e.path()))
            .filter(|path| path.extension().is_some_and(|ext| ext == "md"))
            .filter_map(|path| super::Ticket::from_file(&path).ok())
            .map(|ticket| ticket.id)
            .collect()
    }

    /// Create a new ticket from template with pre-filled values, without
    /// launching an editor. Used by MCP and other headless callers.
    pub fn create_ticket_headless(
        &self,
        template_type: TemplateType,
        values: &HashMap<String, String>,
    ) -> Result<PathBuf> {
        self.create_from_template(
            template_type.as_str(),
            template_type.template_content(),
            values,
        )
    }

    /// Render `template` with `values` and write it to the queue as a ticket
    /// of issue type `type_key`.
    pub fn create_from_template(
        &self,
        type_key: &str,
        template: &str,
        values: &HashMap<String, String>,
    ) -> Result<PathBuf> {
        let project = values
            .get("project")
            .filter(|p| !p.is_empty())
            .cloned()
            .unwrap_or_else(|| "global".to_string());

        let slug = values
            .get("summary")
            .map(|summary| ids::slugify(summary, 50))
            .filter(|slug| !slug.is_empty())
            .or_else(|| values.get("id").map(|id| ids::slugify(id, 50)))
            .unwrap_or_else(|| "new-ticket".to_string());
        let queue_path = self.queue_path();
        let filepath = ids::unique_path(
            &queue_path,
            &format!("{timestamp}-{type_str}-{project}-{slug}"),
        );

        let template = template_type.template_content();
        let content = render_template(template, values)?;
        let content = stamp_collection(&content, self.collection.as_deref());
        let content = match values.get("project").filter(|p| !p.is_empty()) {
            Some(project) => stamp_frontmatter_field(&content, "project", project),
            None => content,
        };
        self.write_rendered(type_key, values, &content)
    }

        fs::create_dir_all(&queue_path).context("Failed to create queue directory")?;
        fs::write(&filepath, &content).context("Failed to write ticket file")?;

        Ok(filepath)
    }

    /// The filename a ticket with `values` would be written to right now.
    pub fn next_queue_path(&self, type_key: &str, values: &HashMap<String, String>) -> PathBuf {
        let timestamp = Utc::now().format("%Y%m%d-%H%M").to_string();
        let project = values
            .get("project")
            .map(|p| filename_project(p))
            .filter(|p| !p.is_empty())
            .unwrap_or_else(|| DEFAULT_PROJECT.to_string());

        let slug = values
            .get("summary")
            .map(|summary| ids::slugify(summary, 50))
            .filter(|slug| !slug.is_empty())
            .or_else(|| values.get("id").map(|id| ids::slugify(id, 50)))
            .unwrap_or_else(|| "new-ticket".to_string());
        ids::unique_path(
            &self.queue_path(),
            &format!("{timestamp}-{type_key}-{project}-{slug}"),
        )
    }

    /// Create a new ticket from template with pre-filled values and open in editor.
    ///
    /// Returns the path to the created ticket file.
    /// `editor_cmd` is the resolved editor command (e.g. from `EditorConfig::file_editor()`).
    pub fn create_ticket_with_values(
        &self,
        template_type: TemplateType,
        values: &HashMap<String, String>,
        editor_cmd: &str,
    ) -> Result<PathBuf> {
        let filepath = self.create_ticket_headless(template_type, values)?;
        self.open_in_editor(&filepath, editor_cmd)?;
        Ok(filepath)
    }

    /// Create a new ticket from template and open in editor (legacy method).
    ///
    /// Returns the path to the created ticket file.
    pub fn create_ticket(
        &self,
        template_type: TemplateType,
        project: &str,
        editor_cmd: &str,
    ) -> Result<PathBuf> {
        let values = self.generate_default_values(template_type, project);
        self.create_ticket_with_values(template_type, &values, editor_cmd)
    }

    /// Generate default values for auto-filled fields
    pub fn generate_default_values(
        &self,
        template_type: TemplateType,
        project: &str,
    ) -> HashMap<String, String> {
        self.generate_default_values_for(template_type.as_str(), template_type.first_step(), project)
    }

    /// Generate auto-filled values (id, dates, status, branch, step) for any
    /// issue type key.
    pub fn generate_default_values_for(
        &self,
        type_key: &str,
        first_step: &str,
        project: &str,
    ) -> HashMap<String, String> {
        let now = Utc::now();
        let date = now.format("%Y-%m-%d").to_string();
        let datetime = now.format("%Y-%m-%d %H:%M").to_string();
        let type_str = template_type.as_str();
        let existing = self.existing_ids();
        let id = ids::next_ticket_id(type_str, existing.iter().map(String::as_str));
        let branch_prefix = type_str.to_lowercase();

        let mut values = HashMap::new();
        values.insert("id".to_string(), id.clone());
        values.insert("created".to_string(), date.clone());
        values.insert("created_date".to_string(), date);
        values.insert("created_datetime".to_string(), datetime);
        values.insert("status".to_string(), "queued".to_string());
        values.insert("project".to_string(), project.to_string());
        values.insert(
            "branch".to_string(),
            format!("{branch_prefix}/{id}-short-description"),
        );
        values.insert("step".to_string(), first_step.to_string());

        values
    }

    /// Open a file in the user's preferred editor
    fn open_in_editor(&self, filepath: &PathBuf, editor_cmd: &str) -> Result<()> {
        let (prog, args) = crate::editors::EditorConfig::split_command(editor_cmd);

        let status = Command::new(prog)
            .args(&args)
            .arg(filepath)
            .status()
            .context(format!("Failed to open editor: {editor_cmd}"))?;

        if !status.success() {
            anyhow::bail!("Editor exited with non-zero status");
        }

        Ok(())
    }
}

/// Render a template using handlebars
pub fn render_template(template: &str, values: &HashMap<String, String>) -> Result<String> {
    let mut hb = Handlebars::new();

    // Don't escape HTML entities in the output
    hb.register_escape_fn(handlebars::no_escape);

    hb.register_template_string("ticket", template)
        .context("Failed to parse template")?;

    hb.render("ticket", values)
        .context("Failed to render template")
}

/// Parse a template schema from JSON and sort fields by `display_order`
pub fn parse_and_sort_schema(json: &str) -> Result<TemplateSchema> {
    let mut schema = TemplateSchema::from_json(json).context("Failed to parse template schema")?;

    // Sort fields by display_order (None values go last)
    schema.fields.sort_by(|a, b| {
        let order_a = a.display_order.unwrap_or(i32::MAX);
        let order_b = b.display_order.unwrap_or(i32::MAX);
        order_a.cmp(&order_b)
    });

    Ok(schema)
}

/// Get user-editable fields from a schema (fields the user should fill in the form)
/// Excludes auto-generated fields (`user_editable` = false)
pub fn get_user_fields(schema: &TemplateSchema) -> Vec<&crate::templates::schema::FieldSchema> {
    schema.fields.iter().filter(|f| f.user_editable).collect()
}

/// Separate fields into required and optional
pub fn split_required_optional(
    fields: Vec<&crate::templates::schema::FieldSchema>,
) -> (
    Vec<&crate::templates::schema::FieldSchema>,
    Vec<&crate::templates::schema::FieldSchema>,
) {
    let required: Vec<_> = fields.iter().filter(|f| f.required).copied().collect();
    let optional: Vec<_> = fields.iter().filter(|f| !f.required).copied().collect();
    (required, optional)
}

/// Inject `collection: <name>` into a rendered ticket's YAML frontmatter.
///
/// No-op when no collection is set, the content has no frontmatter, or the
/// template already writes its own `collection:` field.
fn stamp_collection(content: &str, collection: Option<&str>) -> String {
    match collection {
        Some(collection) => stamp_frontmatter_field(content, "collection", collection),
        None => content.to_string(),
    }
}

/// Prepend `field: value` to a rendered ticket's YAML frontmatter.
///
/// No-op when the content has no frontmatter or already sets `field`.
pub fn stamp_frontmatter_field(content: &str, field: &str, value: &str) -> String {
    let Some(rest) = content.strip_prefix("---\n") else {
        return content.to_string();
    };
    let Some(end) = rest.find("\n---") else {
        return content.to_string();
    };
    let prefix = format!("{field}:");
    let has_field = rest[..end].lines().any(|line| line.starts_with(&prefix));
    if has_field {
        return content.to_string();
    }
    format!("---\n{field}: {value}\n{rest}")
}

/// Project segment of a ticket filename. Filenames only carry lowercase
/// alphanumerics there; the full name lives in the `project:` frontmatter.
fn filename_project(project: &str) -> String {
    project
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .collect::<String>()
        .to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stamp_collection_injects_into_frontmatter() {
        let content = "---\nid: TASK-0001\nstatus: queued\n---\n\n# Task: hello\n";
        let stamped = stamp_collection(content, Some("ralph_loop"));
        assert!(stamped.starts_with("---\ncollection: ralph_loop\nid: TASK-0001\n"));
        assert!(stamped.ends_with("# Task: hello\n"));
    }

    #[test]
    fn test_stamp_collection_none_is_noop() {
        let content = "---\nid: TASK-0001\n---\n\n# Task\n";
        assert_eq!(stamp_collection(content, None), content);
    }

    #[test]
    fn test_stamp_collection_respects_existing_field() {
        let content = "---\nid: TASK-0001\ncollection: custom\n---\n\n# Task\n";
        assert_eq!(stamp_collection(content, Some("ralph_loop")), content);
    }

    #[test]
    fn test_stamp_collection_no_frontmatter_is_noop() {
        let content = "# Task: no frontmatter\n";
        assert_eq!(stamp_collection(content, Some("ralph_loop")), content);
    }

    #[test]
    fn test_create_ticket_headless_stamps_collection() {
        let temp_dir = tempfile::tempdir().unwrap();
        let creator = TicketCreator {
            tickets_root: temp_dir.path().to_path_buf(),
            collection: Some("dev_kanban".to_string()),
        };
        let mut values = HashMap::new();
        values.insert("id".to_string(), "TASK-0001".to_string());
        values.insert("status".to_string(), "queued".to_string());
        values.insert("project".to_string(), "operator".to_string());
        values.insert("summary".to_string(), "stamp me".to_string());

        let path = creator
            .create_ticket_headless(TemplateType::Task, &values)
            .unwrap();
        let written = fs::read_to_string(path).unwrap();
        assert!(
            written.contains("collection: dev_kanban"),
            "created ticket should carry the active collection: {written}"
        );
    }

    #[test]
    fn test_render_template() {
        let template = "ID: {{ id }}\nProject: {{ project }}\nSummary: {{ summary }}";
        let mut values = HashMap::new();
        values.insert("id".to_string(), "FEAT-1234".to_string());
        values.insert("project".to_string(), "gamesvc".to_string());
        values.insert("summary".to_string(), "Add new feature".to_string());

        let result = render_template(template, &values).unwrap();

        assert!(result.contains("FEAT-1234"));
        assert!(result.contains("gamesvc"));
        assert!(result.contains("Add new feature"));
    }

    #[test]
    fn test_render_template_with_empty_values() {
        let template = "ID: {{ id }}\nContext: {{ context }}";
        let mut values = HashMap::new();
        values.insert("id".to_string(), "FIX-5678".to_string());
        values.insert("context".to_string(), String::new());

        let result = render_template(template, &values).unwrap();

        assert!(result.contains("FIX-5678"));
        assert!(result.contains("Context: "));
    }

    #[test]
    fn test_generate_default_values() {
        let temp_dir = tempfile::tempdir().unwrap();
        let creator = TicketCreator {
            tickets_root: temp_dir.path().to_path_buf(),
            collection: None,
        };

        let values = creator.generate_default_values(TemplateType::Feature, "myproject");

        assert_eq!(values.get("id").unwrap(), "FEAT-0001");
        assert_eq!(values.get("project").unwrap(), "myproject");
        assert_eq!(values.get("status").unwrap(), "queued");
        assert!(values.get("branch").unwrap().starts_with("feat/FEAT-"));
        assert!(values.contains_key("created"));
    }

    #[test]
    fn test_bulk_headless_creates_never_collide() {
        let temp_dir = tempfile::tempdir().unwrap();
        let creator = TicketCreator {
            tickets_root: temp_dir.path().to_path_buf(),
            collection: None,
        };
        for _ in 0..50 {
            let mut values = creator.generate_default_values(TemplateType::Task, "demo");
            values.insert("summary".to_string(), "same summary".to_string());
            creator
                .create_ticket_headless(TemplateType::Task, &values)
                .unwrap();
        }
        let mut ids = creator.existing_ids();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), 50);
        assert_eq!(ids.last().map(String::as_str), Some("TASK-0050"));
        assert_eq!(
            fs::read_dir(temp_dir.path().join("queue")).unwrap().count(),
            50
        );
    }

    #[test]
    fn test_ids_continue_across_in_progress_and_completed() {
        let temp_dir = tempfile::tempdir().unwrap();
        let done = temp_dir.path().join("completed");
        fs::create_dir_all(&done).unwrap();
        fs::write(
            done.join("20260101-1200-FEAT-demo-old.md"),
            "---\nid: FEAT-0041\nstatus: completed\n---\n\n# Feature: old\n",
        )
        .unwrap();
        let creator = TicketCreator {
            tickets_root: temp_dir.path().to_path_buf(),
            collection: None,
        };
        let values = creator.generate_default_values(TemplateType::Feature, "demo");
        assert_eq!(values.get("id").unwrap(), "FEAT-0042");
    }

    #[test]
    fn test_parse_and_sort_schema() {
        let json = r#"{
            "key": "TEST",
            "name": "Test",
            "description": "Test template",
            "mode": "autonomous",
            "glyph": "*",
            "fields": [
                {"name": "c", "description": "Third", "type": "string", "required": true, "default": "", "display_order": 3},
                {"name": "a", "description": "First", "type": "string", "required": true, "default": "", "display_order": 1},
                {"name": "b", "description": "Second", "type": "string", "required": true, "default": "", "display_order": 2}
            ],
            "steps": [
                {"name": "do", "status": "DOING", "outputs": ["code"], "prompt": "Do it", "allowed_tools": ["Read"]}
            ]
        }"#;

        let schema = parse_and_sort_schema(json).unwrap();

        assert_eq!(schema.fields[0].name, "a");
        assert_eq!(schema.fields[1].name, "b");
        assert_eq!(schema.fields[2].name, "c");
    }

    #[test]
    fn test_get_user_fields() {
        let json = r#"{
            "key": "TEST",
            "name": "Test",
            "description": "Test template",
            "mode": "autonomous",
            "glyph": "*",
            "fields": [
                {"name": "id", "description": "ID", "type": "string", "required": true, "auto": "id", "user_editable": false},
                {"name": "created", "description": "Date", "type": "date", "required": true, "auto": "date", "user_editable": false},
                {"name": "summary", "description": "Summary", "type": "string", "required": true, "default": ""},
                {"name": "context", "description": "Context", "type": "text", "required": false}
            ],
            "steps": [
                {"name": "do", "status": "DOING", "outputs": ["code"], "prompt": "Do it", "allowed_tools": ["Read"]}
            ]
        }"#;

        let schema = TemplateSchema::from_json(json).unwrap();
        let user_fields = get_user_fields(&schema);

        assert_eq!(user_fields.len(), 2);
        assert_eq!(user_fields[0].name, "summary");
        assert_eq!(user_fields[1].name, "context");
    }

    #[test]
    fn test_step_omitted_when_empty() {
        // Step should be omitted from frontmatter when empty/falsey
        let template = r"---
id: {{ id }}
{{#if step }}step: {{ step }}
{{/if}}status: {{ status }}
---

# Feature: {{ summary }}
";
        let mut values = HashMap::new();
        values.insert("id".to_string(), "FEAT-1234".to_string());
        values.insert("step".to_string(), String::new()); // Empty step
        values.insert("status".to_string(), "queued".to_string());
        values.insert("summary".to_string(), "Test feature".to_string());

        let result = render_template(template, &values).unwrap();

        // Step line should NOT appear in output when empty
        assert!(
            !result.contains("step:"),
            "Empty step should be omitted from frontmatter"
        );
        assert!(result.contains("id: FEAT-1234"));
        assert!(result.contains("status: queued"));
    }

    #[test]
    fn test_step_included_when_present() {
        // Step should be included when it has a value
        let template = r"---
id: {{ id }}
{{#if step }}step: {{ step }}
{{/if}}status: {{ status }}
---

# Feature: {{ summary }}
";
        let mut values = HashMap::new();
        values.insert("id".to_string(), "FEAT-1234".to_string());
        values.insert("step".to_string(), "plan".to_string()); // Non-empty step
        values.insert("status".to_string(), "queued".to_string());
        values.insert("summary".to_string(), "Test feature".to_string());

        let result = render_template(template, &values).unwrap();

        // Step line should appear in output when non-empty
        assert!(
            result.contains("step: plan"),
            "Non-empty step should be included in frontmatter"
        );
        assert!(result.contains("id: FEAT-1234"));
        assert!(result.contains("status: queued"));
    }

    #[test]
    fn test_template_step_omitted_when_empty_after_fix() {
        // After the fix, templates use conditional rendering for step
        // and empty step values should be omitted

        // Using the fixed feature.md template pattern
        let template = r"---
id: {{ id }}
{{#if step }}step: {{ step }}
{{/if}}status: {{ status }}
---

# Feature: {{ summary }}
";
        let mut values = HashMap::new();
        values.insert("id".to_string(), "FEAT-1234".to_string());
        values.insert("step".to_string(), String::new()); // Empty step
        values.insert("status".to_string(), "queued".to_string());
        values.insert("summary".to_string(), "Test feature".to_string());

        let result = render_template(template, &values).unwrap();

        // Fixed behavior: step line is NOT rendered when empty
        assert!(
            !result.contains("step:"),
            "Empty step should be omitted from frontmatter"
        );
    }
}

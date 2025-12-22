//! Ticket creation logic

use anyhow::{Context, Result};
use chrono::Utc;
use handlebars::Handlebars;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use crate::config::Config;
use crate::templates::schema::TemplateSchema;
use crate::templates::TemplateType;

/// Creates new tickets from templates
pub struct TicketCreator {
    queue_path: PathBuf,
}

impl TicketCreator {
    /// Create a new ticket creator
    pub fn new(config: &Config) -> Self {
        let tickets_path = config.tickets_path();
        Self {
            queue_path: tickets_path.join("queue"),
        }
    }

    /// Create a new ticket from template with pre-filled values and open in $EDITOR
    ///
    /// Returns the path to the created ticket file
    pub fn create_ticket_with_values(
        &self,
        template_type: TemplateType,
        values: &HashMap<String, String>,
    ) -> Result<PathBuf> {
        // Generate filename with timestamp
        let now = Utc::now();
        let timestamp = now.format("%Y%m%d-%H%M").to_string();
        let type_str = template_type.as_str();

        // Get project from values or use "global"
        let project = values
            .get("project")
            .filter(|p| !p.is_empty())
            .cloned()
            .unwrap_or_else(|| "global".to_string());

        let filename = format!("{}-{}-{}-new-ticket.md", timestamp, type_str, project);
        let filepath = self.queue_path.join(&filename);

        // Render template with handlebar values
        let template = template_type.template_content();
        let content = render_template(template, values)?;

        // Ensure queue directory exists
        fs::create_dir_all(&self.queue_path).context("Failed to create queue directory")?;

        // Write to file
        fs::write(&filepath, &content).context("Failed to write ticket file")?;

        // Open in $EDITOR
        self.open_in_editor(&filepath)?;

        Ok(filepath)
    }

    /// Create a new ticket from template and open in $EDITOR (legacy method)
    ///
    /// Returns the path to the created ticket file
    pub fn create_ticket(&self, template_type: TemplateType, project: &str) -> Result<PathBuf> {
        // Generate auto-filled values
        let values = self.generate_default_values(template_type, project);
        self.create_ticket_with_values(template_type, &values)
    }

    /// Generate default values for auto-filled fields
    pub fn generate_default_values(
        &self,
        template_type: TemplateType,
        project: &str,
    ) -> HashMap<String, String> {
        let now = Utc::now();
        let date = now.format("%Y-%m-%d").to_string();
        let datetime = now.format("%Y-%m-%d %H:%M").to_string();
        let id = format!("{:04}", now.timestamp() % 10000);
        let type_str = template_type.as_str();
        let branch_prefix = template_type.branch_prefix();

        let mut values = HashMap::new();
        values.insert("id".to_string(), format!("{}-{}", type_str, id));
        values.insert("created".to_string(), date.clone());
        values.insert("created_date".to_string(), date);
        values.insert("created_datetime".to_string(), datetime);
        values.insert("status".to_string(), "queued".to_string());
        values.insert("project".to_string(), project.to_string());
        values.insert(
            "branch".to_string(),
            format!("{}/{}-{}-short-description", branch_prefix, type_str, id),
        );
        values.insert("step".to_string(), template_type.first_step().to_string());

        values
    }

    /// Open a file in the user's preferred editor
    fn open_in_editor(&self, filepath: &PathBuf) -> Result<()> {
        let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());

        let status = Command::new(&editor)
            .arg(filepath)
            .status()
            .context(format!("Failed to open editor: {}", editor))?;

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

/// Parse a template schema from JSON and sort fields by display_order
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
/// Excludes auto-generated fields (user_editable = false)
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

#[cfg(test)]
mod tests {
    use super::*;

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
        values.insert("context".to_string(), "".to_string());

        let result = render_template(template, &values).unwrap();

        assert!(result.contains("FIX-5678"));
        assert!(result.contains("Context: "));
    }

    #[test]
    fn test_generate_default_values() {
        let creator = TicketCreator {
            queue_path: PathBuf::from("/tmp"),
        };

        let values = creator.generate_default_values(TemplateType::Feature, "myproject");

        assert!(values.get("id").unwrap().starts_with("FEAT-"));
        assert_eq!(values.get("project").unwrap(), "myproject");
        assert_eq!(values.get("status").unwrap(), "queued");
        assert!(values.get("branch").unwrap().starts_with("feature/FEAT-"));
        assert!(values.contains_key("created"));
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
}

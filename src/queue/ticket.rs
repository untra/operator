use anyhow::{Context, Result};
use chrono::Local;
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::templates::{schema::TemplateSchema, TemplateType};

#[derive(Debug, Clone)]
pub struct Ticket {
    pub filename: String,
    pub filepath: String,
    pub timestamp: String,
    pub ticket_type: String,
    pub project: String,
    pub id: String,
    pub summary: String,
    pub priority: String,
    pub status: String,
    pub step: String,
    pub content: String,
}

impl Ticket {
    /// Parse a ticket from a markdown file
    pub fn from_file(path: &Path) -> Result<Self> {
        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        let content = fs::read_to_string(path).context("Failed to read ticket file")?;

        // Parse filename: YYYYMMDD-HHMM-TYPE-PROJECT-description.md
        let (timestamp, ticket_type, project) = parse_filename(&filename)?;

        // Try to extract metadata from YAML frontmatter first, fall back to legacy regex parsing
        let (id, priority, status, step, summary) = if let Some((frontmatter, body)) =
            extract_frontmatter(&content)
        {
            let id = frontmatter
                .get("id")
                .cloned()
                .unwrap_or_else(|| format!("{}-{}", ticket_type, timestamp.replace('-', "")));
            let priority = frontmatter
                .get("priority")
                .cloned()
                .unwrap_or_else(|| "P2-medium".to_string());
            let status = frontmatter
                .get("status")
                .cloned()
                .unwrap_or_else(|| "queued".to_string());
            let step = frontmatter.get("step").cloned().unwrap_or_default();
            // Extract summary from body (after frontmatter)
            let summary = extract_summary(body);
            (id, priority, status, step, summary)
        } else {
            // Legacy parsing using regex for inline metadata
            let id = extract_field(&content, "ID")
                .unwrap_or_else(|| format!("{}-{}", ticket_type, timestamp.replace('-', "")));
            let priority =
                extract_field(&content, "Priority").unwrap_or_else(|| "P2-medium".to_string());
            let status = extract_field(&content, "Status").unwrap_or_else(|| "queued".to_string());
            let step = extract_field(&content, "Step").unwrap_or_default();
            let summary = extract_summary(&content);
            (id, priority, status, step, summary)
        };

        Ok(Self {
            filename,
            filepath: path.to_string_lossy().to_string(),
            timestamp,
            ticket_type,
            project,
            id,
            summary,
            priority,
            status,
            step,
            content,
        })
    }

    /// Check if this is a paired ticket type (requires human interaction)
    pub fn is_paired(&self) -> bool {
        matches!(self.ticket_type.as_str(), "SPIKE" | "INV")
    }

    /// Check if this is an autonomous ticket type
    pub fn is_autonomous(&self) -> bool {
        matches!(self.ticket_type.as_str(), "FEAT" | "FIX")
    }

    /// Get the branch name for this ticket
    pub fn branch_name(&self) -> String {
        let prefix = match self.ticket_type.as_str() {
            "FEAT" => "feature",
            "FIX" => "fix",
            "SPIKE" => "spike",
            "INV" => "investigation",
            _ => "work",
        };

        let slug = self
            .summary
            .to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '-' })
            .collect::<String>();

        let slug: String = slug
            .split('-')
            .filter(|s| !s.is_empty())
            .take(5)
            .collect::<Vec<_>>()
            .join("-");

        format!("{}/{}-{}", prefix, self.id, slug)
    }

    /// Update a frontmatter field in the ticket file and save
    pub fn update_field(&mut self, field: &str, value: &str) -> Result<()> {
        // Parse frontmatter
        if let Some((mut frontmatter, body)) = extract_frontmatter(&self.content) {
            frontmatter.insert(field.to_string(), value.to_string());

            // Rebuild the frontmatter
            let mut yaml_lines = Vec::new();
            for (k, v) in &frontmatter {
                yaml_lines.push(format!("{}: {}", k, v));
            }
            yaml_lines.sort(); // Keep consistent order

            let new_content = format!("---\n{}\n---{}", yaml_lines.join("\n"), body);
            self.content = new_content.clone();

            // Update the in-memory field
            match field {
                "step" => self.step = value.to_string(),
                "status" => self.status = value.to_string(),
                "priority" => self.priority = value.to_string(),
                _ => {}
            }

            // Write back to file
            fs::write(&self.filepath, new_content).context("Failed to write ticket file")?;
        }
        Ok(())
    }

    /// Append an entry to the ## History section (create if missing)
    pub fn append_history(&mut self, entry: &str) -> Result<()> {
        let history_header = "## History";

        if let Some(pos) = self.content.find(history_header) {
            // Find the end of the History section (next ## or end of file)
            let after_header = &self.content[pos + history_header.len()..];
            let section_end = after_header
                .find("\n## ")
                .map(|p| pos + history_header.len() + p)
                .unwrap_or(self.content.len());

            // Insert the new entry at the end of the History section
            let insert_pos = section_end;
            self.content.insert_str(insert_pos, &format!("\n{}", entry));
        } else {
            // No History section exists, add it at the end
            self.content
                .push_str(&format!("\n\n{}\n\n{}", history_header, entry));
        }

        // Write back to file
        fs::write(&self.filepath, &self.content).context("Failed to write ticket file")?;
        Ok(())
    }

    /// Add a timestamped AWAITING entry to the History section
    pub fn add_awaiting_entry(&mut self, step_display_name: &str) -> Result<()> {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
        let entry = format!(
            "- **{}** - Moved to AWAITING during \"{}\" step",
            timestamp, step_display_name
        );
        self.append_history(&entry)
    }

    /// Get the TemplateSchema for this ticket's type
    pub fn template_schema(&self) -> Option<TemplateSchema> {
        TemplateType::from_key(&self.ticket_type)
            .and_then(|tt| TemplateSchema::from_json(tt.schema()).ok())
    }

    /// Get the current step schema from the template
    pub fn current_step_schema(&self) -> Option<crate::templates::schema::StepSchema> {
        let schema = self.template_schema()?;
        schema.steps.into_iter().find(|s| s.name == self.step)
    }

    /// Get the display name of the current step
    pub fn current_step_display_name(&self) -> String {
        self.current_step_schema()
            .and_then(|s| s.display_name.clone())
            .unwrap_or_else(|| self.step.clone())
    }

    /// Advance to the next step in the workflow
    /// Returns the name of the new step, or None if this was the final step
    pub fn advance_step(&mut self) -> Result<Option<String>> {
        let current_step = self.current_step_schema();
        if let Some(step) = current_step {
            if let Some(next_step) = step.next_step {
                self.update_field("step", &next_step)?;
                return Ok(Some(next_step));
            }
        }
        Ok(None)
    }

    /// Check if the current step requires review before advancing
    pub fn step_requires_review(&self) -> bool {
        self.current_step_schema()
            .map(|s| s.requires_review)
            .unwrap_or(false)
    }
}

/// Extract YAML frontmatter from markdown content
/// Returns the parsed frontmatter as a HashMap and the content after the frontmatter
fn extract_frontmatter(content: &str) -> Option<(HashMap<String, String>, &str)> {
    let content = content.trim_start();
    if !content.starts_with("---") {
        return None;
    }

    // Find the closing ---
    let after_open = &content[3..];
    let end_idx = after_open.find("\n---")?;
    let yaml_str = &after_open[..end_idx].trim();
    let rest = &after_open[end_idx + 4..]; // Skip past the closing ---

    // Parse YAML into HashMap
    let frontmatter: HashMap<String, serde_yaml::Value> = serde_yaml::from_str(yaml_str).ok()?;

    // Convert all values to strings
    let string_map: HashMap<String, String> = frontmatter
        .into_iter()
        .map(|(k, v)| {
            let s = match v {
                serde_yaml::Value::String(s) => s,
                serde_yaml::Value::Number(n) => n.to_string(),
                serde_yaml::Value::Bool(b) => b.to_string(),
                _ => v.as_str().unwrap_or("").to_string(),
            };
            (k, s)
        })
        .collect();

    Some((string_map, rest))
}

fn parse_filename(filename: &str) -> Result<(String, String, String)> {
    // YYYYMMDD-HHMM-TYPE-PROJECT-description.md
    // Project names don't contain hyphens (gamesvc, global, etc.)
    let re = Regex::new(r"^(\d{8}-\d{4})-([A-Z]+)-([a-z0-9]+)-")?;

    if let Some(caps) = re.captures(filename) {
        Ok((
            caps.get(1).map_or("", |m| m.as_str()).to_string(),
            caps.get(2).map_or("", |m| m.as_str()).to_string(),
            caps.get(3).map_or("", |m| m.as_str()).to_string(),
        ))
    } else {
        // Try simpler patterns for legacy or manually created tickets
        let parts: Vec<&str> = filename.trim_end_matches(".md").split('-').collect();
        if parts.len() >= 4 {
            Ok((
                format!("{}-{}", parts[0], parts[1]),
                parts[2].to_string(),
                parts[3].to_string(),
            ))
        } else {
            anyhow::bail!("Could not parse filename: {}", filename)
        }
    }
}

fn extract_field(content: &str, field: &str) -> Option<String> {
    let pattern = format!(r"\*\*{}\*\*:\s*(.+)", field);
    let re = Regex::new(&pattern).ok()?;

    re.captures(content)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().trim().to_string())
}

fn extract_summary(content: &str) -> String {
    // Try to find the Summary section
    if let Some(idx) = content.find("## Summary") {
        let after = &content[idx + 10..];
        if let Some(line) = after.lines().find(|l| !l.trim().is_empty()) {
            let summary = line.trim();
            if !summary.starts_with('#') && !summary.starts_with('[') {
                return summary.to_string();
            }
        }
    }

    // Fall back to first non-header, non-metadata line
    for line in content.lines() {
        let trimmed = line.trim();
        if !trimmed.is_empty()
            && !trimmed.starts_with('#')
            && !trimmed.starts_with('-')
            && !trimmed.starts_with('*')
            && !trimmed.starts_with('|')
        {
            return trimmed.chars().take(100).collect();
        }
    }

    "No summary".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_filename() {
        let (ts, tt, proj) =
            parse_filename("20241221-1430-FEAT-gamesvc-add-leaderboard.md").unwrap();
        assert_eq!(ts, "20241221-1430");
        assert_eq!(tt, "FEAT");
        assert_eq!(proj, "gamesvc");
    }

    #[test]
    fn test_parse_filename_investigation() {
        let (ts, tt, proj) = parse_filename("20241221-1520-INV-global-500-errors.md").unwrap();
        assert_eq!(ts, "20241221-1520");
        assert_eq!(tt, "INV");
        assert_eq!(proj, "global");
    }
}

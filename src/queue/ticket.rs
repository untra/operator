#![allow(dead_code)]

use anyhow::{Context, Result};
use chrono::Local;
use regex::Regex;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use ts_rs::TS;

use crate::templates::{schema::TemplateSchema, TemplateType};

/// LLM task metadata for delegate mode integration
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, JsonSchema, TS)]
#[ts(export)]
pub struct LlmTask {
    /// LLM task ID (e.g., Claude delegate mode task UUID)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// LLM task status: "open" or "resolved"
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    /// List of task IDs that must resolve before this task
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub blocked_by: Vec<String>,
}

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
    /// Claude session IDs per step (step_name -> session_uuid)
    pub sessions: HashMap<String, String>,
    /// LLM task metadata for delegate mode integration
    pub llm_task: LlmTask,
    /// Path to the git worktree for this ticket (per-ticket isolation)
    pub worktree_path: Option<String>,
    /// Git branch name for this ticket
    pub branch: Option<String>,
    /// External issue ID from kanban provider (e.g., "PROJ-123" for Jira, "ENG-456" for Linear)
    pub external_id: Option<String>,
    /// Full URL to the issue in the external provider's web UI
    pub external_url: Option<String>,
    /// Provider name for the external issue (e.g., "jira", "linear")
    pub external_provider: Option<String>,
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
        let (
            id,
            priority,
            status,
            step,
            summary,
            sessions,
            llm_task,
            worktree_path,
            branch,
            external_id,
            external_url,
            external_provider,
        ) = if let Some((frontmatter, sessions, llm_task, body)) = extract_frontmatter(&content) {
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
            // Extract worktree fields
            let worktree_path = frontmatter.get("worktree_path").cloned();
            let branch = frontmatter.get("branch").cloned();
            // Extract external kanban provider fields
            let external_id = frontmatter.get("external_id").cloned();
            let external_url = frontmatter.get("external_url").cloned();
            let external_provider = frontmatter.get("external_provider").cloned();
            // Extract summary from body (after frontmatter)
            let summary = extract_summary(body);
            (
                id,
                priority,
                status,
                step,
                summary,
                sessions,
                llm_task,
                worktree_path,
                branch,
                external_id,
                external_url,
                external_provider,
            )
        } else {
            // Legacy parsing using regex for inline metadata
            let id = extract_field(&content, "ID")
                .unwrap_or_else(|| format!("{}-{}", ticket_type, timestamp.replace('-', "")));
            let priority =
                extract_field(&content, "Priority").unwrap_or_else(|| "P2-medium".to_string());
            let status = extract_field(&content, "Status").unwrap_or_else(|| "queued".to_string());
            let step = extract_field(&content, "Step").unwrap_or_default();
            let summary = extract_summary(&content);
            (
                id,
                priority,
                status,
                step,
                summary,
                HashMap::new(),
                LlmTask::default(),
                None,
                None,
                None,
                None,
                None,
            )
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
            sessions,
            llm_task,
            worktree_path,
            branch,
            external_id,
            external_url,
            external_provider,
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
        if let Some((mut frontmatter, sessions, llm_task, body)) =
            extract_frontmatter(&self.content)
        {
            frontmatter.insert(field.to_string(), value.to_string());

            // Rebuild the frontmatter
            let mut yaml_lines = Vec::new();
            for (k, v) in &frontmatter {
                yaml_lines.push(format!("{}: {}", k, v));
            }

            // Add sessions if present
            if !sessions.is_empty() {
                yaml_lines.push("sessions:".to_string());
                for (step, session_id) in &sessions {
                    yaml_lines.push(format!("  {}: {}", step, session_id));
                }
            }

            // Add llm_task if present
            if llm_task.id.is_some() || llm_task.status.is_some() || !llm_task.blocked_by.is_empty()
            {
                yaml_lines.push("llm_task:".to_string());
                if let Some(ref id) = llm_task.id {
                    yaml_lines.push(format!("  id: {}", id));
                }
                if let Some(ref status) = llm_task.status {
                    yaml_lines.push(format!("  status: {}", status));
                }
                if !llm_task.blocked_by.is_empty() {
                    yaml_lines.push("  blocked_by:".to_string());
                    for task_id in &llm_task.blocked_by {
                        yaml_lines.push(format!("    - {}", task_id));
                    }
                }
            }

            yaml_lines.sort(); // Keep consistent order

            let new_content = format!("---\n{}\n---{}", yaml_lines.join("\n"), body);
            self.content = new_content.clone();

            // Update the in-memory field
            match field {
                "step" => self.step = value.to_string(),
                "status" => self.status = value.to_string(),
                "priority" => self.priority = value.to_string(),
                "worktree_path" => self.worktree_path = Some(value.to_string()),
                "branch" => self.branch = Some(value.to_string()),
                "external_id" => self.external_id = Some(value.to_string()),
                "external_url" => self.external_url = Some(value.to_string()),
                "external_provider" => self.external_provider = Some(value.to_string()),
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
            .map(|s| s.requires_review())
            .unwrap_or(false)
    }

    /// Get the session ID for a specific step
    pub fn get_session_id(&self, step_name: &str) -> Option<&String> {
        self.sessions.get(step_name)
    }

    /// Set the session ID for a specific step and save to frontmatter
    pub fn set_session_id(&mut self, step_name: &str, session_id: &str) -> Result<()> {
        self.sessions
            .insert(step_name.to_string(), session_id.to_string());
        self.save_sessions_to_frontmatter()
    }

    /// Set the LLM task ID and save to frontmatter
    pub fn set_llm_task_id(&mut self, id: &str) -> Result<()> {
        self.llm_task.id = Some(id.to_string());
        self.save_llm_task_to_frontmatter()
    }

    /// Set the LLM task status and save to frontmatter
    pub fn set_llm_task_status(&mut self, status: &str) -> Result<()> {
        self.llm_task.status = Some(status.to_string());
        self.save_llm_task_to_frontmatter()
    }

    /// Set the LLM task blocked_by list and save to frontmatter
    pub fn set_llm_task_blocked_by(&mut self, blocked_by: Vec<String>) -> Result<()> {
        self.llm_task.blocked_by = blocked_by;
        self.save_llm_task_to_frontmatter()
    }

    /// Set the worktree path and save to frontmatter
    pub fn set_worktree_path(&mut self, path: &str) -> Result<()> {
        self.worktree_path = Some(path.to_string());
        self.update_field("worktree_path", path)
    }

    /// Set the branch name and save to frontmatter
    pub fn set_branch(&mut self, branch: &str) -> Result<()> {
        self.branch = Some(branch.to_string());
        self.update_field("branch", branch)
    }

    /// Check if this ticket has a worktree
    pub fn has_worktree(&self) -> bool {
        self.worktree_path.is_some()
    }

    /// Save the LLM task to the ticket frontmatter
    fn save_llm_task_to_frontmatter(&mut self) -> Result<()> {
        let content = self.content.trim_start();

        if !content.starts_with("---") {
            // No frontmatter, create one
            let llm_task_yaml = self.format_llm_task_yaml();
            let new_content = format!(
                "---\nid: {}\nstatus: {}\npriority: {}\nstep: {}\n{}\n---\n{}",
                self.id, self.status, self.priority, self.step, llm_task_yaml, content
            );
            self.content = new_content.clone();
            fs::write(&self.filepath, new_content).context("Failed to write ticket file")?;
            return Ok(());
        }

        // Find the closing ---
        let after_open = &content[3..];
        if let Some(end_idx) = after_open.find("\n---") {
            let yaml_str = &after_open[..end_idx];
            let rest = &after_open[end_idx + 4..]; // Content after closing ---

            // Parse existing YAML
            let mut frontmatter: serde_yaml::Value = serde_yaml::from_str(yaml_str)
                .unwrap_or(serde_yaml::Value::Mapping(serde_yaml::Mapping::new()));

            // Update llm_task in the YAML
            if let serde_yaml::Value::Mapping(ref mut map) = frontmatter {
                // Build llm_task mapping
                let mut llm_task_map = serde_yaml::Mapping::new();
                if let Some(ref id) = self.llm_task.id {
                    llm_task_map.insert(
                        serde_yaml::Value::String("id".to_string()),
                        serde_yaml::Value::String(id.clone()),
                    );
                }
                if let Some(ref status) = self.llm_task.status {
                    llm_task_map.insert(
                        serde_yaml::Value::String("status".to_string()),
                        serde_yaml::Value::String(status.clone()),
                    );
                }
                if !self.llm_task.blocked_by.is_empty() {
                    let blocked_by_seq: Vec<serde_yaml::Value> = self
                        .llm_task
                        .blocked_by
                        .iter()
                        .map(|s| serde_yaml::Value::String(s.clone()))
                        .collect();
                    llm_task_map.insert(
                        serde_yaml::Value::String("blocked_by".to_string()),
                        serde_yaml::Value::Sequence(blocked_by_seq),
                    );
                }

                // Only add llm_task if it has any data
                if !llm_task_map.is_empty() {
                    map.insert(
                        serde_yaml::Value::String("llm_task".to_string()),
                        serde_yaml::Value::Mapping(llm_task_map),
                    );
                }
            }

            // Serialize back to YAML
            let new_yaml =
                serde_yaml::to_string(&frontmatter).context("Failed to serialize frontmatter")?;

            let new_content = format!("---\n{}---{}", new_yaml, rest);
            self.content = new_content.clone();
            fs::write(&self.filepath, new_content).context("Failed to write ticket file")?;
        }

        Ok(())
    }

    /// Format LLM task as YAML for frontmatter
    fn format_llm_task_yaml(&self) -> String {
        if self.llm_task.id.is_none()
            && self.llm_task.status.is_none()
            && self.llm_task.blocked_by.is_empty()
        {
            return String::new();
        }

        let mut lines = vec!["llm_task:".to_string()];
        if let Some(ref id) = self.llm_task.id {
            lines.push(format!("  id: {}", id));
        }
        if let Some(ref status) = self.llm_task.status {
            lines.push(format!("  status: {}", status));
        }
        if !self.llm_task.blocked_by.is_empty() {
            lines.push("  blocked_by:".to_string());
            for task_id in &self.llm_task.blocked_by {
                lines.push(format!("    - {}", task_id));
            }
        }
        lines.join("\n")
    }

    /// Save the sessions map to the ticket frontmatter
    fn save_sessions_to_frontmatter(&mut self) -> Result<()> {
        let content = self.content.trim_start();

        if !content.starts_with("---") {
            // No frontmatter, create one
            let sessions_yaml = self.format_sessions_yaml();
            let new_content = format!(
                "---\nid: {}\nstatus: {}\npriority: {}\nstep: {}\n{}\n---\n{}",
                self.id, self.status, self.priority, self.step, sessions_yaml, content
            );
            self.content = new_content.clone();
            fs::write(&self.filepath, new_content).context("Failed to write ticket file")?;
            return Ok(());
        }

        // Find the closing ---
        let after_open = &content[3..];
        if let Some(end_idx) = after_open.find("\n---") {
            let yaml_str = &after_open[..end_idx];
            let rest = &after_open[end_idx + 4..]; // Content after closing ---

            // Parse existing YAML
            let mut frontmatter: serde_yaml::Value = serde_yaml::from_str(yaml_str)
                .unwrap_or(serde_yaml::Value::Mapping(serde_yaml::Mapping::new()));

            // Update sessions in the YAML
            if let serde_yaml::Value::Mapping(ref mut map) = frontmatter {
                // Build sessions mapping
                let mut sessions_map = serde_yaml::Mapping::new();
                for (step, session_id) in &self.sessions {
                    sessions_map.insert(
                        serde_yaml::Value::String(step.clone()),
                        serde_yaml::Value::String(session_id.clone()),
                    );
                }
                map.insert(
                    serde_yaml::Value::String("sessions".to_string()),
                    serde_yaml::Value::Mapping(sessions_map),
                );
            }

            // Serialize back to YAML
            let new_yaml =
                serde_yaml::to_string(&frontmatter).context("Failed to serialize frontmatter")?;

            let new_content = format!("---\n{}---{}", new_yaml, rest);
            self.content = new_content.clone();
            fs::write(&self.filepath, new_content).context("Failed to write ticket file")?;
        }

        Ok(())
    }

    /// Format sessions as YAML for frontmatter
    fn format_sessions_yaml(&self) -> String {
        if self.sessions.is_empty() {
            return String::new();
        }

        let mut lines = vec!["sessions:".to_string()];
        for (step, session_id) in &self.sessions {
            lines.push(format!("  {}: {}", step, session_id));
        }
        lines.join("\n")
    }
}

/// Extract sessions mapping from YAML frontmatter value
fn extract_sessions_from_yaml(
    frontmatter: &HashMap<String, serde_yaml::Value>,
) -> HashMap<String, String> {
    if let Some(serde_yaml::Value::Mapping(map)) = frontmatter.get("sessions") {
        map.iter()
            .filter_map(|(k, v)| {
                let key = k.as_str()?.to_string();
                let value = v.as_str()?.to_string();
                Some((key, value))
            })
            .collect()
    } else {
        HashMap::new()
    }
}

/// Extract LLM task metadata from YAML frontmatter value
fn extract_llm_task_from_yaml(frontmatter: &HashMap<String, serde_yaml::Value>) -> LlmTask {
    if let Some(serde_yaml::Value::Mapping(map)) = frontmatter.get("llm_task") {
        let id = map
            .get(serde_yaml::Value::String("id".to_string()))
            .and_then(|v| v.as_str())
            .map(String::from);

        let status = map
            .get(serde_yaml::Value::String("status".to_string()))
            .and_then(|v| v.as_str())
            .map(String::from);

        let blocked_by = map
            .get(serde_yaml::Value::String("blocked_by".to_string()))
            .and_then(|v| v.as_sequence())
            .map(|seq| {
                seq.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        LlmTask {
            id,
            status,
            blocked_by,
        }
    } else {
        LlmTask::default()
    }
}

/// Extract YAML frontmatter from markdown content
/// Returns the parsed frontmatter as a HashMap, sessions HashMap, LlmTask, and the content after the frontmatter
#[allow(clippy::type_complexity)]
fn extract_frontmatter(
    content: &str,
) -> Option<(
    HashMap<String, String>,
    HashMap<String, String>,
    LlmTask,
    &str,
)> {
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

    // Extract sessions before converting to strings
    let sessions = extract_sessions_from_yaml(&frontmatter);

    // Extract LLM task metadata
    let llm_task = extract_llm_task_from_yaml(&frontmatter);

    // Convert scalar values to strings (skip mappings like sessions, llm_task)
    let string_map: HashMap<String, String> = frontmatter
        .iter()
        .filter_map(|(k, v)| {
            let s = match v {
                serde_yaml::Value::String(s) => s.clone(),
                serde_yaml::Value::Number(n) => n.to_string(),
                serde_yaml::Value::Bool(b) => b.to_string(),
                serde_yaml::Value::Null => String::new(),
                // Skip mappings/sequences - they're handled separately
                serde_yaml::Value::Mapping(_) | serde_yaml::Value::Sequence(_) => return None,
                _ => v.as_str().unwrap_or("").to_string(),
            };
            Some((k.clone(), s))
        })
        .collect();

    Some((string_map, sessions, llm_task, rest))
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
    // Try to find the "# Type: Summary" header pattern used by templates
    // Supports: # Feature: X, # Fix: X, # Spike: X, # Investigation: X, # Task: X
    let type_header_pattern =
        Regex::new(r"^#\s+(?:Feature|Fix|Spike|Investigation|Task):\s*(.+)$").unwrap();

    for line in content.lines() {
        if let Some(caps) = type_header_pattern.captures(line.trim()) {
            if let Some(summary) = caps.get(1) {
                let text = summary.as_str().trim();
                if !text.is_empty() {
                    return text.to_string();
                }
            }
        }
    }

    // Try to find the ## Summary section (legacy format)
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

    #[test]
    fn test_extract_summary_from_feature_header() {
        // Summary should be extracted from "# Feature: X" format
        let content = r#"
# Feature: Add user authentication

## Context
This is the context.
"#;
        let summary = extract_summary(content);
        assert_eq!(summary, "Add user authentication");
    }

    #[test]
    fn test_extract_summary_from_fix_header() {
        // Summary should be extracted from "# Fix: X" format
        let content = r#"
# Fix: Resolve login timeout issue

## Context
Users are experiencing timeouts.
"#;
        let summary = extract_summary(content);
        assert_eq!(summary, "Resolve login timeout issue");
    }

    #[test]
    fn test_extract_summary_from_spike_header() {
        let content = r#"
# Spike: Investigate caching strategies

## Context
Need to explore caching options.
"#;
        let summary = extract_summary(content);
        assert_eq!(summary, "Investigate caching strategies");
    }

    #[test]
    fn test_extract_summary_from_investigation_header() {
        let content = r#"
# Investigation: Database connection failures

## Observed Behavior
Connections are dropping.
"#;
        let summary = extract_summary(content);
        assert_eq!(summary, "Database connection failures");
    }

    #[test]
    fn test_extract_summary_from_task_header() {
        let content = r#"
# Task: Update dependencies

## Context
Routine maintenance.
"#;
        let summary = extract_summary(content);
        assert_eq!(summary, "Update dependencies");
    }

    #[test]
    fn test_extract_summary_from_summary_section() {
        // Legacy format with ## Summary section should still work
        let content = r#"
## Summary
This is the summary text.

## Details
More details here.
"#;
        let summary = extract_summary(content);
        assert_eq!(summary, "This is the summary text.");
    }

    #[test]
    fn test_extract_summary_fallback_to_first_line() {
        // When no recognized format, should fall back to first non-header line
        let content = r#"
This is just some text without headers.
"#;
        let summary = extract_summary(content);
        assert_eq!(summary, "This is just some text without headers.");
    }

    #[test]
    fn test_extract_summary_returns_no_summary_when_empty() {
        let content = "";
        let summary = extract_summary(content);
        assert_eq!(summary, "No summary");
    }

    #[test]
    fn test_extract_frontmatter_with_empty_step() {
        // Frontmatter with empty step should return empty string
        let content = r#"---
id: FEAT-1234
step:
status: queued
---

# Feature: Test feature
"#;
        let (frontmatter, _sessions, _llm_task, _body) = extract_frontmatter(content).unwrap();
        let step = frontmatter.get("step").cloned().unwrap_or_default();
        assert!(
            step.is_empty(),
            "Empty step should be empty string, got: '{}'",
            step
        );
    }

    #[test]
    fn test_extract_frontmatter_without_step() {
        // Frontmatter without step field should be handled gracefully
        let content = r#"---
id: FEAT-1234
status: queued
---

# Feature: Test feature
"#;
        let (frontmatter, _sessions, _llm_task, _body) = extract_frontmatter(content).unwrap();
        let step = frontmatter.get("step").cloned().unwrap_or_default();
        assert!(
            step.is_empty(),
            "Missing step should default to empty string"
        );
    }

    #[test]
    fn test_ticket_id_does_not_duplicate_type() {
        // The ticket.id field should be the full ID like "FEAT-1234"
        // and should NOT be duplicated when displayed
        let content = r#"---
id: FEAT-7598
status: queued
project: operator
---

# Feature: Test summary
"#;

        // Create temp file for testing
        let temp_dir = tempfile::tempdir().unwrap();
        let ticket_path = temp_dir.path().join("20241221-1430-FEAT-operator-test.md");
        std::fs::write(&ticket_path, content).unwrap();

        let ticket = Ticket::from_file(&ticket_path).unwrap();

        // The ID should be exactly "FEAT-7598", not "FEAT-FEAT-7598"
        assert_eq!(
            ticket.id, "FEAT-7598",
            "ID should be FEAT-7598, not duplicated"
        );
        assert_eq!(ticket.ticket_type, "FEAT");

        // Verify we don't get duplication when formatting for display
        let display_id = &ticket.id; // Should use this directly, not format!("{}-{}", type, id)
        assert!(
            !display_id.starts_with("FEAT-FEAT"),
            "Display ID should not have duplicated prefix"
        );
    }

    #[test]
    fn test_sessions_frontmatter_parsing() {
        // Frontmatter with sessions should parse correctly
        let content = r#"---
id: FEAT-1234
status: running
step: implement
sessions:
  plan: 550e8400-e29b-41d4-a716-446655440000
  implement: 6ba7b810-9dad-11d1-80b4-00c04fd430c8
---

# Feature: Test feature
"#;
        let (frontmatter, sessions, _llm_task, _body) = extract_frontmatter(content).unwrap();
        assert_eq!(frontmatter.get("id").unwrap(), "FEAT-1234");
        assert_eq!(sessions.len(), 2);
        assert_eq!(
            sessions.get("plan").unwrap(),
            "550e8400-e29b-41d4-a716-446655440000"
        );
        assert_eq!(
            sessions.get("implement").unwrap(),
            "6ba7b810-9dad-11d1-80b4-00c04fd430c8"
        );
    }

    #[test]
    fn test_sessions_empty_when_not_present() {
        let content = r#"---
id: FEAT-1234
status: queued
---

# Feature: Test feature
"#;
        let (_frontmatter, sessions, _llm_task, _body) = extract_frontmatter(content).unwrap();
        assert!(sessions.is_empty());
    }

    #[test]
    fn test_ticket_from_file_with_sessions() {
        let content = r#"---
id: FEAT-5678
status: running
step: implement
sessions:
  plan: 550e8400-e29b-41d4-a716-446655440000
  implement: 6ba7b810-9dad-11d1-80b4-00c04fd430c8
---

# Feature: Test with sessions
"#;
        let temp_dir = tempfile::tempdir().unwrap();
        let ticket_path = temp_dir.path().join("20241221-1430-FEAT-operator-test.md");
        std::fs::write(&ticket_path, content).unwrap();

        let ticket = Ticket::from_file(&ticket_path).unwrap();

        assert_eq!(ticket.sessions.len(), 2);
        assert_eq!(
            ticket.get_session_id("plan").unwrap(),
            "550e8400-e29b-41d4-a716-446655440000"
        );
        assert_eq!(
            ticket.get_session_id("implement").unwrap(),
            "6ba7b810-9dad-11d1-80b4-00c04fd430c8"
        );
    }

    #[test]
    fn test_set_session_id() {
        let content = r#"---
id: FEAT-9999
status: queued
step: plan
---

# Feature: Test set session
"#;
        let temp_dir = tempfile::tempdir().unwrap();
        let ticket_path = temp_dir.path().join("20241221-1430-FEAT-operator-test.md");
        std::fs::write(&ticket_path, content).unwrap();

        let mut ticket = Ticket::from_file(&ticket_path).unwrap();

        // Set a session ID
        let session_uuid = "abcd1234-5678-90ab-cdef-1234567890ab";
        ticket.set_session_id("plan", session_uuid).unwrap();

        // Verify in memory
        assert_eq!(ticket.get_session_id("plan").unwrap(), session_uuid);

        // Reload from file and verify persistence
        let reloaded = Ticket::from_file(&ticket_path).unwrap();
        assert_eq!(reloaded.get_session_id("plan").unwrap(), session_uuid);
    }

    #[test]
    fn test_set_multiple_session_ids() {
        let content = r#"---
id: FEAT-8888
status: queued
step: plan
---

# Feature: Test multiple sessions
"#;
        let temp_dir = tempfile::tempdir().unwrap();
        let ticket_path = temp_dir.path().join("20241221-1430-FEAT-operator-test.md");
        std::fs::write(&ticket_path, content).unwrap();

        let mut ticket = Ticket::from_file(&ticket_path).unwrap();

        // Set session IDs for multiple steps
        let plan_uuid = "11111111-1111-1111-1111-111111111111";
        let implement_uuid = "22222222-2222-2222-2222-222222222222";

        ticket.set_session_id("plan", plan_uuid).unwrap();
        ticket.set_session_id("implement", implement_uuid).unwrap();

        // Verify in memory
        assert_eq!(ticket.sessions.len(), 2);
        assert_eq!(ticket.get_session_id("plan").unwrap(), plan_uuid);
        assert_eq!(ticket.get_session_id("implement").unwrap(), implement_uuid);

        // Reload from file and verify persistence
        let reloaded = Ticket::from_file(&ticket_path).unwrap();
        assert_eq!(reloaded.sessions.len(), 2);
        assert_eq!(reloaded.get_session_id("plan").unwrap(), plan_uuid);
        assert_eq!(
            reloaded.get_session_id("implement").unwrap(),
            implement_uuid
        );
    }

    #[test]
    fn test_llm_task_frontmatter_parsing() {
        // Frontmatter with llm_task should parse correctly
        let content = r#"---
id: FEAT-1234
status: running
step: implement
llm_task:
  id: abc12345-6789-0abc-def0-123456789abc
  status: open
  blocked_by:
    - task-001
    - task-002
---

# Feature: Test feature
"#;
        let (_frontmatter, _sessions, llm_task, _body) = extract_frontmatter(content).unwrap();
        assert_eq!(
            llm_task.id,
            Some("abc12345-6789-0abc-def0-123456789abc".to_string())
        );
        assert_eq!(llm_task.status, Some("open".to_string()));
        assert_eq!(llm_task.blocked_by.len(), 2);
        assert_eq!(llm_task.blocked_by[0], "task-001");
        assert_eq!(llm_task.blocked_by[1], "task-002");
    }

    #[test]
    fn test_llm_task_empty_when_not_present() {
        let content = r#"---
id: FEAT-1234
status: queued
---

# Feature: Test feature
"#;
        let (_frontmatter, _sessions, llm_task, _body) = extract_frontmatter(content).unwrap();
        assert_eq!(llm_task, LlmTask::default());
    }

    #[test]
    fn test_ticket_from_file_with_llm_task() {
        let content = r#"---
id: FEAT-5678
status: running
step: implement
llm_task:
  id: test-task-uuid
  status: open
---

# Feature: Test with LLM task
"#;
        let temp_dir = tempfile::tempdir().unwrap();
        let ticket_path = temp_dir.path().join("20241221-1430-FEAT-operator-test.md");
        std::fs::write(&ticket_path, content).unwrap();

        let ticket = Ticket::from_file(&ticket_path).unwrap();

        assert_eq!(ticket.llm_task.id, Some("test-task-uuid".to_string()));
        assert_eq!(ticket.llm_task.status, Some("open".to_string()));
        assert!(ticket.llm_task.blocked_by.is_empty());
    }

    #[test]
    fn test_set_llm_task_id() {
        let content = r#"---
id: FEAT-9999
status: queued
step: plan
---

# Feature: Test set LLM task
"#;
        let temp_dir = tempfile::tempdir().unwrap();
        let ticket_path = temp_dir.path().join("20241221-1430-FEAT-operator-test.md");
        std::fs::write(&ticket_path, content).unwrap();

        let mut ticket = Ticket::from_file(&ticket_path).unwrap();

        // Set LLM task ID
        let task_id = "new-task-id-12345";
        ticket.set_llm_task_id(task_id).unwrap();

        // Verify in memory
        assert_eq!(ticket.llm_task.id, Some(task_id.to_string()));

        // Reload from file and verify persistence
        let reloaded = Ticket::from_file(&ticket_path).unwrap();
        assert_eq!(reloaded.llm_task.id, Some(task_id.to_string()));
    }

    #[test]
    fn test_set_llm_task_status() {
        let content = r#"---
id: FEAT-7777
status: queued
step: plan
---

# Feature: Test set LLM task status
"#;
        let temp_dir = tempfile::tempdir().unwrap();
        let ticket_path = temp_dir.path().join("20241221-1430-FEAT-operator-test.md");
        std::fs::write(&ticket_path, content).unwrap();

        let mut ticket = Ticket::from_file(&ticket_path).unwrap();

        // Set LLM task status
        ticket.set_llm_task_status("resolved").unwrap();

        // Verify in memory
        assert_eq!(ticket.llm_task.status, Some("resolved".to_string()));

        // Reload from file and verify persistence
        let reloaded = Ticket::from_file(&ticket_path).unwrap();
        assert_eq!(reloaded.llm_task.status, Some("resolved".to_string()));
    }

    #[test]
    fn test_set_llm_task_blocked_by() {
        let content = r#"---
id: FEAT-6666
status: queued
step: plan
---

# Feature: Test set LLM task blocked_by
"#;
        let temp_dir = tempfile::tempdir().unwrap();
        let ticket_path = temp_dir.path().join("20241221-1430-FEAT-operator-test.md");
        std::fs::write(&ticket_path, content).unwrap();

        let mut ticket = Ticket::from_file(&ticket_path).unwrap();

        // Set LLM task blocked_by
        let blockers = vec!["blocker-1".to_string(), "blocker-2".to_string()];
        ticket.set_llm_task_blocked_by(blockers.clone()).unwrap();

        // Verify in memory
        assert_eq!(ticket.llm_task.blocked_by, blockers);

        // Reload from file and verify persistence
        let reloaded = Ticket::from_file(&ticket_path).unwrap();
        assert_eq!(reloaded.llm_task.blocked_by, blockers);
    }
}

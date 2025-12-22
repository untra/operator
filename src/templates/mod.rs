//! Embedded templates and schemas for ticket creation

pub mod schema;

use once_cell::sync::Lazy;
use schema::TemplateSchema;
use std::collections::HashMap;

/// Cached map of issuetype key to glyph (built at startup)
static GLYPH_MAP: Lazy<HashMap<String, String>> = Lazy::new(|| {
    let mut map = HashMap::new();
    for tt in TemplateType::all() {
        if let Ok(schema) = TemplateSchema::from_json(tt.schema()) {
            map.insert(schema.key.clone(), schema.glyph.clone());
        }
    }
    map
});

/// Cached map of issuetype key to color (built at startup)
static COLOR_MAP: Lazy<HashMap<String, String>> = Lazy::new(|| {
    let mut map = HashMap::new();
    for tt in TemplateType::all() {
        if let Ok(schema) = TemplateSchema::from_json(tt.schema()) {
            if let Some(color) = schema.color {
                map.insert(schema.key.clone(), color);
            }
        }
    }
    map
});

/// Get glyph for a ticket type key, returns "?" if not found
pub fn glyph_for_key(key: &str) -> &str {
    GLYPH_MAP.get(key).map(|s| s.as_str()).unwrap_or("?")
}

/// Get color for a ticket type key, returns None if not set
pub fn color_for_key(key: &str) -> Option<&str> {
    COLOR_MAP.get(key).map(|s| s.as_str())
}

/// Template types supported by the operator
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TemplateType {
    Feature,
    Fix,
    Task,
    Spike,
    Investigation,
}

impl TemplateType {
    /// Returns all template types in display order
    pub fn all() -> &'static [TemplateType] {
        &[
            TemplateType::Feature,
            TemplateType::Fix,
            TemplateType::Task,
            TemplateType::Spike,
            TemplateType::Investigation,
        ]
    }

    /// Returns the short type code used in filenames (e.g., "FEAT", "FIX")
    pub fn as_str(&self) -> &'static str {
        match self {
            TemplateType::Feature => "FEAT",
            TemplateType::Fix => "FIX",
            TemplateType::Task => "TASK",
            TemplateType::Spike => "SPIKE",
            TemplateType::Investigation => "INV",
        }
    }

    /// Returns the human-readable display name
    pub fn display_name(&self) -> &'static str {
        match self {
            TemplateType::Feature => "Feature",
            TemplateType::Fix => "Fix/Bug",
            TemplateType::Task => "Task",
            TemplateType::Spike => "Spike",
            TemplateType::Investigation => "Investigation",
        }
    }

    /// Returns a brief description of when to use this template
    pub fn description(&self) -> &'static str {
        match self {
            TemplateType::Feature => "New feature or enhancement",
            TemplateType::Fix => "Bug fix, follow-up work, tech debt, refactoring",
            TemplateType::Task => "Neutral task that outputs a plan for execution",
            TemplateType::Spike => "Research or exploration (paired mode)",
            TemplateType::Investigation => "Incident investigation (paired mode)",
        }
    }

    /// Returns the embedded markdown template content
    pub fn template_content(&self) -> &'static str {
        match self {
            TemplateType::Feature => include_str!("feature.md"),
            TemplateType::Fix => include_str!("fix.md"),
            TemplateType::Task => include_str!("task.md"),
            TemplateType::Spike => include_str!("spike.md"),
            TemplateType::Investigation => include_str!("investigation.md"),
        }
    }

    /// Returns the embedded JSON schema content
    pub fn schema(&self) -> &'static str {
        match self {
            TemplateType::Feature => include_str!("feature.json"),
            TemplateType::Fix => include_str!("fix.json"),
            TemplateType::Task => include_str!("task.json"),
            TemplateType::Spike => include_str!("spike.json"),
            TemplateType::Investigation => include_str!("investigation.json"),
        }
    }

    /// Returns true if this template type requires paired mode (human interaction)
    pub fn is_paired(&self) -> bool {
        matches!(self, TemplateType::Spike | TemplateType::Investigation)
    }

    /// Returns true if project is optional for this template type
    pub fn project_optional(&self) -> bool {
        matches!(
            self,
            TemplateType::Spike | TemplateType::Investigation | TemplateType::Task
        )
    }

    /// Returns the git branch prefix for this template type
    pub fn branch_prefix(&self) -> &'static str {
        match self {
            TemplateType::Feature => "feature",
            TemplateType::Fix => "fix",
            TemplateType::Task => "task",
            TemplateType::Spike => "spike",
            TemplateType::Investigation => "investigation",
        }
    }

    /// Returns the first step name for this template type
    pub fn first_step(&self) -> &'static str {
        match self {
            TemplateType::Feature => "plan",
            TemplateType::Fix => "reproduce",
            TemplateType::Task => "analyze",
            TemplateType::Spike => "explore",
            TemplateType::Investigation => "triage",
        }
    }

    /// Parse template type from string key (e.g., "FEAT", "FIX", "TASK")
    pub fn from_key(key: &str) -> Option<TemplateType> {
        match key.to_uppercase().as_str() {
            "FEAT" => Some(TemplateType::Feature),
            "FIX" => Some(TemplateType::Fix),
            "TASK" => Some(TemplateType::Task),
            "SPIKE" => Some(TemplateType::Spike),
            "INV" => Some(TemplateType::Investigation),
            _ => None,
        }
    }
}

impl std::fmt::Display for TemplateType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

//! Embedded templates and schemas for ticket creation
//!
//! This module provides built-in template types and schemas.
//! For dynamic issue types and collections, see the `issuetypes` module.

#![allow(dead_code)] // Registry helper functions used when registry is integrated

pub mod schema;
pub mod step_type;

use crate::issuetypes::IssueTypeRegistry;
use schema::TemplateSchema;
use std::collections::HashMap;

/// Cached map of issuetype key to glyph (built at startup from builtin types)
static GLYPH_MAP: std::sync::LazyLock<HashMap<String, String>> = std::sync::LazyLock::new(|| {
    let mut map = HashMap::new();
    for tt in TemplateType::all() {
        if let Ok(schema) = TemplateSchema::from_json(tt.schema()) {
            map.insert(schema.key.clone(), schema.glyph.clone());
        }
    }
    map
});

/// Cached map of issuetype key to color (built at startup from builtin types)
static COLOR_MAP: std::sync::LazyLock<HashMap<String, String>> = std::sync::LazyLock::new(|| {
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

/// Get glyph for a ticket type key
/// Returns "?" if not found in static maps
pub fn glyph_for_key(key: &str) -> &str {
    GLYPH_MAP.get(key).map_or("?", std::string::String::as_str)
}

/// Get glyph for a ticket type key, checking registry first
/// Falls back to static map if not in registry
pub fn glyph_for_key_with_registry(key: &str, registry: &IssueTypeRegistry) -> String {
    if let Some(issue_type) = registry.get(key) {
        issue_type.glyph.clone()
    } else {
        glyph_for_key(key).to_string()
    }
}

/// Get color for a ticket type key, returns None if not set
pub fn color_for_key(key: &str) -> Option<&str> {
    COLOR_MAP.get(key).map(std::string::String::as_str)
}

/// Get color for a ticket type key, checking registry first
/// Falls back to static map if not in registry
pub fn color_for_key_with_registry(key: &str, registry: &IssueTypeRegistry) -> Option<String> {
    if let Some(issue_type) = registry.get(key) {
        issue_type.color.clone()
    } else {
        color_for_key(key).map(std::string::ToString::to_string)
    }
}

/// Check if a key represents a paired mode type, checking registry first
pub fn is_paired_with_registry(key: &str, registry: &IssueTypeRegistry) -> bool {
    if let Some(issue_type) = registry.get(key) {
        issue_type.is_paired()
    } else if let Some(tt) = TemplateType::from_key(key) {
        tt.is_paired()
    } else {
        false // Unknown types default to autonomous
    }
}

/// Template types supported by the operator
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TemplateType {
    Feature,
    Fix,
    Task,
    Spike,
    Investigation,
    Assess,
    Sync,
    Init,
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
            TemplateType::Assess,
            TemplateType::Sync,
            TemplateType::Init,
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
            TemplateType::Assess => "ASSESS",
            TemplateType::Sync => "SYNC",
            TemplateType::Init => "INIT",
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
            TemplateType::Assess => "Project Assessment",
            TemplateType::Sync => "Catalog Sync",
            TemplateType::Init => "Workspace Init",
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
            TemplateType::Assess => "Analyze project and generate catalog-info.yaml",
            TemplateType::Sync => "Refresh catalog entries",
            TemplateType::Init => "Initialize workspace (paired mode)",
        }
    }

    /// Returns the embedded markdown template content
    /// Source of truth: `src/collections/full/`
    pub fn template_content(&self) -> &'static str {
        match self {
            TemplateType::Feature => include_str!("../collections/full/FEAT.md"),
            TemplateType::Fix => include_str!("../collections/full/FIX.md"),
            TemplateType::Task => include_str!("../collections/full/TASK.md"),
            TemplateType::Spike => include_str!("../collections/full/SPIKE.md"),
            TemplateType::Investigation => include_str!("../collections/full/INV.md"),
            TemplateType::Assess => include_str!("../collections/full/ASSESS.md"),
            TemplateType::Sync => include_str!("../collections/full/SYNC.md"),
            TemplateType::Init => include_str!("../collections/full/INIT.md"),
        }
    }

    /// Returns the embedded JSON schema content
    /// Source of truth: `src/collections/full/`
    pub fn schema(&self) -> &'static str {
        match self {
            TemplateType::Feature => include_str!("../collections/full/FEAT.json"),
            TemplateType::Fix => include_str!("../collections/full/FIX.json"),
            TemplateType::Task => include_str!("../collections/full/TASK.json"),
            TemplateType::Spike => include_str!("../collections/full/SPIKE.json"),
            TemplateType::Investigation => include_str!("../collections/full/INV.json"),
            TemplateType::Assess => include_str!("../collections/full/ASSESS.json"),
            TemplateType::Sync => include_str!("../collections/full/SYNC.json"),
            TemplateType::Init => include_str!("../collections/full/INIT.json"),
        }
    }

    /// Returns true if this template type requires paired mode (human interaction)
    pub fn is_paired(&self) -> bool {
        matches!(
            self,
            TemplateType::Spike | TemplateType::Investigation | TemplateType::Init
        )
    }

    /// Returns true if project is optional for this template type
    pub fn project_optional(&self) -> bool {
        matches!(
            self,
            TemplateType::Spike
                | TemplateType::Investigation
                | TemplateType::Task
                | TemplateType::Sync
                | TemplateType::Init
        )
    }

    /// Returns the git branch prefix for this template type (derived from key)
    pub fn branch_prefix(&self) -> String {
        self.as_str().to_lowercase()
    }

    /// Returns the first step name for this template type
    pub fn first_step(&self) -> &'static str {
        match self {
            TemplateType::Feature => "plan",
            TemplateType::Fix => "reproduce",
            TemplateType::Task => "analyze",
            TemplateType::Spike => "explore",
            TemplateType::Investigation => "triage",
            TemplateType::Assess => "analyze",
            TemplateType::Sync => "scan",
            TemplateType::Init => "scaffold",
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
            "ASSESS" => Some(TemplateType::Assess),
            "SYNC" => Some(TemplateType::Sync),
            "INIT" => Some(TemplateType::Init),
            _ => None,
        }
    }
}

impl std::fmt::Display for TemplateType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// (key, glyph, color) source-of-truth, mirroring `src/collections/full/*.json`.
    /// A drift between the embedded schemas and these expectations should fail.
    const KEY_GLYPH_COLOR: &[(&str, &str, &str)] = &[
        ("FEAT", "*", "green"),
        ("FIX", "#", "magenta"),
        ("TASK", ">", "cyan"),
        ("SPIKE", "?", "blue"),
        ("INV", "!", "yellow"),
        ("ASSESS", "~", "magenta"),
        ("SYNC", "@", "blue"),
        ("INIT", "%", "green"),
    ];

    #[test]
    fn test_glyph_for_key_known_types_map_to_expected_glyphs() {
        for (key, glyph, _) in KEY_GLYPH_COLOR {
            assert_eq!(
                glyph_for_key(key),
                *glyph,
                "glyph for {key} should be {glyph}"
            );
        }
    }

    #[test]
    fn test_color_for_key_known_types_map_to_expected_colors() {
        for (key, _, color) in KEY_GLYPH_COLOR {
            assert_eq!(
                color_for_key(key),
                Some(*color),
                "color for {key} should be {color}"
            );
        }
    }

    #[test]
    fn test_glyph_for_key_unknown_returns_question_fallback() {
        // Keys genuinely absent from the map (deliberately NOT "SPIKE", whose
        // real glyph also happens to be "?").
        assert_eq!(glyph_for_key("NOPE"), "?");
        assert_eq!(glyph_for_key(""), "?");
        assert_eq!(glyph_for_key("feat"), "?", "lookup is case-sensitive");
    }

    #[test]
    fn test_color_for_key_unknown_returns_none() {
        assert_eq!(color_for_key("NOPE"), None);
        assert_eq!(color_for_key(""), None);
        assert_eq!(color_for_key("feat"), None, "lookup is case-sensitive");
    }

    #[test]
    fn test_glyph_and_color_maps_cover_every_template_type() {
        // Every builtin TemplateType must have an entry in both maps. Asserted
        // against the maps directly (not via glyph_for_key) because SPIKE's real
        // glyph is "?", indistinguishable from the lookup fallback. Guards
        // against adding an enum variant whose schema omits glyph/color.
        for tt in TemplateType::all() {
            let key = tt.as_str();
            assert!(GLYPH_MAP.contains_key(key), "{key} missing from GLYPH_MAP");
            assert!(COLOR_MAP.contains_key(key), "{key} missing from COLOR_MAP");
        }
    }
}

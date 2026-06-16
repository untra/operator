//! The output-format selector for workflow export.
//!
//! Operator can render a ticket+issuetype into more than one orchestration
//! format. `Claude` is the original Claude Code dynamic-workflow `.js`; `Agnt`
//! is the AGNT.gg workflow graph JSON. The same shared code path
//! (`export_workflow_for_ticket`) dispatches on this enum so every surface — CLI,
//! REST, TUI, VS Code — selects a format uniformly.

use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Which orchestration format `workflow export` should emit.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, ValueEnum, ToSchema,
)]
#[serde(rename_all = "lowercase")]
pub enum WorkflowFormat {
    /// Claude Code dynamic workflow (`.js`).
    #[default]
    Claude,
    /// AGNT.gg workflow graph (`.json`).
    Agnt,
}

impl WorkflowFormat {
    /// Every format, in catalog/display order. Source of truth cross-checked by
    /// `tests/vertical_parity.rs` against the `Workflows` catalog vertical.
    pub const ALL: [WorkflowFormat; 2] = [WorkflowFormat::Claude, WorkflowFormat::Agnt];

    /// Stable lowercase slug — must equal the `Workflows` catalog entry slug.
    pub fn slug(&self) -> &'static str {
        match self {
            WorkflowFormat::Claude => "claude",
            WorkflowFormat::Agnt => "agnt",
        }
    }

    /// Human label, matching the `Workflows` catalog entry label.
    pub fn label(&self) -> &'static str {
        match self {
            WorkflowFormat::Claude => "Claude Workflow",
            WorkflowFormat::Agnt => "AGNT Workflow",
        }
    }

    /// File extension of the emitted artifact (no leading dot).
    pub fn extension(&self) -> &'static str {
        match self {
            WorkflowFormat::Claude => "js",
            WorkflowFormat::Agnt => "json",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_variants_have_distinct_slugs() {
        let slugs: Vec<_> = WorkflowFormat::ALL
            .iter()
            .map(WorkflowFormat::slug)
            .collect();
        assert_eq!(slugs, vec!["claude", "agnt"]);
    }

    #[test]
    fn slug_matches_serde_lowercase() {
        // The catalog parity + REST DTO rely on slug() equaling the serde repr.
        for f in WorkflowFormat::ALL {
            let json = serde_json::to_string(&f).unwrap();
            assert_eq!(json, format!("\"{}\"", f.slug()));
        }
    }

    #[test]
    fn extensions_are_known() {
        assert_eq!(WorkflowFormat::Claude.extension(), "js");
        assert_eq!(WorkflowFormat::Agnt.extension(), "json");
    }
}

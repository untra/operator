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

//! Status section DTOs for `GET /api/v1/sections`.
//!
//! These mirror the canonical status sections shared with the TUI and the VS
//! Code extension. The section *logic* (health rules, child rows) lives in the
//! TUI layer (`crate::ui`), which the shared `rest` module cannot depend on:
//! `rest` compiles into the lib crate, and the lib crate has no `ui` module.
//!
//! To keep that boundary while still running the real section logic, the binary
//! injects a provider via [`register_section_provider`] at startup. In lib-only
//! and test contexts no provider is registered, so the endpoint returns an empty
//! list — the section logic is exercised by the ui-side builder's own tests.

use std::sync::{Arc, OnceLock};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;

use crate::config::Config;
use crate::issuetypes::IssueTypeRegistry;

/// A child row within a status section.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct SectionRowDto {
    /// Stable, section-scoped row id. Clients use it as a tree key and to route
    /// row-specific commands without matching on the (mutable) display label.
    /// Dynamic rows carry their entity key (issue-type key, project name);
    /// static rows carry a fixed slug (e.g. "git-token").
    pub id: String,
    /// Nesting depth within the section (1 = direct child, 2 = grandchild).
    /// Lets clients rebuild the tree (e.g. LLM tools → model aliases).
    pub depth: u16,
    pub label: String,
    pub description: String,
    /// Icon hint (e.g. "check", "warning", "tool", "folder").
    pub icon: String,
    /// Health: "green" | "yellow" | "red" | "gray".
    pub health: String,
}

/// A status section with its health and child rows.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct SectionDto {
    /// Stable section id (e.g. "config", "connections", "kanban").
    pub id: String,
    pub label: String,
    /// Health: "green" | "yellow" | "red" | "gray".
    pub health: String,
    pub description: String,
    /// Section ids that must be Green before this section is usable.
    pub prerequisites: Vec<String>,
    /// Whether all prerequisites are met. Sections are always returned (the web
    /// UI styles unmet ones as locked) rather than hidden by progressive disclosure.
    pub met: bool,
    pub children: Vec<SectionRowDto>,
}

/// Builds the canonical status sections from config + the live issue-type
/// registry. Defined in the binary (the section logic is ui-layer); see module
/// docs for why this is injected rather than called directly.
pub type SectionProvider =
    Arc<dyn Fn(&Config, &IssueTypeRegistry) -> Vec<SectionDto> + Send + Sync>;

static SECTION_PROVIDER: OnceLock<SectionProvider> = OnceLock::new();

/// Register the process-wide section provider. Called once by the binary at
/// startup, before any server starts. Subsequent calls are ignored.
pub fn register_section_provider(provider: SectionProvider) {
    let _ = SECTION_PROVIDER.set(provider);
}

/// Returns the registered provider, if any.
pub fn section_provider() -> Option<SectionProvider> {
    SECTION_PROVIDER.get().cloned()
}

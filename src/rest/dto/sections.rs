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

/// A browser-openable action on a status section row (e.g. "Open Web UI",
/// "Swagger"). Only URL-style actions surface to the web UI; TUI-only actions
/// (toggles, env edits) are omitted.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct RowActionDto {
    /// Display label for the action button/link.
    pub label: String,
    /// Browser URL the action opens.
    pub url: String,
}

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
    /// Optional vendor-brand basename (e.g. "ollama"). When set, the web UI
    /// renders `/icons/{brand_icon}.svg` instead of the semantic `icon`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub brand_icon: Option<String>,
    /// Health: "green" | "yellow" | "red" | "gray".
    pub health: String,
    /// Browser-openable actions for this row (links shown in the web UI).
    pub actions: Vec<RowActionDto>,
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

/// Live connection facts the config-only snapshot can't know on its own.
///
/// The `/api/v1/sections` handler is, by definition, proof the API (and embedded
/// Web UI) are up; it passes these runtime facts to the provider so the
/// connections section reflects reality rather than the config defaults. Internal
/// provider input only — never serialized over the wire.
#[derive(Debug, Clone)]
pub struct LiveConnectionStatus {
    /// Whether the REST API is currently serving.
    pub api_running: bool,
    /// Port the API is bound to.
    pub port: u16,
    /// Whether the MCP HTTP transport is mounted.
    pub mcp_http_enabled: bool,
    /// Count of active MCP sessions.
    pub mcp_active_sessions: usize,
}

/// Builds the canonical status sections from config + the live issue-type
/// registry + live connection facts. Defined in the binary (the section logic is
/// ui-layer); see module docs for why this is injected rather than called directly.
pub type SectionProvider = Arc<
    dyn Fn(&Config, &IssueTypeRegistry, &LiveConnectionStatus) -> Vec<SectionDto> + Send + Sync,
>;

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

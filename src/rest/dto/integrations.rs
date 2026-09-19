//! Vertical integration catalog DTO for `GET /api/v1/integrations`.
//!
//! A thin projection of [`crate::integrations::catalog::all_integrations`] -
//! the single source of truth - exposing each advertised integration with its
//! [`SupportStatus`]. Consumed by the docs site and reserved for future
//! entitlement control.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;

use crate::integrations::support_catalog::VerticalSupport;
use crate::integrations::{all_integrations, SupportStatus};

/// One advertised integration: its vertical, identity, docs link, and support
/// status.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct IntegrationCatalogEntryDto {
    /// Vertical slug (e.g. "kanban", "model", "git", "session", "editor").
    pub vertical: String,
    /// Human label for the vertical (e.g. "Kanban Provider").
    pub vertical_label: String,
    /// Stable entry slug within the vertical (e.g. "jira", "anthropic-api").
    pub slug: String,
    /// Display label for the entry (e.g. "Jira", "Anthropic").
    pub label: String,
    /// Absolute docs URL, or `null` if undocumented.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub docs_url: Option<String>,
    /// Whether this entry carries a curated README badge.
    pub readme_badge: bool,
    /// Official support / maturity status.
    pub status: SupportStatus,
    pub premium: bool,
    /// Implemented session controllers for an IDE; absent for other categories.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_wrappers: Option<Vec<String>>,
    /// Vertical-specific structural support (not the advertising `status` ramp).
    #[ts(type = "unknown")]
    pub support: serde_json::Value,
}

/// Project the catalog source-of-truth into wire DTOs.
pub fn integration_catalog() -> Vec<IntegrationCatalogEntryDto> {
    all_integrations()
        .into_iter()
        .filter(crate::integrations::catalog::CatalogEntry::is_public)
        .map(|e| IntegrationCatalogEntryDto {
            vertical: e.vertical.slug().to_string(),
            vertical_label: e.vertical.label().to_string(),
            slug: e.slug.to_string(),
            label: e.label.to_string(),
            docs_url: e.docs_url(),
            readme_badge: e.readme_badge,
            status: e.status,
            premium: e.premium,
            session_wrappers: (e.vertical == crate::integrations::Vertical::Editor)
                .then(|| crate::integrations::catalog::ide_session_wrappers(e.slug))
                .flatten()
                .map(|wrappers| {
                    wrappers
                        .iter()
                        .map(|wrapper| wrapper.display_name().to_string())
                        .collect()
                }),
            support: support_json(e.support),
        })
        .collect()
}

fn support_json(support: VerticalSupport) -> serde_json::Value {
    match support {
        VerticalSupport::LlmTool(s) => serde_json::json!({
            "kind": "llm-tool",
            "health": s.health.slug(),
            "auth": s.auth.slug(),
            "native_protocols": s.native_protocols.iter().map(|p| p.slug()).collect::<Vec<_>>(),
            "sessions": s.sessions,
            "headless": s.headless,
            "yolo": s.yolo,
            "remote_inventory": s.remote_inventory.slug(),
            "relay": s.relay.slug(),
            "permissions": s.permissions.slug(),
        }),
        VerticalSupport::Model(s) => serde_json::json!({
            "kind": "model",
            "protocol": s.protocol.slug(),
            "class": s.class.slug(),
            "probe": s.probe,
            "connectable_from_defaults": s.connectable_from_defaults,
            "key_injectable": s.key_injectable.slug(),
            "implicit_for": s.implicit_for,
        }),
        VerticalSupport::Kanban(s) => serde_json::json!({
            "kind": "kanban",
            "sync_in": s.sync_in,
            "write_back": s.write_back,
        }),
        VerticalSupport::Git(s) => serde_json::json!({
            "kind": "git",
            "pr_cli": s.pr_cli,
            "remote_preflight": s.remote_preflight,
        }),
        VerticalSupport::Session(s) => serde_json::json!({
            "kind": "session",
            "attach": s.attach,
            "send_keys": s.send_keys,
            "idle_detect": s.idle_detect,
        }),
        VerticalSupport::Transport(s) | VerticalSupport::RemoteTargets(s) => serde_json::json!({
            "kind": "remote",
            "probe": s.probe,
            "tool_inventory": s.tool_inventory,
            "credential_injection": s.credential_injection,
        }),
        VerticalSupport::Editor(s)
        | VerticalSupport::Platform(s)
        | VerticalSupport::Integration(s)
        | VerticalSupport::Workflows(s)
        | VerticalSupport::Notification(s)
        | VerticalSupport::AgentRelay(s) => serde_json::json!({
            "kind": "sparse",
            "coverage": s.coverage.slug(),
            "notes": s.notes,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integration_catalog_projects_all_entries() {
        let dtos = integration_catalog();
        assert_eq!(
            dtos.len(),
            all_integrations().iter().filter(|e| e.is_public()).count()
        );
        let jira = dtos.iter().find(|d| d.slug == "jira").unwrap();
        assert_eq!(jira.vertical, "kanban");
        assert_eq!(jira.status, SupportStatus::Beta);
        assert_eq!(
            jira.docs_url.as_deref(),
            Some("https://operator.untra.io/getting-started/kanban/jira/")
        );
    }

    #[test]
    fn test_proto_entries_are_not_public() {
        let dtos = integration_catalog();
        assert!(dtos.iter().all(|d| d.status != SupportStatus::Proto));
        assert!(dtos
            .iter()
            .any(|d| d.vertical == "remote-targets" && d.premium));
    }

    #[test]
    fn test_claude_support_json_is_llm_tool() {
        let dtos = integration_catalog();
        let claude = dtos
            .iter()
            .find(|d| d.vertical == "llm-tool" && d.slug == "claude")
            .unwrap();
        assert_eq!(claude.support["kind"], "llm-tool");
        assert_eq!(claude.support["health"], "path-version");
        assert_eq!(claude.support["headless"], false);
        let ollama = dtos
            .iter()
            .find(|d| d.vertical == "model" && d.slug == "ollama")
            .unwrap();
        assert_eq!(ollama.support["kind"], "model");
        assert_eq!(ollama.support["protocol"], "openai");
        assert_eq!(ollama.support["class"], "gateway");
    }
}

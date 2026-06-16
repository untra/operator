//! Vertical integration catalog DTO for `GET /api/v1/integrations`.
//!
//! A thin projection of [`crate::integrations::catalog::all_integrations`] —
//! the single source of truth — exposing each advertised integration with its
//! [`SupportStatus`]. Consumed by the docs site and reserved for future
//! entitlement control.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;

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
}

/// Project the catalog source-of-truth into wire DTOs.
pub fn integration_catalog() -> Vec<IntegrationCatalogEntryDto> {
    all_integrations()
        .into_iter()
        .map(|e| IntegrationCatalogEntryDto {
            vertical: e.vertical.slug().to_string(),
            vertical_label: e.vertical.label().to_string(),
            slug: e.slug.to_string(),
            label: e.label.to_string(),
            docs_url: e.docs_url(),
            readme_badge: e.readme_badge,
            status: e.status,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integration_catalog_projects_all_entries() {
        let dtos = integration_catalog();
        assert_eq!(dtos.len(), all_integrations().len());
        let jira = dtos.iter().find(|d| d.slug == "jira").unwrap();
        assert_eq!(jira.vertical, "kanban");
        assert_eq!(jira.status, SupportStatus::Beta);
        assert_eq!(
            jira.docs_url.as_deref(),
            Some("https://operator.untra.io/getting-started/kanban/jira/")
        );
    }

    #[test]
    fn test_proto_entry_has_no_docs_url() {
        let dtos = integration_catalog();
        let lmstudio = dtos.iter().find(|d| d.slug == "lmstudio").unwrap();
        assert!(lmstudio.docs_url.is_none());
        assert!(!lmstudio.readme_badge);
    }
}

//! Kanban issue type definitions synced from external providers.
//!
//! These are provider metadata only -- not operator workflow definitions.
//! A `KanbanIssueType` represents a type/label from Jira or Linear that
//! can be mapped to an operator `IssueType` template (TASK, FEAT, FIX, etc.).

use serde::{Deserialize, Serialize};

use crate::api::providers::kanban::{ExternalField, ExternalIssueType};

/// A kanban issue type synced from an external provider.
///
/// This is provider metadata only -- not an operator workflow definition.
/// Persisted in `.tickets/operator/kanban/<provider>/<project>/issuetypes.json`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KanbanIssueType {
    /// Provider-specific ID (Jira type ID, Linear label ID)
    pub id: String,
    /// Display name (e.g., "Bug", "Story", "Task")
    pub name: String,
    /// Description from the provider
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Icon/avatar URL from the provider
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<String>,
    /// Summary of custom fields defined for this type
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub custom_fields: Vec<ExternalField>,
    /// Provider name ("jira" or "linear")
    pub provider: String,
    /// Project/team key in the provider
    pub project: String,
    /// What this type represents in the provider ("issuetype" for Jira, "label" for Linear)
    pub source_kind: String,
    /// ISO 8601 timestamp of last sync
    pub synced_at: String,
}

/// Lightweight reference to a kanban issue type on an issue.
///
/// Each issue carries one or more of these refs to indicate its
/// provider-side type classification.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KanbanIssueTypeRef {
    /// Provider-specific type/label ID
    pub id: String,
    /// Display name
    pub name: String,
}

impl KanbanIssueType {
    /// Create from an `ExternalIssueType` with provider context.
    pub fn from_external(
        external: &ExternalIssueType,
        provider: &str,
        project: &str,
        source_kind: &str,
        synced_at: &str,
    ) -> Self {
        Self {
            id: external.id.clone(),
            name: external.name.clone(),
            description: external.description.clone(),
            icon_url: external.icon_url.clone(),
            custom_fields: external.custom_fields.clone(),
            provider: provider.to_string(),
            project: project.to_string(),
            source_kind: source_kind.to_string(),
            synced_at: synced_at.to_string(),
        }
    }

    /// Create a `KanbanIssueTypeRef` from this type.
    pub fn as_ref(&self) -> KanbanIssueTypeRef {
        KanbanIssueTypeRef {
            id: self.id.clone(),
            name: self.name.clone(),
        }
    }
}

/// Sanitize an external type name into a valid operator issuetype key.
///
/// Rules: uppercase, letters only, max 10 chars, min 2 chars (padded with X).
pub fn sanitize_key(name: &str) -> String {
    let key: String = name
        .chars()
        .filter(char::is_ascii_alphabetic)
        .take(10)
        .collect::<String>()
        .to_uppercase();

    if key.len() < 2 {
        format!("{key}X")
    } else {
        key
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_external() -> ExternalIssueType {
        ExternalIssueType {
            id: "10001".to_string(),
            name: "Bug".to_string(),
            description: Some("A software bug".to_string()),
            icon_url: Some("https://example.com/bug.png".to_string()),
            custom_fields: vec![],
        }
    }

    #[test]
    fn test_from_external() {
        let external = sample_external();
        let kanban = KanbanIssueType::from_external(
            &external,
            "jira",
            "PROJ",
            "issuetype",
            "2026-04-05T12:00:00Z",
        );

        assert_eq!(kanban.id, "10001");
        assert_eq!(kanban.name, "Bug");
        assert_eq!(kanban.description, Some("A software bug".to_string()));
        assert_eq!(
            kanban.icon_url,
            Some("https://example.com/bug.png".to_string())
        );
        assert_eq!(kanban.provider, "jira");
        assert_eq!(kanban.project, "PROJ");
        assert_eq!(kanban.source_kind, "issuetype");
        assert_eq!(kanban.synced_at, "2026-04-05T12:00:00Z");
    }

    #[test]
    fn test_from_external_linear_label() {
        let external = ExternalIssueType {
            id: "label-abc".to_string(),
            name: "Feature".to_string(),
            description: None,
            icon_url: None,
            custom_fields: vec![],
        };
        let kanban = KanbanIssueType::from_external(
            &external,
            "linear",
            "TEAM-XYZ",
            "label",
            "2026-04-05T12:00:00Z",
        );

        assert_eq!(kanban.provider, "linear");
        assert_eq!(kanban.source_kind, "label");
        assert!(kanban.description.is_none());
        assert!(kanban.icon_url.is_none());
    }

    #[test]
    fn test_as_ref() {
        let kanban = KanbanIssueType::from_external(
            &sample_external(),
            "jira",
            "PROJ",
            "issuetype",
            "2026-04-05T12:00:00Z",
        );
        let r = kanban.as_ref();

        assert_eq!(r.id, "10001");
        assert_eq!(r.name, "Bug");
    }

    #[test]
    fn test_serialization_roundtrip() {
        let kanban = KanbanIssueType::from_external(
            &sample_external(),
            "jira",
            "PROJ",
            "issuetype",
            "2026-04-05T12:00:00Z",
        );

        let json = serde_json::to_string(&kanban).unwrap();
        let deserialized: KanbanIssueType = serde_json::from_str(&json).unwrap();

        assert_eq!(kanban, deserialized);
    }

    #[test]
    fn test_ref_serialization_roundtrip() {
        let r = KanbanIssueTypeRef {
            id: "10001".to_string(),
            name: "Bug".to_string(),
        };

        let json = serde_json::to_string(&r).unwrap();
        let deserialized: KanbanIssueTypeRef = serde_json::from_str(&json).unwrap();

        assert_eq!(r, deserialized);
    }

    #[test]
    fn test_skip_serializing_none_fields() {
        let external = ExternalIssueType {
            id: "10001".to_string(),
            name: "Task".to_string(),
            description: None,
            icon_url: None,
            custom_fields: vec![],
        };
        let kanban = KanbanIssueType::from_external(
            &external,
            "jira",
            "PROJ",
            "issuetype",
            "2026-04-05T12:00:00Z",
        );

        let json = serde_json::to_string(&kanban).unwrap();
        assert!(!json.contains("description"));
        assert!(!json.contains("icon_url"));
        assert!(!json.contains("custom_fields"));
    }

    #[test]
    fn test_sanitize_key_normal() {
        assert_eq!(sanitize_key("Bug"), "BUG");
        assert_eq!(sanitize_key("Story"), "STORY");
        assert_eq!(sanitize_key("Feature"), "FEATURE");
        assert_eq!(sanitize_key("Task"), "TASK");
    }

    #[test]
    fn test_sanitize_key_filters_non_alpha() {
        assert_eq!(sanitize_key("P0 Bug"), "PBUG");
        assert_eq!(sanitize_key("Sub-task"), "SUBTASK");
        assert_eq!(sanitize_key("User Story 2"), "USERSTORY");
    }

    #[test]
    fn test_sanitize_key_truncates_long_names() {
        assert_eq!(sanitize_key("Very Long Issue Type Name"), "VERYLONGIS");
    }

    #[test]
    fn test_sanitize_key_pads_short_names() {
        assert_eq!(sanitize_key("X"), "XX");
        assert_eq!(sanitize_key("A"), "AX");
    }

    #[test]
    fn test_sanitize_key_empty_after_filter() {
        assert_eq!(sanitize_key("123"), "X");
        assert_eq!(sanitize_key(""), "X");
    }

    #[test]
    fn test_vec_serialization() {
        let types = vec![
            KanbanIssueType::from_external(
                &sample_external(),
                "jira",
                "PROJ",
                "issuetype",
                "2026-04-05T12:00:00Z",
            ),
            KanbanIssueType::from_external(
                &ExternalIssueType {
                    id: "10002".to_string(),
                    name: "Story".to_string(),
                    description: None,
                    icon_url: None,
                    custom_fields: vec![],
                },
                "jira",
                "PROJ",
                "issuetype",
                "2026-04-05T12:00:00Z",
            ),
        ];

        let json = serde_json::to_string_pretty(&types).unwrap();
        let deserialized: Vec<KanbanIssueType> = serde_json::from_str(&json).unwrap();

        assert_eq!(types.len(), deserialized.len());
        assert_eq!(types[0], deserialized[0]);
        assert_eq!(types[1], deserialized[1]);
    }
}

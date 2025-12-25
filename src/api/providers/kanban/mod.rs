#![allow(dead_code)]

//! Kanban Provider trait and implementations
//!
//! Supports importing issue types from Jira, Linear, and other kanban providers.

mod jira;
mod linear;

pub use jira::JiraProvider;
pub use linear::LinearProvider;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::api::error::ApiError;
use crate::issuetypes::IssueType;

/// Information about a project/team from a kanban provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectInfo {
    /// Unique identifier in the provider
    pub id: String,
    /// Project key (e.g., "PROJ" for Jira)
    pub key: String,
    /// Human-readable name
    pub name: String,
}

/// External issue type from a kanban provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalIssueType {
    /// Unique identifier in the provider
    pub id: String,
    /// Name (e.g., "Bug", "Story")
    pub name: String,
    /// Description
    pub description: Option<String>,
    /// Icon/avatar URL
    pub icon_url: Option<String>,
    /// Custom fields defined for this type
    pub custom_fields: Vec<ExternalField>,
}

/// External field definition from a kanban provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalField {
    /// Field identifier
    pub id: String,
    /// Display name
    pub name: String,
    /// Field type (string, number, array, etc.)
    pub field_type: String,
    /// Whether the field is required
    pub required: bool,
    /// Options for select/enum fields
    pub options: Vec<String>,
}

/// Trait for kanban providers that can export issue types
#[async_trait]
pub trait KanbanProvider: Send + Sync {
    /// Get the provider name (e.g., "jira", "linear")
    fn name(&self) -> &str;

    /// Check if the provider is configured (has API credentials)
    fn is_configured(&self) -> bool;

    /// List available projects/teams
    async fn list_projects(&self) -> Result<Vec<ProjectInfo>, ApiError>;

    /// Get issue types for a project
    async fn get_issue_types(&self, project_key: &str) -> Result<Vec<ExternalIssueType>, ApiError>;

    /// Convert an external issue type to an Operator IssueType
    fn convert_to_issuetype(&self, external: &ExternalIssueType, project_key: &str) -> IssueType;

    /// Test connectivity to the API
    async fn test_connection(&self) -> Result<bool, ApiError>;
}

/// Detect which kanban providers are configured based on environment variables
pub fn detect_configured_providers() -> Vec<String> {
    let mut providers = Vec::new();

    if JiraProvider::from_env()
        .map(|p| p.is_configured())
        .unwrap_or(false)
    {
        providers.push("jira".to_string());
    }

    if LinearProvider::from_env()
        .map(|p| p.is_configured())
        .unwrap_or(false)
    {
        providers.push("linear".to_string());
    }

    providers
}

/// Get a provider by name
pub fn get_provider(name: &str) -> Option<Box<dyn KanbanProvider>> {
    match name.to_lowercase().as_str() {
        "jira" => JiraProvider::from_env()
            .ok()
            .map(|p| Box::new(p) as Box<dyn KanbanProvider>),
        "linear" => LinearProvider::from_env()
            .ok()
            .map(|p| Box::new(p) as Box<dyn KanbanProvider>),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_info() {
        let project = ProjectInfo {
            id: "10000".to_string(),
            key: "PROJ".to_string(),
            name: "My Project".to_string(),
        };
        assert_eq!(project.key, "PROJ");
    }

    #[test]
    fn test_external_issue_type() {
        let issue_type = ExternalIssueType {
            id: "10001".to_string(),
            name: "Bug".to_string(),
            description: Some("A software bug".to_string()),
            icon_url: None,
            custom_fields: vec![],
        };
        assert_eq!(issue_type.name, "Bug");
    }

    #[test]
    fn test_external_field() {
        let field = ExternalField {
            id: "customfield_10001".to_string(),
            name: "Story Points".to_string(),
            field_type: "number".to_string(),
            required: false,
            options: vec![],
        };
        assert!(!field.required);
    }
}

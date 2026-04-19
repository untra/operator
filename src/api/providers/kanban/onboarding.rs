//! Provider-neutral onboarding types and `KanbanOnboarding` sibling trait.
//!
//! These types are Rust-internal only (no `Serialize`/`Deserialize`/`TS` derives)
//! and live alongside the existing `KanbanProvider` trait without modifying it.

use async_trait::async_trait;

use super::KanbanProviderType;
use crate::api::error::ApiError;

/// Validated workspace returned by `KanbanOnboarding::validate_onboarding`.
///
/// Contains everything the onboarding flow needs to write config and display
/// confirmation without a second round-trip to the provider.
#[derive(Debug, Clone)]
pub struct ValidatedWorkspace {
    /// Which provider family this workspace belongs to.
    pub provider_kind: KanbanProviderType,
    /// Canonical workspace key: Jira domain, Linear `url_key`, GitHub owner login.
    pub workspace_key: String,
    /// Human-readable workspace name for display.
    pub workspace_display_name: String,
    /// Provider-specific user ID to sync issues for.
    pub sync_user_id: String,
    /// Human-readable name of the sync user.
    pub sync_user_display_name: String,
    /// Environment variable name that holds the API key/token.
    pub api_key_env: String,
    /// Projects discovered during validation (if available in a single round-trip).
    pub prefetched_projects: Option<Vec<DiscoveredProject>>,
    /// Provider-specific extra data needed for config upsert.
    pub extra: WorkspaceExtra,
}

/// Provider-specific extra data carried in a `ValidatedWorkspace`.
#[derive(Debug, Clone)]
pub enum WorkspaceExtra {
    /// Jira requires the user's email for Basic Auth config.
    Jira { email: String },
    /// Linear needs no extra data beyond what's in `ValidatedWorkspace`.
    Linear,
    /// GitHub Projects needs no extra data beyond what's in `ValidatedWorkspace`.
    Github,
}

/// A project discovered from a kanban provider.
#[derive(Debug, Clone)]
pub struct DiscoveredProject {
    /// Workspace key this project belongs to (matches `ValidatedWorkspace.workspace_key`).
    pub workspace_key: String,
    /// Provider-specific project key (Jira project key, Linear team key, GitHub node ID).
    pub project_key: String,
    /// Human-readable project name for display.
    pub project_display_name: String,
    /// URL to the project in the provider's web UI (if available).
    pub provider_url: Option<String>,
    /// Provider-native ID (e.g., Jira project ID, Linear team ID, GitHub node ID).
    pub provider_native_id: Option<String>,
}

/// Sibling trait for onboarding flows — does NOT replace `KanbanProvider`.
///
/// Provides a uniform interface across Jira, Linear, and GitHub Projects
/// for credential validation and project discovery during onboarding.
#[async_trait]
pub trait KanbanOnboarding: Send + Sync {
    /// Which provider family this implementation covers.
    fn provider_kind(&self) -> KanbanProviderType;

    /// Validate credentials and return a workspace summary.
    ///
    /// A single round-trip to the provider API that confirms the API key
    /// works and returns user + workspace + (optionally) project data.
    async fn validate_onboarding(&self) -> Result<ValidatedWorkspace, ApiError>;

    /// Discover projects available in the workspace.
    ///
    /// For providers that prefetch projects during validation (Linear, GitHub),
    /// this returns the cached list. For Jira, this makes a separate API call.
    async fn discover_projects(
        &self,
        workspace: &ValidatedWorkspace,
    ) -> Result<Vec<DiscoveredProject>, ApiError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validated_workspace_jira_extra() {
        let ws = ValidatedWorkspace {
            provider_kind: KanbanProviderType::Jira,
            workspace_key: "acme.atlassian.net".to_string(),
            workspace_display_name: "Acme Corp".to_string(),
            sync_user_id: "acct-123".to_string(),
            sync_user_display_name: "Alice".to_string(),
            api_key_env: "OPERATOR_JIRA_API_KEY".to_string(),
            prefetched_projects: None,
            extra: WorkspaceExtra::Jira {
                email: "alice@acme.com".to_string(),
            },
        };

        assert_eq!(ws.provider_kind, KanbanProviderType::Jira);
        assert_eq!(ws.workspace_key, "acme.atlassian.net");
        assert_eq!(ws.sync_user_id, "acct-123");

        match &ws.extra {
            WorkspaceExtra::Jira { email } => assert_eq!(email, "alice@acme.com"),
            _ => panic!("expected Jira extra"),
        }
    }

    #[test]
    fn test_validated_workspace_linear_extra() {
        let ws = ValidatedWorkspace {
            provider_kind: KanbanProviderType::Linear,
            workspace_key: "acme".to_string(),
            workspace_display_name: "Acme Inc".to_string(),
            sync_user_id: "user-uuid-1".to_string(),
            sync_user_display_name: "Bob".to_string(),
            api_key_env: "OPERATOR_LINEAR_API_KEY".to_string(),
            prefetched_projects: Some(vec![DiscoveredProject {
                workspace_key: "acme".to_string(),
                project_key: "ENG".to_string(),
                project_display_name: "Engineering".to_string(),
                provider_url: None,
                provider_native_id: Some("team-id-1".to_string()),
            }]),
            extra: WorkspaceExtra::Linear,
        };

        assert_eq!(ws.provider_kind, KanbanProviderType::Linear);
        assert_eq!(ws.workspace_key, "acme");
        assert!(ws.prefetched_projects.is_some());
        assert_eq!(ws.prefetched_projects.as_ref().unwrap().len(), 1);

        match &ws.extra {
            WorkspaceExtra::Linear => {} // ok
            _ => panic!("expected Linear extra"),
        }
    }

    #[test]
    fn test_discovered_project_fields() {
        let project = DiscoveredProject {
            workspace_key: "my-org".to_string(),
            project_key: "PVT_abc".to_string(),
            project_display_name: "My Board".to_string(),
            provider_url: Some("https://github.com/orgs/my-org/projects/1".to_string()),
            provider_native_id: Some("PVT_abc".to_string()),
        };

        assert_eq!(project.workspace_key, "my-org");
        assert_eq!(project.project_key, "PVT_abc");
        assert_eq!(project.project_display_name, "My Board");
        assert_eq!(
            project.provider_url,
            Some("https://github.com/orgs/my-org/projects/1".to_string())
        );
        assert_eq!(project.provider_native_id, Some("PVT_abc".to_string()));
    }

    #[test]
    fn test_discovered_project_minimal() {
        let project = DiscoveredProject {
            workspace_key: "acme.atlassian.net".to_string(),
            project_key: "PROJ".to_string(),
            project_display_name: "Project".to_string(),
            provider_url: None,
            provider_native_id: None,
        };

        assert!(project.provider_url.is_none());
        assert!(project.provider_native_id.is_none());
    }
}

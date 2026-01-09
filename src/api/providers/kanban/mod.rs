#![allow(dead_code)]

//! Kanban Provider trait and implementations
//!
//! Supports importing issue types and syncing work items from Jira, Linear, and other kanban providers.

mod jira;
mod linear;

pub use jira::JiraProvider;
pub use linear::LinearProvider;

// Re-export Jira API response types for schema/binding generation
pub use jira::{
    JiraAvatarUrls, JiraDescription, JiraIssue, JiraIssueFields, JiraIssueTypeRef, JiraPriority,
    JiraProjectStatus, JiraSearchResponse, JiraStatus, JiraStatusRef, JiraUser,
};

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

/// User information from a kanban provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalUser {
    /// Unique identifier in the provider (e.g., Jira accountId, Linear user ID)
    pub id: String,
    /// Display name
    pub name: String,
    /// Email address (if available)
    pub email: Option<String>,
    /// Avatar/profile picture URL
    pub avatar_url: Option<String>,
}

/// Issue/work item from a kanban provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalIssue {
    /// Unique identifier in the provider
    pub id: String,
    /// Issue key (e.g., "PROJ-123" for Jira, "ENG-456" for Linear)
    pub key: String,
    /// Summary/title
    pub summary: String,
    /// Full description (may be markdown)
    pub description: Option<String>,
    /// Issue type name (e.g., "Bug", "Story", "Task")
    pub issue_type: String,
    /// Current status name (e.g., "To Do", "In Progress")
    pub status: String,
    /// Assigned user (if any)
    pub assignee: Option<ExternalUser>,
    /// Full URL to the issue in the provider's web UI
    pub url: String,
    /// Priority (if available)
    pub priority: Option<String>,
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

// ─── CRUD Request/Response Types ─────────────────────────────────────────────

/// Request to create a new issue in a kanban provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateIssueRequest {
    /// Issue summary/title (required)
    pub summary: String,
    /// Issue description (optional, may be markdown)
    pub description: Option<String>,
    /// Assignee user ID (optional)
    pub assignee_id: Option<String>,
    /// Initial status name (optional, uses provider default if not specified)
    pub status: Option<String>,
    /// Priority name (optional)
    pub priority: Option<String>,
}

/// Response from creating an issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateIssueResponse {
    /// The created issue with its assigned ID/key
    pub issue: ExternalIssue,
}

/// Request to update an issue's status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateStatusRequest {
    /// New status name to transition to
    pub status: String,
}

/// Trait for kanban providers that can export issue types and sync work items
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

    /// List users who can be assigned issues in a project
    ///
    /// Returns assignable users for the given project/team.
    async fn list_users(&self, project_key: &str) -> Result<Vec<ExternalUser>, ApiError>;

    /// List workflow statuses for a project
    ///
    /// Returns available status names (e.g., "To Do", "In Progress", "Done").
    async fn list_statuses(&self, project_key: &str) -> Result<Vec<String>, ApiError>;

    /// Fetch issues assigned to a user in specified statuses
    ///
    /// If `statuses` is empty, fetches issues in the default/first status only.
    async fn list_issues(
        &self,
        project_key: &str,
        user_id: &str,
        statuses: &[String],
    ) -> Result<Vec<ExternalIssue>, ApiError>;

    // ─── CRUD Operations ─────────────────────────────────────────────────────

    /// Create a new issue in the specified project
    ///
    /// Returns the created issue with its assigned ID/key from the provider.
    async fn create_issue(
        &self,
        project_key: &str,
        request: CreateIssueRequest,
    ) -> Result<CreateIssueResponse, ApiError>;

    /// Update an issue's workflow status
    ///
    /// Transitions the issue to the specified status and returns the updated issue.
    /// Returns an error if the transition is not valid for the issue's current state.
    async fn update_issue_status(
        &self,
        issue_key: &str,
        request: UpdateStatusRequest,
    ) -> Result<ExternalIssue, ApiError>;
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

/// Type of kanban provider
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KanbanProviderType {
    Jira,
    Linear,
}

impl KanbanProviderType {
    /// Get the display name
    pub fn display_name(&self) -> &'static str {
        match self {
            KanbanProviderType::Jira => "Jira Cloud",
            KanbanProviderType::Linear => "Linear",
        }
    }

    /// Get the default environment variable name for the API key
    pub fn default_api_key_env(&self) -> &'static str {
        match self {
            KanbanProviderType::Jira => "OPERATOR_JIRA_API_KEY",
            KanbanProviderType::Linear => "OPERATOR_LINEAR_API_KEY",
        }
    }
}

/// Status of a detected kanban provider
#[derive(Debug, Clone)]
pub enum ProviderStatus {
    /// Not yet tested
    Untested,
    /// Currently testing credentials
    Testing,
    /// Credentials are valid
    Valid,
    /// Credentials failed with error message
    Failed { error: String },
}

/// A detected kanban provider from environment variables
#[derive(Debug, Clone)]
pub struct DetectedKanbanProvider {
    /// Type of provider
    pub provider_type: KanbanProviderType,
    /// Domain or workspace identifier (Jira domain or Linear workspace)
    pub domain: String,
    /// Environment variables that were found
    pub env_vars_found: Vec<String>,
    /// Email address (for Jira)
    pub email: Option<String>,
    /// Status of credential validation
    pub status: ProviderStatus,
}

impl DetectedKanbanProvider {
    /// Check if all required env vars are present for this provider
    pub fn has_required_env_vars(&self) -> bool {
        match self.provider_type {
            KanbanProviderType::Jira => {
                // Jira needs domain, email, and API key
                self.env_vars_found
                    .iter()
                    .any(|v| v.contains("DOMAIN") || v.contains("_DOMAIN"))
                    && self
                        .env_vars_found
                        .iter()
                        .any(|v| v.contains("EMAIL") || v.contains("_EMAIL"))
                    && self
                        .env_vars_found
                        .iter()
                        .any(|v| v.contains("API_KEY") || v.contains("TOKEN"))
            }
            KanbanProviderType::Linear => {
                // Linear just needs API key
                self.env_vars_found.iter().any(|v| v.contains("API_KEY"))
            }
        }
    }
}

/// Detect kanban providers from environment variables
///
/// Scans for `OPERATOR_JIRA_*` and `OPERATOR_LINEAR_*` environment variables
/// and returns a list of detected providers.
pub fn detect_kanban_env_vars() -> Vec<DetectedKanbanProvider> {
    use std::env;

    let mut providers = Vec::new();

    // Check for standard Jira environment variables
    let jira_domain = env::var("OPERATOR_JIRA_DOMAIN").ok();
    let jira_email = env::var("OPERATOR_JIRA_EMAIL").ok();
    let jira_api_key = env::var("OPERATOR_JIRA_API_KEY").ok();

    if jira_domain.is_some() || jira_email.is_some() || jira_api_key.is_some() {
        let mut env_vars_found = Vec::new();
        if jira_domain.is_some() {
            env_vars_found.push("OPERATOR_JIRA_DOMAIN".to_string());
        }
        if jira_email.is_some() {
            env_vars_found.push("OPERATOR_JIRA_EMAIL".to_string());
        }
        if jira_api_key.is_some() {
            env_vars_found.push("OPERATOR_JIRA_API_KEY".to_string());
        }

        providers.push(DetectedKanbanProvider {
            provider_type: KanbanProviderType::Jira,
            domain: jira_domain.unwrap_or_else(|| "unknown".to_string()),
            email: jira_email,
            env_vars_found,
            status: ProviderStatus::Untested,
        });
    }

    // Check for standard Linear environment variables
    let linear_api_key = env::var("OPERATOR_LINEAR_API_KEY").ok();

    if linear_api_key.is_some() {
        providers.push(DetectedKanbanProvider {
            provider_type: KanbanProviderType::Linear,
            domain: "linear.app".to_string(),
            email: None,
            env_vars_found: vec!["OPERATOR_LINEAR_API_KEY".to_string()],
            status: ProviderStatus::Untested,
        });
    }

    // Scan for custom-named Jira instances (OPERATOR_JIRA_<NAME>_API_KEY pattern)
    for (key, _value) in env::vars() {
        if let Some(instance_name) = parse_custom_jira_env_var(&key) {
            // Check if we already have this instance
            let domain_env = format!("OPERATOR_JIRA_{}_DOMAIN", instance_name);
            let email_env = format!("OPERATOR_JIRA_{}_EMAIL", instance_name);
            let api_key_env = format!("OPERATOR_JIRA_{}_API_KEY", instance_name);

            let domain = env::var(&domain_env).ok();
            let email = env::var(&email_env).ok();
            let has_api_key = env::var(&api_key_env).is_ok();

            // Only add if we have at least domain or API key
            if domain.is_some() || has_api_key {
                let mut env_vars_found = Vec::new();
                if domain.is_some() {
                    env_vars_found.push(domain_env);
                }
                if email.is_some() {
                    env_vars_found.push(email_env);
                }
                if has_api_key {
                    env_vars_found.push(api_key_env);
                }

                // Check if we already have an entry for this domain
                let domain_str = domain
                    .clone()
                    .unwrap_or_else(|| instance_name.to_lowercase());
                if !providers
                    .iter()
                    .any(|p| p.provider_type == KanbanProviderType::Jira && p.domain == domain_str)
                {
                    providers.push(DetectedKanbanProvider {
                        provider_type: KanbanProviderType::Jira,
                        domain: domain_str,
                        email,
                        env_vars_found,
                        status: ProviderStatus::Untested,
                    });
                }
            }
        }
    }

    providers
}

/// Parse a custom Jira environment variable name to extract the instance name
///
/// Matches patterns like:
/// - OPERATOR_JIRA_FOOBAR_API_KEY -> Some("FOOBAR")
/// - OPERATOR_JIRA_FOOBAR_DOMAIN -> Some("FOOBAR")
/// - OPERATOR_JIRA_API_KEY -> None (standard, not custom)
fn parse_custom_jira_env_var(key: &str) -> Option<&str> {
    if !key.starts_with("OPERATOR_JIRA_") {
        return None;
    }

    let suffix = &key["OPERATOR_JIRA_".len()..];

    // Check if this is a custom instance (has instance name before the field)
    // Standard vars: OPERATOR_JIRA_DOMAIN, OPERATOR_JIRA_EMAIL, OPERATOR_JIRA_API_KEY, OPERATOR_JIRA_TOKEN
    // Custom vars: OPERATOR_JIRA_<NAME>_DOMAIN, OPERATOR_JIRA_<NAME>_API_KEY, etc.
    if suffix == "DOMAIN" || suffix == "EMAIL" || suffix == "API_KEY" || suffix == "TOKEN" {
        return None; // Standard variable
    }

    // Check for _API_KEY suffix (two underscores from instance name)
    if let Some(instance) = suffix.strip_suffix("_API_KEY") {
        if !instance.is_empty() {
            return Some(instance);
        }
    }

    // Check for single-word field suffixes
    if let Some(underscore_pos) = suffix.rfind('_') {
        let potential_field = &suffix[underscore_pos + 1..];
        if potential_field == "DOMAIN" || potential_field == "EMAIL" || potential_field == "TOKEN" {
            let instance = &suffix[..underscore_pos];
            if !instance.is_empty() {
                return Some(instance);
            }
        }
    }

    None
}

/// Test credentials for a detected provider
///
/// Returns Ok(()) if credentials are valid, or an error message if they fail.
pub async fn test_provider_credentials(provider: &DetectedKanbanProvider) -> Result<(), String> {
    if !provider.has_required_env_vars() {
        return Err("Missing required environment variables".to_string());
    }

    match provider.provider_type {
        KanbanProviderType::Jira => {
            let jira = JiraProvider::from_env()
                .map_err(|e| format!("Failed to create provider: {}", e))?;

            jira.test_connection()
                .await
                .map_err(|e| format!("Connection failed: {}", e))?;

            Ok(())
        }
        KanbanProviderType::Linear => {
            let linear = LinearProvider::from_env()
                .map_err(|e| format!("Failed to create provider: {}", e))?;

            linear
                .test_connection()
                .await
                .map_err(|e| format!("Connection failed: {}", e))?;

            Ok(())
        }
    }
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
    fn test_external_user() {
        let user = ExternalUser {
            id: "user-123".to_string(),
            name: "John Doe".to_string(),
            email: Some("john@example.com".to_string()),
            avatar_url: Some("https://example.com/avatar.png".to_string()),
        };
        assert_eq!(user.id, "user-123");
        assert_eq!(user.name, "John Doe");
        assert!(user.email.is_some());
    }

    #[test]
    fn test_external_user_minimal() {
        let user = ExternalUser {
            id: "user-456".to_string(),
            name: "Jane Doe".to_string(),
            email: None,
            avatar_url: None,
        };
        assert_eq!(user.id, "user-456");
        assert!(user.email.is_none());
        assert!(user.avatar_url.is_none());
    }

    #[test]
    fn test_external_issue() {
        let issue = ExternalIssue {
            id: "issue-789".to_string(),
            key: "PROJ-123".to_string(),
            summary: "Fix login bug".to_string(),
            description: Some("Users cannot log in with SSO".to_string()),
            issue_type: "Bug".to_string(),
            status: "To Do".to_string(),
            assignee: Some(ExternalUser {
                id: "user-123".to_string(),
                name: "John Doe".to_string(),
                email: None,
                avatar_url: None,
            }),
            url: "https://example.atlassian.net/browse/PROJ-123".to_string(),
            priority: Some("High".to_string()),
        };
        assert_eq!(issue.key, "PROJ-123");
        assert_eq!(issue.summary, "Fix login bug");
        assert_eq!(issue.status, "To Do");
        assert!(issue.assignee.is_some());
    }

    #[test]
    fn test_external_issue_unassigned() {
        let issue = ExternalIssue {
            id: "issue-999".to_string(),
            key: "ENG-456".to_string(),
            summary: "Add dark mode".to_string(),
            description: None,
            issue_type: "Feature".to_string(),
            status: "Backlog".to_string(),
            assignee: None,
            url: "https://linear.app/team/ENG-456".to_string(),
            priority: None,
        };
        assert_eq!(issue.key, "ENG-456");
        assert!(issue.assignee.is_none());
        assert!(issue.priority.is_none());
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

    #[test]
    fn test_kanban_provider_type_display_name() {
        assert_eq!(KanbanProviderType::Jira.display_name(), "Jira Cloud");
        assert_eq!(KanbanProviderType::Linear.display_name(), "Linear");
    }

    #[test]
    fn test_kanban_provider_type_default_api_key_env() {
        assert_eq!(
            KanbanProviderType::Jira.default_api_key_env(),
            "OPERATOR_JIRA_API_KEY"
        );
        assert_eq!(
            KanbanProviderType::Linear.default_api_key_env(),
            "OPERATOR_LINEAR_API_KEY"
        );
    }

    #[test]
    fn test_detected_provider_has_required_env_vars_jira_complete() {
        let provider = DetectedKanbanProvider {
            provider_type: KanbanProviderType::Jira,
            domain: "test.atlassian.net".to_string(),
            email: Some("test@example.com".to_string()),
            env_vars_found: vec![
                "OPERATOR_JIRA_DOMAIN".to_string(),
                "OPERATOR_JIRA_EMAIL".to_string(),
                "OPERATOR_JIRA_API_KEY".to_string(),
            ],
            status: ProviderStatus::Untested,
        };
        assert!(provider.has_required_env_vars());
    }

    #[test]
    fn test_detected_provider_has_required_env_vars_jira_incomplete() {
        let provider = DetectedKanbanProvider {
            provider_type: KanbanProviderType::Jira,
            domain: "test.atlassian.net".to_string(),
            email: None,
            env_vars_found: vec!["OPERATOR_JIRA_DOMAIN".to_string()],
            status: ProviderStatus::Untested,
        };
        assert!(!provider.has_required_env_vars());
    }

    #[test]
    fn test_detected_provider_has_required_env_vars_linear() {
        let provider = DetectedKanbanProvider {
            provider_type: KanbanProviderType::Linear,
            domain: "linear.app".to_string(),
            email: None,
            env_vars_found: vec!["OPERATOR_LINEAR_API_KEY".to_string()],
            status: ProviderStatus::Untested,
        };
        assert!(provider.has_required_env_vars());
    }

    #[test]
    fn test_parse_custom_jira_env_var_standard() {
        // Standard vars should return None
        assert!(parse_custom_jira_env_var("OPERATOR_JIRA_DOMAIN").is_none());
        assert!(parse_custom_jira_env_var("OPERATOR_JIRA_EMAIL").is_none());
        assert!(parse_custom_jira_env_var("OPERATOR_JIRA_API_KEY").is_none());
        assert!(parse_custom_jira_env_var("OPERATOR_JIRA_TOKEN").is_none());
    }

    #[test]
    fn test_parse_custom_jira_env_var_custom() {
        // Custom vars should return the instance name
        assert_eq!(
            parse_custom_jira_env_var("OPERATOR_JIRA_FOOBAR_API_KEY"),
            Some("FOOBAR")
        );
        assert_eq!(
            parse_custom_jira_env_var("OPERATOR_JIRA_FOOBAR_DOMAIN"),
            Some("FOOBAR")
        );
        assert_eq!(
            parse_custom_jira_env_var("OPERATOR_JIRA_WORK_EMAIL"),
            Some("WORK")
        );
    }

    #[test]
    fn test_parse_custom_jira_env_var_non_jira() {
        // Non-Jira vars should return None
        assert!(parse_custom_jira_env_var("OPERATOR_LINEAR_API_KEY").is_none());
        assert!(parse_custom_jira_env_var("SOME_OTHER_VAR").is_none());
    }
}

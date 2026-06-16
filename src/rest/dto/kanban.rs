use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;

// =============================================================================
// External Issue Type DTOs (from kanban providers)
// =============================================================================

/// Summary of an issue type from an external kanban provider (Jira, Linear)
#[derive(Debug, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct ExternalIssueTypeSummary {
    /// Provider-specific unique identifier
    pub id: String,
    /// Issue type name (e.g., "Bug", "Story", "Task")
    pub name: String,
    /// Description of the issue type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Icon/avatar URL from the provider
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<String>,
}

// =============================================================================
// Kanban Issue Type Catalog DTOs
// =============================================================================

/// A synced kanban issue type from the persisted catalog.
#[derive(Debug, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct KanbanIssueTypeResponse {
    /// Provider-specific ID (Jira type ID, Linear label ID)
    pub id: String,
    /// Display name (e.g., "Bug", "Story", "Task")
    pub name: String,
    /// Description from the provider
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Icon/avatar URL from the provider
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<String>,
    /// Provider name ("jira", "linear", or "github")
    pub provider: String,
    /// Project/team key
    pub project: String,
    /// What this type represents in the provider ("issuetype" or "label")
    pub source_kind: String,
    /// ISO 8601 timestamp of last sync
    pub synced_at: String,
}

/// Response from syncing kanban issue types from a provider.
#[derive(Debug, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct SyncKanbanIssueTypesResponse {
    /// Number of issue types synced
    pub synced: usize,
    /// The synced issue types
    pub types: Vec<KanbanIssueTypeResponse>,
}

// =============================================================================
// Kanban Provider Catalog DTO
// =============================================================================

/// One supported kanban provider, as advertised by
/// `GET /api/v1/kanban/providers`.
///
/// This is the single source of truth (derived from
/// `KanbanProviderType::ALL`) that the web `/#/kanban` list view and the VS
/// Code onboarding picker both render against, so the available options can't
/// drift between surfaces.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct KanbanProviderCatalogEntry {
    /// Stable lowercase slug ("jira" | "linear" | "github").
    pub slug: String,
    /// Human-readable name (e.g. "Jira Cloud", "GitHub Projects").
    pub display_name: String,
    /// One-line connect description shown next to the provider.
    pub description: String,
    /// Credential/token page opened when the user chooses to configure it.
    pub setup_url: String,
    /// VS Code codicon hint (rendered as `$(icon)` in the picker).
    pub icon: String,
    /// Whether at least one instance of this provider is already configured.
    pub configured: bool,
}

// =============================================================================
// Kanban Onboarding DTOs
// =============================================================================

/// Which kanban provider an onboarding request targets.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, TS, PartialEq, Eq)]
#[ts(export)]
#[serde(rename_all = "lowercase")]
pub enum KanbanProviderKind {
    Jira,
    Linear,
    Github,
}

/// Ephemeral Jira credentials supplied by a client during onboarding.
///
/// These are never persisted to disk by the onboarding endpoints that take
/// this struct — the actual secret stays in the env var named in
/// `api_key_env` once set via `/api/v1/kanban/session-env`.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct JiraCredentials {
    /// Jira Cloud domain (e.g., "acme.atlassian.net")
    pub domain: String,
    /// Atlassian account email for Basic Auth
    pub email: String,
    /// API token / personal access token
    pub api_token: String,
}

/// Ephemeral Linear credentials supplied by a client during onboarding.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct LinearCredentials {
    /// Linear API key (prefixed `lin_api_`)
    pub api_key: String,
}

/// Ephemeral GitHub Projects credentials supplied by a client during onboarding.
///
/// The token must have `project` (or `read:project`) scope. A repo-only token
/// (the kind used for `GITHUB_TOKEN` and operator's git provider) will be
/// rejected at validation time with a friendly "lacks `project` scope" error.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct GithubCredentials {
    /// GitHub PAT, fine-grained PAT, or app installation token
    pub token: String,
}

/// Request to validate kanban credentials without persisting them.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct ValidateKanbanCredentialsRequest {
    pub provider: KanbanProviderKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub jira: Option<JiraCredentials>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub linear: Option<LinearCredentials>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub github: Option<GithubCredentials>,
}

/// Jira-specific validation details (returned on success).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct JiraValidationDetailsDto {
    /// Atlassian accountId (used as `sync_user_id`)
    pub account_id: String,
    /// User display name
    pub display_name: String,
}

/// A Linear team exposed to onboarding clients for project selection.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct LinearTeamInfoDto {
    pub id: String,
    pub key: String,
    pub name: String,
}

/// Linear-specific validation details (returned on success).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct LinearValidationDetailsDto {
    /// Linear viewer user ID (used as `sync_user_id`)
    pub user_id: String,
    pub user_name: String,
    pub org_name: String,
    pub teams: Vec<LinearTeamInfoDto>,
}

/// A GitHub Project v2 surfaced during onboarding for project picker UIs.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct GithubProjectInfoDto {
    /// `GraphQL` node ID (e.g., `PVT_kwDOABcdefg`) — used as the project key
    pub node_id: String,
    /// Project number (e.g., 42) within the owner
    pub number: i32,
    /// Human-readable project title
    pub title: String,
    /// Owner login (org or user name)
    pub owner_login: String,
    /// "Organization" or "User"
    pub owner_kind: String,
}

/// GitHub-specific validation details (returned on success).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct GithubValidationDetailsDto {
    /// Authenticated user's login (e.g., "octocat")
    pub user_login: String,
    /// Authenticated user's numeric `databaseId` as a string (used as `sync_user_id`)
    pub user_id: String,
    /// All Projects v2 visible to the token (across viewer + organizations)
    pub projects: Vec<GithubProjectInfoDto>,
    /// The env var name the validated token came from. Used by clients to
    /// display "Connected via `OPERATOR_GITHUB_TOKEN`" so users can rotate the
    /// right token. See Token Disambiguation in the kanban github docs.
    pub resolved_env_var: String,
}

/// Response from validating kanban credentials.
///
/// `valid: false` is returned for auth failures — never a 4xx/5xx HTTP
/// status — so clients can display `error` inline without exception handling.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct ValidateKanbanCredentialsResponse {
    pub valid: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub jira: Option<JiraValidationDetailsDto>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub linear: Option<LinearValidationDetailsDto>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub github: Option<GithubValidationDetailsDto>,
}

/// Request to list projects/teams from a provider using ephemeral creds.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct ListKanbanProjectsRequest {
    pub provider: KanbanProviderKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub jira: Option<JiraCredentials>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub linear: Option<LinearCredentials>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub github: Option<GithubCredentials>,
}

/// A project/team entry returned by `list_projects`.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct KanbanProjectInfo {
    pub id: String,
    pub key: String,
    pub name: String,
}

/// Response wrapper for list-projects (wrapped for utoipa compatibility).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct ListKanbanProjectsResponse {
    pub projects: Vec<KanbanProjectInfo>,
}

/// Body for writing a Jira project config section.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct WriteJiraConfigBody {
    pub domain: String,
    pub email: String,
    pub api_key_env: String,
    pub project_key: String,
    pub sync_user_id: String,
}

/// Body for writing a Linear project/team config section.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct WriteLinearConfigBody {
    pub workspace_key: String,
    pub api_key_env: String,
    pub project_key: String,
    pub sync_user_id: String,
}

/// Body for writing a GitHub Projects v2 config section.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct WriteGithubConfigBody {
    /// GitHub owner login (user or org), used as the workspace key
    pub owner: String,
    /// Env var name where the project-scoped token is set
    /// (default: `OPERATOR_GITHUB_TOKEN`). MUST be distinct from `GITHUB_TOKEN`
    /// — see Token Disambiguation in the kanban github docs.
    pub api_key_env: String,
    /// `GraphQL` project node ID (e.g., `PVT_kwDOABcdefg`)
    pub project_key: String,
    /// Numeric GitHub `databaseId` of the user whose items to sync
    pub sync_user_id: String,
}

/// Request to write or upsert a kanban config section.
///
/// This endpoint does NOT take the secret — only the env var NAME
/// (`api_key_env`). The secret is set via `/api/v1/kanban/session-env`.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct WriteKanbanConfigRequest {
    pub provider: KanbanProviderKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub jira: Option<WriteJiraConfigBody>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub linear: Option<WriteLinearConfigBody>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub github: Option<WriteGithubConfigBody>,
}

/// Response after writing a kanban config section.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct WriteKanbanConfigResponse {
    /// Filesystem path that was written (e.g., ".tickets/operator/config.toml")
    pub written_path: String,
    /// Header of the top-level section that was upserted
    /// (e.g., `[kanban.jira."acme.atlassian.net"]`)
    pub section_header: String,
}

/// Jira session env body — includes the actual secret to set in env.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct JiraSessionEnv {
    pub domain: String,
    pub email: String,
    pub api_token: String,
    pub api_key_env: String,
}

/// Linear session env body — includes the actual secret to set in env.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct LinearSessionEnv {
    pub api_key: String,
    pub api_key_env: String,
}

/// GitHub Projects session env body — includes the actual secret to set in env.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct GithubSessionEnv {
    pub token: String,
    pub api_key_env: String,
}

/// Request to set kanban-related env vars on the server for the current
/// session so subsequent `from_config` calls find the API key.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct SetKanbanSessionEnvRequest {
    pub provider: KanbanProviderKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub jira: Option<JiraSessionEnv>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub linear: Option<LinearSessionEnv>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub github: Option<GithubSessionEnv>,
}

/// Response from setting session env vars.
///
/// `shell_export_block` uses `<your-token>` placeholders, NOT the actual
/// secret — it is meant for the user to copy into their shell profile.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct SetKanbanSessionEnvResponse {
    /// Names (not values) of env vars that were set in the server process.
    pub env_vars_set: Vec<String>,
    /// Multi-line `export FOO="<your-token>"` block for the user to copy
    /// into `~/.zshrc` / `~/.bashrc`.
    pub shell_export_block: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kanban_provider_kind_serializes_lowercase_slugs() {
        // rename_all = "lowercase": these slugs must match the catalog slugs
        // documented on KanbanProviderCatalogEntry ("jira" | "linear" | "github").
        assert_eq!(
            serde_json::to_string(&KanbanProviderKind::Jira).unwrap(),
            "\"jira\""
        );
        assert_eq!(
            serde_json::to_string(&KanbanProviderKind::Linear).unwrap(),
            "\"linear\""
        );
        assert_eq!(
            serde_json::to_string(&KanbanProviderKind::Github).unwrap(),
            "\"github\""
        );
    }

    #[test]
    fn test_kanban_provider_kind_deserializes_from_lowercase_slugs() {
        let jira: KanbanProviderKind = serde_json::from_str("\"jira\"").unwrap();
        assert_eq!(jira, KanbanProviderKind::Jira);
        let github: KanbanProviderKind = serde_json::from_str("\"github\"").unwrap();
        assert_eq!(github, KanbanProviderKind::Github);
    }

    #[test]
    fn test_kanban_provider_kind_rejects_titlecase() {
        // Wire format is strictly lowercase; the Rust variant spelling must not parse.
        assert!(serde_json::from_str::<KanbanProviderKind>("\"Jira\"").is_err());
    }

    #[test]
    fn test_validate_request_serializes_only_targeted_provider_body() {
        // provider-tagged request: only the matching sub-body should appear; the
        // other two (None) are skipped via skip_serializing_if.
        let req = ValidateKanbanCredentialsRequest {
            provider: KanbanProviderKind::Jira,
            jira: Some(JiraCredentials {
                domain: "acme.atlassian.net".to_string(),
                email: "a@b.com".to_string(),
                api_token: "secret-token".to_string(),
            }),
            linear: None,
            github: None,
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"provider\":\"jira\""));
        assert!(json.contains("\"jira\":{"));
        assert!(!json.contains("\"linear\":"));
        assert!(!json.contains("\"github\":"));
    }

    #[test]
    fn test_validate_request_deserializes_with_missing_provider_bodies() {
        // The three credential slots default to None when absent.
        let json = r#"{ "provider": "linear", "linear": { "api_key": "lin_api_x" } }"#;
        let req: ValidateKanbanCredentialsRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.provider, KanbanProviderKind::Linear);
        assert!(req.jira.is_none());
        assert!(req.github.is_none());
        assert_eq!(req.linear.unwrap().api_key, "lin_api_x");
    }

    #[test]
    fn test_validate_response_skips_none_detail_blocks() {
        let resp = ValidateKanbanCredentialsResponse {
            valid: false,
            error: Some("invalid token".to_string()),
            jira: None,
            linear: None,
            github: None,
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"valid\":false"));
        assert!(json.contains("\"error\":\"invalid token\""));
        assert!(!json.contains("\"jira\":"));
        assert!(!json.contains("\"linear\":"));
        assert!(!json.contains("\"github\":"));
    }

    #[test]
    fn test_write_kanban_config_body_carries_env_name_not_secret() {
        // The config-write path stores only the env-var NAME (`api_key_env`) — it
        // must never carry the raw secret. (Contrast with the *SessionEnv bodies
        // below, which deliberately DO carry the secret to set server env.)
        let body = WriteJiraConfigBody {
            domain: "acme.atlassian.net".to_string(),
            email: "a@b.com".to_string(),
            api_key_env: "OPERATOR_JIRA_TOKEN".to_string(),
            project_key: "PROJ".to_string(),
            sync_user_id: "acct-1".to_string(),
        };
        let json = serde_json::to_string(&body).unwrap();
        assert!(json.contains("\"api_key_env\":\"OPERATOR_JIRA_TOKEN\""));
        // No raw secret field channels on the config-write body.
        assert!(!json.contains("api_token"));
        assert!(!json.contains("\"token\":"));
    }

    #[test]
    fn test_jira_session_env_intentionally_carries_secret() {
        // Counterpart to the config-write body: session-env DOES transport the
        // secret (`api_token`) so the server can set it in its process env.
        let env = JiraSessionEnv {
            domain: "acme.atlassian.net".to_string(),
            email: "a@b.com".to_string(),
            api_token: "real-secret".to_string(),
            api_key_env: "OPERATOR_JIRA_TOKEN".to_string(),
        };
        let json = serde_json::to_string(&env).unwrap();
        assert!(json.contains("\"api_token\":\"real-secret\""));
        assert!(json.contains("\"api_key_env\":\"OPERATOR_JIRA_TOKEN\""));
        let parsed: JiraSessionEnv = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.api_token, "real-secret");
    }

    #[test]
    fn test_provider_catalog_entry_roundtrip() {
        let entry = KanbanProviderCatalogEntry {
            slug: "github".to_string(),
            display_name: "GitHub Projects".to_string(),
            description: "Sync from a GitHub Project v2".to_string(),
            setup_url: "https://github.com/settings/tokens".to_string(),
            icon: "github".to_string(),
            configured: true,
        };
        let json = serde_json::to_string(&entry).unwrap();
        let parsed: KanbanProviderCatalogEntry = serde_json::from_str(&json).unwrap();
        // slug aligns with KanbanProviderKind::Github's lowercase wire form.
        assert_eq!(parsed.slug, "github");
        assert!(parsed.configured);
    }

    #[test]
    fn test_kanban_issue_type_response_skips_none_optionals() {
        let resp = KanbanIssueTypeResponse {
            id: "10001".to_string(),
            name: "Bug".to_string(),
            description: None,
            icon_url: None,
            provider: "jira".to_string(),
            project: "PROJ".to_string(),
            source_kind: "issuetype".to_string(),
            synced_at: "2026-06-16T12:00:00Z".to_string(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(!json.contains("description"));
        assert!(!json.contains("icon_url"));
        assert!(json.contains("\"source_kind\":\"issuetype\""));
    }

    #[test]
    fn test_write_kanban_config_request_targets_single_provider() {
        let req = WriteKanbanConfigRequest {
            provider: KanbanProviderKind::Github,
            jira: None,
            linear: None,
            github: Some(WriteGithubConfigBody {
                owner: "acme".to_string(),
                api_key_env: "OPERATOR_GITHUB_TOKEN".to_string(),
                project_key: "PVT_kwDOABcdefg".to_string(),
                sync_user_id: "123".to_string(),
            }),
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"provider\":\"github\""));
        assert!(json.contains("\"github\":{"));
        assert!(!json.contains("\"jira\":"));
        assert!(!json.contains("\"linear\":"));
    }
}

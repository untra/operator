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

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

// ─── Kanban Provider Configuration ─────────────────────────────────────────

/// Kanban provider configuration for syncing issues from external systems
///
/// Providers are keyed by domain/workspace:
/// - Jira: keyed by domain (e.g., "foobar.atlassian.net")
/// - Linear: keyed by workspace slug (e.g., "myworkspace")
/// - GitHub Projects: keyed by owner login (e.g., "my-org")
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS, Default)]
#[ts(export)]
pub struct KanbanConfig {
    /// Jira Cloud instances keyed by domain (e.g., "foobar.atlassian.net")
    #[serde(default)]
    pub jira: std::collections::HashMap<String, JiraConfig>,
    /// Linear instances keyed by workspace slug
    #[serde(default)]
    pub linear: std::collections::HashMap<String, LinearConfig>,
    /// GitHub Projects v2 instances keyed by owner login (user or org)
    ///
    /// NOTE: This is the *kanban* GitHub integration (Projects v2), distinct
    /// from `GitHubConfig` which is the *git provider* used for PRs and
    /// branches. The two use different env vars and different scopes — see
    /// `docs/getting-started/kanban/github.md` for the full disambiguation.
    #[serde(default)]
    pub github: std::collections::HashMap<String, GithubProjectsConfig>,
    /// `OpenSpec` roots keyed by a free-form instance name (e.g., a repo alias).
    /// Experimental, pull-only: each active change under `<root_path>/changes/`
    /// acts as a kanban "project" whose issues are the tasks.md task groups.
    #[serde(default)]
    pub openspec: std::collections::HashMap<String, OpenspecConfig>,
}

/// Jira Cloud provider configuration
///
/// The domain is specified as the `HashMap` key in KanbanConfig.jira
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export)]
pub struct JiraConfig {
    /// Whether this provider is enabled
    #[serde(default)]
    pub enabled: bool,
    /// Environment variable name containing the API key (default: `OPERATOR_JIRA_API_KEY`)
    #[serde(default = "default_jira_api_key_env")]
    pub api_key_env: String,
    /// Atlassian account email for authentication
    #[serde(default)]
    pub email: String,
    /// Per-project sync configuration
    #[serde(default)]
    pub projects: std::collections::HashMap<String, ProjectSyncConfig>,
}

fn default_jira_api_key_env() -> String {
    "OPERATOR_JIRA_API_KEY".to_string()
}

impl Default for JiraConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            api_key_env: default_jira_api_key_env(),
            email: String::new(),
            projects: std::collections::HashMap::new(),
        }
    }
}

/// Linear provider configuration
///
/// The workspace slug is specified as the `HashMap` key in KanbanConfig.linear
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export)]
pub struct LinearConfig {
    /// Whether this provider is enabled
    #[serde(default)]
    pub enabled: bool,
    /// Environment variable name containing the API key (default: `OPERATOR_LINEAR_API_KEY`)
    #[serde(default = "default_linear_api_key_env")]
    pub api_key_env: String,
    /// Per-team sync configuration
    #[serde(default)]
    pub projects: std::collections::HashMap<String, ProjectSyncConfig>,
}

fn default_linear_api_key_env() -> String {
    "OPERATOR_LINEAR_API_KEY".to_string()
}

impl Default for LinearConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            api_key_env: default_linear_api_key_env(),
            projects: std::collections::HashMap::new(),
        }
    }
}

/// GitHub Projects v2 (kanban) provider configuration
///
/// The owner login (user or org) is specified as the `HashMap` key in
/// `KanbanConfig.github`. Project keys inside `projects` are `GraphQL` node
/// IDs (e.g., `PVT_kwDOABcdefg`) — opaque, stable identifiers used directly
/// by every GitHub Projects v2 mutation without needing a lookup.
///
/// **Distinct from `GitHubConfig`** (the git provider used for PR/branch
/// operations). They live in different parts of the config tree, use
/// different env vars (`OPERATOR_GITHUB_TOKEN` vs `GITHUB_TOKEN`), and
/// require different OAuth scopes (`project` vs `repo`). See
/// `docs/getting-started/kanban/github.md` for the full rationale.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export)]
pub struct GithubProjectsConfig {
    /// Whether this provider is enabled
    #[serde(default)]
    pub enabled: bool,
    /// Environment variable name containing the GitHub token (default:
    /// `OPERATOR_GITHUB_TOKEN`). The token must have `project` (or
    /// `read:project`) scope, NOT just `repo` — see the disambiguation
    /// guide in the kanban github docs.
    #[serde(default = "default_github_projects_api_key_env")]
    pub api_key_env: String,
    /// Per-project sync configuration. Keys are `GraphQL` project node IDs.
    #[serde(default)]
    pub projects: std::collections::HashMap<String, ProjectSyncConfig>,
}

fn default_github_projects_api_key_env() -> String {
    "OPERATOR_GITHUB_TOKEN".to_string()
}

impl Default for GithubProjectsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            api_key_env: default_github_projects_api_key_env(),
            projects: std::collections::HashMap::new(),
        }
    }
}

/// `OpenSpec` provider configuration (experimental, pull-only)
///
/// The instance name is the `HashMap` key in `KanbanConfig.openspec`. There
/// are no credentials — the provider reads local markdown under `root_path`.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS, Default)]
#[ts(export)]
pub struct OpenspecConfig {
    /// Whether this provider is enabled
    #[serde(default)]
    pub enabled: bool,
    /// Directory containing the `OpenSpec` `changes/` tree (typically `<repo>/openspec`)
    #[serde(default)]
    pub root_path: String,
    /// Operator project stamped on imported tickets (defaults to the change id)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
}

impl KanbanConfig {
    /// Insert or update a Jira project entry in the config.
    ///
    /// If the workspace (keyed by domain) doesn't exist, it is created with
    /// `enabled = true` and the provided email + `api_key_env`. If it already
    /// exists, the email and `api_key_env` are updated and the project is
    /// upserted into its `projects` map without clobbering sibling projects.
    pub fn upsert_jira_project(
        &mut self,
        domain: &str,
        email: &str,
        api_key_env: &str,
        project_key: &str,
        sync_user_id: &str,
        status_mapping: KanbanStatusMapping,
    ) {
        let entry = self.jira.entry(domain.to_string()).or_default();
        entry.enabled = true;
        entry.email = email.to_string();
        entry.api_key_env = api_key_env.to_string();
        entry.projects.insert(
            project_key.to_string(),
            ProjectSyncConfig {
                sync_user_id: sync_user_id.to_string(),
                status_mapping,
                collection_name: None,
                type_mappings: std::collections::HashMap::new(),
                bidirectional: false,
                ticket_project: None,
            },
        );
    }

    /// Insert or update a Linear team entry in the config.
    ///
    /// If the workspace (keyed by workspace slug) doesn't exist, it is
    /// created with `enabled = true` and the provided `api_key_env`. If it
    /// already exists, the `api_key_env` is updated and the project/team is
    /// upserted into its `projects` map without clobbering siblings.
    pub fn upsert_linear_project(
        &mut self,
        workspace: &str,
        api_key_env: &str,
        project_key: &str,
        sync_user_id: &str,
        status_mapping: KanbanStatusMapping,
    ) {
        let entry = self.linear.entry(workspace.to_string()).or_default();
        entry.enabled = true;
        entry.api_key_env = api_key_env.to_string();
        entry.projects.insert(
            project_key.to_string(),
            ProjectSyncConfig {
                sync_user_id: sync_user_id.to_string(),
                status_mapping,
                collection_name: None,
                type_mappings: std::collections::HashMap::new(),
                bidirectional: false,
                ticket_project: None,
            },
        );
    }

    /// Insert or update a GitHub Projects v2 entry in the config.
    ///
    /// If the owner (keyed by login) doesn't exist, it is created with
    /// `enabled = true` and the provided `api_key_env`. If it already
    /// exists, the `api_key_env` is updated and the project is upserted
    /// into its `projects` map without clobbering siblings.
    ///
    /// `project_key` is the `GraphQL` project node ID (e.g., `PVT_kwDO...`)
    /// and `sync_user_id` is the user's numeric GitHub `databaseId`.
    pub fn upsert_github_project(
        &mut self,
        owner: &str,
        api_key_env: &str,
        project_key: &str,
        sync_user_id: &str,
        status_mapping: KanbanStatusMapping,
    ) {
        let entry = self.github.entry(owner.to_string()).or_default();
        entry.enabled = true;
        entry.api_key_env = api_key_env.to_string();
        entry.projects.insert(
            project_key.to_string(),
            ProjectSyncConfig {
                sync_user_id: sync_user_id.to_string(),
                status_mapping,
                collection_name: None,
                type_mappings: std::collections::HashMap::new(),
                bidirectional: false,
                ticket_project: None,
            },
        );
    }

    /// Provider-neutral upsert dispatcher.
    ///
    /// Delegates to the provider-specific upsert method based on the
    /// `WorkspaceExtra` variant in the validated workspace.
    #[allow(dead_code)] // Used in tests
    pub fn upsert_project(
        &mut self,
        workspace: &crate::api::providers::kanban::ValidatedWorkspace,
        project: &crate::api::providers::kanban::DiscoveredProject,
    ) {
        use crate::api::providers::kanban::WorkspaceExtra;
        match &workspace.extra {
            WorkspaceExtra::Jira { email } => self.upsert_jira_project(
                &workspace.workspace_key,
                email,
                &workspace.api_key_env,
                &project.project_key,
                &workspace.sync_user_id,
                KanbanStatusMapping::default(),
            ),
            WorkspaceExtra::Linear => self.upsert_linear_project(
                &workspace.workspace_key,
                &workspace.api_key_env,
                &project.project_key,
                &workspace.sync_user_id,
                KanbanStatusMapping::default(),
            ),
            WorkspaceExtra::Github => self.upsert_github_project(
                &workspace.workspace_key,
                &workspace.api_key_env,
                &project.project_key,
                &workspace.sync_user_id,
                KanbanStatusMapping::default(),
            ),
            // OpenSpec has no credentials or per-project sync entries; the
            // instance itself is the whole configuration.
            WorkspaceExtra::Openspec { root_path } => {
                self.upsert_openspec_root(&workspace.workspace_key, root_path, None);
            }
        }
    }

    /// Insert or update an `OpenSpec` root entry (no credentials, just a path).
    pub fn upsert_openspec_root(&mut self, instance: &str, root_path: &str, project: Option<&str>) {
        let entry = self.openspec.entry(instance.to_string()).or_default();
        entry.enabled = true;
        entry.root_path = root_path.to_string();
        if project.is_some() {
            entry.project = project.map(ToString::to_string);
        }
    }
}

/// Explicit mapping from operator's strict todo/doing/done states to the
/// external board's column/status names.
///
/// Drives bidirectional sync: issues are pulled from the `todo` column,
/// pushed to `doing` when a ticket is claimed, to `done` when completed, and
/// back to `todo` when requeued. Unset fields fall back per-transition
/// (`doing` → "In Progress", `done` → "Done"); requeue only pushes when
/// `todo` is explicitly mapped.
#[derive(
    Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS, utoipa::ToSchema,
)]
#[ts(export)]
pub struct KanbanStatusMapping {
    /// External column for operator "todo" (queued work; also the pull source)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub todo: Option<String>,
    /// External column for operator "doing" (claimed/launched tickets)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub doing: Option<String>,
    /// External column for operator "done" (completed tickets)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub done: Option<String>,
}

impl KanbanStatusMapping {
    /// True when no column is mapped (used to omit the table from TOML).
    pub fn is_empty(&self) -> bool {
        self.todo.is_none() && self.doing.is_none() && self.done.is_none()
    }

    /// Number of mapped columns.
    pub fn mapped_count(&self) -> usize {
        [&self.todo, &self.doing, &self.done]
            .iter()
            .filter(|s| s.is_some())
            .count()
    }
}

/// Per-project/team sync configuration for a kanban provider
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export)]
pub struct ProjectSyncConfig {
    /// User ID to sync issues for (provider-specific format)
    /// - Jira: accountId (e.g., "5e3f7acd9876543210abcdef")
    /// - Linear: user ID (e.g., "abc12345-6789-0abc-def0-123456789abc")
    /// - GitHub Projects: numeric GitHub `databaseId` (e.g., "12345678")
    #[serde(default)]
    pub sync_user_id: String,
    /// Mapping of operator todo/doing/done to external board columns
    #[serde(default, skip_serializing_if = "KanbanStatusMapping::is_empty")]
    pub status_mapping: KanbanStatusMapping,
    /// Optional `IssueTypeCollection` name this project maps to.
    /// Not required for kanban onboarding or sync.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub collection_name: Option<String>,
    /// Explicit mapping: kanban issue type ID → operator issue type key (e.g., TASK, FEAT, FIX).
    /// Multiple kanban types can map to the same operator template.
    #[serde(default)]
    pub type_mappings: std::collections::HashMap<String, String>,
    /// When true, operator pushes status changes and activity logs back to this kanban project.
    /// Ticket state changes (todo→doing, doing→done) and step completions with delegator info
    /// are reflected upstream. Default: false.
    #[serde(default)]
    pub bidirectional: bool,
    /// Operator project name stamped on tickets created from this source.
    /// Defaults to the external project key when unset.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ticket_project: Option<String>,
}

impl ProjectSyncConfig {
    /// Statuses to pull from the external board: the mapped `todo` column
    /// (queued work) plus `doing` (resume in-flight). Empty when unmapped —
    /// providers then fall back to their default status filter.
    pub fn pull_statuses(&self) -> Vec<String> {
        [&self.status_mapping.todo, &self.status_mapping.doing]
            .into_iter()
            .flatten()
            .cloned()
            .collect()
    }
}

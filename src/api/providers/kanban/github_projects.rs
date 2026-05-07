//! GitHub Projects v2 (kanban) provider implementation
//!
//! # Token Disambiguation
//!
//! GitHub uses one token type but two scope families. Operator splits them
//! into two distinct env vars/config trees:
//!
//! | Subsystem               | Env var                  | Required scopes                          |
//! |-------------------------|--------------------------|------------------------------------------|
//! | Git provider (PRs)      | `GITHUB_TOKEN`           | `repo`                                   |
//! | Kanban provider (this)  | `OPERATOR_GITHUB_TOKEN`  | `project` or `read:project`              |
//!
//! `from_env()` here reads **only** `OPERATOR_GITHUB_TOKEN` and never falls
//! back to `GITHUB_TOKEN`. `validate_detailed()` performs scope verification
//! and returns a friendly "lacks `project` scope" error if the token looks
//! like a repo-only token. See `docs/getting-started/kanban/github.md`.

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use tokio::sync::RwLock;
use tracing::{debug, warn};

use super::{
    CreateIssueRequest, CreateIssueResponse, ExternalIssue, ExternalIssueType, ExternalUser,
    KanbanProvider, ProjectInfo, UpdateStatusRequest,
};
use crate::api::error::ApiError;
use crate::issuetypes::kanban_type::KanbanIssueTypeRef;

const GITHUB_GRAPHQL_URL: &str = "https://api.github.com/graphql";
const GITHUB_REST_USER_URL: &str = "https://api.github.com/user";
const PROVIDER_NAME: &str = "github";
const USER_AGENT: &str = concat!("operator/", env!("CARGO_PKG_VERSION"));
const DEFAULT_ENV_VAR: &str = "OPERATOR_GITHUB_TOKEN";

/// Friendly error returned when a token authenticated but lacks `project` scope.
const SCOPE_ERROR_MSG: &str =
    "Token authenticated but lacks 'project' scope. This looks like a repo-scoped token \
     (the kind operator uses for GitHub PR workflows via GITHUB_TOKEN). Mint a new PAT at \
     https://github.com/settings/tokens with the 'project' (or 'read:project') scope, or \
     extend a fine-grained PAT to include the Projects permission. \
     See docs/getting-started/kanban/github.md.";

// ─── Public types ────────────────────────────────────────────────────────────

/// Info about a GitHub Project v2 returned by `validate_detailed`.
#[derive(Debug, Clone)]
pub struct GithubProjectInfo {
    pub node_id: String,
    pub number: i32,
    pub title: String,
    pub owner_login: String,
    /// "Organization" or "User"
    pub owner_kind: String,
}

/// Detailed validation result for GitHub Projects onboarding.
#[derive(Debug, Clone)]
pub struct GithubValidationDetails {
    /// Authenticated user's login
    pub user_login: String,
    /// Authenticated user's `databaseId` rendered as a string (used as `sync_user_id`)
    pub user_id: String,
    /// Projects visible to the token (across viewer + orgs)
    pub projects: Vec<GithubProjectInfo>,
    /// Env var name the validated token came from. Surfaced to clients so
    /// they can display "Connected via X" and rotate the right token.
    pub resolved_env_var: String,
}

// ─── Status field cache ──────────────────────────────────────────────────────

/// Cached `Status` field info for a single project.
#[derive(Debug, Clone)]
struct StatusFieldCache {
    field_id: String,
    /// Lowercased option name → option id (for case-insensitive lookup).
    options_by_name: HashMap<String, String>,
    /// Original-case option names in declared order (for `list_statuses` output).
    ordered_names: Vec<String>,
}

/// Resolved (project, item) pair for a given external `issue_key`.
///
/// Populated by `list_issues` so `update_issue_status` can resolve the
/// human-readable key (e.g. `octocat/hello#42`) back to the `GraphQL` IDs
/// it needs for the mutation. Cache miss → `update_issue_status` returns
/// a clear error asking the caller to run `list_issues` first.
#[derive(Debug, Clone)]
struct ItemLookup {
    project_id: String,
    item_id: String,
}

// ─── Provider struct ─────────────────────────────────────────────────────────

/// GitHub Projects v2 (kanban) API provider.
pub struct GithubProjectsProvider {
    token: String,
    client: Client,
    /// Env var the token was sourced from. Used by `validate_detailed`.
    resolved_env_var: String,
    /// `project_node_id` → cached Status field info.
    status_field_cache: RwLock<HashMap<String, StatusFieldCache>>,
    /// `issue_key` (as returned in `ExternalIssue.key`) → lookup info, populated by `list_issues`.
    item_lookup: RwLock<HashMap<String, ItemLookup>>,
}

impl GithubProjectsProvider {
    /// Create a new GitHub Projects provider.
    ///
    /// `resolved_env_var` should be the name of the env var the token came
    /// from (e.g. `"OPERATOR_GITHUB_TOKEN"`), used for "Connected via X"
    /// feedback in the validation response.
    pub fn new(token: String, resolved_env_var: String) -> Self {
        Self {
            token,
            client: Client::new(),
            resolved_env_var,
            status_field_cache: RwLock::new(HashMap::new()),
            item_lookup: RwLock::new(HashMap::new()),
        }
    }

    /// Create from environment.
    ///
    /// Reads **only** `OPERATOR_GITHUB_TOKEN`. Does **not** fall back to
    /// `GITHUB_TOKEN` even if it exists — that env var belongs to operator's
    /// git provider (PR/branch workflows) and almost certainly lacks the
    /// `project` scope, which would surface confusing 403s deeper in the
    /// stack. See module-level Token Disambiguation note.
    pub fn from_env() -> Result<Self, ApiError> {
        match env::var(DEFAULT_ENV_VAR) {
            Ok(token) if !token.is_empty() => Ok(Self::new(token, DEFAULT_ENV_VAR.to_string())),
            _ => Err(ApiError::not_configured(PROVIDER_NAME)),
        }
    }

    /// Create from config. The owner is passed for symmetry with the other
    /// providers (it's the `HashMap` key in `KanbanConfig.github`); the
    /// token itself is read from the env var named in `config.api_key_env`.
    pub fn from_config(
        _owner: &str,
        config: &crate::config::GithubProjectsConfig,
    ) -> Result<Self, ApiError> {
        let token = env::var(&config.api_key_env).ok();
        match token {
            Some(t) if !t.is_empty() => Ok(Self::new(t, config.api_key_env.clone())),
            _ => Err(ApiError::not_configured(PROVIDER_NAME)),
        }
    }

    /// Build the standard set of headers used for both `GraphQL` and REST calls.
    fn auth_headers(&self) -> reqwest::header::HeaderMap {
        use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, USER_AGENT as UA};
        let mut h = HeaderMap::new();
        h.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", self.token))
                .unwrap_or_else(|_| HeaderValue::from_static("Bearer invalid")),
        );
        h.insert(
            ACCEPT,
            HeaderValue::from_static("application/vnd.github+json"),
        );
        h.insert(UA, HeaderValue::from_static(USER_AGENT));
        h
    }

    /// Execute a `GraphQL` query against the GitHub API.
    async fn graphql<T: for<'de> Deserialize<'de>>(
        &self,
        query: &str,
        variables: Option<serde_json::Value>,
    ) -> Result<T, ApiError> {
        #[derive(Serialize)]
        struct GraphQLRequest<'a> {
            query: &'a str,
            #[serde(skip_serializing_if = "Option::is_none")]
            variables: Option<serde_json::Value>,
        }

        #[derive(Deserialize)]
        struct GraphQLResponse<T> {
            data: Option<T>,
            errors: Option<Vec<GraphQLError>>,
        }

        #[derive(Deserialize)]
        struct GraphQLError {
            message: String,
            #[serde(default)]
            #[allow(dead_code)]
            #[serde(rename = "type")]
            err_type: Option<String>,
        }

        let request = GraphQLRequest { query, variables };

        debug!("GitHub GraphQL query");

        let response = self
            .client
            .post(GITHUB_GRAPHQL_URL)
            .headers(self.auth_headers())
            .json(&request)
            .send()
            .await
            .map_err(|e| ApiError::network(PROVIDER_NAME, e.to_string()))?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return match status.as_u16() {
                401 => Err(ApiError::unauthorized(PROVIDER_NAME)),
                403 => Err(ApiError::http(PROVIDER_NAME, 403, body)),
                429 => Err(ApiError::rate_limited(PROVIDER_NAME, None)),
                _ => Err(ApiError::http(PROVIDER_NAME, status.as_u16(), body)),
            };
        }

        let gql_response: GraphQLResponse<T> = response
            .json()
            .await
            .map_err(|e| ApiError::http(PROVIDER_NAME, 0, format!("Parse error: {e}")))?;

        if let Some(errors) = gql_response.errors {
            let messages: Vec<String> = errors.into_iter().map(|e| e.message).collect();
            let combined = messages.join("; ");
            // If the `GraphQL` errors mention permissions/scopes/projects, escalate
            // with the friendly disambiguation hint so users see it via the
            // generic provider_error_message helper. Preserve the raw error so
            // legitimate bugs (field-level permission failures, feature-gated
            // fields, etc.) are still debuggable — the hint alone was masking
            // real root causes.
            let lower = combined.to_lowercase();
            if lower.contains("project")
                && (lower.contains("permission")
                    || lower.contains("scope")
                    || lower.contains("not authorized"))
            {
                return Err(ApiError::http(
                    PROVIDER_NAME,
                    403,
                    format!("{SCOPE_ERROR_MSG} (raw GraphQL error: {combined})"),
                ));
            }
            return Err(ApiError::http(PROVIDER_NAME, 0, combined));
        }

        gql_response
            .data
            .ok_or_else(|| ApiError::http(PROVIDER_NAME, 0, "No data in response".to_string()))
    }

    /// Detailed credential validation for onboarding.
    ///
    /// Performs:
    ///
    /// 1. A `viewer { login databaseId projectsV2 organizations }` `GraphQL` query
    ///    to confirm the token is valid and to enumerate visible projects.
    /// 2. A side-channel `GET /user` REST call to read the `x-oauth-scopes`
    ///    header (classic PATs only). If the header is non-empty and does
    ///    not include `project` or `read:project`, returns a friendly error.
    /// 3. A behavior probe: if the `GraphQL` query surfaced no projects at
    ///    all (and the header check was inconclusive, as for fine-grained
    ///    PATs), treats that as a likely scope problem and returns the same
    ///    friendly error.
    pub async fn validate_detailed(&self) -> Result<GithubValidationDetails, ApiError> {
        let query = r"
            query {
                viewer {
                    login
                    databaseId
                    projectsV2(first: 50) {
                        nodes {
                            id
                            number
                            title
                            owner {
                                __typename
                                ... on Organization { login }
                                ... on User { login }
                            }
                        }
                    }
                    organizations(first: 20) {
                        nodes {
                            login
                            projectsV2(first: 50) {
                                nodes {
                                    id
                                    number
                                    title
                                }
                            }
                        }
                    }
                }
            }
        ";

        #[derive(Deserialize)]
        struct ValidateResponse {
            viewer: ViewerNode,
        }

        #[derive(Deserialize)]
        struct ViewerNode {
            login: String,
            #[serde(rename = "databaseId")]
            database_id: i64,
            #[serde(rename = "projectsV2")]
            projects_v2: ProjectsV2Conn,
            organizations: OrgsConn,
        }

        #[derive(Deserialize)]
        struct ProjectsV2Conn {
            nodes: Vec<ProjectNode>,
        }

        #[derive(Deserialize)]
        struct ProjectNode {
            id: String,
            number: i32,
            title: String,
            #[serde(default)]
            owner: Option<OwnerRef>,
        }

        #[derive(Deserialize)]
        struct OwnerRef {
            #[serde(rename = "__typename")]
            typename: String,
            #[serde(default)]
            login: Option<String>,
        }

        #[derive(Deserialize)]
        struct OrgsConn {
            nodes: Vec<OrgNode>,
        }

        #[derive(Deserialize)]
        struct OrgNode {
            login: String,
            #[serde(rename = "projectsV2")]
            projects_v2: OrgProjectsConn,
        }

        #[derive(Deserialize)]
        struct OrgProjectsConn {
            nodes: Vec<OrgProjectNode>,
        }

        #[derive(Deserialize)]
        struct OrgProjectNode {
            id: String,
            number: i32,
            title: String,
        }

        let resp: ValidateResponse = self.graphql(query, None).await?;

        let mut projects: Vec<GithubProjectInfo> = Vec::new();

        // Viewer's own projects (User-owned).
        for p in resp.viewer.projects_v2.nodes {
            let (owner_login, owner_kind) = p
                .owner
                .map(|o| (o.login.unwrap_or_default(), o.typename))
                .unwrap_or_else(|| (resp.viewer.login.clone(), "User".to_string()));
            projects.push(GithubProjectInfo {
                node_id: p.id,
                number: p.number,
                title: p.title,
                owner_login,
                owner_kind,
            });
        }

        // Org-owned projects.
        for org in resp.viewer.organizations.nodes {
            for p in org.projects_v2.nodes {
                projects.push(GithubProjectInfo {
                    node_id: p.id,
                    number: p.number,
                    title: p.title,
                    owner_login: org.login.clone(),
                    owner_kind: "Organization".to_string(),
                });
            }
        }

        // Scope verification — header scrape (classic PATs).
        let scopes_header = self.fetch_oauth_scopes().await;

        if let Some(scopes) = &scopes_header {
            let lower = scopes.to_lowercase();
            if !lower.contains("project") && !lower.contains("read:project") {
                return Err(ApiError::http(
                    PROVIDER_NAME,
                    403,
                    SCOPE_ERROR_MSG.to_string(),
                ));
            }
        } else if projects.is_empty() {
            // Fine-grained PAT (no x-oauth-scopes header) AND no projects came
            // back. Most likely cause: token lacks Projects permission.
            return Err(ApiError::http(
                PROVIDER_NAME,
                403,
                SCOPE_ERROR_MSG.to_string(),
            ));
        }

        Ok(GithubValidationDetails {
            user_login: resp.viewer.login,
            user_id: resp.viewer.database_id.to_string(),
            projects,
            resolved_env_var: self.resolved_env_var.clone(),
        })
    }

    /// Fetch the `x-oauth-scopes` header via a `GET /user` REST call.
    ///
    /// Returns `None` if the header is absent (fine-grained PATs and app
    /// tokens don't return it) or the request fails.
    async fn fetch_oauth_scopes(&self) -> Option<String> {
        let resp = self
            .client
            .get(GITHUB_REST_USER_URL)
            .headers(self.auth_headers())
            .send()
            .await
            .ok()?;

        if !resp.status().is_success() {
            return None;
        }

        resp.headers()
            .get("x-oauth-scopes")
            .and_then(|v| v.to_str().ok())
            .map(std::string::ToString::to_string)
            .filter(|s| !s.is_empty())
    }

    /// Resolve the owner login + kind for a project node id.
    ///
    /// Used by `get_issue_types` to know which org to query for `issueTypes`.
    async fn resolve_project_owner(&self, project_id: &str) -> Result<(String, String), ApiError> {
        let query = r"
            query($projectId: ID!) {
                node(id: $projectId) {
                    ... on ProjectV2 {
                        owner {
                            __typename
                            ... on Organization { login }
                            ... on User { login }
                        }
                    }
                }
            }
        ";

        #[derive(Deserialize)]
        struct Resp {
            node: NodeWrap,
        }

        #[derive(Deserialize)]
        struct NodeWrap {
            #[serde(default)]
            owner: Option<OwnerRef>,
        }

        #[derive(Deserialize)]
        struct OwnerRef {
            #[serde(rename = "__typename")]
            typename: String,
            #[serde(default)]
            login: Option<String>,
        }

        let variables = serde_json::json!({ "projectId": project_id });
        let resp: Resp = self.graphql(query, Some(variables)).await?;
        let owner = resp.node.owner.ok_or_else(|| {
            ApiError::http(
                PROVIDER_NAME,
                404,
                format!("Project {project_id} has no owner"),
            )
        })?;
        Ok((owner.login.unwrap_or_default(), owner.typename))
    }

    /// Try to fetch org-level issue types. Returns `Ok(None)` if the project
    /// owner is a User (orgs only) or if the org has no issue types
    /// configured. Returns `Err` only on auth/network failures.
    async fn fetch_org_issue_types(
        &self,
        owner_login: &str,
    ) -> Result<Option<Vec<ExternalIssueType>>, ApiError> {
        let query = r"
            query($login: String!) {
                organization(login: $login) {
                    issueTypes(first: 20) {
                        nodes {
                            id
                            name
                            description
                        }
                    }
                }
            }
        ";

        #[derive(Deserialize)]
        struct Resp {
            #[serde(default)]
            organization: Option<OrgWrap>,
        }

        #[derive(Deserialize)]
        struct OrgWrap {
            #[serde(rename = "issueTypes", default)]
            issue_types: Option<TypesConn>,
        }

        #[derive(Deserialize)]
        struct TypesConn {
            nodes: Vec<IssueTypeNode>,
        }

        #[derive(Deserialize)]
        struct IssueTypeNode {
            id: String,
            name: String,
            #[serde(default)]
            description: Option<String>,
        }

        let variables = serde_json::json!({ "login": owner_login });
        let resp: Result<Resp, ApiError> = self.graphql(query, Some(variables)).await;

        match resp {
            Ok(r) => {
                let nodes = r
                    .organization
                    .and_then(|o| o.issue_types)
                    .map(|t| t.nodes)
                    .unwrap_or_default();
                if nodes.is_empty() {
                    Ok(None)
                } else {
                    Ok(Some(
                        nodes
                            .into_iter()
                            .map(|n| ExternalIssueType {
                                id: n.id,
                                name: n.name,
                                description: n.description,
                                icon_url: None,
                                custom_fields: vec![],
                            })
                            .collect(),
                    ))
                }
            }
            // `GraphQL` errors here usually mean the schema doesn't expose
            // `issueTypes` (older orgs) — treat that as "no types available"
            // and let the caller fall back to labels.
            Err(ApiError::HttpError { message, .. }) if message.contains("issueTypes") => {
                warn!("issueTypes field not available, falling back to labels");
                Ok(None)
            }
            Err(e) => Err(e),
        }
    }

    /// Aggregate labels from repos linked to the given project, deduped by id.
    /// Used as the fallback path when org-level issue types aren't available.
    async fn fetch_project_labels(
        &self,
        project_id: &str,
    ) -> Result<Vec<ExternalIssueType>, ApiError> {
        let query = r"
            query($projectId: ID!) {
                node(id: $projectId) {
                    ... on ProjectV2 {
                        items(first: 100) {
                            nodes {
                                content {
                                    __typename
                                    ... on Issue {
                                        repository {
                                            labels(first: 50) {
                                                nodes { id name description }
                                            }
                                        }
                                    }
                                    ... on PullRequest {
                                        repository {
                                            labels(first: 50) {
                                                nodes { id name description }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        ";

        #[derive(Deserialize)]
        struct Resp {
            node: NodeWrap,
        }

        #[derive(Deserialize)]
        struct NodeWrap {
            #[serde(default)]
            items: Option<ItemsConn>,
        }

        #[derive(Deserialize)]
        struct ItemsConn {
            nodes: Vec<ItemNode>,
        }

        #[derive(Deserialize)]
        struct ItemNode {
            #[serde(default)]
            content: Option<ContentNode>,
        }

        #[derive(Deserialize)]
        struct ContentNode {
            #[serde(default)]
            repository: Option<RepoNode>,
        }

        #[derive(Deserialize)]
        struct RepoNode {
            #[serde(default)]
            labels: Option<LabelsConn>,
        }

        #[derive(Deserialize)]
        struct LabelsConn {
            nodes: Vec<LabelNode>,
        }

        #[derive(Deserialize)]
        struct LabelNode {
            id: String,
            name: String,
            #[serde(default)]
            description: Option<String>,
        }

        let variables = serde_json::json!({ "projectId": project_id });
        let resp: Resp = self.graphql(query, Some(variables)).await?;

        let mut by_id: HashMap<String, ExternalIssueType> = HashMap::new();
        if let Some(items) = resp.node.items {
            for item in items.nodes {
                let Some(repo) = item.content.and_then(|c| c.repository) else {
                    continue;
                };
                let Some(labels) = repo.labels else { continue };
                for label in labels.nodes {
                    by_id.entry(label.id.clone()).or_insert(ExternalIssueType {
                        id: label.id,
                        name: label.name,
                        description: label.description,
                        icon_url: None,
                        custom_fields: vec![],
                    });
                }
            }
        }
        Ok(by_id.into_values().collect())
    }

    /// Load + cache the `Status` single-select field for a project.
    async fn ensure_status_field(&self, project_id: &str) -> Result<StatusFieldCache, ApiError> {
        if let Some(cached) = self.status_field_cache.read().await.get(project_id) {
            return Ok(cached.clone());
        }

        let query = r#"
            query($projectId: ID!) {
                node(id: $projectId) {
                    ... on ProjectV2 {
                        field(name: "Status") {
                            __typename
                            ... on ProjectV2SingleSelectField {
                                id
                                name
                                options { id name }
                            }
                        }
                    }
                }
            }
        "#;

        #[derive(Deserialize)]
        struct Resp {
            node: NodeWrap,
        }

        #[derive(Deserialize)]
        struct NodeWrap {
            #[serde(default)]
            field: Option<FieldNode>,
        }

        #[derive(Deserialize)]
        struct FieldNode {
            id: String,
            #[serde(default)]
            options: Vec<OptionNode>,
        }

        #[derive(Deserialize)]
        struct OptionNode {
            id: String,
            name: String,
        }

        let variables = serde_json::json!({ "projectId": project_id });
        let resp: Resp = self.graphql(query, Some(variables)).await?;
        let field = resp.node.field.ok_or_else(|| {
            ApiError::http(
                PROVIDER_NAME,
                404,
                format!("Project {project_id} has no Status field"),
            )
        })?;

        let mut options_by_name: HashMap<String, String> = HashMap::new();
        let mut ordered_names: Vec<String> = Vec::new();
        for opt in field.options {
            options_by_name.insert(opt.name.to_lowercase(), opt.id.clone());
            ordered_names.push(opt.name);
        }

        let cache = StatusFieldCache {
            field_id: field.id,
            options_by_name,
            ordered_names,
        };

        self.status_field_cache
            .write()
            .await
            .insert(project_id.to_string(), cache.clone());

        Ok(cache)
    }
}

// ─── Item / list_issues response types ──────────────────────────────────────

#[derive(Debug, Deserialize)]
struct ListItemsResponse {
    node: ListItemsNode,
}

#[derive(Debug, Deserialize)]
struct ListItemsNode {
    #[serde(default)]
    items: Option<ItemsPage>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ItemsPage {
    #[serde(default)]
    page_info: Option<PageInfo>,
    #[serde(default)]
    nodes: Vec<RawProjectItem>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PageInfo {
    has_next_page: bool,
    end_cursor: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RawProjectItem {
    id: String,
    #[serde(default, rename = "type")]
    item_type: Option<String>,
    #[serde(default)]
    content: Option<RawContent>,
    #[serde(default, rename = "fieldValues")]
    field_values: Option<RawFieldValues>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "__typename")]
enum RawContent {
    Issue {
        #[serde(default)]
        id: Option<String>,
        #[serde(default)]
        number: Option<i32>,
        title: String,
        #[serde(default)]
        body: Option<String>,
        #[serde(default)]
        url: Option<String>,
        #[serde(default)]
        repository: Option<RawRepoRef>,
        #[serde(default)]
        assignees: Option<RawAssignees>,
        #[serde(default)]
        labels: Option<RawLabels>,
        #[serde(default, rename = "issueType")]
        issue_type: Option<RawIssueType>,
    },
    PullRequest {
        #[serde(default)]
        id: Option<String>,
        #[serde(default)]
        number: Option<i32>,
        title: String,
        #[serde(default)]
        body: Option<String>,
        #[serde(default)]
        url: Option<String>,
        #[serde(default)]
        repository: Option<RawRepoRef>,
        #[serde(default)]
        assignees: Option<RawAssignees>,
        #[serde(default)]
        labels: Option<RawLabels>,
    },
    DraftIssue {
        #[serde(default)]
        id: Option<String>,
        title: String,
        #[serde(default)]
        body: Option<String>,
        #[serde(default)]
        assignees: Option<RawAssignees>,
    },
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawRepoRef {
    name_with_owner: String,
}

#[derive(Debug, Deserialize)]
struct RawAssignees {
    nodes: Vec<RawAssignee>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawAssignee {
    login: String,
    #[serde(default)]
    database_id: Option<i64>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    email: Option<String>,
    #[serde(default)]
    avatar_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RawLabels {
    nodes: Vec<RawLabel>,
}

#[derive(Debug, Deserialize)]
struct RawLabel {
    id: String,
    name: String,
}

#[derive(Debug, Deserialize)]
struct RawIssueType {
    id: String,
    name: String,
}

#[derive(Debug, Deserialize)]
struct RawFieldValues {
    nodes: Vec<RawFieldValue>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "__typename")]
enum RawFieldValue {
    ProjectV2ItemFieldSingleSelectValue {
        #[serde(default)]
        name: Option<String>,
        #[serde(default)]
        field: Option<RawFieldRef>,
    },
    #[serde(other)]
    Other,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "__typename")]
enum RawFieldRef {
    ProjectV2SingleSelectField {
        #[serde(default)]
        name: Option<String>,
    },
    #[serde(other)]
    Other,
}

// ─── KanbanProvider trait impl ───────────────────────────────────────────────

#[async_trait]
impl KanbanProvider for GithubProjectsProvider {
    fn name(&self) -> &str {
        PROVIDER_NAME
    }

    fn is_configured(&self) -> bool {
        !self.token.is_empty()
    }

    async fn list_projects(&self) -> Result<Vec<ProjectInfo>, ApiError> {
        // Reuse the validate_detailed query — it's the canonical projects discovery.
        let details = self.validate_detailed().await?;
        Ok(details
            .projects
            .into_iter()
            .map(|p| ProjectInfo {
                id: p.node_id.clone(),
                key: p.node_id,
                name: format!("{}/#{} {}", p.owner_login, p.number, p.title),
            })
            .collect())
    }

    async fn get_issue_types(&self, project_key: &str) -> Result<Vec<ExternalIssueType>, ApiError> {
        // Resolve owner first; only orgs can have issueTypes.
        let (owner_login, owner_kind) = self.resolve_project_owner(project_key).await?;

        if owner_kind == "Organization" {
            if let Some(types) = self.fetch_org_issue_types(&owner_login).await? {
                return Ok(types);
            }
        }

        // Fallback: aggregate labels from items' linked repos.
        self.fetch_project_labels(project_key).await
    }

    async fn test_connection(&self) -> Result<bool, ApiError> {
        let query = r"
            query {
                viewer { login }
            }
        ";

        #[derive(Deserialize)]
        struct Resp {
            #[allow(dead_code)]
            viewer: ViewerNode,
        }

        #[derive(Deserialize)]
        struct ViewerNode {
            #[allow(dead_code)]
            login: String,
        }

        match self.graphql::<Resp>(query, None).await {
            Ok(_) => Ok(true),
            Err(e) if e.is_auth_error() => {
                warn!("GitHub authentication failed");
                Ok(false)
            }
            Err(e) => Err(e),
        }
    }

    async fn list_users(&self, project_key: &str) -> Result<Vec<ExternalUser>, ApiError> {
        // Derive users from the union of assignees seen across the project's items.
        let items = self.fetch_items_page(project_key, None).await?;
        let mut by_login: HashMap<String, ExternalUser> = HashMap::new();
        for item in items.nodes {
            let assignees_opt = match item.content {
                Some(RawContent::Issue { assignees, .. }) => assignees,
                Some(RawContent::PullRequest { assignees, .. }) => assignees,
                Some(RawContent::DraftIssue { assignees, .. }) => assignees,
                None => None,
            };
            if let Some(assignees) = assignees_opt {
                for a in assignees.nodes {
                    by_login.entry(a.login.clone()).or_insert(ExternalUser {
                        id: a
                            .database_id
                            .map(|n| n.to_string())
                            .unwrap_or(a.login.clone()),
                        name: a.name.unwrap_or_else(|| a.login.clone()),
                        email: a.email,
                        avatar_url: a.avatar_url,
                    });
                }
            }
        }
        Ok(by_login.into_values().collect())
    }

    async fn list_statuses(&self, project_key: &str) -> Result<Vec<String>, ApiError> {
        let cache = self.ensure_status_field(project_key).await?;
        Ok(cache.ordered_names)
    }

    async fn list_issues(
        &self,
        project_key: &str,
        user_id: &str,
        statuses: &[String],
    ) -> Result<Vec<ExternalIssue>, ApiError> {
        // Paginate through all items.
        let mut all_raw: Vec<RawProjectItem> = Vec::new();
        let mut cursor: Option<String> = None;
        loop {
            let page = self
                .fetch_items_page(project_key, cursor.as_deref())
                .await?;
            all_raw.extend(page.nodes);
            match page.page_info {
                Some(p) if p.has_next_page => {
                    cursor = p.end_cursor;
                    if cursor.is_none() {
                        break;
                    }
                }
                _ => break,
            }
        }

        let status_filter: Vec<String> = statuses.iter().map(|s| s.to_lowercase()).collect();
        let mut out: Vec<ExternalIssue> = Vec::new();
        let mut lookup_writes: Vec<(String, ItemLookup)> = Vec::new();

        for raw in all_raw {
            let item_id = raw.id.clone();
            let (status_name, _priority) = extract_status_and_priority(&raw.field_values);

            // Filter by status if requested.
            if !status_filter.is_empty() {
                let status_matches = status_name
                    .as_ref()
                    .map(|s| status_filter.contains(&s.to_lowercase()))
                    .unwrap_or(false);
                if !status_matches {
                    continue;
                }
            }

            let Some(content) = raw.content else {
                continue;
            };

            let assignees = content_assignees(&content);

            // Filter by user_id (matches against either the assignee's databaseId or login).
            let user_match = assignees.iter().any(|a| {
                a.database_id
                    .map(|n| n.to_string())
                    .as_deref()
                    .map(|s| s == user_id)
                    .unwrap_or(false)
                    || a.login == user_id
            });
            if !user_match {
                continue;
            }

            let assignee = assignees.first().map(|a| ExternalUser {
                id: a
                    .database_id
                    .map(|n| n.to_string())
                    .unwrap_or_else(|| a.login.clone()),
                name: a.name.clone().unwrap_or_else(|| a.login.clone()),
                email: a.email.clone(),
                avatar_url: a.avatar_url.clone(),
            });

            let (issue_id, key, summary, description, url, kanban_issue_types) = match content {
                RawContent::Issue {
                    id,
                    number,
                    title,
                    body,
                    url,
                    repository,
                    labels,
                    issue_type,
                    ..
                } => {
                    let repo = repository
                        .map(|r| r.name_with_owner)
                        .unwrap_or_else(|| "unknown/unknown".to_string());
                    let num = number.unwrap_or(0);
                    let key = format!("{repo}#{num}");
                    let kits = if let Some(it) = issue_type {
                        vec![KanbanIssueTypeRef {
                            id: it.id,
                            name: it.name,
                        }]
                    } else {
                        labels
                            .map(|l| {
                                l.nodes
                                    .into_iter()
                                    .map(|n| KanbanIssueTypeRef {
                                        id: n.id,
                                        name: n.name,
                                    })
                                    .collect()
                            })
                            .unwrap_or_default()
                    };
                    (
                        id.unwrap_or_else(|| key.clone()),
                        key,
                        title,
                        body,
                        url.unwrap_or_default(),
                        kits,
                    )
                }
                RawContent::PullRequest {
                    id,
                    number,
                    title,
                    body,
                    url,
                    repository,
                    labels,
                    ..
                } => {
                    let repo = repository
                        .map(|r| r.name_with_owner)
                        .unwrap_or_else(|| "unknown/unknown".to_string());
                    let num = number.unwrap_or(0);
                    let key = format!("{repo}!{num}");
                    let kits = labels
                        .map(|l| {
                            l.nodes
                                .into_iter()
                                .map(|n| KanbanIssueTypeRef {
                                    id: n.id,
                                    name: n.name,
                                })
                                .collect()
                        })
                        .unwrap_or_default();
                    (
                        id.unwrap_or_else(|| key.clone()),
                        key,
                        title,
                        body,
                        url.unwrap_or_default(),
                        kits,
                    )
                }
                RawContent::DraftIssue {
                    id, title, body, ..
                } => {
                    let key = format!("draft:{item_id}");
                    (
                        id.unwrap_or_else(|| key.clone()),
                        key,
                        title,
                        body,
                        String::new(),
                        Vec::new(),
                    )
                }
            };

            // Cache the lookup so update_issue_status can resolve this key later.
            lookup_writes.push((
                key.clone(),
                ItemLookup {
                    project_id: project_key.to_string(),
                    item_id: item_id.clone(),
                },
            ));

            out.push(ExternalIssue {
                id: issue_id,
                key,
                summary,
                description,
                kanban_issue_types,
                status: status_name.unwrap_or_default(),
                assignee,
                url,
                priority: None, // TODO: extract from a Priority single-select field if present
            });
        }

        // Persist lookups for update_issue_status.
        if !lookup_writes.is_empty() {
            let mut guard = self.item_lookup.write().await;
            for (key, lookup) in lookup_writes {
                guard.insert(key, lookup);
            }
        }

        Ok(out)
    }

    async fn create_issue(
        &self,
        project_key: &str,
        request: CreateIssueRequest,
    ) -> Result<CreateIssueResponse, ApiError> {
        // v1: draft issues only. Real repo issues are out of scope per plan.
        let mutation = r"
            mutation($input: AddProjectV2DraftIssueInput!) {
                addProjectV2DraftIssue(input: $input) {
                    projectItem {
                        id
                        content {
                            __typename
                            ... on DraftIssue {
                                id
                                title
                                body
                            }
                        }
                    }
                }
            }
        ";

        let mut input = serde_json::json!({
            "projectId": project_key,
            "title": request.summary,
        });
        if let Some(body) = request.description {
            input["body"] = serde_json::json!(body);
        }
        if let Some(assignee) = request.assignee_id {
            input["assigneeIds"] = serde_json::json!([assignee]);
        }

        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Resp {
            add_project_v2_draft_issue: AddResp,
        }

        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct AddResp {
            project_item: ProjectItem,
        }

        #[derive(Deserialize)]
        struct ProjectItem {
            id: String,
            #[serde(default)]
            content: Option<DraftContent>,
        }

        #[derive(Deserialize)]
        #[serde(tag = "__typename")]
        enum DraftContent {
            DraftIssue {
                #[serde(default)]
                id: Option<String>,
                title: String,
                #[serde(default)]
                body: Option<String>,
            },
        }

        let resp: Resp = self
            .graphql(mutation, Some(serde_json::json!({ "input": input })))
            .await?;

        let item = resp.add_project_v2_draft_issue.project_item;
        let item_id = item.id;
        let key = format!("draft:{item_id}");

        let (issue_id, summary, description) = match item.content {
            Some(DraftContent::DraftIssue { id, title, body }) => {
                (id.unwrap_or_else(|| item_id.clone()), title, body)
            }
            None => (item_id.clone(), request.summary.clone(), None),
        };

        // Cache the lookup so a follow-up update_issue_status works without
        // a list_issues call.
        self.item_lookup.write().await.insert(
            key.clone(),
            ItemLookup {
                project_id: project_key.to_string(),
                item_id,
            },
        );

        Ok(CreateIssueResponse {
            issue: ExternalIssue {
                id: issue_id,
                key,
                summary,
                description,
                kanban_issue_types: Vec::new(),
                status: String::new(),
                assignee: None,
                url: String::new(),
                priority: None,
            },
        })
    }

    async fn update_issue_status(
        &self,
        issue_key: &str,
        request: UpdateStatusRequest,
    ) -> Result<ExternalIssue, ApiError> {
        // Resolve the (project_id, item_id) pair from the lookup cache.
        let lookup = self
            .item_lookup
            .read()
            .await
            .get(issue_key)
            .cloned()
            .ok_or_else(|| {
                ApiError::http(
                    PROVIDER_NAME,
                    400,
                    format!(
                        "GitHub Projects update_issue_status: no cached lookup for '{issue_key}'. \
                         Call list_issues() (or create_issue()) first so the provider can map the \
                         key back to its project + item ids."
                    ),
                )
            })?;

        let cache = self.ensure_status_field(&lookup.project_id).await?;
        let option_id = cache
            .options_by_name
            .get(&request.status.to_lowercase())
            .cloned()
            .ok_or_else(|| {
                ApiError::http(
                    PROVIDER_NAME,
                    400,
                    format!(
                        "Status '{}' not found in project. Available: {}",
                        request.status,
                        cache.ordered_names.join(", ")
                    ),
                )
            })?;

        let mutation = r"
            mutation($projectId: ID!, $itemId: ID!, $fieldId: ID!, $optionId: String!) {
                updateProjectV2ItemFieldValue(input: {
                    projectId: $projectId,
                    itemId: $itemId,
                    fieldId: $fieldId,
                    value: { singleSelectOptionId: $optionId }
                }) {
                    projectV2Item { id }
                }
            }
        ";

        let variables = serde_json::json!({
            "projectId": lookup.project_id,
            "itemId": lookup.item_id,
            "fieldId": cache.field_id,
            "optionId": option_id,
        });

        // Discard the response — we only care that it didn't error.
        let _: serde_json::Value = self.graphql(mutation, Some(variables)).await?;

        // Return a minimal updated ExternalIssue. Re-fetching the full item
        // would be a second round-trip; the caller already has the rest from
        // its previous list_issues call.
        Ok(ExternalIssue {
            id: lookup.item_id,
            key: issue_key.to_string(),
            summary: String::new(),
            description: None,
            kanban_issue_types: Vec::new(),
            status: request.status,
            assignee: None,
            url: String::new(),
            priority: None,
        })
    }

    async fn update_issue_labels(
        &self,
        issue_key: &str,
        labels: &[String],
    ) -> Result<(), ApiError> {
        if labels.is_empty() {
            return Ok(());
        }

        if issue_key.starts_with("draft:") {
            // For draft items, append labels as metadata text to the body.
            let item_id = issue_key.trim_start_matches("draft:");

            // Query the draft issue: get its internal id + current body
            let query = r"
                query($nodeId: ID!) {
                    node(id: $nodeId) {
                        ... on ProjectV2Item {
                            content {
                                __typename
                                ... on DraftIssue {
                                    id
                                    body
                                }
                            }
                        }
                    }
                }
            ";
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct NodeResp {
                node: NodeContent,
            }
            #[derive(Deserialize)]
            struct NodeContent {
                content: Option<DraftContent>,
            }
            #[allow(dead_code)]
            #[derive(Deserialize)]
            #[serde(tag = "__typename")]
            enum DraftContent {
                DraftIssue {
                    id: String,
                    #[serde(default)]
                    body: Option<String>,
                },
            }

            let vars = serde_json::json!({ "nodeId": item_id });
            let resp: NodeResp = self.graphql(query, Some(vars)).await?;

            let (draft_id, current_body) = match resp.node.content {
                Some(DraftContent::DraftIssue { id, body }) => (id, body.unwrap_or_default()),
                None => return Ok(()), // Not a draft issue item
            };

            let label_line = format!("\n**Labels:** {}", labels.join(", "));
            let new_body = format!("{current_body}{label_line}");

            let mutation = r"
                mutation($input: UpdateProjectV2DraftIssueInput!) {
                    updateProjectV2DraftIssue(input: $input) {
                        draftIssue { id }
                    }
                }
            ";
            let _: serde_json::Value = self
                .graphql(
                    mutation,
                    Some(serde_json::json!({
                        "input": { "draftIssueId": draft_id, "body": new_body }
                    })),
                )
                .await?;
        } else if let Some((owner_repo, number_str)) = issue_key.split_once('#') {
            // Real repo issue: addLabelsToLabelable
            let (owner, repo) = owner_repo.split_once('/').ok_or_else(|| {
                ApiError::http(
                    PROVIDER_NAME,
                    400,
                    format!("Invalid issue key: {issue_key}"),
                )
            })?;

            // Get the issue node ID
            let id_query = r"
                query($owner: String!, $repo: String!, $number: Int!) {
                    repository(owner: $owner, name: $repo) {
                        issue(number: $number) { id }
                    }
                }
            ";
            let number: i64 = number_str.parse().map_err(|_| {
                ApiError::http(
                    PROVIDER_NAME,
                    400,
                    format!("Invalid issue number in key: {issue_key}"),
                )
            })?;
            #[derive(Deserialize)]
            struct IdResp {
                repository: RepoWithIssue,
            }
            #[derive(Deserialize)]
            struct RepoWithIssue {
                issue: IssueNode,
            }
            #[derive(Deserialize)]
            struct IssueNode {
                id: String,
            }
            let vars = serde_json::json!({ "owner": owner, "repo": repo, "number": number });
            let id_resp: IdResp = self.graphql(id_query, Some(vars)).await?;
            let issue_node_id = id_resp.repository.issue.id;

            // Find or create each label on the repo, then add all to the issue
            let mut label_ids: Vec<String> = Vec::new();
            for label_name in labels {
                // Query for existing repo label by name
                let label_query = r"
                    query($owner: String!, $repo: String!, $name: String!) {
                        repository(owner: $owner, name: $repo) {
                            label(name: $name) { id }
                        }
                    }
                ";
                #[derive(Deserialize)]
                struct LabelResp {
                    repository: RepoWithLabel,
                }
                #[derive(Deserialize)]
                struct RepoWithLabel {
                    label: Option<LabelId>,
                }
                #[derive(Deserialize)]
                struct LabelId {
                    id: String,
                }
                let label_vars =
                    serde_json::json!({ "owner": owner, "repo": repo, "name": label_name });
                let label_resp: LabelResp = self.graphql(label_query, Some(label_vars)).await?;

                let label_id = if let Some(existing) = label_resp.repository.label {
                    existing.id
                } else {
                    // Create the label
                    let create_mutation = r"
                        mutation($repoId: ID!, $name: String!, $color: String!) {
                            createLabel(input: { repositoryId: $repoId, name: $name, color: $color }) {
                                label { id }
                            }
                        }
                    ";
                    // Get repo node ID first
                    let repo_id_query = r"
                        query($owner: String!, $repo: String!) {
                            repository(owner: $owner, name: $repo) { id }
                        }
                    ";
                    #[derive(Deserialize)]
                    struct RepoIdResp {
                        repository: RepoId,
                    }
                    #[derive(Deserialize)]
                    struct RepoId {
                        id: String,
                    }
                    let repo_id_vars = serde_json::json!({ "owner": owner, "repo": repo });
                    let repo_id_resp: RepoIdResp =
                        self.graphql(repo_id_query, Some(repo_id_vars)).await?;

                    #[derive(Deserialize)]
                    struct CreateLabelResp {
                        #[serde(rename = "createLabel")]
                        create_label: CreateLabelPayload,
                    }
                    #[derive(Deserialize)]
                    struct CreateLabelPayload {
                        label: LabelId,
                    }
                    let create_vars = serde_json::json!({
                        "repoId": repo_id_resp.repository.id,
                        "name": label_name,
                        "color": "ededed"  // default gray
                    });
                    let create_resp: CreateLabelResp =
                        self.graphql(create_mutation, Some(create_vars)).await?;
                    create_resp.create_label.label.id
                };
                label_ids.push(label_id);
            }

            let add_mutation = r"
                mutation($labelableId: ID!, $labelIds: [ID!]!) {
                    addLabelsToLabelable(input: { labelableId: $labelableId, labelIds: $labelIds }) {
                        labelable { ... on Issue { id } }
                    }
                }
            ";
            let _: serde_json::Value = self
                .graphql(
                    add_mutation,
                    Some(serde_json::json!({
                        "labelableId": issue_node_id,
                        "labelIds": label_ids
                    })),
                )
                .await?;
        }

        Ok(())
    }

    async fn append_activity_log(
        &self,
        issue_key: &str,
        entry: &super::ActivityLogEntry,
    ) -> Result<(), ApiError> {
        let timestamp = entry.completed_at.format("%Y-%m-%d %H:%M UTC").to_string();
        let summary_text = entry.summary.as_deref().unwrap_or("");

        if issue_key.starts_with("draft:") {
            let item_id = issue_key.trim_start_matches("draft:");

            // Query draft issue id + current body
            let query = r"
                query($nodeId: ID!) {
                    node(id: $nodeId) {
                        ... on ProjectV2Item {
                            content {
                                __typename
                                ... on DraftIssue {
                                    id
                                    body
                                }
                            }
                        }
                    }
                }
            ";
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct NodeResp {
                node: NodeContent,
            }
            #[derive(Deserialize)]
            struct NodeContent {
                content: Option<DraftContent>,
            }
            #[allow(dead_code)]
            #[derive(Deserialize)]
            #[serde(tag = "__typename")]
            enum DraftContent {
                DraftIssue {
                    id: String,
                    #[serde(default)]
                    body: Option<String>,
                },
            }

            let vars = serde_json::json!({ "nodeId": item_id });
            let resp: NodeResp = self.graphql(query, Some(vars)).await?;

            let (draft_id, current_body) = match resp.node.content {
                Some(DraftContent::DraftIssue { id, body }) => (id, body.unwrap_or_default()),
                None => return Ok(()),
            };

            let log_line = if summary_text.is_empty() {
                format!(
                    "\n\n---\n**Agent Activity** (opr8r)\n| Step | Delegator | Completed |\n|------|-----------|----------|\n| {} | {} | {} |",
                    entry.step, entry.delegator, timestamp
                )
            } else {
                format!(
                    "\n\n---\n**Agent Activity** (opr8r)\n| Step | Delegator | Completed | Summary |\n|------|-----------|----------|---------|\n| {} | {} | {} | {} |",
                    entry.step, entry.delegator, timestamp, summary_text
                )
            };

            // If there's already an Agent Activity section, append a new table row instead
            let new_body = if current_body.contains("**Agent Activity** (opr8r)") {
                // Append another row to the existing table
                let new_row = if summary_text.is_empty() {
                    format!("\n| {} | {} | {} |", entry.step, entry.delegator, timestamp)
                } else {
                    format!(
                        "\n| {} | {} | {} | {} |",
                        entry.step, entry.delegator, timestamp, summary_text
                    )
                };
                format!("{current_body}{new_row}")
            } else {
                format!("{current_body}{log_line}")
            };

            let mutation = r"
                mutation($input: UpdateProjectV2DraftIssueInput!) {
                    updateProjectV2DraftIssue(input: $input) {
                        draftIssue { id }
                    }
                }
            ";
            let _: serde_json::Value = self
                .graphql(
                    mutation,
                    Some(serde_json::json!({
                        "input": { "draftIssueId": draft_id, "body": new_body }
                    })),
                )
                .await?;
        } else if let Some((owner_repo, number_str)) = issue_key.split_once('#') {
            // Real repo issue: add a comment
            let (owner, repo) = owner_repo.split_once('/').ok_or_else(|| {
                ApiError::http(
                    PROVIDER_NAME,
                    400,
                    format!("Invalid issue key: {issue_key}"),
                )
            })?;
            let number: i64 = number_str.parse().map_err(|_| {
                ApiError::http(
                    PROVIDER_NAME,
                    400,
                    format!("Invalid issue number: {issue_key}"),
                )
            })?;

            // Get issue node ID
            let id_query = r"
                query($owner: String!, $repo: String!, $number: Int!) {
                    repository(owner: $owner, name: $repo) {
                        issue(number: $number) { id }
                    }
                }
            ";
            #[derive(Deserialize)]
            struct IdResp {
                repository: RepoWithIssue,
            }
            #[derive(Deserialize)]
            struct RepoWithIssue {
                issue: IssueNode,
            }
            #[derive(Deserialize)]
            struct IssueNode {
                id: String,
            }
            let vars = serde_json::json!({ "owner": owner, "repo": repo, "number": number });
            let id_resp: IdResp = self.graphql(id_query, Some(vars)).await?;

            let comment_body = if summary_text.is_empty() {
                format!(
                    "🤖 **opr8r activity** — step: `{}` | delegator: `{}` | {}",
                    entry.step, entry.delegator, timestamp
                )
            } else {
                format!(
                    "🤖 **opr8r activity** — step: `{}` | delegator: `{}` | {}\n\n> {}",
                    entry.step, entry.delegator, timestamp, summary_text
                )
            };

            let add_comment = r"
                mutation($subjectId: ID!, $body: String!) {
                    addComment(input: { subjectId: $subjectId, body: $body }) {
                        commentEdge { node { id } }
                    }
                }
            ";
            let _: serde_json::Value = self
                .graphql(
                    add_comment,
                    Some(serde_json::json!({
                        "subjectId": id_resp.repository.issue.id,
                        "body": comment_body
                    })),
                )
                .await?;
        }

        Ok(())
    }
}

impl GithubProjectsProvider {
    /// Fetch a single page of project items. Helper used by both
    /// `list_issues` and `list_users`.
    async fn fetch_items_page(
        &self,
        project_id: &str,
        after: Option<&str>,
    ) -> Result<ItemsPage, ApiError> {
        // NOTE: assignees.nodes must NOT request `email` — GitHub gates the
        // `User.email` field behind `user:email` or `read:user` scope, which
        // is orthogonal to the `project` scope this provider requires and
        // would break any token scoped to projects-only. `RawAssignee.email`
        // stays in the struct (serde-default `None`) for forward compat.
        let query = r"
            query($projectId: ID!, $first: Int!, $after: String) {
                node(id: $projectId) {
                    ... on ProjectV2 {
                        items(first: $first, after: $after) {
                            pageInfo { hasNextPage endCursor }
                            nodes {
                                id
                                type
                                content {
                                    __typename
                                    ... on Issue {
                                        id
                                        number
                                        title
                                        body
                                        url
                                        repository { nameWithOwner }
                                        assignees(first: 10) {
                                            nodes {
                                                login
                                                databaseId
                                                name
                                                avatarUrl
                                            }
                                        }
                                        labels(first: 20) { nodes { id name } }
                                        issueType { id name }
                                    }
                                    ... on PullRequest {
                                        id
                                        number
                                        title
                                        body
                                        url
                                        repository { nameWithOwner }
                                        assignees(first: 10) {
                                            nodes {
                                                login
                                                databaseId
                                                name
                                                avatarUrl
                                            }
                                        }
                                        labels(first: 20) { nodes { id name } }
                                    }
                                    ... on DraftIssue {
                                        id
                                        title
                                        body
                                        assignees(first: 10) {
                                            nodes {
                                                login
                                                databaseId
                                                name
                                                avatarUrl
                                            }
                                        }
                                    }
                                }
                                fieldValues(first: 20) {
                                    nodes {
                                        __typename
                                        ... on ProjectV2ItemFieldSingleSelectValue {
                                            name
                                            field {
                                                __typename
                                                ... on ProjectV2SingleSelectField { name }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        ";

        let variables = serde_json::json!({
            "projectId": project_id,
            "first": 100,
            "after": after,
        });

        let resp: ListItemsResponse = self.graphql(query, Some(variables)).await?;
        Ok(resp.node.items.unwrap_or(ItemsPage {
            page_info: None,
            nodes: Vec::new(),
        }))
    }
}

// ─── Helper functions ────────────────────────────────────────────────────────

/// Extract the Status field value (and a placeholder for Priority) from
/// an item's `fieldValues` connection.
fn extract_status_and_priority(
    field_values: &Option<RawFieldValues>,
) -> (Option<String>, Option<String>) {
    let Some(values) = field_values else {
        return (None, None);
    };
    let mut status: Option<String> = None;
    for v in &values.nodes {
        if let RawFieldValue::ProjectV2ItemFieldSingleSelectValue { name, field } = v {
            let field_name = match field {
                Some(RawFieldRef::ProjectV2SingleSelectField { name }) => name.clone(),
                _ => None,
            };
            if let Some(fname) = field_name {
                if fname.eq_ignore_ascii_case("Status") && status.is_none() {
                    status.clone_from(name);
                }
            }
        }
    }
    (status, None)
}

/// Extract assignees from a content variant. Returns an empty slice for None.
fn content_assignees(content: &RawContent) -> &[RawAssignee] {
    let assignees = match content {
        RawContent::Issue { assignees, .. } => assignees.as_ref(),
        RawContent::PullRequest { assignees, .. } => assignees.as_ref(),
        RawContent::DraftIssue { assignees, .. } => assignees.as_ref(),
    };
    assignees.map(|a| a.nodes.as_slice()).unwrap_or(&[])
}

#[async_trait]
impl super::onboarding::KanbanOnboarding for GithubProjectsProvider {
    fn provider_kind(&self) -> super::KanbanProviderType {
        super::KanbanProviderType::Github
    }

    async fn validate_onboarding(&self) -> Result<super::onboarding::ValidatedWorkspace, ApiError> {
        let details = self.validate_detailed().await?;
        let prefetched: Vec<super::onboarding::DiscoveredProject> = details
            .projects
            .iter()
            .map(|p| super::onboarding::DiscoveredProject {
                workspace_key: details.user_login.clone(),
                project_key: p.node_id.clone(),
                project_display_name: format!("{}/{} ({})", p.owner_login, p.title, p.number),
                provider_url: None,
                provider_native_id: Some(p.node_id.clone()),
            })
            .collect();
        Ok(super::onboarding::ValidatedWorkspace {
            provider_kind: super::KanbanProviderType::Github,
            workspace_key: details.user_login,
            workspace_display_name: "github.com".to_string(),
            sync_user_id: details.user_id,
            sync_user_display_name: String::new(),
            api_key_env: details.resolved_env_var,
            prefetched_projects: Some(prefetched),
            extra: super::onboarding::WorkspaceExtra::Github,
        })
    }

    async fn discover_projects(
        &self,
        workspace: &super::onboarding::ValidatedWorkspace,
    ) -> Result<Vec<super::onboarding::DiscoveredProject>, ApiError> {
        Ok(workspace.prefetched_projects.clone().unwrap_or_default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_env_not_configured() {
        env::remove_var("OPERATOR_GITHUB_TOKEN");
        let result = GithubProjectsProvider::from_env();
        assert!(result.is_err());
    }

    #[test]
    fn test_from_env_does_not_fall_back_to_github_token() {
        // Token Disambiguation rule 1: must NOT use GITHUB_TOKEN.
        env::remove_var("OPERATOR_GITHUB_TOKEN");
        env::set_var("GITHUB_TOKEN", "ghp_should_not_be_used");
        let result = GithubProjectsProvider::from_env();
        assert!(
            result.is_err(),
            "from_env must not fall back to GITHUB_TOKEN — see Token Disambiguation rule 1"
        );
        env::remove_var("GITHUB_TOKEN");
    }

    #[test]
    fn test_deserialize_items_page_with_issue() {
        let json = r#"{
            "node": {
                "items": {
                    "pageInfo": { "hasNextPage": false, "endCursor": null },
                    "nodes": [
                        {
                            "id": "PVTI_lAHO_test",
                            "type": "ISSUE",
                            "content": {
                                "__typename": "Issue",
                                "id": "I_kwDO_test",
                                "number": 42,
                                "title": "Fix login bug",
                                "body": "Users cannot log in",
                                "url": "https://github.com/octocat/hello/issues/42",
                                "repository": { "nameWithOwner": "octocat/hello" },
                                "assignees": {
                                    "nodes": [
                                        {
                                            "login": "octocat",
                                            "databaseId": 583231,
                                            "name": "The Octocat",
                                            "email": null,
                                            "avatarUrl": "https://github.com/octocat.png"
                                        }
                                    ]
                                },
                                "labels": {
                                    "nodes": [
                                        { "id": "L_bug", "name": "bug" }
                                    ]
                                },
                                "issueType": null
                            },
                            "fieldValues": {
                                "nodes": [
                                    {
                                        "__typename": "ProjectV2ItemFieldSingleSelectValue",
                                        "name": "In Progress",
                                        "field": {
                                            "__typename": "ProjectV2SingleSelectField",
                                            "name": "Status"
                                        }
                                    }
                                ]
                            }
                        }
                    ]
                }
            }
        }"#;

        let resp: ListItemsResponse = serde_json::from_str(json).unwrap();
        let page = resp.node.items.unwrap();
        assert_eq!(page.nodes.len(), 1);
        let item = &page.nodes[0];
        assert_eq!(item.id, "PVTI_lAHO_test");

        let (status, _) = extract_status_and_priority(&item.field_values);
        assert_eq!(status.as_deref(), Some("In Progress"));
    }

    #[test]
    fn test_deserialize_items_page_with_draft() {
        let json = r#"{
            "node": {
                "items": {
                    "pageInfo": { "hasNextPage": false, "endCursor": null },
                    "nodes": [
                        {
                            "id": "PVTI_lAHO_draft",
                            "type": "DRAFT_ISSUE",
                            "content": {
                                "__typename": "DraftIssue",
                                "id": "DI_lAHO_test",
                                "title": "A draft idea",
                                "body": "needs fleshing out",
                                "assignees": { "nodes": [] }
                            },
                            "fieldValues": { "nodes": [] }
                        }
                    ]
                }
            }
        }"#;

        let resp: ListItemsResponse = serde_json::from_str(json).unwrap();
        let page = resp.node.items.unwrap();
        assert_eq!(page.nodes.len(), 1);
        let item = &page.nodes[0];
        match &item.content {
            Some(RawContent::DraftIssue { title, .. }) => {
                assert_eq!(title, "A draft idea");
            }
            _ => panic!("Expected DraftIssue variant"),
        }
    }

    #[test]
    fn test_extract_status_and_priority_no_field_values() {
        let (status, priority) = extract_status_and_priority(&None);
        assert!(status.is_none());
        assert!(priority.is_none());
    }

    #[test]
    fn test_extract_status_picks_status_field_only() {
        let json = r#"{
            "nodes": [
                {
                    "__typename": "ProjectV2ItemFieldSingleSelectValue",
                    "name": "P1",
                    "field": {
                        "__typename": "ProjectV2SingleSelectField",
                        "name": "Priority"
                    }
                },
                {
                    "__typename": "ProjectV2ItemFieldSingleSelectValue",
                    "name": "Done",
                    "field": {
                        "__typename": "ProjectV2SingleSelectField",
                        "name": "Status"
                    }
                }
            ]
        }"#;
        let fv: RawFieldValues = serde_json::from_str(json).unwrap();
        let (status, _) = extract_status_and_priority(&Some(fv));
        assert_eq!(status.as_deref(), Some("Done"));
    }
}

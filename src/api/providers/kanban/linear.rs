//! Linear kanban provider implementation

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use tracing::{debug, warn};

use super::{ExternalIssue, ExternalIssueType, ExternalUser, KanbanProvider, ProjectInfo};
use crate::api::error::ApiError;
use crate::issuetypes::kanban_type::KanbanIssueTypeRef;

const LINEAR_API_URL: &str = "https://api.linear.app/graphql";
const PROVIDER_NAME: &str = "linear";

/// Info about a Linear team (returned by `validate_detailed`).
#[derive(Debug, Clone)]
pub struct LinearTeamInfo {
    pub id: String,
    pub key: String,
    pub name: String,
}

/// Detailed validation result for Linear onboarding.
///
/// Richer than `KanbanProvider::test_connection` - includes viewer, org, and
/// the full list of teams available to the API key in a single round-trip.
#[derive(Debug, Clone)]
pub struct LinearValidationDetails {
    pub user_id: String,
    pub user_name: String,
    pub org_name: String,
    pub url_key: String,
    pub teams: Vec<LinearTeamInfo>,
}

/// Linear API provider
pub struct LinearProvider {
    api_key: String,
    client: Client,
}

impl LinearProvider {
    /// Create a new Linear provider
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: crate::http_client::default_client(),
        }
    }

    /// Create from environment variables
    ///
    /// Required environment variables:
    /// - `OPERATOR_LINEAR_API_KEY`: Your Linear API key (`lin_api`_...)
    pub fn from_env() -> Result<Self, ApiError> {
        match env::var("OPERATOR_LINEAR_API_KEY") {
            Ok(key) if !key.is_empty() => Ok(Self::new(key)),
            _ => Err(ApiError::not_configured(PROVIDER_NAME)),
        }
    }

    /// Create from config with workspace as key
    ///
    /// The workspace slug is passed for identification (it's the `HashMap` key in `KanbanConfig`).
    /// The `api_key` is read from the environment variable specified in `config.api_key_env`.
    pub fn from_config(
        _workspace: &str,
        config: &crate::config::LinearConfig,
    ) -> Result<Self, ApiError> {
        let api_key = env::var(&config.api_key_env).ok();

        match api_key {
            Some(key) if !key.is_empty() => Ok(Self::new(key)),
            _ => Err(ApiError::not_configured(PROVIDER_NAME)),
        }
    }

    /// Execute a `GraphQL` query
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
        }

        let request = GraphQLRequest { query, variables };

        debug!("Linear GraphQL query: {}", query);

        let response = self
            .client
            .post(LINEAR_API_URL)
            .header("Authorization", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| ApiError::network(PROVIDER_NAME, e.to_string()))?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return match status.as_u16() {
                401 => Err(ApiError::unauthorized(PROVIDER_NAME)),
                403 => Err(ApiError::forbidden(PROVIDER_NAME)),
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
            return Err(ApiError::http(PROVIDER_NAME, 0, messages.join("; ")));
        }

        gql_response
            .data
            .ok_or_else(|| ApiError::http(PROVIDER_NAME, 0, "No data in response".to_string()))
    }

    /// Get the internal UUID for an issue from its identifier (e.g., "ENG-123")
    async fn get_issue_info(&self, identifier: &str) -> Result<(String, String), ApiError> {
        let query = r"
            query($identifier: String!) {
                issue(id: $identifier) {
                    id
                    team {
                        id
                    }
                }
            }
        ";

        let variables = serde_json::json!({
            "identifier": identifier
        });

        let response: IssueQueryResponse = self.graphql(query, Some(variables)).await?;
        Ok((response.issue.id, response.issue.team.id))
    }

    /// Find the state ID for a given status name in a team
    async fn find_state_id(&self, team_id: &str, status_name: &str) -> Result<String, ApiError> {
        let query = r"
            query($teamId: String!) {
                team(id: $teamId) {
                    states {
                        nodes {
                            id
                            name
                        }
                    }
                }
            }
        ";

        let variables = serde_json::json!({
            "teamId": team_id
        });

        let response: TeamStatesWithIdResponse = self.graphql(query, Some(variables)).await?;

        response
            .team
            .states
            .nodes
            .iter()
            .find(|s| s.name.eq_ignore_ascii_case(status_name))
            .map(|s| s.id.clone())
            .ok_or_else(|| {
                ApiError::http(
                    PROVIDER_NAME,
                    400,
                    format!(
                        "Status '{}' not found. Available: {}",
                        status_name,
                        response
                            .team
                            .states
                            .nodes
                            .iter()
                            .map(|s| s.name.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                    ),
                )
            })
    }

    /// Detailed credential validation for onboarding.
    ///
    /// A single `GraphQL` query returns viewer (user), organization, and the
    /// full list of teams accessible to the API key. The onboarding flow
    /// uses the `viewer.id` as `sync_user_id` and shows the team list in a
    /// picker.
    pub async fn validate_detailed(&self) -> Result<LinearValidationDetails, ApiError> {
        let query = r"
            query {
                viewer { id name }
                organization { name urlKey }
                teams { nodes { id key name } }
            }
        ";

        #[derive(Deserialize)]
        struct ValidateResponse {
            viewer: ViewerNode,
            organization: OrgNode,
            teams: TeamsNodesPayload,
        }

        #[derive(Deserialize)]
        struct ViewerNode {
            id: String,
            #[serde(default)]
            name: String,
        }

        #[derive(Deserialize)]
        struct OrgNode {
            #[serde(default)]
            name: String,
            #[serde(default, rename = "urlKey")]
            url_key: String,
        }

        #[derive(Deserialize)]
        struct TeamsNodesPayload {
            nodes: Vec<TeamNodePayload>,
        }

        #[derive(Deserialize)]
        struct TeamNodePayload {
            id: String,
            key: String,
            name: String,
        }

        let resp: ValidateResponse = self.graphql(query, None).await?;
        Ok(LinearValidationDetails {
            user_id: resp.viewer.id,
            user_name: resp.viewer.name,
            org_name: resp.organization.name,
            url_key: resp.organization.url_key,
            teams: resp
                .teams
                .nodes
                .into_iter()
                .map(|t| LinearTeamInfo {
                    id: t.id,
                    key: t.key,
                    name: t.name,
                })
                .collect(),
        })
    }
}

// Linear GraphQL response types
#[derive(Debug, Deserialize)]
struct TeamsResponse {
    teams: TeamsNodes,
}

#[derive(Debug, Deserialize)]
struct TeamsNodes {
    nodes: Vec<LinearTeam>,
}

#[derive(Debug, Deserialize)]
struct LinearTeam {
    id: String,
    key: String,
    name: String,
}

#[derive(Debug, Deserialize)]
struct LabelsResponse {
    team: TeamWithLabels,
}

#[derive(Debug, Deserialize)]
struct TeamWithLabels {
    labels: LabelsNodes,
}

#[derive(Debug, Deserialize)]
struct LabelsNodes {
    nodes: Vec<LinearLabel>,
}

#[derive(Debug, Deserialize)]
struct LinearLabel {
    id: String,
    name: String,
    description: Option<String>,
    #[allow(dead_code)]
    color: Option<String>,
}

// ─── Team Members Response ───────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct TeamMembersResponse {
    team: TeamWithMembers,
}

#[derive(Debug, Deserialize)]
struct TeamWithMembers {
    members: MembersConnection,
}

#[derive(Debug, Deserialize)]
struct MembersConnection {
    nodes: Vec<LinearUser>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LinearUser {
    id: String,
    name: String,
    email: Option<String>,
    avatar_url: Option<String>,
}

// ─── Team States (Workflow Statuses) Response ────────────────────────────────

#[derive(Debug, Deserialize)]
struct TeamStatesResponse {
    team: TeamWithStates,
}

#[derive(Debug, Deserialize)]
struct TeamWithStates {
    states: StatesConnection,
}

#[derive(Debug, Deserialize)]
struct StatesConnection {
    nodes: Vec<LinearState>,
}

#[derive(Debug, Deserialize)]
struct LinearState {
    name: String,
    position: f64,
}

// ─── Issues Query Response ───────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct IssuesResponse {
    issues: IssuesConnection,
}

#[derive(Debug, Deserialize)]
struct IssuesConnection {
    nodes: Vec<LinearIssue>,
    #[serde(default, rename = "pageInfo")]
    page_info: Option<PageInfo>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PageInfo {
    has_next_page: bool,
    #[serde(default)]
    end_cursor: Option<String>,
}

/// A project and one page of its issues.
#[derive(Debug, Deserialize)]
struct ProjectScopeResponse {
    project: LinearProjectHeader,
    issues: IssuesConnection,
}

#[derive(Debug, Deserialize)]
struct LinearProjectHeader {
    name: String,
    #[serde(default)]
    description: Option<String>,
    url: String,
}

/// Pages fetched per scoped query before giving up (100 issues each).
const MAX_SCOPE_PAGES: usize = 50;

/// A project and one page of all its issues (any assignee), optionally filtered by state.
fn project_scope_query(with_states: bool) -> String {
    let (state_var, state_filter) = if with_states {
        (
            ", $stateNames: [String!]!",
            "state: { name: { in: $stateNames } }",
        )
    } else {
        ("", "")
    };
    format!(
        "query($projectId: String!, $projectFilterId: ID!, $after: String{state_var}) {{
            project(id: $projectId) {{ name description url }}
            issues(
                filter: {{ project: {{ id: {{ eq: $projectFilterId }} }} {state_filter} }}
                first: 100
                after: $after
            ) {{
                pageInfo {{ hasNextPage endCursor }}
                nodes {{ ...SyncedIssue }}
            }}
        }}
        {SYNCED_ISSUE_FRAGMENT}"
    )
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LinearIssue {
    id: String,
    identifier: String,
    title: String,
    description: Option<String>,
    state: LinearStateRef,
    assignee: Option<LinearUser>,
    priority: i32,
    url: String,
    #[serde(default)]
    labels: Option<LabelsNodes>,
    #[serde(default)]
    inverse_relations: Option<RelationNodes>,
    #[serde(default)]
    parent: Option<IdentifierRef>,
    #[serde(default)]
    children: Option<IdentifierNodes>,
    #[serde(default)]
    project: Option<LinearProjectRef>,
}

#[derive(Debug, Deserialize)]
struct RelationNodes {
    nodes: Vec<LinearRelation>,
}

/// An inverse relation: `issue` relates to this one with `type`.
#[derive(Debug, Deserialize)]
struct LinearRelation {
    #[serde(rename = "type")]
    relation_type: String,
    issue: IdentifierRef,
}

#[derive(Debug, Deserialize)]
struct IdentifierRef {
    identifier: String,
}

#[derive(Debug, Deserialize)]
struct IdentifierNodes {
    nodes: Vec<IdentifierRef>,
}

#[derive(Debug, Deserialize)]
struct LinearProjectRef {
    id: String,
}

/// Issue fields read by sync, shared by every `issues` query.
const SYNCED_ISSUE_FRAGMENT: &str = r"
    fragment SyncedIssue on Issue {
        id
        identifier
        title
        description
        state { name }
        assignee { id name email avatarUrl }
        priority
        url
        labels { nodes { id name } }
        inverseRelations { nodes { type issue { identifier } } }
        parent { identifier }
        children { nodes { identifier } }
        project { id name }
    }
";

/// Normalize a Linear issue: a project outranks a parent issue as the container.
fn issue_from_linear(issue: LinearIssue) -> ExternalIssue {
    let kanban_issue_types = issue
        .labels
        .map(|labels| {
            labels
                .nodes
                .into_iter()
                .map(|l| KanbanIssueTypeRef {
                    id: l.id,
                    name: l.name,
                })
                .collect()
        })
        .unwrap_or_default();
    let blocked_by = issue
        .inverse_relations
        .map(|r| {
            r.nodes
                .into_iter()
                .filter(|rel| rel.relation_type == "blocks")
                .map(|rel| rel.issue.identifier)
                .collect()
        })
        .unwrap_or_default();
    let parent = issue
        .project
        .map(|p| super::ExternalParent {
            key: p.id,
            kind: super::ParentKind::Project,
        })
        .or_else(|| {
            issue.parent.map(|p| super::ExternalParent {
                key: p.identifier,
                kind: super::ParentKind::Issue,
            })
        });
    ExternalIssue {
        id: issue.id,
        key: issue.identifier,
        summary: issue.title,
        description: issue.description,
        kanban_issue_types,
        status: issue.state.name,
        assignee: issue.assignee.map(|u| ExternalUser {
            id: u.id,
            name: u.name,
            email: u.email,
            avatar_url: u.avatar_url,
        }),
        url: issue.url,
        priority: priority_to_string(issue.priority),
        blocked_by,
        parent,
        subtasks: issue
            .children
            .map(|c| c.nodes.into_iter().map(|n| n.identifier).collect())
            .unwrap_or_default(),
    }
}

#[derive(Debug, Deserialize)]
struct LinearStateRef {
    name: String,
}

// ─── Mutation Response Types ─────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct IssueCreateResponse {
    issue_create: IssueCreateResult,
}

#[derive(Debug, Deserialize)]
struct IssueCreateResult {
    success: bool,
    issue: Option<LinearIssue>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct IssueUpdateResponse {
    issue_update: IssueUpdateResult,
}

#[derive(Debug, Deserialize)]
struct IssueUpdateResult {
    success: bool,
    issue: Option<LinearIssue>,
}

// ─── Helper Query Response Types ─────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct IssueQueryResponse {
    issue: IssueWithTeam,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct IssueWithTeam {
    id: String,
    team: TeamRef,
}

#[derive(Debug, Deserialize)]
struct TeamRef {
    id: String,
}

#[derive(Debug, Deserialize)]
struct TeamStatesWithIdResponse {
    team: TeamWithStatesId,
}

#[derive(Debug, Deserialize)]
struct TeamWithStatesId {
    states: StatesConnectionWithId,
}

#[derive(Debug, Deserialize)]
struct StatesConnectionWithId {
    nodes: Vec<LinearStateWithId>,
}

#[derive(Debug, Deserialize)]
struct LinearStateWithId {
    id: String,
    name: String,
}

/// Convert Linear priority number to human-readable string
fn priority_to_string(priority: i32) -> Option<String> {
    match priority {
        0 => None,
        1 => Some("Urgent".to_string()),
        2 => Some("High".to_string()),
        3 => Some("Medium".to_string()),
        4 => Some("Low".to_string()),
        _ => None,
    }
}

#[async_trait]
impl KanbanProvider for LinearProvider {
    fn name(&self) -> &str {
        PROVIDER_NAME
    }

    fn is_configured(&self) -> bool {
        !self.api_key.is_empty()
    }

    async fn list_projects(&self) -> Result<Vec<ProjectInfo>, ApiError> {
        let query = r"
            query {
                teams {
                    nodes {
                        id
                        key
                        name
                    }
                }
            }
        ";

        let response: TeamsResponse = self.graphql(query, None).await?;

        Ok(response
            .teams
            .nodes
            .into_iter()
            .map(|t| ProjectInfo {
                id: t.id,
                key: t.key,
                name: t.name,
            })
            .collect())
    }

    async fn get_issue_types(&self, project_key: &str) -> Result<Vec<ExternalIssueType>, ApiError> {
        // Linear doesn't have traditional issue types like Jira.
        // We use labels as a proxy for issue types.
        let query = r"
            query($teamKey: String!) {
                team(id: $teamKey) {
                    labels {
                        nodes {
                            id
                            name
                            description
                            color
                        }
                    }
                }
            }
        ";

        let variables = serde_json::json!({
            "teamKey": project_key
        });

        let response: LabelsResponse = self.graphql(query, Some(variables)).await?;

        Ok(response
            .team
            .labels
            .nodes
            .into_iter()
            .map(|label| ExternalIssueType {
                id: label.id,
                name: label.name,
                description: label.description,
                icon_url: None,
                custom_fields: vec![],
            })
            .collect())
    }

    async fn test_connection(&self) -> Result<bool, ApiError> {
        let query = r"
            query {
                viewer {
                    id
                }
            }
        ";

        #[derive(Deserialize)]
        struct ViewerResponse {
            viewer: Viewer,
        }

        #[derive(Deserialize)]
        struct Viewer {
            #[allow(dead_code)]
            id: String,
        }

        match self.graphql::<ViewerResponse>(query, None).await {
            Ok(_) => Ok(true),
            Err(e) if e.is_auth_error() => {
                warn!("Linear authentication failed");
                Ok(false)
            }
            Err(e) => Err(e),
        }
    }

    async fn list_users(&self, project_key: &str) -> Result<Vec<ExternalUser>, ApiError> {
        let query = r"
            query($teamId: String!) {
                team(id: $teamId) {
                    members {
                        nodes {
                            id
                            name
                            email
                            avatarUrl
                        }
                    }
                }
            }
        ";

        let variables = serde_json::json!({
            "teamId": project_key
        });

        let response: TeamMembersResponse = self.graphql(query, Some(variables)).await?;

        Ok(response
            .team
            .members
            .nodes
            .into_iter()
            .map(|m| ExternalUser {
                id: m.id,
                name: m.name,
                email: m.email,
                avatar_url: m.avatar_url,
            })
            .collect())
    }

    async fn list_statuses(&self, project_key: &str) -> Result<Vec<String>, ApiError> {
        let query = r"
            query($teamId: String!) {
                team(id: $teamId) {
                    states {
                        nodes {
                            name
                            position
                        }
                    }
                }
            }
        ";

        let variables = serde_json::json!({
            "teamId": project_key
        });

        let response: TeamStatesResponse = self.graphql(query, Some(variables)).await?;

        // Sort by position and return names
        let mut states = response.team.states.nodes;
        states.sort_by(|a, b| {
            a.position
                .partial_cmp(&b.position)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(states.into_iter().map(|s| s.name).collect())
    }

    async fn list_issues(
        &self,
        project_key: &str,
        user_id: &str,
        statuses: &[String],
    ) -> Result<Vec<ExternalIssue>, ApiError> {
        // Build the filter object dynamically
        // Linear's GraphQL requires the filter to be inline, so we use a different query
        // depending on whether we have status filters
        let query = if statuses.is_empty() {
            r"
                query($teamId: ID!, $userId: ID!) {
                    issues(
                        filter: {
                            team: { id: { eq: $teamId } }
                            assignee: { id: { eq: $userId } }
                        }
                        first: 100
                    ) {
                        nodes {
                            ...SyncedIssue
                        }
                    }
                }
            "
        } else {
            r"
                query($teamId: ID!, $userId: ID!, $stateNames: [String!]!) {
                    issues(
                        filter: {
                            team: { id: { eq: $teamId } }
                            assignee: { id: { eq: $userId } }
                            state: { name: { in: $stateNames } }
                        }
                        first: 100
                    ) {
                        nodes {
                            ...SyncedIssue
                        }
                    }
                }
            "
        };

        let variables = if statuses.is_empty() {
            serde_json::json!({
                "teamId": project_key,
                "userId": user_id
            })
        } else {
            serde_json::json!({
                "teamId": project_key,
                "userId": user_id,
                "stateNames": statuses
            })
        };

        let query = format!("{query}{SYNCED_ISSUE_FRAGMENT}");
        let response: IssuesResponse = self.graphql(&query, Some(variables)).await?;

        Ok(response
            .issues
            .nodes
            .into_iter()
            .map(issue_from_linear)
            .collect())
    }

    async fn list_scope(
        &self,
        scope: &crate::config::SyncScope,
        statuses: &[String],
    ) -> Result<super::ScopeContents, ApiError> {
        let crate::config::SyncScope::LinearProject(project_id) = scope else {
            return Err(ApiError::http(
                "linear",
                400,
                format!("linear cannot sync scope {scope:?}"),
            ));
        };
        let query = project_scope_query(!statuses.is_empty());
        let mut header = None;
        let mut nodes = Vec::new();
        let mut after: Option<String> = None;
        for _ in 0..MAX_SCOPE_PAGES {
            let variables = serde_json::json!({
                "projectId": project_id,
                "projectFilterId": project_id,
                "after": after,
                "stateNames": statuses,
            });
            let page: ProjectScopeResponse = self.graphql(&query, Some(variables)).await?;
            header.get_or_insert(page.project);
            nodes.extend(page.issues.nodes);
            after = page
                .issues
                .page_info
                .filter(|p| p.has_next_page)
                .and_then(|p| p.end_cursor);
            if after.is_none() {
                break;
            }
        }
        let header = header.ok_or_else(|| ApiError::http("linear", 404, "project not found"))?;
        Ok(super::ScopeContents {
            key: project_id.clone(),
            title: header.name,
            description: header.description,
            url: header.url,
            issues: nodes.into_iter().map(issue_from_linear).collect(),
        })
    }

    async fn create_issue(
        &self,
        project_key: &str,
        request: super::CreateIssueRequest,
    ) -> Result<super::CreateIssueResponse, ApiError> {
        let mutation = r"
            mutation CreateIssue($input: IssueCreateInput!) {
                issueCreate(input: $input) {
                    success
                    issue {
                        id
                        identifier
                        title
                        description
                        state {
                            name
                        }
                        assignee {
                            id
                            name
                            email
                            avatarUrl
                        }
                        priority
                        url
                        labels {
                            nodes {
                                id
                                name
                            }
                        }
                    }
                }
            }
        ";

        let mut input = serde_json::json!({
            "teamId": project_key,
            "title": request.summary
        });

        if let Some(desc) = &request.description {
            input["description"] = serde_json::json!(desc);
        }
        if let Some(assignee) = &request.assignee_id {
            input["assigneeId"] = serde_json::json!(assignee);
        }

        let variables = serde_json::json!({ "input": input });

        let response: IssueCreateResponse = self.graphql(mutation, Some(variables)).await?;

        if !response.issue_create.success {
            return Err(ApiError::http(
                PROVIDER_NAME,
                400,
                "Failed to create issue".to_string(),
            ));
        }

        let issue = response.issue_create.issue.ok_or_else(|| {
            ApiError::http(
                PROVIDER_NAME,
                500,
                "No issue returned from create".to_string(),
            )
        })?;

        Ok(super::CreateIssueResponse {
            issue: ExternalIssue {
                blocked_by: Vec::new(),
                parent: None,
                subtasks: Vec::new(),
                id: issue.id,
                key: issue.identifier,
                summary: issue.title,
                description: issue.description,
                kanban_issue_types: issue
                    .labels
                    .map(|labels| {
                        labels
                            .nodes
                            .into_iter()
                            .map(|l| KanbanIssueTypeRef {
                                id: l.id,
                                name: l.name,
                            })
                            .collect()
                    })
                    .unwrap_or_default(),
                status: issue.state.name,
                assignee: issue.assignee.map(|u| ExternalUser {
                    id: u.id,
                    name: u.name,
                    email: u.email,
                    avatar_url: u.avatar_url,
                }),
                url: issue.url,
                priority: priority_to_string(issue.priority),
            },
        })
    }

    async fn update_issue_status(
        &self,
        issue_key: &str,
        request: super::UpdateStatusRequest,
    ) -> Result<super::ExternalIssue, ApiError> {
        // Get the issue's internal ID and team ID
        let (issue_id, team_id) = self.get_issue_info(issue_key).await?;

        // Find the state ID for the target status
        let state_id = self.find_state_id(&team_id, &request.status).await?;

        let mutation = r"
            mutation UpdateIssueState($issueId: String!, $stateId: String!) {
                issueUpdate(id: $issueId, input: { stateId: $stateId }) {
                    success
                    issue {
                        id
                        identifier
                        title
                        description
                        state {
                            name
                        }
                        assignee {
                            id
                            name
                            email
                            avatarUrl
                        }
                        priority
                        url
                        labels {
                            nodes {
                                id
                                name
                            }
                        }
                    }
                }
            }
        ";

        let variables = serde_json::json!({
            "issueId": issue_id,
            "stateId": state_id
        });

        let response: IssueUpdateResponse = self.graphql(mutation, Some(variables)).await?;

        if !response.issue_update.success {
            return Err(ApiError::http(
                PROVIDER_NAME,
                400,
                format!("Failed to update issue status to '{}'", request.status),
            ));
        }

        let issue = response.issue_update.issue.ok_or_else(|| {
            ApiError::http(
                PROVIDER_NAME,
                500,
                "No issue returned from update".to_string(),
            )
        })?;

        Ok(ExternalIssue {
            blocked_by: Vec::new(),
            parent: None,
            subtasks: Vec::new(),
            id: issue.id,
            key: issue.identifier,
            summary: issue.title,
            description: issue.description,
            kanban_issue_types: issue
                .labels
                .map(|labels| {
                    labels
                        .nodes
                        .into_iter()
                        .map(|l| KanbanIssueTypeRef {
                            id: l.id,
                            name: l.name,
                        })
                        .collect()
                })
                .unwrap_or_default(),
            status: issue.state.name,
            assignee: issue.assignee.map(|u| ExternalUser {
                id: u.id,
                name: u.name,
                email: u.email,
                avatar_url: u.avatar_url,
            }),
            url: issue.url,
            priority: priority_to_string(issue.priority),
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

        let (issue_id, team_id) = self.get_issue_info(issue_key).await?;

        // Query: current issue labels + all team labels
        let query = r"
            query($issueId: String!, $teamId: String!) {
                issue(id: $issueId) {
                    labels {
                        nodes { id name }
                    }
                }
                team(id: $teamId) {
                    labels {
                        nodes { id name }
                    }
                }
            }
        ";

        #[derive(Deserialize)]
        struct LabelNode {
            id: String,
            name: String,
        }
        #[derive(Deserialize)]
        struct LabelNodes {
            nodes: Vec<LabelNode>,
        }
        #[derive(Deserialize)]
        struct IssueLabels {
            labels: LabelNodes,
        }
        #[derive(Deserialize)]
        struct TeamLabels {
            labels: LabelNodes,
        }
        #[derive(Deserialize)]
        struct LabelsQueryResponse {
            issue: IssueLabels,
            team: TeamLabels,
        }

        let vars = serde_json::json!({ "issueId": issue_id, "teamId": team_id });
        let data: LabelsQueryResponse = self.graphql(query, Some(vars)).await?;

        // Start with existing issue label IDs
        let mut label_ids: Vec<String> = data
            .issue
            .labels
            .nodes
            .iter()
            .map(|l| l.id.clone())
            .collect();

        // For each desired label, find or create it
        for label_name in labels {
            // Check if already on the issue
            if data
                .issue
                .labels
                .nodes
                .iter()
                .any(|l| l.name.eq_ignore_ascii_case(label_name))
            {
                continue;
            }
            // Find existing team label
            if let Some(team_label) = data
                .team
                .labels
                .nodes
                .iter()
                .find(|l| l.name.eq_ignore_ascii_case(label_name))
            {
                label_ids.push(team_label.id.clone());
            } else {
                // Create new label
                let create_mutation = r"
                    mutation($input: IssueLabelCreateInput!) {
                        issueLabelCreate(input: $input) {
                            issueLabel { id name }
                        }
                    }
                ";
                #[derive(Deserialize)]
                struct IssueLabelCreateResponse {
                    #[serde(rename = "issueLabelCreate")]
                    issue_label_create: IssueLabelPayload,
                }
                #[derive(Deserialize)]
                struct IssueLabelPayload {
                    #[serde(rename = "issueLabel")]
                    issue_label: Option<LabelNode>,
                }
                let create_vars = serde_json::json!({
                    "input": { "name": label_name, "teamId": team_id }
                });
                let create_resp: IssueLabelCreateResponse =
                    self.graphql(create_mutation, Some(create_vars)).await?;
                if let Some(new_label) = create_resp.issue_label_create.issue_label {
                    label_ids.push(new_label.id);
                }
            }
        }

        // Update issue with all label IDs
        let update_mutation = r"
            mutation($id: String!, $labelIds: [String!]!) {
                issueUpdate(id: $id, input: { labelIds: $labelIds }) {
                    success
                }
            }
        ";
        #[derive(Deserialize)]
        struct LocalIssueUpdateResponse {
            #[serde(rename = "issueUpdate")]
            issue_update: LocalIssueUpdatePayload,
        }
        #[derive(Deserialize)]
        struct LocalIssueUpdatePayload {
            success: bool,
        }
        let update_vars = serde_json::json!({ "id": issue_id, "labelIds": label_ids });
        let update_resp: LocalIssueUpdateResponse =
            self.graphql(update_mutation, Some(update_vars)).await?;

        if !update_resp.issue_update.success {
            return Err(ApiError::http(
                PROVIDER_NAME,
                400,
                "Failed to update issue labels".to_string(),
            ));
        }
        Ok(())
    }

    async fn append_activity_log(
        &self,
        issue_key: &str,
        entry: &super::ActivityLogEntry,
    ) -> Result<(), ApiError> {
        let (issue_id, _team_id) = self.get_issue_info(issue_key).await?;

        let timestamp = entry.completed_at.format("%Y-%m-%d %H:%M UTC");
        let mut body = format!(
            "**opr8r activity** - step: `{}` | delegator: `{}` | {}",
            entry.step, entry.delegator, timestamp
        );
        if let Some(ref summary) = entry.summary {
            body.push_str(&format!("\n\n> {summary}"));
        }

        let mutation = r"
            mutation($input: CommentCreateInput!) {
                commentCreate(input: $input) {
                    success
                }
            }
        ";

        #[derive(Deserialize)]
        struct CommentCreateResponse {
            #[serde(rename = "commentCreate")]
            comment_create: CommentCreatePayload,
        }
        #[derive(Deserialize)]
        struct CommentCreatePayload {
            success: bool,
        }

        let vars = serde_json::json!({
            "input": { "issueId": issue_id, "body": body }
        });

        let resp: CommentCreateResponse = self.graphql(mutation, Some(vars)).await?;
        if !resp.comment_create.success {
            return Err(ApiError::http(
                PROVIDER_NAME,
                400,
                "Failed to create comment".to_string(),
            ));
        }
        Ok(())
    }
}

#[async_trait]
impl super::onboarding::KanbanOnboarding for LinearProvider {
    fn provider_kind(&self) -> super::KanbanProviderType {
        super::KanbanProviderType::Linear
    }

    async fn validate_onboarding(&self) -> Result<super::onboarding::ValidatedWorkspace, ApiError> {
        let details = self.validate_detailed().await?;
        let prefetched: Vec<super::onboarding::DiscoveredProject> = details
            .teams
            .iter()
            .map(|t| super::onboarding::DiscoveredProject {
                workspace_key: details.url_key.clone(),
                project_key: t.key.clone(),
                project_display_name: t.name.clone(),
                provider_url: None,
                provider_native_id: Some(t.id.clone()),
            })
            .collect();
        Ok(super::onboarding::ValidatedWorkspace {
            provider_kind: super::KanbanProviderType::Linear,
            workspace_key: details.url_key,
            workspace_display_name: details.org_name,
            sync_user_id: details.user_id,
            sync_user_display_name: details.user_name,
            api_key_env: "OPERATOR_LINEAR_API_KEY".to_string(),
            prefetched_projects: Some(prefetched),
            extra: super::onboarding::WorkspaceExtra::Linear,
        })
    }

    async fn discover_projects(
        &self,
        workspace: &super::onboarding::ValidatedWorkspace,
    ) -> Result<Vec<super::onboarding::DiscoveredProject>, ApiError> {
        Ok(workspace.prefetched_projects.clone().unwrap_or_default())
    }
}

/// Issues normalized from the checked-in relationship fixture.
#[cfg(test)]
pub(crate) fn fixture_issues() -> Vec<ExternalIssue> {
    let body =
        include_str!("../../../../tests/fixtures/kanban/linear/issues_with_relationships.json");
    let response: IssuesResponse = serde_json::from_str(body).unwrap();
    response
        .issues
        .nodes
        .into_iter()
        .map(issue_from_linear)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_env_not_configured() {
        env::remove_var("OPERATOR_LINEAR_API_KEY");

        let result = LinearProvider::from_env();
        assert!(result.is_err());
    }

    #[test]
    fn test_labels_deserialized_on_issue() {
        let json = r#"{
            "issues": {
                "nodes": [
                    {
                        "id": "issue-with-labels",
                        "identifier": "ENG-789",
                        "title": "Issue with labels",
                        "description": null,
                        "state": { "name": "Todo" },
                        "assignee": null,
                        "priority": 3,
                        "url": "https://linear.app/team/issue/ENG-789",
                        "labels": {
                            "nodes": [
                                { "id": "label-bug", "name": "Bug", "description": null, "color": null },
                                { "id": "label-urgent", "name": "Urgent", "description": null, "color": null }
                            ]
                        }
                    }
                ]
            }
        }"#;

        let response: IssuesResponse = serde_json::from_str(json).unwrap();
        let issue = &response.issues.nodes[0];
        assert!(issue.labels.is_some());
        let labels = issue.labels.as_ref().unwrap();
        assert_eq!(labels.nodes.len(), 2);
        assert_eq!(labels.nodes[0].id, "label-bug");
        assert_eq!(labels.nodes[0].name, "Bug");
    }

    #[test]
    fn test_priority_to_string() {
        assert_eq!(priority_to_string(0), None);
        assert_eq!(priority_to_string(1), Some("Urgent".to_string()));
        assert_eq!(priority_to_string(2), Some("High".to_string()));
        assert_eq!(priority_to_string(3), Some("Medium".to_string()));
        assert_eq!(priority_to_string(4), Some("Low".to_string()));
        assert_eq!(priority_to_string(5), None);
        assert_eq!(priority_to_string(-1), None);
    }

    #[test]
    fn test_deserialize_team_members_response() {
        let json = r#"{
            "team": {
                "members": {
                    "nodes": [
                        {
                            "id": "user-123",
                            "name": "John Doe",
                            "email": "john@example.com",
                            "avatarUrl": "https://example.com/avatar.png"
                        },
                        {
                            "id": "user-456",
                            "name": "Jane Smith",
                            "email": null,
                            "avatarUrl": null
                        }
                    ]
                }
            }
        }"#;

        let response: TeamMembersResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.team.members.nodes.len(), 2);
        assert_eq!(response.team.members.nodes[0].id, "user-123");
        assert_eq!(response.team.members.nodes[0].name, "John Doe");
        assert_eq!(
            response.team.members.nodes[0].email,
            Some("john@example.com".to_string())
        );
        assert!(response.team.members.nodes[1].email.is_none());
    }

    #[test]
    fn test_deserialize_team_states_response() {
        let json = r#"{
            "team": {
                "states": {
                    "nodes": [
                        { "name": "Backlog", "position": 0.0 },
                        { "name": "In Progress", "position": 2.0 },
                        { "name": "Todo", "position": 1.0 },
                        { "name": "Done", "position": 3.0 }
                    ]
                }
            }
        }"#;

        let response: TeamStatesResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.team.states.nodes.len(), 4);

        // Verify sorting works when applied
        let mut states = response.team.states.nodes;
        states.sort_by(|a, b| {
            a.position
                .partial_cmp(&b.position)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        assert_eq!(states[0].name, "Backlog");
        assert_eq!(states[1].name, "Todo");
        assert_eq!(states[2].name, "In Progress");
        assert_eq!(states[3].name, "Done");
    }

    #[test]
    fn test_issue_from_linear_maps_only_blocks_inverse_relations() {
        assert_eq!(fixture_issues()[0].blocked_by, vec!["ENG-3"]);
    }

    #[test]
    fn test_issue_from_linear_project_outranks_parent_issue() {
        let issues = fixture_issues();
        assert_eq!(
            issues[0].parent,
            Some(super::super::ExternalParent {
                key: "proj-billing".into(),
                kind: super::super::ParentKind::Project,
            })
        );
        assert_eq!(issues[0].subtasks, vec!["ENG-8", "ENG-9"]);
        assert_eq!(
            issues[1].parent,
            Some(super::super::ExternalParent {
                key: "ENG-7".into(),
                kind: super::super::ParentKind::Issue,
            })
        );
        assert!(issues[1].blocked_by.is_empty());
    }

    #[test]
    fn test_project_scope_query_filters_by_project_not_assignee() {
        let query = project_scope_query(false);
        assert!(query.contains("project: { id: { eq: $projectFilterId } }"));
        assert!(
            !query.contains("assignee: {"),
            "scoped sync must not filter by assignee"
        );
        assert!(!query.contains("$stateNames"));
        assert!(query.contains("fragment SyncedIssue"));
        assert!(project_scope_query(true).contains("state: { name: { in: $stateNames } }"));
    }

    #[test]
    fn test_project_scope_response_reads_header_and_page_info() {
        let page: ProjectScopeResponse = serde_json::from_str(
            r#"{"project": {"name": "Billing v2", "description": null, "url": "https://linear.app/acme/project/billing"},
                "issues": {"pageInfo": {"hasNextPage": true, "endCursor": "c1"}, "nodes": []}}"#,
        )
        .unwrap();
        assert_eq!(page.project.name, "Billing v2");
        let info = page.issues.page_info.unwrap();
        assert!(info.has_next_page);
        assert_eq!(info.end_cursor.as_deref(), Some("c1"));
    }

    #[test]
    fn test_synced_issue_fragment_requests_relationships() {
        for field in ["inverseRelations", "parent", "children", "project"] {
            assert!(SYNCED_ISSUE_FRAGMENT.contains(field), "{field}");
        }
    }

    #[test]
    fn test_deserialize_issues_response() {
        let json = r#"{
            "issues": {
                "nodes": [
                    {
                        "id": "issue-abc",
                        "identifier": "ENG-123",
                        "title": "Fix login bug",
                        "description": "Users cannot log in with SSO",
                        "state": { "name": "In Progress" },
                        "assignee": {
                            "id": "user-123",
                            "name": "John Doe",
                            "email": "john@example.com",
                            "avatarUrl": null
                        },
                        "priority": 2,
                        "url": "https://linear.app/team/issue/ENG-123"
                    }
                ]
            }
        }"#;

        let response: IssuesResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.issues.nodes.len(), 1);

        let issue = &response.issues.nodes[0];
        assert_eq!(issue.id, "issue-abc");
        assert_eq!(issue.identifier, "ENG-123");
        assert_eq!(issue.title, "Fix login bug");
        assert_eq!(
            issue.description,
            Some("Users cannot log in with SSO".to_string())
        );
        assert_eq!(issue.state.name, "In Progress");
        assert_eq!(issue.priority, 2);
        assert!(issue.assignee.is_some());
        assert_eq!(issue.assignee.as_ref().unwrap().name, "John Doe");
    }

    #[test]
    fn test_deserialize_issue_without_assignee() {
        let json = r#"{
            "issues": {
                "nodes": [
                    {
                        "id": "issue-xyz",
                        "identifier": "ENG-456",
                        "title": "Unassigned task",
                        "description": null,
                        "state": { "name": "Backlog" },
                        "assignee": null,
                        "priority": 0,
                        "url": "https://linear.app/team/issue/ENG-456"
                    }
                ]
            }
        }"#;

        let response: IssuesResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.issues.nodes.len(), 1);

        let issue = &response.issues.nodes[0];
        assert!(issue.assignee.is_none());
        assert!(issue.description.is_none());
        assert_eq!(issue.priority, 0);
        assert_eq!(priority_to_string(issue.priority), None);
    }
}

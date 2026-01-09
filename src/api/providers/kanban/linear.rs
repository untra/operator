//! Linear kanban provider implementation

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use tracing::{debug, warn};

use super::{ExternalIssue, ExternalIssueType, ExternalUser, KanbanProvider, ProjectInfo};
use crate::api::error::ApiError;
use crate::issuetypes::IssueType;

const LINEAR_API_URL: &str = "https://api.linear.app/graphql";
const PROVIDER_NAME: &str = "linear";

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
            client: Client::new(),
        }
    }

    /// Create from environment variables
    ///
    /// Required environment variables:
    /// - OPERATOR_LINEAR_API_KEY: Your Linear API key (lin_api_...)
    pub fn from_env() -> Result<Self, ApiError> {
        match env::var("OPERATOR_LINEAR_API_KEY") {
            Ok(key) if !key.is_empty() => Ok(Self::new(key)),
            _ => Err(ApiError::not_configured(PROVIDER_NAME)),
        }
    }

    /// Create from config with workspace as key
    ///
    /// The workspace slug is passed for identification (it's the HashMap key in KanbanConfig).
    /// The api_key is read from the environment variable specified in config.api_key_env.
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

    /// Execute a GraphQL query
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
            .map_err(|e| ApiError::http(PROVIDER_NAME, 0, format!("Parse error: {}", e)))?;

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
        let query = r#"
            query($identifier: String!) {
                issue(id: $identifier) {
                    id
                    team {
                        id
                    }
                }
            }
        "#;

        let variables = serde_json::json!({
            "identifier": identifier
        });

        let response: IssueQueryResponse = self.graphql(query, Some(variables)).await?;
        Ok((response.issue.id, response.issue.team.id))
    }

    /// Find the state ID for a given status name in a team
    async fn find_state_id(&self, team_id: &str, status_name: &str) -> Result<String, ApiError> {
        let query = r#"
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
        "#;

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
    nodes: Vec<LinearMember>,
}

#[derive(Debug, Deserialize)]
struct LinearMember {
    user: LinearUser,
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
        let query = r#"
            query {
                teams {
                    nodes {
                        id
                        key
                        name
                    }
                }
            }
        "#;

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
        let query = r#"
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
        "#;

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

    fn convert_to_issuetype(&self, external: &ExternalIssueType, project_key: &str) -> IssueType {
        // Sanitize key: uppercase, letters only, max 10 chars
        let key: String = external
            .name
            .chars()
            .filter(|c| c.is_ascii_alphabetic())
            .take(10)
            .collect::<String>()
            .to_uppercase();

        // Ensure minimum key length
        let key = if key.len() < 2 {
            format!("{}X", key)
        } else {
            key
        };

        IssueType::new_imported(
            key,
            external.name.clone(),
            external
                .description
                .clone()
                .unwrap_or_else(|| format!("Imported from Linear: {}", external.name)),
            "linear".to_string(),
            project_key.to_string(),
            Some(external.id.clone()),
        )
    }

    async fn test_connection(&self) -> Result<bool, ApiError> {
        let query = r#"
            query {
                viewer {
                    id
                }
            }
        "#;

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
        let query = r#"
            query($teamId: String!) {
                team(id: $teamId) {
                    members {
                        nodes {
                            user {
                                id
                                name
                                email
                                avatarUrl
                            }
                        }
                    }
                }
            }
        "#;

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
                id: m.user.id,
                name: m.user.name,
                email: m.user.email,
                avatar_url: m.user.avatar_url,
            })
            .collect())
    }

    async fn list_statuses(&self, project_key: &str) -> Result<Vec<String>, ApiError> {
        let query = r#"
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
        "#;

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
            r#"
                query($teamId: String!, $userId: String!) {
                    issues(
                        filter: {
                            team: { id: { eq: $teamId } }
                            assignee: { id: { eq: $userId } }
                        }
                        first: 100
                    ) {
                        nodes {
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
                        }
                    }
                }
            "#
        } else {
            r#"
                query($teamId: String!, $userId: String!, $stateNames: [String!]!) {
                    issues(
                        filter: {
                            team: { id: { eq: $teamId } }
                            assignee: { id: { eq: $userId } }
                            state: { name: { in: $stateNames } }
                        }
                        first: 100
                    ) {
                        nodes {
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
                        }
                    }
                }
            "#
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

        let response: IssuesResponse = self.graphql(query, Some(variables)).await?;

        Ok(response
            .issues
            .nodes
            .into_iter()
            .map(|issue| ExternalIssue {
                id: issue.id,
                key: issue.identifier,
                summary: issue.title,
                description: issue.description,
                issue_type: "Issue".to_string(), // Linear doesn't have issue types
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
            .collect())
    }

    async fn create_issue(
        &self,
        project_key: &str,
        request: super::CreateIssueRequest,
    ) -> Result<super::CreateIssueResponse, ApiError> {
        let mutation = r#"
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
                    }
                }
            }
        "#;

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
                id: issue.id,
                key: issue.identifier,
                summary: issue.title,
                description: issue.description,
                issue_type: "Issue".to_string(),
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

        let mutation = r#"
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
                    }
                }
            }
        "#;

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
            id: issue.id,
            key: issue.identifier,
            summary: issue.title,
            description: issue.description,
            issue_type: "Issue".to_string(),
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
    fn test_convert_to_issuetype() {
        let provider = LinearProvider::new("test_key".to_string());

        let external = ExternalIssueType {
            id: "label-123".to_string(),
            name: "Feature".to_string(),
            description: Some("A feature request".to_string()),
            icon_url: None,
            custom_fields: vec![],
        };

        let issue_type = provider.convert_to_issuetype(&external, "TEAM-ABC");

        assert_eq!(issue_type.key, "FEATURE");
        assert_eq!(issue_type.name, "Feature");
        assert_eq!(issue_type.glyph, "F");
        assert!(issue_type.is_autonomous());
    }

    #[test]
    fn test_convert_with_numbers() {
        let provider = LinearProvider::new("test_key".to_string());

        let external = ExternalIssueType {
            id: "label-123".to_string(),
            name: "P0 Bug".to_string(),
            description: None,
            icon_url: None,
            custom_fields: vec![],
        };

        let issue_type = provider.convert_to_issuetype(&external, "TEAM-ABC");

        // Should filter out numbers and spaces
        assert_eq!(issue_type.key, "PBUG");
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
                            "user": {
                                "id": "user-123",
                                "name": "John Doe",
                                "email": "john@example.com",
                                "avatarUrl": "https://example.com/avatar.png"
                            }
                        },
                        {
                            "user": {
                                "id": "user-456",
                                "name": "Jane Smith",
                                "email": null,
                                "avatarUrl": null
                            }
                        }
                    ]
                }
            }
        }"#;

        let response: TeamMembersResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.team.members.nodes.len(), 2);
        assert_eq!(response.team.members.nodes[0].user.id, "user-123");
        assert_eq!(response.team.members.nodes[0].user.name, "John Doe");
        assert_eq!(
            response.team.members.nodes[0].user.email,
            Some("john@example.com".to_string())
        );
        assert!(response.team.members.nodes[1].user.email.is_none());
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

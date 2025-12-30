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

    async fn list_users(&self, _project_key: &str) -> Result<Vec<ExternalUser>, ApiError> {
        // TODO: Implement Linear user listing
        // GraphQL: query { team(id: $teamKey) { members { nodes { id name email avatarUrl } } } }
        Err(ApiError::http(
            PROVIDER_NAME,
            501,
            "list_users not yet implemented".to_string(),
        ))
    }

    async fn list_statuses(&self, _project_key: &str) -> Result<Vec<String>, ApiError> {
        // TODO: Implement Linear status listing
        // GraphQL: query { team(id: $teamKey) { states { nodes { name position } } } }
        Err(ApiError::http(
            PROVIDER_NAME,
            501,
            "list_statuses not yet implemented".to_string(),
        ))
    }

    async fn list_issues(
        &self,
        _project_key: &str,
        _user_id: &str,
        _statuses: &[String],
    ) -> Result<Vec<ExternalIssue>, ApiError> {
        // TODO: Implement Linear issue listing
        // GraphQL: query { issues(filter: { team: {key: {eq: X}}, assignee: {id: {eq: Y}}, state: {name: {in: [...]}} }) { nodes { ... } } }
        Err(ApiError::http(
            PROVIDER_NAME,
            501,
            "list_issues not yet implemented".to_string(),
        ))
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
}

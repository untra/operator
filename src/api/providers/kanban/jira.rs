//! Jira kanban provider implementation

use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use std::env;
use tracing::{debug, warn};

use super::{ExternalIssueType, KanbanProvider, ProjectInfo};
use crate::api::error::ApiError;
use crate::issuetypes::IssueType;

const PROVIDER_NAME: &str = "jira";

/// Jira Cloud API provider
pub struct JiraProvider {
    domain: String,
    email: String,
    api_token: String,
    client: Client,
}

impl JiraProvider {
    /// Create a new Jira provider
    pub fn new(domain: String, email: String, api_token: String) -> Self {
        Self {
            domain,
            email,
            api_token,
            client: Client::new(),
        }
    }

    /// Create from environment variables
    ///
    /// Required environment variables:
    /// - OPERATOR_JIRA_DOMAIN: Your Jira domain (e.g., "your-domain.atlassian.net")
    /// - OPERATOR_JIRA_EMAIL: Your Atlassian account email
    /// - OPERATOR_JIRA_TOKEN: Your Jira API token
    pub fn from_env() -> Result<Self, ApiError> {
        let domain = env::var("OPERATOR_JIRA_DOMAIN").ok();
        let email = env::var("OPERATOR_JIRA_EMAIL").ok();
        let token = env::var("OPERATOR_JIRA_TOKEN").ok();

        match (domain, email, token) {
            (Some(d), Some(e), Some(t)) if !d.is_empty() && !e.is_empty() && !t.is_empty() => {
                Ok(Self::new(d, e, t))
            }
            _ => Err(ApiError::not_configured(PROVIDER_NAME)),
        }
    }

    /// Get the base URL for API requests
    fn base_url(&self) -> String {
        format!("https://{}/rest/api/3", self.domain)
    }

    /// Get Basic Auth header value (simple Base64 encoding)
    fn auth_header(&self) -> String {
        let credentials = format!("{}:{}", self.email, self.api_token);
        let encoded = simple_base64_encode(credentials.as_bytes());
        format!("Basic {}", encoded)
    }

    /// Make an authenticated GET request
    async fn get<T: for<'de> Deserialize<'de>>(&self, path: &str) -> Result<T, ApiError> {
        let url = format!("{}{}", self.base_url(), path);
        debug!("Jira GET: {}", url);

        let response = self
            .client
            .get(&url)
            .header("Authorization", self.auth_header())
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| ApiError::network(PROVIDER_NAME, e.to_string()))?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return match status.as_u16() {
                401 => Err(ApiError::unauthorized(PROVIDER_NAME)),
                403 => Err(ApiError::forbidden(PROVIDER_NAME)),
                404 => Err(ApiError::http(
                    PROVIDER_NAME,
                    404,
                    format!("Not found: {}", path),
                )),
                429 => Err(ApiError::rate_limited(PROVIDER_NAME, None)),
                _ => Err(ApiError::http(PROVIDER_NAME, status.as_u16(), body)),
            };
        }

        response
            .json()
            .await
            .map_err(|e| ApiError::http(PROVIDER_NAME, 0, format!("Parse error: {}", e)))
    }
}

/// Simple Base64 encoding implementation (for Basic Auth only)
fn simple_base64_encode(data: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    let mut result = String::new();
    let mut chunks = data.chunks_exact(3);

    for chunk in chunks.by_ref() {
        let n = ((chunk[0] as u32) << 16) | ((chunk[1] as u32) << 8) | (chunk[2] as u32);
        result.push(ALPHABET[(n >> 18 & 0x3F) as usize] as char);
        result.push(ALPHABET[(n >> 12 & 0x3F) as usize] as char);
        result.push(ALPHABET[(n >> 6 & 0x3F) as usize] as char);
        result.push(ALPHABET[(n & 0x3F) as usize] as char);
    }

    let remainder = chunks.remainder();
    if remainder.len() == 1 {
        let n = (remainder[0] as u32) << 16;
        result.push(ALPHABET[(n >> 18 & 0x3F) as usize] as char);
        result.push(ALPHABET[(n >> 12 & 0x3F) as usize] as char);
        result.push_str("==");
    } else if remainder.len() == 2 {
        let n = ((remainder[0] as u32) << 16) | ((remainder[1] as u32) << 8);
        result.push(ALPHABET[(n >> 18 & 0x3F) as usize] as char);
        result.push(ALPHABET[(n >> 12 & 0x3F) as usize] as char);
        result.push(ALPHABET[(n >> 6 & 0x3F) as usize] as char);
        result.push('=');
    }

    result
}

// Jira API response types
#[derive(Debug, Deserialize)]
struct JiraProjectsResponse {
    values: Vec<JiraProject>,
}

#[derive(Debug, Deserialize)]
struct JiraProject {
    id: String,
    key: String,
    name: String,
}

#[derive(Debug, Deserialize)]
struct JiraIssueTypesResponse {
    #[serde(rename = "issueTypes")]
    issue_types: Vec<JiraIssueType>,
}

#[derive(Debug, Deserialize)]
struct JiraIssueType {
    id: String,
    name: String,
    description: Option<String>,
    #[serde(rename = "iconUrl")]
    icon_url: Option<String>,
}

#[async_trait]
impl KanbanProvider for JiraProvider {
    fn name(&self) -> &str {
        PROVIDER_NAME
    }

    fn is_configured(&self) -> bool {
        !self.domain.is_empty() && !self.email.is_empty() && !self.api_token.is_empty()
    }

    async fn list_projects(&self) -> Result<Vec<ProjectInfo>, ApiError> {
        let response: JiraProjectsResponse = self.get("/project/search").await?;

        Ok(response
            .values
            .into_iter()
            .map(|p| ProjectInfo {
                id: p.id,
                key: p.key,
                name: p.name,
            })
            .collect())
    }

    async fn get_issue_types(&self, project_key: &str) -> Result<Vec<ExternalIssueType>, ApiError> {
        let path = format!("/issue/createmeta/{}/issuetypes", project_key);
        let response: JiraIssueTypesResponse = self.get(&path).await?;

        Ok(response
            .issue_types
            .into_iter()
            .map(|it| ExternalIssueType {
                id: it.id,
                name: it.name,
                description: it.description,
                icon_url: it.icon_url,
                custom_fields: vec![], // Could fetch with /issue/createmeta/{project}/issuetypes/{issueTypeId}
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
                .unwrap_or_else(|| format!("Imported from Jira: {}", external.name)),
            "jira".to_string(),
            project_key.to_string(),
            Some(external.id.clone()),
        )
    }

    async fn test_connection(&self) -> Result<bool, ApiError> {
        // Try to get current user to verify credentials
        #[derive(Deserialize)]
        struct MySelf {
            #[serde(rename = "accountId")]
            #[allow(dead_code)]
            account_id: String,
        }

        match self.get::<MySelf>("/myself").await {
            Ok(_) => Ok(true),
            Err(e) if e.is_auth_error() => {
                warn!("Jira authentication failed");
                Ok(false)
            }
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base64_encode() {
        assert_eq!(simple_base64_encode(b"Hello"), "SGVsbG8=");
        assert_eq!(
            simple_base64_encode(b"Hello, World!"),
            "SGVsbG8sIFdvcmxkIQ=="
        );
        assert_eq!(simple_base64_encode(b"abc"), "YWJj");
        assert_eq!(simple_base64_encode(b"ab"), "YWI=");
        assert_eq!(simple_base64_encode(b"a"), "YQ==");
    }

    #[test]
    fn test_from_env_not_configured() {
        // Clear any existing env vars for the test
        env::remove_var("OPERATOR_JIRA_DOMAIN");
        env::remove_var("OPERATOR_JIRA_EMAIL");
        env::remove_var("OPERATOR_JIRA_TOKEN");

        let result = JiraProvider::from_env();
        assert!(result.is_err());
    }

    #[test]
    fn test_convert_to_issuetype() {
        let provider = JiraProvider::new(
            "test.atlassian.net".to_string(),
            "test@test.com".to_string(),
            "token".to_string(),
        );

        let external = ExternalIssueType {
            id: "10001".to_string(),
            name: "Bug".to_string(),
            description: Some("A software bug".to_string()),
            icon_url: None,
            custom_fields: vec![],
        };

        let issue_type = provider.convert_to_issuetype(&external, "PROJ");

        assert_eq!(issue_type.key, "BUG");
        assert_eq!(issue_type.name, "Bug");
        assert_eq!(issue_type.glyph, "B");
        assert!(issue_type.is_autonomous());
    }

    #[test]
    fn test_convert_long_name() {
        let provider = JiraProvider::new(
            "test.atlassian.net".to_string(),
            "test@test.com".to_string(),
            "token".to_string(),
        );

        let external = ExternalIssueType {
            id: "10001".to_string(),
            name: "Very Long Issue Type Name".to_string(),
            description: None,
            icon_url: None,
            custom_fields: vec![],
        };

        let issue_type = provider.convert_to_issuetype(&external, "PROJ");

        // Should be truncated to 10 chars
        assert!(issue_type.key.len() <= 10);
        assert!(issue_type.key.chars().all(|c| c.is_ascii_uppercase()));
    }

    #[test]
    fn test_convert_short_name() {
        let provider = JiraProvider::new(
            "test.atlassian.net".to_string(),
            "test@test.com".to_string(),
            "token".to_string(),
        );

        let external = ExternalIssueType {
            id: "10001".to_string(),
            name: "X".to_string(),
            description: None,
            icon_url: None,
            custom_fields: vec![],
        };

        let issue_type = provider.convert_to_issuetype(&external, "PROJ");

        // Should be padded to minimum 2 chars
        assert!(issue_type.key.len() >= 2);
    }
}

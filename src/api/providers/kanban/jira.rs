//! Jira kanban provider implementation

use async_trait::async_trait;
use reqwest::Client;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::env;
use tracing::{debug, warn};
use ts_rs::TS;

use super::{ExternalIssue, ExternalIssueType, ExternalUser, KanbanProvider, ProjectInfo};
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
    /// - OPERATOR_JIRA_API_KEY: Your Jira API key/token
    pub fn from_env() -> Result<Self, ApiError> {
        let domain = env::var("OPERATOR_JIRA_DOMAIN").ok();
        let email = env::var("OPERATOR_JIRA_EMAIL").ok();
        let token = env::var("OPERATOR_JIRA_API_KEY").ok();

        match (domain, email, token) {
            (Some(d), Some(e), Some(t)) if !d.is_empty() && !e.is_empty() && !t.is_empty() => {
                Ok(Self::new(d, e, t))
            }
            _ => Err(ApiError::not_configured(PROVIDER_NAME)),
        }
    }

    /// Create from config with domain as key
    ///
    /// The domain is passed separately since it's the HashMap key in KanbanConfig.
    /// The api_key is read from the environment variable specified in config.api_key_env.
    pub fn from_config(domain: &str, config: &crate::config::JiraConfig) -> Result<Self, ApiError> {
        let api_key = env::var(&config.api_key_env).ok();

        match api_key {
            Some(key) if !key.is_empty() && !config.email.is_empty() => {
                Ok(Self::new(domain.to_string(), config.email.clone(), key))
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

    /// Make an authenticated POST request
    async fn post<T: for<'de> Deserialize<'de>, B: Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, ApiError> {
        let url = format!("{}{}", self.base_url(), path);
        debug!("Jira POST: {}", url);

        let response = self
            .client
            .post(&url)
            .header("Authorization", self.auth_header())
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            .json(body)
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

    /// Make an authenticated POST request that returns no content (204)
    async fn post_no_content<B: Serialize>(&self, path: &str, body: &B) -> Result<(), ApiError> {
        let url = format!("{}{}", self.base_url(), path);
        debug!("Jira POST (no content): {}", url);

        let response = self
            .client
            .post(&url)
            .header("Authorization", self.auth_header())
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            .json(body)
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

        Ok(())
    }

    /// Fetch a single issue by key and convert to ExternalIssue
    async fn fetch_issue(&self, issue_key: &str) -> Result<ExternalIssue, ApiError> {
        let path = format!("/issue/{}", issue_key);
        let issue: JiraIssue = self.get(&path).await?;

        let url = format!("https://{}/browse/{}", self.domain, issue.key);
        Ok(ExternalIssue {
            id: issue.id,
            key: issue.key,
            summary: issue.fields.summary,
            description: extract_description_text(&issue.fields.description),
            issue_type: issue.fields.issuetype.name,
            status: issue.fields.status.name,
            assignee: issue.fields.assignee.map(|u| ExternalUser {
                id: u.account_id,
                name: u.display_name,
                email: u.email_address,
                avatar_url: u.avatar_urls.and_then(|a| a.large),
            }),
            url,
            priority: issue.fields.priority.map(|p| p.name),
        })
    }
}

/// Simple Base64 encoding implementation (for Basic Auth only)
fn simple_base64_encode(data: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    let mut result = String::new();
    let mut chunks = data.chunks_exact(3);

    for chunk in chunks.by_ref() {
        let n = ((chunk[0] as u32) << 16) | ((chunk[1] as u32) << 8) | (chunk[2] as u32);
        result.push(ALPHABET[((n >> 18) & 0x3F) as usize] as char);
        result.push(ALPHABET[((n >> 12) & 0x3F) as usize] as char);
        result.push(ALPHABET[((n >> 6) & 0x3F) as usize] as char);
        result.push(ALPHABET[(n & 0x3F) as usize] as char);
    }

    let remainder = chunks.remainder();
    if remainder.len() == 1 {
        let n = (remainder[0] as u32) << 16;
        result.push(ALPHABET[((n >> 18) & 0x3F) as usize] as char);
        result.push(ALPHABET[((n >> 12) & 0x3F) as usize] as char);
        result.push_str("==");
    } else if remainder.len() == 2 {
        let n = ((remainder[0] as u32) << 16) | ((remainder[1] as u32) << 8);
        result.push(ALPHABET[((n >> 18) & 0x3F) as usize] as char);
        result.push(ALPHABET[((n >> 12) & 0x3F) as usize] as char);
        result.push(ALPHABET[((n >> 6) & 0x3F) as usize] as char);
        result.push('=');
    }

    result
}

/// Simple URL encoding for JQL queries
fn simple_url_encode(s: &str) -> String {
    let mut result = String::with_capacity(s.len() * 3);
    for c in s.chars() {
        match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => result.push(c),
            ' ' => result.push_str("%20"),
            '"' => result.push_str("%22"),
            '=' => result.push_str("%3D"),
            '(' => result.push_str("%28"),
            ')' => result.push_str("%29"),
            ',' => result.push_str("%2C"),
            _ => {
                for b in c.to_string().as_bytes() {
                    result.push_str(&format!("%{:02X}", b));
                }
            }
        }
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

// ─── Jira API Response Types ────────────────────────────────────────────────
// These types are exported for schema/binding generation

/// Jira user information from assignable users API
/// GET /rest/api/3/user/assignable/search?project={key}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export)]
pub struct JiraUser {
    /// Atlassian account ID (e.g., "5e3f7acd9876543210abcdef")
    #[serde(rename = "accountId")]
    pub account_id: String,
    /// User's display name
    #[serde(rename = "displayName")]
    pub display_name: String,
    /// User's email address (may be hidden by privacy settings)
    #[serde(rename = "emailAddress")]
    pub email_address: Option<String>,
    /// Avatar URLs in various sizes
    #[serde(rename = "avatarUrls")]
    pub avatar_urls: Option<JiraAvatarUrls>,
}

/// Avatar URLs for a Jira user
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export)]
pub struct JiraAvatarUrls {
    /// 48x48 pixel avatar URL
    #[serde(rename = "48x48")]
    pub large: Option<String>,
}

/// Project status information from Jira
/// GET /rest/api/3/project/{key}/statuses
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export)]
pub struct JiraProjectStatus {
    /// List of statuses available for this issue type
    pub statuses: Vec<JiraStatus>,
}

/// Workflow status in Jira
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export)]
pub struct JiraStatus {
    /// Status name (e.g., "To Do", "In Progress", "Done")
    pub name: String,
}

/// Search response from Jira JQL query
/// GET /rest/api/3/search?jql=...
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export)]
pub struct JiraSearchResponse {
    /// List of issues matching the JQL query
    pub issues: Vec<JiraIssue>,
}

/// A Jira issue from search results
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export)]
pub struct JiraIssue {
    /// Internal Jira issue ID
    pub id: String,
    /// Issue key (e.g., "PROJ-123")
    pub key: String,
    /// Issue fields containing summary, status, etc.
    pub fields: JiraIssueFields,
}

/// Fields of a Jira issue
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export)]
pub struct JiraIssueFields {
    /// Issue summary/title
    pub summary: String,
    /// Issue description in ADF format
    pub description: Option<JiraDescription>,
    /// Issue type (Bug, Story, Task, etc.)
    pub issuetype: JiraIssueTypeRef,
    /// Current workflow status
    pub status: JiraStatusRef,
    /// Assigned user (if any)
    pub assignee: Option<JiraUser>,
    /// Issue priority (if set)
    pub priority: Option<JiraPriority>,
}

/// Jira description in Atlassian Document Format (ADF)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export)]
pub struct JiraDescription {
    /// ADF content nodes - parsed to extract plain text
    pub content: Option<Vec<serde_json::Value>>,
}

/// Reference to an issue type
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export)]
pub struct JiraIssueTypeRef {
    /// Issue type name (e.g., "Bug", "Story", "Task")
    pub name: String,
}

/// Reference to a workflow status
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export)]
pub struct JiraStatusRef {
    /// Status name (e.g., "To Do", "In Progress", "Done")
    pub name: String,
}

/// Issue priority level
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export)]
pub struct JiraPriority {
    /// Priority name (e.g., "Highest", "High", "Medium", "Low", "Lowest")
    pub name: String,
}

// ─── Create Issue Types ──────────────────────────────────────────────────────

/// Response from creating a Jira issue
#[derive(Debug, Deserialize)]
struct JiraCreateIssueResponse {
    id: String,
    key: String,
}

/// Jira transitions response
#[derive(Debug, Deserialize)]
struct JiraTransitionsResponse {
    transitions: Vec<JiraTransition>,
}

/// A workflow transition in Jira
#[derive(Debug, Deserialize)]
struct JiraTransition {
    id: String,
    name: String,
    to: JiraTransitionTarget,
}

/// Target status for a transition
#[derive(Debug, Deserialize)]
struct JiraTransitionTarget {
    name: String,
}

/// Extract plain text from Jira's ADF (Atlassian Document Format) description
fn extract_description_text(desc: &Option<JiraDescription>) -> Option<String> {
    desc.as_ref()
        .and_then(|d| {
            d.content
                .as_ref()
                .map(|content| extract_text_from_adf(content))
        })
        .filter(|s| !s.is_empty())
}

fn extract_text_from_adf(nodes: &[serde_json::Value]) -> String {
    let mut text = String::new();
    for node in nodes {
        if let Some(t) = node.get("text").and_then(|v| v.as_str()) {
            text.push_str(t);
        }
        if let Some(content) = node.get("content").and_then(|v| v.as_array()) {
            text.push_str(&extract_text_from_adf(content));
        }
    }
    text
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

    async fn list_users(&self, project_key: &str) -> Result<Vec<ExternalUser>, ApiError> {
        let path = format!("/user/assignable/search?project={}", project_key);
        let users: Vec<JiraUser> = self.get(&path).await?;

        Ok(users
            .into_iter()
            .map(|u| ExternalUser {
                id: u.account_id,
                name: u.display_name,
                email: u.email_address,
                avatar_url: u.avatar_urls.and_then(|a| a.large),
            })
            .collect())
    }

    async fn list_statuses(&self, project_key: &str) -> Result<Vec<String>, ApiError> {
        let path = format!("/project/{}/statuses", project_key);
        let response: Vec<JiraProjectStatus> = self.get(&path).await?;

        // Flatten statuses from all issue types, deduplicate
        let mut statuses: Vec<String> = response
            .into_iter()
            .flat_map(|ps| ps.statuses.into_iter().map(|s| s.name))
            .collect();
        statuses.sort();
        statuses.dedup();
        Ok(statuses)
    }

    async fn list_issues(
        &self,
        project_key: &str,
        user_id: &str,
        statuses: &[String],
    ) -> Result<Vec<ExternalIssue>, ApiError> {
        // Build JQL query
        let status_clause = if statuses.is_empty() {
            String::new()
        } else {
            let quoted: Vec<String> = statuses.iter().map(|s| format!("\"{}\"", s)).collect();
            format!(" AND status IN ({})", quoted.join(","))
        };

        let jql = format!(
            "project = \"{}\" AND assignee = \"{}\"{}",
            project_key, user_id, status_clause
        );
        let encoded_jql = simple_url_encode(&jql);
        let path = format!("/search?jql={}&maxResults=100", encoded_jql);

        let response: JiraSearchResponse = self.get(&path).await?;

        Ok(response
            .issues
            .into_iter()
            .map(|issue| {
                let url = format!("https://{}/browse/{}", self.domain, issue.key);
                ExternalIssue {
                    id: issue.id,
                    key: issue.key,
                    summary: issue.fields.summary,
                    description: extract_description_text(&issue.fields.description),
                    issue_type: issue.fields.issuetype.name,
                    status: issue.fields.status.name,
                    assignee: issue.fields.assignee.map(|u| ExternalUser {
                        id: u.account_id,
                        name: u.display_name,
                        email: u.email_address,
                        avatar_url: u.avatar_urls.and_then(|a| a.large),
                    }),
                    url,
                    priority: issue.fields.priority.map(|p| p.name),
                }
            })
            .collect())
    }

    async fn create_issue(
        &self,
        project_key: &str,
        request: super::CreateIssueRequest,
    ) -> Result<super::CreateIssueResponse, ApiError> {
        // Build ADF description if provided
        let description = request.description.map(|text| {
            serde_json::json!({
                "type": "doc",
                "version": 1,
                "content": [{
                    "type": "paragraph",
                    "content": [{
                        "type": "text",
                        "text": text
                    }]
                }]
            })
        });

        // Build request body
        let mut fields = serde_json::json!({
            "project": { "key": project_key },
            "summary": request.summary,
            "issuetype": { "name": "Task" }  // Default to Task
        });

        if let Some(desc) = description {
            fields["description"] = desc;
        }

        if let Some(assignee_id) = request.assignee_id {
            fields["assignee"] = serde_json::json!({ "accountId": assignee_id });
        }

        let body = serde_json::json!({ "fields": fields });

        // Create the issue
        let created: JiraCreateIssueResponse = self.post("/issue", &body).await?;

        // Fetch the full issue details to return
        let issue = self.fetch_issue(&created.key).await?;

        Ok(super::CreateIssueResponse { issue })
    }

    async fn update_issue_status(
        &self,
        issue_key: &str,
        request: super::UpdateStatusRequest,
    ) -> Result<ExternalIssue, ApiError> {
        // Get available transitions
        let path = format!("/issue/{}/transitions", issue_key);
        let transitions_response: JiraTransitionsResponse = self.get(&path).await?;

        // Find the transition to the target status
        let transition = transitions_response
            .transitions
            .iter()
            .find(|t| t.to.name.eq_ignore_ascii_case(&request.status))
            .ok_or_else(|| {
                ApiError::http(
                    PROVIDER_NAME,
                    400,
                    format!(
                        "No transition to status '{}'. Available: {}",
                        request.status,
                        transitions_response
                            .transitions
                            .iter()
                            .map(|t| t.to.name.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                    ),
                )
            })?;

        // Execute the transition
        let transition_body = serde_json::json!({
            "transition": { "id": transition.id }
        });
        let transitions_path = format!("/issue/{}/transitions", issue_key);
        self.post_no_content(&transitions_path, &transition_body)
            .await?;

        // Fetch and return the updated issue
        self.fetch_issue(issue_key).await
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

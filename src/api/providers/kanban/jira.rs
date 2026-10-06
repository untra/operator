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
use crate::issuetypes::kanban_type::KanbanIssueTypeRef;

const PROVIDER_NAME: &str = "jira";

/// Detailed validation result for Jira onboarding.
///
/// Richer than `KanbanProvider::test_connection` - includes the authenticated
/// user's `accountId` (used as `sync_user_id` in config) and display name.
#[derive(Debug, Clone)]
pub struct JiraValidationDetails {
    pub account_id: String,
    pub display_name: String,
}

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
            client: crate::http_client::default_client(),
        }
    }

    /// Create from environment variables
    ///
    /// Required environment variables:
    /// - `OPERATOR_JIRA_DOMAIN`: Your Jira domain (e.g., "your-domain.atlassian.net")
    /// - `OPERATOR_JIRA_EMAIL`: Your Atlassian account email
    /// - `OPERATOR_JIRA_API_KEY`: Your Jira API key/token
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
    /// The domain is passed separately since it's the `HashMap` key in `KanbanConfig`.
    /// The `api_key` is read from the environment variable specified in `config.api_key_env`.
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

    /// Make an authenticated GET request
    async fn get<T: for<'de> Deserialize<'de>>(&self, path: &str) -> Result<T, ApiError> {
        let url = format!("{}{}", self.base_url(), path);
        debug!("Jira GET: {}", url);

        let response = self
            .client
            .get(&url)
            .basic_auth(&self.email, Some(&self.api_token))
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
                    format!("Not found: {path}"),
                )),
                429 => Err(ApiError::rate_limited(PROVIDER_NAME, None)),
                _ => Err(ApiError::http(PROVIDER_NAME, status.as_u16(), body)),
            };
        }

        response
            .json()
            .await
            .map_err(|e| ApiError::http(PROVIDER_NAME, 0, format!("Parse error: {e}")))
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
            .basic_auth(&self.email, Some(&self.api_token))
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
                    format!("Not found: {path}"),
                )),
                429 => Err(ApiError::rate_limited(PROVIDER_NAME, None)),
                _ => Err(ApiError::http(PROVIDER_NAME, status.as_u16(), body)),
            };
        }

        response
            .json()
            .await
            .map_err(|e| ApiError::http(PROVIDER_NAME, 0, format!("Parse error: {e}")))
    }

    /// Make an authenticated PUT request that returns no content (204/200)
    async fn put_no_content<B: Serialize>(&self, path: &str, body: &B) -> Result<(), ApiError> {
        let url = format!("{}{}", self.base_url(), path);
        debug!("Jira PUT (no content): {}", url);

        let response = self
            .client
            .put(&url)
            .basic_auth(&self.email, Some(&self.api_token))
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
                    format!("Not found: {path}"),
                )),
                429 => Err(ApiError::rate_limited(PROVIDER_NAME, None)),
                _ => Err(ApiError::http(PROVIDER_NAME, status.as_u16(), body)),
            };
        }
        Ok(())
    }

    /// Make an authenticated POST request that returns no content (204)
    async fn post_no_content<B: Serialize>(&self, path: &str, body: &B) -> Result<(), ApiError> {
        let url = format!("{}{}", self.base_url(), path);
        debug!("Jira POST (no content): {}", url);

        let response = self
            .client
            .post(&url)
            .basic_auth(&self.email, Some(&self.api_token))
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
                    format!("Not found: {path}"),
                )),
                429 => Err(ApiError::rate_limited(PROVIDER_NAME, None)),
                _ => Err(ApiError::http(PROVIDER_NAME, status.as_u16(), body)),
            };
        }

        Ok(())
    }

    /// Fetch a single issue by key and convert to `ExternalIssue`
    async fn fetch_issue(&self, issue_key: &str) -> Result<ExternalIssue, ApiError> {
        let path = format!("/issue/{issue_key}");
        let issue: JiraIssue = self.get(&path).await?;

        let url = format!("https://{}/browse/{}", self.domain, issue.key);
        Ok(ExternalIssue {
            blocked_by: Vec::new(),
            parent: None,
            subtasks: Vec::new(),
            id: issue.id,
            key: issue.key,
            summary: issue.fields.summary,
            description: extract_description_text(&issue.fields.description),
            kanban_issue_types: vec![KanbanIssueTypeRef {
                id: String::new(), // not available from single issue fetch
                name: issue.fields.issuetype.name,
            }],
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

    /// Detailed credential validation for onboarding.
    ///
    /// Hits `/rest/api/3/myself` and returns the authenticated user's
    /// `accountId` + `displayName`, which the onboarding flow uses as
    /// `sync_user_id` in `ProjectSyncConfig`.
    pub async fn validate_detailed(&self) -> Result<JiraValidationDetails, ApiError> {
        #[derive(Deserialize)]
        struct MySelf {
            #[serde(rename = "accountId")]
            account_id: String,
            #[serde(rename = "displayName", default)]
            display_name: String,
        }

        let me: MySelf = self.get("/myself").await?;
        Ok(JiraValidationDetails {
            account_id: me.account_id,
            display_name: me.display_name,
        })
    }
}

/// Issue fields requested from `/search/jql`, which otherwise returns only ids.
const SEARCH_FIELDS: &str =
    "summary,description,issuetype,status,assignee,priority,issuelinks,parent,subtasks";

/// Pages fetched per scoped search before giving up (100 issues each).
const MAX_SEARCH_PAGES: usize = 50;

/// ` AND status IN (...)`, or nothing when no statuses are given.
fn status_clause(statuses: &[String]) -> String {
    if statuses.is_empty() {
        return String::new();
    }
    let quoted: Vec<String> = statuses.iter().map(|s| format!("\"{s}\"")).collect();
    format!(" AND status IN ({})", quoted.join(","))
}

/// Every child of an epic or parent issue, any assignee.
fn scope_jql(parent_key: &str, statuses: &[String]) -> String {
    format!("parent = \"{parent_key}\"{}", status_clause(statuses))
}

/// Search path for a JQL query, with the query string form-encoded.
fn search_path(jql: &str, page_token: Option<&str>) -> String {
    let mut query = url::form_urlencoded::Serializer::new(String::new());
    query
        .append_pair("jql", jql)
        .append_pair("maxResults", "100")
        .append_pair("fields", SEARCH_FIELDS);
    if let Some(token) = page_token {
        query.append_pair("nextPageToken", token);
    }
    let query = query.finish();
    format!("/search/jql?{query}")
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
    /// Token for the next page; absent on the last page
    #[serde(default, rename = "nextPageToken")]
    pub next_page_token: Option<String>,
}

/// An issue's title fields, fetched for a scope container
#[derive(Debug, Deserialize)]
struct JiraIssueHeader {
    key: String,
    fields: JiraHeaderFields,
}

#[derive(Debug, Deserialize)]
struct JiraHeaderFields {
    summary: String,
    #[serde(default)]
    description: Option<JiraDescription>,
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
    /// Links to other issues (blocks, relates, ...)
    #[serde(default)]
    pub issuelinks: Vec<JiraIssueLink>,
    /// Parent issue or epic
    #[serde(default)]
    pub parent: Option<JiraLinkedIssue>,
    /// Sub-task issues
    #[serde(default)]
    pub subtasks: Vec<JiraLinkedIssue>,
}

/// A link between two Jira issues
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct JiraIssueLink {
    /// Link type, e.g. `Blocks` with inward text `is blocked by`
    #[serde(rename = "type")]
    pub link_type: JiraIssueLinkType,
    /// Present when the other issue is on the inward side (e.g. the blocker)
    #[serde(default)]
    pub inward_issue: Option<JiraLinkedIssue>,
    /// Present when the other issue is on the outward side (e.g. the blocked)
    #[serde(default)]
    pub outward_issue: Option<JiraLinkedIssue>,
}

/// Jira issue link type
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export)]
pub struct JiraIssueLinkType {
    /// Link type name (e.g. "Blocks")
    pub name: String,
    /// Inward description (e.g. "is blocked by")
    #[serde(default)]
    pub inward: String,
}

/// A related issue as embedded in links, parent and subtasks
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export)]
pub struct JiraLinkedIssue {
    /// Issue key (e.g. "PROJ-12")
    pub key: String,
    /// Partial fields; only the issue type is read
    #[serde(default)]
    pub fields: Option<JiraLinkedFields>,
}

/// Partial fields of a related issue
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export)]
pub struct JiraLinkedFields {
    /// Issue type of the related issue
    #[serde(default)]
    pub issuetype: Option<JiraIssueTypeRef>,
}

/// Keys of issues that block this one: `Blocks` links whose blocker is on the inward side.
fn jira_blocked_by(links: &[JiraIssueLink]) -> Vec<String> {
    links
        .iter()
        .filter(|l| {
            l.link_type.name.eq_ignore_ascii_case("blocks")
                || l.link_type
                    .inward
                    .to_ascii_lowercase()
                    .contains("blocked by")
        })
        .filter_map(|l| l.inward_issue.as_ref().map(|i| i.key.clone()))
        .collect()
}

/// The parent as an epic when its issue type is `Epic`, otherwise a plain parent issue.
fn jira_parent(parent: Option<&JiraLinkedIssue>) -> Option<super::ExternalParent> {
    parent.map(|p| {
        let is_epic = p
            .fields
            .as_ref()
            .and_then(|f| f.issuetype.as_ref())
            .is_some_and(|t| t.name.eq_ignore_ascii_case("epic"));
        super::ExternalParent {
            key: p.key.clone(),
            kind: if is_epic {
                super::ParentKind::Epic
            } else {
                super::ParentKind::Issue
            },
        }
    })
}

/// Normalize a Jira search result into an `ExternalIssue`.
fn issue_from_jira(domain: &str, issue: JiraIssue) -> ExternalIssue {
    let fields = issue.fields;
    ExternalIssue {
        url: format!("https://{domain}/browse/{}", issue.key),
        id: issue.id,
        key: issue.key,
        summary: fields.summary,
        description: extract_description_text(&fields.description),
        kanban_issue_types: vec![KanbanIssueTypeRef {
            id: fields.issuetype.id.unwrap_or_default(),
            name: fields.issuetype.name,
        }],
        status: fields.status.name,
        assignee: fields.assignee.map(|u| ExternalUser {
            id: u.account_id,
            name: u.display_name,
            email: u.email_address,
            avatar_url: u.avatar_urls.and_then(|a| a.large),
        }),
        priority: fields.priority.map(|p| p.name),
        blocked_by: jira_blocked_by(&fields.issuelinks),
        parent: jira_parent(fields.parent.as_ref()),
        subtasks: fields.subtasks.into_iter().map(|s| s.key).collect(),
    }
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
    /// Issue type ID (e.g., "10001")
    #[serde(default)]
    pub id: Option<String>,
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
        let path = format!("/issue/createmeta/{project_key}/issuetypes");
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
        let path = format!("/user/assignable/search?project={project_key}");
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
        let path = format!("/project/{project_key}/statuses");
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
        let jql = format!(
            "project = \"{project_key}\" AND assignee = \"{user_id}\"{}",
            status_clause(statuses)
        );
        let response: JiraSearchResponse = self.get(&search_path(&jql, None)).await?;

        Ok(response
            .issues
            .into_iter()
            .map(|issue| issue_from_jira(&self.domain, issue))
            .collect())
    }

    async fn list_scope(
        &self,
        scope: &crate::config::SyncScope,
        statuses: &[String],
    ) -> Result<super::ScopeContents, ApiError> {
        let crate::config::SyncScope::JiraEpic(key) = scope else {
            return Err(ApiError::http(
                "jira",
                400,
                format!("jira cannot sync scope {scope:?}"),
            ));
        };
        let header: JiraIssueHeader = self
            .get(&format!("/issue/{key}?fields=summary,description"))
            .await?;
        let jql = scope_jql(key, statuses);
        let mut issues = Vec::new();
        let mut token: Option<String> = None;
        for _ in 0..MAX_SEARCH_PAGES {
            let page: JiraSearchResponse = self.get(&search_path(&jql, token.as_deref())).await?;
            issues.extend(page.issues);
            token = page.next_page_token;
            if token.is_none() {
                break;
            }
        }
        Ok(super::ScopeContents {
            url: format!("https://{}/browse/{}", self.domain, header.key),
            key: header.key,
            title: header.fields.summary,
            description: extract_description_text(&header.fields.description),
            issues: issues
                .into_iter()
                .map(|issue| issue_from_jira(&self.domain, issue))
                .collect(),
        })
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
        let path = format!("/issue/{issue_key}/transitions");
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
        let transitions_path = format!("/issue/{issue_key}/transitions");
        self.post_no_content(&transitions_path, &transition_body)
            .await?;

        // Fetch and return the updated issue
        self.fetch_issue(issue_key).await
    }

    async fn update_issue_labels(
        &self,
        issue_key: &str,
        labels: &[String],
    ) -> Result<(), ApiError> {
        // Fetch current labels using a labels-only fields query
        #[derive(Deserialize)]
        struct LabelFields {
            #[serde(default)]
            labels: Vec<String>,
        }
        #[derive(Deserialize)]
        struct LabelIssue {
            fields: LabelFields,
        }

        let path = format!("/issue/{issue_key}?fields=labels");
        let current: LabelIssue = self.get(&path).await?;

        // Merge: existing labels + new labels (deduplicated)
        let mut merged: Vec<String> = current.fields.labels;
        for label in labels {
            if !merged.contains(label) {
                merged.push(label.clone());
            }
        }

        // PUT the merged label set back
        let body = serde_json::json!({
            "fields": { "labels": merged }
        });
        let put_path = format!("/issue/{issue_key}");
        self.put_no_content(&put_path, &body).await
    }

    async fn append_activity_log(
        &self,
        issue_key: &str,
        entry: &super::ActivityLogEntry,
    ) -> Result<(), ApiError> {
        // Format the comment text
        let timestamp = entry.completed_at.format("%Y-%m-%d %H:%M UTC").to_string();
        let mut text = format!(
            "🤖 opr8r - step: {} | delegator: {} | {}",
            entry.step, entry.delegator, timestamp
        );
        if let Some(ref summary) = entry.summary {
            text.push('\n');
            text.push_str(summary);
        }

        // Build ADF comment body
        let body = serde_json::json!({
            "body": {
                "type": "doc",
                "version": 1,
                "content": [{
                    "type": "paragraph",
                    "content": [{
                        "type": "text",
                        "text": text
                    }]
                }]
            }
        });

        let path = format!("/issue/{issue_key}/comment");
        let _: serde_json::Value = self.post(&path, &body).await?;
        Ok(())
    }
}

#[async_trait]
impl super::onboarding::KanbanOnboarding for JiraProvider {
    fn provider_kind(&self) -> super::KanbanProviderType {
        super::KanbanProviderType::Jira
    }

    async fn validate_onboarding(&self) -> Result<super::onboarding::ValidatedWorkspace, ApiError> {
        let details = self.validate_detailed().await?;
        Ok(super::onboarding::ValidatedWorkspace {
            provider_kind: super::KanbanProviderType::Jira,
            workspace_key: self.domain.clone(),
            workspace_display_name: self.domain.clone(),
            sync_user_id: details.account_id,
            sync_user_display_name: details.display_name,
            api_key_env: "OPERATOR_JIRA_API_KEY".to_string(),
            prefetched_projects: None,
            extra: super::onboarding::WorkspaceExtra::Jira {
                email: self.email.clone(),
            },
        })
    }

    async fn discover_projects(
        &self,
        workspace: &super::onboarding::ValidatedWorkspace,
    ) -> Result<Vec<super::onboarding::DiscoveredProject>, ApiError> {
        let projects = self.list_projects().await?;
        Ok(projects
            .into_iter()
            .map(|p| super::onboarding::DiscoveredProject {
                workspace_key: workspace.workspace_key.clone(),
                project_key: p.key,
                project_display_name: p.name,
                provider_url: Some(format!(
                    "https://{}/browse/{}",
                    workspace.workspace_key, p.id
                )),
                provider_native_id: Some(p.id),
            })
            .collect())
    }
}

/// Issues normalized from the checked-in relationship fixture.
#[cfg(test)]
pub(crate) fn fixture_issues() -> Vec<ExternalIssue> {
    let body =
        include_str!("../../../../tests/fixtures/kanban/jira/search_with_relationships.json");
    let response: JiraSearchResponse = serde_json::from_str(body).unwrap();
    response
        .issues
        .into_iter()
        .map(|i| issue_from_jira("acme.atlassian.net", i))
        .collect()
}

/// The `PROJ-1` epic and its six children from the checked-in fixture.
#[cfg(test)]
pub(crate) fn epic_fixture() -> super::ScopeContents {
    let body = include_str!("../../../../tests/fixtures/kanban/jira/epic_children.json");
    let response: JiraSearchResponse = serde_json::from_str(body).unwrap();
    super::ScopeContents {
        key: "PROJ-1".into(),
        title: "Pricing overhaul".into(),
        description: Some("Ship v2 pricing.".into()),
        url: "https://acme.atlassian.net/browse/PROJ-1".into(),
        issues: response
            .issues
            .into_iter()
            .map(|i| issue_from_jira("acme.atlassian.net", i))
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_path_round_trips_jql() {
        let jql = r#"project = "OPS" AND status IN ("To Do","In Progress")"#;
        let path = search_path(jql, Some("tok"));
        let query = path.strip_prefix("/search/jql?").unwrap();
        let pairs: Vec<(String, String)> = url::form_urlencoded::parse(query.as_bytes())
            .into_owned()
            .collect();
        assert_eq!(
            pairs,
            vec![
                ("jql".to_string(), jql.to_string()),
                ("maxResults".to_string(), "100".to_string()),
                ("fields".to_string(), SEARCH_FIELDS.to_string()),
                ("nextPageToken".to_string(), "tok".to_string()),
            ]
        );
    }

    #[test]
    fn test_scope_jql_drops_assignee_and_keeps_statuses() {
        assert_eq!(scope_jql("PROJ-1", &[]), r#"parent = "PROJ-1""#);
        assert_eq!(
            scope_jql("PROJ-1", &["To Do".to_string()]),
            r#"parent = "PROJ-1" AND status IN ("To Do")"#
        );
    }

    #[test]
    fn test_search_response_reads_next_page_token() {
        let page: JiraSearchResponse =
            serde_json::from_str(r#"{"issues": [], "nextPageToken": "abc"}"#).unwrap();
        assert_eq!(page.next_page_token.as_deref(), Some("abc"));
        let last: JiraSearchResponse = serde_json::from_str(r#"{"issues": []}"#).unwrap();
        assert!(last.next_page_token.is_none());
    }

    #[test]
    fn test_search_fields_request_relationships() {
        for field in ["issuelinks", "parent", "subtasks", "summary", "status"] {
            assert!(SEARCH_FIELDS.split(',').any(|f| f == field), "{field}");
        }
    }

    #[test]
    fn test_issue_from_jira_maps_inward_blocks_links_only() {
        let issue = &fixture_issues()[0];
        assert_eq!(issue.blocked_by, vec!["PROJ-12"]);
        assert_eq!(issue.url, "https://acme.atlassian.net/browse/PROJ-42");
    }

    #[test]
    fn test_issue_from_jira_maps_epic_parent_and_subtasks() {
        let issue = &fixture_issues()[0];
        assert_eq!(
            issue.parent,
            Some(super::super::ExternalParent {
                key: "PROJ-1".into(),
                kind: super::super::ParentKind::Epic,
            })
        );
        assert_eq!(issue.subtasks, vec!["PROJ-43", "PROJ-44"]);
    }

    #[test]
    fn test_issue_from_jira_without_links_defaults_empty() {
        let issue = &fixture_issues()[1];
        assert!(issue.blocked_by.is_empty());
        assert!(issue.subtasks.is_empty());
        assert_eq!(
            issue.parent.as_ref().map(|p| p.kind),
            Some(super::super::ParentKind::Issue)
        );
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
    fn test_issue_type_ref_has_id() {
        // JiraIssueTypeRef now includes optional id for kanban issue type refs
        let json = r#"{"id": "10001", "name": "Bug"}"#;
        let type_ref: JiraIssueTypeRef = serde_json::from_str(json).unwrap();
        assert_eq!(type_ref.id, Some("10001".to_string()));
        assert_eq!(type_ref.name, "Bug");
    }

    #[test]
    fn test_issue_type_ref_no_id() {
        // id is optional for backward compatibility
        let json = r#"{"name": "Bug"}"#;
        let type_ref: JiraIssueTypeRef = serde_json::from_str(json).unwrap();
        assert_eq!(type_ref.id, None);
        assert_eq!(type_ref.name, "Bug");
    }
}

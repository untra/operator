use std::collections::{HashMap, HashSet};
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

use async_trait::async_trait;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use url::Url;

use super::{
    ActivityLogEntry, CreateIssueRequest, CreateIssueResponse, ExternalIssue, ExternalIssueType,
    ExternalUser, KanbanProvider, ProjectInfo, UpdateStatusRequest,
};
use crate::api::error::ApiError;
use crate::config::GiteaProjectsConfig;
use crate::issuetypes::kanban_type::KanbanIssueTypeRef;

const PROVIDER: &str = "gitea";
const PAGE_SIZE: usize = 50;
const MAX_PAGES: usize = 1000;
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
const READ_ONLY: &str = "Gitea Projects Alpha is import-only; upstream writes are unsupported";
type SessionTokens = Mutex<HashMap<(String, String), String>>;
static SESSION_TOKENS: OnceLock<SessionTokens> = OnceLock::new();

pub struct GiteaProjectsProvider {
    base: Url,
    repositories: Vec<String>,
    token: String,
    client: reqwest::Client,
}

#[derive(Deserialize)]
struct Project {
    id: u64,
    title: String,
}

#[derive(Deserialize)]
struct Column {
    id: u64,
    title: String,
    #[serde(default)]
    sorting: i64,
    #[serde(default)]
    default: bool,
}

#[derive(Clone, Deserialize)]
struct User {
    id: u64,
    login: String,
    #[serde(default)]
    full_name: String,
    email: Option<String>,
    avatar_url: Option<String>,
}

impl From<User> for ExternalUser {
    fn from(user: User) -> Self {
        Self {
            id: user.id.to_string(),
            name: if user.full_name.is_empty() {
                user.login
            } else {
                user.full_name
            },
            email: user.email.filter(|s| !s.is_empty()),
            avatar_url: user.avatar_url,
        }
    }
}

#[derive(Deserialize)]
struct Label {
    id: u64,
    name: String,
    description: Option<String>,
}

#[derive(Deserialize)]
struct Issue {
    number: u64,
    title: String,
    body: Option<String>,
    #[serde(default)]
    labels: Vec<Label>,
    #[serde(default)]
    assignees: Option<Vec<User>>,
    assignee: Option<User>,
    pull_request: Option<serde_json::Value>,
}

fn invalid(message: impl Into<String>) -> ApiError {
    ApiError::http(PROVIDER, 400, message)
}

fn base_url(value: &str) -> Result<Url, ApiError> {
    let mut url = Url::parse(value).map_err(|_| invalid("Invalid Gitea instance URL"))?;
    if !matches!(url.scheme(), "https" | "http")
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err(invalid(
            "Gitea instance must be an HTTP(S) URL without credentials, query or fragment",
        ));
    }
    if !url.path().ends_with('/') {
        url.set_path(&format!("{}/", url.path()));
    }
    Ok(url)
}

fn repository_parts(repository: &str) -> Result<(&str, &str), ApiError> {
    let parts: Vec<_> = repository.split('/').collect();
    if parts.len() != 2
        || parts
            .iter()
            .any(|p| p.is_empty() || *p == "." || *p == ".." || p.contains(['%', '?', '#', '\\']))
    {
        return Err(invalid("Repository must be owner/name"));
    }
    Ok((parts[0], parts[1]))
}

fn board_path(base: &Url, repositories: &[String], key: &str) -> Result<String, ApiError> {
    let relative = key
        .strip_prefix(base.as_str())
        .ok_or_else(|| invalid("Board does not belong to this Gitea instance"))?;
    let parts: Vec<_> = relative.split('/').collect();
    if parts.len() != 4
        || parts[2] != "projects"
        || parts[3].parse::<u64>().ok().is_none_or(|id| id == 0)
    {
        return Err(invalid("Expected an absolute repository project URL"));
    }
    let repository = format!("{}/{}", parts[0], parts[1]);
    repository_parts(&repository)?;
    if !repositories.contains(&repository) {
        return Err(invalid(
            "Board repository is not selected for this instance",
        ));
    }
    Ok(format!("repos/{relative}"))
}

impl GiteaProjectsProvider {
    pub fn new(base: &str, repositories: Vec<String>, token: String) -> Result<Self, ApiError> {
        let base = base_url(base)?;
        if token.trim().is_empty() {
            return Err(ApiError::not_configured(PROVIDER));
        }
        if repositories.is_empty() {
            return Err(invalid("Select at least one Gitea repository"));
        }
        for repo in &repositories {
            repository_parts(repo)?;
        }
        let client = crate::http_client::client_builder()
            .timeout(REQUEST_TIMEOUT)
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|_| ApiError::network(PROVIDER, "Cannot construct HTTP client"))?;
        Ok(Self {
            base,
            repositories,
            token,
            client,
        })
    }

    pub fn validate_config(cfg: &GiteaProjectsConfig) -> Result<(), ApiError> {
        let base = base_url(&cfg.base_url)?;
        if cfg.repositories.is_empty() {
            return Err(invalid("Select at least one Gitea repository"));
        }
        for repository in &cfg.repositories {
            repository_parts(repository)?;
        }
        for (key, project) in &cfg.projects {
            board_path(&base, &cfg.repositories, key)?;
            if project.bidirectional {
                return Err(invalid(READ_ONLY));
            }
            if project.scope.is_some() {
                return Err(invalid(
                    "Gitea Alpha supports repository boards, not campaign scopes",
                ));
            }
        }
        Ok(())
    }

    pub fn from_config(cfg: &GiteaProjectsConfig) -> Result<Self, ApiError> {
        Self::validate_config(cfg)?;
        let base = base_url(&cfg.base_url)?.to_string();
        let token = SESSION_TOKENS
            .get_or_init(Mutex::default)
            .lock()
            .map_err(|_| invalid("Credential store unavailable"))?
            .get(&(base, cfg.api_key_env.clone()))
            .cloned()
            .or_else(|| std::env::var(&cfg.api_key_env).ok())
            .ok_or_else(|| ApiError::not_configured(PROVIDER))?;
        Self::new(&cfg.base_url, cfg.repositories.clone(), token)
    }

    pub fn set_session_token(base: &str, env: &str, token: String) -> Result<(), ApiError> {
        if env.is_empty()
            || !env
                .bytes()
                .enumerate()
                .all(|(i, c)| c == b'_' || c.is_ascii_alphabetic() || (i > 0 && c.is_ascii_digit()))
        {
            return Err(invalid("Invalid token environment variable name"));
        }
        if token.trim().is_empty() {
            return Err(ApiError::not_configured(PROVIDER));
        }
        SESSION_TOKENS
            .get_or_init(Mutex::default)
            .lock()
            .map_err(|_| invalid("Credential store unavailable"))?
            .insert((base_url(base)?.to_string(), env.into()), token);
        Ok(())
    }

    fn project_key(&self, repository: &str, id: u64) -> Result<String, ApiError> {
        repository_parts(repository)?;
        Ok(self
            .base
            .join(&format!("{repository}/projects/{id}"))
            .map_err(|_| invalid("Invalid project URL"))?
            .to_string())
    }

    fn board_path(&self, key: &str) -> Result<String, ApiError> {
        board_path(&self.base, &self.repositories, key)
    }

    async fn get<T: DeserializeOwned>(
        &self,
        path: &str,
        page: Option<usize>,
    ) -> Result<T, ApiError> {
        let url = self
            .base
            .join(&format!("api/v1/{path}"))
            .map_err(|_| invalid("Invalid API path"))?;
        let mut request = self.client.get(url).header(
            reqwest::header::AUTHORIZATION,
            format!("token {}", self.token),
        );
        if let Some(page) = page {
            request = request.query(&[("page", page), ("limit", PAGE_SIZE)]);
        }
        let response = request
            .send()
            .await
            .map_err(|_| ApiError::network(PROVIDER, "Gitea request failed or timed out"))?;
        match response.status().as_u16() {
            200..=299 => response
                .json()
                .await
                .map_err(|_| ApiError::http(PROVIDER, 502, "Invalid Gitea API response")),
            401 => Err(ApiError::unauthorized(PROVIDER)),
            403 => Err(ApiError::forbidden(PROVIDER)),
            429 => Err(ApiError::rate_limited(
                PROVIDER,
                response
                    .headers()
                    .get("retry-after")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|v| v.parse().ok()),
            )),
            404 => Err(ApiError::http(
                PROVIDER,
                404,
                "Repository or board is inaccessible, or this Gitea version lacks project APIs",
            )),
            status => Err(ApiError::http(PROVIDER, status, "Gitea API request failed")),
        }
    }

    async fn pages<T: DeserializeOwned>(&self, path: &str) -> Result<Vec<T>, ApiError> {
        let mut items = Vec::new();
        for page in 1..=MAX_PAGES {
            let batch: Vec<T> = self.get(path, Some(page)).await?;
            if batch.is_empty() {
                return Ok(items);
            }
            items.extend(batch);
        }
        Err(ApiError::http(
            PROVIDER,
            502,
            "Gitea pagination limit exceeded",
        ))
    }

    async fn columns(&self, key: &str) -> Result<Vec<Column>, ApiError> {
        let path = self.board_path(key)?;
        let mut columns: Vec<Column> = self.pages(&format!("{path}/columns")).await?;
        columns.sort_by_key(|c| (!c.default, c.sorting, c.id));
        let mut names = HashSet::new();
        if columns
            .iter()
            .any(|c| c.title.is_empty() || !names.insert(c.title.clone()))
        {
            return Err(invalid(
                "Board columns must have distinct nonempty names for status mapping",
            ));
        }
        Ok(columns)
    }

    pub async fn authenticated_user(&self) -> Result<ExternalUser, ApiError> {
        self.get::<User>("user", None).await.map(Into::into)
    }
}

#[async_trait]
impl KanbanProvider for GiteaProjectsProvider {
    fn name(&self) -> &str {
        PROVIDER
    }
    fn is_configured(&self) -> bool {
        !self.token.is_empty()
    }

    async fn test_connection(&self) -> Result<bool, ApiError> {
        self.authenticated_user().await?;
        for project in self.list_projects().await? {
            self.columns(&project.key).await?;
        }
        for repo in &self.repositories {
            let _: Vec<Label> = self.get(&format!("repos/{repo}/labels"), Some(1)).await?;
        }
        Ok(true)
    }

    async fn list_projects(&self) -> Result<Vec<ProjectInfo>, ApiError> {
        let mut projects = Vec::new();
        for repo in &self.repositories {
            for project in self
                .pages::<Project>(&format!("repos/{repo}/projects"))
                .await?
            {
                let key = self.project_key(repo, project.id)?;
                projects.push(ProjectInfo {
                    id: key.clone(),
                    key,
                    name: format!("{repo}: {}", project.title),
                });
            }
        }
        Ok(projects)
    }

    async fn get_issue_types(&self, key: &str) -> Result<Vec<ExternalIssueType>, ApiError> {
        let path = self.board_path(key)?;
        let repo = path
            .split("/projects/")
            .next()
            .ok_or_else(|| invalid("Invalid project"))?;
        Ok(self
            .pages::<Label>(&format!("{repo}/labels"))
            .await?
            .into_iter()
            .map(|label| ExternalIssueType {
                id: label.id.to_string(),
                name: label.name,
                description: label.description,
                icon_url: None,
                custom_fields: vec![],
            })
            .collect())
    }

    async fn list_users(&self, key: &str) -> Result<Vec<ExternalUser>, ApiError> {
        let path = self.board_path(key)?;
        let repo = path
            .split("/projects/")
            .next()
            .ok_or_else(|| invalid("Invalid project"))?;
        Ok(self
            .get::<Vec<User>>(&format!("{repo}/assignees"), None)
            .await?
            .into_iter()
            .map(Into::into)
            .collect())
    }

    async fn list_statuses(&self, key: &str) -> Result<Vec<String>, ApiError> {
        Ok(self
            .columns(key)
            .await?
            .into_iter()
            .map(|c| c.title)
            .collect())
    }

    async fn list_issues(
        &self,
        key: &str,
        user_id: &str,
        statuses: &[String],
    ) -> Result<Vec<ExternalIssue>, ApiError> {
        if user_id.is_empty() {
            return Err(invalid("A sync user ID is required"));
        }
        let path = self.board_path(key)?;
        let repo = path
            .strip_prefix("repos/")
            .and_then(|p| p.split("/projects/").next())
            .ok_or_else(|| invalid("Invalid project"))?;
        let columns = self.columns(key).await?;
        if statuses
            .iter()
            .any(|s| !columns.iter().any(|c| &c.title == s))
        {
            return Err(invalid(
                "A mapped column no longer exists; update the status mapping",
            ));
        }
        let mut issues = Vec::new();
        let mut seen = HashSet::new();
        for (index, column) in columns.iter().enumerate() {
            if if statuses.is_empty() {
                index != 0
            } else {
                !statuses.contains(&column.title)
            } {
                continue;
            }
            for issue in self
                .pages::<Issue>(&format!("{path}/columns/{}/issues", column.id))
                .await?
            {
                if issue.pull_request.is_some() {
                    continue;
                }
                let user = issue
                    .assignees
                    .unwrap_or_default()
                    .into_iter()
                    .chain(issue.assignee)
                    .find(|u| u.id.to_string() == user_id);
                if user.is_none() {
                    continue;
                }
                let url = self
                    .base
                    .join(&format!("{repo}/issues/{}", issue.number))
                    .map_err(|_| invalid("Invalid issue URL"))?
                    .to_string();
                if !seen.insert(url.clone()) {
                    continue;
                }
                issues.push(ExternalIssue {
                    id: url.clone(),
                    key: url.clone(),
                    summary: issue.title,
                    description: issue.body,
                    kanban_issue_types: issue
                        .labels
                        .into_iter()
                        .map(|l| KanbanIssueTypeRef {
                            id: l.id.to_string(),
                            name: l.name,
                        })
                        .collect(),
                    status: column.title.clone(),
                    assignee: user.map(Into::into),
                    url,
                    priority: None,
                    blocked_by: vec![],
                    parent: None,
                    subtasks: vec![],
                });
            }
        }
        Ok(issues)
    }

    async fn create_issue(
        &self,
        _: &str,
        _: CreateIssueRequest,
    ) -> Result<CreateIssueResponse, ApiError> {
        Err(invalid(READ_ONLY))
    }
    async fn update_issue_status(
        &self,
        _: &str,
        _: UpdateStatusRequest,
    ) -> Result<ExternalIssue, ApiError> {
        Err(invalid(READ_ONLY))
    }
    async fn update_issue_labels(&self, _: &str, _: &[String]) -> Result<(), ApiError> {
        Err(invalid(READ_ONLY))
    }
    async fn append_activity_log(&self, _: &str, _: &ActivityLogEntry) -> Result<(), ApiError> {
        Err(invalid(READ_ONLY))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        extract::Request, http::StatusCode, response::IntoResponse, routing::get, Json, Router,
    };
    use serde_json::json;

    async fn fixture(request: Request) -> impl IntoResponse {
        if request.headers().get("authorization").unwrap() != "token board-token" {
            return (StatusCode::UNAUTHORIZED, Json(json!({})));
        }
        if request.uri().path() == "/api/v1/incomplete" {
            return if request.uri().query().is_some_and(|q| q.contains("page=2")) {
                (StatusCode::SERVICE_UNAVAILABLE, Json(json!({})))
            } else {
                (StatusCode::OK, Json(json!([{"id": 1}])))
            };
        }
        if !request.uri().path().ends_with("/assignees")
            && request.uri().query().is_some_and(|q| q.contains("page=2"))
        {
            return (StatusCode::OK, Json(json!([])));
        }
        let issue = json!({"number": 7, "title": "Assigned task", "body": "details", "labels": [], "assignees": [{"id": 4, "login": "agent"}]});
        let body = match request.uri().path() {
            "/api/v1/forbidden" => return (StatusCode::FORBIDDEN, Json(json!({}))),
            "/api/v1/limited" => return (StatusCode::TOO_MANY_REQUESTS, Json(json!({}))),
            "/api/v1/user" => json!({"id":4,"login":"agent"}),
            "/api/v1/repos/team/repo/projects" => json!([{"id":1,"title":"Work"}]),
            "/api/v1/repos/team/repo/projects/1/columns" => {
                json!([{"id":2,"title":"Todo","sorting":0,"default":true}, {"id":3,"title":"Done","sorting":1}])
            }
            "/api/v1/repos/team/repo/projects/1/columns/2/issues" => {
                json!([issue, issue, {"number":8,"title":"PR","pull_request":{},"assignees":[{"id":4,"login":"agent"}]}, {"number":9,"title":"Other user","assignees":[{"id":5,"login":"other"}]}])
            }
            "/api/v1/repos/team/repo/assignees" => json!([{"id":4,"login":"agent"}]),
            "/api/v1/repos/team/repo/labels" => json!([]),
            _ => return (StatusCode::NOT_FOUND, Json(json!({}))),
        };
        (StatusCode::OK, Json(body))
    }

    #[tokio::test]
    async fn test_import_filters_assignee_and_prs_and_deduplicates() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let server = tokio::spawn(async move {
            axum::serve(listener, Router::new().route("/{*path}", get(fixture)))
                .await
                .unwrap();
        });
        let provider =
            GiteaProjectsProvider::new(&base, vec!["team/repo".into()], "board-token".into())
                .unwrap();
        assert!(provider.test_connection().await.unwrap());
        let board = provider.list_projects().await.unwrap().remove(0);
        assert_eq!(
            provider.list_statuses(&board.key).await.unwrap(),
            ["Todo", "Done"]
        );
        assert_eq!(provider.list_users(&board.key).await.unwrap()[0].id, "4");
        let issues = provider.list_issues(&board.key, "4", &[]).await.unwrap();
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].id, format!("{base}/team/repo/issues/7"));
        assert!(provider
            .list_issues(&board.key, "4", &["Removed".into()])
            .await
            .is_err());
        assert!(provider
            .update_issue_status(
                &issues[0].key,
                UpdateStatusRequest {
                    status: "Done".into()
                }
            )
            .await
            .is_err());
        assert!(provider
            .update_issue_labels(&issues[0].key, &["bug".into()])
            .await
            .is_err());
        let invalid =
            GiteaProjectsProvider::new(&base, vec!["team/repo".into()], "wrong-token".into())
                .unwrap();
        assert!(matches!(
            invalid.test_connection().await,
            Err(ApiError::Unauthorized { .. })
        ));
        assert!(matches!(
            provider.get::<serde_json::Value>("forbidden", None).await,
            Err(ApiError::Forbidden { .. })
        ));
        assert!(matches!(
            provider.get::<serde_json::Value>("limited", None).await,
            Err(ApiError::RateLimited { .. })
        ));
        assert!(matches!(
            provider.get::<serde_json::Value>("missing", None).await,
            Err(ApiError::HttpError { status: 404, .. })
        ));
        assert!(provider
            .pages::<serde_json::Value>("incomplete")
            .await
            .is_err());
        server.abort();
    }

    #[test]
    fn test_board_identity_rejects_other_instance_or_repository() {
        let provider = GiteaProjectsProvider::new(
            "https://forge.example/prefix/",
            vec!["team/repo".into()],
            "token".into(),
        )
        .unwrap();
        assert!(provider
            .board_path("https://forge.example/prefix/team/repo/projects/1")
            .is_ok());
        for key in [
            "https://other.example/prefix/team/repo/projects/1",
            "https://forge.example/prefix/other/repo/projects/1",
            "https://forge.example/prefix/team/repo/projects/1?token=secret",
        ] {
            assert!(provider.board_path(key).is_err());
        }
    }

    #[test]
    fn test_session_credentials_are_bound_to_instance_and_do_not_mutate_env() {
        const ENV: &str = "OPERATOR_GITEA_INSTANCE_ISOLATION_TEST";
        let before = std::env::var(ENV).ok();
        GiteaProjectsProvider::set_session_token(
            "https://first.example",
            ENV,
            "first-secret".into(),
        )
        .unwrap();
        GiteaProjectsProvider::set_session_token(
            "https://second.example",
            ENV,
            "second-secret".into(),
        )
        .unwrap();
        for (base, expected) in [
            ("https://first.example", "first-secret"),
            ("https://second.example", "second-secret"),
        ] {
            let cfg = GiteaProjectsConfig {
                enabled: true,
                base_url: base.into(),
                repositories: vec!["team/repo".into()],
                api_key_env: ENV.into(),
                projects: HashMap::new(),
            };
            assert_eq!(
                GiteaProjectsProvider::from_config(&cfg).unwrap().token,
                expected
            );
        }
        assert_eq!(std::env::var(ENV).ok(), before);
        assert!(GiteaProjectsProvider::set_session_token(
            "https://first.example",
            "TOKEN; command",
            "secret".into()
        )
        .is_err());
    }

    #[test]
    fn test_config_rejects_writes_without_reading_credentials() {
        let mut cfg = GiteaProjectsConfig {
            enabled: true,
            base_url: "https://forge.example".into(),
            repositories: vec!["team/repo".into()],
            api_key_env: "MISSING_TOKEN".into(),
            projects: HashMap::new(),
        };
        cfg.projects.insert(
            "https://forge.example/team/repo/projects/1".into(),
            crate::config::ProjectSyncConfig {
                bidirectional: true,
                ..Default::default()
            },
        );
        assert!(GiteaProjectsProvider::validate_config(&cfg)
            .unwrap_err()
            .to_string()
            .contains("import-only"));
    }
}

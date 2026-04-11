//! Kanban onboarding service.
//!
//! Owns validation / project listing / config writing / session env setup
//! for Jira, Linear, and GitHub Projects onboarding flows. Both the REST API
//! handlers and the TUI onboarding dialog call the same functions here so
//! there's a single source of truth for config mutation.

use std::path::PathBuf;

use tracing::info;

use crate::api::providers::kanban::{GithubProjectsProvider, JiraProvider, LinearProvider};
use crate::config::Config;
use crate::rest::dto::{
    GithubProjectInfoDto, GithubValidationDetailsDto, JiraValidationDetailsDto, KanbanProjectInfo,
    KanbanProviderKind, LinearTeamInfoDto, LinearValidationDetailsDto, ListKanbanProjectsRequest,
    ListKanbanProjectsResponse, SetKanbanSessionEnvRequest, SetKanbanSessionEnvResponse,
    ValidateKanbanCredentialsRequest, ValidateKanbanCredentialsResponse, WriteKanbanConfigRequest,
    WriteKanbanConfigResponse,
};
use crate::rest::error::ApiError;

// ─── Error helpers ──────────────────────────────────────────────────────────

/// Map a provider-layer `ApiError` into a human-readable string for inline
/// display in client UIs.
fn provider_error_message(err: &crate::api::error::ApiError) -> String {
    use crate::api::error::ApiError as ProviderErr;
    match err {
        ProviderErr::Unauthorized { .. } => {
            "Invalid credentials (401). Check your email/domain and API token.".to_string()
        }
        ProviderErr::Forbidden { .. } => {
            "Access forbidden (403). Token may lack required permissions.".to_string()
        }
        ProviderErr::RateLimited { .. } => {
            "Rate limited by provider. Please try again in a moment.".to_string()
        }
        ProviderErr::NetworkError { message, .. } => {
            format!("Network error: {message}")
        }
        ProviderErr::HttpError {
            status, message, ..
        } => {
            format!("Provider HTTP {status}: {message}")
        }
        ProviderErr::NotConfigured { .. } => "Provider not configured.".to_string(),
    }
}

// ─── validate_credentials ───────────────────────────────────────────────────

/// Validate credentials against the live provider API without persisting
/// anything or mutating any state.
pub async fn validate_credentials(
    req: ValidateKanbanCredentialsRequest,
) -> Result<ValidateKanbanCredentialsResponse, ApiError> {
    match req.provider {
        KanbanProviderKind::Jira => {
            let creds = req.jira.ok_or_else(|| {
                ApiError::BadRequest("Missing `jira` field for jira provider".to_string())
            })?;
            let provider =
                JiraProvider::new(creds.domain.clone(), creds.email.clone(), creds.api_token);

            match provider.validate_detailed().await {
                Ok(details) => Ok(ValidateKanbanCredentialsResponse {
                    valid: true,
                    error: None,
                    jira: Some(JiraValidationDetailsDto {
                        account_id: details.account_id,
                        display_name: details.display_name,
                    }),
                    linear: None,
                    github: None,
                }),
                Err(e) => Ok(ValidateKanbanCredentialsResponse {
                    valid: false,
                    error: Some(provider_error_message(&e)),
                    jira: None,
                    linear: None,
                    github: None,
                }),
            }
        }
        KanbanProviderKind::Linear => {
            let creds = req.linear.ok_or_else(|| {
                ApiError::BadRequest("Missing `linear` field for linear provider".to_string())
            })?;
            let provider = LinearProvider::new(creds.api_key);

            match provider.validate_detailed().await {
                Ok(details) => Ok(ValidateKanbanCredentialsResponse {
                    valid: true,
                    error: None,
                    jira: None,
                    linear: Some(LinearValidationDetailsDto {
                        user_id: details.user_id,
                        user_name: details.user_name,
                        org_name: details.org_name,
                        teams: details
                            .teams
                            .into_iter()
                            .map(|t| LinearTeamInfoDto {
                                id: t.id,
                                key: t.key,
                                name: t.name,
                            })
                            .collect(),
                    }),
                    github: None,
                }),
                Err(e) => Ok(ValidateKanbanCredentialsResponse {
                    valid: false,
                    error: Some(provider_error_message(&e)),
                    jira: None,
                    linear: None,
                    github: None,
                }),
            }
        }
        KanbanProviderKind::Github => {
            let creds = req.github.ok_or_else(|| {
                ApiError::BadRequest("Missing `github` field for github provider".to_string())
            })?;
            // Onboarding always uses an ephemeral session token; the env var
            // it would land in defaults to OPERATOR_GITHUB_TOKEN unless the
            // client overrode it via /api/v1/kanban/session-env.
            let provider =
                GithubProjectsProvider::new(creds.token, "OPERATOR_GITHUB_TOKEN".to_string());

            match provider.validate_detailed().await {
                Ok(details) => Ok(ValidateKanbanCredentialsResponse {
                    valid: true,
                    error: None,
                    jira: None,
                    linear: None,
                    github: Some(GithubValidationDetailsDto {
                        user_login: details.user_login,
                        user_id: details.user_id,
                        projects: details
                            .projects
                            .into_iter()
                            .map(|p| GithubProjectInfoDto {
                                node_id: p.node_id,
                                number: p.number,
                                title: p.title,
                                owner_login: p.owner_login,
                                owner_kind: p.owner_kind,
                            })
                            .collect(),
                        resolved_env_var: details.resolved_env_var,
                    }),
                }),
                Err(e) => Ok(ValidateKanbanCredentialsResponse {
                    valid: false,
                    error: Some(provider_error_message(&e)),
                    jira: None,
                    linear: None,
                    github: None,
                }),
            }
        }
    }
}

// ─── list_projects ──────────────────────────────────────────────────────────

/// Fetch the list of projects (Jira) or teams (Linear) for the given creds.
pub async fn list_projects(
    req: ListKanbanProjectsRequest,
) -> Result<ListKanbanProjectsResponse, ApiError> {
    use crate::api::providers::kanban::KanbanProvider;

    let projects = match req.provider {
        KanbanProviderKind::Jira => {
            let creds = req.jira.ok_or_else(|| {
                ApiError::BadRequest("Missing `jira` field for jira provider".to_string())
            })?;
            let provider = JiraProvider::new(creds.domain, creds.email, creds.api_token);
            provider
                .list_projects()
                .await
                .map_err(|e| ApiError::BadRequest(provider_error_message(&e)))?
        }
        KanbanProviderKind::Linear => {
            let creds = req.linear.ok_or_else(|| {
                ApiError::BadRequest("Missing `linear` field for linear provider".to_string())
            })?;
            let provider = LinearProvider::new(creds.api_key);
            provider
                .list_projects()
                .await
                .map_err(|e| ApiError::BadRequest(provider_error_message(&e)))?
        }
        KanbanProviderKind::Github => {
            let creds = req.github.ok_or_else(|| {
                ApiError::BadRequest("Missing `github` field for github provider".to_string())
            })?;
            let provider =
                GithubProjectsProvider::new(creds.token, "OPERATOR_GITHUB_TOKEN".to_string());
            provider
                .list_projects()
                .await
                .map_err(|e| ApiError::BadRequest(provider_error_message(&e)))?
        }
    };

    Ok(ListKanbanProjectsResponse {
        projects: projects
            .into_iter()
            .map(|p| KanbanProjectInfo {
                id: p.id,
                key: p.key,
                name: p.name,
            })
            .collect(),
    })
}

// ─── write_config ───────────────────────────────────────────────────────────

/// Write or upsert a kanban config section to `config.toml`.
///
/// `config_override_path` is optional — when `None`, falls back to
/// `Config::operator_config_path()` (which is what production uses).
/// When `Some`, the config is loaded from and saved to that path instead
/// (used by unit tests).
pub fn write_config(
    req: WriteKanbanConfigRequest,
    config_override_path: Option<&PathBuf>,
) -> Result<WriteKanbanConfigResponse, ApiError> {
    // Load existing config (from disk — not from in-memory ApiState, so that
    // concurrent writes don't clobber each other). If load fails, start with
    // a default config.
    let mut config = match config_override_path {
        Some(p) => load_config_from_path(p).unwrap_or_default(),
        None => Config::load(None).unwrap_or_default(),
    };

    let section_header = match req.provider {
        KanbanProviderKind::Jira => {
            let body = req.jira.ok_or_else(|| {
                ApiError::BadRequest("Missing `jira` field for jira provider".to_string())
            })?;
            config.kanban.upsert_jira_project(
                &body.domain,
                &body.email,
                &body.api_key_env,
                &body.project_key,
                &body.sync_user_id,
            );
            format!("[kanban.jira.\"{}\"]", body.domain)
        }
        KanbanProviderKind::Linear => {
            let body = req.linear.ok_or_else(|| {
                ApiError::BadRequest("Missing `linear` field for linear provider".to_string())
            })?;
            config.kanban.upsert_linear_project(
                &body.workspace_key,
                &body.api_key_env,
                &body.project_key,
                &body.sync_user_id,
            );
            format!("[kanban.linear.\"{}\"]", body.workspace_key)
        }
        KanbanProviderKind::Github => {
            let body = req.github.ok_or_else(|| {
                ApiError::BadRequest("Missing `github` field for github provider".to_string())
            })?;
            config.kanban.upsert_github_project(
                &body.owner,
                &body.api_key_env,
                &body.project_key,
                &body.sync_user_id,
            );
            format!("[kanban.github.\"{}\"]", body.owner)
        }
    };

    let written_path = if let Some(p) = config_override_path {
        save_config_to_path(&config, p)
            .map_err(|e| ApiError::InternalError(format!("Failed to save config: {e}")))?;
        p.display().to_string()
    } else {
        config
            .save()
            .map_err(|e| ApiError::InternalError(format!("Failed to save config: {e}")))?;
        Config::operator_config_path().display().to_string()
    };

    info!(section = %section_header, "Wrote kanban config section");

    Ok(WriteKanbanConfigResponse {
        written_path,
        section_header,
    })
}

/// Test-only helper: load a Config from an explicit TOML path.
fn load_config_from_path(path: &PathBuf) -> anyhow::Result<Config> {
    let raw = std::fs::read_to_string(path)?;
    let cfg: Config = toml::from_str(&raw)?;
    Ok(cfg)
}

/// Test-only helper: save a Config to an explicit TOML path.
fn save_config_to_path(config: &Config, path: &PathBuf) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let raw = toml::to_string_pretty(config)?;
    std::fs::write(path, raw)?;
    Ok(())
}

// ─── set_session_env ────────────────────────────────────────────────────────

/// Set kanban-related env vars on the server process for the current session
/// and return a shell export block the client can show to the user for
/// copying into their shell profile.
///
/// Security note: the `shell_export_block` uses `<your-token>` placeholders,
/// NOT the actual secret value supplied in the request. The secret lives
/// only in the process env.
pub fn set_session_env(req: SetKanbanSessionEnvRequest) -> SetKanbanSessionEnvResponse {
    let mut env_vars_set: Vec<String> = Vec::new();

    match req.provider {
        KanbanProviderKind::Jira => {
            if let Some(body) = req.jira {
                // SAFETY: set_var is safe in single-threaded startup contexts;
                // the operator REST server runs inside a tokio runtime, but
                // the set_var pattern is already established in
                // src/app/git_onboarding.rs and src/main.rs. Kanban onboarding
                // is a user-driven one-shot and we accept the same tradeoff.
                std::env::set_var(&body.api_key_env, &body.api_token);
                std::env::set_var("OPERATOR_JIRA_DOMAIN", &body.domain);
                std::env::set_var("OPERATOR_JIRA_EMAIL", &body.email);
                env_vars_set.push(body.api_key_env.clone());
                env_vars_set.push("OPERATOR_JIRA_DOMAIN".to_string());
                env_vars_set.push("OPERATOR_JIRA_EMAIL".to_string());

                let shell_export_block = build_shell_export_block_jira(&body.api_key_env);
                return SetKanbanSessionEnvResponse {
                    env_vars_set,
                    shell_export_block,
                };
            }
        }
        KanbanProviderKind::Linear => {
            if let Some(body) = req.linear {
                std::env::set_var(&body.api_key_env, &body.api_key);
                env_vars_set.push(body.api_key_env.clone());

                let shell_export_block = build_shell_export_block_linear(&body.api_key_env);
                return SetKanbanSessionEnvResponse {
                    env_vars_set,
                    shell_export_block,
                };
            }
        }
        KanbanProviderKind::Github => {
            if let Some(body) = req.github {
                std::env::set_var(&body.api_key_env, &body.token);
                env_vars_set.push(body.api_key_env.clone());

                let shell_export_block = build_shell_export_block_github(&body.api_key_env);
                return SetKanbanSessionEnvResponse {
                    env_vars_set,
                    shell_export_block,
                };
            }
        }
    }

    // No body supplied for the selected provider — return empty envelope.
    SetKanbanSessionEnvResponse {
        env_vars_set,
        shell_export_block: String::new(),
    }
}

/// Build a copy-paste-ready `export` block for Jira's env vars.
///
/// Uses placeholders — never embeds the actual token in the returned
/// string.
pub fn build_shell_export_block_jira(api_key_env: &str) -> String {
    format!("export {api_key_env}=\"<your-jira-api-token>\"")
}

/// Build a copy-paste-ready `export` block for Linear's env var.
///
/// Uses placeholders — never embeds the actual token in the returned
/// string.
pub fn build_shell_export_block_linear(api_key_env: &str) -> String {
    format!("export {api_key_env}=\"<your-linear-api-key>\"")
}

/// Build a copy-paste-ready `export` block for the GitHub Projects token.
///
/// Uses placeholders — never embeds the actual token in the returned string.
/// The placeholder text reminds the user this is the *projects* token, not
/// the repo token used by `GITHUB_TOKEN` (Token Disambiguation rule 4).
pub fn build_shell_export_block_github(api_key_env: &str) -> String {
    format!("export {api_key_env}=\"<your-github-projects-token>\"")
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rest::dto::{WriteGithubConfigBody, WriteJiraConfigBody, WriteLinearConfigBody};
    use tempfile::tempdir;

    #[test]
    fn test_write_config_jira_writes_new_section() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.toml");

        let req = WriteKanbanConfigRequest {
            provider: KanbanProviderKind::Jira,
            jira: Some(WriteJiraConfigBody {
                domain: "acme.atlassian.net".to_string(),
                email: "user@acme.com".to_string(),
                api_key_env: "OPERATOR_JIRA_API_KEY".to_string(),
                project_key: "PROJ".to_string(),
                sync_user_id: "acct-123".to_string(),
            }),
            linear: None,
            github: None,
        };

        let resp = write_config(req, Some(&path)).unwrap();
        assert_eq!(resp.section_header, "[kanban.jira.\"acme.atlassian.net\"]");

        let contents = std::fs::read_to_string(&path).unwrap();
        assert!(contents.contains("acme.atlassian.net"));
        assert!(contents.contains("user@acme.com"));
        assert!(contents.contains("OPERATOR_JIRA_API_KEY"));
        assert!(contents.contains("PROJ"));
        assert!(contents.contains("acct-123"));
    }

    #[test]
    fn test_write_config_linear_writes_new_section() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.toml");

        let req = WriteKanbanConfigRequest {
            provider: KanbanProviderKind::Linear,
            jira: None,
            linear: Some(WriteLinearConfigBody {
                workspace_key: "myws".to_string(),
                api_key_env: "OPERATOR_LINEAR_API_KEY".to_string(),
                project_key: "ENG".to_string(),
                sync_user_id: "user-uuid-42".to_string(),
            }),
            github: None,
        };

        let resp = write_config(req, Some(&path)).unwrap();
        assert_eq!(resp.section_header, "[kanban.linear.\"myws\"]");

        let contents = std::fs::read_to_string(&path).unwrap();
        assert!(contents.contains("myws"));
        assert!(contents.contains("OPERATOR_LINEAR_API_KEY"));
        assert!(contents.contains("ENG"));
        assert!(contents.contains("user-uuid-42"));
    }

    #[test]
    fn test_write_config_upsert_preserves_siblings() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.toml");

        // Write first project
        write_config(
            WriteKanbanConfigRequest {
                provider: KanbanProviderKind::Jira,
                jira: Some(WriteJiraConfigBody {
                    domain: "acme.atlassian.net".to_string(),
                    email: "u@acme.com".to_string(),
                    api_key_env: "OPERATOR_JIRA_API_KEY".to_string(),
                    project_key: "FIRST".to_string(),
                    sync_user_id: "acct-1".to_string(),
                }),
                linear: None,
                github: None,
            },
            Some(&path),
        )
        .unwrap();

        // Write second project to the same workspace
        write_config(
            WriteKanbanConfigRequest {
                provider: KanbanProviderKind::Jira,
                jira: Some(WriteJiraConfigBody {
                    domain: "acme.atlassian.net".to_string(),
                    email: "u@acme.com".to_string(),
                    api_key_env: "OPERATOR_JIRA_API_KEY".to_string(),
                    project_key: "SECOND".to_string(),
                    sync_user_id: "acct-2".to_string(),
                }),
                linear: None,
                github: None,
            },
            Some(&path),
        )
        .unwrap();

        let contents = std::fs::read_to_string(&path).unwrap();
        assert!(contents.contains("FIRST"), "first project preserved");
        assert!(contents.contains("SECOND"), "second project added");
    }

    #[test]
    fn test_build_shell_export_block_jira_uses_placeholder() {
        let block = build_shell_export_block_jira("OPERATOR_JIRA_API_KEY");
        assert_eq!(
            block,
            "export OPERATOR_JIRA_API_KEY=\"<your-jira-api-token>\""
        );
        assert!(!block.contains("real"), "no real secret should leak");
    }

    #[test]
    fn test_build_shell_export_block_linear_uses_placeholder() {
        let block = build_shell_export_block_linear("OPERATOR_LINEAR_API_KEY");
        assert_eq!(
            block,
            "export OPERATOR_LINEAR_API_KEY=\"<your-linear-api-key>\""
        );
    }

    #[test]
    fn test_build_shell_export_block_github_uses_placeholder() {
        let block = build_shell_export_block_github("OPERATOR_GITHUB_TOKEN");
        assert_eq!(
            block,
            "export OPERATOR_GITHUB_TOKEN=\"<your-github-projects-token>\""
        );
        // The placeholder must distinguish this from the repo-token GITHUB_TOKEN
        // (Token Disambiguation rule 4).
        assert!(block.contains("github-projects"));
    }

    #[test]
    fn test_write_config_github_writes_new_section() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.toml");

        let req = WriteKanbanConfigRequest {
            provider: KanbanProviderKind::Github,
            jira: None,
            linear: None,
            github: Some(WriteGithubConfigBody {
                owner: "octo-org".to_string(),
                api_key_env: "OPERATOR_GITHUB_TOKEN".to_string(),
                project_key: "PVT_kwDOABcdefg".to_string(),
                sync_user_id: "12345678".to_string(),
            }),
        };

        let resp = write_config(req, Some(&path)).unwrap();
        assert_eq!(resp.section_header, "[kanban.github.\"octo-org\"]");

        let contents = std::fs::read_to_string(&path).unwrap();
        assert!(contents.contains("octo-org"));
        assert!(contents.contains("OPERATOR_GITHUB_TOKEN"));
        assert!(contents.contains("PVT_kwDOABcdefg"));
        assert!(contents.contains("12345678"));
    }

    #[test]
    fn test_validate_missing_jira_body_returns_bad_request() {
        let req = ValidateKanbanCredentialsRequest {
            provider: KanbanProviderKind::Jira,
            jira: None,
            linear: None,
            github: None,
        };
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(validate_credentials(req));
        assert!(matches!(result, Err(ApiError::BadRequest(_))));
    }

    #[test]
    fn test_validate_missing_github_body_returns_bad_request() {
        let req = ValidateKanbanCredentialsRequest {
            provider: KanbanProviderKind::Github,
            jira: None,
            linear: None,
            github: None,
        };
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(validate_credentials(req));
        assert!(matches!(result, Err(ApiError::BadRequest(_))));
    }
}

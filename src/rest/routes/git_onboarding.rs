use axum::{extract::State, Json};

use crate::integrations::catalog::{onboardable, Vertical};
use crate::rest::dto::{
    GitOnboardingState, GitProviderOnboardingResponse, SetGitSessionEnvRequest,
    SetGitSessionEnvResponse, ValidateGitTokenRequest, ValidateGitTokenResponse,
    WriteGitConfigRequest, WriteGitConfigResponse,
};
use crate::rest::error::ApiError;
use crate::rest::state::ApiState;
use crate::services::git_onboarding::{
    self, configure_git_provider, resolve_onboarding_with_config, OnboardingStep,
};

fn provider_is_offered(provider: &str) -> bool {
    onboardable(Vertical::Git)
        .iter()
        .any(|entry| entry.slug == provider)
}

fn validate_env_name(name: &str) -> Result<(), ApiError> {
    let mut chars = name.chars();
    let valid_first = chars
        .next()
        .is_some_and(|character| character == '_' || character.is_ascii_alphabetic());
    if !valid_first || !chars.all(|character| character == '_' || character.is_ascii_alphanumeric())
    {
        return Err(ApiError::ValidationError(format!(
            "'{name}' is not a valid environment variable name"
        )));
    }
    Ok(())
}

#[utoipa::path(
    get,
    path = "/api/v1/git/providers",
    tag = "Git",
    operation_id = "git_providers",
    responses((status = 200, body = [GitProviderOnboardingResponse]))
)]
pub async fn providers(
    State(state): State<ApiState>,
) -> Result<Json<Vec<GitProviderOnboardingResponse>>, ApiError> {
    let config = state.config();
    tokio::task::spawn_blocking(move || {
        onboardable(Vertical::Git)
            .into_iter()
            .filter_map(|entry| {
                let step = resolve_onboarding_with_config(&config, entry.slug)?;
                let token_env = git_onboarding::token_env_for(&config, entry.slug)?;
                let command = crate::api::cli_detection::onboarding_spec_for_slug(entry.slug)?
                    .command
                    .to_string();
                let configured = git_onboarding::provider_is_configured(&config, entry.slug);
                let (state, action_url, username) = match step {
                    OnboardingStep::InstallCli { install_url, .. } => {
                        (GitOnboardingState::CliMissing, install_url, None)
                    }
                    OnboardingStep::CollectToken { pat_url, .. } => {
                        (GitOnboardingState::TokenRequired, pat_url, None)
                    }
                    OnboardingStep::AutoConfigured { username, .. } => (
                        GitOnboardingState::Authenticated,
                        entry.docs_url().unwrap_or_default(),
                        Some(username),
                    ),
                };
                Some(GitProviderOnboardingResponse {
                    slug: entry.slug.to_string(),
                    label: entry.label.to_string(),
                    docs_url: entry.docs_url().unwrap_or_default(),
                    configured,
                    command,
                    token_env,
                    state,
                    action_url,
                    username,
                })
            })
            .collect()
    })
    .await
    .map(Json)
    .map_err(|error| ApiError::InternalError(format!("Git detection task failed: {error}")))
}

#[utoipa::path(
    post,
    path = "/api/v1/git/validate",
    tag = "Git",
    operation_id = "git_validate",
    request_body = ValidateGitTokenRequest,
    responses((status = 200, body = ValidateGitTokenResponse))
)]
pub async fn validate(
    State(state): State<ApiState>,
    Json(request): Json<ValidateGitTokenRequest>,
) -> Result<Json<ValidateGitTokenResponse>, ApiError> {
    if !provider_is_offered(&request.provider) {
        return Err(ApiError::ValidationError(format!(
            "Git provider '{}' is not available for onboarding",
            request.provider
        )));
    }
    let config = state.config();
    let provider = request.provider;
    let token = request.token;
    let result = tokio::task::spawn_blocking(move || {
        git_onboarding::validate_token_with_config(&config, &provider, &token)
    })
    .await
    .map_err(|error| ApiError::InternalError(format!("Git validation task failed: {error}")))?;
    Ok(Json(match result {
        Ok(username) => ValidateGitTokenResponse {
            valid: true,
            username: Some(username),
            error: None,
        },
        Err(error) => ValidateGitTokenResponse {
            valid: false,
            username: None,
            error: Some(error.to_string()),
        },
    }))
}

#[utoipa::path(
    put,
    path = "/api/v1/git/config",
    tag = "Git",
    operation_id = "git_write_config",
    request_body = WriteGitConfigRequest,
    responses((status = 200, body = WriteGitConfigResponse))
)]
pub async fn write_config(
    State(state): State<ApiState>,
    Json(request): Json<WriteGitConfigRequest>,
) -> Result<Json<WriteGitConfigResponse>, ApiError> {
    if !provider_is_offered(&request.provider) {
        return Err(ApiError::ValidationError(format!(
            "Git provider '{}' is not available for onboarding",
            request.provider
        )));
    }
    validate_env_name(&request.token_env)?;

    let detection_config = state.config();
    let detection_provider = request.provider.clone();
    let adopted = tokio::task::spawn_blocking(move || {
        match resolve_onboarding_with_config(&detection_config, &detection_provider) {
            Some(OnboardingStep::AutoConfigured {
                username, token, ..
            }) => Some((username, token)),
            _ => None,
        }
    })
    .await
    .map_err(|error| ApiError::InternalError(format!("Git detection task failed: {error}")))?;

    if let Some((_, token)) = &adopted {
        std::env::set_var(&request.token_env, token);
    }
    let provider = request.provider;
    let token_env = request.token_env;
    let username = adopted.map(|(username, _)| username);
    let response = state
        .mutate_config(move |config| {
            configure_git_provider(config, &provider, &token_env)
                .map_err(|error| ApiError::ValidationError(error.to_string()))?;
            Ok(WriteGitConfigResponse {
                provider,
                shell_export_block: git_onboarding::shell_export_block(&token_env),
                token_env,
                username,
            })
        })
        .await?;
    Ok(Json(response))
}

#[utoipa::path(
    post,
    path = "/api/v1/git/session-env",
    tag = "Git",
    operation_id = "git_set_session_env",
    request_body = SetGitSessionEnvRequest,
    responses((status = 200, body = SetGitSessionEnvResponse))
)]
pub async fn set_session_env(
    State(state): State<ApiState>,
    Json(request): Json<SetGitSessionEnvRequest>,
) -> Result<Json<SetGitSessionEnvResponse>, ApiError> {
    if !provider_is_offered(&request.provider) {
        return Err(ApiError::ValidationError(format!(
            "Git provider '{}' is not available for onboarding",
            request.provider
        )));
    }
    let token_env = git_onboarding::token_env_for(&state.config(), &request.provider)
        .ok_or_else(|| ApiError::ValidationError("Unsupported git provider".to_string()))?;
    validate_env_name(&token_env)?;
    tokio::task::spawn_blocking({
        let token_env = token_env.clone();
        move || std::env::set_var(token_env, request.token)
    })
    .await
    .map_err(|error| ApiError::InternalError(format!("Git environment task failed: {error}")))?;
    Ok(Json(SetGitSessionEnvResponse {
        shell_export_block: git_onboarding::shell_export_block(&token_env),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_validate_env_name_rejects_shell_syntax() {
        assert!(validate_env_name("TOKEN;echo").is_err());
        assert!(validate_env_name("9TOKEN").is_err());
        assert!(validate_env_name("TOKEN_NAME").is_ok());
    }

    #[test]
    fn test_git_provider_set_matches_catalog() {
        let offered: HashSet<_> = onboardable(Vertical::Git)
            .into_iter()
            .map(|entry| entry.slug)
            .collect();
        assert!(offered.iter().all(|slug| provider_is_offered(slug)));
    }
}

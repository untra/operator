//! Delegator CRUD endpoints.
//!
//! Manages agent delegator configurations that define named {tool, model}
//! pairings for autonomous ticket launching.

use axum::{
    extract::{Path, State},
    Json,
};

use crate::config::{
    agent_profile::{delegator_to_profile, profile_to_delegator, AgentProfile},
    Config, Delegator, DelegatorLaunchConfig,
};
use crate::rest::dto::{
    CreateDelegatorFromToolRequest, CreateDelegatorRequest, DelegatorLaunchConfigDto,
    DelegatorResponse, DelegatorsResponse,
};
use crate::rest::error::ApiError;
use crate::rest::state::ApiState;

/// List all configured delegators
#[utoipa::path(
    operation_id = "delegators_list",
    get,
    path = "/api/v1/delegators",
    tag = "Delegators",
    responses(
        (status = 200, description = "List of delegators", body = DelegatorsResponse)
    )
)]
pub async fn list(State(state): State<ApiState>) -> Json<DelegatorsResponse> {
    let delegators: Vec<DelegatorResponse> = state
        .config
        .delegators
        .iter()
        .map(delegator_to_response)
        .collect();
    let total = delegators.len();
    Json(DelegatorsResponse { delegators, total })
}

/// Get a single delegator by name
#[utoipa::path(
    operation_id = "delegators_get_one",
    get,
    path = "/api/v1/delegators/{name}",
    tag = "Delegators",
    params(
        ("name" = String, Path, description = "Delegator name")
    ),
    responses(
        (status = 200, description = "Delegator details", body = DelegatorResponse),
        (status = 404, description = "Delegator not found")
    )
)]
pub async fn get_one(
    State(state): State<ApiState>,
    Path(name): Path<String>,
) -> Result<Json<DelegatorResponse>, ApiError> {
    let delegator = state
        .config
        .delegators
        .iter()
        .find(|d| d.name == name)
        .ok_or_else(|| ApiError::NotFound(format!("Delegator '{name}' not found")))?;

    Ok(Json(delegator_to_response(delegator)))
}

/// Create a new delegator
#[utoipa::path(
    operation_id = "delegators_create",
    post,
    path = "/api/v1/delegators",
    tag = "Delegators",
    request_body = CreateDelegatorRequest,
    responses(
        (status = 200, description = "Delegator created", body = DelegatorResponse),
        (status = 409, description = "Delegator already exists")
    )
)]
pub async fn create(
    State(state): State<ApiState>,
    Json(req): Json<CreateDelegatorRequest>,
) -> Result<Json<DelegatorResponse>, ApiError> {
    // Check for duplicate name
    if state.config.delegators.iter().any(|d| d.name == req.name) {
        return Err(ApiError::Conflict(format!(
            "Delegator '{}' already exists",
            req.name
        )));
    }

    let delegator = Delegator {
        name: req.name,
        llm_tool: req.llm_tool,
        model: req.model,
        display_name: req.display_name,
        model_properties: req.model_properties,
        model_server: req.model_server,
        launch_config: req.launch_config.map(dto_to_launch_config),
        remote_agent: req.remote_agent,
        x_agnt: None,
        x_openai: None,
        unmapped_core: None,
    };

    // Read current config, add delegator, save
    let mut config = Config::load(None).unwrap_or_else(|_| (*state.config).clone());
    config.delegators.push(delegator.clone());
    config
        .save()
        .map_err(|e| ApiError::InternalError(format!("Failed to save config: {e}")))?;

    Ok(Json(delegator_to_response(&delegator)))
}

/// Delete a delegator by name
#[utoipa::path(
    operation_id = "delegators_delete",
    delete,
    path = "/api/v1/delegators/{name}",
    tag = "Delegators",
    params(
        ("name" = String, Path, description = "Delegator name")
    ),
    responses(
        (status = 200, description = "Delegator deleted", body = DelegatorResponse),
        (status = 404, description = "Delegator not found")
    )
)]
pub async fn delete(
    State(state): State<ApiState>,
    Path(name): Path<String>,
) -> Result<Json<DelegatorResponse>, ApiError> {
    // Find the delegator first for the response
    let delegator = state
        .config
        .delegators
        .iter()
        .find(|d| d.name == name)
        .ok_or_else(|| ApiError::NotFound(format!("Delegator '{name}' not found")))?;
    let response = delegator_to_response(delegator);

    // Read current config, remove delegator, save
    let mut config = Config::load(None).unwrap_or_else(|_| (*state.config).clone());
    config.delegators.retain(|d| d.name != name);
    config
        .save()
        .map_err(|e| ApiError::InternalError(format!("Failed to save config: {e}")))?;

    Ok(Json(response))
}

/// Convert a `DelegatorLaunchConfigDto` to a `DelegatorLaunchConfig`
fn dto_to_launch_config(lc: DelegatorLaunchConfigDto) -> DelegatorLaunchConfig {
    DelegatorLaunchConfig {
        yolo: lc.yolo,
        permission_mode: lc.permission_mode,
        flags: lc.flags,
        use_worktrees: lc.use_worktrees,
        create_branch: lc.create_branch,
        docker: lc.docker,
        prompt_prefix: lc.prompt_prefix,
        prompt_suffix: lc.prompt_suffix,
        operator_relay: lc.operator_relay,
    }
}

/// Convert a `DelegatorLaunchConfig` to a `DelegatorLaunchConfigDto`
fn launch_config_to_dto(lc: &DelegatorLaunchConfig) -> DelegatorLaunchConfigDto {
    DelegatorLaunchConfigDto {
        yolo: lc.yolo,
        permission_mode: lc.permission_mode.clone(),
        flags: lc.flags.clone(),
        use_worktrees: lc.use_worktrees,
        create_branch: lc.create_branch,
        docker: lc.docker,
        prompt_prefix: lc.prompt_prefix.clone(),
        prompt_suffix: lc.prompt_suffix.clone(),
        operator_relay: lc.operator_relay,
    }
}

/// Convert a Delegator config to a `DelegatorResponse` DTO
fn delegator_to_response(d: &Delegator) -> DelegatorResponse {
    DelegatorResponse {
        name: d.name.clone(),
        llm_tool: d.llm_tool.clone(),
        model: d.model.clone(),
        display_name: d.display_name.clone(),
        model_properties: d.model_properties.clone(),
        model_server: d.model_server.clone(),
        launch_config: d.launch_config.as_ref().map(launch_config_to_dto),
        remote_agent: d.remote_agent.clone(),
    }
}

/// Create a delegator from a detected LLM tool
///
/// Pre-populates delegator fields from the detected tool, requiring minimal input.
#[utoipa::path(
    operation_id = "delegators_create_from_tool",
    post,
    path = "/api/v1/delegators/from-tool",
    tag = "Delegators",
    request_body = CreateDelegatorFromToolRequest,
    responses(
        (status = 200, description = "Delegator created from tool", body = DelegatorResponse),
        (status = 404, description = "Tool not detected"),
        (status = 409, description = "Delegator already exists")
    )
)]
pub async fn create_from_tool(
    State(state): State<ApiState>,
    Json(req): Json<CreateDelegatorFromToolRequest>,
) -> Result<Json<DelegatorResponse>, ApiError> {
    // Find the detected tool
    let tool = state
        .config
        .llm_tools
        .detected
        .iter()
        .find(|t| t.name == req.tool_name)
        .ok_or_else(|| ApiError::NotFound(format!("Tool '{}' not detected", req.tool_name)))?;

    // Resolve model (explicit or first alias or "default")
    let model = req.model.unwrap_or_else(|| {
        tool.model_aliases
            .first()
            .cloned()
            .unwrap_or_else(|| "default".to_string())
    });

    // Auto-generate name if not provided
    let name = req
        .name
        .unwrap_or_else(|| format!("{}-{}", tool.name, model));

    // Check for duplicate
    if state.config.delegators.iter().any(|d| d.name == name) {
        return Err(ApiError::Conflict(format!(
            "Delegator '{name}' already exists"
        )));
    }

    let delegator = Delegator {
        name,
        llm_tool: tool.name.clone(),
        model,
        display_name: req.display_name,
        model_properties: std::collections::HashMap::new(),
        model_server: req.model_server.clone(),
        launch_config: req.launch_config.map(dto_to_launch_config),
        remote_agent: None,
        x_agnt: None,
        x_openai: None,
        unmapped_core: None,
    };

    // Save to config
    let mut config = Config::load(None).unwrap_or_else(|_| (*state.config).clone());
    config.delegators.push(delegator.clone());
    config
        .save()
        .map_err(|e| ApiError::InternalError(format!("Failed to save config: {e}")))?;

    Ok(Json(delegator_to_response(&delegator)))
}

/// Update an existing delegator
#[utoipa::path(
    operation_id = "delegators_update",
    put,
    path = "/api/v1/delegators/{name}",
    tag = "Delegators",
    params(
        ("name" = String, Path, description = "Delegator name")
    ),
    request_body = CreateDelegatorRequest,
    responses(
        (status = 200, description = "Delegator updated", body = DelegatorResponse),
        (status = 404, description = "Delegator not found")
    )
)]
pub async fn update(
    State(state): State<ApiState>,
    Path(name): Path<String>,
    Json(req): Json<CreateDelegatorRequest>,
) -> Result<Json<DelegatorResponse>, ApiError> {
    // Verify the delegator exists, and capture its opaque AGNT carry-fields so an
    // update through the (AGNT-unaware) request DTO doesn't drop them.
    let existing = state
        .config
        .delegators
        .iter()
        .find(|d| d.name == name)
        .ok_or_else(|| ApiError::NotFound(format!("Delegator '{name}' not found")))?;

    let updated = Delegator {
        name: name.clone(),
        llm_tool: req.llm_tool,
        model: req.model,
        display_name: req.display_name,
        model_properties: req.model_properties,
        model_server: req.model_server,
        launch_config: req.launch_config.map(dto_to_launch_config),
        remote_agent: req.remote_agent,
        x_agnt: existing.x_agnt.clone(),
        x_openai: existing.x_openai.clone(),
        unmapped_core: existing.unmapped_core.clone(),
    };

    // Replace in config and save
    let mut config = Config::load(None).unwrap_or_else(|_| (*state.config).clone());
    if let Some(existing) = config.delegators.iter_mut().find(|d| d.name == name) {
        *existing = updated.clone();
    }
    config
        .save()
        .map_err(|e| ApiError::InternalError(format!("Failed to save config: {e}")))?;

    Ok(Json(delegator_to_response(&updated)))
}

/// Export a delegator as a portable `AgentProfile` (`agent-profile.json`).
///
/// The shared core plus the Operator-namespaced `x_operator` bag; any opaque
/// AGNT (`x_agnt`) and shared-core carry the delegator holds are restored so the
/// profile is a lossless interchange artifact.
#[utoipa::path(
    operation_id = "delegators_export_profile",
    get,
    path = "/api/v1/delegators/{name}/profile",
    tag = "Delegators",
    params(
        ("name" = String, Path, description = "Delegator name")
    ),
    responses(
        (status = 200, description = "Agent profile", body = AgentProfile),
        (status = 404, description = "Delegator not found")
    )
)]
pub async fn export_profile(
    State(state): State<ApiState>,
    Path(name): Path<String>,
) -> Result<Json<AgentProfile>, ApiError> {
    let delegator = state
        .config
        .delegators
        .iter()
        .find(|d| d.name == name)
        .ok_or_else(|| ApiError::NotFound(format!("Delegator '{name}' not found")))?;

    Ok(Json(delegator_to_profile(delegator)))
}

/// Import an `AgentProfile` as a new delegator.
///
/// The shared-core fields Operator can't model and the opaque `x_agnt` bag are
/// preserved on the created delegator so a later export round-trips losslessly.
#[utoipa::path(
    operation_id = "delegators_import_profile",
    post,
    path = "/api/v1/delegators/import-profile",
    tag = "Delegators",
    request_body = AgentProfile,
    responses(
        (status = 200, description = "Delegator created from profile", body = DelegatorResponse),
        (status = 409, description = "Delegator already exists")
    )
)]
pub async fn import_profile(
    State(state): State<ApiState>,
    Json(profile): Json<AgentProfile>,
) -> Result<Json<DelegatorResponse>, ApiError> {
    if state
        .config
        .delegators
        .iter()
        .any(|d| d.name == profile.name)
    {
        return Err(ApiError::Conflict(format!(
            "Delegator '{}' already exists",
            profile.name
        )));
    }

    let delegator = profile_to_delegator(&profile);

    let mut config = Config::load(None).unwrap_or_else(|_| (*state.config).clone());
    config.delegators.push(delegator.clone());
    config
        .save()
        .map_err(|e| ApiError::InternalError(format!("Failed to save config: {e}")))?;

    Ok(Json(delegator_to_response(&delegator)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_list_empty() {
        let config = Config::default();
        let state = ApiState::new(config, PathBuf::from("/tmp/test"));

        let resp = list(State(state)).await;
        assert_eq!(resp.total, 0);
        assert!(resp.delegators.is_empty());
    }

    #[tokio::test]
    async fn test_list_with_delegators() {
        let mut config = Config::default();
        config.delegators.push(Delegator {
            name: "test-delegator".to_string(),
            llm_tool: "claude".to_string(),
            model: "opus".to_string(),
            display_name: Some("Test".to_string()),
            model_properties: std::collections::HashMap::new(),
            model_server: None,
            launch_config: None,
            remote_agent: None,
            x_agnt: None,
            x_openai: None,
            unmapped_core: None,
        });
        let state = ApiState::new(config, PathBuf::from("/tmp/test"));

        let resp = list(State(state)).await;
        assert_eq!(resp.total, 1);
        assert_eq!(resp.delegators[0].name, "test-delegator");
    }

    #[tokio::test]
    async fn test_get_one_not_found() {
        let config = Config::default();
        let state = ApiState::new(config, PathBuf::from("/tmp/test"));

        let result = get_one(State(state), Path("nonexistent".to_string())).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_one_found() {
        let mut config = Config::default();
        config.delegators.push(Delegator {
            name: "my-delegator".to_string(),
            llm_tool: "codex".to_string(),
            model: "gpt-4o".to_string(),
            display_name: None,
            model_properties: std::collections::HashMap::new(),
            model_server: None,
            launch_config: Some(DelegatorLaunchConfig {
                yolo: true,
                permission_mode: None,
                flags: vec![],
                ..Default::default()
            }),
            remote_agent: None,
            x_agnt: None,
            x_openai: None,
            unmapped_core: None,
        });
        let state = ApiState::new(config, PathBuf::from("/tmp/test"));

        let result = get_one(State(state), Path("my-delegator".to_string())).await;
        assert!(result.is_ok());
        let resp = result.unwrap();
        assert_eq!(resp.name, "my-delegator");
        assert_eq!(resp.llm_tool, "codex");
        assert!(resp.launch_config.as_ref().unwrap().yolo);
    }

    #[tokio::test]
    async fn test_get_one_with_extended_launch_config() {
        let mut config = Config::default();
        config.delegators.push(Delegator {
            name: "full-config".to_string(),
            llm_tool: "claude".to_string(),
            model: "opus".to_string(),
            display_name: Some("Full Config".to_string()),
            model_properties: std::collections::HashMap::new(),
            model_server: None,
            launch_config: Some(DelegatorLaunchConfig {
                yolo: true,
                permission_mode: Some("accept-edits".to_string()),
                flags: vec!["--verbose".to_string()],
                use_worktrees: Some(true),
                create_branch: Some(true),
                docker: Some(false),
                prompt_prefix: Some("Always follow TDD.".to_string()),
                prompt_suffix: Some("Run tests before finishing.".to_string()),
                operator_relay: None,
            }),
            remote_agent: None,
            x_agnt: None,
            x_openai: None,
            unmapped_core: None,
        });
        let state = ApiState::new(config, PathBuf::from("/tmp/test"));

        let result = get_one(State(state), Path("full-config".to_string())).await;
        assert!(result.is_ok());
        let resp = result.unwrap();
        let lc = resp.launch_config.as_ref().unwrap();
        assert!(lc.yolo);
        assert_eq!(lc.use_worktrees, Some(true));
        assert_eq!(lc.create_branch, Some(true));
        assert_eq!(lc.docker, Some(false));
        assert_eq!(lc.prompt_prefix.as_deref(), Some("Always follow TDD."));
        assert_eq!(
            lc.prompt_suffix.as_deref(),
            Some("Run tests before finishing.")
        );
    }

    #[tokio::test]
    async fn test_create_from_tool_unknown() {
        let config = Config::default();
        let state = ApiState::new(config, PathBuf::from("/tmp/test"));

        let req = crate::rest::dto::CreateDelegatorFromToolRequest {
            tool_name: "nonexistent".to_string(),
            model: None,
            name: None,
            display_name: None,
            model_server: None,
            launch_config: None,
        };

        let result = create_from_tool(State(state), Json(req)).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn export_profile_returns_agent_profile() {
        let mut config = Config::default();
        config.delegators.push(Delegator {
            name: "claude-opus".to_string(),
            llm_tool: "claude".to_string(),
            model: "opus".to_string(),
            display_name: Some("Claude Opus".to_string()),
            model_properties: std::collections::HashMap::new(),
            model_server: None,
            launch_config: None,
            remote_agent: None,
            x_agnt: None,
            x_openai: None,
            unmapped_core: None,
        });
        let state = ApiState::new(config, PathBuf::from("/tmp/test"));

        let resp = export_profile(State(state), Path("claude-opus".to_string()))
            .await
            .expect("export ok");
        assert_eq!(resp.name, "claude-opus");
        assert_eq!(resp.provider, "claude");
        assert_eq!(resp.model, "opus");
        assert_eq!(
            resp.x_operator.as_ref().unwrap().display_name.as_deref(),
            Some("Claude Opus")
        );
    }

    #[tokio::test]
    async fn import_profile_conflicts_on_existing_name() {
        // The happy path calls Config::save() (a fixed global path), so — like the
        // create() tests — we only exercise the pre-save conflict branch here. The
        // profile→delegator conversion (incl. x_agnt/shared-core preservation) is
        // covered by the unit tests in `config::agent_profile`.
        let mut config = Config::default();
        config.delegators.push(Delegator {
            name: "dup".to_string(),
            llm_tool: "claude".to_string(),
            model: "opus".to_string(),
            display_name: None,
            model_properties: std::collections::HashMap::new(),
            model_server: None,
            launch_config: None,
            remote_agent: None,
            x_agnt: None,
            x_openai: None,
            unmapped_core: None,
        });
        let state = ApiState::new(config, PathBuf::from("/tmp/test"));

        let profile = AgentProfile {
            name: "dup".to_string(),
            provider: "anthropic".to_string(),
            model: "claude-3-5-sonnet".to_string(),
            system_prompt: None,
            skills: vec![],
            mcp_servers: vec![],
            tools: vec![],
            remote_agent: Some(crate::config::RemoteAgentRef {
                platform: "agnt".to_string(),
                id: "Research Assistant".to_string(),
            }),
            x_operator: None,
            x_agnt: Some(serde_json::json!({ "creditLimit": 1000 })),
            x_openai: None,
        };

        let result = import_profile(State(state), Json(profile)).await;
        assert!(
            matches!(result, Err(ApiError::Conflict(_))),
            "importing a profile whose name already exists must 409"
        );
    }

    #[test]
    fn test_dto_round_trips_operator_relay() {
        let config = DelegatorLaunchConfig {
            yolo: false,
            permission_mode: None,
            flags: vec![],
            use_worktrees: None,
            create_branch: None,
            docker: None,
            prompt_prefix: None,
            prompt_suffix: None,
            operator_relay: Some(true),
        };
        let dto = launch_config_to_dto(&config);
        assert_eq!(dto.operator_relay, Some(true));
        let round_tripped = dto_to_launch_config(dto);
        assert_eq!(round_tripped.operator_relay, Some(true));
    }
}

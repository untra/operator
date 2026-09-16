use std::collections::{HashMap, HashSet};

use axum::{extract::State, Json};

use crate::api::providers::model_server::ModelServerKind;
use crate::collections::fetch::{CollectionOrigin, ResolvedCollection};
use crate::config::{CollectionPreset, ModelServer};
use crate::integrations::catalog::{onboardable, Vertical};
use crate::rest::dto::{
    SetupCollectionResponse, SetupExecutionTarget, SetupInitializeRequest, SetupInitializeResponse,
    SetupStatusResponse, SetupStepResponse,
};
use crate::rest::error::ApiError;
use crate::rest::state::ApiState;
use crate::setup::{initialize_workspace, FetchedCollection, SetupOptions, COMMON_OPTIONAL_FIELDS};
use crate::startup::steps::SetupStep;

async fn resolve_collections(config: &crate::config::Config) -> Vec<ResolvedCollection> {
    let manifest = config
        .templates
        .collections_fetch_enabled
        .then_some(config.templates.collections_manifest_url.as_deref())
        .flatten();
    crate::collections::fetch::resolve_for_setup(
        manifest,
        config.templates.collections_fetch_timeout_secs,
    )
    .await
}

#[utoipa::path(
    get,
    path = "/api/v1/setup/status",
    tag = "Setup",
    operation_id = "setup_status",
    responses((status = 200, body = SetupStatusResponse))
)]
pub async fn status(State(state): State<ApiState>) -> Result<Json<SetupStatusResponse>, ApiError> {
    let config = state.config();
    let projects_path = config.projects_path();
    let store = state.auth.store.clone();
    let (projects_by_tool, admin_configured) = tokio::task::spawn_blocking(move || {
        let projects = crate::projects::discover_projects_by_tool(&projects_path);
        let configured = store
            .bootstrap_state()
            .is_ok_and(|status| status != crate::rest::dto::BootstrapState::Uninitialized);
        (projects, configured)
    })
    .await
    .map_err(|error| ApiError::InternalError(format!("Setup status task failed: {error}")))?;

    Ok(Json(SetupStatusResponse {
        initialized: crate::startup::workspace_initialized(&config),
        admin_configured,
        config_path: config.operator_config_path_for().display().to_string(),
        tickets_path: config.tickets_path().display().to_string(),
        projects_by_tool,
        default_acceptance_criteria: include_str!("../../templates/ACCEPTANCE_CRITERIA.md")
            .to_string(),
    }))
}

#[utoipa::path(
    get,
    path = "/api/v1/setup/steps",
    tag = "Setup",
    operation_id = "setup_steps",
    responses((status = 200, body = [SetupStepResponse]))
)]
pub async fn steps() -> Json<Vec<SetupStepResponse>> {
    Json(
        SetupStep::ALL
            .into_iter()
            .enumerate()
            .map(|(order, slug)| {
                let info = slug.info();
                SetupStepResponse {
                    slug,
                    name: info.name.to_string(),
                    description: info.description.to_string(),
                    help_text: info.help_text.to_string(),
                    order,
                }
            })
            .collect(),
    )
}

#[utoipa::path(
    get,
    path = "/api/v1/setup/collections",
    tag = "Setup",
    operation_id = "setup_collections",
    responses((status = 200, body = [SetupCollectionResponse]))
)]
pub async fn collections(State(state): State<ApiState>) -> Json<Vec<SetupCollectionResponse>> {
    let resolved = resolve_collections(&state.config()).await;
    Json(
        resolved
            .into_iter()
            .map(|collection| {
                let types = collection.manifest.type_keys();
                let origin = match collection.origin {
                    CollectionOrigin::Hosted => "hosted",
                    CollectionOrigin::Embedded => "embedded",
                };
                SetupCollectionResponse {
                    id: collection.manifest.id,
                    name: collection.manifest.name,
                    description: collection.manifest.description,
                    types,
                    default_selected: collection.manifest.default_selected,
                    origin: origin.to_string(),
                    note: collection.note,
                    checksum: collection.manifest.checksum.unwrap_or_default(),
                }
            })
            .collect(),
    )
}

fn validate_task_fields(fields: &[String]) -> Result<Vec<String>, ApiError> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for field in fields {
        if !COMMON_OPTIONAL_FIELDS.contains(&field.as_str()) {
            return Err(ApiError::ValidationError(format!(
                "Unknown TASK field '{field}'"
            )));
        }
        if seen.insert(field.clone()) {
            out.push(field.clone());
        }
    }
    Ok(out)
}

fn selected_models(slugs: &[String]) -> Result<Vec<ModelServer>, ApiError> {
    let offered: HashSet<&str> = onboardable(Vertical::Model)
        .into_iter()
        .map(|entry| entry.slug)
        .collect();
    let mut seen = HashSet::new();
    let mut servers = Vec::new();
    for slug in slugs {
        if !offered.contains(slug.as_str()) || !seen.insert(slug.clone()) {
            if seen.contains(slug) {
                continue;
            }
            return Err(ApiError::ValidationError(format!(
                "Model provider '{slug}' is not available for onboarding"
            )));
        }
        let kind = ModelServerKind::from_slug(slug)
            .ok_or_else(|| ApiError::ValidationError(format!("Unknown model provider '{slug}'")))?;
        if !kind.connectable_from_defaults() {
            return Err(ApiError::ValidationError(format!(
                "Model provider '{slug}' requires a custom base URL"
            )));
        }
        servers.push(ModelServer {
            name: slug.clone(),
            kind: slug.clone(),
            base_url: kind.default_base_url().map(str::to_string),
            api_key_env: kind.default_api_key_env().map(str::to_string),
            extra_env: HashMap::new(),
            display_name: Some(kind.display_name().to_string()),
        });
    }
    Ok(servers)
}

fn selected_collections(
    request: &SetupInitializeRequest,
    resolved: Vec<ResolvedCollection>,
) -> Result<(Vec<FetchedCollection>, Vec<String>, Option<String>), ApiError> {
    if request.preset != CollectionPreset::Custom {
        if !request.hosted_collections.is_empty() {
            return Err(ApiError::ValidationError(
                "Hosted collections require the custom preset".to_string(),
            ));
        }
        return Ok((Vec::new(), Vec::new(), None));
    }
    if request.hosted_collections.is_empty() {
        return Err(ApiError::ValidationError(
            "The custom preset requires at least one hosted collection".to_string(),
        ));
    }

    let mut fetched = Vec::new();
    let mut issue_types = Vec::new();
    for selected in &request.hosted_collections {
        let collection = resolved
            .iter()
            .find(|candidate| candidate.manifest.id == selected.id)
            .ok_or_else(|| {
                ApiError::Conflict(format!(
                    "Collection '{}' is no longer available; reload the collection list",
                    selected.id
                ))
            })?;
        if collection.manifest.checksum.as_deref() != Some(selected.checksum.as_str()) {
            return Err(ApiError::Conflict(format!(
                "Collection '{}' changed; reload the collection list",
                selected.id
            )));
        }
        let keys = if collection.manifest.default_selected.is_empty() {
            collection.manifest.type_keys()
        } else {
            collection.manifest.default_selected.clone()
        };
        for key in keys {
            if !issue_types.contains(&key) {
                issue_types.push(key);
            }
        }
        fetched.push((
            collection.manifest.clone(),
            collection.files.clone(),
            collection.icon_svg.clone(),
        ));
    }
    let active = (fetched.len() == 1).then(|| fetched[0].0.id.clone());
    Ok((fetched, issue_types, active))
}

fn selected_execution_target(
    target: SetupExecutionTarget,
    wrapper: crate::config::SessionWrapperType,
) -> Result<crate::config::TargetDef, ApiError> {
    match target {
        SetupExecutionTarget::Local => Ok(crate::config::TargetDef::local()),
        SetupExecutionTarget::Coder {
            name,
            template,
            parameters,
        } => {
            let name = name.trim();
            let template = template.trim();
            if wrapper == crate::config::SessionWrapperType::Zellij {
                return Err(ApiError::ValidationError(
                    "Coder targets cannot use the Zellij session wrapper".to_string(),
                ));
            }
            if name.is_empty()
                || matches!(
                    name,
                    crate::config::TARGET_LOCAL | crate::config::TARGET_DOCKER
                )
            {
                return Err(ApiError::ValidationError(
                    "Coder target name is empty or reserved".to_string(),
                ));
            }
            if template.is_empty() {
                return Err(ApiError::ValidationError(
                    "Coder template is required".to_string(),
                ));
            }
            Ok(crate::config::TargetDef {
                name: name.to_string(),
                display_name: Some("Coder".to_string()),
                kind: crate::config::TargetKind::Coder(crate::config::CoderConfig {
                    template: template.to_string(),
                    parameters,
                    ..Default::default()
                }),
            })
        }
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/setup/initialize",
    tag = "Setup",
    operation_id = "setup_initialize",
    request_body = SetupInitializeRequest,
    responses(
        (status = 200, body = SetupInitializeResponse),
        (status = 402, description = "Premium required for remote execution"),
        (status = 409, description = "Workspace already initialized")
    )
)]
pub async fn initialize(
    State(state): State<ApiState>,
    Json(request): Json<SetupInitializeRequest>,
) -> Result<Json<SetupInitializeResponse>, ApiError> {
    let task_fields = validate_task_fields(&request.task_fields)?;
    let model_servers = selected_models(&request.model_servers)?;
    let resolved = if request.preset == CollectionPreset::Custom {
        resolve_collections(&state.config()).await
    } else {
        Vec::new()
    };
    let (hosted_collections, custom_collection, active_collection) =
        selected_collections(&request, resolved)?;
    let execution_target = selected_execution_target(request.execution_target, request.wrapper)?;
    crate::licensing::require_target(&state.config(), &execution_target)?;
    let options = SetupOptions {
        preset: request.preset,
        task_fields,
        use_worktrees: request.use_worktrees,
        wrapper: Some(request.wrapper),
        execution_target: Some(execution_target),
        acceptance_criteria: Some(request.acceptance_criteria),
        custom_collection,
        active_collection,
        hosted_collections,
        model_servers,
        ..Default::default()
    };
    let tickets_path = state.config().tickets_path();
    let result = state
        .mutate_config(move |config| {
            if crate::startup::workspace_initialized(config) {
                return Err(ApiError::Conflict(
                    "Workspace is already initialized".to_string(),
                ));
            }
            initialize_workspace(config, &options).map_err(ApiError::from)
        })
        .await?;
    crate::startup::mark_workspace_initialized(&state.config()).map_err(ApiError::from)?;

    let registry = crate::startup::templates::load_registry(&tickets_path);
    *state.registry.write().await = registry;

    Ok(Json(SetupInitializeResponse {
        initialized: true,
        config_path: result.config_path.display().to_string(),
        tickets_path: tickets_path.display().to_string(),
        files_created: result
            .files_created
            .into_iter()
            .map(|path| path.display().to_string())
            .collect(),
        files_skipped: result
            .files_skipped
            .into_iter()
            .map(|path| path.display().to_string())
            .collect(),
        projects: result
            .discovered
            .into_iter()
            .map(|project| project.name)
            .collect(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn initialize_request() -> SetupInitializeRequest {
        SetupInitializeRequest {
            preset: CollectionPreset::Simple,
            task_fields: vec!["priority".to_string()],
            wrapper: crate::config::SessionWrapperType::Tmux,
            execution_target: SetupExecutionTarget::Local,
            use_worktrees: true,
            acceptance_criteria: "Ship only when verified.".to_string(),
            model_servers: Vec::new(),
            hosted_collections: Vec::new(),
        }
    }

    #[test]
    fn test_validate_task_fields_rejects_unknown_field() {
        let error = validate_task_fields(&["mystery".to_string()]).unwrap_err();
        assert!(matches!(error, ApiError::ValidationError(_)));
    }

    #[test]
    fn test_setup_steps_match_catalog_order() {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let response = runtime.block_on(steps());
        assert_eq!(response.0.len(), SetupStep::ALL.len());
        for (order, step) in response.0.iter().enumerate() {
            assert_eq!(step.slug, SetupStep::ALL[order]);
            assert_eq!(step.order, order);
        }
    }

    #[test]
    fn test_coder_target_rejects_zellij() {
        let result = selected_execution_target(
            SetupExecutionTarget::Coder {
                name: "coder-agents".to_string(),
                template: "operator".to_string(),
                parameters: std::collections::HashMap::new(),
            },
            crate::config::SessionWrapperType::Zellij,
        );
        assert!(matches!(result, Err(ApiError::ValidationError(_))));
    }

    #[test]
    fn test_coder_target_preserves_template_parameters() {
        let parameters = std::collections::HashMap::from([
            ("region".to_string(), "us-west".to_string()),
            ("optional".to_string(), String::new()),
        ]);
        let target = selected_execution_target(
            SetupExecutionTarget::Coder {
                name: "coder-agents".to_string(),
                template: "operator".to_string(),
                parameters: parameters.clone(),
            },
            crate::config::SessionWrapperType::Tmux,
        )
        .unwrap();
        let crate::config::TargetKind::Coder(coder) = target.kind else {
            panic!("expected coder target");
        };
        assert_eq!(coder.parameters, parameters);
    }

    #[test]
    fn test_coder_target_without_parameters_remains_compatible() {
        let target: SetupExecutionTarget = serde_json::from_value(serde_json::json!({
            "kind": "coder",
            "name": "coder-agents",
            "template": "operator"
        }))
        .unwrap();
        let SetupExecutionTarget::Coder { parameters, .. } = target else {
            panic!("expected coder target");
        };
        assert!(parameters.is_empty());
    }

    #[tokio::test]
    async fn test_initialize_persists_and_rejects_reinitialization() {
        let temp = tempfile::TempDir::new().unwrap();
        let mut config = crate::config::Config::default();
        config.paths.tickets = temp.path().join("tickets").display().to_string();
        config.paths.state = temp.path().join("state").display().to_string();
        config.paths.projects = temp.path().join("projects").display().to_string();
        let state = ApiState::new(config, temp.path().join("tickets"));

        let response = initialize(State(state.clone()), Json(initialize_request()))
            .await
            .unwrap();
        assert!(response.initialized);
        assert!(state.config().operator_config_path_for().is_file());
        assert!(crate::startup::workspace_initialized(&state.config()));
        assert_eq!(
            state.config().sessions.wrapper,
            crate::config::SessionWrapperType::Tmux
        );
        assert_eq!(
            state.config().launch.target.as_deref(),
            Some(crate::config::TARGET_LOCAL)
        );

        let second = initialize(State(state), Json(initialize_request())).await;
        assert!(matches!(second, Err(ApiError::Conflict(_))));
    }
}

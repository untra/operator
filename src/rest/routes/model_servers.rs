//! Model server CRUD endpoints.
//!
//! A model server is a named host that serves models via an inference API
//! (ollama, lmstudio, vllm, any OpenAI-compatible endpoint). Implicit builtin
//! servers (`anthropic-api`, `openai-api`, `google-api`) are returned on list
//! but cannot be created, updated, or deleted.

use axum::{
    extract::{Path, State},
    Json,
};

use crate::api::providers::model_server::{probe_models, ModelServerKind};
use crate::config::{implicit_model_server_for_tool, Config, ModelServer};
use crate::rest::dto::{
    CreateModelServerRequest, ModelEntry, ModelServerKindEntry, ModelServerModelsResponse,
    ModelServerResponse, ModelServersResponse, UpdateModelServerRequest,
};
use crate::rest::error::ApiError;
use crate::rest::state::ApiState;

const IMPLICIT_TOOL_NAMES: &[&str] = &["claude", "codex", "gemini"];

/// Find a server by name among user-declared servers, then implicit builtins.
fn find_server(config: &Config, name: &str) -> Option<(ModelServer, bool)> {
    if let Some(s) = config.model_servers.iter().find(|s| s.name == name) {
        return Some((s.clone(), true));
    }
    IMPLICIT_TOOL_NAMES
        .iter()
        .map(|t| implicit_model_server_for_tool(t))
        .find(|s| s.name == name)
        .map(|s| (s, false))
}

fn server_to_response(s: &ModelServer, user_declared: bool) -> ModelServerResponse {
    ModelServerResponse {
        name: s.name.clone(),
        kind: s.kind.clone(),
        base_url: s.base_url.clone(),
        api_key_env: s.api_key_env.clone(),
        extra_env: s.extra_env.clone(),
        display_name: s.display_name.clone(),
        user_declared,
    }
}

/// List all model servers (user-declared + implicit builtins)
#[utoipa::path(
    operation_id = "model_servers_list",
    get,
    path = "/api/v1/model-servers",
    tag = "ModelServers",
    responses(
        (status = 200, description = "List of model servers", body = ModelServersResponse)
    )
)]
pub async fn list(State(state): State<ApiState>) -> Json<ModelServersResponse> {
    let mut servers: Vec<ModelServerResponse> = state
        .config
        .model_servers
        .iter()
        .map(|s| server_to_response(s, true))
        .collect();

    for tool in IMPLICIT_TOOL_NAMES {
        let implicit = implicit_model_server_for_tool(tool);
        if !servers.iter().any(|s| s.name == implicit.name) {
            servers.push(server_to_response(&implicit, false));
        }
    }

    let total = servers.len();
    Json(ModelServersResponse { servers, total })
}

/// Get a single model server by name
#[utoipa::path(
    operation_id = "model_servers_get_one",
    get,
    path = "/api/v1/model-servers/{name}",
    tag = "ModelServers",
    params(
        ("name" = String, Path, description = "Model server name")
    ),
    responses(
        (status = 200, description = "Model server details", body = ModelServerResponse),
        (status = 404, description = "Model server not found")
    )
)]
pub async fn get_one(
    State(state): State<ApiState>,
    Path(name): Path<String>,
) -> Result<Json<ModelServerResponse>, ApiError> {
    if let Some(server) = state.config.model_servers.iter().find(|s| s.name == name) {
        return Ok(Json(server_to_response(server, true)));
    }
    for tool in IMPLICIT_TOOL_NAMES {
        let implicit = implicit_model_server_for_tool(tool);
        if implicit.name == name {
            return Ok(Json(server_to_response(&implicit, false)));
        }
    }
    Err(ApiError::NotFound(format!(
        "Model server '{name}' not found"
    )))
}

/// Create a new model server
#[utoipa::path(
    operation_id = "model_servers_create",
    post,
    path = "/api/v1/model-servers",
    tag = "ModelServers",
    request_body = CreateModelServerRequest,
    responses(
        (status = 200, description = "Model server created", body = ModelServerResponse),
        (status = 409, description = "Model server already exists")
    )
)]
pub async fn create(
    State(state): State<ApiState>,
    Json(req): Json<CreateModelServerRequest>,
) -> Result<Json<ModelServerResponse>, ApiError> {
    if state
        .config
        .model_servers
        .iter()
        .any(|s| s.name == req.name)
    {
        return Err(ApiError::Conflict(format!(
            "Model server '{}' already exists",
            req.name
        )));
    }
    if IMPLICIT_TOOL_NAMES
        .iter()
        .any(|t| implicit_model_server_for_tool(t).name == req.name)
    {
        return Err(ApiError::Conflict(format!(
            "'{}' is a reserved implicit builtin name",
            req.name
        )));
    }

    let server = ModelServer {
        name: req.name,
        kind: req.kind,
        base_url: req.base_url,
        api_key_env: req.api_key_env,
        extra_env: req.extra_env,
        display_name: req.display_name,
    };

    let mut config = Config::load(None).unwrap_or_else(|_| (*state.config).clone());
    config.model_servers.push(server.clone());
    config
        .save()
        .map_err(|e| ApiError::InternalError(format!("Failed to save config: {e}")))?;

    Ok(Json(server_to_response(&server, true)))
}

/// Delete a user-declared model server by name
///
/// Implicit builtin servers cannot be deleted.
#[utoipa::path(
    operation_id = "model_servers_delete",
    delete,
    path = "/api/v1/model-servers/{name}",
    tag = "ModelServers",
    params(
        ("name" = String, Path, description = "Model server name")
    ),
    responses(
        (status = 200, description = "Model server deleted", body = ModelServerResponse),
        (status = 404, description = "Model server not found"),
        (status = 409, description = "Cannot delete implicit builtin server")
    )
)]
pub async fn delete(
    State(state): State<ApiState>,
    Path(name): Path<String>,
) -> Result<Json<ModelServerResponse>, ApiError> {
    if IMPLICIT_TOOL_NAMES
        .iter()
        .any(|t| implicit_model_server_for_tool(t).name == name)
    {
        return Err(ApiError::Conflict(format!(
            "'{name}' is an implicit builtin and cannot be deleted"
        )));
    }

    let server = state
        .config
        .model_servers
        .iter()
        .find(|s| s.name == name)
        .ok_or_else(|| ApiError::NotFound(format!("Model server '{name}' not found")))?
        .clone();

    let response = server_to_response(&server, true);

    let mut config = Config::load(None).unwrap_or_else(|_| (*state.config).clone());
    config.model_servers.retain(|s| s.name != name);
    config
        .save()
        .map_err(|e| ApiError::InternalError(format!("Failed to save config: {e}")))?;

    Ok(Json(response))
}

/// Update an existing user-declared model server
#[utoipa::path(
    operation_id = "model_servers_update",
    put,
    path = "/api/v1/model-servers/{name}",
    tag = "ModelServers",
    params(
        ("name" = String, Path, description = "Model server name")
    ),
    request_body = UpdateModelServerRequest,
    responses(
        (status = 200, description = "Model server updated", body = ModelServerResponse),
        (status = 404, description = "Model server not found"),
        (status = 409, description = "Cannot update implicit builtin server")
    )
)]
pub async fn update(
    State(state): State<ApiState>,
    Path(name): Path<String>,
    Json(req): Json<UpdateModelServerRequest>,
) -> Result<Json<ModelServerResponse>, ApiError> {
    if IMPLICIT_TOOL_NAMES
        .iter()
        .any(|t| implicit_model_server_for_tool(t).name == name)
    {
        return Err(ApiError::Conflict(format!(
            "'{name}' is an implicit builtin and cannot be updated"
        )));
    }

    let mut config = Config::load(None).unwrap_or_else(|_| (*state.config).clone());
    let server = config
        .model_servers
        .iter_mut()
        .find(|s| s.name == name)
        .ok_or_else(|| ApiError::NotFound(format!("Model server '{name}' not found")))?;

    server.kind = req.kind;
    server.base_url = req.base_url;
    server.api_key_env = req.api_key_env;
    server.extra_env = req.extra_env;
    server.display_name = req.display_name;
    let updated = server.clone();

    config
        .save()
        .map_err(|e| ApiError::InternalError(format!("Failed to save config: {e}")))?;

    Ok(Json(server_to_response(&updated, true)))
}

/// List the models a server offers, via a live probe of its inference endpoint.
///
/// The probe doubles as a reachability check — `reachable: false` with an `error`
/// when the endpoint is unreachable or rejects the request.
#[utoipa::path(
    operation_id = "model_servers_models",
    get,
    path = "/api/v1/model-servers/{name}/models",
    tag = "ModelServers",
    params(
        ("name" = String, Path, description = "Model server name")
    ),
    responses(
        (status = 200, description = "Models offered by the server", body = ModelServerModelsResponse),
        (status = 404, description = "Model server not found")
    )
)]
pub async fn models(
    State(state): State<ApiState>,
    Path(name): Path<String>,
) -> Result<Json<ModelServerModelsResponse>, ApiError> {
    let (server, _) = find_server(&state.config, &name)
        .ok_or_else(|| ApiError::NotFound(format!("Model server '{name}' not found")))?;

    let outcome = probe_models(&server).await;
    Ok(Json(ModelServerModelsResponse {
        server: name,
        reachable: outcome.reachable,
        models: outcome
            .models
            .into_iter()
            .map(|m| ModelEntry {
                id: m.id,
                display_name: m.display_name,
            })
            .collect(),
        error: outcome.error,
    }))
}

/// The catalog of supported model-server kinds (single source of truth).
#[utoipa::path(
    operation_id = "model_servers_kinds",
    get,
    path = "/api/v1/model-servers/kinds",
    tag = "ModelServers",
    responses(
        (status = 200, description = "Supported model-server kinds", body = [ModelServerKindEntry])
    )
)]
pub async fn kinds() -> Json<Vec<ModelServerKindEntry>> {
    Json(
        ModelServerKind::ALL
            .iter()
            .map(|k| ModelServerKindEntry {
                slug: k.slug().to_string(),
                display_name: k.display_name().to_string(),
                description: k.connect_blurb().to_string(),
                setup_url: k.setup_url().to_string(),
                icon: k.icon().to_string(),
                is_builtin: k.is_builtin(),
                category: k.provider_class().slug().to_string(),
                category_label: k.provider_class().display_name().to_string(),
                brand_icon: k.brand_icon().map(str::to_string),
                default_base_url: k.default_base_url().map(str::to_string),
                default_api_key_env: k.default_api_key_env().map(str::to_string),
                connectable: k.connectable_from_defaults(),
            })
            .collect(),
    )
}

/// List the models a *provider kind* offers, via a live probe.
///
/// Resolves to the declared instance of that kind (if the user has one) else a
/// transient instance built from the kind's probe defaults — so the Model
/// Providers catalog can show connection state + live models for every supported
/// provider without first declaring one. `reachable` doubles as "connected".
#[utoipa::path(
    operation_id = "model_servers_kind_models",
    get,
    path = "/api/v1/model-servers/kinds/{slug}/models",
    tag = "ModelServers",
    params(
        ("slug" = String, Path, description = "Model-provider kind slug (e.g. \"anthropic-api\")")
    ),
    responses(
        (status = 200, description = "Models offered by the provider", body = ModelServerModelsResponse),
        (status = 404, description = "Unknown provider kind")
    )
)]
pub async fn kind_models(
    State(state): State<ApiState>,
    Path(slug): Path<String>,
) -> Result<Json<ModelServerModelsResponse>, ApiError> {
    let kind = ModelServerKind::from_slug(&slug)
        .ok_or_else(|| ApiError::NotFound(format!("Unknown provider kind '{slug}'")))?;

    // Prefer a user-declared instance of this kind; otherwise probe from the
    // kind's built-in defaults (the probe fills in base_url/api_key_env).
    let server = state
        .config
        .model_servers
        .iter()
        .find(|s| s.kind == slug)
        .cloned()
        .unwrap_or_else(|| ModelServer {
            name: slug.clone(),
            kind: slug.clone(),
            base_url: None,
            api_key_env: None,
            extra_env: std::collections::HashMap::new(),
            display_name: None,
        });

    let outcome = probe_models(&server).await;
    Ok(Json(ModelServerModelsResponse {
        server: kind.slug().to_string(),
        reachable: outcome.reachable,
        models: outcome
            .models
            .into_iter()
            .map(|m| ModelEntry {
                id: m.id,
                display_name: m.display_name,
            })
            .collect(),
        error: outcome.error,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_list_returns_builtins_when_no_user_servers() {
        let config = Config::default();
        let state = ApiState::new(config, PathBuf::from("/tmp/test-ms"));
        let resp = list(State(state)).await;

        // At minimum, one implicit server for each known tool.
        assert!(resp.total >= IMPLICIT_TOOL_NAMES.len());
        assert!(resp.servers.iter().any(|s| s.name == "anthropic-api"));
        assert!(resp.servers.iter().any(|s| s.name == "openai-api"));
        assert!(resp.servers.iter().any(|s| s.name == "google-api"));
        assert!(resp.servers.iter().all(|s| !s.user_declared));
    }

    #[tokio::test]
    async fn test_list_includes_user_declared_servers() {
        let mut config = Config::default();
        config.model_servers.push(ModelServer {
            name: "ollama-local".to_string(),
            kind: "ollama".to_string(),
            base_url: Some("http://localhost:11434".to_string()),
            api_key_env: None,
            extra_env: std::collections::HashMap::new(),
            display_name: Some("Ollama (local)".to_string()),
        });
        let state = ApiState::new(config, PathBuf::from("/tmp/test-ms-user"));
        let resp = list(State(state)).await;

        let ollama = resp
            .servers
            .iter()
            .find(|s| s.name == "ollama-local")
            .expect("ollama-local should appear");
        assert!(ollama.user_declared);
        assert_eq!(ollama.kind, "ollama");
    }

    #[tokio::test]
    async fn test_get_one_returns_implicit_builtin() {
        let config = Config::default();
        let state = ApiState::new(config, PathBuf::from("/tmp/test-ms-get"));
        let resp = get_one(State(state), Path("openai-api".to_string())).await;
        let server = resp.expect("implicit openai-api should resolve").0;
        assert_eq!(server.name, "openai-api");
        assert!(!server.user_declared);
    }

    #[tokio::test]
    async fn test_kinds_lists_all_catalog_entries() {
        let resp = kinds().await;
        let slugs: Vec<&str> = resp.0.iter().map(|k| k.slug.as_str()).collect();
        assert!(slugs.contains(&"ollama"));
        assert!(slugs.contains(&"openrouter"));
        assert!(slugs.contains(&"openai-compat"));
        assert!(slugs.contains(&"anthropic-api"));
        // First-party providers are flagged builtin and carry the first-party class.
        let anthropic = resp.0.iter().find(|k| k.slug == "anthropic-api").unwrap();
        assert!(anthropic.is_builtin);
        assert_eq!(anthropic.category, "first-party");
        // Probe-connectable from defaults, with the standard key env surfaced.
        assert!(anthropic.connectable);
        assert_eq!(
            anthropic.default_base_url.as_deref(),
            Some("https://api.anthropic.com")
        );
        assert_eq!(
            anthropic.default_api_key_env.as_deref(),
            Some("ANTHROPIC_API_KEY")
        );
        // Gateways (ollama, openrouter, …) are not builtins.
        let ollama = resp.0.iter().find(|k| k.slug == "ollama").unwrap();
        assert!(!ollama.is_builtin);
        assert_eq!(ollama.category, "gateway");
        let openrouter = resp.0.iter().find(|k| k.slug == "openrouter").unwrap();
        assert_eq!(openrouter.category, "gateway");
        assert_eq!(openrouter.category_label, "Gateways");
        // openai-compat is bring-your-own-endpoint: not connectable from defaults.
        let compat = resp.0.iter().find(|k| k.slug == "openai-compat").unwrap();
        assert!(!compat.connectable);
        // Brand icons: anthropic/google/ollama/openrouter carry one; openai-api
        // falls back to a codicon.
        assert_eq!(openrouter.brand_icon.as_deref(), Some("openrouter"));
        assert_eq!(ollama.brand_icon.as_deref(), Some("ollama"));
        assert_eq!(anthropic.brand_icon.as_deref(), Some("anthropic"));
        let openai = resp.0.iter().find(|k| k.slug == "openai-api").unwrap();
        assert_eq!(openai.brand_icon, None);
    }

    #[tokio::test]
    async fn test_models_unreachable_endpoint_reports_error() {
        // Probe a declared server pointing at a closed local port — deterministic
        // and offline (connection refused), exercising the unreachable path
        // without any external network dependency.
        let mut config = Config::default();
        config.model_servers.push(ModelServer {
            name: "closed-port".into(),
            kind: "openai-compat".into(),
            base_url: Some("http://127.0.0.1:1".into()),
            api_key_env: None,
            extra_env: std::collections::HashMap::new(),
            display_name: None,
        });
        let state = ApiState::new(config, PathBuf::from("/tmp/test-ms-models"));
        let resp = models(State(state), Path("closed-port".to_string()))
            .await
            .expect("declared server resolves");
        assert_eq!(resp.0.server, "closed-port");
        assert!(!resp.0.reachable);
        assert!(resp.0.models.is_empty());
        assert!(resp.0.error.is_some());
    }

    #[tokio::test]
    async fn test_models_404_on_unknown() {
        let config = Config::default();
        let state = ApiState::new(config, PathBuf::from("/tmp/test-ms-models-404"));
        let resp = models(State(state), Path("nope".to_string())).await;
        assert!(resp.is_err());
    }

    #[tokio::test]
    async fn test_get_one_404_on_unknown() {
        let config = Config::default();
        let state = ApiState::new(config, PathBuf::from("/tmp/test-ms-404"));
        let resp = get_one(State(state), Path("nope".to_string())).await;
        assert!(resp.is_err());
    }

    #[tokio::test]
    async fn test_kind_models_404_on_unknown_kind() {
        let config = Config::default();
        let state = ApiState::new(config, PathBuf::from("/tmp/test-ms-km-404"));
        let resp = kind_models(State(state), Path("nope".to_string())).await;
        assert!(resp.is_err());
    }

    #[tokio::test]
    async fn test_kind_models_bring_your_own_endpoint_unreachable_from_defaults() {
        // `openai-compat` has no default base_url and no declared instance, so a
        // kind-level probe has nowhere to connect — reachable:false, offline.
        let config = Config::default();
        let state = ApiState::new(config, PathBuf::from("/tmp/test-ms-km-byo"));
        let resp = kind_models(State(state), Path("openai-compat".to_string()))
            .await
            .expect("known kind resolves");
        // Response is keyed by the kind slug (not an instance name).
        assert_eq!(resp.0.server, "openai-compat");
        assert!(!resp.0.reachable);
        assert!(resp.0.models.is_empty());
        assert!(resp.0.error.is_some());
    }

    #[tokio::test]
    async fn test_kind_models_prefers_declared_instance() {
        // A declared instance of the kind is probed in preference to defaults.
        // Point it at a closed local port: deterministic, offline (conn refused).
        let mut config = Config::default();
        config.model_servers.push(ModelServer {
            name: "my-vllm".into(),
            kind: "openai-compat".into(),
            base_url: Some("http://127.0.0.1:1".into()),
            api_key_env: None,
            extra_env: std::collections::HashMap::new(),
            display_name: None,
        });
        let state = ApiState::new(config, PathBuf::from("/tmp/test-ms-km-declared"));
        let resp = kind_models(State(state), Path("openai-compat".to_string()))
            .await
            .expect("known kind resolves");
        // Still keyed by the kind slug, and unreachable (the closed port refuses).
        assert_eq!(resp.0.server, "openai-compat");
        assert!(!resp.0.reachable);
        assert!(resp.0.error.is_some());
    }
}

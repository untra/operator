//! Issue type CRUD endpoints.
//!
//! All endpoints are collection-aware: keys are unique only within a
//! collection, an optional `?collection=` scopes lookups, and writes land in
//! the collection-scoped store (`.tickets/templates/<collection>/`).

use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;
use utoipa::IntoParams;

use crate::issuetypes::schema::IssueTypeSource;
use crate::rest::dto::{
    CreateIssueTypeRequest, IssueTypeResponse, IssueTypeSummary, UpdateIssueTypeRequest,
};
use crate::rest::error::{ApiError, ErrorResponse};
use crate::rest::state::ApiState;

/// Optional collection scope for issuetype lookups.
#[derive(Debug, Deserialize, IntoParams)]
pub struct CollectionQuery {
    /// Collection to scope the lookup to (defaults to resolution-order lookup)
    pub collection: Option<String>,
}

/// The reserved kanban-imports namespace: readable, never writable via CRUD.
const IMPORTS_NAMESPACE: &str = "imports";

/// List issue types (all collections deduped, or one collection via `?collection=`)
#[utoipa::path(
    operation_id = "issuetypes_list",
    get,
    path = "/api/v1/issuetypes",
    tag = "Issue Types",
    params(CollectionQuery),
    responses(
        (status = 200, description = "List of issue types", body = Vec<IssueTypeSummary>),
        (status = 404, description = "Unknown collection", body = ErrorResponse)
    )
)]
pub async fn list(
    State(state): State<ApiState>,
    Query(query): Query<CollectionQuery>,
) -> Result<Json<Vec<IssueTypeSummary>>, ApiError> {
    let registry = state.registry.read().await;
    let types: Vec<IssueTypeSummary> = match query.collection {
        Some(name) => registry
            .types_in(&name)
            .ok_or_else(|| ApiError::NotFound(format!("Collection '{name}' not found")))?
            .into_iter()
            .map(|it| IssueTypeSummary::with_collection(it, Some(&name)))
            .collect(),
        None => registry
            .types_with_collections()
            .map(|(collection, it)| IssueTypeSummary::with_collection(it, Some(collection)))
            .collect(),
    };
    Ok(Json(types))
}

/// Get a single issue type by key
#[utoipa::path(
    operation_id = "issuetypes_get_one",
    get,
    path = "/api/v1/issuetypes/{key}",
    tag = "Issue Types",
    params(
        ("key" = String, Path, description = "Issue type key (e.g., FEAT, FIX)"),
        CollectionQuery
    ),
    responses(
        (status = 200, description = "Issue type details", body = IssueTypeResponse),
        (status = 404, description = "Issue type not found", body = ErrorResponse)
    )
)]
pub async fn get_one(
    State(state): State<ApiState>,
    Path(key): Path<String>,
    Query(query): Query<CollectionQuery>,
) -> Result<Json<IssueTypeResponse>, ApiError> {
    let key = key.to_uppercase();
    let registry = state.registry.read().await;
    let issue_type = registry
        .resolve(query.collection.as_deref(), &key)
        .ok_or_else(|| ApiError::NotFound(format!("Issue type '{key}' not found")))?;

    let owner = match &query.collection {
        Some(name) if registry.get_in(name, &key).is_some() => Some(name.as_str()),
        _ => registry.collection_of(&key),
    };
    Ok(Json(IssueTypeResponse::with_collection(issue_type, owner)))
}

/// Create a new issue type
#[utoipa::path(
    operation_id = "issuetypes_create",
    post,
    path = "/api/v1/issuetypes",
    tag = "Issue Types",
    request_body = CreateIssueTypeRequest,
    responses(
        (status = 200, description = "Issue type created", body = IssueTypeResponse),
        (status = 400, description = "Validation error", body = ErrorResponse),
        (status = 409, description = "Issue type already exists", body = ErrorResponse)
    )
)]
pub async fn create(
    State(state): State<ApiState>,
    Json(request): Json<CreateIssueTypeRequest>,
) -> Result<Json<IssueTypeResponse>, ApiError> {
    let requested_collection = request.collection.clone();
    let issue_type = request.into_issue_type();

    // Validate the issue type
    issue_type.validate().map_err(|errors| {
        let msgs: Vec<String> = errors
            .iter()
            .map(std::string::ToString::to_string)
            .collect();
        ApiError::ValidationError(msgs.join("; "))
    })?;

    // Resolve target collection (default: active) and check for a duplicate
    // within it — the same key in another collection is fine.
    let target = {
        let registry = state.registry.read().await;
        let target =
            requested_collection.unwrap_or_else(|| registry.active_collection_name().to_string());
        if target == IMPORTS_NAMESPACE {
            return Err(ApiError::ValidationError(
                "The 'imports' namespace is provider-managed".to_string(),
            ));
        }
        if registry.get_in(&target, &issue_type.key).is_some() {
            return Err(ApiError::Conflict(format!(
                "Issue type '{}' already exists in collection '{target}'",
                issue_type.key
            )));
        }
        target
    };

    // Persist to the collection-scoped store
    let dir = state.templates_path().join(&target);
    tokio::fs::create_dir_all(&dir).await?;
    let filepath = dir.join(format!("{}.json", issue_type.key));
    let json = issue_type.to_json()?;
    tokio::fs::write(&filepath, json).await?;

    // Register in memory
    let mut registry = state.registry.write().await;
    registry
        .register_in(&target, issue_type.clone())
        .map_err(|e| ApiError::InternalError(format!("Failed to register issue type: {e}")))?;

    Ok(Json(IssueTypeResponse::with_collection(
        &issue_type,
        Some(&target),
    )))
}

/// Update an existing issue type
#[utoipa::path(
    operation_id = "issuetypes_update",
    put,
    path = "/api/v1/issuetypes/{key}",
    tag = "Issue Types",
    params(
        ("key" = String, Path, description = "Issue type key")
    ),
    request_body = UpdateIssueTypeRequest,
    responses(
        (status = 200, description = "Issue type updated", body = IssueTypeResponse),
        (status = 400, description = "Validation error", body = ErrorResponse),
        (status = 403, description = "Cannot modify builtin type", body = ErrorResponse),
        (status = 404, description = "Issue type not found", body = ErrorResponse)
    )
)]
pub async fn update(
    State(state): State<ApiState>,
    Path(key): Path<String>,
    Query(query): Query<CollectionQuery>,
    Json(request): Json<UpdateIssueTypeRequest>,
) -> Result<Json<IssueTypeResponse>, ApiError> {
    let key = key.to_uppercase();

    // Get existing issue type and its owning collection
    let (mut issue_type, owner) = {
        let registry = state.registry.read().await;
        let owner = owning_collection(&registry, &key, query.collection.as_deref())?;
        let issue_type = registry
            .get_in(&owner, &key)
            .ok_or_else(|| ApiError::NotFound(format!("Issue type '{key}' not found")))?
            .clone();
        (issue_type, owner)
    };

    // Check if it's a builtin
    if matches!(issue_type.source, IssueTypeSource::Builtin) {
        return Err(ApiError::BuiltinReadOnly(format!(
            "Cannot modify builtin issue type '{key}'"
        )));
    }

    // Apply updates
    if let Some(name) = request.name {
        issue_type.name = name;
    }
    if let Some(description) = request.description {
        issue_type.description = description;
    }
    if let Some(mode) = request.mode {
        issue_type.mode = if mode == "paired" {
            crate::templates::schema::ExecutionMode::Paired
        } else {
            crate::templates::schema::ExecutionMode::Autonomous
        };
    }
    if let Some(glyph) = request.glyph {
        issue_type.glyph = glyph;
    }
    if let Some(color) = request.color {
        issue_type.color = Some(color);
    }
    if let Some(project_required) = request.project_required {
        issue_type.project_required = project_required;
    }
    if let Some(fields) = request.fields {
        issue_type.fields = fields.into_iter().map(std::convert::Into::into).collect();
    }
    if let Some(steps) = request.steps {
        issue_type.steps = steps.into_iter().map(std::convert::Into::into).collect();
    }

    // Validate updated issue type
    issue_type.validate().map_err(|errors| {
        let msgs: Vec<String> = errors
            .iter()
            .map(std::string::ToString::to_string)
            .collect();
        ApiError::ValidationError(msgs.join("; "))
    })?;

    // Persist to the collection-scoped store
    let dir = state.templates_path().join(&owner);
    tokio::fs::create_dir_all(&dir).await?;
    let filepath = dir.join(format!("{key}.json"));
    let json = issue_type.to_json()?;
    tokio::fs::write(&filepath, json).await?;

    // Update in memory
    let mut registry = state.registry.write().await;
    registry
        .register_in(&owner, issue_type.clone())
        .map_err(|e| ApiError::InternalError(format!("Failed to update issue type: {e}")))?;

    Ok(Json(IssueTypeResponse::with_collection(
        &issue_type,
        Some(&owner),
    )))
}

/// Resolve which collection a write should target: the explicit query
/// collection, or wherever the key resolves. Imports are never writable.
fn owning_collection(
    registry: &crate::issuetypes::IssueTypeRegistry,
    key: &str,
    requested: Option<&str>,
) -> Result<String, ApiError> {
    let owner = match requested {
        Some(name) => name.to_string(),
        None => registry
            .collection_of(key)
            .ok_or_else(|| ApiError::NotFound(format!("Issue type '{key}' not found")))?
            .to_string(),
    };
    if owner == IMPORTS_NAMESPACE {
        return Err(ApiError::ValidationError(
            "The 'imports' namespace is provider-managed".to_string(),
        ));
    }
    Ok(owner)
}

/// Delete an issue type
#[utoipa::path(
    operation_id = "issuetypes_delete",
    delete,
    path = "/api/v1/issuetypes/{key}",
    tag = "Issue Types",
    params(
        ("key" = String, Path, description = "Issue type key")
    ),
    responses(
        (status = 200, description = "Issue type deleted"),
        (status = 403, description = "Cannot delete builtin type", body = ErrorResponse),
        (status = 404, description = "Issue type not found", body = ErrorResponse)
    )
)]
pub async fn delete(
    State(state): State<ApiState>,
    Path(key): Path<String>,
    Query(query): Query<CollectionQuery>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let key = key.to_uppercase();

    // Check it exists, find its collection, and reject builtins
    let owner = {
        let registry = state.registry.read().await;
        let owner = owning_collection(&registry, &key, query.collection.as_deref())?;
        let issue_type = registry
            .get_in(&owner, &key)
            .ok_or_else(|| ApiError::NotFound(format!("Issue type '{key}' not found")))?;

        if matches!(issue_type.source, IssueTypeSource::Builtin) {
            return Err(ApiError::BuiltinReadOnly(format!(
                "Cannot delete builtin issue type '{key}'"
            )));
        }
        owner
    };

    // Delete from the collection-scoped store
    let filepath = state
        .templates_path()
        .join(&owner)
        .join(format!("{key}.json"));
    if filepath.exists() {
        tokio::fs::remove_file(&filepath).await?;
    }

    // Remove from memory
    let mut registry = state.registry.write().await;
    registry.remove_from(&owner, &key);

    Ok(Json(serde_json::json!({
        "deleted": key,
        "collection": owner,
        "message": "Issue type deleted."
    })))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use std::path::PathBuf;

    fn make_state() -> ApiState {
        let config = Config::default();
        ApiState::new(config, PathBuf::from("/tmp/test"))
    }

    fn make_temp_state() -> (ApiState, tempfile::TempDir) {
        let temp = tempfile::tempdir().unwrap();
        let config = Config::default();
        let state = ApiState::new(config, temp.path().to_path_buf());
        (state, temp)
    }

    fn no_collection() -> Query<CollectionQuery> {
        Query(CollectionQuery { collection: None })
    }

    fn in_collection(name: &str) -> Query<CollectionQuery> {
        Query(CollectionQuery {
            collection: Some(name.to_string()),
        })
    }

    fn sample_create(key: &str, collection: Option<&str>) -> CreateIssueTypeRequest {
        serde_json::from_value(serde_json::json!({
            "key": key,
            "name": "Sample",
            "description": "A sample type",
            "glyph": "s",
            "steps": [{"name": "execute", "outputs": [], "prompt": "Do it."}],
            "collection": collection,
        }))
        .unwrap()
    }

    #[tokio::test]
    async fn test_list() {
        let state = make_state();
        let resp = list(State(state), no_collection()).await.unwrap();
        assert!(!resp.0.is_empty());
        // Every summary reports its owning collection.
        assert!(resp.0.iter().all(|s| s.collection.is_some()));
    }

    #[tokio::test]
    async fn test_list_filters_by_collection() {
        let state = make_state();
        let resp = list(State(state), in_collection("simple")).await.unwrap();
        let keys: Vec<&str> = resp.0.iter().map(|s| s.key.as_str()).collect();
        assert_eq!(keys, vec!["TASK"]);
        assert_eq!(resp.0[0].collection.as_deref(), Some("simple"));
    }

    #[tokio::test]
    async fn test_list_unknown_collection_404s() {
        let state = make_state();
        let result = list(State(state), in_collection("nope")).await;
        assert!(matches!(result, Err(ApiError::NotFound(_))));
    }

    #[tokio::test]
    async fn test_get_one_exists() {
        let state = make_state();
        let result = get_one(State(state), Path("FEAT".to_string()), no_collection()).await;
        assert!(result.is_ok());
        let resp = result.unwrap();
        assert_eq!(resp.key, "FEAT");
        assert!(resp.collection.is_some());
    }

    #[tokio::test]
    async fn test_get_one_not_found() {
        let state = make_state();
        let result = get_one(State(state), Path("NOTEXIST".to_string()), no_collection()).await;
        assert!(matches!(result, Err(ApiError::NotFound(_))));
    }

    #[tokio::test]
    async fn test_create_persists_into_collection_dir() {
        let (state, _temp) = make_temp_state();
        let resp = create(
            State(state.clone()),
            Json(sample_create("NEWT", Some("mine"))),
        )
        .await
        .unwrap();
        assert_eq!(resp.collection.as_deref(), Some("mine"));

        let filepath = state.templates_path().join("mine/NEWT.json");
        assert!(filepath.exists(), "should persist to templates/mine/");
        let registry = state.registry.read().await;
        assert!(registry.get_in("mine", "NEWT").is_some());
    }

    #[tokio::test]
    async fn test_create_defaults_to_active_collection() {
        let (state, _temp) = make_temp_state();
        let active = {
            let registry = state.registry.read().await;
            registry.active_collection_name().to_string()
        };
        let resp = create(State(state.clone()), Json(sample_create("NEWT", None)))
            .await
            .unwrap();
        assert_eq!(resp.collection.as_deref(), Some(active.as_str()));
        assert!(state
            .templates_path()
            .join(format!("{active}/NEWT.json"))
            .exists());
    }

    #[tokio::test]
    async fn test_create_same_key_other_collection_allowed() {
        let (state, _temp) = make_temp_state();
        let _ = create(
            State(state.clone()),
            Json(sample_create("NEWT", Some("mine"))),
        )
        .await
        .unwrap();
        // Same key in a different collection is fine (namespaced)...
        let _ = create(
            State(state.clone()),
            Json(sample_create("NEWT", Some("other"))),
        )
        .await
        .unwrap();
        // ...but a duplicate within the same collection conflicts.
        let dup = create(State(state), Json(sample_create("NEWT", Some("mine")))).await;
        assert!(matches!(dup, Err(ApiError::Conflict(_))));
    }

    #[tokio::test]
    async fn test_delete_removes_from_memory_and_disk() {
        let (state, _temp) = make_temp_state();
        let _ = create(
            State(state.clone()),
            Json(sample_create("NEWT", Some("mine"))),
        )
        .await
        .unwrap();
        let _ = delete(
            State(state.clone()),
            Path("NEWT".to_string()),
            in_collection("mine"),
        )
        .await
        .unwrap();
        assert!(!state.templates_path().join("mine/NEWT.json").exists());
        let registry = state.registry.read().await;
        assert!(registry.get_in("mine", "NEWT").is_none());
    }
}

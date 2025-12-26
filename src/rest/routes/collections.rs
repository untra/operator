//! Collection management endpoints.

use axum::{
    extract::{Path, State},
    Json,
};

use crate::rest::dto::CollectionResponse;
use crate::rest::error::{ApiError, ErrorResponse};
use crate::rest::state::ApiState;

/// List all collections
#[utoipa::path(
    get,
    path = "/api/v1/collections",
    tag = "Collections",
    responses(
        (status = 200, description = "List of all collections", body = Vec<CollectionResponse>)
    )
)]
pub async fn list(State(state): State<ApiState>) -> Json<Vec<CollectionResponse>> {
    let registry = state.registry.read().await;
    let active_name = registry.active_collection_name();

    let collections: Vec<CollectionResponse> = registry
        .all_collections()
        .map(|c| CollectionResponse::from_collection(c, c.name == active_name))
        .collect();

    Json(collections)
}

/// Get the currently active collection
#[utoipa::path(
    get,
    path = "/api/v1/collections/active",
    tag = "Collections",
    responses(
        (status = 200, description = "Active collection", body = CollectionResponse),
        (status = 404, description = "No active collection", body = ErrorResponse)
    )
)]
pub async fn get_active(
    State(state): State<ApiState>,
) -> Result<Json<CollectionResponse>, ApiError> {
    let registry = state.registry.read().await;

    let collection = registry
        .active_collection()
        .ok_or_else(|| ApiError::NotFound("No active collection".to_string()))?;

    Ok(Json(CollectionResponse::from_collection(collection, true)))
}

/// Get a single collection by name
#[utoipa::path(
    get,
    path = "/api/v1/collections/{name}",
    tag = "Collections",
    params(
        ("name" = String, Path, description = "Collection name")
    ),
    responses(
        (status = 200, description = "Collection details", body = CollectionResponse),
        (status = 404, description = "Collection not found", body = ErrorResponse)
    )
)]
pub async fn get_one(
    State(state): State<ApiState>,
    Path(name): Path<String>,
) -> Result<Json<CollectionResponse>, ApiError> {
    let registry = state.registry.read().await;
    let active_name = registry.active_collection_name();

    let collection = registry
        .get_collection(&name)
        .ok_or_else(|| ApiError::NotFound(format!("Collection '{}' not found", name)))?;

    Ok(Json(CollectionResponse::from_collection(
        collection,
        collection.name == active_name,
    )))
}

/// Activate a collection
#[utoipa::path(
    put,
    path = "/api/v1/collections/{name}/activate",
    tag = "Collections",
    params(
        ("name" = String, Path, description = "Collection name to activate")
    ),
    responses(
        (status = 200, description = "Collection activated", body = CollectionResponse),
        (status = 404, description = "Collection not found", body = ErrorResponse)
    )
)]
pub async fn activate(
    State(state): State<ApiState>,
    Path(name): Path<String>,
) -> Result<Json<CollectionResponse>, ApiError> {
    let mut registry = state.registry.write().await;

    // Activate the collection
    registry
        .activate_collection(&name)
        .map_err(|e| ApiError::NotFound(format!("Failed to activate collection: {}", e)))?;

    // Get the now-active collection
    let collection = registry.active_collection().ok_or_else(|| {
        ApiError::InternalError("Collection disappeared after activation".to_string())
    })?;

    Ok(Json(CollectionResponse::from_collection(collection, true)))
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

    #[tokio::test]
    async fn test_list_collections() {
        let state = make_state();
        let resp = list(State(state)).await;
        assert!(!resp.0.is_empty());

        // At least one should be active
        assert!(resp.0.iter().any(|c| c.is_active));
    }

    #[tokio::test]
    async fn test_get_active() {
        let state = make_state();
        let result = get_active(State(state)).await;
        assert!(result.is_ok());
        let resp = result.unwrap();
        assert!(resp.is_active);
    }

    #[tokio::test]
    async fn test_get_collection_not_found() {
        let state = make_state();
        let result = get_one(State(state), Path("nonexistent".to_string())).await;
        assert!(matches!(result, Err(ApiError::NotFound(_))));
    }

    #[tokio::test]
    async fn test_activate_collection() {
        let state = make_state();

        // First check current active
        {
            let registry = state.registry.read().await;
            let _current = registry.active_collection_name();
        }

        // Activate simple
        let result = activate(State(state.clone()), Path("simple".to_string())).await;
        assert!(result.is_ok());

        // Verify it's now active
        let registry = state.registry.read().await;
        assert_eq!(registry.active_collection_name(), "simple");
    }
}

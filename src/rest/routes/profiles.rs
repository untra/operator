use std::sync::Arc;

use axum::{extract::Path, Extension, Json};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::profiles::{ProfileSummary, ServerProfiles};
use crate::rest::error::ApiError;

#[derive(Deserialize, Serialize, ToSchema)]
pub struct ProfileNameRequest {
    pub name: String,
}

#[utoipa::path(get, path = "/api/v1/profiles",
    operation_id = "profiles_list", tag = "Configuration", responses((status=200, description="Server configurations", body=Vec<ProfileSummary>)))]
pub async fn list(
    Extension(profiles): Extension<Arc<ServerProfiles>>,
) -> Json<Vec<ProfileSummary>> {
    Json(profiles.list())
}

#[utoipa::path(post, path = "/api/v1/profiles",
    operation_id = "profiles_create", tag = "Configuration", request_body=ProfileNameRequest, responses((status=200, description="Draft configuration", body=ProfileSummary), (status=409, description="Name already exists")))]
pub async fn create(
    Extension(profiles): Extension<Arc<ServerProfiles>>,
    Json(request): Json<ProfileNameRequest>,
) -> Result<Json<ProfileSummary>, ApiError> {
    crate::profiles::validate_name(&request.name)
        .map_err(|e| ApiError::ValidationError(e.to_string()))?;
    if profiles
        .list()
        .iter()
        .any(|profile| profile.name == request.name)
    {
        return Err(ApiError::Conflict(
            "Configuration name already exists".into(),
        ));
    }
    profiles
        .create(&request.name)
        .map(Json)
        .map_err(|e| ApiError::Conflict(e.to_string()))
}

#[utoipa::path(patch, path = "/api/v1/profiles/{profile_id}",
    operation_id = "profiles_rename", tag = "Configuration", params(("profile_id"=Uuid, Path, description="Configuration ID")), request_body=ProfileNameRequest, responses((status=200, description="Renamed configuration", body=ProfileSummary)))]
pub async fn rename(
    Extension(profiles): Extension<Arc<ServerProfiles>>,
    Path(id): Path<Uuid>,
    Json(request): Json<ProfileNameRequest>,
) -> Result<Json<ProfileSummary>, ApiError> {
    profiles.rename(id, request.name).await.map(Json)
}

#[utoipa::path(get, path = "/api/v1/profiles/{profile_id}",
    operation_id = "profiles_get", tag = "Configuration", params(("profile_id"=Uuid, Path, description="Configuration ID")), responses((status=200, description="Configuration metadata", body=ProfileSummary)))]
pub async fn get_one(
    Extension(profiles): Extension<Arc<ServerProfiles>>,
    Path(id): Path<Uuid>,
) -> Result<Json<ProfileSummary>, ApiError> {
    profiles
        .state(id)
        .map(|state| Json(profiles.summary(&state.config())))
        .ok_or_else(|| ApiError::NotFound("Configuration not found".into()))
}

use axum::{extract::State, Json};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::licensing::{self, LicenseResponse};
use crate::rest::{error::ApiError, state::ApiState};

#[derive(Deserialize, ToSchema)]
pub struct InstallLicenseRequest {
    pub license_key: String,
}

#[utoipa::path(get, path = "/api/v1/license",
    operation_id = "license_get", tag = "License", responses((status = 200, body = LicenseResponse)))]
pub async fn get(State(state): State<ApiState>) -> Json<LicenseResponse> {
    Json(licensing::status(&state.config()))
}

#[utoipa::path(put, path = "/api/v1/license",
    operation_id = "license_install", tag = "License", request_body = InstallLicenseRequest,
    responses((status = 200, body = LicenseResponse), (status = 400, description = "License rejected")))]
pub async fn install(
    State(state): State<ApiState>,
    Json(request): Json<InstallLicenseRequest>,
) -> Result<Json<LicenseResponse>, ApiError> {
    licensing::install(&state.config(), &request.license_key)
        .map(Json)
        .map_err(|error| ApiError::ValidationError(error.to_string()))
}

#[utoipa::path(delete, path = "/api/v1/license",
    operation_id = "license_remove", tag = "License", responses((status = 200, body = LicenseResponse)))]
pub async fn remove(State(state): State<ApiState>) -> Result<Json<LicenseResponse>, ApiError> {
    licensing::remove(&state.config())
        .map(Json)
        .map_err(ApiError::from)
}

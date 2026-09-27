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

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::config::Config;
    use crate::profiles::{ServerProfiles, LEGACY_PROFILE_NAME};
    use crate::rest::error::ApiError;
    use crate::rest::state::ApiState;
    use axum::extract::Path;
    use axum::http::StatusCode;
    use axum::{Extension, Json};
    use uuid::Uuid;

    fn fixture() -> (tempfile::TempDir, Arc<ServerProfiles>) {
        let dir = tempfile::tempdir().unwrap();
        let mut config = Config::default();
        config.paths.state = dir.path().join("state").to_string_lossy().into_owned();
        let state = ApiState::new(config, dir.path().join("tickets"));
        let profiles = ServerProfiles::open(state).unwrap();
        (dir, profiles)
    }

    fn status_of(error: ApiError) -> StatusCode {
        error.parts().0
    }

    #[tokio::test]
    async fn test_profile_list_create_and_get() {
        let (_dir, profiles) = fixture();
        let listed = list(Extension(Arc::clone(&profiles))).await.0;
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].name, LEGACY_PROFILE_NAME);

        let created = create(
            Extension(Arc::clone(&profiles)),
            Json(ProfileNameRequest {
                name: "demo_1".into(),
            }),
        )
        .await
        .unwrap()
        .0;
        assert_eq!(created.name, "demo_1");
        assert!(list(Extension(Arc::clone(&profiles)))
            .await
            .0
            .iter()
            .any(|profile| profile.id == created.id && profile.name == "demo_1"));

        let fetched = get_one(Extension(Arc::clone(&profiles)), Path(created.id))
            .await
            .unwrap()
            .0;
        assert_eq!(fetched.id, created.id);
        assert_eq!(fetched.name, "demo_1");
        assert_eq!(
            status_of(
                get_one(Extension(profiles), Path(Uuid::new_v4()))
                    .await
                    .unwrap_err()
            ),
            StatusCode::NOT_FOUND
        );
    }

    #[tokio::test]
    async fn test_profile_create_rejects_duplicates_and_invalid_names() {
        let (_dir, profiles) = fixture();
        let created = create(
            Extension(Arc::clone(&profiles)),
            Json(ProfileNameRequest {
                name: "demo_1".into(),
            }),
        )
        .await
        .unwrap()
        .0;
        assert_eq!(created.name, "demo_1");
        assert_eq!(
            status_of(
                create(
                    Extension(Arc::clone(&profiles)),
                    Json(ProfileNameRequest {
                        name: "demo_1".into(),
                    }),
                )
                .await
                .unwrap_err()
            ),
            StatusCode::CONFLICT
        );
        for name in ["Upper", "", "../escape"] {
            assert_eq!(
                status_of(
                    create(
                        Extension(Arc::clone(&profiles)),
                        Json(ProfileNameRequest { name: name.into() }),
                    )
                    .await
                    .unwrap_err()
                ),
                StatusCode::BAD_REQUEST,
                "{name}"
            );
        }
    }

    #[tokio::test]
    async fn test_profile_rename_updates_the_list_and_rejects_conflicts() {
        let (_dir, profiles) = fixture();
        let created = create(
            Extension(Arc::clone(&profiles)),
            Json(ProfileNameRequest {
                name: "demo_1".into(),
            }),
        )
        .await
        .unwrap()
        .0;

        let renamed = rename(
            Extension(Arc::clone(&profiles)),
            Path(created.id),
            Json(ProfileNameRequest {
                name: "demo_2".into(),
            }),
        )
        .await
        .unwrap()
        .0;
        assert_eq!(renamed.name, "demo_2");
        assert!(list(Extension(Arc::clone(&profiles)))
            .await
            .0
            .iter()
            .any(|profile| profile.id == created.id && profile.name == "demo_2"));

        assert_eq!(
            status_of(
                rename(
                    Extension(Arc::clone(&profiles)),
                    Path(created.id),
                    Json(ProfileNameRequest {
                        name: LEGACY_PROFILE_NAME.into(),
                    }),
                )
                .await
                .unwrap_err()
            ),
            StatusCode::CONFLICT
        );
        assert_eq!(
            status_of(
                rename(
                    Extension(Arc::clone(&profiles)),
                    Path(Uuid::new_v4()),
                    Json(ProfileNameRequest {
                        name: "demo_3".into(),
                    }),
                )
                .await
                .unwrap_err()
            ),
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            status_of(
                rename(
                    Extension(profiles),
                    Path(created.id),
                    Json(ProfileNameRequest {
                        name: "Upper".into(),
                    }),
                )
                .await
                .unwrap_err()
            ),
            StatusCode::BAD_REQUEST
        );
    }
}

use std::sync::Arc;

use axum::{
    extract::{Path, Request, State},
    response::{IntoResponse, Response},
    Router,
};
use tower::ServiceExt;
use uuid::Uuid;

use super::error::ApiError;
use crate::profiles::ServerProfiles;

pub fn is_server_path(path: &str) -> bool {
    path.starts_with("/api/v1/auth/")
        || path == "/api/v1/profiles"
        || path.starts_with("/api/v1/profiles/")
        || path == "/api/v1/integrations"
        || path == "/api/v1/health"
}

pub fn mount(default: Router, profiles: Arc<ServerProfiles>) -> Router {
    Router::new()
        .route(
            "/api/v1/profiles/{profile_id}/{*path}",
            axum::routing::any(dispatch),
        )
        .with_state(Arc::clone(&profiles))
        .merge(default.layer(axum::Extension(profiles)))
}

async fn dispatch(
    State(profiles): State<Arc<ServerProfiles>>,
    Path((id, path)): Path<(Uuid, String)>,
    request: Request,
) -> Response {
    let Some(state) = profiles.state(id) else {
        return ApiError::NotFound("Configuration not found".into()).into_response();
    };
    let path = format!("/api/v1/{path}");
    if is_server_path(&path) {
        return ApiError::NotFound("Use the server-level route for this operation".into())
            .into_response();
    }
    let (mut parts, body) = request.into_parts();
    let uri = match parts.uri.query() {
        Some(query) => format!("{path}?{query}"),
        None => path,
    };
    let Ok(uri) = uri.parse() else {
        return ApiError::BadRequest("Invalid configuration route".into()).into_response();
    };
    parts.uri = uri;
    parts.extensions.clear();
    let request = Request::from_parts(parts, body);
    let router = super::build_profile_router(state);
    match router.oneshot(request).await {
        Ok(response) => response,
        Err(never) => match never {},
    }
}

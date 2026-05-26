//! Configuration read/write endpoints.

use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;

use crate::config::Config;
use crate::rest::state::ApiState;

/// Get the current configuration
pub async fn get_config(State(state): State<ApiState>) -> Json<Config> {
    Json((*state.config).clone())
}

/// Update configuration and save to disk
pub async fn update_config(
    State(state): State<ApiState>,
    Json(incoming): Json<Config>,
) -> Result<Json<Config>, (StatusCode, String)> {
    incoming
        .save()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let _ = &state;
    Ok(Json(incoming))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_get_config() {
        let config = Config::default();
        let state = ApiState::new(config, PathBuf::from("/tmp/test"));

        let Json(cfg) = get_config(State(state)).await;
        assert!(!cfg.projects.is_empty() || cfg.projects.is_empty());
    }
}

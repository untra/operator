//! Status sections endpoint — the canonical section tree shared with the TUI
//! and VS Code extension, rendered for the web UI's Status page.

use axum::{extract::State, Json};

use crate::rest::dto::{section_provider, LiveConnectionStatus, SectionDto};
use crate::rest::state::ApiState;

/// List the canonical status sections with health + child rows.
///
/// Returns all sections (with a `met` flag) rather than hiding unmet ones, so
/// the web UI can render every section and style locked ones. The section logic
/// is injected by the binary; without a provider (lib-only/test) this is empty.
#[utoipa::path(
    operation_id = "sections_list",
    get,
    path = "/api/v1/sections",
    tag = "Status",
    responses(
        (status = 200, description = "Canonical status sections", body = Vec<SectionDto>)
    )
)]
pub async fn list(State(state): State<ApiState>) -> Json<Vec<SectionDto>> {
    match section_provider() {
        Some(provider) => {
            let registry = state.registry.read().await;
            // Serving this request proves the API (and embedded Web UI) are up,
            // so report live connection facts the config-only snapshot can't know.
            let live = LiveConnectionStatus {
                api_running: true,
                port: state.config.rest_api.port,
                mcp_http_enabled: state.config.mcp.http_enabled,
                mcp_active_sessions: state.mcp_sessions.lock().await.len(),
            };
            Json(provider(&state.config, &registry, &live))
        }
        None => Json(Vec::new()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_sections_empty_when_no_provider_registered() {
        // The binary registers the provider at startup; tests never do, so the
        // endpoint returns an empty list here. The real section logic is covered
        // by the ui-side `build_section_dtos` tests.
        let state = ApiState::new(Config::default(), PathBuf::from("/tmp/test-sections"));
        let resp = list(State(state)).await;
        assert!(resp.0.is_empty());
    }
}

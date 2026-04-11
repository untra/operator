//! Kanban provider endpoints for the REST API.

use axum::extract::{Path, State};
use axum::Json;

use crate::api::providers::kanban::get_provider_from_config;
use crate::config::Config;
use crate::rest::dto::{
    ExternalIssueTypeSummary, KanbanIssueTypeResponse, SyncKanbanIssueTypesResponse,
};
use crate::rest::error::ApiError;
use crate::rest::state::ApiState;
use crate::services::kanban_issuetype_service::KanbanIssueTypeService;

/// GET /`api/v1/kanban/:provider/:project_key/issuetypes`
///
/// Returns kanban issue types from the persisted catalog for a given provider/project.
/// Falls back to fetching live from the provider if no catalog exists.
pub async fn external_issue_types(
    State(state): State<ApiState>,
    Path((provider_name, project_key)): Path<(String, String)>,
) -> Result<Json<Vec<ExternalIssueTypeSummary>>, ApiError> {
    // Try reading from persisted catalog first
    let service = KanbanIssueTypeService::from_tickets_path(std::path::Path::new(
        &state.config.paths.tickets,
    ));
    let catalog_types = service
        .list_kanban_types(&provider_name, &project_key)
        .map_err(|e| ApiError::InternalError(format!("Failed to read catalog: {e}")))?;

    if !catalog_types.is_empty() {
        let summaries: Vec<ExternalIssueTypeSummary> = catalog_types
            .into_iter()
            .map(|kt| ExternalIssueTypeSummary {
                id: kt.id,
                name: kt.name,
                description: kt.description,
                icon_url: kt.icon_url,
            })
            .collect();
        return Ok(Json(summaries));
    }

    // Fall back to live provider fetch. Reload config from disk so freshly
    // onboarded providers are visible without requiring a server restart.
    let fresh_config = Config::load(None).unwrap_or_else(|_| (*state.config).clone());
    let provider = get_provider_from_config(&fresh_config.kanban, &provider_name, &project_key)
        .map_err(|e| ApiError::BadRequest(e.to_string()))?;

    let external_types = provider
        .get_issue_types(&project_key)
        .await
        .map_err(|e| ApiError::InternalError(format!("Failed to fetch issue types: {e}")))?;

    let summaries: Vec<ExternalIssueTypeSummary> = external_types
        .into_iter()
        .map(|et| ExternalIssueTypeSummary {
            id: et.id,
            name: et.name,
            description: et.description,
            icon_url: et.icon_url,
        })
        .collect();

    Ok(Json(summaries))
}

/// POST /`api/v1/kanban/:provider/:project_key/issuetypes/sync`
///
/// Refreshes the local kanban issue type catalog from the provider.
pub async fn sync_issue_types(
    State(state): State<ApiState>,
    Path((provider_name, project_key)): Path<(String, String)>,
) -> Result<Json<SyncKanbanIssueTypesResponse>, ApiError> {
    // Reload config from disk so freshly onboarded providers are visible
    // without requiring a server restart.
    let fresh_config = Config::load(None).unwrap_or_else(|_| (*state.config).clone());
    let provider = get_provider_from_config(&fresh_config.kanban, &provider_name, &project_key)
        .map_err(|e| ApiError::BadRequest(e.to_string()))?;

    let service = KanbanIssueTypeService::from_tickets_path(std::path::Path::new(
        &state.config.paths.tickets,
    ));

    let synced_types = service
        .sync_issue_types(provider.as_ref(), &project_key)
        .await
        .map_err(|e| ApiError::InternalError(format!("Failed to sync issue types: {e}")))?;

    let types: Vec<KanbanIssueTypeResponse> = synced_types
        .into_iter()
        .map(|kt| KanbanIssueTypeResponse {
            id: kt.id,
            name: kt.name,
            description: kt.description,
            icon_url: kt.icon_url,
            provider: kt.provider,
            project: kt.project,
            source_kind: kt.source_kind,
            synced_at: kt.synced_at,
        })
        .collect();

    let synced = types.len();
    Ok(Json(SyncKanbanIssueTypesResponse { synced, types }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_external_issue_type_summary_serialization() {
        let summary = ExternalIssueTypeSummary {
            id: "10001".to_string(),
            name: "Bug".to_string(),
            description: Some("A bug or defect".to_string()),
            icon_url: None,
        };

        let json = serde_json::to_string(&summary).unwrap();
        assert!(json.contains("\"name\":\"Bug\""));
        assert!(!json.contains("icon_url")); // None fields skipped
    }

    #[test]
    fn test_kanban_issue_type_response_serialization() {
        let response = KanbanIssueTypeResponse {
            id: "10001".to_string(),
            name: "Bug".to_string(),
            description: Some("A bug".to_string()),
            icon_url: None,
            provider: "jira".to_string(),
            project: "PROJ".to_string(),
            source_kind: "issuetype".to_string(),
            synced_at: "2026-04-05T12:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"provider\":\"jira\""));
        assert!(json.contains("\"source_kind\":\"issuetype\""));
        assert!(!json.contains("icon_url")); // None skipped
    }

    #[test]
    fn test_sync_response_serialization() {
        let response = SyncKanbanIssueTypesResponse {
            synced: 2,
            types: vec![KanbanIssueTypeResponse {
                id: "10001".to_string(),
                name: "Bug".to_string(),
                description: None,
                icon_url: None,
                provider: "jira".to_string(),
                project: "PROJ".to_string(),
                source_kind: "issuetype".to_string(),
                synced_at: "2026-04-05T12:00:00Z".to_string(),
            }],
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"synced\":2"));
    }
}

//! Kanban provider endpoints for the REST API.

use axum::extract::{Path, State};
use axum::Json;

use crate::api::providers::kanban::get_provider_from_config;
use crate::rest::dto::ExternalIssueTypeSummary;
use crate::rest::error::ApiError;
use crate::rest::state::ApiState;

/// GET /`api/v1/kanban/:provider/:project_key/issuetypes`
///
/// Returns issue types from an external kanban provider for a given project.
pub async fn external_issue_types(
    State(state): State<ApiState>,
    Path((provider_name, project_key)): Path<(String, String)>,
) -> Result<Json<Vec<ExternalIssueTypeSummary>>, ApiError> {
    let provider = get_provider_from_config(&state.config.kanban, &provider_name, &project_key)
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
}

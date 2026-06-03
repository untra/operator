//! Kanban provider endpoints for the REST API.

use axum::extract::{Path, State};
use axum::Json;

use crate::api::providers::kanban::{get_provider_from_config, KanbanProviderType};
use crate::config::kanban::KanbanConfig;
use crate::config::Config;
use crate::rest::dto::{
    ExternalIssueTypeSummary, KanbanIssueTypeResponse, KanbanProviderCatalogEntry,
    SyncKanbanIssueTypesResponse,
};
use crate::rest::error::ApiError;
use crate::rest::state::ApiState;
use crate::services::kanban_issuetype_service::KanbanIssueTypeService;

/// Build the provider catalog from the canonical `KanbanProviderType::ALL`
/// list, flagging each provider as `configured` when the kanban config already
/// holds at least one instance of it.
fn build_provider_catalog(kanban: &KanbanConfig) -> Vec<KanbanProviderCatalogEntry> {
    KanbanProviderType::ALL
        .into_iter()
        .map(|p| {
            let configured = match p {
                KanbanProviderType::Jira => !kanban.jira.is_empty(),
                KanbanProviderType::Linear => !kanban.linear.is_empty(),
                KanbanProviderType::Github => !kanban.github.is_empty(),
            };
            KanbanProviderCatalogEntry {
                slug: p.slug().to_string(),
                display_name: p.display_name().to_string(),
                description: p.connect_blurb().to_string(),
                setup_url: p.setup_url().to_string(),
                icon: p.icon().to_string(),
                configured,
            }
        })
        .collect()
}

/// GET /`api/v1/kanban/providers`
///
/// Returns the catalog of supported kanban providers (Jira, Linear, GitHub),
/// each flagged with whether it is already configured. This is the shared
/// source of truth for the web `/#/kanban` list view and the VS Code
/// onboarding picker.
#[utoipa::path(
    get,
    path = "/api/v1/kanban/providers",
    tag = "Kanban",
    operation_id = "kanban_provider_catalog",
    responses(
        (status = 200, description = "Supported kanban providers", body = Vec<KanbanProviderCatalogEntry>)
    )
)]
pub async fn provider_catalog(
    State(state): State<ApiState>,
) -> Result<Json<Vec<KanbanProviderCatalogEntry>>, ApiError> {
    // Reload config from disk so freshly onboarded providers are reflected in
    // the `configured` flags without requiring a server restart.
    let fresh_config = Config::load(None).unwrap_or_else(|_| (*state.config).clone());
    Ok(Json(build_provider_catalog(&fresh_config.kanban)))
}

/// GET /`api/v1/kanban/:provider/:project_key/issuetypes`
///
/// Returns kanban issue types from the persisted catalog for a given provider/project.
/// Falls back to fetching live from the provider if no catalog exists.
#[utoipa::path(
    get,
    path = "/api/v1/kanban/{provider}/{project_key}/issuetypes",
    tag = "Kanban",
    operation_id = "kanban_external_issue_types",
    params(
        ("provider" = String, Path, description = "Kanban provider name (e.g. jira, linear, github)"),
        ("project_key" = String, Path, description = "Provider project/team key")
    ),
    responses(
        (status = 200, description = "External issue types", body = Vec<ExternalIssueTypeSummary>),
        (status = 400, description = "Unknown provider/project"),
        (status = 500, description = "Failed to read catalog or fetch from provider")
    )
)]
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
#[utoipa::path(
    post,
    path = "/api/v1/kanban/{provider}/{project_key}/issuetypes/sync",
    tag = "Kanban",
    operation_id = "kanban_sync_issue_types",
    params(
        ("provider" = String, Path, description = "Kanban provider name (e.g. jira, linear, github)"),
        ("project_key" = String, Path, description = "Provider project/team key")
    ),
    responses(
        (status = 200, description = "Synced issue types", body = SyncKanbanIssueTypesResponse),
        (status = 400, description = "Unknown provider/project"),
        (status = 500, description = "Failed to sync from provider")
    )
)]
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
    fn test_build_provider_catalog_lists_all_three_with_configured_flags() {
        let mut kanban = crate::config::kanban::KanbanConfig::default();
        kanban.github.insert(
            "my-org".into(),
            crate::config::kanban::GithubProjectsConfig::default(),
        );

        let catalog = build_provider_catalog(&kanban);

        let slugs: Vec<&str> = catalog.iter().map(|e| e.slug.as_str()).collect();
        assert_eq!(slugs, vec!["jira", "linear", "github"]);

        let github = catalog.iter().find(|e| e.slug == "github").unwrap();
        assert!(github.configured);
        assert_eq!(github.display_name, "GitHub Projects");
        assert_eq!(github.icon, "github");
        assert_eq!(
            github.setup_url,
            "https://github.com/settings/personal-access-tokens"
        );

        let jira = catalog.iter().find(|e| e.slug == "jira").unwrap();
        assert!(!jira.configured);
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

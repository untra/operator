//! Ticket batches (free) and campaigns (Premium).

use axum::{
    extract::{Path, State},
    Json,
};

use crate::campaigns::batch::{self, BatchCreated, BatchError, BatchPreview, BatchRequest};
use crate::campaigns::model::Campaign;
use crate::campaigns::service::{
    self, CampaignDetail, CampaignLaunchOutcome, CampaignStatusView, NewCampaignInput,
};
use crate::campaigns::{members, CampaignStore};
use crate::config::Config;
use crate::licensing::{self, PremiumFeature};
use crate::queue::Queue;
use crate::rest::dto::{
    AssignCampaignRequest, AssignCampaignResponse, CampaignListResponse, CampaignSummary,
    LaunchCampaignRequest,
};
use crate::rest::{error::ApiError, state::ApiState};
use crate::services::campaign_runner::{self, LauncherPort, StartError, TickReport};

fn entitled(config: &Config) -> Result<(), ApiError> {
    licensing::require_premium(config, PremiumFeature::Campaigns)?;
    Ok(())
}

/// Map a campaign operation's failure to its HTTP shape.
fn campaign_error(error: anyhow::Error) -> ApiError {
    if let Some(e) = error.downcast_ref::<licensing::NotEntitled>() {
        return ApiError::NotEntitled(e.clone());
    }
    match error.downcast_ref::<StartError>() {
        Some(StartError::Lifecycle(m)) => return ApiError::Conflict(m.clone()),
        Some(StartError::Invalid(m)) => return ApiError::ValidationError(m.clone()),
        Some(StartError::NotEntitled) | None => {}
    }
    let message = error.to_string();
    if message.contains("not found") {
        ApiError::NotFound(message)
    } else {
        ApiError::ValidationError(message)
    }
}

async fn blocking<T: Send + 'static>(
    work: impl FnOnce() -> Result<T, ApiError> + Send + 'static,
) -> Result<T, ApiError> {
    tokio::task::spawn_blocking(work)
        .await
        .map_err(|e| ApiError::InternalError(e.to_string()))?
}

fn all_tickets(config: &Config) -> Result<Vec<crate::queue::Ticket>, ApiError> {
    let queue = Queue::new(config).map_err(|e| ApiError::InternalError(e.to_string()))?;
    let mut all = Vec::new();
    for list in [
        queue.list_queue(),
        queue.list_in_progress(),
        queue.list_completed(),
    ] {
        all.extend(list.map_err(|e| ApiError::InternalError(e.to_string()))?);
    }
    Ok(all)
}

/// Preview a batch: allocated ids, resolved dependencies, waves and errors. Writes nothing.
#[utoipa::path(post, path = "/api/v1/tickets/batch/preview", operation_id = "tickets_batch_preview",
    tag = "Tickets", request_body = BatchRequest, responses((status = 200, body = BatchPreview)))]
pub async fn batch_preview(
    State(state): State<ApiState>,
    Json(request): Json<BatchRequest>,
) -> Result<Json<BatchPreview>, ApiError> {
    let config = (*state.config()).clone();
    blocking(move || {
        let existing = all_tickets(&config)?;
        let campaign_exists = request
            .campaign
            .as_deref()
            .is_some_and(|id| CampaignStore::new(&config).get(id).is_some());
        Ok(Json(batch::preview(
            &config,
            &existing,
            &request,
            campaign_exists,
        )))
    })
    .await
}

/// Create every ticket in a batch, or none.
#[utoipa::path(post, path = "/api/v1/tickets/batch", operation_id = "tickets_batch_create",
    tag = "Tickets", request_body = BatchRequest,
    responses((status = 200, body = BatchCreated), (status = 400, description = "Batch invalid; the message carries the preview JSON and nothing is written")))]
pub async fn batch_create(
    State(state): State<ApiState>,
    Json(request): Json<BatchRequest>,
) -> Result<Json<BatchCreated>, ApiError> {
    let config = (*state.config()).clone();
    blocking(move || match batch::create(&config, &request) {
        Ok(created) => Ok(Json(created)),
        Err(BatchError::Invalid(preview)) => Err(ApiError::ValidationError(
            serde_json::to_string(&preview).unwrap_or_else(|_| "batch is invalid".into()),
        )),
        Err(e @ BatchError::Write(_)) => Err(ApiError::InternalError(e.to_string())),
    })
    .await
}

#[utoipa::path(get, path = "/api/v1/campaigns", operation_id = "campaigns_list", tag = "Campaigns",
    responses((status = 200, body = CampaignListResponse), (status = 402, description = "Premium required")))]
pub async fn list(State(state): State<ApiState>) -> Result<Json<CampaignListResponse>, ApiError> {
    let config = (*state.config()).clone();
    entitled(&config)?;
    blocking(move || {
        let all = all_tickets(&config)?;
        let campaigns: Vec<CampaignSummary> = CampaignStore::new(&config)
            .list()
            .into_iter()
            .map(|campaign| CampaignSummary {
                members: members(&campaign.id, &all).len(),
                campaign,
            })
            .collect();
        Ok(Json(CampaignListResponse {
            total: campaigns.len(),
            campaigns,
        }))
    })
    .await
}

#[utoipa::path(post, path = "/api/v1/campaigns", operation_id = "campaigns_create", tag = "Campaigns",
    request_body = NewCampaignInput,
    responses((status = 200, body = Campaign), (status = 402, description = "Premium required")))]
pub async fn create(
    State(state): State<ApiState>,
    Json(input): Json<NewCampaignInput>,
) -> Result<Json<Campaign>, ApiError> {
    let config = (*state.config()).clone();
    entitled(&config)?;
    blocking(move || {
        service::create_campaign(&config, input)
            .map(Json)
            .map_err(campaign_error)
    })
    .await
}

#[utoipa::path(get, path = "/api/v1/campaigns/{id}", operation_id = "campaigns_get", tag = "Campaigns",
    params(("id" = String, Path)),
    responses((status = 200, body = CampaignDetail), (status = 402, description = "Premium required")))]
pub async fn get_one(
    State(state): State<ApiState>,
    Path(id): Path<String>,
) -> Result<Json<CampaignDetail>, ApiError> {
    let config = (*state.config()).clone();
    entitled(&config)?;
    blocking(move || {
        service::detail(&config, &id)
            .map(Json)
            .map_err(campaign_error)
    })
    .await
}

#[utoipa::path(put, path = "/api/v1/tickets/{id}/campaign", operation_id = "tickets_assign_campaign",
    tag = "Campaigns", params(("id" = String, Path)), request_body = AssignCampaignRequest,
    responses((status = 200, body = AssignCampaignResponse), (status = 402, description = "Premium required")))]
pub async fn assign(
    State(state): State<ApiState>,
    Path(id): Path<String>,
    Json(request): Json<AssignCampaignRequest>,
) -> Result<Json<AssignCampaignResponse>, ApiError> {
    let config = (*state.config()).clone();
    entitled(&config)?;
    blocking(move || {
        service::assign_ticket(
            &config,
            &id,
            request.campaign.as_deref(),
            request.depends_on,
        )
        .map(|ticket| {
            Json(AssignCampaignResponse {
                id: ticket.id,
                campaign: ticket.campaign,
                depends_on: ticket.depends_on,
            })
        })
        .map_err(campaign_error)
    })
    .await
}

/// Start a campaign and launch its first wave; `dry_run` only reports.
#[utoipa::path(post, path = "/api/v1/campaigns/{id}/launch", operation_id = "campaigns_launch",
    tag = "Campaigns", params(("id" = String, Path)), request_body = LaunchCampaignRequest,
    responses((status = 200, body = CampaignLaunchOutcome), (status = 402, description = "Premium required"),
        (status = 409, description = "Campaign cannot start from its current status")))]
pub async fn launch(
    State(state): State<ApiState>,
    Path(id): Path<String>,
    Json(request): Json<LaunchCampaignRequest>,
) -> Result<Json<CampaignLaunchOutcome>, ApiError> {
    let config = (*state.config()).clone();
    entitled(&config)?;
    let port = LauncherPort::new(config.clone());
    service::launch(&config, &port, &id, request.dry_run)
        .await
        .map(Json)
        .map_err(campaign_error)
}

/// Stop new launches; running agents continue.
#[utoipa::path(post, path = "/api/v1/campaigns/{id}/pause", operation_id = "campaigns_pause",
    tag = "Campaigns", params(("id" = String, Path)),
    responses((status = 200, body = TickReport), (status = 402, description = "Premium required")))]
pub async fn pause(
    State(state): State<ApiState>,
    Path(id): Path<String>,
) -> Result<Json<TickReport>, ApiError> {
    let config = (*state.config()).clone();
    entitled(&config)?;
    blocking(move || {
        campaign_runner::pause_campaign(&config, &id, "paused by operator")
            .map(Json)
            .map_err(campaign_error)
    })
    .await
}

#[utoipa::path(get, path = "/api/v1/campaigns/{id}/status", operation_id = "campaigns_status",
    tag = "Campaigns", params(("id" = String, Path)),
    responses((status = 200, body = CampaignStatusView), (status = 402, description = "Premium required")))]
pub async fn status(
    State(state): State<ApiState>,
    Path(id): Path<String>,
) -> Result<Json<CampaignStatusView>, ApiError> {
    let config = (*state.config()).clone();
    entitled(&config)?;
    blocking(move || {
        let port = LauncherPort::new(config.clone());
        service::status(&config, &port, &id)
            .map(Json)
            .map_err(campaign_error)
    })
    .await
}

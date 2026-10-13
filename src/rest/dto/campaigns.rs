//! DTOs for ticket batches and campaigns. Domain views live in `crate::campaigns`.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;

use crate::campaigns::model::Campaign;

/// Move a ticket into or out of a campaign.
#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct AssignCampaignRequest {
    /// Campaign id; null removes the ticket from its campaign.
    #[serde(default)]
    pub campaign: Option<String>,
    /// Replaces the ticket's `depends_on` when present.
    #[serde(default)]
    pub depends_on: Option<Vec<String>>,
}

/// Launch a campaign, or preview the launch.
#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct LaunchCampaignRequest {
    /// Report the plan, policy and violations without starting.
    #[serde(default)]
    pub dry_run: bool,
}

/// A campaign with its member count.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct CampaignSummary {
    pub campaign: Campaign,
    pub members: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct CampaignListResponse {
    pub campaigns: Vec<CampaignSummary>,
    pub total: usize,
}

/// A ticket's campaign membership after an assignment.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct AssignCampaignResponse {
    pub id: String,
    pub campaign: Option<String>,
    pub depends_on: Vec<String>,
}

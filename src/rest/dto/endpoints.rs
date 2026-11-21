//! DTOs for paired endpoint machines.

use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;

use crate::endpoint::protocol::Capabilities;
use crate::endpoint::registry::Liveness;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct EndpointSummary {
    pub id: String,
    pub name: String,
    pub liveness: Liveness,
    pub created_at: DateTime<Utc>,
    pub last_seen_at: Option<DateTime<Utc>>,
    pub gpu_count: usize,
    /// Admin switch; launches also need the endpoint's own opt-in.
    pub launch_enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct EndpointDetail {
    #[serde(flatten)]
    pub summary: EndpointSummary,
    /// Live while connected, otherwise the last reported.
    pub capabilities: Option<Capabilities>,
    /// The endpoint's heartbeats announce capabilities it has not sent.
    pub capabilities_stale: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct EndpointsResponse {
    pub endpoints: Vec<EndpointSummary>,
    pub total: usize,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct UpdateEndpointRequest {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub launch_enabled: Option<bool>,
}

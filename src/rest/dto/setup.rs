use std::collections::HashMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;

use crate::config::{CollectionPreset, SessionWrapperType};
use crate::startup::steps::SetupStep;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct SetupStatusResponse {
    pub initialized: bool,
    pub admin_configured: bool,
    pub config_path: String,
    pub tickets_path: String,
    pub projects_by_tool: HashMap<String, Vec<String>>,
    pub default_acceptance_criteria: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct SetupStepResponse {
    pub slug: SetupStep,
    pub name: String,
    pub description: String,
    pub help_text: String,
    pub order: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct SetupCollectionResponse {
    pub id: String,
    pub name: String,
    pub description: String,
    pub types: Vec<String>,
    pub default_selected: Vec<String>,
    pub origin: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    pub checksum: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct HostedCollectionSelection {
    pub id: String,
    pub checksum: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[serde(tag = "kind", rename_all = "lowercase")]
#[ts(export)]
pub enum SetupExecutionTarget {
    Local,
    Coder {
        name: String,
        template: String,
        #[serde(default)]
        parameters: HashMap<String, String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct SetupInitializeRequest {
    pub preset: CollectionPreset,
    #[serde(default)]
    pub task_fields: Vec<String>,
    pub wrapper: SessionWrapperType,
    pub execution_target: SetupExecutionTarget,
    #[serde(default)]
    pub use_worktrees: bool,
    pub acceptance_criteria: String,
    #[serde(default)]
    pub model_servers: Vec<String>,
    #[serde(default)]
    pub hosted_collections: Vec<HostedCollectionSelection>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct SetupInitializeResponse {
    pub initialized: bool,
    pub config_path: String,
    pub tickets_path: String,
    pub files_created: Vec<String>,
    pub files_skipped: Vec<String>,
    pub projects: Vec<String>,
}

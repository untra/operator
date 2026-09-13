use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
#[serde(rename_all = "kebab-case")]
pub enum GitOnboardingState {
    CliMissing,
    TokenRequired,
    Authenticated,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct GitProviderOnboardingResponse {
    pub slug: String,
    pub label: String,
    pub docs_url: String,
    pub configured: bool,
    pub command: String,
    pub token_env: String,
    pub state: GitOnboardingState,
    pub action_url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct ValidateGitTokenRequest {
    pub provider: String,
    #[schema(write_only, format = Password)]
    pub token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct ValidateGitTokenResponse {
    pub valid: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct WriteGitConfigRequest {
    pub provider: String,
    pub token_env: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct WriteGitConfigResponse {
    pub provider: String,
    pub token_env: String,
    pub shell_export_block: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct SetGitSessionEnvRequest {
    pub provider: String,
    #[schema(write_only, format = Password)]
    pub token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct SetGitSessionEnvResponse {
    pub shell_export_block: String,
}

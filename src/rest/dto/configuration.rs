use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;

// =============================================================================
// Project DTOs
// =============================================================================

/// Summary of a project with analysis data
#[derive(Debug, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct ProjectSummary {
    /// Project directory name
    pub project_name: String,
    /// Absolute path to project root
    pub project_path: String,
    /// Whether the project directory exists on disk
    pub exists: bool,
    /// Whether catalog-info.yaml exists
    pub has_catalog_info: bool,
    /// Whether project-context.json exists
    pub has_project_context: bool,
    /// Primary Kind from `kind_assessment` (e.g., "microservice")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    /// Kind confidence score 0.0-1.0
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind_confidence: Option<f64>,
    /// Taxonomy tier (e.g., "engines")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind_tier: Option<String>,
    /// Language display names
    pub languages: Vec<String>,
    /// Framework display names
    pub frameworks: Vec<String>,
    /// Database display names
    pub databases: Vec<String>,
    /// Has Dockerfile or docker-compose
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_docker: Option<bool>,
    /// Has test frameworks detected
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_tests: Option<bool>,
    /// Detected port numbers
    pub ports: Vec<u16>,
    /// Number of environment variables
    pub env_var_count: usize,
    /// Number of entry points
    pub entry_point_count: usize,
    /// Available command names (start, dev, test, etc.)
    pub commands: Vec<String>,
}

/// Response from creating an ASSESS ticket
#[derive(Debug, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct AssessTicketResponse {
    /// Ticket ID (e.g., "ASSESS-1234")
    pub ticket_id: String,
    /// Path to the created ticket file
    pub ticket_path: String,
    /// Project name that was assessed
    pub project_name: String,
}

// =============================================================================
// Skills DTOs
// =============================================================================

/// A single discovered skill file
#[derive(Debug, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct SkillEntry {
    /// Tool this skill belongs to (e.g., "claude", "codex")
    pub tool_name: String,
    /// Filename of the skill (e.g., "commit.md")
    pub filename: String,
    /// Full path to the skill file
    pub file_path: String,
    /// Scope: "global" or "project"
    pub scope: String,
}

/// Response for skills listing
#[derive(Debug, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct SkillsResponse {
    /// List of discovered skills
    pub skills: Vec<SkillEntry>,
    /// Total count
    pub total: usize,
}

// =============================================================================
// Delegator DTOs
// =============================================================================

/// Response for a single delegator
#[derive(Debug, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct DelegatorResponse {
    /// Unique name
    pub name: String,
    /// LLM tool name (e.g., "claude")
    pub llm_tool: String,
    /// Model alias (e.g., "opus")
    pub model: String,
    /// Optional display name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    /// Arbitrary model properties
    pub model_properties: std::collections::HashMap<String, String>,
    /// Name of a declared `ModelServer`. `None` means use the `llm_tool`'s implicit vendor default.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_server: Option<String>,
    /// Optional launch configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub launch_config: Option<DelegatorLaunchConfigDto>,
}

/// Request to create a new delegator
#[derive(Debug, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct CreateDelegatorRequest {
    /// Unique name for the delegator
    pub name: String,
    /// LLM tool name (must match a detected tool)
    pub llm_tool: String,
    /// Model alias
    pub model: String,
    /// Optional display name
    #[serde(default)]
    pub display_name: Option<String>,
    /// Arbitrary model properties
    #[serde(default)]
    pub model_properties: std::collections::HashMap<String, String>,
    /// Name of a declared `ModelServer`. `None` means use the `llm_tool`'s implicit vendor default.
    #[serde(default)]
    pub model_server: Option<String>,
    /// Optional launch configuration
    #[serde(default)]
    pub launch_config: Option<DelegatorLaunchConfigDto>,
}

/// Launch configuration DTO for delegators
///
/// Optional fields use tri-state semantics: `None` = inherit global config,
/// `Some(true/false)` = explicit override per-delegator.
#[derive(Debug, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct DelegatorLaunchConfigDto {
    /// Run in YOLO mode
    #[serde(default)]
    pub yolo: bool,
    /// Permission mode override
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub permission_mode: Option<String>,
    /// Additional CLI flags
    #[serde(default)]
    pub flags: Vec<String>,
    /// Override global `git.use_worktrees` (None = use global setting)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub use_worktrees: Option<bool>,
    /// Whether to create a git branch for the ticket (None = default behavior)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub create_branch: Option<bool>,
    /// Run in docker container (None = use global `launch.docker.enabled`)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub docker: Option<bool>,
    /// Prompt text to prepend before the generated step prompt
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt_prefix: Option<String>,
    /// Prompt text to append after the generated step prompt
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt_suffix: Option<String>,
    /// Override global relay auto-inject MCP setting per-delegator (None = use global setting)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operator_relay: Option<bool>,
}

/// Response listing all delegators
#[derive(Debug, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct DelegatorsResponse {
    /// List of delegators
    pub delegators: Vec<DelegatorResponse>,
    /// Total count
    pub total: usize,
}

/// Request to create a delegator from a detected LLM tool
///
/// Pre-populates delegator fields from the detected tool, requiring minimal input.
/// If `name` is omitted, auto-generates as `"{tool_name}-{model}"`.
/// If `model` is omitted, uses the tool's first model alias.
#[derive(Debug, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct CreateDelegatorFromToolRequest {
    /// Name of the detected tool (e.g., "claude", "codex", "gemini")
    pub tool_name: String,
    /// Model alias to use (e.g., "opus"). If omitted, uses the tool's first model alias.
    #[serde(default)]
    pub model: Option<String>,
    /// Custom delegator name. If omitted, auto-generates as `"{tool_name}-{model}"`.
    #[serde(default)]
    pub name: Option<String>,
    /// Optional display name for UI
    #[serde(default)]
    pub display_name: Option<String>,
    /// Name of a declared `ModelServer`. `None` means use the `llm_tool`'s implicit vendor default.
    #[serde(default)]
    pub model_server: Option<String>,
    /// Optional launch configuration
    #[serde(default)]
    pub launch_config: Option<DelegatorLaunchConfigDto>,
}

// =============================================================================
// Model Server DTOs
// =============================================================================

/// Response for a single model server
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct ModelServerResponse {
    /// Unique name (e.g., "ollama-local")
    pub name: String,
    /// Kind: "ollama", "openai-compat", "anthropic-api", "openai-api", "google-api", "lmstudio"
    pub kind: String,
    /// Base URL of the inference endpoint (e.g., `http://localhost:11434`)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    /// Name of an env var providing the API key (e.g., `OLLAMA_API_KEY`)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key_env: Option<String>,
    /// Additional environment variables set when spawning agents that use this server
    pub extra_env: std::collections::HashMap<String, String>,
    /// Optional display name for UI
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    /// Whether this is a user-declared server (true) or an implicit builtin (false)
    pub user_declared: bool,
}

/// Response listing all model servers (declared + implicit builtins)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct ModelServersResponse {
    /// List of model servers
    pub servers: Vec<ModelServerResponse>,
    /// Total count
    pub total: usize,
}

/// Request to create a new model server
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct CreateModelServerRequest {
    /// Unique name for this model server
    pub name: String,
    /// Kind: "ollama", "openai-compat", "anthropic-api", "openai-api", "google-api", "lmstudio"
    pub kind: String,
    /// Base URL of the inference endpoint
    #[serde(default)]
    pub base_url: Option<String>,
    /// Name of an env var providing the API key
    #[serde(default)]
    pub api_key_env: Option<String>,
    /// Additional environment variables
    #[serde(default)]
    pub extra_env: std::collections::HashMap<String, String>,
    /// Optional display name for UI
    #[serde(default)]
    pub display_name: Option<String>,
}

/// Request to update an existing user-declared model server.
///
/// All fields except `kind` are replaced; `name` is taken from the path.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct UpdateModelServerRequest {
    /// Kind: "ollama", "openai-compat", "anthropic-api", "openai-api", "google-api", "lmstudio"
    pub kind: String,
    /// Base URL of the inference endpoint
    #[serde(default)]
    pub base_url: Option<String>,
    /// Name of an env var providing the API key
    #[serde(default)]
    pub api_key_env: Option<String>,
    /// Additional environment variables
    #[serde(default)]
    pub extra_env: std::collections::HashMap<String, String>,
    /// Optional display name for UI
    #[serde(default)]
    pub display_name: Option<String>,
}

/// A model-server kind from the shared catalog (single source of truth).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct ModelServerKindEntry {
    /// Stable slug stored as `ModelServer.kind` (e.g. "ollama")
    pub slug: String,
    /// Human-friendly display name
    pub display_name: String,
    /// One-line connect blurb
    pub description: String,
    /// Help/credential setup page
    pub setup_url: String,
    /// Codicon hint
    pub icon: String,
    /// Whether this is an implicit vendor builtin (always present, not deletable)
    pub is_builtin: bool,
}

/// A single model offered by a server (from a live probe).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct ModelEntry {
    /// Model id passed to `--model`
    pub id: String,
    /// Display name when the server provides one
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
}

/// Response listing the models a server offers, plus reachability.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct ModelServerModelsResponse {
    /// Server name probed
    pub server: String,
    /// Whether the endpoint was reachable
    pub reachable: bool,
    /// Models offered (empty when unreachable)
    pub models: Vec<ModelEntry>,
    /// Error message when unreachable
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

// =============================================================================
// LLM Tools DTOs
// =============================================================================

/// Response listing detected LLM tools
#[derive(Debug, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct LlmToolsResponse {
    /// Detected CLI tools with model aliases and capabilities
    pub tools: Vec<crate::config::DetectedTool>,
    /// Total count
    pub total: usize,
}

/// Request to set the global default LLM tool and model
#[derive(Debug, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct SetDefaultLlmRequest {
    /// Tool name (must match a detected tool, e.g., "claude")
    pub tool: String,
    /// Model alias (e.g., "opus", "sonnet")
    pub model: String,
}

/// Response with the current default LLM tool and model
#[derive(Debug, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct DefaultLlmResponse {
    /// Default tool name (empty string if not set)
    pub tool: String,
    /// Default model alias (empty string if not set)
    pub model: String,
}

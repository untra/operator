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
    /// Declarative reference to a remote, named agent (AGNT, `OpenAI`, ...). When
    /// set, the delegator is export-only and cannot be launched locally.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remote_agent: Option<crate::config::RemoteAgentRef>,
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
    /// Declarative reference to a remote, named agent (AGNT, `OpenAI`, ...). When
    /// set, the delegator is export-only and cannot be launched locally.
    #[serde(default)]
    pub remote_agent: Option<crate::config::RemoteAgentRef>,
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
    /// Provider-class slug grouping kinds within the Model Provider vertical
    /// ("first-party" or "gateway"). Distinct from `is_builtin`. (Field name
    /// kept as `category` for wire/TS stability.)
    pub category: String,
    /// Human-friendly provider-class group header (e.g. "First-party")
    pub category_label: String,
    /// Brand-icon basename (e.g. "ollama") for surfaces that render vendor
    /// logos, or `None` to fall back to the `icon` codicon.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub brand_icon: Option<String>,
    /// Default inference base URL used to probe this provider's models when no
    /// instance declares one, or `None` for bring-your-own-endpoint kinds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_base_url: Option<String>,
    /// Default env var the probe reads to authenticate (e.g. `ANTHROPIC_API_KEY`),
    /// or `None` when no key is needed by default.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_api_key_env: Option<String>,
    /// Whether operator can probe this provider from built-in defaults (it has a
    /// `default_base_url`) without the user first declaring an instance.
    pub connectable: bool,
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_project_summary_skips_optional_kind_fields_when_none() {
        let summary = ProjectSummary {
            project_name: "gamesvc".to_string(),
            project_path: "/repos/gamesvc".to_string(),
            exists: true,
            has_catalog_info: false,
            has_project_context: false,
            kind: None,
            kind_confidence: None,
            kind_tier: None,
            languages: vec!["Rust".to_string()],
            frameworks: vec![],
            databases: vec![],
            has_docker: None,
            has_tests: None,
            ports: vec![6400, 6401],
            env_var_count: 0,
            entry_point_count: 1,
            commands: vec!["test".to_string()],
        };
        let json = serde_json::to_string(&summary).unwrap();
        assert!(!json.contains("kind_confidence"));
        assert!(!json.contains("has_docker"));
        // Non-optional Vec fields stay present even when empty.
        assert!(json.contains("\"frameworks\":[]"));
        assert!(json.contains("\"ports\":[6400,6401]"));

        let parsed: ProjectSummary = serde_json::from_str(&json).unwrap();
        assert!(parsed.kind.is_none());
        assert_eq!(parsed.languages, vec!["Rust".to_string()]);
    }

    #[test]
    fn test_delegator_response_roundtrip() {
        let resp = DelegatorResponse {
            name: "claude-opus".to_string(),
            llm_tool: "claude".to_string(),
            model: "opus".to_string(),
            display_name: Some("Claude Opus".to_string()),
            model_properties: HashMap::new(),
            model_server: None,
            launch_config: None,
            remote_agent: None,
        };
        let json = serde_json::to_string(&resp).unwrap();
        // None optionals are skipped.
        assert!(!json.contains("model_server"));
        assert!(!json.contains("launch_config"));
        assert!(!json.contains("remote_agent"));
        let parsed: DelegatorResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, "claude-opus");
        assert_eq!(parsed.display_name, Some("Claude Opus".to_string()));
    }

    #[test]
    fn test_delegator_launch_config_dto_defaults_when_absent() {
        // yolo defaults to false; tri-state overrides default to None.
        let dto: DelegatorLaunchConfigDto = serde_json::from_str("{}").unwrap();
        assert!(!dto.yolo);
        assert!(dto.permission_mode.is_none());
        assert!(dto.flags.is_empty());
        assert!(dto.use_worktrees.is_none());
        assert!(dto.operator_relay.is_none());
    }

    #[test]
    fn test_model_server_response_carries_api_key_env_name_not_raw_secret() {
        // Secret-by-reference: the DTO references the API key only by env-var NAME
        // (`api_key_env`); it has no field that could carry the raw secret value.
        let resp = ModelServerResponse {
            name: "ollama-local".to_string(),
            kind: "ollama".to_string(),
            base_url: Some("http://localhost:11434".to_string()),
            api_key_env: Some("OLLAMA_API_KEY".to_string()),
            extra_env: HashMap::new(),
            display_name: None,
            user_declared: true,
        };
        let json = serde_json::to_string(&resp).unwrap();
        // The env-var NAME reference is present...
        assert!(json.contains("\"api_key_env\":\"OLLAMA_API_KEY\""));
        // ...but there is no raw `api_key` secret field on the wire.
        assert!(!json.contains("\"api_key\":"));
    }

    #[test]
    fn test_model_server_response_skips_none_optionals() {
        let resp = ModelServerResponse {
            name: "anthropic".to_string(),
            kind: "anthropic-api".to_string(),
            base_url: None,
            api_key_env: None,
            extra_env: HashMap::new(),
            display_name: None,
            user_declared: false,
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(!json.contains("base_url"));
        assert!(!json.contains("api_key_env"));
        assert!(!json.contains("display_name"));
        // extra_env has no skip attribute, so it is always present.
        assert!(json.contains("\"extra_env\":{}"));
    }

    #[test]
    fn test_create_model_server_request_extra_env_defaults_empty() {
        let json = r#"{ "name": "ollama-local", "kind": "ollama" }"#;
        let req: CreateModelServerRequest = serde_json::from_str(json).unwrap();
        assert!(req.extra_env.is_empty());
        assert!(req.base_url.is_none());
        assert!(req.api_key_env.is_none());
    }

    #[test]
    fn test_update_model_server_request_extra_env_defaults_empty() {
        let json = r#"{ "kind": "ollama" }"#;
        let req: UpdateModelServerRequest = serde_json::from_str(json).unwrap();
        assert!(req.extra_env.is_empty());
        assert!(req.display_name.is_none());
    }

    #[test]
    fn test_model_server_kind_entry_skips_none_brand_and_defaults() {
        let entry = ModelServerKindEntry {
            slug: "ollama".to_string(),
            display_name: "Ollama".to_string(),
            description: "Local models".to_string(),
            setup_url: "https://ollama.com".to_string(),
            icon: "server".to_string(),
            is_builtin: true,
            category: "first-party".to_string(),
            category_label: "First-party".to_string(),
            brand_icon: None,
            default_base_url: Some("http://localhost:11434".to_string()),
            default_api_key_env: None,
            connectable: true,
        };
        let json = serde_json::to_string(&entry).unwrap();
        // `category` field name is kept for wire/TS stability.
        assert!(json.contains("\"category\":\"first-party\""));
        assert!(!json.contains("brand_icon"));
        assert!(!json.contains("default_api_key_env"));
        let parsed: ModelServerKindEntry = serde_json::from_str(&json).unwrap();
        assert!(parsed.connectable);
        assert!(parsed.brand_icon.is_none());
    }

    #[test]
    fn test_model_server_models_response_error_absent_when_reachable() {
        let resp = ModelServerModelsResponse {
            server: "ollama-local".to_string(),
            reachable: true,
            models: vec![ModelEntry {
                id: "llama3".to_string(),
                display_name: None,
            }],
            error: None,
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(!json.contains("error"));
        assert!(!json.contains("display_name"));
        assert!(json.contains("\"reachable\":true"));
    }

    #[test]
    fn test_default_llm_response_roundtrip_with_empty_strings() {
        let resp = DefaultLlmResponse {
            tool: String::new(),
            model: String::new(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert_eq!(json, r#"{"tool":"","model":""}"#);
        let parsed: DefaultLlmResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.tool, "");
    }
}

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// LLM CLI tools configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default, JsonSchema, TS)]
#[ts(export)]
pub struct LlmToolsConfig {
    /// Detected CLI tools (populated on first startup)
    #[serde(default)]
    pub detected: Vec<DetectedTool>,

    /// Available {tool, model} pairs for launching tickets
    /// Built from detected tools + their model aliases
    #[serde(default)]
    pub providers: Vec<LlmProvider>,

    /// Whether detection has been completed
    #[serde(default)]
    pub detection_complete: bool,

    /// User's preferred default LLM tool (e.g., "claude")
    #[serde(default)]
    pub default_tool: Option<String>,

    /// User's preferred default model alias (e.g., "opus")
    #[serde(default)]
    pub default_model: Option<String>,

    /// Per-tool overrides for skill directories (keyed by `tool_name`)
    #[serde(default)]
    pub skill_directory_overrides: std::collections::HashMap<String, SkillDirectoriesOverride>,
}

/// A detected CLI tool (e.g., claude binary)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS, utoipa::ToSchema)]
#[ts(export)]
pub struct DetectedTool {
    /// Tool name (e.g., "claude")
    pub name: String,
    /// Path to the binary
    pub path: String,
    /// Version string
    pub version: String,
    /// Minimum required version for Operator compatibility
    #[serde(default)]
    pub min_version: Option<String>,
    /// Whether the installed version meets the minimum requirement
    #[serde(default)]
    pub version_ok: bool,
    /// Available model aliases (e.g., ["opus", "sonnet", "haiku"])
    #[serde(default)]
    pub model_aliases: Vec<String>,
    /// Command template with {{model}}, {{`session_id`}}, {{`prompt_file`}} placeholders
    #[serde(default)]
    pub command_template: String,
    /// Tool capabilities
    #[serde(default)]
    pub capabilities: ToolCapabilities,
    /// CLI flags for YOLO (auto-accept) mode
    #[serde(default)]
    pub yolo_flags: Vec<String>,
}

/// Tool capabilities
#[derive(Debug, Clone, Serialize, Deserialize, Default, JsonSchema, TS, utoipa::ToSchema)]
#[ts(export)]
pub struct ToolCapabilities {
    /// Whether the tool supports session continuity via UUID
    #[serde(default)]
    pub supports_sessions: bool,
    /// Whether the tool can run in headless/non-interactive mode
    #[serde(default)]
    pub supports_headless: bool,
}

/// A {tool, model} pair that can be selected when launching tickets.
/// Includes optional variant fields adopted from vibe-kanban's profile system.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, JsonSchema, TS)]
#[ts(export)]
pub struct LlmProvider {
    /// CLI tool name (e.g., "claude", "codex", "gemini")
    pub tool: String,
    /// Model alias or name (e.g., "opus", "sonnet", "gpt-4.1")
    pub model: String,
    /// Optional display name for UI (e.g., "Claude Opus", "Codex High")
    #[serde(default)]
    pub display_name: Option<String>,

    // ─── Variant fields (all optional) ───────────────────────────────
    /// Additional CLI flags for this provider (e.g., ["--dangerously-skip-permissions"])
    #[serde(default)]
    pub flags: Vec<String>,

    /// Environment variables to set when launching
    #[serde(default)]
    pub env: std::collections::HashMap<String, String>,

    /// Whether this provider requires approval gates
    #[serde(default)]
    pub approvals: bool,

    /// Whether to run in plan-only mode
    #[serde(default)]
    pub plan_only: bool,

    /// Reasoning effort level (Codex: "low", "medium", "high")
    #[serde(default)]
    pub reasoning_effort: Option<String>,

    /// Sandbox mode (Codex: "danger-full-access", "workspace-write")
    #[serde(default)]
    pub sandbox: Option<String>,
}

/// Per-tool skill directory overrides
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export)]
pub struct SkillDirectoriesOverride {
    /// Additional global skill directories
    #[serde(default)]
    pub global: Vec<String>,
    /// Additional project-relative skill directories
    #[serde(default)]
    pub project: Vec<String>,
}

/// A declarative reference to a remote, named agent hosted by another platform.
///
/// `platform` is the hosting service (`"agnt"`, `"openai"`) — deliberately
/// distinct from the core `provider`/`llm_tool` (the model or coding CLI). These
/// agents are API/memory-native and live on the remote side; Operator has no
/// runtime client for them, so a delegator carrying one is **export-only** and
/// cannot be launched locally (see the guard in `delegator_resolution`).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS, utoipa::ToSchema)]
#[ts(export)]
pub struct RemoteAgentRef {
    /// Hosting platform (e.g. `"agnt"`, `"openai"`).
    pub platform: String,
    /// Platform-native agent identifier (e.g. an AGNT agent name, an `OpenAI` `asst_…` id).
    pub id: String,
}

/// Agent delegator configuration for autonomous ticket launching
///
/// A delegator is a named {tool, model} pairing with optional launch configuration
/// that can be used to launch agents for tickets.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export)]
pub struct Delegator {
    /// Unique name for this delegator (e.g., "claude-opus-auto")
    pub name: String,
    /// LLM tool name (must match a detected tool, e.g., "claude", "codex")
    pub llm_tool: String,
    /// Model alias (e.g., "opus", "sonnet", "gpt-4o")
    pub model: String,
    /// Optional display name for UI
    #[serde(default)]
    pub display_name: Option<String>,
    /// Arbitrary model properties (e.g., `reasoning_effort`, sandbox)
    #[serde(default)]
    pub model_properties: std::collections::HashMap<String, String>,
    /// Optional launch configuration
    #[serde(default)]
    pub launch_config: Option<DelegatorLaunchConfig>,
    /// Name of a declared `ModelServer` (from `Config.model_servers`).
    /// `None` means use the `llm_tool`'s implicit vendor default
    /// (claude → anthropic-api, codex → openai-api, gemini → google-api).
    #[serde(default)]
    pub model_server: Option<String>,
    /// Declarative reference to a remote, named agent on another platform
    /// (e.g. an AGNT agent or an `OpenAI` Assistant; see [`crate::config::AgentProfile`]).
    ///
    /// Export-only: Operator has no runtime client for those platforms, so a
    /// delegator carrying this CANNOT be launched locally — resolution errors out
    /// (see `delegator_resolution`). It is stored, listed, serialized into an
    /// `AgentProfile`, and — for `platform == "agnt"` — surfaced in the
    /// `--format agnt` workflow export as an `agnt-agent` node. `None` = ordinary,
    /// locally launchable delegator.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remote_agent: Option<RemoteAgentRef>,
    /// Opaque AGNT-namespaced extension fields, preserved verbatim across an
    /// `AgentProfile` round-trip so re-export is lossless (e.g. `memory`,
    /// `assignedWorkflows`, `creditLimit`). Operator never interprets this.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub x_agnt: Option<serde_json::Value>,
    /// Opaque OpenAI-namespaced extension fields, preserved verbatim across an
    /// `AgentProfile` round-trip (e.g. `instructions`, `tools`, `tool_resources`,
    /// `metadata`, thread refs). Mirror of [`Self::x_agnt`]; never interpreted.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub x_openai: Option<serde_json::Value>,
    /// Opaque carry for `AgentProfile` shared-core fields Operator cannot model
    /// first-class (`system_prompt` / `skills` / `mcp_servers` / `tools`) so an
    /// import→export round-trip is lossless. Distinct from `x_agnt`: these are
    /// shared-core fields, not AGNT-specific, so folding them into `x_agnt` would
    /// corrupt that namespace. Operator never interprets this.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unmapped_core: Option<serde_json::Value>,
}

/// A named host that serves models via an inference API.
///
/// Model servers are orthogonal to `llm_tools`: a delegator pairs an agentic CLI
/// (`llm_tool`, e.g. claude/codex/gemini) with a model-serving endpoint
/// (`model_server`, e.g. ollama-local, openai-api, a custom vllm host).
///
/// Implicit builtin servers (`anthropic-api`, `openai-api`, `google-api`) are
/// returned by [`implicit_model_server_for_tool`] and do not need to be declared
/// in config.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export)]
pub struct ModelServer {
    /// Unique name (e.g., "ollama-local", "vllm-gpu1")
    pub name: String,
    /// Kind: "ollama", "openrouter", "openai-compat", "anthropic-api", "openai-api", "google-api", "lmstudio"
    pub kind: String,
    /// Base URL of the inference endpoint (e.g., `http://localhost:11434`).
    /// `None` for implicit vendor servers means use the SDK default.
    #[serde(default)]
    pub base_url: Option<String>,
    /// Name of an env var providing the API key (e.g., `OLLAMA_API_KEY`)
    #[serde(default)]
    pub api_key_env: Option<String>,
    /// Additional environment variables set when spawning agents that use this server
    #[serde(default)]
    pub extra_env: std::collections::HashMap<String, String>,
    /// Optional display name for UI
    #[serde(default)]
    pub display_name: Option<String>,
}

/// Returns the implicit builtin `ModelServer` associated with a given `llm_tool`.
///
/// Used when a `Delegator` has no explicit `model_server`. Unknown tools
/// fall back to an `"openai-api"` server so arbitrary future tools still resolve.
pub fn implicit_model_server_for_tool(tool: &str) -> ModelServer {
    let (name, kind) = match tool {
        "claude" => ("anthropic-api", "anthropic-api"),
        "codex" => ("openai-api", "openai-api"),
        "gemini" => ("google-api", "google-api"),
        _ => ("openai-api", "openai-api"),
    };
    ModelServer {
        name: name.to_string(),
        kind: kind.to_string(),
        base_url: None,
        api_key_env: None,
        extra_env: std::collections::HashMap::new(),
        display_name: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delegator_deserializes_without_new_fields() {
        // Pre-existing config JSON that has none of remote_agent/x_agnt/x_openai/unmapped_core.
        let json = r#"{
            "name": "claude-opus",
            "llm_tool": "claude",
            "model": "opus"
        }"#;
        let d: Delegator = serde_json::from_str(json).expect("legacy delegator still deserializes");
        assert_eq!(d.name, "claude-opus");
        assert!(d.remote_agent.is_none());
        assert!(d.x_agnt.is_none());
        assert!(d.x_openai.is_none());
        assert!(d.unmapped_core.is_none());
    }

    #[test]
    fn delegator_serializes_omits_none_new_fields() {
        let d = Delegator {
            name: "claude-opus".to_string(),
            llm_tool: "claude".to_string(),
            model: "opus".to_string(),
            display_name: None,
            model_properties: std::collections::HashMap::new(),
            launch_config: None,
            model_server: None,
            remote_agent: None,
            x_agnt: None,
            x_openai: None,
            unmapped_core: None,
        };
        let v = serde_json::to_value(&d).unwrap();
        assert!(
            v.get("remote_agent").is_none(),
            "None remote_agent is omitted"
        );
        assert!(v.get("x_agnt").is_none(), "None x_agnt is omitted");
        assert!(v.get("x_openai").is_none(), "None x_openai is omitted");
        assert!(
            v.get("unmapped_core").is_none(),
            "None unmapped_core is omitted"
        );
    }
}

/// Launch configuration for a delegator
///
/// Controls how the delegator launches agents. Optional fields use tri-state
/// semantics: `None` = inherit from global config, `Some(true/false)` = override.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema, TS, utoipa::ToSchema)]
#[ts(export)]
pub struct DelegatorLaunchConfig {
    /// Run in YOLO (auto-accept) mode
    #[serde(default)]
    pub yolo: bool,
    /// Permission mode override
    #[serde(default)]
    pub permission_mode: Option<String>,
    /// Additional CLI flags
    #[serde(default)]
    pub flags: Vec<String>,
    /// Override global `git.use_worktrees` per-delegator (None = use global setting)
    #[serde(default)]
    pub use_worktrees: Option<bool>,
    /// Whether to create a git branch for the ticket (None = default behavior)
    #[serde(default)]
    pub create_branch: Option<bool>,
    /// Run in docker container (None = use global `launch.docker.enabled`)
    #[serde(default)]
    pub docker: Option<bool>,
    /// Prompt text to prepend before the generated step prompt
    #[serde(default)]
    pub prompt_prefix: Option<String>,
    /// Prompt text to append after the generated step prompt
    #[serde(default)]
    pub prompt_suffix: Option<String>,
    /// Override global relay auto-inject MCP setting per-delegator (None = use global setting)
    #[serde(default)]
    pub operator_relay: Option<bool>,
}

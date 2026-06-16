//! A canonical, cross-tool **agent interchange profile** (`agent-profile.json`).
//!
//! Operator's [`Delegator`] and an AGNT.gg *Agent* are the same primitive seen
//! from two sides: a Delegator is CLI/git-native (it spawns a real coding CLI
//! with a permission sandbox), an AGNT Agent is API/memory-native (provider
//! calls + memory). They share a large core. Rather than merge them, this module
//! defines a namespaced interchange format both sides can serialize to and from
//! *losslessly*: a shared core, an Operator-namespaced bag (`x_operator`), and an
//! AGNT-namespaced bag (`x_agnt`). Each side reads the core and its own bag, and
//! preserves the other side's bag verbatim — the same lossy-but-honest discipline
//! as the `OPERATOR-GAP` markers in [`crate::workflow_gen`].
//!
//! This is the schema half of the remote-agent bridge. There is deliberately
//! **no** runtime client for any remote platform: a profile carrying
//! [`AgentProfile::remote_agent`] is a *declarative* reference — surfaced in the
//! `--format agnt` export when its platform is AGNT, but never executed by
//! Operator (see the launch guard in `delegator_resolution`).

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::llm_tools::{Delegator, DelegatorLaunchConfig, RemoteAgentRef};

/// A canonical cross-tool agent definition (serialized as `agent-profile.json`).
///
/// Shared core (`name`/`provider`/`model`/`system_prompt`/`skills`/`mcp_servers`/
/// `tools`) plus namespaced extension bags. `x_operator` is typed (Operator owns
/// those fields); `x_agnt` and `x_openai` are opaque per-platform pass-throughs.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS, utoipa::ToSchema)]
#[ts(export)]
pub struct AgentProfile {
    /// Unique agent name (maps to [`Delegator::name`]).
    pub name: String,
    /// Inference provider / CLI tool (maps to [`Delegator::llm_tool`]).
    pub provider: String,
    /// Model alias or id (maps to [`Delegator::model`]).
    pub model: String,
    /// System prompt. Operator has no first-class system prompt, so this is
    /// preserved opaquely across import (see [`Delegator::unmapped_core`]).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub system_prompt: Option<String>,
    /// Named skills. Preserved opaquely across import.
    #[serde(default)]
    pub skills: Vec<String>,
    /// MCP server names. Preserved opaquely across import.
    #[serde(default)]
    pub mcp_servers: Vec<String>,
    /// Tool names. Preserved opaquely across import.
    #[serde(default)]
    pub tools: Vec<String>,
    /// Declarative reference to a remote, named agent (AGNT, `OpenAI`, ...).
    /// `None` = a locally launchable agent, not bound to a remote platform.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remote_agent: Option<RemoteAgentRef>,
    /// Operator-owned extension fields (typed). `None` when the agent carries no
    /// Operator-specific configuration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub x_operator: Option<XOperator>,
    /// AGNT-owned extension fields, opaque (`memory`, `assignedWorkflows`,
    /// `creditLimit`, ...). Operator never interprets this — pure pass-through.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub x_agnt: Option<serde_json::Value>,
    /// OpenAI-owned extension fields, opaque (`instructions`, `tools`,
    /// `tool_resources`, `metadata`, thread refs, ...). Mirror of `x_agnt` for a
    /// second platform — never interpreted. This field is the whole per-tool cost
    /// of adding `OpenAI`: a passthrough bag, no mapping logic.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub x_openai: Option<serde_json::Value>,
}

/// The Operator-namespaced half of an [`AgentProfile`] — the fields a Delegator
/// carries that have no shared-core equivalent. AGNT ignores this bag; Operator
/// round-trips it losslessly.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema, TS, utoipa::ToSchema)]
#[ts(export)]
pub struct XOperator {
    /// Optional display name for UI.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    /// Arbitrary model properties (e.g. `reasoning_effort`, sandbox).
    #[serde(default, skip_serializing_if = "std::collections::HashMap::is_empty")]
    pub model_properties: std::collections::HashMap<String, String>,
    /// Name of a declared `ModelServer` (`None` = implicit vendor default).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_server: Option<String>,
    /// Launch configuration (permission mode, flags, worktree/docker, prompt
    /// wrapping, ...).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub launch_config: Option<DelegatorLaunchConfig>,
}

impl XOperator {
    /// Whether this bag carries any Operator-specific data worth serializing.
    fn is_empty(&self) -> bool {
        self.display_name.is_none()
            && self.model_properties.is_empty()
            && self.model_server.is_none()
            && self.launch_config.is_none()
    }
}

/// The shared-core fields Operator cannot model first-class. Stashed verbatim
/// into [`Delegator::unmapped_core`] on import so a later export restores them.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct UnmappedCore {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    system_prompt: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    skills: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    mcp_servers: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    tools: Vec<String>,
}

impl UnmappedCore {
    fn is_empty(&self) -> bool {
        self.system_prompt.is_none()
            && self.skills.is_empty()
            && self.mcp_servers.is_empty()
            && self.tools.is_empty()
    }
}

/// Serialize a [`Delegator`] into an [`AgentProfile`].
///
/// Operator-only fields go into `x_operator`; the opaque per-platform bags
/// (`x_agnt`, `x_openai`) and the shared-core carry ([`Delegator::unmapped_core`])
/// are restored so a profile that *originated* on a remote platform re-exports
/// identically. A delegator authored natively in Operator has no `unmapped_core`,
/// so the shared-core fields export empty (`OPERATOR-GAP`: Operator has no
/// first-class `system_prompt`/`skills`/`tools`).
pub fn delegator_to_profile(d: &Delegator) -> AgentProfile {
    let core: UnmappedCore = d
        .unmapped_core
        .as_ref()
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();

    let x_operator = XOperator {
        display_name: d.display_name.clone(),
        model_properties: d.model_properties.clone(),
        model_server: d.model_server.clone(),
        launch_config: d.launch_config.clone(),
    };

    AgentProfile {
        name: d.name.clone(),
        provider: d.llm_tool.clone(),
        model: d.model.clone(),
        system_prompt: core.system_prompt,
        skills: core.skills,
        mcp_servers: core.mcp_servers,
        tools: core.tools,
        remote_agent: d.remote_agent.clone(),
        x_operator: if x_operator.is_empty() {
            None
        } else {
            Some(x_operator)
        },
        x_agnt: d.x_agnt.clone(),
        x_openai: d.x_openai.clone(),
    }
}

/// Import an [`AgentProfile`] into a [`Delegator`].
///
/// Shared-core fields Operator can't model (`system_prompt`/`skills`/
/// `mcp_servers`/`tools`) are stashed into [`Delegator::unmapped_core`] as opaque
/// JSON; the opaque per-platform bags (`x_agnt`, `x_openai`) are passed through.
/// All ensure a subsequent [`delegator_to_profile`] is lossless.
pub fn profile_to_delegator(p: &AgentProfile) -> Delegator {
    let x = p.x_operator.clone().unwrap_or_default();

    let core = UnmappedCore {
        system_prompt: p.system_prompt.clone(),
        skills: p.skills.clone(),
        mcp_servers: p.mcp_servers.clone(),
        tools: p.tools.clone(),
    };
    let unmapped_core = if core.is_empty() {
        None
    } else {
        serde_json::to_value(&core).ok()
    };

    Delegator {
        name: p.name.clone(),
        llm_tool: p.provider.clone(),
        model: p.model.clone(),
        display_name: x.display_name,
        model_properties: x.model_properties,
        launch_config: x.launch_config,
        model_server: x.model_server,
        remote_agent: p.remote_agent.clone(),
        x_agnt: p.x_agnt.clone(),
        x_openai: p.x_openai.clone(),
        unmapped_core,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn json(d: &Delegator) -> serde_json::Value {
        serde_json::to_value(d).unwrap()
    }

    fn profile_json(p: &AgentProfile) -> serde_json::Value {
        serde_json::to_value(p).unwrap()
    }

    fn full_delegator() -> Delegator {
        let mut props = std::collections::HashMap::new();
        props.insert("reasoning_effort".to_string(), "high".to_string());
        Delegator {
            name: "claude-opus-auto".to_string(),
            llm_tool: "claude".to_string(),
            model: "opus".to_string(),
            display_name: Some("Claude Opus (Auto)".to_string()),
            model_properties: props,
            launch_config: Some(DelegatorLaunchConfig {
                yolo: true,
                permission_mode: Some("plan".to_string()),
                flags: vec!["--verbose".to_string()],
                use_worktrees: Some(true),
                create_branch: Some(false),
                docker: Some(true),
                prompt_prefix: Some("PREFIX".to_string()),
                prompt_suffix: Some("SUFFIX".to_string()),
                operator_relay: Some(true),
            }),
            model_server: Some("anthropic-api".to_string()),
            remote_agent: None,
            x_agnt: None,
            x_openai: None,
            unmapped_core: None,
        }
    }

    #[test]
    fn delegator_to_profile_maps_core_fields() {
        let d = full_delegator();
        let p = delegator_to_profile(&d);
        assert_eq!(p.name, "claude-opus-auto");
        assert_eq!(p.provider, "claude");
        assert_eq!(p.model, "opus");
    }

    #[test]
    fn delegator_to_profile_puts_operator_only_fields_in_x_operator() {
        let d = full_delegator();
        let p = delegator_to_profile(&d);
        let x = p
            .x_operator
            .expect("operator-only data lives in x_operator");
        assert_eq!(x.display_name.as_deref(), Some("Claude Opus (Auto)"));
        assert_eq!(x.model_server.as_deref(), Some("anthropic-api"));
        assert_eq!(x.model_properties.get("reasoning_effort").unwrap(), "high");
        assert!(x.launch_config.unwrap().yolo);
        // Shared core fields Operator can't model export empty.
        assert!(p.system_prompt.is_none());
        assert!(p.skills.is_empty());
    }

    #[test]
    fn delegator_with_no_operator_data_has_no_x_operator() {
        let d = Delegator {
            name: "bare".to_string(),
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
        let p = delegator_to_profile(&d);
        assert!(p.x_operator.is_none());
    }

    #[test]
    fn delegator_roundtrip_lossless_for_operator_fields() {
        let d = full_delegator();
        let round_tripped = profile_to_delegator(&delegator_to_profile(&d));
        assert_eq!(
            json(&d),
            json(&round_tripped),
            "Delegator -> profile -> Delegator must be lossless"
        );
    }

    #[test]
    fn profile_roundtrip_preserves_x_agnt_opaque() {
        let p = AgentProfile {
            name: "agnt-researcher".to_string(),
            provider: "anthropic".to_string(),
            model: "claude-3-5-sonnet".to_string(),
            system_prompt: None,
            skills: vec![],
            mcp_servers: vec![],
            tools: vec![],
            remote_agent: Some(RemoteAgentRef {
                platform: "agnt".to_string(),
                id: "Research Assistant".to_string(),
            }),
            x_operator: None,
            x_agnt: Some(serde_json::json!({
                "memory": { "window": 10, "vectorStore": "pinecone" },
                "assignedWorkflows": ["wf-1", "wf-2"],
                "creditLimit": 1000
            })),
            x_openai: None,
        };
        let round_tripped = delegator_to_profile(&profile_to_delegator(&p));
        assert_eq!(
            profile_json(&p),
            profile_json(&round_tripped),
            "x_agnt must survive profile -> Delegator -> profile byte-for-byte"
        );
    }

    #[test]
    fn openai_profile_roundtrips_with_x_openai() {
        // The structural twin of the x_agnt test, for a second platform — proving
        // the per-tool cost is exactly one opaque bag + the generic remote ref.
        let p = AgentProfile {
            name: "openai-reviewer".to_string(),
            provider: "openai".to_string(),
            model: "gpt-4o".to_string(),
            system_prompt: None,
            skills: vec![],
            mcp_servers: vec![],
            tools: vec![],
            remote_agent: Some(RemoteAgentRef {
                platform: "openai".to_string(),
                id: "asst_abc123".to_string(),
            }),
            x_operator: None,
            x_agnt: None,
            x_openai: Some(serde_json::json!({
                "instructions": "You are a careful reviewer.",
                "tools": [{ "type": "file_search" }],
                "tool_resources": { "file_search": { "vector_store_ids": ["vs_1"] } },
                "metadata": { "team": "platform" }
            })),
        };
        let round_tripped = delegator_to_profile(&profile_to_delegator(&p));
        assert_eq!(
            profile_json(&p),
            profile_json(&round_tripped),
            "x_openai must survive profile -> Delegator -> profile byte-for-byte"
        );
    }

    #[test]
    fn profile_roundtrip_preserves_shared_core() {
        let p = AgentProfile {
            name: "careful-reviewer".to_string(),
            provider: "anthropic".to_string(),
            model: "claude-3-5-sonnet".to_string(),
            system_prompt: Some("You are a careful reviewer.".to_string()),
            skills: vec!["rust-testing".to_string()],
            mcp_servers: vec!["github".to_string()],
            tools: vec!["web-search".to_string()],
            remote_agent: None,
            x_operator: None,
            x_agnt: None,
            x_openai: None,
        };
        let round_tripped = delegator_to_profile(&profile_to_delegator(&p));
        assert_eq!(
            profile_json(&p),
            profile_json(&round_tripped),
            "shared-core fields Operator can't model must survive round-trip"
        );
    }

    #[test]
    fn profile_with_remote_agent_survives_roundtrip() {
        let p = AgentProfile {
            name: "agnt-researcher".to_string(),
            provider: "agnt".to_string(),
            model: "default".to_string(),
            system_prompt: None,
            skills: vec![],
            mcp_servers: vec![],
            tools: vec![],
            remote_agent: Some(RemoteAgentRef {
                platform: "agnt".to_string(),
                id: "Research Assistant".to_string(),
            }),
            x_operator: None,
            x_agnt: None,
            x_openai: None,
        };
        let d = profile_to_delegator(&p);
        let r = d.remote_agent.as_ref().expect("remote_agent preserved");
        assert_eq!(r.platform, "agnt");
        assert_eq!(r.id, "Research Assistant");
        let back = delegator_to_profile(&d);
        let rb = back
            .remote_agent
            .as_ref()
            .expect("remote_agent re-exported");
        assert_eq!(rb.platform, "agnt");
        assert_eq!(rb.id, "Research Assistant");
    }
}

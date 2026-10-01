//! Per-vertical structural support and cross-vertical compatibility.
//!
//! [`SupportStatus`] is the advertising ramp (Proto/Alpha/Beta/GA). This
//! module is the *facts*: what Operator can actually do with an entry, and
//! whether an LLM tool can speak a model server's protocol without a bridge.
//!
//! Distinct from [`crate::integrations::inventory`] (slash/MCP/REST/TUI
//! surface parity).

use super::catalog::Vertical;

/// Inference wire protocol. Shared by LLM tools (native set) and model
/// servers (exactly one).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InferenceProtocol {
    Anthropic,
    OpenAi,
    Google,
}

impl InferenceProtocol {
    pub fn slug(self) -> &'static str {
        match self {
            Self::Anthropic => "anthropic",
            Self::OpenAi => "openai",
            Self::Google => "google",
        }
    }
}

impl ToolHealth {
    pub fn slug(self) -> &'static str {
        match self {
            Self::PathVersion => "path-version",
            Self::HealthCommand => "health-command",
            Self::AuthProbe => "auth-probe",
            Self::Unverifiable => "unverifiable",
        }
    }
}

impl ToolAuth {
    pub fn slug(self) -> &'static str {
        match self {
            Self::OauthAndKey => "oauth-and-key",
            Self::ApiKey => "api-key",
        }
    }
}

impl RemoteInventory {
    pub fn slug(self) -> &'static str {
        match self {
            Self::Opr8rTools => "opr8r-tools",
            Self::CommandV => "command-v",
            Self::None => "none",
        }
    }
}

impl RelaySupport {
    pub fn slug(self) -> &'static str {
        match self {
            Self::Supported => "supported",
            Self::Partial => "partial",
            Self::None => "none",
        }
    }
}

impl PermissionsSupport {
    pub fn slug(self) -> &'static str {
        match self {
            Self::Wired => "wired",
            Self::Deferred => "deferred",
        }
    }
}

impl ProviderClass {
    pub fn slug(self) -> &'static str {
        match self {
            Self::FirstParty => "first-party",
            Self::Gateway => "gateway",
        }
    }
}

impl KeyInjectable {
    pub fn slug(self) -> &'static str {
        match self {
            Self::Yes => "yes",
            Self::Optional => "optional",
            Self::No => "no",
        }
    }
}

impl Coverage {
    pub fn slug(self) -> &'static str {
        match self {
            Self::Partial => "partial",
            Self::Full => "full",
        }
    }
}

/// How Operator knows an LLM CLI is usable on a host.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolHealth {
    /// `which` + version command. Not proof of login or API key.
    PathVersion,
    /// `detection.health_command` (always-mode / remote-only tools).
    HealthCommand,
    /// Operator can prove the CLI's own credential works.
    AuthProbe,
    /// Always-mode with nothing locally verifiable.
    Unverifiable,
}

/// How a tool authenticates for launch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolAuth {
    OauthAndKey,
    ApiKey,
}

/// How a remote target reports this binary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemoteInventory {
    Opr8rTools,
    CommandV,
    None,
}

/// Operator relay MCP injection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelaySupport {
    Supported,
    Partial,
    None,
}

/// Permission-translator wiring into launch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionsSupport {
    Wired,
    Deferred,
}

/// First-party vendor vs gateway/host. Mirrors
/// [`crate::api::providers::model_server::ModelProviderClass`] as a catalog fact.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderClass {
    FirstParty,
    Gateway,
}

/// Whether `env_for_server` can inject an API key reference.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyInjectable {
    Yes,
    Optional,
    No,
}

/// Sparse coverage for verticals that do not yet have rich axes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Coverage {
    Partial,
    Full,
}

/// LLM tool product claims (not invocation flags from `tools/*.json`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LlmToolSupport {
    pub health: ToolHealth,
    pub auth: ToolAuth,
    pub native_protocols: &'static [InferenceProtocol],
    pub sessions: bool,
    pub headless: bool,
    pub yolo: bool,
    pub remote_inventory: RemoteInventory,
    pub relay: RelaySupport,
    pub permissions: PermissionsSupport,
}

/// Model provider product claims.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModelProviderSupport {
    pub protocol: InferenceProtocol,
    pub class: ProviderClass,
    pub probe: bool,
    pub connectable_from_defaults: bool,
    pub key_injectable: KeyInjectable,
    /// Binary name this kind is the implicit default for, if any.
    pub implicit_for: Option<&'static str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KanbanSupport {
    pub sync_in: bool,
    pub write_back: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GitSupport {
    pub pr_cli: bool,
    pub remote_preflight: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SessionSupport {
    pub attach: bool,
    pub send_keys: bool,
    pub idle_detect: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RemoteTargetSupport {
    pub probe: bool,
    pub tool_inventory: bool,
    pub credential_injection: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SparseSupport {
    pub coverage: Coverage,
    pub notes: Option<&'static str>,
}

/// Typed support record, one variant per catalog vertical.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerticalSupport {
    LlmTool(LlmToolSupport),
    Model(ModelProviderSupport),
    Kanban(KanbanSupport),
    Git(GitSupport),
    Session(SessionSupport),
    Editor(SparseSupport),
    Platform(SparseSupport),
    Integration(SparseSupport),
    Workflows(SparseSupport),
    Notification(SparseSupport),
    Transport(RemoteTargetSupport),
    AgentRelay(SparseSupport),
    RemoteTargets(RemoteTargetSupport),
}

/// LLM tool × model server protocol relationship.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Compatibility {
    Native,
    Bridge { note: &'static str },
    Incompatible { reason: &'static str },
}

impl Compatibility {
    pub fn slug(self) -> &'static str {
        match self {
            Self::Native => "native",
            Self::Bridge { .. } => "bridge",
            Self::Incompatible { .. } => "incompatible",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Native => "Native",
            Self::Bridge { .. } => "Bridge",
            Self::Incompatible { .. } => "Incompatible",
        }
    }
}

const BRIDGE_GATEWAY: &str =
    "protocol-preserving front (e.g. claude-code-router, litellm) in front of this gateway";
const BRIDGE_OPENAI_FIRST_PARTY: &str = "Anthropic-protocol proxy in front of the OpenAI API";
const INCOMPATIBLE_FIRST_PARTY: &str =
    "first-party API speaks a different protocol than this CLI natively does";

/// Support record for a catalog `(vertical, slug)`, or a sparse default.
pub fn support_for(vertical: Vertical, slug: &str) -> VerticalSupport {
    match (vertical, slug) {
        (Vertical::LlmTool, slug) => llm_tool_supports()
            .iter()
            .find(|(s, _)| *s == slug)
            .map(|(_, support)| VerticalSupport::LlmTool(*support))
            .unwrap_or(VerticalSupport::LlmTool(UNKNOWN_LLM)),
        (Vertical::Model, slug) => model_supports()
            .iter()
            .find(|(s, _)| *s == slug)
            .map(|(_, support)| VerticalSupport::Model(*support))
            .unwrap_or(VerticalSupport::Model(UNKNOWN_MODEL)),
        (Vertical::Kanban, "operator") => VerticalSupport::Kanban(KanbanSupport {
            sync_in: true,
            write_back: true,
        }),
        (Vertical::Kanban, "jira" | "linear" | "github") => {
            VerticalSupport::Kanban(KanbanSupport {
                sync_in: true,
                write_back: true,
            })
        }
        (Vertical::Kanban, "openspec") => VerticalSupport::Kanban(KanbanSupport {
            sync_in: true,
            write_back: false,
        }),
        (Vertical::Git, _) => VerticalSupport::Git(GitSupport {
            pr_cli: true,
            remote_preflight: true,
        }),
        (Vertical::Session, _) => VerticalSupport::Session(SessionSupport {
            attach: true,
            send_keys: true,
            idle_detect: true,
        }),
        (Vertical::RemoteTargets, "ssh") => VerticalSupport::RemoteTargets(RemoteTargetSupport {
            probe: true,
            tool_inventory: true,
            credential_injection: true,
        }),
        (Vertical::RemoteTargets, "coder") => VerticalSupport::RemoteTargets(RemoteTargetSupport {
            probe: true,
            tool_inventory: false,
            credential_injection: false,
        }),
        (Vertical::Transport, "ssh") => VerticalSupport::Transport(RemoteTargetSupport {
            probe: true,
            tool_inventory: true,
            credential_injection: true,
        }),
        (Vertical::Transport, "local") => VerticalSupport::Transport(RemoteTargetSupport {
            probe: true,
            tool_inventory: true,
            credential_injection: false,
        }),
        (Vertical::Editor, _) => VerticalSupport::Editor(sparse_partial(None)),
        (Vertical::Platform, _) => VerticalSupport::Platform(sparse_partial(None)),
        (Vertical::Integration, _) => VerticalSupport::Integration(sparse_partial(None)),
        (Vertical::Workflows, _) => VerticalSupport::Workflows(sparse_partial(None)),
        (Vertical::Notification, _) => VerticalSupport::Notification(sparse_partial(None)),
        (Vertical::AgentRelay, _) => VerticalSupport::AgentRelay(sparse_partial(Some(
            "Relay MCP injection is Claude-first; Codex is partial.",
        ))),

        (Vertical::Kanban, _) => VerticalSupport::Kanban(KanbanSupport {
            sync_in: false,
            write_back: false,
        }),
        (Vertical::Transport, _) => VerticalSupport::Transport(RemoteTargetSupport {
            probe: false,
            tool_inventory: false,
            credential_injection: false,
        }),
        (Vertical::RemoteTargets, _) => VerticalSupport::RemoteTargets(RemoteTargetSupport {
            probe: false,
            tool_inventory: false,
            credential_injection: false,
        }),
    }
}

fn sparse_partial(notes: Option<&'static str>) -> SparseSupport {
    SparseSupport {
        coverage: Coverage::Partial,
        notes,
    }
}

const CLAUDE: LlmToolSupport = LlmToolSupport {
    health: ToolHealth::PathVersion,
    auth: ToolAuth::OauthAndKey,
    native_protocols: &[InferenceProtocol::Anthropic],
    sessions: true,
    headless: false,
    yolo: true,
    remote_inventory: RemoteInventory::Opr8rTools,
    relay: RelaySupport::Supported,
    permissions: PermissionsSupport::Deferred,
};

const CODEX: LlmToolSupport = LlmToolSupport {
    health: ToolHealth::PathVersion,
    auth: ToolAuth::ApiKey,
    native_protocols: &[InferenceProtocol::OpenAi],
    sessions: true,
    headless: true,
    yolo: true,
    remote_inventory: RemoteInventory::Opr8rTools,
    relay: RelaySupport::Partial,
    permissions: PermissionsSupport::Deferred,
};

const GEMINI: LlmToolSupport = LlmToolSupport {
    health: ToolHealth::PathVersion,
    auth: ToolAuth::ApiKey,
    native_protocols: &[InferenceProtocol::Google],
    sessions: true,
    headless: true,
    yolo: true,
    remote_inventory: RemoteInventory::Opr8rTools,
    relay: RelaySupport::None,
    permissions: PermissionsSupport::Deferred,
};

const GROK: LlmToolSupport = LlmToolSupport {
    health: ToolHealth::PathVersion,
    auth: ToolAuth::OauthAndKey,
    native_protocols: &[InferenceProtocol::OpenAi],
    sessions: true,
    headless: true,
    yolo: true,
    remote_inventory: RemoteInventory::Opr8rTools,
    relay: RelaySupport::None,
    permissions: PermissionsSupport::Deferred,
};

const UNKNOWN_LLM: LlmToolSupport = LlmToolSupport {
    health: ToolHealth::Unverifiable,
    auth: ToolAuth::ApiKey,
    native_protocols: &[],
    sessions: false,
    headless: false,
    yolo: false,
    remote_inventory: RemoteInventory::None,
    relay: RelaySupport::None,
    permissions: PermissionsSupport::Deferred,
};

const ANTHROPIC: ModelProviderSupport = ModelProviderSupport {
    protocol: InferenceProtocol::Anthropic,
    class: ProviderClass::FirstParty,
    probe: true,
    connectable_from_defaults: true,
    key_injectable: KeyInjectable::Yes,
    implicit_for: Some("claude"),
};

const OPENAI: ModelProviderSupport = ModelProviderSupport {
    protocol: InferenceProtocol::OpenAi,
    class: ProviderClass::FirstParty,
    probe: true,
    connectable_from_defaults: true,
    key_injectable: KeyInjectable::Yes,
    implicit_for: Some("codex"),
};

const GOOGLE: ModelProviderSupport = ModelProviderSupport {
    protocol: InferenceProtocol::Google,
    class: ProviderClass::FirstParty,
    probe: true,
    connectable_from_defaults: true,
    key_injectable: KeyInjectable::Yes,
    implicit_for: Some("gemini"),
};

const XAI: ModelProviderSupport = ModelProviderSupport {
    protocol: InferenceProtocol::OpenAi,
    class: ProviderClass::FirstParty,
    probe: true,
    connectable_from_defaults: true,
    key_injectable: KeyInjectable::Yes,
    implicit_for: Some("grok"),
};

const OLLAMA: ModelProviderSupport = ModelProviderSupport {
    protocol: InferenceProtocol::OpenAi,
    class: ProviderClass::Gateway,
    probe: true,
    connectable_from_defaults: true,
    key_injectable: KeyInjectable::Optional,
    implicit_for: None,
};

const OPENROUTER: ModelProviderSupport = ModelProviderSupport {
    protocol: InferenceProtocol::OpenAi,
    class: ProviderClass::Gateway,
    probe: true,
    connectable_from_defaults: true,
    key_injectable: KeyInjectable::Yes,
    implicit_for: None,
};

const OPENAI_COMPAT: ModelProviderSupport = ModelProviderSupport {
    protocol: InferenceProtocol::OpenAi,
    class: ProviderClass::Gateway,
    probe: true,
    connectable_from_defaults: false,
    key_injectable: KeyInjectable::Yes,
    implicit_for: None,
};

const LMSTUDIO: ModelProviderSupport = ModelProviderSupport {
    protocol: InferenceProtocol::OpenAi,
    class: ProviderClass::Gateway,
    probe: true,
    connectable_from_defaults: false,
    key_injectable: KeyInjectable::Optional,
    implicit_for: None,
};

const UNKNOWN_MODEL: ModelProviderSupport = ModelProviderSupport {
    protocol: InferenceProtocol::OpenAi,
    class: ProviderClass::Gateway,
    probe: false,
    connectable_from_defaults: false,
    key_injectable: KeyInjectable::No,
    implicit_for: None,
};

fn llm_support_for_id(id: &str) -> Option<LlmToolSupport> {
    match id {
        "claude" => Some(CLAUDE),
        "codex" => Some(CODEX),
        "gemini" | "gemini-cli" => Some(GEMINI),
        "grok" => Some(GROK),
        _ => None,
    }
}

fn model_support_for_kind(kind: &str) -> Option<ModelProviderSupport> {
    model_supports()
        .iter()
        .find(|(slug, _)| *slug == kind)
        .map(|(_, support)| *support)
}

/// Protocol compatibility of a shipped LLM tool (catalog slug or binary)
/// with a model-server kind slug.
pub fn compatibility(tool_id: &str, model_kind: &str) -> Option<Compatibility> {
    let tool = llm_support_for_id(tool_id)?;
    let model = model_support_for_kind(model_kind)?;
    if tool.native_protocols.contains(&model.protocol) {
        return Some(Compatibility::Native);
    }
    if model.class == ProviderClass::Gateway {
        return Some(Compatibility::Bridge {
            note: BRIDGE_GATEWAY,
        });
    }
    if tool
        .native_protocols
        .contains(&InferenceProtocol::Anthropic)
        && model.protocol == InferenceProtocol::OpenAi
        && model.class == ProviderClass::FirstParty
    {
        return Some(Compatibility::Bridge {
            note: BRIDGE_OPENAI_FIRST_PARTY,
        });
    }
    Some(Compatibility::Incompatible {
        reason: INCOMPATIBLE_FIRST_PARTY,
    })
}

/// Shipped LLM tools in catalog display order (slug, support).
pub fn llm_tool_supports() -> &'static [(&'static str, LlmToolSupport)] {
    &[
        ("claude", CLAUDE),
        ("codex", CODEX),
        ("gemini-cli", GEMINI),
        ("grok", GROK),
    ]
}

/// Model kinds that have a support row, in catalog display order.
pub fn model_supports() -> &'static [(&'static str, ModelProviderSupport)] {
    &[
        ("anthropic-api", ANTHROPIC),
        ("openai-api", OPENAI),
        ("google-api", GOOGLE),
        ("xai-api", XAI),
        ("ollama", OLLAMA),
        ("openrouter", OPENROUTER),
        ("openai-compat", OPENAI_COMPAT),
        ("lmstudio", LMSTUDIO),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_claude_ollama_is_bridge() {
        let c = compatibility("claude", "ollama").unwrap();
        assert_eq!(c.slug(), "bridge");
    }

    #[test]
    fn test_codex_ollama_is_native() {
        assert_eq!(
            compatibility("codex", "ollama"),
            Some(Compatibility::Native)
        );
    }

    #[test]
    fn test_claude_google_is_incompatible() {
        let c = compatibility("claude", "google-api").unwrap();
        assert_eq!(c.slug(), "incompatible");
    }

    #[test]
    fn test_claude_openai_api_is_bridge() {
        assert_eq!(
            compatibility("claude", "openai-api").unwrap().slug(),
            "bridge"
        );
    }

    #[test]
    fn test_gemini_openai_api_is_incompatible() {
        assert_eq!(
            compatibility("gemini", "openai-api").unwrap().slug(),
            "incompatible"
        );
        assert_eq!(
            compatibility("gemini-cli", "openai-api").unwrap().slug(),
            "incompatible"
        );
    }

    #[test]
    fn test_gemini_ollama_is_bridge() {
        assert_eq!(compatibility("gemini", "ollama").unwrap().slug(), "bridge");
    }

    #[test]
    fn test_codex_anthropic_is_incompatible() {
        assert_eq!(
            compatibility("codex", "anthropic-api").unwrap().slug(),
            "incompatible"
        );
    }

    #[test]
    fn test_implicit_pairs_are_native() {
        assert_eq!(
            compatibility("claude", "anthropic-api"),
            Some(Compatibility::Native)
        );
        assert_eq!(
            compatibility("codex", "openai-api"),
            Some(Compatibility::Native)
        );
        assert_eq!(
            compatibility("gemini", "google-api"),
            Some(Compatibility::Native)
        );
        assert_eq!(
            compatibility("grok", "xai-api"),
            Some(Compatibility::Native)
        );
    }

    #[test]
    fn test_compatibility_matrix_table() {
        let cases = [
            ("claude", "anthropic-api", "native"),
            ("claude", "openai-api", "bridge"),
            ("claude", "google-api", "incompatible"),
            ("claude", "xai-api", "bridge"),
            ("claude", "ollama", "bridge"),
            ("codex", "anthropic-api", "incompatible"),
            ("codex", "openai-api", "native"),
            ("codex", "xai-api", "native"),
            ("codex", "ollama", "native"),
            ("gemini", "google-api", "native"),
            ("gemini", "openai-api", "incompatible"),
            ("gemini", "xai-api", "incompatible"),
            ("gemini", "ollama", "bridge"),
            ("grok", "xai-api", "native"),
            ("grok", "openai-api", "native"),
            ("grok", "ollama", "native"),
            ("grok", "anthropic-api", "incompatible"),
            ("grok", "google-api", "incompatible"),
        ];
        for (tool, kind, want) in cases {
            let got = compatibility(tool, kind).map(Compatibility::slug);
            assert_eq!(got, Some(want), "{tool} × {kind}");
        }
    }

    #[test]
    fn test_unknown_tool_has_no_matrix_row() {
        assert!(compatibility("agy", "ollama").is_none());
    }

    #[test]
    fn test_claude_health_is_path_version_not_auth_probe() {
        match support_for(Vertical::LlmTool, "claude") {
            VerticalSupport::LlmTool(s) => {
                assert_eq!(s.health, ToolHealth::PathVersion);
                assert_ne!(s.health, ToolHealth::AuthProbe);
                assert!(!s.headless);
                assert_eq!(s.native_protocols, &[InferenceProtocol::Anthropic]);
            }
            other => panic!("expected LlmTool, got {other:?}"),
        }
    }

    #[test]
    fn test_anthropic_probe_is_stronger_than_tool_health() {
        match support_for(Vertical::Model, "anthropic-api") {
            VerticalSupport::Model(s) => {
                assert!(s.probe);
                assert_eq!(s.implicit_for, Some("claude"));
                assert_eq!(s.protocol, InferenceProtocol::Anthropic);
            }
            other => panic!("expected Model, got {other:?}"),
        }
    }

    #[test]
    fn test_ga_llm_bar_claude() {
        match support_for(Vertical::LlmTool, "claude") {
            VerticalSupport::LlmTool(s) => {
                assert!(s.sessions);
                assert!(s.yolo);
                assert!(!s.native_protocols.is_empty());
                assert_eq!(s.health, ToolHealth::PathVersion);
            }
            other => panic!("expected LlmTool, got {other:?}"),
        }
    }
}

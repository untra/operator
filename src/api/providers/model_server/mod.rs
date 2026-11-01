#![allow(dead_code)]

//! Model server provider catalog and live model-listing probe.
//!
//! A *model server* is a named inference endpoint a delegator can target
//! (anthropic-api / openai-api / google-api builtins, plus user-declared
//! ollama / openai-compat / lmstudio hosts). The [`ModelServerKind`] enum is the
//! single source of truth for the closed set of supported protocols - every
//! surface (TUI status section, web `/#/model-servers` projection, the REST
//! catalog endpoint, and the VS Code status tree) derives its list from
//! [`ModelServerKind::ALL`] so the options can't drift apart.
//!
//! This mirrors the `KanbanProviderType` pattern in
//! [`crate::api::providers::kanban`].

mod probe;

use std::collections::{BTreeSet, HashMap};

pub use probe::{probe_models, ModelInfo, ProbeError, ProbeOutcome};

use crate::config::ModelServer;

/// Build the environment variables an agent CLI needs to target this server.
///
/// This is the mapping the `TODO(model-servers-v2)` called for: a resolved
/// [`ModelServer`] → the env vars exported before the agent spawns.
///
/// - `base_url` → the protocol's [`ModelServerKind::base_url_env_var`].
/// - `api_key_env` (if set) → a **shell indirection** `${api_key_env}` under the
///   protocol's [`ModelServerKind::api_key_env_var`]. The spawned script inherits
///   operator's environment, so the reference resolves at run time and the secret
///   value is never written to the on-disk command file.
/// - `extra_env` → passed through verbatim as the user's explicit escape hatch
///   (takes precedence over the derived vars).
///
/// Pure (no environment reads), so the secret never transits this function.
/// Returns an empty map for implicit builtins with no `base_url` - preserving the
/// vendor-default path exactly as before.
pub fn env_for_server(server: &ModelServer) -> HashMap<String, String> {
    let mut env = HashMap::new();

    if let Some(kind) = ModelServerKind::from_slug(&server.kind) {
        if let Some(base) = server.base_url.as_deref().filter(|u| !u.is_empty()) {
            env.insert(
                kind.base_url_env_var().to_string(),
                kind.spawn_base_url(base),
            );
        }
        // Map the user's key var to the canonical one the CLI reads, by reference
        // (no value copy). When they already match this is a harmless self-export.
        if let Some(var) = server.api_key_env.as_deref().filter(|v| !v.is_empty()) {
            env.insert(kind.api_key_env_var().to_string(), format!("${{{var}}}"));
        }
    }

    for (k, v) in &server.extra_env {
        env.insert(k.clone(), v.clone());
    }

    env
}

const LM_STUDIO_PROVIDER: &str = "operator_lmstudio";
const LM_STUDIO_TOKEN_ENV: &str = "OPERATOR_LM_STUDIO_TOKEN";
const LM_STUDIO_PLACEHOLDER_TOKEN: &str = "lmstudio";
const CLAUDE_TOOL: &str = "claude";
const CODEX_TOOL: &str = "codex";
const ANTHROPIC_AUTH_TOKEN_ENV: &str = "ANTHROPIC_AUTH_TOKEN";
const CLAUDE_ATTRIBUTION_HEADER_ENV: &str = "CLAUDE_CODE_ATTRIBUTION_HEADER";

/// Every env var a model-server launch setting can export. A step that switches
/// delegators unsets the ones its own provider does not set, so a baseline
/// server's endpoint cannot leak into the next step.
pub fn launch_env_vars() -> BTreeSet<&'static str> {
    ModelServerKind::ALL
        .iter()
        .flat_map(|kind| [kind.base_url_env_var(), kind.api_key_env_var()])
        .chain([
            ANTHROPIC_AUTH_TOKEN_ENV,
            CLAUDE_ATTRIBUTION_HEADER_ENV,
            LM_STUDIO_TOKEN_ENV,
        ])
        .collect()
}

#[derive(Debug, Default)]
pub struct ModelServerLaunchSettings {
    pub env: HashMap<String, String>,
    pub flags: Vec<String>,
}

pub fn openai_v1_base(base: &str) -> String {
    let base = base.trim_end_matches('/');
    if base.ends_with("/v1") {
        base.to_string()
    } else {
        format!("{base}/v1")
    }
}

pub(crate) fn openai_root(base: &str) -> &str {
    let base = base.trim_end_matches('/');
    base.strip_suffix("/v1").unwrap_or(base)
}

pub fn launch_settings_for_server(
    server: &ModelServer,
    tool: &str,
) -> anyhow::Result<ModelServerLaunchSettings> {
    let mut settings = ModelServerLaunchSettings {
        env: env_for_server(server),
        flags: Vec::new(),
    };
    if server.kind != ModelServerKind::LmStudio.slug() {
        return Ok(settings);
    }
    let base = server
        .base_url
        .as_deref()
        .filter(|base| !base.is_empty())
        .ok_or_else(|| anyhow::anyhow!("LM Studio server '{}' requires base_url", server.name))?;
    let url = url::Url::parse(base)?;
    anyhow::ensure!(
        matches!(url.scheme(), "http" | "https"),
        "LM Studio base_url requires http or https"
    );
    let key = server.api_key_env.as_deref().filter(|key| !key.is_empty());
    match tool {
        CLAUDE_TOOL => {
            settings.env = HashMap::from([
                (
                    ModelServerKind::AnthropicApi.base_url_env_var().into(),
                    openai_root(base).into(),
                ),
                (
                    ANTHROPIC_AUTH_TOKEN_ENV.into(),
                    key.map_or_else(
                        || LM_STUDIO_PLACEHOLDER_TOKEN.into(),
                        |key| format!("${{{key}}}"),
                    ),
                ),
                (
                    ModelServerKind::AnthropicApi.api_key_env_var().into(),
                    String::new(),
                ),
                (CLAUDE_ATTRIBUTION_HEADER_ENV.into(), "0".into()),
            ]);
        }
        CODEX_TOOL => {
            settings.env.insert(
                ModelServerKind::LmStudio.api_key_env_var().into(),
                String::new(),
            );
            if let Some(key) = key {
                settings
                    .env
                    .insert(LM_STUDIO_TOKEN_ENV.into(), format!("${{{key}}}"));
            }
            let base = settings
                .env
                .get(ModelServerKind::LmStudio.base_url_env_var())
                .cloned()
                .unwrap_or_else(|| openai_v1_base(base));
            let base = toml::Value::String(base);
            let auth = if key.is_some() {
                format!(",env_key=\"{LM_STUDIO_TOKEN_ENV}\"")
            } else {
                String::new()
            };
            settings.flags = vec![
                "-c".into(), format!("model_provider=\"{LM_STUDIO_PROVIDER}\""),
                "-c".into(), format!("model_providers.{LM_STUDIO_PROVIDER}={{name=\"LM Studio\",base_url={base},wire_api=\"responses\",requires_openai_auth=false{auth}}}"),
            ];
        }
        _ => {}
    }
    settings.env.extend(server.extra_env.clone());
    Ok(settings)
}

/// The sub-class of the *Model Provider* vertical a [`ModelServerKind`] belongs
/// to. Every kind is a model provider; this groups them by *who makes the
/// models*.
///
/// Single source of truth for the grouping every surface renders (README
/// badges, docs nav, the REST `/kinds` catalog, the web Model Providers view,
/// and the VS Code section).
///
/// Distinct from [`ModelServerKind::is_builtin`] - `is_builtin` governs
/// delete-protection / the zero-config implicit default, whereas
/// `provider_class` is about *first-party vendor* vs *gateway/host*. They happen
/// to partition the same way today, but they answer different questions, so both
/// are kept.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelProviderClass {
    /// A company that produces its own models, served from its own API
    /// (Anthropic, `OpenAI`, Google).
    FirstParty,
    /// A gateway or host that fronts other parties' models behind one endpoint
    /// (`OpenRouter`, ollama, any `OpenAI`-compatible server, LM Studio).
    Gateway,
}

impl ModelProviderClass {
    /// Stable wire slug, carried on the REST `/kinds` catalog entry.
    pub fn slug(&self) -> &'static str {
        match self {
            ModelProviderClass::FirstParty => "first-party",
            ModelProviderClass::Gateway => "gateway",
        }
    }

    /// Human-friendly group header shown in catalog UIs.
    pub fn display_name(&self) -> &'static str {
        match self {
            ModelProviderClass::FirstParty => "First-party",
            ModelProviderClass::Gateway => "Gateways",
        }
    }
}

/// Read the server's API key from **this process's** environment: the
/// instance's named env var, else the kind's default (`ANTHROPIC_API_KEY` /
/// `OPENAI_API_KEY` / `GEMINI_API_KEY` / ...). Only for requests the daemon
/// itself makes (model probes, native LLM calls); agent spawns get the key by
/// reference via [`env_for_server`] instead.
pub fn resolve_api_key(server: &ModelServer, kind: ModelServerKind) -> Option<String> {
    server
        .api_key_env
        .as_deref()
        .or_else(|| kind.default_api_key_env())
        .and_then(|var| std::env::var(var).ok())
        .filter(|k| !k.is_empty())
}

/// A model-server protocol kind.
///
/// `OpenAiCompat` is the explicit catch-all for any OpenAI-API-compatible server
/// (vllm, groq, together.ai, …). Distinct from a *server instance*, which is
/// open and user-named (`ModelServer.name`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelServerKind {
    AnthropicApi,
    OpenAiApi,
    GoogleApi,
    XaiApi,
    Ollama,
    OpenRouter,
    OpenAiCompat,
    LmStudio,
}

impl ModelServerKind {
    pub fn spawn_base_url(self, base: &str) -> String {
        match self {
            Self::OpenAiApi | Self::XaiApi | Self::OpenAiCompat | Self::LmStudio | Self::Ollama => {
                openai_v1_base(base)
            }
            _ => base.to_string(),
        }
    }

    pub fn models_url(self, base: &str) -> String {
        let base = match self {
            Self::OpenAiApi | Self::XaiApi | Self::OpenAiCompat | Self::LmStudio | Self::Ollama => {
                openai_root(base)
            }
            _ => base.trim_end_matches('/'),
        };
        format!("{base}{}", self.models_endpoint())
    }

    /// The canonical list of supported model-server kinds, in display order.
    ///
    /// Single source of truth - every surface derives its catalog from here.
    pub const ALL: [ModelServerKind; 8] = [
        ModelServerKind::AnthropicApi,
        ModelServerKind::OpenAiApi,
        ModelServerKind::GoogleApi,
        ModelServerKind::XaiApi,
        ModelServerKind::Ollama,
        ModelServerKind::OpenRouter,
        ModelServerKind::OpenAiCompat,
        ModelServerKind::LmStudio,
    ];

    /// Which sub-class of the Model Provider vertical this kind belongs to -
    /// drives the grouping across every surface.
    pub fn provider_class(&self) -> ModelProviderClass {
        match self {
            ModelServerKind::AnthropicApi
            | ModelServerKind::OpenAiApi
            | ModelServerKind::GoogleApi
            | ModelServerKind::XaiApi => ModelProviderClass::FirstParty,
            ModelServerKind::Ollama
            | ModelServerKind::OpenRouter
            | ModelServerKind::OpenAiCompat
            | ModelServerKind::LmStudio => ModelProviderClass::Gateway,
        }
    }

    /// Stable wire slug - matches the `kind` string stored on
    /// [`crate::config::ModelServer`] and used in config, the REST catalog, and
    /// the `ConfigureModelServer` action.
    pub fn slug(&self) -> &'static str {
        match self {
            ModelServerKind::AnthropicApi => "anthropic-api",
            ModelServerKind::OpenAiApi => "openai-api",
            ModelServerKind::GoogleApi => "google-api",
            ModelServerKind::XaiApi => "xai-api",
            ModelServerKind::Ollama => "ollama",
            ModelServerKind::OpenRouter => "openrouter",
            ModelServerKind::OpenAiCompat => "openai-compat",
            ModelServerKind::LmStudio => "lmstudio",
        }
    }

    /// Parse a kind from its [`slug`](Self::slug) (i.e. a `ModelServer.kind`).
    pub fn from_slug(slug: &str) -> Option<ModelServerKind> {
        ModelServerKind::ALL.into_iter().find(|k| k.slug() == slug)
    }

    /// Human-friendly display name.
    pub fn display_name(&self) -> &'static str {
        match self {
            ModelServerKind::AnthropicApi => "Anthropic API",
            ModelServerKind::OpenAiApi => "OpenAI API",
            ModelServerKind::GoogleApi => "Google Gemini API",
            ModelServerKind::XaiApi => "xAI API",
            ModelServerKind::Ollama => "Ollama",
            ModelServerKind::OpenRouter => "OpenRouter",
            ModelServerKind::OpenAiCompat => "OpenAI-compatible",
            ModelServerKind::LmStudio => "LM Studio",
        }
    }

    /// Whether this kind is one of the implicit vendor builtins
    /// (`anthropic-api` / `openai-api` / `google-api`) that always exist and
    /// cannot be deleted.
    pub fn is_builtin(&self) -> bool {
        matches!(
            self,
            ModelServerKind::AnthropicApi
                | ModelServerKind::OpenAiApi
                | ModelServerKind::GoogleApi
                | ModelServerKind::XaiApi
        )
    }

    /// One-line "connect" blurb shown next to the kind in catalog rows.
    pub fn connect_blurb(&self) -> &'static str {
        match self {
            ModelServerKind::AnthropicApi => "Anthropic Console or a compatible proxy",
            ModelServerKind::OpenAiApi => "OpenAI or a compatible proxy",
            ModelServerKind::GoogleApi => "Google Gemini API",
            ModelServerKind::XaiApi => "xAI Console (Grok)",
            ModelServerKind::Ollama => "Local ollama server (ollama serve)",
            ModelServerKind::OpenRouter => {
                "Hosted gateway to 300+ models (one OpenAI-compatible key)"
            }
            ModelServerKind::OpenAiCompat => "Any OpenAI-compatible server (vllm, groq, …)",
            ModelServerKind::LmStudio => "LM Studio's local server",
        }
    }

    /// Help/credential page opened by the "Configure" action and the web link.
    pub fn setup_url(&self) -> &'static str {
        match self {
            ModelServerKind::AnthropicApi => "https://console.anthropic.com/settings/keys",
            ModelServerKind::OpenAiApi => "https://platform.openai.com/api-keys",
            ModelServerKind::GoogleApi => "https://aistudio.google.com/app/apikey",
            ModelServerKind::XaiApi => "https://console.x.ai/",
            ModelServerKind::Ollama => "https://ollama.com/download",
            ModelServerKind::OpenRouter => "https://openrouter.ai/keys",
            ModelServerKind::OpenAiCompat => {
                "https://operator.untra.io/getting-started/model-servers/"
            }
            ModelServerKind::LmStudio => "https://lmstudio.ai/",
        }
    }

    /// Codicon hint (rendered as `$(icon)` in VS Code, `codicon-{icon}` on web).
    pub fn icon(&self) -> &'static str {
        match self {
            // Hosted endpoints (vendor APIs + the OpenRouter gateway).
            ModelServerKind::AnthropicApi
            | ModelServerKind::OpenAiApi
            | ModelServerKind::GoogleApi
            | ModelServerKind::XaiApi
            | ModelServerKind::OpenRouter => "cloud",
            // Self-hosted / local servers.
            ModelServerKind::Ollama | ModelServerKind::OpenAiCompat | ModelServerKind::LmStudio => {
                "server"
            }
        }
    }

    /// Brand-icon basename for surfaces that render vendor logos, or `None` to
    /// fall back to the semantic [`icon`](Self::icon) codicon.
    ///
    /// One basename feeds every surface: docs map it to
    /// `/assets/icons/{b}.svg`, VS Code to the `operator-{b}` `ThemeIcon`, and the
    /// web UI to `/icons/{b}.svg` - so the brand set can't drift between them.
    /// `openai-api` deliberately stays on a codicon (no first-party logo asset).
    pub fn brand_icon(&self) -> Option<&'static str> {
        match self {
            ModelServerKind::AnthropicApi => Some("anthropic"),
            ModelServerKind::GoogleApi => Some("google"),
            ModelServerKind::XaiApi => Some("xai"),
            ModelServerKind::Ollama => Some("ollama"),
            ModelServerKind::OpenRouter => Some("openrouter"),
            ModelServerKind::LmStudio => Some("lmstudio"),
            ModelServerKind::OpenAiApi | ModelServerKind::OpenAiCompat => None,
        }
    }

    /// Path appended to a server's `base_url` to list the models it serves.
    ///
    /// The protocol determines the shape of the response - see
    /// [`probe::probe_models`] for parsing. Endpoints reflect each vendor's
    /// documented model-list route.
    pub fn models_endpoint(&self) -> &'static str {
        match self {
            // ollama's native tag list
            ModelServerKind::Ollama => "/api/tags",
            // OpenRouter's documented base already ends in `/api/v1`, so the
            // model list is just `/models` relative to it.
            ModelServerKind::OpenRouter => "/models",
            // OpenAI-protocol model list
            ModelServerKind::OpenAiApi
            | ModelServerKind::XaiApi
            | ModelServerKind::OpenAiCompat
            | ModelServerKind::LmStudio => "/v1/models",
            // Anthropic's model list
            ModelServerKind::AnthropicApi => "/v1/models",
            // Gemini's model list
            ModelServerKind::GoogleApi => "/v1beta/models",
        }
    }

    /// The env var an agent CLI reads to override its inference base URL.
    ///
    /// Keyed by *protocol* (the kind), not the tool: the SDK that reads the var
    /// is determined by which API protocol the server speaks. e.g. codex talking
    /// to ollama uses `OPENAI_BASE_URL` because ollama speaks the `OpenAI`
    /// protocol; claude via an anthropic bridge uses `ANTHROPIC_BASE_URL`.
    pub fn base_url_env_var(&self) -> &'static str {
        match self {
            ModelServerKind::AnthropicApi => "ANTHROPIC_BASE_URL",
            ModelServerKind::OpenAiApi
            | ModelServerKind::XaiApi
            | ModelServerKind::OpenAiCompat
            | ModelServerKind::Ollama
            | ModelServerKind::OpenRouter
            | ModelServerKind::LmStudio => "OPENAI_BASE_URL",
            ModelServerKind::GoogleApi => "GOOGLE_GEMINI_BASE_URL",
        }
    }

    /// The canonical env var an agent CLI reads for its API key, by protocol.
    pub fn api_key_env_var(&self) -> &'static str {
        match self {
            ModelServerKind::AnthropicApi => "ANTHROPIC_API_KEY",
            ModelServerKind::XaiApi => "XAI_API_KEY",
            ModelServerKind::OpenAiApi
            | ModelServerKind::OpenAiCompat
            | ModelServerKind::Ollama
            | ModelServerKind::OpenRouter
            | ModelServerKind::LmStudio => "OPENAI_API_KEY",
            ModelServerKind::GoogleApi => "GEMINI_API_KEY",
        }
    }

    /// The default inference base URL used to **probe** a provider for its live
    /// model list when no instance declares one. `None` means the provider must
    /// be declared with an explicit `base_url` before it can be probed
    /// (`openai-compat` / `lmstudio` are bring-your-own-endpoint).
    ///
    /// Probe-only: this is **never** injected into the agent spawn environment -
    /// see [`env_for_server`], which only exports a `base_url` a server sets
    /// explicitly, preserving the vendor-default / OAuth launch path for the
    /// implicit builtins.
    pub fn default_base_url(&self) -> Option<&'static str> {
        match self {
            ModelServerKind::AnthropicApi => Some("https://api.anthropic.com"),
            ModelServerKind::OpenAiApi => Some("https://api.openai.com"),
            ModelServerKind::GoogleApi => Some("https://generativelanguage.googleapis.com"),
            ModelServerKind::XaiApi => Some("https://api.x.ai"),
            ModelServerKind::OpenRouter => Some("https://openrouter.ai/api/v1"),
            ModelServerKind::Ollama => Some("http://localhost:11434"),
            ModelServerKind::OpenAiCompat | ModelServerKind::LmStudio => None,
        }
    }

    /// The default env var the **probe** reads to authenticate when an instance
    /// doesn't name one. `None` means no key is needed by default (a local
    /// ollama server) or the provider must declare its own.
    ///
    /// Distinct from [`api_key_env_var`](Self::api_key_env_var), which is the
    /// canonical var the *agent CLI* reads at spawn - this is the var the
    /// *operator probe* reads from its own environment to list models.
    pub fn default_api_key_env(&self) -> Option<&'static str> {
        match self {
            ModelServerKind::AnthropicApi => Some("ANTHROPIC_API_KEY"),
            ModelServerKind::OpenAiApi => Some("OPENAI_API_KEY"),
            ModelServerKind::GoogleApi => Some("GEMINI_API_KEY"),
            ModelServerKind::XaiApi => Some("XAI_API_KEY"),
            ModelServerKind::OpenRouter => Some("OPENROUTER_API_KEY"),
            ModelServerKind::Ollama | ModelServerKind::OpenAiCompat | ModelServerKind::LmStudio => {
                None
            }
        }
    }

    /// Whether operator can probe this provider's models from built-in defaults
    /// (i.e. it has a [`default_base_url`](Self::default_base_url)) without the
    /// user first declaring an instance. `false` for bring-your-own-endpoint
    /// kinds that need an explicit `base_url`.
    pub fn connectable_from_defaults(&self) -> bool {
        self.default_base_url().is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lmstudio_missing_endpoint_fails_instead_of_using_cloud() {
        let server = server("lmstudio", None);
        for tool in ["codex", "claude"] {
            assert!(launch_settings_for_server(&server, tool).is_err());
        }
    }

    #[test]
    fn test_lmstudio_overrides_and_unauthenticated_provider() {
        let mut server = server("lmstudio", Some("http://pc:1234"));
        assert!(resolve_api_key(&server, ModelServerKind::LmStudio).is_none());
        let settings = launch_settings_for_server(&server, "codex").unwrap();
        assert!(settings.flags.iter().all(|flag| !flag.contains("env_key")));
        assert_eq!(settings.env["OPENAI_API_KEY"], "");
        server
            .extra_env
            .insert("OPENAI_BASE_URL".into(), "https://pc.example/v1".into());
        server
            .extra_env
            .insert("ANTHROPIC_BASE_URL".into(), "https://pc.example".into());
        assert!(launch_settings_for_server(&server, "codex")
            .unwrap()
            .flags
            .iter()
            .any(|flag| flag.contains("https://pc.example/v1")));
        assert_eq!(
            launch_settings_for_server(&server, "claude").unwrap().env["ANTHROPIC_BASE_URL"],
            "https://pc.example"
        );
    }

    #[test]
    fn test_spawn_bases_preserve_other_protocols() {
        for (kind, base) in [
            (ModelServerKind::AnthropicApi, "https://api.anthropic.com"),
            (ModelServerKind::OpenRouter, "https://openrouter.ai/api/v1"),
            (
                ModelServerKind::GoogleApi,
                "https://generativelanguage.googleapis.com",
            ),
        ] {
            assert_eq!(kind.spawn_base_url(base), base);
        }
    }

    #[test]
    fn test_lmstudio_launch_settings_are_tool_specific_and_secret_references() {
        let server = ModelServer {
            name: "pc".into(),
            kind: "lmstudio".into(),
            base_url: Some("http://pc:1234/v1/".into()),
            api_key_env: Some("LM_STUDIO_TOKEN".into()),
            extra_env: HashMap::new(),
            display_name: None,
        };
        let codex = launch_settings_for_server(&server, "codex").unwrap();
        assert_eq!(codex.env["OPERATOR_LM_STUDIO_TOKEN"], "${LM_STUDIO_TOKEN}");
        assert!(codex
            .flags
            .iter()
            .any(|flag| flag.contains("model_provider=")));
        assert!(codex
            .flags
            .iter()
            .any(|flag| flag.contains("requires_openai_auth=false")));
        assert!(codex
            .flags
            .iter()
            .any(|flag| flag.contains("http://pc:1234/v1")));
        let claude = launch_settings_for_server(&server, "claude").unwrap();
        assert_eq!(claude.env["ANTHROPIC_BASE_URL"], "http://pc:1234");
        assert_eq!(claude.env["ANTHROPIC_AUTH_TOKEN"], "${LM_STUDIO_TOKEN}");
        assert_eq!(claude.env["ANTHROPIC_API_KEY"], "");
        assert!(claude.flags.is_empty());
        assert!(!claude.env.contains_key("OPENAI_API_KEY"));
    }

    #[test]
    fn test_model_probe_urls_accept_root_and_versioned_bases() {
        for kind in [
            ModelServerKind::LmStudio,
            ModelServerKind::OpenAiCompat,
            ModelServerKind::Ollama,
        ] {
            for base in [
                "http://pc:1234",
                "http://pc:1234/",
                "http://pc:1234/v1",
                "http://pc:1234/v1/",
            ] {
                let endpoint = if kind == ModelServerKind::Ollama {
                    "/api/tags"
                } else {
                    "/v1/models"
                };
                assert_eq!(kind.models_url(base), format!("http://pc:1234{endpoint}"));
            }
        }
        assert_eq!(
            ModelServerKind::OpenRouter.models_url("https://openrouter.ai/api/v1/"),
            "https://openrouter.ai/api/v1/models"
        );
    }

    #[test]
    fn test_lmstudio_spawn_base_includes_v1() {
        for base in [
            "http://pc:1234",
            "http://pc:1234/",
            "http://pc:1234/v1",
            "http://pc:1234/v1/",
        ] {
            let server = ModelServer {
                name: "gaming-pc".into(),
                kind: "lmstudio".into(),
                base_url: Some(base.into()),
                api_key_env: None,
                extra_env: HashMap::new(),
                display_name: None,
            };
            assert_eq!(
                env_for_server(&server)["OPENAI_BASE_URL"],
                "http://pc:1234/v1"
            );
        }
    }

    #[test]
    fn test_slug_roundtrip_for_all_kinds() {
        for kind in ModelServerKind::ALL {
            assert_eq!(ModelServerKind::from_slug(kind.slug()), Some(kind));
        }
    }

    #[test]
    fn test_from_slug_matches_existing_config_kind_strings() {
        // These are the kind strings already documented + persisted in configs.
        assert_eq!(
            ModelServerKind::from_slug("ollama"),
            Some(ModelServerKind::Ollama)
        );
        assert_eq!(
            ModelServerKind::from_slug("openai-compat"),
            Some(ModelServerKind::OpenAiCompat)
        );
        assert_eq!(
            ModelServerKind::from_slug("anthropic-api"),
            Some(ModelServerKind::AnthropicApi)
        );
        assert_eq!(ModelServerKind::from_slug("nope"), None);
    }

    #[test]
    fn test_openrouter_is_an_openai_protocol_model_provider() {
        let k = ModelServerKind::OpenRouter;
        assert_eq!(k.slug(), "openrouter");
        assert_eq!(k.provider_class(), ModelProviderClass::Gateway);
        assert!(!k.is_builtin());
        // Speaks the OpenAI protocol, so codex can target it directly.
        assert_eq!(k.base_url_env_var(), "OPENAI_BASE_URL");
        assert_eq!(k.api_key_env_var(), "OPENAI_API_KEY");
        // Its documented base already ends in `/api/v1`.
        assert_eq!(k.models_endpoint(), "/models");
    }

    #[test]
    fn test_brand_icon_basenames() {
        assert_eq!(
            ModelServerKind::AnthropicApi.brand_icon(),
            Some("anthropic")
        );
        assert_eq!(ModelServerKind::GoogleApi.brand_icon(), Some("google"));
        assert_eq!(ModelServerKind::Ollama.brand_icon(), Some("ollama"));
        assert_eq!(ModelServerKind::OpenRouter.brand_icon(), Some("openrouter"));
        // OpenAI API + the generic OpenAI-compatible hosts use codicons.
        assert_eq!(ModelServerKind::OpenAiApi.brand_icon(), None);
        assert_eq!(ModelServerKind::OpenAiCompat.brand_icon(), None);
        // Whatever a kind returns, it's never an empty string.
        for kind in ModelServerKind::ALL {
            assert!(kind.brand_icon().is_none_or(|b| !b.is_empty()));
        }
    }

    #[test]
    fn test_provider_class_partitions_all_kinds() {
        // Every kind has a provider class, and the two classes are a disjoint
        // cover of ALL.
        let first_party: Vec<_> = ModelServerKind::ALL
            .into_iter()
            .filter(|k| k.provider_class() == ModelProviderClass::FirstParty)
            .map(|k| k.slug())
            .collect();
        let gateways: Vec<_> = ModelServerKind::ALL
            .into_iter()
            .filter(|k| k.provider_class() == ModelProviderClass::Gateway)
            .map(|k| k.slug())
            .collect();
        assert_eq!(
            first_party,
            vec!["anthropic-api", "openai-api", "google-api", "xai-api"]
        );
        assert_eq!(
            gateways,
            vec!["ollama", "openrouter", "openai-compat", "lmstudio"]
        );
        assert_eq!(
            first_party.len() + gateways.len(),
            ModelServerKind::ALL.len()
        );
    }

    #[test]
    fn test_default_base_url_and_key_env_for_first_party() {
        assert_eq!(
            ModelServerKind::AnthropicApi.default_base_url(),
            Some("https://api.anthropic.com")
        );
        assert_eq!(
            ModelServerKind::OpenAiApi.default_base_url(),
            Some("https://api.openai.com")
        );
        assert_eq!(
            ModelServerKind::GoogleApi.default_base_url(),
            Some("https://generativelanguage.googleapis.com")
        );
        assert_eq!(
            ModelServerKind::AnthropicApi.default_api_key_env(),
            Some("ANTHROPIC_API_KEY")
        );
        assert_eq!(
            ModelServerKind::GoogleApi.default_api_key_env(),
            Some("GEMINI_API_KEY")
        );
        // Bring-your-own-endpoint kinds have no probe defaults.
        assert_eq!(ModelServerKind::OpenAiCompat.default_base_url(), None);
        assert!(!ModelServerKind::LmStudio.connectable_from_defaults());
        assert!(ModelServerKind::AnthropicApi.connectable_from_defaults());
    }

    #[test]
    fn test_builtins_are_the_vendor_apis() {
        let builtins: Vec<_> = ModelServerKind::ALL
            .into_iter()
            .filter(ModelServerKind::is_builtin)
            .map(|k| k.slug())
            .collect();
        assert_eq!(
            builtins,
            vec!["anthropic-api", "openai-api", "google-api", "xai-api"]
        );
    }

    #[test]
    fn test_models_endpoint_by_protocol() {
        assert_eq!(ModelServerKind::Ollama.models_endpoint(), "/api/tags");
        assert_eq!(
            ModelServerKind::OpenAiCompat.models_endpoint(),
            "/v1/models"
        );
        assert_eq!(
            ModelServerKind::GoogleApi.models_endpoint(),
            "/v1beta/models"
        );
    }

    fn server(kind: &str, base_url: Option<&str>) -> ModelServer {
        ModelServer {
            name: "test".into(),
            kind: kind.into(),
            base_url: base_url.map(str::to_string),
            api_key_env: None,
            extra_env: HashMap::new(),
            display_name: None,
        }
    }

    #[test]
    fn test_env_for_server_maps_base_url_by_protocol() {
        let s = server("ollama", Some("http://localhost:11434"));
        let env = env_for_server(&s);
        assert_eq!(
            env.get("OPENAI_BASE_URL").map(String::as_str),
            Some("http://localhost:11434/v1")
        );
    }

    #[test]
    fn test_env_for_server_builtin_without_base_url_is_empty() {
        let s = server("anthropic-api", None);
        assert!(env_for_server(&s).is_empty());
    }

    #[test]
    fn test_probe_defaults_do_not_leak_into_spawn_env() {
        // The provider has a probe-only default_base_url, but a builtin server
        // declares no base_url - so the spawn env must stay empty. This keeps the
        // vendor-default / OAuth launch path intact; defaults are probe-only.
        for kind in [
            ModelServerKind::AnthropicApi,
            ModelServerKind::OpenAiApi,
            ModelServerKind::GoogleApi,
        ] {
            assert!(kind.default_base_url().is_some());
            let s = server(kind.slug(), None);
            assert!(
                env_for_server(&s).is_empty(),
                "{} leaked probe defaults into spawn env",
                kind.slug()
            );
        }
    }

    #[test]
    fn test_env_for_server_api_key_is_reference_not_value() {
        let mut s = server("openai-compat", Some("http://gpu:8000"));
        s.api_key_env = Some("MY_SECRET_KEY".into());
        let env = env_for_server(&s);
        // Mapped to the canonical var by reference - the secret value is never read.
        assert_eq!(
            env.get("OPENAI_API_KEY").map(String::as_str),
            Some("${MY_SECRET_KEY}")
        );
    }

    #[test]
    fn test_env_for_server_extra_env_passthrough_and_precedence() {
        let mut s = server("openai-compat", Some("http://gpu:8000"));
        s.extra_env
            .insert("OPENAI_BASE_URL".into(), "http://override:9000".into());
        s.extra_env
            .insert("HTTP_PROXY".into(), "http://proxy".into());
        let env = env_for_server(&s);
        // extra_env wins over the derived base_url mapping.
        assert_eq!(
            env.get("OPENAI_BASE_URL").map(String::as_str),
            Some("http://override:9000")
        );
        assert_eq!(
            env.get("HTTP_PROXY").map(String::as_str),
            Some("http://proxy")
        );
    }

    #[test]
    fn test_base_url_env_var_is_protocol_keyed() {
        assert_eq!(
            ModelServerKind::Ollama.base_url_env_var(),
            "OPENAI_BASE_URL"
        );
        assert_eq!(
            ModelServerKind::AnthropicApi.base_url_env_var(),
            "ANTHROPIC_BASE_URL"
        );
    }
}

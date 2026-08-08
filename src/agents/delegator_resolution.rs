//! Shared delegator resolution logic for building `LaunchOptions`.
//!
//! Used by both the REST API launch endpoint and the TUI auto-launch path.

use crate::agents::LaunchOptions;
use crate::config::{
    implicit_model_server_for_tool, Config, Delegator, DelegatorLaunchConfig, LlmProvider,
    ModelServer, TargetDef, TargetKind, TARGET_DOCKER, TARGET_LOCAL,
};

/// Issuetype/step agent context for delegator resolution during launch.
///
/// Extracted from the issuetype registry before calling resolution,
/// so the registry read lock doesn't need to be held across the entire call.
pub struct AgentContext {
    /// Agent (delegator) name from the ticket's current step (highest priority)
    pub step_agent: Option<String>,
    /// Agent (delegator) name from the issuetype level (fallback)
    pub issuetype_agent: Option<String>,
}

/// Error type for delegator resolution failures.
#[derive(Debug, thiserror::Error)]
#[allow(clippy::enum_variant_names)] // The `Unknown*` variants share a semantic prefix; not a naming smell.
pub enum ResolutionError {
    #[error("Unknown delegator '{0}'")]
    UnknownDelegator(String),
    #[error("Unknown provider '{0}'")]
    UnknownProvider(String),
    #[error("Unknown model_server '{0}'")]
    UnknownModelServer(String),
    /// The delegator declaratively references a remote, named agent on another
    /// platform (AGNT, `OpenAI`, ...). Operator has no runtime client for those
    /// platforms, so such a delegator is export-only and cannot be resolved into a
    /// launchable provider.
    #[error("Delegator '{name}' references {platform} agent '{agent_id}' and is export-only; it cannot be launched locally (Operator has no {platform} runtime client)")]
    RemoteOnlyDelegator {
        name: String,
        platform: String,
        agent_id: String,
    },
    #[error(
        "Delegator launch_config references unknown host '{0}' (no [[hosts]] entry with that name)"
    )]
    UnknownRemoteHost(String),
    #[error("Unknown execution target '{name}' (known targets: {known})")]
    UnknownTarget { name: String, known: String },
    #[error("Remote host '{host}' cannot be combined with {feature} in v1")]
    RemoteHostConflict { host: String, feature: &'static str },
}

/// Resolve a delegator's `ModelServer`: named lookup if set, else implicit vendor default.
pub(crate) fn resolve_model_server_for_delegator(
    config: &Config,
    d: &Delegator,
) -> Result<ModelServer, ResolutionError> {
    match d.model_server.as_deref() {
        Some(name) => config
            .model_servers
            .iter()
            .find(|s| s.name == name)
            .cloned()
            .ok_or_else(|| ResolutionError::UnknownModelServer(name.to_string())),
        None => Ok(implicit_model_server_for_tool(&d.llm_tool)),
    }
}

/// Convert a `Delegator` into an `LlmProvider`, resolving its `model_server` and
/// threading the server's env vars (base URL, API key, extra env) into
/// [`LlmProvider::env`] so they are exported when the agent spawns.
///
/// Errors if the delegator names a `model_server` that isn't declared. Builtins
/// and servers without a `base_url` contribute no env vars (the vendor-default
/// path is unchanged).
pub(crate) fn delegator_to_provider(
    config: &Config,
    d: &Delegator,
) -> Result<LlmProvider, ResolutionError> {
    // A delegator that declaratively references a remote agent (AGNT, OpenAI, ...)
    // is export-only: Operator cannot spawn it (no runtime client for those
    // platforms). Guard at the single resolution choke point so every local-launch
    // path (explicit, step-agent, issuetype-agent, default, and the multi-agent
    // fan-out) is covered, for every platform.
    if let Some(r) = &d.remote_agent {
        return Err(ResolutionError::RemoteOnlyDelegator {
            name: d.name.clone(),
            platform: r.platform.clone(),
            agent_id: r.id.clone(),
        });
    }
    let server = resolve_model_server_for_delegator(config, d)?;
    let env = crate::api::providers::model_server::env_for_server(&server);
    Ok(LlmProvider {
        tool: d.llm_tool.clone(),
        model: d.model.clone(),
        env,
        ..Default::default()
    })
}

/// Resolve env vars for an ad-hoc launch (`--llm-tool`/`--model`/`--model-server`).
///
/// A named `model_server` is looked up among declared servers and the implicit
/// builtins; absent a name, the tool's implicit vendor default is used (which
/// contributes no env). Mirrors [`delegator_to_provider`] for the non-delegator path.
fn adhoc_model_server_env(
    config: &Config,
    tool: &str,
    model_server: Option<&str>,
) -> Result<std::collections::HashMap<String, String>, ResolutionError> {
    let server = match model_server {
        Some(name) => config
            .model_servers
            .iter()
            .find(|s| s.name == name)
            .cloned()
            .or_else(|| {
                ["claude", "codex", "gemini"]
                    .iter()
                    .map(|t| implicit_model_server_for_tool(t))
                    .find(|s| s.name == name)
            })
            .ok_or_else(|| ResolutionError::UnknownModelServer(name.to_string()))?,
        None => implicit_model_server_for_tool(tool),
    };
    Ok(crate::api::providers::model_server::env_for_server(&server))
}

/// Resolve the execution target for a launch — the one pure decision point.
///
/// Precedence:
/// 1. `target` name set → look up (explicit `[[targets]]`, builtin
///    `local`/`docker`, or a `[[hosts]]` name); unknown = hard error
/// 2. `host` name set (deprecated) → ssh target of that name
/// 3. `docker: Some(true)` (deprecated) → synthesized docker target
/// 4. `docker: Some(false)` → local
/// 5. `launch.docker.enabled` → synthesized docker target
/// 6. → local
///
/// Deprecated combinations resolve deterministically (`target` wins over
/// `host`/`docker`; `host` wins over `docker: true`) with one deprecation
/// warning per process instead of the former hard error.
pub fn resolve_target(
    launch_config: Option<&DelegatorLaunchConfig>,
    config: &Config,
) -> Result<TargetDef, ResolutionError> {
    static TARGET_WINS: std::sync::Once = std::sync::Once::new();
    static HOST_WINS: std::sync::Once = std::sync::Once::new();

    if let Some(lc) = launch_config {
        if let Some(ref name) = lc.target {
            if lc.docker.is_some() || lc.host.is_some() {
                TARGET_WINS.call_once(|| {
                    tracing::warn!(
                        "launch_config sets `target` together with deprecated `docker`/`host`; \
                         `target` wins"
                    );
                });
            }
            return resolve_named_target(config, name);
        }
        if let Some(ref host_name) = lc.host {
            if lc.docker == Some(true) {
                HOST_WINS.call_once(|| {
                    tracing::warn!(
                        "launch_config sets both `docker` and `host` (deprecated); host wins — \
                         migrate to `target`"
                    );
                });
            }
            let host = config
                .hosts
                .iter()
                .find(|h| h.name == *host_name)
                .ok_or_else(|| ResolutionError::UnknownRemoteHost(host_name.clone()))?;
            return Ok(TargetDef::from_host(host));
        }
        match lc.docker {
            Some(true) => return Ok(TargetDef::docker(config.launch.docker.clone())),
            Some(false) => return Ok(TargetDef::local()),
            None => {}
        }
    }
    if config.launch.docker.enabled {
        return Ok(TargetDef::docker(config.launch.docker.clone()));
    }
    Ok(TargetDef::local())
}

/// Name lookup including synthesis: explicit `[[targets]]` entries first, then
/// the builtin `local`/`docker` targets, then one ssh target per `[[hosts]]`.
pub fn resolve_named_target(config: &Config, name: &str) -> Result<TargetDef, ResolutionError> {
    if let Some(def) = config.targets.iter().find(|t| t.name == name) {
        return Ok(def.clone());
    }
    match name {
        TARGET_LOCAL => return Ok(TargetDef::local()),
        TARGET_DOCKER => return Ok(TargetDef::docker(config.launch.docker.clone())),
        _ => {}
    }
    if let Some(host) = config.hosts.iter().find(|h| h.name == name) {
        return Ok(TargetDef::from_host(host));
    }
    Err(ResolutionError::UnknownTarget {
        name: name.to_string(),
        known: crate::config::known_target_names(config).join(", "),
    })
}

/// Apply a delegator's launch config to launch options.
///
/// Resolves the execution target via [`resolve_target`] (rows 5-6 apply even
/// when there is no launch config) and enforces the v1 remote-launch
/// constraints: worktrees and relay injection are forced off for ssh/coder
/// targets (both assume the local filesystem), and the zellij session wrapper
/// is a hard conflict rather than a silent degradation.
pub(crate) fn apply_delegator_launch_config(
    options: &mut LaunchOptions,
    launch_config: &Option<DelegatorLaunchConfig>,
    config: &Config,
) -> Result<(), ResolutionError> {
    if let Some(ref lc) = launch_config {
        options.yolo_mode = options.yolo_mode || lc.yolo;
        options.extra_flags.clone_from(&lc.flags);
        options.use_worktrees_override = lc.use_worktrees;
        options.create_branch_override = lc.create_branch;
        options.prompt_prefix.clone_from(&lc.prompt_prefix);
        options.prompt_suffix.clone_from(&lc.prompt_suffix);
        options.operator_relay = lc.operator_relay;
    }

    let target = resolve_target(launch_config.as_ref(), config)?;
    apply_target_to_options(options, target, config)
}

/// Install a resolved target on launch options, enforcing the v1 remote
/// constraints (worktrees + relay off for ssh/coder; zellij is a hard
/// conflict). Also used for per-request target overrides.
pub(crate) fn apply_target_to_options(
    options: &mut LaunchOptions,
    target: TargetDef,
    config: &Config,
) -> Result<(), ResolutionError> {
    if matches!(target.kind, TargetKind::Ssh(_) | TargetKind::Coder(_)) {
        if config.sessions.wrapper == crate::config::SessionWrapperType::Zellij {
            return Err(ResolutionError::RemoteHostConflict {
                host: target.name,
                feature: "the zellij session wrapper",
            });
        }
        options.use_worktrees_override = Some(false);
        options.operator_relay = Some(false);
    }
    options.target = target;
    Ok(())
}

/// Resolve a default delegator when none is explicitly specified.
///
/// Resolution chain:
/// 1. Single configured delegator -> use it
/// 2. Delegator matching the user's preferred LLM tool -> use it
/// 3. None -> caller falls back to first detected tool + first model alias
pub(crate) fn resolve_default_delegator(config: &Config) -> Option<&Delegator> {
    match config.delegators.len() {
        0 => None,
        1 => Some(&config.delegators[0]),
        _ => {
            let preferred_tool = config
                .llm_tools
                .default_tool
                .as_deref()
                .or_else(|| config.llm_tools.detected.first().map(|t| t.name.as_str()));
            if let Some(tool_name) = preferred_tool {
                config.delegators.iter().find(|d| d.llm_tool == tool_name)
            } else {
                Some(&config.delegators[0])
            }
        }
    }
}

/// Look up a delegator by name in the config
fn resolve_delegator_by_name<'a>(config: &'a Config, name: &str) -> Option<&'a Delegator> {
    config.delegators.iter().find(|d| d.name == name)
}

/// Resolve launch options from config, an optional explicit request, and agent context.
///
/// Resolution chain (highest to lowest priority):
/// 1. Explicit delegator name
/// 2. Step-level agent from issuetype
/// 3. Issuetype-level agent
/// 4. Legacy provider/model
/// 5. Default delegator from config
/// 6. Detected tool defaults
pub fn resolve_launch_options(
    config: &Config,
    explicit_delegator: Option<&str>,
    explicit_provider: Option<&str>,
    explicit_model: Option<&str>,
    explicit_model_server: Option<&str>,
    yolo_mode: bool,
    agent_context: Option<&AgentContext>,
) -> Result<LaunchOptions, ResolutionError> {
    let mut options = LaunchOptions {
        yolo_mode,
        ..Default::default()
    };

    // 1. Explicit delegator name takes precedence
    if let Some(delegator_name) = explicit_delegator {
        let delegator = config
            .delegators
            .iter()
            .find(|d| d.name == delegator_name)
            .ok_or_else(|| ResolutionError::UnknownDelegator(delegator_name.to_string()))?;

        options.provider = Some(delegator_to_provider(config, delegator)?);
        options.delegator_name = Some(delegator.name.clone());
        apply_delegator_launch_config(&mut options, &delegator.launch_config, config)?;
        return Ok(options);
    }

    // 2. Step-level agent from issuetype template
    if let Some(ctx) = agent_context {
        if let Some(ref step_agent) = ctx.step_agent {
            if let Some(delegator) = resolve_delegator_by_name(config, step_agent) {
                options.provider = Some(delegator_to_provider(config, delegator)?);
                options.delegator_name = Some(delegator.name.clone());
                apply_delegator_launch_config(&mut options, &delegator.launch_config, config)?;
                return Ok(options);
            }
            // Step agent name doesn't match any delegator — fall through
        }

        // 3. Issuetype-level agent
        if let Some(ref it_agent) = ctx.issuetype_agent {
            if let Some(delegator) = resolve_delegator_by_name(config, it_agent) {
                options.provider = Some(delegator_to_provider(config, delegator)?);
                options.delegator_name = Some(delegator.name.clone());
                apply_delegator_launch_config(&mut options, &delegator.launch_config, config)?;
                return Ok(options);
            }
        }
    }

    // 4. Legacy: explicit provider/model
    if let Some(provider_name) = explicit_provider {
        let provider = config
            .llm_tools
            .providers
            .iter()
            .find(|p| p.tool == *provider_name)
            .cloned();

        if let Some(p) = provider {
            let model = explicit_model
                .map(std::string::ToString::to_string)
                .unwrap_or(p.model.clone());
            let env = adhoc_model_server_env(config, &p.tool, explicit_model_server)?;
            options.provider = Some(LlmProvider {
                tool: p.tool,
                model,
                env,
                ..Default::default()
            });
        } else {
            return Err(ResolutionError::UnknownProvider(provider_name.to_string()));
        }

        return Ok(options);
    }

    if let Some(model) = explicit_model {
        if let Some(p) = config.llm_tools.providers.first().cloned() {
            let env = adhoc_model_server_env(config, &p.tool, explicit_model_server)?;
            options.provider = Some(LlmProvider {
                tool: p.tool,
                model: model.to_string(),
                env,
                ..Default::default()
            });
        }

        return Ok(options);
    }

    // 5. No explicit selection — resolve default delegator
    if let Some(delegator) = resolve_default_delegator(config) {
        options.provider = Some(delegator_to_provider(config, delegator)?);
        options.delegator_name = Some(delegator.name.clone());
        apply_delegator_launch_config(&mut options, &delegator.launch_config, config)?;
        return Ok(options);
    }

    // 6. No delegators at all — fall back to default tool/model or first detected
    let tool = config
        .llm_tools
        .default_tool
        .as_deref()
        .and_then(|name| config.llm_tools.detected.iter().find(|t| t.name == name))
        .or_else(|| config.llm_tools.detected.first());

    if let Some(tool) = tool {
        let model = config
            .llm_tools
            .default_model
            .clone()
            .or_else(|| tool.model_aliases.first().cloned())
            .unwrap_or_else(|| "default".to_string());
        options.provider = Some(LlmProvider {
            tool: tool.name.clone(),
            model,
            ..Default::default()
        });
    }

    Ok(options)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    fn make_delegator(name: &str, tool: &str, model: &str) -> Delegator {
        Delegator {
            name: name.to_string(),
            llm_tool: tool.to_string(),
            model: model.to_string(),
            display_name: None,
            model_properties: std::collections::HashMap::new(),
            model_server: None,
            launch_config: None,
            remote_agent: None,
            x_agnt: None,
            x_openai: None,
            unmapped_core: None,
        }
    }

    #[test]
    fn test_resolve_default_no_delegators() {
        let config = Config::default();
        let options = resolve_launch_options(&config, None, None, None, None, false, None).unwrap();
        assert!(options.provider.is_none());
        assert!(!options.yolo_mode);
    }

    #[test]
    fn test_resolve_model_server_implicit_for_claude() {
        let config = Config::default();
        let d = make_delegator("claude-opus", "claude", "opus");
        let server = resolve_model_server_for_delegator(&config, &d).unwrap();
        assert_eq!(server.name, "anthropic-api");
        assert_eq!(server.kind, "anthropic-api");
    }

    #[test]
    fn test_resolve_model_server_implicit_for_codex() {
        let config = Config::default();
        let d = make_delegator("codex-gpt", "codex", "gpt-4o");
        let server = resolve_model_server_for_delegator(&config, &d).unwrap();
        assert_eq!(server.name, "openai-api");
    }

    #[test]
    fn test_resolve_model_server_named_lookup() {
        let mut config = Config::default();
        config.model_servers.push(crate::config::ModelServer {
            name: "ollama-local".to_string(),
            kind: "ollama".to_string(),
            base_url: Some("http://localhost:11434".to_string()),
            api_key_env: None,
            extra_env: std::collections::HashMap::new(),
            display_name: None,
        });

        let mut d = make_delegator("codex-local-qwen", "codex", "qwen2.5-coder");
        d.model_server = Some("ollama-local".to_string());

        let server = resolve_model_server_for_delegator(&config, &d).unwrap();
        assert_eq!(server.name, "ollama-local");
        assert_eq!(server.kind, "ollama");
        assert_eq!(server.base_url.as_deref(), Some("http://localhost:11434"));
    }

    #[test]
    fn test_resolve_model_server_unknown_name_errors() {
        let config = Config::default();
        let mut d = make_delegator("d", "claude", "opus");
        d.model_server = Some("nonexistent".to_string());
        let err = resolve_model_server_for_delegator(&config, &d).unwrap_err();
        assert!(matches!(err, ResolutionError::UnknownModelServer(_)));
    }

    fn make_remote_config(host_name: &str) -> Config {
        let mut config = Config::default();
        config.hosts.push(crate::config::RemoteHost {
            name: host_name.to_string(),
            ssh_alias: "vm-alias".to_string(),
            workdir: "/srv/agents".to_string(),
            display_name: None,
            ssh_config_path: None,
        });
        let mut d = make_delegator("claude-remote", "claude", "opus");
        d.launch_config = Some(DelegatorLaunchConfig {
            host: Some(host_name.to_string()),
            ..Default::default()
        });
        config.delegators.push(d);
        config
    }

    #[test]
    fn test_resolve_known_host_populates_remote_host() {
        let config = make_remote_config("gpu-vm");
        let options = resolve_launch_options(
            &config,
            Some("claude-remote"),
            None,
            None,
            None,
            false,
            None,
        )
        .unwrap();
        let host = options
            .target
            .as_remote_host()
            .expect("remote host resolved");
        assert_eq!(host.name, "gpu-vm");
        assert_eq!(host.ssh_alias, "vm-alias");
        assert_eq!(host.workdir, "/srv/agents");
    }

    #[test]
    fn test_resolve_unknown_host_errors() {
        let mut config = make_remote_config("gpu-vm");
        config.hosts.clear();
        let err = resolve_launch_options(
            &config,
            Some("claude-remote"),
            None,
            None,
            None,
            false,
            None,
        )
        .unwrap_err();
        assert!(matches!(err, ResolutionError::UnknownRemoteHost(_)));
    }

    #[test]
    fn test_resolve_no_host_leaves_remote_host_none() {
        let mut config = Config::default();
        config
            .delegators
            .push(make_delegator("local", "claude", "opus"));
        let options =
            resolve_launch_options(&config, Some("local"), None, None, None, false, None).unwrap();
        assert!(options.target.as_remote_host().is_none());
    }

    #[test]
    fn test_remote_host_forces_worktrees_and_relay_off() {
        let mut config = make_remote_config("gpu-vm");
        let lc = config.delegators[0].launch_config.as_mut().unwrap();
        lc.use_worktrees = Some(true);
        lc.operator_relay = Some(true);
        let options = resolve_launch_options(
            &config,
            Some("claude-remote"),
            None,
            None,
            None,
            false,
            None,
        )
        .unwrap();
        assert_eq!(options.use_worktrees_override, Some(false));
        assert_eq!(options.operator_relay, Some(false));
    }

    #[test]
    fn test_remote_host_plus_docker_resolves_to_host() {
        // Legacy both-set configs now resolve deterministically to the host
        // (with a deprecation warning) instead of a hard error.
        let mut config = make_remote_config("gpu-vm");
        config.delegators[0].launch_config.as_mut().unwrap().docker = Some(true);
        let options = resolve_launch_options(
            &config,
            Some("claude-remote"),
            None,
            None,
            None,
            false,
            None,
        )
        .unwrap();
        let host = options.target.as_remote_host().expect("host wins");
        assert_eq!(host.name, "gpu-vm");
        assert!(!options.is_docker());
    }

    #[test]
    fn test_remote_host_plus_zellij_errors() {
        let mut config = make_remote_config("gpu-vm");
        config.sessions.wrapper = crate::config::SessionWrapperType::Zellij;
        let err = resolve_launch_options(
            &config,
            Some("claude-remote"),
            None,
            None,
            None,
            false,
            None,
        )
        .unwrap_err();
        assert!(matches!(err, ResolutionError::RemoteHostConflict { .. }));
    }

    #[test]
    fn test_resolve_single_delegator_is_default() {
        let mut config = Config::default();
        config
            .delegators
            .push(make_delegator("claude-opus", "claude", "opus"));

        let options = resolve_launch_options(&config, None, None, None, None, false, None).unwrap();
        let provider = options.provider.unwrap();
        assert_eq!(provider.tool, "claude");
        assert_eq!(provider.model, "opus");
        assert_eq!(options.delegator_name.as_deref(), Some("claude-opus"));
    }

    #[test]
    fn test_resolve_explicit_delegator() {
        let mut config = Config::default();
        config
            .delegators
            .push(make_delegator("claude-opus", "claude", "opus"));
        config
            .delegators
            .push(make_delegator("gemini-pro", "gemini", "pro"));

        let options =
            resolve_launch_options(&config, Some("gemini-pro"), None, None, None, false, None)
                .unwrap();
        let provider = options.provider.unwrap();
        assert_eq!(provider.tool, "gemini");
        assert_eq!(provider.model, "pro");
    }

    #[test]
    fn test_resolve_unknown_delegator_errors() {
        let config = Config::default();
        let result =
            resolve_launch_options(&config, Some("nonexistent"), None, None, None, false, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_resolve_step_agent_overrides_issuetype() {
        let mut config = Config::default();
        config
            .delegators
            .push(make_delegator("claude-opus", "claude", "opus"));
        config
            .delegators
            .push(make_delegator("claude-sonnet", "claude", "sonnet"));

        let ctx = AgentContext {
            step_agent: Some("claude-opus".to_string()),
            issuetype_agent: Some("claude-sonnet".to_string()),
        };

        let options =
            resolve_launch_options(&config, None, None, None, None, false, Some(&ctx)).unwrap();
        let provider = options.provider.unwrap();
        assert_eq!(provider.model, "opus");
    }

    #[test]
    fn test_resolve_issuetype_agent_fallback() {
        let mut config = Config::default();
        config
            .delegators
            .push(make_delegator("claude-opus", "claude", "opus"));

        let ctx = AgentContext {
            step_agent: None,
            issuetype_agent: Some("claude-opus".to_string()),
        };

        let options =
            resolve_launch_options(&config, None, None, None, None, false, Some(&ctx)).unwrap();
        let provider = options.provider.unwrap();
        assert_eq!(provider.model, "opus");
    }

    #[test]
    fn test_resolve_unknown_step_agent_falls_through() {
        let mut config = Config::default();
        config
            .delegators
            .push(make_delegator("claude-opus", "claude", "opus"));

        let ctx = AgentContext {
            step_agent: Some("nonexistent".to_string()),
            issuetype_agent: Some("claude-opus".to_string()),
        };

        let options =
            resolve_launch_options(&config, None, None, None, None, false, Some(&ctx)).unwrap();
        let provider = options.provider.unwrap();
        assert_eq!(provider.model, "opus");
    }

    #[test]
    fn test_resolve_delegator_applies_launch_config() {
        let mut config = Config::default();
        config.delegators.push(Delegator {
            name: "full".to_string(),
            llm_tool: "claude".to_string(),
            model: "opus".to_string(),
            display_name: None,
            model_properties: std::collections::HashMap::new(),
            model_server: None,
            launch_config: Some(DelegatorLaunchConfig {
                yolo: true,
                permission_mode: None,
                flags: vec!["--verbose".to_string()],
                use_worktrees: Some(true),
                create_branch: Some(false),
                docker: Some(true),
                prompt_prefix: Some("PREFIX".to_string()),
                prompt_suffix: Some("SUFFIX".to_string()),
                operator_relay: None,
                host: None,
                target: None,
            }),
            remote_agent: None,
            x_agnt: None,
            x_openai: None,
            unmapped_core: None,
        });

        let options =
            resolve_launch_options(&config, Some("full"), None, None, None, false, None).unwrap();
        assert!(options.yolo_mode);
        assert!(options.is_docker());
        assert_eq!(options.use_worktrees_override, Some(true));
        assert_eq!(options.create_branch_override, Some(false));
        assert_eq!(options.extra_flags, vec!["--verbose".to_string()]);
        assert_eq!(options.prompt_prefix.as_deref(), Some("PREFIX"));
        assert_eq!(options.prompt_suffix.as_deref(), Some("SUFFIX"));
    }

    #[test]
    fn resolve_remote_only_delegator_errors() {
        let mut config = Config::default();
        let mut d = make_delegator("agnt-researcher", "anthropic", "claude-3-5-sonnet");
        d.remote_agent = Some(crate::config::RemoteAgentRef {
            platform: "agnt".to_string(),
            id: "Research Assistant".to_string(),
        });
        config.delegators.push(d);

        let err = resolve_launch_options(
            &config,
            Some("agnt-researcher"),
            None,
            None,
            None,
            false,
            None,
        )
        .unwrap_err();
        assert!(
            matches!(err, ResolutionError::RemoteOnlyDelegator { .. }),
            "explicit remote-only delegator must error, got {err:?}"
        );
    }

    #[test]
    fn resolve_remote_only_is_platform_agnostic() {
        // The guard fires for any platform, not just AGNT — an OpenAI Assistant
        // delegator is equally export-only. Proves the generalization.
        let mut config = Config::default();
        let mut d = make_delegator("openai-reviewer", "openai", "gpt-4o");
        d.remote_agent = Some(crate::config::RemoteAgentRef {
            platform: "openai".to_string(),
            id: "asst_abc123".to_string(),
        });
        config.delegators.push(d);

        let err = resolve_launch_options(
            &config,
            Some("openai-reviewer"),
            None,
            None,
            None,
            false,
            None,
        )
        .unwrap_err();
        match err {
            ResolutionError::RemoteOnlyDelegator { platform, .. } => assert_eq!(platform, "openai"),
            other => panic!("expected RemoteOnlyDelegator for openai, got {other:?}"),
        }
    }

    #[test]
    fn resolve_remote_only_step_agent_errors() {
        // The guard sits in the single resolution choke point, so the step-agent
        // path errors too — proving the export-only contract holds on every path.
        let mut config = Config::default();
        let mut d = make_delegator("agnt-researcher", "anthropic", "claude-3-5-sonnet");
        d.remote_agent = Some(crate::config::RemoteAgentRef {
            platform: "agnt".to_string(),
            id: "Research Assistant".to_string(),
        });
        config.delegators.push(d);

        let ctx = AgentContext {
            step_agent: Some("agnt-researcher".to_string()),
            issuetype_agent: None,
        };
        let err =
            resolve_launch_options(&config, None, None, None, None, false, Some(&ctx)).unwrap_err();
        assert!(
            matches!(err, ResolutionError::RemoteOnlyDelegator { .. }),
            "step-agent remote-only delegator must error, got {err:?}"
        );
    }

    #[test]
    fn test_resolve_yolo_passthrough() {
        let config = Config::default();
        let options = resolve_launch_options(&config, None, None, None, None, true, None).unwrap();
        assert!(options.yolo_mode);
    }

    // ========================================
    // resolve_target() precedence table
    // ========================================

    fn lc(f: impl FnOnce(&mut DelegatorLaunchConfig)) -> DelegatorLaunchConfig {
        let mut lc = DelegatorLaunchConfig::default();
        f(&mut lc);
        lc
    }

    #[test]
    fn test_target_row1_target_name_beats_host_and_docker() {
        let mut config = Config::default();
        config.hosts.push(crate::config::RemoteHost {
            name: "gpu-vm".to_string(),
            ssh_alias: "gpu".to_string(),
            workdir: "/p".to_string(),
            display_name: None,
            ssh_config_path: None,
        });
        config.targets.push(TargetDef {
            name: "sandbox".to_string(),
            display_name: None,
            kind: TargetKind::Docker(crate::config::DockerConfig {
                image: "img:explicit".to_string(),
                ..Default::default()
            }),
        });
        let launch = lc(|l| {
            l.target = Some("sandbox".to_string());
            l.host = Some("gpu-vm".to_string());
            l.docker = Some(true);
        });
        let target = resolve_target(Some(&launch), &config).unwrap();
        assert_eq!(target.name, "sandbox");
        assert!(
            matches!(&target.kind, TargetKind::Docker(d) if d.image == "img:explicit"),
            "explicit target payload must win: {target:?}"
        );
    }

    #[test]
    fn test_target_row2_host_beats_docker_true() {
        let mut config = Config::default();
        config.hosts.push(crate::config::RemoteHost {
            name: "gpu-vm".to_string(),
            ssh_alias: "gpu".to_string(),
            workdir: "/p".to_string(),
            display_name: Some("GPU".to_string()),
            ssh_config_path: None,
        });
        let launch = lc(|l| {
            l.host = Some("gpu-vm".to_string());
            l.docker = Some(true);
        });
        let target = resolve_target(Some(&launch), &config).unwrap();
        let host = target.as_remote_host().expect("host wins over docker");
        assert_eq!(host.name, "gpu-vm");
        assert_eq!(host.display_name.as_deref(), Some("GPU"));
    }

    #[test]
    fn test_target_row3_docker_true_synthesizes_docker() {
        let mut config = Config::default();
        config.launch.docker.image = "img:global".to_string();
        let launch = lc(|l| l.docker = Some(true));
        let target = resolve_target(Some(&launch), &config).unwrap();
        assert_eq!(target.name, crate::config::TARGET_DOCKER);
        assert!(matches!(&target.kind, TargetKind::Docker(d) if d.image == "img:global"));
    }

    #[test]
    fn test_target_row4_docker_false_shields_enabled_fallback() {
        let mut config = Config::default();
        config.launch.docker.enabled = true;
        let launch = lc(|l| l.docker = Some(false));
        let target = resolve_target(Some(&launch), &config).unwrap();
        assert_eq!(target.kind, TargetKind::Local);
    }

    #[test]
    fn test_target_row5_enabled_true_now_targets_docker_for_auto_launches() {
        // BEHAVIOR CHANGE (approved): launch.docker.enabled was previously only
        // a TUI dialog gate; it is now a real resolution fallback, so REST/CLI/
        // auto launches with enabled = true run in docker.
        let mut config = Config::default();
        config.launch.docker.enabled = true;
        for launch in [None, Some(lc(|_| {}))] {
            let target = resolve_target(launch.as_ref(), &config).unwrap();
            assert!(
                matches!(target.kind, TargetKind::Docker(_)),
                "enabled=true must resolve to docker (input: {launch:?})"
            );
        }
    }

    #[test]
    fn test_target_row6_default_is_local() {
        let config = Config::default();
        assert_eq!(
            resolve_target(None, &config).unwrap().kind,
            TargetKind::Local
        );
        assert_eq!(
            resolve_target(Some(&DelegatorLaunchConfig::default()), &config)
                .unwrap()
                .kind,
            TargetKind::Local
        );
    }

    #[test]
    fn test_resolve_named_target_builtins_and_synthesis() {
        let mut config = Config::default();
        config.launch.docker.image = "img:g".to_string();
        config.hosts.push(crate::config::RemoteHost {
            name: "legacy".to_string(),
            ssh_alias: "l".to_string(),
            workdir: "/p".to_string(),
            display_name: None,
            ssh_config_path: None,
        });

        assert_eq!(
            resolve_named_target(&config, "local").unwrap().kind,
            TargetKind::Local
        );
        assert!(matches!(
            resolve_named_target(&config, "docker").unwrap().kind,
            TargetKind::Docker(_)
        ));
        let legacy = resolve_named_target(&config, "legacy").unwrap();
        assert!(matches!(legacy.kind, TargetKind::Ssh(_)));
        assert_eq!(legacy.name, "legacy");
    }

    #[test]
    fn test_resolve_named_target_unknown_is_hard_error_listing_known() {
        let config = Config::default();
        let launch = lc(|l| l.target = Some("nope".to_string()));
        let err = resolve_target(Some(&launch), &config).unwrap_err();
        let msg = err.to_string();
        assert!(
            matches!(err, ResolutionError::UnknownTarget { .. }),
            "unknown target must never silently fall back to local"
        );
        assert!(msg.contains("local") && msg.contains("docker"), "{msg}");
    }

    #[test]
    fn test_legacy_host_and_target_synthesis_equivalent() {
        // A [[hosts]] entry and an equivalent [[targets]] ssh entry resolve to
        // the same RemoteHost shape.
        let host = crate::config::RemoteHost {
            name: "vm".to_string(),
            ssh_alias: "vm-a".to_string(),
            workdir: "/w".to_string(),
            display_name: None,
            ssh_config_path: None,
        };
        let mut host_config = Config::default();
        host_config.hosts.push(host.clone());
        let via_host = resolve_named_target(&host_config, "vm").unwrap();

        let mut target_config = Config::default();
        target_config.targets.push(TargetDef::from_host(&host));
        let via_target = resolve_named_target(&target_config, "vm").unwrap();

        assert_eq!(via_host.as_remote_host(), via_target.as_remote_host());
    }
}

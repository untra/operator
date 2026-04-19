//! Shared delegator resolution logic for building `LaunchOptions`.
//!
//! Used by both the REST API launch endpoint and the TUI auto-launch path.

use crate::agents::LaunchOptions;
use crate::config::{
    implicit_model_server_for_tool, Config, Delegator, DelegatorLaunchConfig, LlmProvider,
    ModelServer,
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
#[allow(clippy::enum_variant_names)] // All failures are "unknown reference to X"; prefix is semantic.
pub enum ResolutionError {
    #[error("Unknown delegator '{0}'")]
    UnknownDelegator(String),
    #[error("Unknown provider '{0}'")]
    UnknownProvider(String),
    #[error("Unknown model_server '{0}'")]
    UnknownModelServer(String),
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

/// Convert a `Delegator` into an `LlmProvider`.
///
/// v1: populates `tool` and `model` only. The delegator's `model_server` is
/// resolved and env vars are expected to be injected into `LlmProvider.env`
/// at spawn time — currently a no-op. TODO(model-servers-v2): thread the
/// resolved `ModelServer` through to `LlmProvider.env` via a per-tool mapping.
pub(crate) fn delegator_to_provider(d: &Delegator) -> LlmProvider {
    LlmProvider {
        tool: d.llm_tool.clone(),
        model: d.model.clone(),
        ..Default::default()
    }
}

/// Apply a delegator's launch config to launch options
pub(crate) fn apply_delegator_launch_config(
    options: &mut LaunchOptions,
    launch_config: &Option<DelegatorLaunchConfig>,
) {
    if let Some(ref lc) = launch_config {
        options.yolo_mode = options.yolo_mode || lc.yolo;
        options.extra_flags.clone_from(&lc.flags);
        if let Some(docker) = lc.docker {
            options.docker_mode = docker;
        }
        options.use_worktrees_override = lc.use_worktrees;
        options.create_branch_override = lc.create_branch;
        options.prompt_prefix.clone_from(&lc.prompt_prefix);
        options.prompt_suffix.clone_from(&lc.prompt_suffix);
    }
}

/// Resolve a default delegator when none is explicitly specified.
///
/// Resolution chain:
/// 1. Single configured delegator -> use it
/// 2. Delegator matching the user's preferred LLM tool -> use it
/// 3. None -> caller falls back to first detected tool + first model alias
fn resolve_default_delegator(config: &Config) -> Option<&Delegator> {
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

        options.provider = Some(delegator_to_provider(delegator));
        options.delegator_name = Some(delegator.name.clone());
        apply_delegator_launch_config(&mut options, &delegator.launch_config);
        return Ok(options);
    }

    // 2. Step-level agent from issuetype template
    if let Some(ctx) = agent_context {
        if let Some(ref step_agent) = ctx.step_agent {
            if let Some(delegator) = resolve_delegator_by_name(config, step_agent) {
                options.provider = Some(delegator_to_provider(delegator));
                options.delegator_name = Some(delegator.name.clone());
                apply_delegator_launch_config(&mut options, &delegator.launch_config);
                return Ok(options);
            }
            // Step agent name doesn't match any delegator — fall through
        }

        // 3. Issuetype-level agent
        if let Some(ref it_agent) = ctx.issuetype_agent {
            if let Some(delegator) = resolve_delegator_by_name(config, it_agent) {
                options.provider = Some(delegator_to_provider(delegator));
                options.delegator_name = Some(delegator.name.clone());
                apply_delegator_launch_config(&mut options, &delegator.launch_config);
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
            options.provider = Some(LlmProvider {
                tool: p.tool,
                model,
                ..Default::default()
            });
        } else {
            return Err(ResolutionError::UnknownProvider(provider_name.to_string()));
        }

        return Ok(options);
    }

    if let Some(model) = explicit_model {
        if let Some(p) = config.llm_tools.providers.first().cloned() {
            options.provider = Some(LlmProvider {
                tool: p.tool,
                model: model.to_string(),
                ..Default::default()
            });
        }

        return Ok(options);
    }

    // 5. No explicit selection — resolve default delegator
    if let Some(delegator) = resolve_default_delegator(config) {
        options.provider = Some(delegator_to_provider(delegator));
        options.delegator_name = Some(delegator.name.clone());
        apply_delegator_launch_config(&mut options, &delegator.launch_config);
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
        }
    }

    #[test]
    fn test_resolve_default_no_delegators() {
        let config = Config::default();
        let options = resolve_launch_options(&config, None, None, None, false, None).unwrap();
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

    #[test]
    fn test_resolve_single_delegator_is_default() {
        let mut config = Config::default();
        config
            .delegators
            .push(make_delegator("claude-opus", "claude", "opus"));

        let options = resolve_launch_options(&config, None, None, None, false, None).unwrap();
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
            resolve_launch_options(&config, Some("gemini-pro"), None, None, false, None).unwrap();
        let provider = options.provider.unwrap();
        assert_eq!(provider.tool, "gemini");
        assert_eq!(provider.model, "pro");
    }

    #[test]
    fn test_resolve_unknown_delegator_errors() {
        let config = Config::default();
        let result = resolve_launch_options(&config, Some("nonexistent"), None, None, false, None);
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

        let options = resolve_launch_options(&config, None, None, None, false, Some(&ctx)).unwrap();
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

        let options = resolve_launch_options(&config, None, None, None, false, Some(&ctx)).unwrap();
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

        let options = resolve_launch_options(&config, None, None, None, false, Some(&ctx)).unwrap();
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
            }),
        });

        let options =
            resolve_launch_options(&config, Some("full"), None, None, false, None).unwrap();
        assert!(options.yolo_mode);
        assert!(options.docker_mode);
        assert_eq!(options.use_worktrees_override, Some(true));
        assert_eq!(options.create_branch_override, Some(false));
        assert_eq!(options.extra_flags, vec!["--verbose".to_string()]);
        assert_eq!(options.prompt_prefix.as_deref(), Some("PREFIX"));
        assert_eq!(options.prompt_suffix.as_deref(), Some("SUFFIX"));
    }

    #[test]
    fn test_resolve_yolo_passthrough() {
        let config = Config::default();
        let options = resolve_launch_options(&config, None, None, None, true, None).unwrap();
        assert!(options.yolo_mode);
    }
}

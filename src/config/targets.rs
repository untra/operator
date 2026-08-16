//! Named execution targets — where a launched agent process runs.
//!
//! `[[targets]]` entries collapse the legacy trio of environment knobs
//! (`launch.docker` + `DelegatorLaunchConfig.docker`, `[[hosts]]` +
//! `DelegatorLaunchConfig.host`) into one registry referenced by name from
//! `DelegatorLaunchConfig.target`. Legacy inputs are synthesized into
//! `TargetDef`s at resolution time rather than branching, so the resolver has
//! exactly one output type.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::llm_tools::RemoteHost;
use super::DockerConfig;

/// Reserved target name for the local (no-wrapper) environment.
pub const TARGET_LOCAL: &str = "local";
/// Reserved target name synthesized from `[launch.docker]`.
pub const TARGET_DOCKER: &str = "docker";

/// A named execution target agents can be launched on.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export)]
pub struct TargetDef {
    /// Unique name, referenced by `DelegatorLaunchConfig.target`.
    /// `local` and `docker` are reserved for synthesized targets.
    pub name: String,
    /// Human-readable name for UI surfaces
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    /// Target kind and its kind-specific configuration
    #[serde(flatten)]
    pub kind: TargetKind,
}

impl Default for TargetDef {
    fn default() -> Self {
        Self::local()
    }
}

impl TargetDef {
    /// The synthesized local target: run the agent process directly.
    pub fn local() -> Self {
        Self {
            name: TARGET_LOCAL.to_string(),
            display_name: None,
            kind: TargetKind::Local,
        }
    }

    /// The synthesized docker target from `[launch.docker]`.
    pub fn docker(config: DockerConfig) -> Self {
        Self {
            name: TARGET_DOCKER.to_string(),
            display_name: None,
            kind: TargetKind::Docker(config),
        }
    }

    /// A synthesized ssh target from a `[[hosts]]` entry.
    pub fn from_host(host: &RemoteHost) -> Self {
        Self {
            name: host.name.clone(),
            display_name: host.display_name.clone(),
            kind: TargetKind::Ssh(SshTarget {
                ssh_alias: host.ssh_alias.clone(),
                workdir: host.workdir.clone(),
                ssh_config_path: host.ssh_config_path.clone(),
            }),
        }
    }

    /// For ssh targets, the full `RemoteHost` the remote launch path consumes
    /// (`name`/`display_name` from the def, connection details from the payload).
    pub fn as_remote_host(&self) -> Option<RemoteHost> {
        match &self.kind {
            TargetKind::Ssh(ssh) => Some(RemoteHost {
                name: self.name.clone(),
                ssh_alias: ssh.ssh_alias.clone(),
                workdir: ssh.workdir.clone(),
                display_name: self.display_name.clone(),
                ssh_config_path: ssh.ssh_config_path.clone(),
            }),
            _ => None,
        }
    }
}

/// Execution-target kind, tagged by `kind` in config.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(tag = "kind", rename_all = "lowercase")]
#[ts(export)]
pub enum TargetKind {
    /// Run the agent process directly on this machine
    Local,
    /// Wrap the agent command in a `docker run` container.
    Docker(DockerConfig),
    /// Run inside a Coder workspace over a provisioned SSH alias
    Coder(CoderConfig),
    /// Run on a remote machine over SSH
    Ssh(SshTarget),
}

/// SSH target payload. Name and display name live on `TargetDef`; this is the
/// connection shape (`RemoteHost` minus identity).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export)]
pub struct SshTarget {
    /// Host alias resolved via the user's `~/.ssh/config` (or `ssh_config_path`)
    pub ssh_alias: String,
    /// Absolute project root on the remote machine
    pub workdir: String,
    /// SSH config fragment passed with `-F` (used by provisioned coder aliases)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ssh_config_path: Option<String>,
}

/// Coder workspace target: lifecycle + alias provisioning around the shared
/// SSH remote-launch path. There is no `enabled` field — presence in
/// `[[targets]]` is the enablement.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export)]
pub struct CoderConfig {
    /// Coder template child workspaces are created from (an allowlist —
    /// never per-ticket input)
    pub template: String,
    /// Env var NAME holding the Coder deployment URL
    #[serde(default = "default_coder_url_env")]
    pub url_env: String,
    /// Env var NAME holding the Coder session token. The variable is stripped
    /// from every agent's spawn environment on all target kinds.
    #[serde(default = "default_coder_token_env")]
    pub token_env: String,
    /// Workspace name prefix for deterministic per-ticket naming
    #[serde(default = "default_coder_name_prefix")]
    pub name_prefix: String,
    /// Project root inside the workspace (None = workspace $HOME)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workdir: Option<String>,
    /// Stop the workspace when the ticket completes (never delete)
    #[serde(default = "default_true")]
    pub stop_on_complete: bool,
    /// Bound on workspace create + agent-ready wait
    #[serde(default = "default_coder_create_timeout_secs")]
    pub create_timeout_secs: u64,
    /// Control-plane-reachable `OPERATOR_API_URL` override for detached
    /// multi-step (empty/None = reverse tunnel default)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub callback_url: Option<String>,
    /// Passthrough `-p` template parameters for `coder create`
    #[serde(default, skip_serializing_if = "std::collections::HashMap::is_empty")]
    pub parameters: std::collections::HashMap<String, String>,
}

fn default_coder_url_env() -> String {
    "CODER_URL".to_string()
}

fn default_coder_token_env() -> String {
    "CODER_SESSION_TOKEN".to_string()
}

fn default_coder_name_prefix() -> String {
    "op".to_string()
}

fn default_coder_create_timeout_secs() -> u64 {
    300
}

fn default_true() -> bool {
    true
}

/// Validate the target registry and every reference into it. Hard errors keep
/// startup honest (an unknown target must never silently fall back to local);
/// deprecated-combination cases warn instead so legacy configs keep working.
pub fn validate_targets(config: &super::Config) -> anyhow::Result<()> {
    let mut seen = std::collections::HashSet::new();
    for target in &config.targets {
        if !seen.insert(target.name.as_str()) {
            anyhow::bail!("Duplicate [[targets]] name '{}'", target.name);
        }
        if target.name == TARGET_LOCAL || target.name == TARGET_DOCKER {
            anyhow::bail!(
                "[[targets]] name '{}' is reserved for the synthesized {} target",
                target.name,
                target.name
            );
        }
        if config.hosts.iter().any(|h| h.name == target.name) {
            anyhow::bail!(
                "[[targets]] name '{}' collides with a [[hosts]] entry of the same name",
                target.name
            );
        }
        if let TargetKind::Docker(d) = &target.kind {
            if d.enabled {
                tracing::warn!(
                    target = %target.name,
                    "`enabled` is ignored inside a [[targets]] entry — presence is enablement"
                );
            }
        }
    }

    for delegator in &config.delegators {
        let Some(lc) = &delegator.launch_config else {
            continue;
        };
        if let Some(name) = &lc.target {
            if !known_target_name(config, name) {
                anyhow::bail!(
                    "Delegator '{}' references unknown target '{}' (known: {})",
                    delegator.name,
                    name,
                    known_target_names(config).join(", ")
                );
            }
            if lc.docker.is_some() || lc.host.is_some() {
                tracing::warn!(
                    delegator = %delegator.name,
                    "launch_config sets `target` together with deprecated `docker`/`host`; \
                     `target` wins"
                );
            }
        } else if lc.docker == Some(true) && lc.host.is_some() {
            tracing::warn!(
                delegator = %delegator.name,
                "launch_config sets both `docker` and `host` (deprecated); host wins — \
                 migrate to `target`"
            );
        }
    }

    Ok(())
}

/// Target names offered by launch pickers: `local` first, the synthesized
/// `docker` target when `launch.docker.enabled` gates it on, then explicit
/// `[[targets]]` entries and `[[hosts]]` synths.
pub fn launchable_target_names(config: &super::Config) -> Vec<String> {
    let mut names = vec![TARGET_LOCAL.to_string()];
    if config.launch.docker.enabled {
        names.push(TARGET_DOCKER.to_string());
    }
    names.extend(config.targets.iter().map(|t| t.name.clone()));
    names.extend(config.hosts.iter().map(|h| h.name.clone()));
    names
}

/// Env-var NAMES holding Coder session tokens across all configured coder
/// targets. These are stripped from every agent's spawn environment on ALL
/// target kinds — an agent launched with a Local target inside the operator's
/// own Coder workspace would otherwise read the token straight out of `env`.
pub fn coder_token_envs(config: &super::Config) -> Vec<String> {
    let mut names: Vec<String> = config
        .targets
        .iter()
        .filter_map(|t| match &t.kind {
            TargetKind::Coder(c) => Some(c.token_env.clone()),
            _ => None,
        })
        .collect();
    names.sort();
    names.dedup();
    names
}

/// Whether `name` resolves to any explicit or synthesized target.
pub fn known_target_name(config: &super::Config, name: &str) -> bool {
    name == TARGET_LOCAL
        || name == TARGET_DOCKER
        || config.targets.iter().any(|t| t.name == name)
        || config.hosts.iter().any(|h| h.name == name)
}

/// All resolvable target names: explicit entries, builtins, then host synths.
pub fn known_target_names(config: &super::Config) -> Vec<String> {
    let mut names: Vec<String> = config.targets.iter().map(|t| t.name.clone()).collect();
    names.push(TARGET_LOCAL.to_string());
    names.push(TARGET_DOCKER.to_string());
    names.extend(config.hosts.iter().map(|h| h.name.clone()));
    names
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_target_def_docker_toml_roundtrip() {
        let toml_src = r#"
name = "sandbox"
kind = "docker"
image = "ghcr.io/untra/operator:0.2.6"
"#;
        let def: TargetDef = toml::from_str(toml_src).unwrap();
        assert_eq!(def.name, "sandbox");
        match &def.kind {
            TargetKind::Docker(d) => assert_eq!(d.image, "ghcr.io/untra/operator:0.2.6"),
            other => panic!("expected docker kind, got {other:?}"),
        }
        let back = toml::to_string(&def).unwrap();
        let re: TargetDef = toml::from_str(&back).unwrap();
        assert_eq!(def, re);
    }

    #[test]
    fn test_target_def_ssh_toml() {
        let toml_src = r#"
name = "gpu-vm"
kind = "ssh"
ssh_alias = "gpu-vm"
workdir = "/home/me/proj"
"#;
        let def: TargetDef = toml::from_str(toml_src).unwrap();
        match &def.kind {
            TargetKind::Ssh(s) => {
                assert_eq!(s.ssh_alias, "gpu-vm");
                assert_eq!(s.workdir, "/home/me/proj");
                assert_eq!(s.ssh_config_path, None);
            }
            other => panic!("expected ssh kind, got {other:?}"),
        }
    }

    #[test]
    fn test_target_def_coder_toml_defaults() {
        let toml_src = r#"
name = "cloud"
kind = "coder"
template = "operator-agent"
"#;
        let def: TargetDef = toml::from_str(toml_src).unwrap();
        match &def.kind {
            TargetKind::Coder(c) => {
                assert_eq!(c.template, "operator-agent");
                assert_eq!(c.url_env, "CODER_URL");
                assert_eq!(c.token_env, "CODER_SESSION_TOKEN");
                assert_eq!(c.name_prefix, "op");
                assert!(c.stop_on_complete);
                assert_eq!(c.create_timeout_secs, 300);
                assert!(c.callback_url.is_none());
                assert!(c.parameters.is_empty());
            }
            other => panic!("expected coder kind, got {other:?}"),
        }
    }

    #[test]
    fn test_target_def_local_toml() {
        let def: TargetDef = toml::from_str("name = \"here\"\nkind = \"local\"\n").unwrap();
        assert_eq!(def.kind, TargetKind::Local);
    }

    #[test]
    fn test_target_def_unknown_kind_errors() {
        let result: Result<TargetDef, _> = toml::from_str("name = \"x\"\nkind = \"lambda\"\n");
        assert!(result.is_err(), "unknown kind must be a hard parse error");
    }

    #[test]
    fn test_target_def_missing_kind_errors() {
        let result: Result<TargetDef, _> = toml::from_str("name = \"x\"\n");
        assert!(result.is_err(), "missing kind must be a hard parse error");
    }

    #[test]
    fn test_from_host_and_as_remote_host_roundtrip() {
        let host = RemoteHost {
            name: "gpu-vm".to_string(),
            ssh_alias: "gpu-alias".to_string(),
            workdir: "/home/me/proj".to_string(),
            display_name: Some("GPU VM".to_string()),
            ssh_config_path: None,
        };
        let def = TargetDef::from_host(&host);
        assert_eq!(def.name, "gpu-vm");
        assert_eq!(def.as_remote_host(), Some(host));
    }

    #[test]
    fn test_as_remote_host_none_for_non_ssh() {
        assert_eq!(TargetDef::local().as_remote_host(), None);
    }

    #[test]
    fn test_default_target_is_local() {
        assert_eq!(TargetDef::default(), TargetDef::local());
    }

    fn config_with_targets(targets: Vec<TargetDef>) -> crate::config::Config {
        crate::config::Config {
            targets,
            ..Default::default()
        }
    }

    fn ssh_def(name: &str) -> TargetDef {
        TargetDef {
            name: name.to_string(),
            display_name: None,
            kind: TargetKind::Ssh(SshTarget {
                ssh_alias: name.to_string(),
                workdir: "/proj".to_string(),
                ssh_config_path: None,
            }),
        }
    }

    #[test]
    fn test_validate_targets_duplicate_names_error() {
        let config = config_with_targets(vec![ssh_def("a"), ssh_def("a")]);
        let err = validate_targets(&config).unwrap_err().to_string();
        assert!(err.contains("Duplicate"), "{err}");
    }

    #[test]
    fn test_validate_targets_reserved_names_error() {
        for reserved in [TARGET_LOCAL, TARGET_DOCKER] {
            let config = config_with_targets(vec![ssh_def(reserved)]);
            let err = validate_targets(&config).unwrap_err().to_string();
            assert!(err.contains("reserved"), "{err}");
        }
    }

    #[test]
    fn test_validate_targets_host_collision_error() {
        let mut config = config_with_targets(vec![ssh_def("gpu-vm")]);
        config.hosts.push(RemoteHost {
            name: "gpu-vm".to_string(),
            ssh_alias: "gpu".to_string(),
            workdir: "/p".to_string(),
            display_name: None,
            ssh_config_path: None,
        });
        let err = validate_targets(&config).unwrap_err().to_string();
        assert!(err.contains("collides"), "{err}");
    }

    #[test]
    fn test_validate_targets_unknown_delegator_reference_error() {
        let mut config = config_with_targets(vec![]);
        config.delegators.push(crate::config::Delegator {
            name: "heavy".to_string(),
            llm_tool: "claude".to_string(),
            model: "opus".to_string(),
            display_name: None,
            model_properties: std::collections::HashMap::new(),
            launch_config: Some(crate::config::DelegatorLaunchConfig {
                target: Some("nope".to_string()),
                ..Default::default()
            }),
            model_server: None,
            remote_agent: None,
            x_agnt: None,
            x_openai: None,
            unmapped_core: None,
        });
        let err = validate_targets(&config).unwrap_err().to_string();
        assert!(err.contains("unknown target 'nope'"), "{err}");
        assert!(
            err.contains("local"),
            "error should list known names: {err}"
        );
    }

    #[test]
    fn test_validate_targets_builtin_and_host_references_ok() {
        let mut config = config_with_targets(vec![ssh_def("gpu-vm")]);
        config.hosts.push(RemoteHost {
            name: "legacy-host".to_string(),
            ssh_alias: "l".to_string(),
            workdir: "/p".to_string(),
            display_name: None,
            ssh_config_path: None,
        });
        for name in ["local", "docker", "gpu-vm", "legacy-host"] {
            config.delegators = vec![crate::config::Delegator {
                name: "d".to_string(),
                llm_tool: "claude".to_string(),
                model: "opus".to_string(),
                display_name: None,
                model_properties: std::collections::HashMap::new(),
                launch_config: Some(crate::config::DelegatorLaunchConfig {
                    target: Some(name.to_string()),
                    ..Default::default()
                }),
                model_server: None,
                remote_agent: None,
                x_agnt: None,
                x_openai: None,
                unmapped_core: None,
            }];
            assert!(
                validate_targets(&config).is_ok(),
                "'{name}' should be a known target name"
            );
        }
    }
}

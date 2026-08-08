//! Launch and relaunch options for agent sessions

use crate::config::{LlmProvider, TargetDef, TargetKind};

/// Launch options for starting an agent with specific provider and mode settings
#[derive(Debug, Clone, Default)]
pub struct LaunchOptions {
    /// LLM provider to use (if None, use default)
    pub provider: Option<LlmProvider>,
    /// Delegator name used for this launch (for state tracking and step-level switching)
    pub delegator_name: Option<String>,
    /// Additional CLI flags from delegator `launch_config`
    pub extra_flags: Vec<String>,
    /// Resolved execution target for the agent process (default: local)
    pub target: TargetDef,
    /// Run in YOLO (auto-accept) mode
    pub yolo_mode: bool,
    /// Override project path (if None, use ticket's project)
    pub project_override: Option<String>,
    /// Override global `git.use_worktrees` from delegator (None = use global config)
    pub use_worktrees_override: Option<bool>,
    /// Override branch creation from delegator (None = default behavior)
    pub create_branch_override: Option<bool>,
    /// Prompt text to prepend before the generated step prompt
    pub prompt_prefix: Option<String>,
    /// Prompt text to append after the generated step prompt
    pub prompt_suffix: Option<String>,
    /// Suffix appended to the generated session name to differentiate
    /// multiple sub-agents launched for the same ticket (multi-agent steps).
    /// When `None`, session name is the usual `{prefix}{sanitized-ticket-id}`.
    pub session_suffix: Option<String>,
    /// Enable relay MCP server injection for this launch (None = use global config)
    pub operator_relay: Option<bool>,
    /// Provisioned remote host for coder targets (set by the launcher after
    /// workspace provisioning; None until then and for non-coder targets)
    pub provisioned_host: Option<crate::config::RemoteHost>,
    /// `OPERATOR_API_URL` override for remote launches (a coder target's
    /// control-plane-reachable `callback_url`); None = reverse-tunnel default
    pub api_url_override: Option<String>,
}

/// Persisted launch-mode kind, derived from the resolved execution target.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LaunchModeKind {
    #[default]
    Local,
    Docker,
    Coder,
    Ssh,
}

impl LaunchModeKind {
    fn base(self) -> &'static str {
        match self {
            Self::Local => "default",
            Self::Docker => "docker",
            Self::Coder => "coder",
            Self::Ssh => "ssh",
        }
    }
}

/// A `launch_mode` string parsed back into structured form.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ParsedLaunchMode {
    pub kind: LaunchModeKind,
    pub yolo: bool,
}

/// Parse a persisted `launch_mode` string. Legacy values ("default", "yolo",
/// "docker", "docker-yolo") and unknown strings all parse — old state files
/// predate the coder/ssh vocabulary.
pub fn parse_launch_mode(s: &str) -> ParsedLaunchMode {
    let (base, yolo) = match s.strip_suffix("-yolo") {
        Some(base) => (base, true),
        None if s == "yolo" => ("default", true),
        None => (s, false),
    };
    let kind = match base {
        "docker" => LaunchModeKind::Docker,
        "coder" => LaunchModeKind::Coder,
        "ssh" => LaunchModeKind::Ssh,
        _ => LaunchModeKind::Local,
    };
    ParsedLaunchMode { kind, yolo }
}

impl LaunchOptions {
    /// The launch-mode kind for this launch's resolved target.
    pub fn launch_mode_kind(&self) -> LaunchModeKind {
        match self.target.kind {
            TargetKind::Local => LaunchModeKind::Local,
            TargetKind::Docker(_) => LaunchModeKind::Docker,
            TargetKind::Coder(_) => LaunchModeKind::Coder,
            TargetKind::Ssh(_) => LaunchModeKind::Ssh,
        }
    }

    /// Whether this launch wraps the command in a docker container.
    pub fn is_docker(&self) -> bool {
        matches!(self.target.kind, TargetKind::Docker(_))
    }

    /// The remote host this launch runs on, if any: the provisioned coder
    /// workspace when set, else an ssh target's declared host.
    pub fn remote_host(&self) -> Option<crate::config::RemoteHost> {
        self.provisioned_host
            .clone()
            .or_else(|| self.target.as_remote_host())
    }

    /// Get the launch mode string for state tracking — the single derivation
    /// point for the persisted vocabulary:
    /// `default|yolo|docker[-yolo]|coder[-yolo]|ssh[-yolo]`.
    pub fn launch_mode_string(&self) -> String {
        let kind = self.launch_mode_kind();
        match (kind, self.yolo_mode) {
            (LaunchModeKind::Local, false) => "default".to_string(),
            (LaunchModeKind::Local, true) => "yolo".to_string(),
            (kind, false) => kind.base().to_string(),
            (kind, true) => format!("{}-yolo", kind.base()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{CoderConfig, DockerConfig, SshTarget};

    fn target(kind: TargetKind) -> TargetDef {
        TargetDef {
            name: "t".to_string(),
            display_name: None,
            kind,
        }
    }

    fn options_with(kind: TargetKind, yolo: bool) -> LaunchOptions {
        LaunchOptions {
            target: target(kind),
            yolo_mode: yolo,
            ..Default::default()
        }
    }

    fn coder_config() -> CoderConfig {
        CoderConfig {
            template: "operator-agent".to_string(),
            url_env: "CODER_URL".to_string(),
            token_env: "CODER_SESSION_TOKEN".to_string(),
            name_prefix: "op".to_string(),
            workdir: None,
            stop_on_complete: true,
            create_timeout_secs: 300,
            callback_url: None,
            parameters: std::collections::HashMap::new(),
        }
    }

    fn ssh_target() -> SshTarget {
        SshTarget {
            ssh_alias: "gpu".to_string(),
            workdir: "/proj".to_string(),
            ssh_config_path: None,
        }
    }

    #[test]
    fn test_launch_options_carries_operator_relay() {
        let opts = LaunchOptions {
            provider: None,
            delegator_name: None,
            extra_flags: vec![],
            target: TargetDef::local(),
            yolo_mode: false,
            project_override: None,
            use_worktrees_override: None,
            create_branch_override: None,
            prompt_prefix: None,
            prompt_suffix: None,
            session_suffix: None,
            operator_relay: Some(false),
            provisioned_host: None,
            api_url_override: None,
        };
        assert_eq!(opts.operator_relay, Some(false));
    }

    #[test]
    fn test_launch_mode_string_all_kinds() {
        let cases = [
            (TargetKind::Local, false, "default"),
            (TargetKind::Local, true, "yolo"),
            (TargetKind::Docker(DockerConfig::default()), false, "docker"),
            (
                TargetKind::Docker(DockerConfig::default()),
                true,
                "docker-yolo",
            ),
            (TargetKind::Coder(coder_config()), false, "coder"),
            (TargetKind::Coder(coder_config()), true, "coder-yolo"),
            (TargetKind::Ssh(ssh_target()), false, "ssh"),
            (TargetKind::Ssh(ssh_target()), true, "ssh-yolo"),
        ];
        for (kind, yolo, expected) in cases {
            assert_eq!(options_with(kind, yolo).launch_mode_string(), expected);
        }
    }

    #[test]
    fn test_launch_mode_roundtrip() {
        let kinds = [
            TargetKind::Local,
            TargetKind::Docker(DockerConfig::default()),
            TargetKind::Coder(coder_config()),
            TargetKind::Ssh(ssh_target()),
        ];
        for kind in kinds {
            for yolo in [false, true] {
                let opts = options_with(kind.clone(), yolo);
                let parsed = parse_launch_mode(&opts.launch_mode_string());
                assert_eq!(parsed.kind, opts.launch_mode_kind());
                assert_eq!(parsed.yolo, yolo);
            }
        }
    }

    #[test]
    fn test_parse_launch_mode_legacy_strings() {
        assert_eq!(
            parse_launch_mode("default"),
            ParsedLaunchMode {
                kind: LaunchModeKind::Local,
                yolo: false
            }
        );
        assert_eq!(
            parse_launch_mode("yolo"),
            ParsedLaunchMode {
                kind: LaunchModeKind::Local,
                yolo: true
            }
        );
        assert_eq!(
            parse_launch_mode("docker"),
            ParsedLaunchMode {
                kind: LaunchModeKind::Docker,
                yolo: false
            }
        );
        assert_eq!(
            parse_launch_mode("docker-yolo"),
            ParsedLaunchMode {
                kind: LaunchModeKind::Docker,
                yolo: true
            }
        );
    }

    #[test]
    fn test_parse_launch_mode_unknown_is_local() {
        assert_eq!(parse_launch_mode("garbage"), ParsedLaunchMode::default());
        assert_eq!(parse_launch_mode(""), ParsedLaunchMode::default());
    }
}

/// Options for relaunching an existing in-progress ticket
#[derive(Debug, Clone, Default)]
pub struct RelaunchOptions {
    /// Base launch options
    pub launch_options: LaunchOptions,
    /// Existing Claude session UUID to resume (optional)
    /// If provided, uses --resume flag with the existing prompt file
    pub resume_session_id: Option<String>,
    /// Feedback from previous attempt (what went wrong)
    pub retry_reason: Option<String>,
}

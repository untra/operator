//! Centralized environment variable registry.
//!
//! This module provides a single source of truth for all environment variables
//! used by Operator. It is consumed by:
//! - `CliDocGenerator` for generating documentation
//! - Runtime validation (future)
//!
//! All environment variables use the `OPERATOR_` prefix with `__` separator
//! for nested config paths (e.g., `OPERATOR_AGENTS__MAX_PARALLEL`).

/// An environment variable definition
#[derive(Debug, Clone)]
pub struct EnvVar {
    /// Environment variable name (e.g., "OPERATOR_AGENTS__MAX_PARALLEL")
    pub name: &'static str,
    /// Human-readable description
    pub description: &'static str,
    /// Category for grouping in documentation
    pub category: EnvVarCategory,
    /// Whether this variable is required for operation
    pub required: bool,
    /// Default value if not set
    pub default: Option<&'static str>,
    /// Example value for documentation
    pub example: Option<&'static str>,
}

/// Categories for organizing environment variables
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EnvVarCategory {
    /// API keys and tokens
    Authentication,
    /// Agent lifecycle configuration
    Agents,
    /// Queue processing configuration
    Queue,
    /// Notification settings
    Notifications,
    /// File path configuration
    Paths,
    /// Terminal UI settings
    Ui,
    /// Agent launch behavior
    Launch,
    /// Tmux integration
    Tmux,
    /// Backstage server settings
    Backstage,
    /// LLM tool allowlist/denylist
    LlmTools,
    /// Logging configuration
    Logging,
}

impl EnvVarCategory {
    /// Display name for this category
    pub fn display_name(&self) -> &'static str {
        match self {
            EnvVarCategory::Authentication => "Authentication",
            EnvVarCategory::Agents => "Agents",
            EnvVarCategory::Queue => "Queue",
            EnvVarCategory::Notifications => "Notifications",
            EnvVarCategory::Paths => "Paths",
            EnvVarCategory::Ui => "UI",
            EnvVarCategory::Launch => "Launch",
            EnvVarCategory::Tmux => "Tmux",
            EnvVarCategory::Backstage => "Backstage",
            EnvVarCategory::LlmTools => "LLM Tools",
            EnvVarCategory::Logging => "Logging",
        }
    }

    /// All categories in display order
    pub fn all() -> &'static [EnvVarCategory] {
        &[
            EnvVarCategory::Authentication,
            EnvVarCategory::Agents,
            EnvVarCategory::Queue,
            EnvVarCategory::Notifications,
            EnvVarCategory::Paths,
            EnvVarCategory::Ui,
            EnvVarCategory::Launch,
            EnvVarCategory::Tmux,
            EnvVarCategory::Backstage,
            EnvVarCategory::LlmTools,
            EnvVarCategory::Logging,
        ]
    }
}

/// Static registry of all documented environment variables
pub static ENV_VARS: &[EnvVar] = &[
    // === Authentication ===
    EnvVar {
        name: "OPERATOR_API__ANTHROPIC_API_KEY",
        description: "Anthropic API key for rate limit monitoring and AI provider status",
        category: EnvVarCategory::Authentication,
        required: false,
        default: None,
        example: Some("sk-ant-api03-..."),
    },
    EnvVar {
        name: "OPERATOR_API__GITHUB_TOKEN",
        description: "GitHub personal access token for PR/issue tracking integration",
        category: EnvVarCategory::Authentication,
        required: false,
        default: None,
        example: Some("ghp_..."),
    },
    // === Agents ===
    EnvVar {
        name: "OPERATOR_AGENTS__MAX_PARALLEL",
        description: "Maximum number of agents that can run in parallel",
        category: EnvVarCategory::Agents,
        required: false,
        default: Some("4"),
        example: Some("2"),
    },
    EnvVar {
        name: "OPERATOR_AGENTS__CORES_RESERVED",
        description: "Number of CPU cores to reserve (not used by agents)",
        category: EnvVarCategory::Agents,
        required: false,
        default: Some("2"),
        example: Some("4"),
    },
    EnvVar {
        name: "OPERATOR_AGENTS__STALE_MINUTES",
        description: "Minutes of inactivity before an agent is considered stale",
        category: EnvVarCategory::Agents,
        required: false,
        default: Some("30"),
        example: Some("60"),
    },
    EnvVar {
        name: "OPERATOR_AGENTS__HEALTH_CHECK_INTERVAL_SECS",
        description: "Interval in seconds between agent health checks",
        category: EnvVarCategory::Agents,
        required: false,
        default: Some("60"),
        example: Some("30"),
    },
    EnvVar {
        name: "OPERATOR_AGENTS__COMPLETION_DETECTION_INTERVAL_SECS",
        description: "Interval in seconds between completion detection checks",
        category: EnvVarCategory::Agents,
        required: false,
        default: Some("5"),
        example: Some("10"),
    },
    EnvVar {
        name: "OPERATOR_AGENTS__SESSION_DIR",
        description: "Directory for storing agent session data",
        category: EnvVarCategory::Agents,
        required: false,
        default: Some(".claude/sessions"),
        example: Some("/tmp/operator/sessions"),
    },
    EnvVar {
        name: "OPERATOR_AGENTS__ENABLE_NOTIFICATIONS",
        description: "Enable macOS notifications for agent events",
        category: EnvVarCategory::Agents,
        required: false,
        default: Some("true"),
        example: Some("false"),
    },
    // === Queue ===
    EnvVar {
        name: "OPERATOR_QUEUE__AUTO_ASSIGN",
        description: "Automatically assign tickets to available agents",
        category: EnvVarCategory::Queue,
        required: false,
        default: Some("true"),
        example: Some("false"),
    },
    EnvVar {
        name: "OPERATOR_QUEUE__POLL_INTERVAL_SECS",
        description: "Interval in seconds between queue polling cycles",
        category: EnvVarCategory::Queue,
        required: false,
        default: Some("5"),
        example: Some("10"),
    },
    EnvVar {
        name: "OPERATOR_QUEUE__PRIORITY_ORDER",
        description: "Comma-separated list of ticket types in priority order",
        category: EnvVarCategory::Queue,
        required: false,
        default: Some("INV,FIX,FEAT,SPIKE"),
        example: Some("FIX,FEAT,INV,SPIKE"),
    },
    // === Notifications ===
    EnvVar {
        name: "OPERATOR_NOTIFICATIONS__ENABLED",
        description: "Enable the notification system",
        category: EnvVarCategory::Notifications,
        required: false,
        default: Some("true"),
        example: Some("false"),
    },
    EnvVar {
        name: "OPERATOR_NOTIFICATIONS__ON_LAUNCH",
        description: "Send notification when an agent launches",
        category: EnvVarCategory::Notifications,
        required: false,
        default: Some("true"),
        example: Some("false"),
    },
    EnvVar {
        name: "OPERATOR_NOTIFICATIONS__ON_COMPLETE",
        description: "Send notification when an agent completes",
        category: EnvVarCategory::Notifications,
        required: false,
        default: Some("true"),
        example: Some("false"),
    },
    EnvVar {
        name: "OPERATOR_NOTIFICATIONS__ON_STALL",
        description: "Send notification when an agent stalls",
        category: EnvVarCategory::Notifications,
        required: false,
        default: Some("true"),
        example: Some("false"),
    },
    EnvVar {
        name: "OPERATOR_NOTIFICATIONS__ON_ERROR",
        description: "Send notification on agent errors",
        category: EnvVarCategory::Notifications,
        required: false,
        default: Some("true"),
        example: Some("false"),
    },
    EnvVar {
        name: "OPERATOR_NOTIFICATIONS__SOUND",
        description: "Play sound with notifications",
        category: EnvVarCategory::Notifications,
        required: false,
        default: Some("true"),
        example: Some("false"),
    },
    // === Paths ===
    EnvVar {
        name: "OPERATOR_PATHS__TICKETS",
        description: "Directory containing ticket files",
        category: EnvVarCategory::Paths,
        required: false,
        default: Some(".tickets"),
        example: Some("/path/to/tickets"),
    },
    EnvVar {
        name: "OPERATOR_PATHS__PROJECTS",
        description: "Root directory for project discovery",
        category: EnvVarCategory::Paths,
        required: false,
        default: Some("."),
        example: Some("/home/user/projects"),
    },
    EnvVar {
        name: "OPERATOR_PATHS__STATE",
        description: "Directory for persistent operator state",
        category: EnvVarCategory::Paths,
        required: false,
        default: Some(".tickets/operator"),
        example: Some("/var/lib/operator/state"),
    },
    // === UI ===
    EnvVar {
        name: "OPERATOR_UI__REFRESH_RATE_MS",
        description: "UI refresh rate in milliseconds",
        category: EnvVarCategory::Ui,
        required: false,
        default: Some("250"),
        example: Some("100"),
    },
    EnvVar {
        name: "OPERATOR_UI__SUMMARY_MAX_LENGTH",
        description: "Maximum length of ticket summaries in the UI",
        category: EnvVarCategory::Ui,
        required: false,
        default: Some("60"),
        example: Some("80"),
    },
    // === Launch ===
    EnvVar {
        name: "OPERATOR_LAUNCH__MODE",
        description: "Agent launch mode (tmux or direct)",
        category: EnvVarCategory::Launch,
        required: false,
        default: Some("tmux"),
        example: Some("direct"),
    },
    EnvVar {
        name: "OPERATOR_LAUNCH__CONFIRM",
        description: "Require confirmation before launching agents",
        category: EnvVarCategory::Launch,
        required: false,
        default: Some("true"),
        example: Some("false"),
    },
    // === Tmux ===
    EnvVar {
        name: "OPERATOR_TMUX__SESSION_PREFIX",
        description: "Prefix for tmux session names",
        category: EnvVarCategory::Tmux,
        required: false,
        default: Some("operator"),
        example: Some("agent"),
    },
    // === Backstage ===
    EnvVar {
        name: "OPERATOR_BACKSTAGE__PORT",
        description: "Port for the Backstage web server",
        category: EnvVarCategory::Backstage,
        required: false,
        default: Some("3000"),
        example: Some("8080"),
    },
    EnvVar {
        name: "OPERATOR_BACKSTAGE__AUTO_START",
        description: "Automatically start Backstage server with TUI",
        category: EnvVarCategory::Backstage,
        required: false,
        default: Some("false"),
        example: Some("true"),
    },
    // === LLM Tools ===
    EnvVar {
        name: "OPERATOR_LLM_TOOLS__ENABLED",
        description: "Enable LLM tool allowlist/denylist functionality",
        category: EnvVarCategory::LlmTools,
        required: false,
        default: Some("true"),
        example: Some("false"),
    },
    EnvVar {
        name: "OPERATOR_LLM_TOOLS__ALLOWED",
        description: "Comma-separated list of allowed LLM tools (empty = all allowed)",
        category: EnvVarCategory::LlmTools,
        required: false,
        default: Some(""),
        example: Some("Read,Write,Edit"),
    },
    EnvVar {
        name: "OPERATOR_LLM_TOOLS__DENIED",
        description: "Comma-separated list of denied LLM tools",
        category: EnvVarCategory::LlmTools,
        required: false,
        default: Some(""),
        example: Some("Bash,WebFetch"),
    },
    // === Logging ===
    EnvVar {
        name: "OPERATOR_LOGGING__LEVEL",
        description: "Log level (trace, debug, info, warn, error)",
        category: EnvVarCategory::Logging,
        required: false,
        default: Some("info"),
        example: Some("debug"),
    },
    EnvVar {
        name: "OPERATOR_LOGGING__TO_FILE",
        description: "Write logs to file in addition to stderr",
        category: EnvVarCategory::Logging,
        required: false,
        default: Some("true"),
        example: Some("false"),
    },
];

/// Get all environment variables for a given category
pub fn env_vars_for_category(category: EnvVarCategory) -> impl Iterator<Item = &'static EnvVar> {
    ENV_VARS.iter().filter(move |v| v.category == category)
}

/// Get environment variables grouped by category
pub fn env_vars_by_category() -> Vec<(EnvVarCategory, Vec<&'static EnvVar>)> {
    EnvVarCategory::all()
        .iter()
        .map(|cat| {
            let vars: Vec<&EnvVar> = env_vars_for_category(*cat).collect();
            (*cat, vars)
        })
        .filter(|(_, vars)| !vars.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_env_vars_have_descriptions() {
        for var in ENV_VARS {
            assert!(
                !var.description.is_empty(),
                "EnvVar {} has empty description",
                var.name
            );
        }
    }

    #[test]
    fn test_all_env_vars_have_operator_prefix() {
        for var in ENV_VARS {
            assert!(
                var.name.starts_with("OPERATOR_"),
                "EnvVar {} does not have OPERATOR_ prefix",
                var.name
            );
        }
    }

    #[test]
    fn test_env_vars_by_category() {
        let grouped = env_vars_by_category();
        assert!(!grouped.is_empty());

        // Should have authentication category
        let auth_category = grouped
            .iter()
            .find(|(cat, _)| *cat == EnvVarCategory::Authentication);
        assert!(auth_category.is_some());
    }

    #[test]
    fn test_category_display_names() {
        assert_eq!(
            EnvVarCategory::Authentication.display_name(),
            "Authentication"
        );
        assert_eq!(EnvVarCategory::Agents.display_name(), "Agents");
        assert_eq!(EnvVarCategory::Queue.display_name(), "Queue");
        assert_eq!(
            EnvVarCategory::Notifications.display_name(),
            "Notifications"
        );
        assert_eq!(EnvVarCategory::Paths.display_name(), "Paths");
        assert_eq!(EnvVarCategory::Ui.display_name(), "UI");
        assert_eq!(EnvVarCategory::Launch.display_name(), "Launch");
        assert_eq!(EnvVarCategory::Tmux.display_name(), "Tmux");
        assert_eq!(EnvVarCategory::Backstage.display_name(), "Backstage");
        assert_eq!(EnvVarCategory::LlmTools.display_name(), "LLM Tools");
        assert_eq!(EnvVarCategory::Logging.display_name(), "Logging");
    }

    #[test]
    fn test_all_categories_in_order() {
        let all = EnvVarCategory::all();
        assert_eq!(all.len(), 11);
        assert_eq!(all[0], EnvVarCategory::Authentication);
        assert_eq!(all[10], EnvVarCategory::Logging);
    }
}

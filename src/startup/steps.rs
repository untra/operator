//! The setup wizard's step catalog: identity, order and copy.
//!
//! This is the single source of truth for the wizard. The ratatui renderer
//! matches on [`SetupStep`], `docs_gen` renders [`setup_steps`] into
//! `docs/startup/index.md`, and `bindings/SetupStep.ts` is generated from the
//! same enum. Adding a variant is a compile error until [`SetupStep::info`]
//! and [`SetupStep::slug`] account for it, and until [`SetupStep::ALL`] is
//! resized to place it in the walk order.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;

/// Documentation copy for one wizard step.
#[allow(dead_code)] // Used via binary and docs_gen, not reachable from lib.rs
#[derive(Debug, Clone)]
pub struct SetupStepInfo {
    /// Display name of the step (e.g., "Welcome")
    pub name: &'static str,
    /// Brief description of what happens in this step
    pub description: &'static str,
    /// Detailed help text explaining the step
    pub help_text: &'static str,
    /// Navigation instructions (keys to use)
    pub navigation: &'static str,
}

/// A step in the setup wizard.
#[allow(dead_code)] // Used via binary and docs_gen, not reachable from lib.rs
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, ToSchema, TS,
)]
#[ts(export)]
pub enum SetupStep {
    /// Splash screen with discovered projects and detected LLM tools
    #[serde(rename = "welcome")]
    Welcome,
    /// Kanban integration overview and provider credential detection
    #[serde(rename = "kanban-info")]
    KanbanInfo,
    /// Declare which model providers this workspace uses
    #[serde(rename = "model-server")]
    ModelServer,
    /// Connect a git provider so agents can branch, push and open PRs
    #[serde(rename = "git-provider")]
    GitProvider,
    /// Choose which issue type collection to use
    #[serde(rename = "collection-source")]
    CollectionSource,
    /// Browse and multi-select hosted collections
    #[serde(rename = "hosted-collections")]
    HostedCollectionFetch,
    /// Configure optional TASK fields
    #[serde(rename = "task-field-config")]
    TaskFieldConfig,
    /// Select the session wrapper agents launch into
    #[serde(rename = "session-wrapper-choice")]
    SessionWrapperChoice,
    /// Choose where agent commands execute
    #[serde(rename = "execution-target")]
    ExecutionTarget,
    /// Choose in-place branches or per-ticket worktrees
    #[serde(rename = "worktree-preference")]
    WorktreePreference,
    /// Optional admin password for the web dashboard
    #[serde(rename = "admin-password")]
    AdminPassword,
    /// tmux help, shown only when tmux is selected
    #[serde(rename = "tmux-onboarding")]
    TmuxOnboarding,
    /// VS Code extension setup, shown only when VS Code is selected
    #[serde(rename = "vscode-setup")]
    VSCodeSetup,
    /// cmux setup, shown only when cmux is selected
    #[serde(rename = "cmux-setup")]
    CmuxSetup,
    /// Zellij setup, shown only when Zellij is selected
    #[serde(rename = "zellij-setup")]
    ZellijSetup,
    /// Review the acceptance criteria template
    #[serde(rename = "acceptance-criteria")]
    AcceptanceCriteria,
    /// Optionally create bootstrap tickets
    #[serde(rename = "startup-tickets")]
    StartupTickets,
    /// Review and confirm initialization
    #[serde(rename = "confirm")]
    Confirm,
}

#[allow(dead_code)] // Used via binary and docs_gen, not reachable from lib.rs
impl SetupStep {
    /// Every step, in the order the wizard walks them. Conditional steps
    /// (the per-wrapper ones) appear here even though a given run skips most.
    pub const ALL: [SetupStep; 18] = [
        SetupStep::Welcome,
        SetupStep::KanbanInfo,
        SetupStep::ModelServer,
        SetupStep::GitProvider,
        SetupStep::CollectionSource,
        SetupStep::HostedCollectionFetch,
        SetupStep::TaskFieldConfig,
        SetupStep::SessionWrapperChoice,
        SetupStep::ExecutionTarget,
        SetupStep::WorktreePreference,
        SetupStep::AdminPassword,
        SetupStep::TmuxOnboarding,
        SetupStep::VSCodeSetup,
        SetupStep::CmuxSetup,
        SetupStep::ZellijSetup,
        SetupStep::AcceptanceCriteria,
        SetupStep::StartupTickets,
        SetupStep::Confirm,
    ];

    /// Stable identifier used by docs URLs and the web renderer.
    pub fn slug(self) -> &'static str {
        match self {
            SetupStep::Welcome => "welcome",
            SetupStep::KanbanInfo => "kanban-info",
            SetupStep::ModelServer => "model-server",
            SetupStep::GitProvider => "git-provider",
            SetupStep::CollectionSource => "collection-source",
            SetupStep::HostedCollectionFetch => "hosted-collections",
            SetupStep::TaskFieldConfig => "task-field-config",
            SetupStep::SessionWrapperChoice => "session-wrapper-choice",
            SetupStep::ExecutionTarget => "execution-target",
            SetupStep::WorktreePreference => "worktree-preference",
            SetupStep::AdminPassword => "admin-password",
            SetupStep::TmuxOnboarding => "tmux-onboarding",
            SetupStep::VSCodeSetup => "vscode-setup",
            SetupStep::CmuxSetup => "cmux-setup",
            SetupStep::ZellijSetup => "zellij-setup",
            SetupStep::AcceptanceCriteria => "acceptance-criteria",
            SetupStep::StartupTickets => "startup-tickets",
            SetupStep::Confirm => "confirm",
        }
    }

    /// Documentation copy for this step.
    pub fn info(self) -> SetupStepInfo {
        match self {
            SetupStep::Welcome => SetupStepInfo {
                name: "Welcome",
                description: "Splash screen showing detected LLM tools and discovered projects",
                help_text: "The welcome screen displays:\n\
                    - Detected LLM tools (Claude, Gemini, Codex, etc.) with version and model count\n\
                    - Discovered projects organized by which LLM tool marker files they contain\n\
                    - The path where the tickets directory will be created\n\n\
                    This gives you an overview of your development environment before proceeding.",
                navigation: "Enter to continue, Esc to cancel",
            },
            SetupStep::KanbanInfo => SetupStepInfo {
                name: "Kanban Info",
                description: "Connect a kanban provider, or skip and connect one later",
                help_text:
                    "Operator can sync with external kanban providers to pull in issues as tickets.\n\
                    Supported providers: Jira, Linear, GitHub Projects.\n\n\
                    Credentials already exported (e.g. OPERATOR_JIRA_API_KEY) are listed as \
                    detected providers.\n\n\
                    **Connect a kanban provider** opens the same onboarding dialog the dashboard \
                    uses: pick a provider, enter its credentials, validate them against the live \
                    API, and choose a project. The provider section is written to config.toml and \
                    the token is exported into this session, with a shell snippet to make it \
                    permanent.\n\n\
                    **Skip for now** moves on; press `K` from the dashboard at any time.",
                navigation: "↑/↓ to select, Enter to confirm, Esc to go back",
            },
            SetupStep::ModelServer => SetupStepInfo {
                name: "Model Server",
                description: "Declare which model providers this workspace uses",
                help_text:
                    "Model providers are where inference happens - distinct from the agent CLI \
                    that calls them.\n\n\
                    Each provider is probed live: a row reads `N models` when Operator can reach \
                    it, `key missing` when its API key env var is unset, or `unreachable` with \
                    the reason.\n\n\
                    Space declares a provider, writing a `[[model_servers]]` entry. Operator \
                    stores only the *name* of the environment variable holding the key, never \
                    the key itself - export it in your shell to make it permanent.\n\n\
                    Providers needing a custom base URL (OpenAI-compatible, LM Studio) are \
                    listed but not selectable here; add them to config.toml directly.\n\n\
                    This step is optional - Operator ships working defaults for the first-party \
                    vendors.",
                navigation:
                    "↑/↓ or j/k to navigate, Space to declare, Enter to continue, Esc to go back",
            },
            SetupStep::GitProvider => SetupStepInfo {
                name: "Git Provider",
                description: "Connect a git provider so agents can branch, push and open PRs",
                help_text:
                    "Operator branches per ticket and opens pull requests on your behalf, which \
                    needs a provider and a token.\n\n\
                    Each row reports what was found: the provider CLI (`gh`, `glab`, `tea`) not \
                    installed, an existing CLI login Operator can adopt with no typing, or a \
                    prompt for a personal access token.\n\n\
                    Only the *name* of the environment variable holding the token is written to \
                    config.toml. The token itself is exported into this session, and the step \
                    prints the shell line to make that permanent - without it the token is gone \
                    when Operator exits.\n\n\
                    This step is optional; Operator works against a local repository with no \
                    provider connected.",
                navigation:
                    "↑/↓ or j/k to navigate, Enter to connect, Esc to go back",
            },
            SetupStep::CollectionSource => SetupStepInfo {
                name: "Collection Source",
                description: "Choose which issue type collection to use",
                help_text: "Select a preset collection of issue types:\n\
                    - **Simple**: Just TASK - minimal setup for general work\n\
                    - **Dev Kanban**: 3 types (TASK, FEAT, FIX) for development workflows\n\
                    - **DevOps Kanban**: 5 types (TASK, SPIKE, INV, FEAT, FIX) for full DevOps\n\
                    - **Custom Selection**: Choose individual issue types",
                navigation: "↑/↓ or j/k to navigate, Enter to select, Esc to go back",
            },
            SetupStep::HostedCollectionFetch => SetupStepInfo {
                name: "Hosted Collections",
                description: "Browse and select hosted collections (only shown if Browse chosen)",
                help_text: "Pick one or more curated collections published at             operator.untra.io.\n\n\
                    The list is fetched from the collections manifest; if it cannot be             reached, the collections bundled with Operator are offered instead.             Each collection brings its own issue types and workflow steps.\n\n\
                    Selections are additive - choose as many as apply.",
                navigation: "↑/↓ or j/k to navigate, Space to toggle, Enter to continue, Esc to go back",
            },
            SetupStep::TaskFieldConfig => SetupStepInfo {
                name: "Task Field Config",
                description: "Configure optional fields for TASK issue type",
                help_text:
                    "TASK is the foundational issue type. Configure which optional fields to include:\n\
                    - **priority**: Priority level (P0-critical to P3-low)\n\
                    - **points**: Story points estimate\n\
                    - **user_story**: User story or background context\n\n\
                    These choices propagate to other issue types. The 'summary' field is always required, \
                    and 'id' is auto-generated.",
                navigation: "↑/↓ or j/k to navigate, Space to toggle, Enter to continue, Esc to go back",
            },
            SetupStep::SessionWrapperChoice => SetupStepInfo {
                name: "Session Wrapper Choice",
                description: "Select which session wrapper to use for launching coding agents",
                help_text: "Choose how Operator will manage coding agent sessions:\n\
                    - **tmux**: Terminal multiplexer, recommended for most setups\n\
                    - **VS Code**: Launch agents as VS Code tasks (requires extension)\n\
                    - **cmux**: Native macOS terminal for AI agents, organized into windows and workspaces\n\
                    - **Zellij**: Modern terminal workspace with built-in layouts\n\n\
                    Your choice determines which setup steps follow.",
                navigation: "↑/↓ or j/k to navigate, Enter to select, Esc to go back",
            },
            SetupStep::ExecutionTarget => SetupStepInfo {
                name: "Execution Target",
                description: "Choose whether agents run locally or in Coder workspaces",
                help_text: "Local runs agent commands on the same machine as Operator. Coder creates or starts a per-ticket workspace and launches there over SSH.\n\nCoder configuration stores only environment variable names for the deployment URL and session token. Secret values remain in the process environment.\n\nCoder targets disable git worktrees and relay injection, and cannot be combined with Zellij.",
                navigation: "↑/↓ to select, Tab to switch fields, Enter to continue, Esc to go back",
            },
            SetupStep::WorktreePreference => SetupStepInfo {
                name: "Worktree Preference",
                description: "Choose whether to use git worktrees for ticket isolation",
                help_text: "Configure how Operator manages git branches per ticket:\n\
                    - **In-place branches**: Each agent works in the main checkout, switching branches\n\
                    - **Git worktrees**: Each ticket gets its own worktree directory for full isolation\n\n\
                    Worktrees allow multiple agents to work on different tickets simultaneously \
                    without branch conflicts.",
                navigation: "↑/↓ or j/k to navigate, Enter to select, Esc to go back",
            },
            SetupStep::AdminPassword => SetupStepInfo {
                name: "Web UI Password",
                description: "Optionally set the admin password for the web dashboard",
                help_text: "Operator has a single human account, `admin`.\n\n\
                    This terminal and the CLI need no password: a loopback process             authenticates with an owner-only token file in the state directory.             A browser cannot read that file, so the web dashboard stays locked             until an admin password exists.\n\n\
                    Leave both fields blank to skip. You can set one later with             `operator auth bootstrap` or from the /setup page.\n\n\
                    The password must be at least 12 characters. This step is hidden             when an admin account already exists.",
                navigation: "Tab to switch fields, Enter to continue (blank to skip), Esc to go back",
            },
            SetupStep::TmuxOnboarding => SetupStepInfo {
                name: "Tmux Onboarding",
                description:
                    "Help and documentation about tmux session management (shown if tmux selected)",
                help_text: "Operator launches Coding agents in tmux sessions. Essential commands:\n\
                    - **Detach from session**: Ctrl+a (quick, no prefix needed!)\n\
                    - **Fallback detach**: Ctrl+b then d\n\
                    - **List sessions**: `tmux ls`\n\
                    - **Attach to session**: `tmux attach -t <name>`\n\n\
                    Operator session names start with 'op-' for easy identification.",
                navigation: "Enter to continue, Esc to go back",
            },
            SetupStep::VSCodeSetup => SetupStepInfo {
                name: "VS Code Setup",
                description: "VS Code extension setup and verification (shown if VS Code selected)",
                help_text: "Operator integrates with the VS Code extension to launch agents as tasks.\n\
                    This step verifies the extension is installed and the webhook server is reachable.\n\n\
                    Install the extension from the VS Code marketplace if prompted.",
                navigation: "Enter to continue, Esc to go back",
            },
            SetupStep::CmuxSetup => SetupStepInfo {
                name: "Cmux Setup",
                description: "cmux session wrapper setup (shown if cmux selected)",
                help_text: "cmux is a native macOS terminal that organizes AI agent sessions into \
                    windows and workspaces.\n\n\
                    This step verifies the cmux app's CLI binary exists at the configured \
                    binary_path (by default inside /Applications/cmux.app) and meets the \
                    minimum supported version.",
                navigation: "Enter to continue, Esc to go back",
            },
            SetupStep::ZellijSetup => SetupStepInfo {
                name: "Zellij Setup",
                description: "Zellij session wrapper setup (shown if Zellij selected)",
                help_text:
                    "Zellij is a modern terminal workspace with built-in layouts and multiplexing.\n\n\
                    This step verifies Zellij is installed and configures the layout Operator will use \
                    when launching agents.",
                navigation: "Enter to continue, Esc to go back",
            },
            SetupStep::AcceptanceCriteria => SetupStepInfo {
                name: "Acceptance Criteria",
                description: "Review and configure acceptance criteria for ticket completion",
                help_text: "Define what 'done' means for tickets in this workspace.\n\
                    Acceptance criteria are checked by agents before marking a ticket complete.\n\n\
                    The default criteria cover formatting, tests, and lint checks. \
                    You can customize them for your team's standards.",
                navigation: "Enter to continue, Esc to go back",
            },
            SetupStep::StartupTickets => SetupStepInfo {
                name: "Startup Tickets",
                description: "Optionally create tickets to bootstrap your projects",
                help_text: "Create startup tickets to help initialize your projects:\n\
                    - **ASSESS tickets**: Scan projects for catalog-info.yaml, create if missing\n\
                    - **AGENT_SETUP tickets**: Configure Claude agents for each project\n\
                    - **PROJECT_INIT tickets**: Run both ASSESS and AGENT_SETUP for each project\n\n\
                    These tickets are optional and help automate common setup tasks.",
                navigation: "↑/↓ or j/k to navigate, Space to toggle, Enter to continue, Esc to go back",
            },
            SetupStep::Confirm => SetupStepInfo {
                name: "Confirm",
                description: "Review settings and confirm initialization",
                help_text: "Review your configuration before initialization:\n\
                    - Path where `.tickets/` will be created\n\
                    - Selected issue types and preset name\n\
                    - Directories that will be created: queue/, in-progress/, completed/, templates/\n\n\
                    Choose Initialize to create the ticket queue, or Cancel to exit without changes.",
                navigation: "Tab or Space to toggle selection, Enter to confirm, Esc to go back",
            },
        }
    }
}

/// The full catalog in wizard order, for documentation generation.
#[allow(dead_code)] // Used via binary and docs_gen, not reachable from lib.rs
pub fn setup_steps() -> Vec<SetupStepInfo> {
    SetupStep::ALL.iter().map(|s| s.info()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_every_step_info_field_is_non_empty() {
        for step in SetupStep::ALL {
            let info = step.info();
            assert!(!info.name.is_empty(), "{step:?}: empty name");
            assert!(!info.description.is_empty(), "{step:?}: empty description");
            assert!(!info.help_text.is_empty(), "{step:?}: empty help_text");
            assert!(!info.navigation.is_empty(), "{step:?}: empty navigation");
        }
    }

    #[test]
    fn test_all_contains_no_duplicates() {
        let unique: HashSet<_> = SetupStep::ALL.iter().collect();
        assert_eq!(unique.len(), SetupStep::ALL.len());
    }

    #[test]
    fn test_slugs_are_unique() {
        let unique: HashSet<_> = SetupStep::ALL.iter().map(|s| s.slug()).collect();
        assert_eq!(unique.len(), SetupStep::ALL.len());
    }

    #[test]
    fn test_step_names_are_unique() {
        let unique: HashSet<_> = SetupStep::ALL.iter().map(|s| s.info().name).collect();
        assert_eq!(unique.len(), SetupStep::ALL.len());
    }

    /// Slugs key docs URLs and the web renderer's component map, so a rename is
    /// a breaking change and must be deliberate.
    #[test]
    fn test_slugs_match_frozen_snapshot() {
        let slugs: Vec<&str> = SetupStep::ALL.iter().map(|s| s.slug()).collect();
        assert_eq!(
            slugs,
            vec![
                "welcome",
                "kanban-info",
                "model-server",
                "git-provider",
                "collection-source",
                "hosted-collections",
                "task-field-config",
                "session-wrapper-choice",
                "execution-target",
                "worktree-preference",
                "admin-password",
                "tmux-onboarding",
                "vscode-setup",
                "cmux-setup",
                "zellij-setup",
                "acceptance-criteria",
                "startup-tickets",
                "confirm",
            ]
        );
    }

    #[test]
    fn test_serde_representation_matches_slug() {
        for step in SetupStep::ALL {
            let json = serde_json::to_string(&step).unwrap();
            assert_eq!(json, format!("\"{}\"", step.slug()));
        }
    }

    #[test]
    fn test_setup_steps_covers_every_variant() {
        assert_eq!(setup_steps().len(), SetupStep::ALL.len());
    }
}

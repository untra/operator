//! Type definitions for the setup wizard

use crate::api::providers::kanban::{DetectedKanbanProvider, KanbanProviderType};
use crate::config::{CollectionPreset, SessionWrapperType};

/// Simplified tool info for display on the welcome screen
#[derive(Debug, Clone)]
pub struct DetectedToolInfo {
    pub name: String,
    pub version: String,
    pub model_count: usize,
}

/// Optional fields that can be configured for TASK (and propagated to other types)
/// Note: 'summary' and 'description' remain required, 'id' is auto-generated
pub const TASK_OPTIONAL_FIELDS: &[(&str, &str)] = &[
    ("priority", "Priority level (P0-critical to P3-low)"),
    ("points", "Story points estimate (0 or greater)"),
    ("user_story", "User story or background context"),
];

/// A configured kanban provider that issuetypes can be imported from.
///
/// Built from a provider detected during setup. The author attribution on a
/// collection imported from this provider lists the provider name + workspace
/// (and, at import time, the chosen project/team).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportProviderRef {
    /// Which kanban provider (Jira / Linear / GitHub).
    pub provider: KanbanProviderType,
    /// Domain / workspace slug / owner login (the provider's workspace key).
    pub workspace_key: String,
    /// Base URL of the provider instance (for author attribution + display).
    pub base_url: String,
}

impl ImportProviderRef {
    /// Author attribution for a collection imported from this provider, before a
    /// specific project/team is chosen (e.g. `Jira Cloud (acme.atlassian.net)`).
    pub fn author_attribution(&self) -> String {
        format!("{} ({})", self.provider.display_name(), self.workspace_key)
    }
}

/// Collection source options shown in setup.
///
/// The curated options ([`Simple`](Self::Simple), [`DevKanban`](Self::DevKanban),
/// [`DevopsKanban`](Self::DevopsKanban)) ship with operator. [`Browse`](Self::Browse)
/// opens the hosted-manifest picker. [`ImportFromProvider`](Self::ImportFromProvider)
/// entries are generated dynamically, one per configured kanban provider.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CollectionSourceOption {
    Simple,
    DevKanban,
    DevopsKanban,
    /// Browse the operator-hosted manifest (multi-select collections).
    Browse,
    /// Import issuetypes from a configured kanban provider.
    ImportFromProvider(ImportProviderRef),
}

impl CollectionSourceOption {
    /// The curated, install-bundled options plus the hosted browser, in display
    /// order. Per-provider import options are appended by [`with_providers`](Self::with_providers).
    pub fn curated() -> Vec<CollectionSourceOption> {
        vec![
            CollectionSourceOption::Simple,
            CollectionSourceOption::DevKanban,
            CollectionSourceOption::DevopsKanban,
            CollectionSourceOption::Browse,
        ]
    }

    /// Build the full option list: the curated options followed by one import
    /// option per configured kanban provider. `providers` is the set detected
    /// during setup; a provider missing required env vars is skipped, so no
    /// import options appear on a fresh install with nothing configured.
    pub fn with_providers(providers: &[DetectedKanbanProvider]) -> Vec<CollectionSourceOption> {
        let mut options = Self::curated();
        for p in providers {
            if !p.has_required_env_vars() {
                continue;
            }
            options.push(CollectionSourceOption::ImportFromProvider(
                ImportProviderRef {
                    provider: p.provider_type,
                    workspace_key: p.domain.clone(),
                    base_url: provider_base_url(p),
                },
            ));
        }
        options
    }

    pub fn label(&self) -> String {
        match self {
            CollectionSourceOption::Simple => "Simple".to_string(),
            CollectionSourceOption::DevKanban => "Dev Kanban".to_string(),
            CollectionSourceOption::DevopsKanban => "DevOps Kanban".to_string(),
            CollectionSourceOption::Browse => "Browse Hosted Collections".to_string(),
            CollectionSourceOption::ImportFromProvider(r) => {
                format!(
                    "Import from {} ({})",
                    r.provider.display_name(),
                    r.workspace_key
                )
            }
        }
    }

    pub fn description(&self) -> String {
        match self {
            CollectionSourceOption::Simple => {
                "Just TASK - minimal setup for general work".to_string()
            }
            CollectionSourceOption::DevKanban => "3 issue types: TASK, FEAT, FIX".to_string(),
            CollectionSourceOption::DevopsKanban => {
                "5 issue types: TASK, SPIKE, INV, FEAT, FIX".to_string()
            }
            CollectionSourceOption::Browse => {
                "Pick curated collections from operator.untra.io".to_string()
            }
            CollectionSourceOption::ImportFromProvider(r) => {
                format!("Import issuetypes from {}", r.base_url)
            }
        }
    }
}

/// Derive a base URL for author attribution from a detected provider.
fn provider_base_url(p: &DetectedKanbanProvider) -> String {
    match p.provider_type {
        KanbanProviderType::Jira => {
            if p.domain.is_empty() {
                p.provider_type.setup_url().to_string()
            } else if p.domain.starts_with("http") {
                p.domain.clone()
            } else {
                format!("https://{}", p.domain)
            }
        }
        KanbanProviderType::Linear => "https://linear.app".to_string(),
        KanbanProviderType::Github => "https://github.com".to_string(),
    }
}

/// Result of setup screen actions
#[derive(Debug, Clone)]
pub enum SetupResult {
    /// Continue to next step
    Continue,
    /// Cancel/quit setup
    Cancel,
    /// Setup complete, initialize
    Initialize,
}

/// Startup ticket options for project initialization
#[derive(Debug, Clone)]
pub struct StartupTicketOption {
    /// Key identifier for the ticket type (e.g., "assess", "`agent_setup`")
    pub key: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub enabled: bool,
}

impl StartupTicketOption {
    pub fn all() -> Vec<StartupTicketOption> {
        vec![
            StartupTicketOption {
                key: "assess",
                name: "ASSESS tickets",
                description: "Scan projects for catalog-info.yaml, create if missing",
                enabled: true,
            },
            StartupTicketOption {
                key: "agent_setup",
                name: "AGENT-SETUP tickets",
                description: "Configure Claude agents for each project",
                enabled: false,
            },
            StartupTicketOption {
                key: "project_init",
                name: "PROJECT-INIT tickets",
                description: "Run both ASSESS and AGENT-SETUP for each project",
                enabled: false,
            },
        ]
    }
}

/// Tmux availability detection status
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum TmuxDetectionStatus {
    /// Not yet checked
    #[default]
    NotChecked,
    /// Tmux is available with the given version
    Available { version: String },
    /// Tmux is not installed
    NotInstalled,
    /// Tmux is installed but version is too old
    VersionTooOld { current: String, required: String },
}

/// VS Code extension detection status
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[allow(dead_code)] // Placeholder for VS Code extension detection (Phase 2)
pub enum VSCodeDetectionStatus {
    /// Not yet checked
    #[default]
    NotChecked,
    /// Currently checking connection
    Checking,
    /// Connected to extension with the given version
    Connected { version: String },
    /// Extension not reachable
    NotReachable,
}

/// Session wrapper options shown in setup
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionWrapperOption {
    Tmux,
    VSCode,
    Cmux,
    Zellij,
}

impl SessionWrapperOption {
    pub fn all() -> &'static [SessionWrapperOption] {
        &[
            SessionWrapperOption::Tmux,
            SessionWrapperOption::VSCode,
            SessionWrapperOption::Cmux,
            SessionWrapperOption::Zellij,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            SessionWrapperOption::Tmux => "Tmux (default)",
            SessionWrapperOption::VSCode => "VS Code Integrated Terminal",
            SessionWrapperOption::Cmux => "cmux (macOS terminal multiplexer)",
            SessionWrapperOption::Zellij => "Zellij (terminal workspace manager)",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            SessionWrapperOption::Tmux => "Run agents in standalone tmux sessions",
            SessionWrapperOption::VSCode => {
                "Run agents in VS Code terminal panels (requires extension)"
            }
            SessionWrapperOption::Cmux => {
                "Run agents in cmux workspaces (requires running inside cmux, macOS only)"
            }
            SessionWrapperOption::Zellij => {
                "Run agents in Zellij panes (requires running inside Zellij)"
            }
        }
    }

    pub fn to_wrapper_type(self) -> SessionWrapperType {
        match self {
            SessionWrapperOption::Tmux => SessionWrapperType::Tmux,
            SessionWrapperOption::VSCode => SessionWrapperType::Vscode,
            SessionWrapperOption::Cmux => SessionWrapperType::Cmux,
            SessionWrapperOption::Zellij => SessionWrapperType::Zellij,
        }
    }
}

/// Git workflow options shown in setup
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorktreeOption {
    /// Work directly in the project directory with feature branches (default)
    InPlace,
    /// Use isolated git worktrees for parallel development
    Worktrees,
}

impl WorktreeOption {
    pub fn all() -> &'static [WorktreeOption] {
        &[WorktreeOption::InPlace, WorktreeOption::Worktrees]
    }

    pub fn label(&self) -> &'static str {
        match self {
            WorktreeOption::InPlace => "Work in project directory (recommended)",
            WorktreeOption::Worktrees => "Use isolated worktrees",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            WorktreeOption::InPlace => {
                "Creates feature branches directly in the project. Simpler setup, no trust dialogs."
            }
            WorktreeOption::Worktrees => {
                "Creates isolated worktrees per ticket. Enables parallel work but requires trust approval."
            }
        }
    }

    /// Convert to the config boolean value
    pub fn to_use_worktrees(self) -> bool {
        match self {
            WorktreeOption::InPlace => false,
            WorktreeOption::Worktrees => true,
        }
    }

    /// Create from config boolean value
    #[allow(dead_code)] // Useful for future config-to-UI state conversion
    pub fn from_use_worktrees(use_worktrees: bool) -> Self {
        if use_worktrees {
            WorktreeOption::Worktrees
        } else {
            WorktreeOption::InPlace
        }
    }
}

/// Steps in the setup process
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SetupStep {
    /// Welcome splash screen with discovered projects
    Welcome,
    /// Select template collection source
    CollectionSource,
    /// Browse and multi-select hosted collections (fetched from the manifest URL)
    HostedCollectionFetch,
    /// Configure TASK optional fields
    TaskFieldConfig,
    /// Select session wrapper (tmux or vscode)
    SessionWrapperChoice,
    /// Git worktree preference (use worktrees vs in-place branches)
    WorktreePreference,
    /// Tmux onboarding/help (only shown if tmux selected)
    TmuxOnboarding,
    /// VS Code extension setup (only shown if vscode selected)
    VSCodeSetup,
    /// cmux setup (only shown if cmux selected)
    CmuxSetup,
    /// Zellij setup (only shown if zellij selected)
    ZellijSetup,
    /// Kanban integration info and provider detection
    KanbanInfo,
    /// Per-provider setup with project selection (index into `valid_providers`)
    KanbanProviderSetup { provider_index: usize },
    /// Review and configure acceptance criteria
    AcceptanceCriteria,
    /// Optional startup tickets creation
    StartupTickets,
    /// Confirm initialization
    Confirm,
}

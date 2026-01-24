//! Type definitions for the setup wizard

use crate::config::{CollectionPreset, SessionWrapperType};

/// Simplified tool info for display on the welcome screen
#[derive(Debug, Clone)]
pub struct DetectedToolInfo {
    pub name: String,
    pub version: String,
    pub model_count: usize,
}

/// Available issuetype collections (all known types)
pub const ALL_ISSUE_TYPES: &[&str] = &["TASK", "FEAT", "FIX", "SPIKE", "INV"];

/// Optional fields that can be configured for TASK (and propagated to other types)
/// Note: 'summary' and 'description' remain required, 'id' is auto-generated
pub const TASK_OPTIONAL_FIELDS: &[(&str, &str)] = &[
    ("priority", "Priority level (P0-critical to P3-low)"),
    ("points", "Story points estimate (0 or greater)"),
    ("user_story", "User story or background context"),
];

/// Collection source options shown in setup
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollectionSourceOption {
    Simple,
    DevKanban,
    DevopsKanban,
    ImportJira,
    ImportNotion,
    CustomSelection,
}

impl CollectionSourceOption {
    pub fn all() -> &'static [CollectionSourceOption] {
        &[
            CollectionSourceOption::Simple,
            CollectionSourceOption::DevKanban,
            CollectionSourceOption::DevopsKanban,
            CollectionSourceOption::ImportJira,
            CollectionSourceOption::ImportNotion,
            CollectionSourceOption::CustomSelection,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            CollectionSourceOption::Simple => "Simple",
            CollectionSourceOption::DevKanban => "Dev Kanban",
            CollectionSourceOption::DevopsKanban => "DevOps Kanban",
            CollectionSourceOption::ImportJira => "Import from Jira",
            CollectionSourceOption::ImportNotion => "Import from Notion",
            CollectionSourceOption::CustomSelection => "Custom Selection",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            CollectionSourceOption::Simple => "Just TASK - minimal setup for general work",
            CollectionSourceOption::DevKanban => "3 issue types: TASK, FEAT, FIX",
            CollectionSourceOption::DevopsKanban => "5 issue types: TASK, SPIKE, INV, FEAT, FIX",
            CollectionSourceOption::ImportJira => "(Coming soon)",
            CollectionSourceOption::ImportNotion => "(Coming soon)",
            CollectionSourceOption::CustomSelection => "Choose individual issue types",
        }
    }

    pub fn is_unimplemented(&self) -> bool {
        matches!(
            self,
            CollectionSourceOption::ImportJira | CollectionSourceOption::ImportNotion
        )
    }
}

/// Result of setup screen actions
#[derive(Debug, Clone)]
pub enum SetupResult {
    /// Continue to next step
    Continue,
    /// Cancel/quit setup
    Cancel,
    /// Exit with unimplemented message
    ExitUnimplemented(String),
    /// Setup complete, initialize
    Initialize,
}

/// Startup ticket options for project initialization
#[derive(Debug, Clone)]
pub struct StartupTicketOption {
    /// Key identifier for the ticket type (e.g., "assess", "agent_setup")
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TmuxDetectionStatus {
    /// Not yet checked
    NotChecked,
    /// Tmux is available with the given version
    Available { version: String },
    /// Tmux is not installed
    NotInstalled,
    /// Tmux is installed but version is too old
    VersionTooOld { current: String, required: String },
}

impl Default for TmuxDetectionStatus {
    fn default() -> Self {
        Self::NotChecked
    }
}

/// VS Code extension detection status
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)] // Variants will be used when VS Code extension support is implemented
pub enum VSCodeDetectionStatus {
    /// Not yet checked
    NotChecked,
    /// Currently checking connection
    Checking,
    /// Connected to extension with the given version
    Connected { version: String },
    /// Extension not reachable
    NotReachable,
}

impl Default for VSCodeDetectionStatus {
    fn default() -> Self {
        Self::NotChecked
    }
}

/// Session wrapper options shown in setup
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionWrapperOption {
    Tmux,
    VSCode,
}

impl SessionWrapperOption {
    pub fn all() -> &'static [SessionWrapperOption] {
        &[SessionWrapperOption::Tmux, SessionWrapperOption::VSCode]
    }

    pub fn label(&self) -> &'static str {
        match self {
            SessionWrapperOption::Tmux => "Tmux (default)",
            SessionWrapperOption::VSCode => "VS Code Integrated Terminal",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            SessionWrapperOption::Tmux => "Run agents in standalone tmux sessions",
            SessionWrapperOption::VSCode => {
                "Run agents in VS Code terminal panels (requires extension)"
            }
        }
    }

    pub fn to_wrapper_type(self) -> SessionWrapperType {
        match self {
            SessionWrapperOption::Tmux => SessionWrapperType::Tmux,
            SessionWrapperOption::VSCode => SessionWrapperType::Vscode,
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
    /// Custom issuetype selection (optional)
    CustomCollection,
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
    /// Kanban integration info and provider detection
    KanbanInfo,
    /// Per-provider setup with project selection (index into valid_providers)
    KanbanProviderSetup { provider_index: usize },
    /// Review and configure acceptance criteria
    AcceptanceCriteria,
    /// Optional startup tickets creation
    StartupTickets,
    /// Confirm initialization
    Confirm,
}

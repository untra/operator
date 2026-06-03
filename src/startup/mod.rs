//! Setup wizard step registry and template initialization.
//!
//! This module defines the setup wizard steps that appear during first-time
//! initialization when no `.tickets/` directory exists. These definitions
//! serve as the source-of-truth for auto-generated documentation.
//!
//! It also provides template initialization functions to copy embedded
//! template files to the filesystem.
//!
//! ## Usage
//!
//! ```rust,ignore
//! use crate::startup::{SETUP_STEPS, SetupStepInfo};
//!
//! for step in SETUP_STEPS {
//!     println!("{}: {}", step.name, step.description);
//! }
//!
//! // Initialize default templates
//! use crate::startup::templates::init_default_templates;
//! init_default_templates(&templates_path)?;
//! ```

pub mod templates;

/// Information about a setup wizard step for documentation purposes.
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

/// All setup wizard steps in order.
///
/// These steps correspond to the `SetupStep` enum in `src/ui/setup/types.rs`.
/// Steps follow a progressive disclosure model: config/welcome first, then
/// connections (session wrapper + git), then kanban providers, then issue type
/// selection, and finally confirmation.
///
/// When adding new steps to the setup wizard, add corresponding entries here.
#[allow(dead_code)] // Used via binary and docs_gen, not reachable from lib.rs
pub static SETUP_STEPS: &[SetupStepInfo] = &[
    // ── Tier 0: Config / Welcome ─────────────────────────────────────────────
    SetupStepInfo {
        name: "Welcome",
        description: "Splash screen showing detected LLM tools and discovered projects",
        help_text: "The welcome screen displays:\n\
            - Detected LLM tools (Claude, Gemini, Codex, etc.) with version and model count\n\
            - Discovered projects organized by which LLM tool marker files they contain\n\
            - The path where the tickets directory will be created\n\n\
            This gives you an overview of your development environment before proceeding.",
        navigation: "Enter to continue, Esc to cancel",
    },
    // ── Tier 1: Connections (session wrapper + git) ──────────────────────────
    SetupStepInfo {
        name: "Session Wrapper Choice",
        description: "Select which session wrapper to use for launching coding agents",
        help_text: "Choose how Operator will manage coding agent sessions:\n\
            - **tmux**: Terminal multiplexer, recommended for most setups\n\
            - **VS Code**: Launch agents as VS Code tasks (requires extension)\n\
            - **cmux**: Lightweight tmux wrapper with operator defaults pre-applied\n\
            - **Zellij**: Modern terminal workspace with built-in layouts\n\n\
            Your choice determines which setup steps follow.",
        navigation: "↑/↓ or j/k to navigate, Enter to select, Esc to go back",
    },
    SetupStepInfo {
        name: "Worktree Preference",
        description: "Choose whether to use git worktrees for ticket isolation",
        help_text: "Configure how Operator manages git branches per ticket:\n\
            - **In-place branches**: Each agent works in the main checkout, switching branches\n\
            - **Git worktrees**: Each ticket gets its own worktree directory for full isolation\n\n\
            Worktrees allow multiple agents to work on different tickets simultaneously \
            without branch conflicts.",
        navigation: "↑/↓ or j/k to navigate, Enter to select, Esc to go back",
    },
    SetupStepInfo {
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
    SetupStepInfo {
        name: "VS Code Setup",
        description: "VS Code extension setup and verification (shown if VS Code selected)",
        help_text: "Operator integrates with the VS Code extension to launch agents as tasks.\n\
            This step verifies the extension is installed and the webhook server is reachable.\n\n\
            Install the extension from the VS Code marketplace if prompted.",
        navigation: "Enter to continue, Esc to go back",
    },
    SetupStepInfo {
        name: "Cmux Setup",
        description: "cmux session wrapper setup (shown if cmux selected)",
        help_text: "cmux is a lightweight tmux wrapper that pre-applies Operator's preferred \
            session defaults.\n\n\
            This step verifies cmux is installed and accessible in your PATH.",
        navigation: "Enter to continue, Esc to go back",
    },
    SetupStepInfo {
        name: "Zellij Setup",
        description: "Zellij session wrapper setup (shown if Zellij selected)",
        help_text:
            "Zellij is a modern terminal workspace with built-in layouts and multiplexing.\n\n\
            This step verifies Zellij is installed and configures the layout Operator will use \
            when launching agents.",
        navigation: "Enter to continue, Esc to go back",
    },
    // ── Tier 2: Kanban providers ─────────────────────────────────────────────
    SetupStepInfo {
        name: "Kanban Info",
        description: "Kanban integration overview and provider credential detection",
        help_text:
            "Operator can sync with external kanban providers to pull in issues as tickets.\n\
            Supported providers: Jira, Linear, GitHub Projects.\n\n\
            Credentials are read from environment variables (e.g. OPERATOR_JIRA_API_KEY). \
            This step shows which providers were detected and validates connectivity.",
        navigation: "Enter to continue, Esc to go back",
    },
    SetupStepInfo {
        name: "Kanban Provider Setup",
        description: "Per-provider credential validation and project selection",
        help_text: "For each detected provider, Operator:\n\
            1. Validates your API credentials against the provider\n\
            2. Fetches your workspace and user information\n\
            3. Discovers available projects for you to select\n\n\
            Only projects you select will be synced to your ticket queue. \
            You can skip this step to configure kanban providers later.",
        navigation:
            "↑/↓ or j/k to navigate, Space to select projects, Enter to confirm, Esc to go back",
    },
    // ── Tier 3: Issue types (configured after kanban providers are connected) ─
    SetupStepInfo {
        name: "Collection Source",
        description: "Choose which issue type collection to use",
        help_text: "Select a preset collection of issue types:\n\
            - **Simple**: Just TASK - minimal setup for general work\n\
            - **Dev Kanban**: 3 types (TASK, FEAT, FIX) for development workflows\n\
            - **DevOps Kanban**: 5 types (TASK, SPIKE, INV, FEAT, FIX) for full DevOps\n\
            - **Custom Selection**: Choose individual issue types",
        navigation: "↑/↓ or j/k to navigate, Enter to select, Esc to go back",
    },
    SetupStepInfo {
        name: "Custom Collection",
        description: "Select individual issue types (only shown if Custom Selection chosen)",
        help_text: "Toggle individual issue types to include:\n\
            - **TASK**: Focused task that executes one specific thing\n\
            - **FEAT**: New feature or enhancement\n\
            - **FIX**: Bug fix, follow-up work, tech debt\n\
            - **SPIKE**: Research or exploration (paired mode)\n\
            - **INV**: Incident investigation (paired mode)\n\n\
            At least one issue type must be selected to proceed.",
        navigation: "↑/↓ or j/k to navigate, Space to toggle, Enter to continue, Esc to go back",
    },
    SetupStepInfo {
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
    // ── Tier 4: Finalize ─────────────────────────────────────────────────────
    SetupStepInfo {
        name: "Acceptance Criteria",
        description: "Review and configure acceptance criteria for ticket completion",
        help_text: "Define what 'done' means for tickets in this workspace.\n\
            Acceptance criteria are checked by agents before marking a ticket complete.\n\n\
            The default criteria cover formatting, tests, and lint checks. \
            You can customize them for your team's standards.",
        navigation: "Enter to continue, Esc to go back",
    },
    SetupStepInfo {
        name: "Startup Tickets",
        description: "Optionally create tickets to bootstrap your projects",
        help_text: "Create startup tickets to help initialize your projects:\n\
            - **ASSESS tickets**: Scan projects for catalog-info.yaml, create if missing\n\
            - **AGENT-SETUP tickets**: Configure Claude agents for each project\n\
            - **PROJECT-INIT tickets**: Run both ASSESS and AGENT-SETUP for each project\n\n\
            These tickets are optional and help automate common setup tasks.",
        navigation: "↑/↓ or j/k to navigate, Space to toggle, Enter to continue, Esc to go back",
    },
    SetupStepInfo {
        name: "Confirm",
        description: "Review settings and confirm initialization",
        help_text: "Review your configuration before initialization:\n\
            - Path where `.tickets/` will be created\n\
            - Selected issue types and preset name\n\
            - Directories that will be created: queue/, in-progress/, completed/, templates/\n\n\
            Choose Initialize to create the ticket queue, or Cancel to exit without changes.",
        navigation: "Tab or Space to toggle selection, Enter to confirm, Esc to go back",
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_setup_steps_not_empty() {
        assert!(!SETUP_STEPS.is_empty());
    }

    #[test]
    fn test_setup_steps_have_required_fields() {
        for step in SETUP_STEPS {
            assert!(!step.name.is_empty(), "Step name should not be empty");
            assert!(
                !step.description.is_empty(),
                "Step description should not be empty"
            );
            assert!(
                !step.help_text.is_empty(),
                "Step help_text should not be empty"
            );
            assert!(
                !step.navigation.is_empty(),
                "Step navigation should not be empty"
            );
        }
    }

    #[test]
    fn test_setup_steps_count_matches_enum() {
        // 15 steps: Welcome, SessionWrapperChoice, WorktreePreference,
        // TmuxOnboarding, VSCodeSetup, CmuxSetup, ZellijSetup,
        // KanbanInfo, KanbanProviderSetup,
        // CollectionSource, CustomCollection, TaskFieldConfig,
        // AcceptanceCriteria, StartupTickets, Confirm
        assert_eq!(SETUP_STEPS.len(), 15);
    }

    #[test]
    fn test_step_names_are_unique() {
        let names: Vec<&str> = SETUP_STEPS.iter().map(|s| s.name).collect();
        let mut unique_names = names.clone();
        unique_names.sort_unstable();
        unique_names.dedup();
        assert_eq!(
            names.len(),
            unique_names.len(),
            "Step names should be unique"
        );
    }
}

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
/// These steps correspond to the `SetupStep` enum in `src/ui/setup.rs`.
/// When adding new steps to the setup wizard, add corresponding entries here.
pub static SETUP_STEPS: &[SetupStepInfo] = &[
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
    SetupStepInfo {
        name: "Collection Source",
        description: "Choose which ticket template collection to use",
        help_text: "Select a preset collection of issue types:\n\
            - **Simple**: Just TASK - minimal setup for general work\n\
            - **Dev Kanban**: 3 types (TASK, FEAT, FIX) for development workflows\n\
            - **DevOps Kanban**: 5 types (TASK, SPIKE, INV, FEAT, FIX) for full DevOps\n\
            - **Import from Jira**: (Coming soon)\n\
            - **Import from Notion**: (Coming soon)\n\
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
            - **context**: Background context for the task\n\n\
            These choices propagate to other issue types. The 'summary' field is always required, \
            and 'id' is auto-generated.",
        navigation: "↑/↓ or j/k to navigate, Space to toggle, Enter to continue, Esc to go back",
    },
    SetupStepInfo {
        name: "Tmux Onboarding",
        description: "Help and documentation about tmux session management",
        help_text: "Operator launches Claude agents in tmux sessions. Essential commands:\n\
            - **Detach from session**: Ctrl+a (quick, no prefix needed!)\n\
            - **Fallback detach**: Ctrl+b then d\n\
            - **List sessions**: `tmux ls`\n\
            - **Attach to session**: `tmux attach -t <name>`\n\n\
            Operator session names start with 'op-' for easy identification.",
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
        // There are 7 steps in SetupStep enum (Welcome, CollectionSource, CustomCollection,
        // TaskFieldConfig, TmuxOnboarding, StartupTickets, Confirm)
        assert_eq!(SETUP_STEPS.len(), 7);
    }

    #[test]
    fn test_step_names_are_unique() {
        let names: Vec<&str> = SETUP_STEPS.iter().map(|s| s.name).collect();
        let mut unique_names = names.clone();
        unique_names.sort();
        unique_names.dedup();
        assert_eq!(
            names.len(),
            unique_names.len(),
            "Step names should be unique"
        );
    }
}

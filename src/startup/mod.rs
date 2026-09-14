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
//! use crate::startup::setup_steps;
//!
//! for step in setup_steps() {
//!     println!("{}: {}", step.name, step.description);
//! }
//!
//! // Initialize default templates
//! use crate::startup::templates::init_default_templates;
//! init_default_templates(&templates_path)?;
//! ```

pub mod recovery;
pub mod steps;
pub mod templates;

/// Whether a workspace has been initialized at `config`'s tickets path.
///
/// One predicate for every surface: the TUI decides whether to show the setup
/// wizard from it, and it keeps "is this set up?" from drifting between them.
#[allow(dead_code)] // Used via binary today; the REST setup surface will share it
pub fn workspace_initialized(config: &crate::config::Config) -> bool {
    config.tickets_path().join("queue").exists()
}

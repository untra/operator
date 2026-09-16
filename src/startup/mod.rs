//! Setup wizard step registry and template initialization.
//!
//! Defines the setup wizard steps that appear during first-time initialization when no `.tickets/` directory exists.
//! These definitions serve as the source-of-truth for auto-generated documentation.
//!
//! It also provides template initialization functions to copy embedded template files to the filesystem.
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

const SETUP_COMPLETE_FILE: &str = "setup-complete";

/// Whether a workspace has been initialized at `config`'s tickets path.
///
/// One predicate for every surface - the TUI wizard, the REST setup routes and
/// `ProfileSummary.initialized` - so "is this set up?" cannot drift between them.
pub fn workspace_initialized(config: &crate::config::Config) -> bool {
    setup_complete_path(config).is_file()
        || (config.profile.id.is_nil() && config.tickets_path().join("queue").exists())
}

fn setup_complete_path(config: &crate::config::Config) -> std::path::PathBuf {
    config
        .tickets_path()
        .join("operator")
        .join(SETUP_COMPLETE_FILE)
}

pub fn mark_workspace_initialized(config: &crate::config::Config) -> std::io::Result<()> {
    let path = setup_complete_path(config);
    std::fs::create_dir_all(path.parent().expect("setup marker has a parent"))?;
    std::fs::write(path, env!("CARGO_PKG_VERSION"))
}

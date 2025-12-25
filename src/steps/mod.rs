//! # Deferred Module: Step Workflow Manager
//!
//! **Status**: Complete implementation, not yet integrated into main application
//!
//! **Purpose**: Multi-step workflow orchestration for complex tickets. Enables breaking
//! down large tickets into sequential steps that can be executed, monitored, and resumed.
//!
//! **Integration Point**: `agents/launcher.rs` - wrap ticket launches in StepSession
//!
//! **Milestone**: TBD - Define when step-based workflows are needed for complex multi-phase work
//!
//! ## Components
//!
//! - [`StepManager`]: Orchestrates step transitions and state management
//! - [`StepSession`]: Persists step progress for resumability
//!
//! ## Usage When Integrated
//!
//! ```rust,ignore
//! use crate::steps::{StepManager, StepSession};
//!
//! let step_manager = StepManager::new(&ticket);
//! for step in step_manager.steps() {
//!     let session = StepSession::new(&step);
//!     launcher.launch_step(&step).await?;
//!     session.mark_complete()?;
//! }
//! ```

#![allow(dead_code)] // DEFERRED: See module docs for integration plan
#![allow(unused_imports)]

pub mod manager;
pub mod session;

pub use manager::StepManager;
pub use session::StepSession;

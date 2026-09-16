#![allow(dead_code)]
#![allow(unused_imports)]

//! Provider trait definitions for external service integrations. Defines the trait interfaces for different provider categories:

pub mod ai;
pub mod kanban;
pub mod model_server;
pub mod repo;

// Re-export commonly used types
pub use ai::{AiProvider, RateLimitInfo};
pub use kanban::{ExternalIssueType, KanbanProvider, ProjectInfo};
pub use model_server::{ModelInfo, ModelServerKind, ProbeOutcome};
pub use repo::{PrStatus, RepoProvider};

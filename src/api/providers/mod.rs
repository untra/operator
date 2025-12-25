#![allow(dead_code)]
#![allow(unused_imports)]

//! Provider trait definitions for external service integrations
//!
//! This module defines the trait interfaces for different provider categories:
//! - AI providers (Anthropic, OpenAI, Gemini)
//! - Repository providers (GitHub, GitLab, Azure Repos)
//! - Kanban providers (Jira, Linear) for importing issue types

pub mod ai;
pub mod kanban;
pub mod repo;

// Re-export commonly used types
pub use ai::{AiProvider, RateLimitInfo};
pub use kanban::{ExternalIssueType, KanbanProvider, ProjectInfo};
pub use repo::{PrStatus, RepoProvider};

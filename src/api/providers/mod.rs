//! Provider trait definitions for external service integrations
//!
//! This module defines the trait interfaces for different provider categories:
//! - AI providers (Anthropic, OpenAI, Gemini)
//! - Repository providers (GitHub, GitLab, Azure Repos)
//! - Project management providers (Jira, Notion, GitHub Projects)

pub mod ai;
pub mod repo;

// Re-export commonly used types
pub use ai::{AiProvider, RateLimitInfo};
pub use repo::{PrStatus, RepoProvider};

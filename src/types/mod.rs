//! Unified types for Operator - compatible with vibe-kanban type generation patterns.
//!
//! These types use ts-rs and schemars to generate TypeScript definitions
//! and JSON schemas from a single Rust source of truth.

#![allow(dead_code)] // Types are for generation and future integration
#![allow(unused_imports)] // Re-exports for generate_types binary

mod attempt;
pub mod llm_stats;
pub mod pr;
mod project;

pub use attempt::{
    AttemptStatus, ExecutionProcess, ProcessStatus, RunReason, Session, StepAttempt,
};
pub use llm_stats::{LlmModelUsage, LlmToolUsage, ProjectLlmStats};
pub use pr::{
    CreatePrError, CreatePrRequest, GitHubRepoError, GitHubRepoInfo, GitProvider, PrReviewState,
    PrState, PullRequestInfo, RepoInfo, RepoInfoError, UnifiedPrComment,
};
pub use project::{Project, ProjectRepo};

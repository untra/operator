//! Types for VSCode extension webhook server API.
//!
//! These types define the HTTP API contract between Operator and the VSCode extension
//! webhook server. They are exported to TypeScript via ts-rs to ensure type safety
//! on both sides.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use ts_rs::TS;

/// Session discovery file written to .tickets/operator/vscode-session.json
/// Used by Operator to discover the extension's webhook server.
#[derive(Debug, Clone, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct VsCodeSessionInfo {
    /// Wrapper type identifier (always "vscode")
    pub wrapper: String,
    /// Actual port the webhook server is listening on
    pub port: u16,
    /// Process ID of VS Code
    pub pid: u32,
    /// Extension version
    pub version: String,
    /// ISO timestamp when server started
    pub started_at: String,
    /// Workspace folder path
    pub workspace: String,
}

/// Health check response from webhook server
#[derive(Debug, Clone, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export)]
pub struct VsCodeHealthResponse {
    /// Health status (always "ok" when healthy)
    pub status: String,
    /// Extension version
    pub version: String,
    /// Port the server is listening on
    pub port: u16,
}

/// Terminal activity state
#[derive(Debug, Clone, Serialize, Deserialize, TS, JsonSchema, PartialEq, Eq)]
#[ts(export)]
#[serde(rename_all = "lowercase")]
pub enum VsCodeActivityState {
    /// Terminal is idle (waiting for input)
    Idle,
    /// Terminal is running a command
    Running,
    /// Activity state cannot be determined
    Unknown,
}

/// State of a managed terminal
#[derive(Debug, Clone, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct VsCodeTerminalState {
    /// Terminal name
    pub name: String,
    /// Process ID if available
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub pid: Option<u32>,
    /// Current activity state
    pub activity: VsCodeActivityState,
    /// Unix timestamp when terminal was created (milliseconds)
    pub created_at: f64,
}

/// Options for creating a new terminal
#[derive(Debug, Clone, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct VsCodeTerminalCreateOptions {
    /// Terminal name (e.g., "op-FEAT-123")
    pub name: String,
    /// Working directory for the terminal
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub working_dir: Option<String>,
    /// Environment variables to set
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub env: Option<HashMap<String, String>>,
}

/// Request to send a command to a terminal
#[derive(Debug, Clone, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export)]
pub struct VsCodeSendCommandRequest {
    /// Command to execute
    pub command: String,
}

/// Generic success response
#[derive(Debug, Clone, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export)]
pub struct VsCodeSuccessResponse {
    /// Whether the operation succeeded
    pub success: bool,
    /// Optional terminal name (for create operations)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub name: Option<String>,
}

/// Terminal exists check response
#[derive(Debug, Clone, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export)]
pub struct VsCodeExistsResponse {
    /// Whether the terminal exists
    pub exists: bool,
}

/// Activity query response
#[derive(Debug, Clone, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export)]
pub struct VsCodeActivityResponse {
    /// Current activity state
    pub activity: VsCodeActivityState,
}

/// Terminal list response
#[derive(Debug, Clone, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export)]
pub struct VsCodeListResponse {
    /// List of managed terminals
    pub terminals: Vec<VsCodeTerminalState>,
}

/// Error response from webhook server
#[derive(Debug, Clone, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export)]
pub struct VsCodeErrorResponse {
    /// Error message
    pub error: String,
}

/// Ticket status in the workflow
#[derive(Debug, Clone, Serialize, Deserialize, TS, JsonSchema, PartialEq, Eq)]
#[ts(export)]
#[serde(rename_all = "kebab-case")]
pub enum VsCodeTicketStatus {
    InProgress,
    Queue,
    Completed,
}

/// Information about a ticket from .tickets directory
#[derive(Debug, Clone, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct VsCodeTicketInfo {
    /// Ticket ID (e.g., "FEAT-123")
    pub id: String,
    /// Ticket title from markdown heading
    pub title: String,
    /// Ticket type key (e.g., "FEAT", "FIX", or any custom type)
    #[serde(rename = "type")]
    #[ts(rename = "type")]
    pub ticket_type: String,
    /// Current status
    pub status: VsCodeTicketStatus,
    /// Path to the ticket markdown file
    pub file_path: String,
    /// Terminal name if in-progress (e.g., "op-FEAT-123")
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub terminal_name: Option<String>,
}

/// Model options for Claude CLI
#[derive(Debug, Clone, Serialize, Deserialize, TS, JsonSchema, PartialEq, Eq)]
#[ts(export)]
#[serde(rename_all = "lowercase")]
pub enum VsCodeModelOption {
    Sonnet,
    Opus,
    Haiku,
}

/// Launch options for starting an agent on a ticket
#[derive(Debug, Clone, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct VsCodeLaunchOptions {
    /// Model to use (sonnet, opus, haiku)
    pub model: VsCodeModelOption,
    /// YOLO mode - auto-accept all prompts
    pub yolo_mode: bool,
    /// Resume from existing session (uses session_id from ticket)
    pub resume_session: bool,
}

/// Parsed ticket metadata from YAML frontmatter
#[derive(Debug, Clone, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct VsCodeTicketMetadata {
    /// Ticket ID
    pub id: String,
    /// Current status
    pub status: String,
    /// Current step name
    pub step: String,
    /// Priority level
    pub priority: String,
    /// Project name
    pub project: String,
    /// Session UUIDs by step name
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub sessions: Option<HashMap<String, String>>,
    /// Git worktree path if using per-ticket worktrees
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub worktree_path: Option<String>,
    /// Git branch name
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub branch: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_activity_state_serialization() {
        assert_eq!(
            serde_json::to_string(&VsCodeActivityState::Idle).unwrap(),
            "\"idle\""
        );
        assert_eq!(
            serde_json::to_string(&VsCodeActivityState::Running).unwrap(),
            "\"running\""
        );
        assert_eq!(
            serde_json::to_string(&VsCodeActivityState::Unknown).unwrap(),
            "\"unknown\""
        );
    }

    #[test]
    fn test_ticket_status_serialization() {
        assert_eq!(
            serde_json::to_string(&VsCodeTicketStatus::InProgress).unwrap(),
            "\"in-progress\""
        );
        assert_eq!(
            serde_json::to_string(&VsCodeTicketStatus::Queue).unwrap(),
            "\"queue\""
        );
    }

    #[test]
    fn test_model_option_serialization() {
        assert_eq!(
            serde_json::to_string(&VsCodeModelOption::Sonnet).unwrap(),
            "\"sonnet\""
        );
        assert_eq!(
            serde_json::to_string(&VsCodeModelOption::Opus).unwrap(),
            "\"opus\""
        );
    }

    #[test]
    fn test_session_info_serialization() {
        let info = VsCodeSessionInfo {
            wrapper: "vscode".to_string(),
            port: 7009,
            pid: 12345,
            version: "0.1.12".to_string(),
            started_at: "2024-01-15T10:30:00Z".to_string(),
            workspace: "/path/to/workspace".to_string(),
        };

        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("\"wrapper\":\"vscode\""));
        assert!(json.contains("\"port\":7009"));
    }
}

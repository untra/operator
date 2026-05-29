//! The shared "export workflow for a ticket" operation.
//!
//! This is the single code path every surface goes through: the CLI and TUI
//! call it directly in-process, and the REST handler calls it after resolving
//! the ticket — so the web UI and VS Code extension reach the same logic over
//! HTTP. Ticket resolution (filesystem/queue I/O) stays at the edges; this
//! function takes an already-resolved ticket plus the issue-type registry.

use anyhow::{anyhow, Result};

use super::export_workflow;
use crate::issuetypes::IssueTypeRegistry;
use crate::pr_config::PrConfig;
use crate::queue::Ticket;

/// The result of exporting a ticket to a Claude dynamic workflow.
#[derive(Debug, Clone)]
pub struct ExportedWorkflow {
    /// The ticket the workflow was generated from.
    pub ticket_id: String,
    /// The issue type key that supplied the step structure.
    pub issuetype_key: String,
    /// A suggested filename for writing the workflow (`<ticket-id>.workflow.js`).
    pub suggested_filename: String,
    /// The generated `.js` workflow source.
    pub contents: String,
}

/// Resolve `ticket`'s issue type from `registry` and render it into a Claude
/// dynamic workflow. Errors if the ticket's type has no matching issue type.
pub fn export_workflow_for_ticket(
    ticket: &Ticket,
    registry: &IssueTypeRegistry,
    pr_config: Option<&PrConfig>,
) -> Result<ExportedWorkflow> {
    let issuetype = registry.get(&ticket.ticket_type).ok_or_else(|| {
        anyhow!(
            "No issue type '{}' registered for ticket {}",
            ticket.ticket_type,
            ticket.id
        )
    })?;

    let contents = export_workflow(ticket, issuetype, pr_config)?;

    Ok(ExportedWorkflow {
        ticket_id: ticket.id.clone(),
        issuetype_key: issuetype.key.clone(),
        suggested_filename: format!("{}.workflow.js", ticket.id),
        contents,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn registry() -> IssueTypeRegistry {
        let mut r = IssueTypeRegistry::new();
        r.load_builtins().expect("load builtins");
        r
    }

    fn ticket(ticket_type: &str) -> Ticket {
        Ticket {
            filename: "20241221-1430-FEAT-gamesvc-test.md".to_string(),
            filepath: "/test/path".to_string(),
            timestamp: "20241221-1430".to_string(),
            ticket_type: ticket_type.to_string(),
            project: "gamesvc".to_string(),
            id: "FEAT-1234".to_string(),
            summary: "Add pagination".to_string(),
            priority: "P2-medium".to_string(),
            status: "queued".to_string(),
            step: String::new(),
            content: "body".to_string(),
            sessions: std::collections::HashMap::new(),
            step_delegators: std::collections::HashMap::new(),
            llm_task: crate::queue::LlmTask::default(),
            worktree_path: None,
            branch: None,
            external_id: None,
            external_url: None,
            external_provider: None,
        }
    }

    #[test]
    fn exports_known_ticket_type_into_workflow() {
        let exported = export_workflow_for_ticket(&ticket("FEAT"), &registry(), None).unwrap();
        assert_eq!(exported.ticket_id, "FEAT-1234");
        assert_eq!(exported.issuetype_key, "FEAT");
        assert_eq!(exported.suggested_filename, "FEAT-1234.workflow.js");
        assert!(
            exported.contents.contains("agent("),
            "workflow body missing:\n{}",
            exported.contents
        );
        assert!(
            exported.contents.contains("FEAT-1234"),
            "ticket id not interpolated"
        );
    }

    #[test]
    fn errors_when_issuetype_unknown() {
        let result = export_workflow_for_ticket(&ticket("NOPE"), &registry(), None);
        assert!(result.is_err(), "unknown issuetype should error");
    }
}

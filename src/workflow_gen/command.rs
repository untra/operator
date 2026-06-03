//! The shared "export workflow for a ticket" operation.
//!
//! This is the single code path every surface goes through: the CLI and TUI
//! call it directly in-process, and the REST handler calls it after resolving
//! the ticket — so the web UI and VS Code extension reach the same logic over
//! HTTP. Ticket resolution (filesystem/queue I/O) stays at the edges; this
//! function takes an already-resolved ticket plus the issue-type registry.

use anyhow::{anyhow, Result};

use super::export::PipelineEnv;
use super::export_workflow;
use crate::config::Config;
use crate::issuetypes::{IssueType, IssueTypeRegistry};
use crate::pr_config::PrConfig;
use crate::queue::{LlmTask, Ticket};
use crate::templates::schema::ItemSource;

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
    config: &Config,
) -> Result<ExportedWorkflow> {
    let issuetype = registry.get(&ticket.ticket_type).ok_or_else(|| {
        anyhow!(
            "No issue type '{}' registered for ticket {}",
            ticket.ticket_type,
            ticket.id
        )
    })?;

    let env = pipeline_env_for(config, ticket, issuetype);
    let contents = export_workflow(ticket, issuetype, pr_config, &env)?;

    Ok(ExportedWorkflow {
        ticket_id: ticket.id.clone(),
        issuetype_key: issuetype.key.clone(),
        suggested_filename: format!("{}.workflow.js", ticket.id),
        contents,
    })
}

/// Render a *preview* workflow for an issue type, without a concrete ticket.
///
/// Used by the UI to visualize an issue type's workflow shape. A placeholder
/// ticket is synthesized so the existing renderer can interpolate handlebars
/// variables — values are illustrative, not real. This is filesystem-safe:
/// `worktree_path: None` short-circuits any step-output loading, and the
/// handlebars renderer runs with strict mode off, so missing variables render
/// as empty strings rather than erroring.
pub fn export_workflow_for_issuetype(issuetype: &IssueType) -> Result<ExportedWorkflow> {
    let ticket = preview_ticket(issuetype);
    // No config/filesystem context in a preview: environment-dependent
    // pipeline item sources (projects/glob) render as symbolic placeholders.
    let contents = export_workflow(&ticket, issuetype, None, &PipelineEnv::default())?;

    Ok(ExportedWorkflow {
        ticket_id: ticket.id,
        issuetype_key: issuetype.key.clone(),
        suggested_filename: format!("{}.preview.workflow.js", issuetype.key),
        contents,
    })
}

/// Build the pipeline-resolution environment for a real ticket export.
///
/// Project discovery (a directory scan) only runs when the issuetype actually
/// contains a `projects` item source; the glob root is a pure path join.
fn pipeline_env_for(config: &Config, ticket: &Ticket, issuetype: &IssueType) -> PipelineEnv {
    let needs_projects = issuetype.steps.iter().any(|s| {
        matches!(
            s.pipeline_config.as_ref().map(|c| &c.item_source),
            Some(ItemSource::Projects)
        )
    });
    PipelineEnv {
        projects: if needs_projects {
            config.discover_projects()
        } else {
            Vec::new()
        },
        glob_root: Some(config.projects_path().join(&ticket.project)),
    }
}

/// Build an illustrative placeholder ticket for an issue type preview.
fn preview_ticket(issuetype: &IssueType) -> Ticket {
    let key = &issuetype.key;
    Ticket {
        filename: format!("{key}-PREVIEW.md"),
        filepath: format!("preview/{key}-PREVIEW.md"),
        timestamp: "preview".to_string(),
        ticket_type: key.clone(),
        project: "preview".to_string(),
        id: format!("{key}-PREVIEW"),
        summary: format!("Sample {}", issuetype.name),
        priority: "P2-medium".to_string(),
        status: "queued".to_string(),
        step: String::new(),
        content: String::new(),
        sessions: std::collections::HashMap::new(),
        step_delegators: std::collections::HashMap::new(),
        llm_task: LlmTask::default(),
        worktree_path: None,
        branch: None,
        external_id: None,
        external_url: None,
        external_provider: None,
    }
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
        let exported =
            export_workflow_for_ticket(&ticket("FEAT"), &registry(), None, &Config::default())
                .unwrap();
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
        let result =
            export_workflow_for_ticket(&ticket("NOPE"), &registry(), None, &Config::default());
        assert!(result.is_err(), "unknown issuetype should error");
    }

    #[test]
    fn issuetype_preview_renders_without_a_ticket() {
        let reg = registry();
        let feat = reg.get("FEAT").expect("FEAT builtin");
        let exported = export_workflow_for_issuetype(feat).unwrap();
        assert_eq!(exported.issuetype_key, "FEAT");
        assert_eq!(exported.suggested_filename, "FEAT.preview.workflow.js");
        assert!(
            exported.contents.contains("export const meta"),
            "preview missing meta:\n{}",
            exported.contents
        );
        assert!(
            exported.contents.contains("await agent("),
            "preview missing agent calls:\n{}",
            exported.contents
        );
        // Compiler-native: no default-export wrapper (would hide the body).
        assert!(
            !exported.contents.contains("export default async function"),
            "preview must not wrap body in a function:\n{}",
            exported.contents
        );
    }

    /// Regression gate: dump every builtin issue type's preview workflow to
    /// `target/workflow-previews/` so the naiveworkflow compiler can be run
    /// against them (see the verification step in the plan / docs). Also
    /// asserts each preview is non-empty.
    #[test]
    fn dumps_all_builtin_previews_for_compiler_check() {
        let reg = registry();
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join("workflow-previews");
        std::fs::create_dir_all(&dir).expect("create preview dir");
        for it in reg.all_types() {
            let exported = export_workflow_for_issuetype(it).unwrap();
            assert!(
                !exported.contents.trim().is_empty(),
                "empty preview for {}",
                it.key
            );
            std::fs::write(dir.join(&exported.suggested_filename), &exported.contents)
                .expect("write preview");
        }
    }
}

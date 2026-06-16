//! The **Workflows** status section: the export formats a ticket + issuetype can
//! be rendered into (Claude dynamic workflow `.js`, AGNT graph `.json`).
//!
//! Info-only — formats carry no credentials, so the section is `Gray`. It gates
//! on `Connections`: preview/export run against the hosted API, so the section
//! only appears once connections are ready. Each row names a format, its support
//! status + file extension, and
//! links to its docs. The primary action opens the web UI's Workflows page where
//! per-issuetype preview / per-ticket export run against the existing endpoints.
//! The format list is the same source of truth the REST `workflow-formats`
//! endpoint and the `Workflows` catalog vertical derive from: [`WorkflowFormat`].

use crate::integrations::{catalog, Vertical};
use crate::ui::status_panel::{
    ActionMeta, ActionSet, SectionHealth, SectionId, StatusAction, StatusIcon, StatusSection,
    StatusSnapshot, TreeRow,
};
use crate::workflow_gen::WorkflowFormat;

pub struct WorkflowsSection;

impl StatusSection for WorkflowsSection {
    fn section_id(&self) -> SectionId {
        SectionId::Workflows
    }

    fn label(&self) -> &'static str {
        "Workflows"
    }

    fn prerequisites(&self) -> &[SectionId] {
        &[SectionId::Connections]
    }

    fn health(&self, _snapshot: &StatusSnapshot) -> SectionHealth {
        SectionHealth::Gray
    }

    fn description(&self, _snapshot: &StatusSnapshot) -> String {
        format!("{} export formats", WorkflowFormat::ALL.len())
    }

    fn children(&self, snapshot: &StatusSnapshot) -> Vec<TreeRow> {
        WorkflowFormat::ALL
            .into_iter()
            .map(|fmt| {
                let entry = catalog::entry_for(Vertical::Workflows, fmt.slug());
                let status = entry.as_ref().map(|e| e.status.label()).unwrap_or("Proto");
                let docs_url = entry.and_then(|e| e.docs_url());

                // Primary opens the web Workflows page (where preview/export run);
                // special links to the format's docs when documented.
                let primary = match snapshot.api_port() {
                    Some(port) => StatusAction::OpenWebUiAt {
                        port,
                        route: "/workflows".into(),
                    },
                    None => StatusAction::None,
                };
                let (special, special_meta) = match docs_url {
                    Some(url) => (
                        StatusAction::OpenUrl(url),
                        Some(ActionMeta {
                            title: "Docs",
                            tooltip: "Open this workflow format's documentation",
                        }),
                    ),
                    None => (StatusAction::None, None),
                };

                TreeRow {
                    section_id: SectionId::Workflows,
                    id: format!("workflow-format-{}", fmt.slug()),
                    depth: 1,
                    label: fmt.label().to_string(),
                    description: format!("{status} · .{}", fmt.extension()),
                    icon: StatusIcon::Tool,
                    brand_icon: None,
                    is_header: false,
                    actions: ActionSet {
                        primary,
                        back: StatusAction::None,
                        special,
                        special_meta,
                        refresh: StatusAction::None,
                        refresh_meta: None,
                    },
                    health: SectionHealth::Gray,
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workflows_section_is_info_only() {
        let snap = StatusSnapshot::from_config(&crate::config::Config::default(), vec![]);
        assert_eq!(WorkflowsSection.health(&snap), SectionHealth::Gray);
        // Gated on Connections: preview/export run against the hosted API.
        assert_eq!(WorkflowsSection.prerequisites(), &[SectionId::Connections]);
    }

    #[test]
    fn workflows_section_lists_every_format() {
        let snap = StatusSnapshot::from_config(&crate::config::Config::default(), vec![]);
        let rows = WorkflowsSection.children(&snap);
        assert_eq!(rows.len(), WorkflowFormat::ALL.len());
        let claude = rows.iter().find(|r| r.label == "Claude Workflow").unwrap();
        assert_eq!(claude.description, "GA · .js");
        // Documented formats expose a Docs link in the special slot.
        assert!(matches!(claude.actions.special, StatusAction::OpenUrl(_)));
    }
}

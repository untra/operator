use crate::ui::status_panel::{
    ActionSet, SectionHealth, SectionId, StatusIcon, StatusSection, StatusSnapshot, TreeRow,
};

/// Issue Types section — mirrors the VS Code extension's `IssueTypeSection`.
/// Visible once Kanban is configured; lists the active issue types.
pub struct IssueTypeSection;

impl StatusSection for IssueTypeSection {
    fn section_id(&self) -> SectionId {
        SectionId::IssueTypes
    }

    fn label(&self) -> &'static str {
        "Issue Types"
    }

    fn prerequisites(&self) -> &[SectionId] {
        &[SectionId::Kanban]
    }

    fn health(&self, snapshot: &StatusSnapshot) -> SectionHealth {
        if snapshot.issue_types.is_empty() {
            SectionHealth::Yellow
        } else {
            SectionHealth::Green
        }
    }

    fn description(&self, snapshot: &StatusSnapshot) -> String {
        let count = snapshot.issue_types.len();
        if count == 0 {
            "Not configured".into()
        } else {
            format!("{count} type{}", if count == 1 { "" } else { "s" })
        }
    }

    fn children(&self, snapshot: &StatusSnapshot) -> Vec<TreeRow> {
        snapshot
            .issue_types
            .iter()
            .map(|it| TreeRow {
                section_id: SectionId::IssueTypes,
                id: it.key.clone(),
                depth: 1,
                label: it.key.clone(),
                description: format!("{} · {}", it.name, it.mode),
                icon: StatusIcon::Tool,
                brand_icon: None,
                is_header: false,
                actions: ActionSet::none(),
                health: SectionHealth::Gray,
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rest::RestApiStatus;
    use crate::ui::status_panel::{IssueTypeInfo, WrapperConnectionStatus};

    fn base_snapshot() -> StatusSnapshot {
        StatusSnapshot {
            working_dir: "/test".into(),
            config_file_found: true,
            config_path: "operator.toml".into(),
            tickets_dir: ".tickets".into(),
            tickets_dir_exists: true,
            wrapper_type: "tmux".into(),
            operator_inside_wrapper: false,
            operator_version: "0.1.28".into(),
            api_status: RestApiStatus::Running { port: 7008 },
            kanban_providers: vec![],
            llm_tools: vec![],
            delegators: vec![],
            model_servers: vec![],
            issue_types: vec![],
            managed_projects: vec![],
            git_provider: None,
            git_token_set: false,
            git_branch_format: None,
            git_use_worktrees: false,
            default_llm_tool: None,
            default_llm_model: None,
            update_available_version: None,
            wrapper_connection_status: WrapperConnectionStatus::Tmux {
                available: true,
                server_running: true,
                version: Some("tmux 3.4".into()),
            },
            env_editor: "vim".into(),
            env_visual: String::new(),
            mcp_http_status: crate::ui::status_panel::McpHttpStatus::Mounted { port: 7008 },
            mcp_stdio_advertised: true,
            mcp_active_sessions: 0,
            acp_stdio_advertised: true,
            acp_active_sessions: 0,
            embed_ui_available: true,
        }
    }

    fn snapshot_with(issue_types: Vec<IssueTypeInfo>) -> StatusSnapshot {
        StatusSnapshot {
            issue_types,
            ..base_snapshot()
        }
    }

    fn sample_types() -> Vec<IssueTypeInfo> {
        vec![
            IssueTypeInfo {
                key: "FEAT".into(),
                name: "Feature".into(),
                mode: "autonomous".into(),
            },
            IssueTypeInfo {
                key: "SPIKE".into(),
                name: "Spike".into(),
                mode: "paired".into(),
            },
        ]
    }

    #[test]
    fn test_issuetype_prerequisite_is_kanban() {
        assert_eq!(IssueTypeSection.prerequisites(), &[SectionId::Kanban]);
    }

    #[test]
    fn test_issuetype_health_yellow_when_empty() {
        let snap = snapshot_with(vec![]);
        assert_eq!(IssueTypeSection.health(&snap), SectionHealth::Yellow);
    }

    #[test]
    fn test_issuetype_health_green_when_present() {
        let snap = snapshot_with(sample_types());
        assert_eq!(IssueTypeSection.health(&snap), SectionHealth::Green);
    }

    #[test]
    fn test_issuetype_description_counts_types() {
        assert_eq!(
            IssueTypeSection.description(&snapshot_with(vec![])),
            "Not configured"
        );
        assert_eq!(
            IssueTypeSection.description(&snapshot_with(sample_types())),
            "2 types"
        );
    }

    #[test]
    fn test_issuetype_children_render_key_name_mode() {
        let snap = snapshot_with(sample_types());
        let children = IssueTypeSection.children(&snap);
        assert_eq!(children.len(), 2);
        assert_eq!(children[0].label, "FEAT");
        assert_eq!(children[0].description, "Feature · autonomous");
        assert_eq!(children[1].label, "SPIKE");
        assert_eq!(children[1].description, "Spike · paired");
    }
}

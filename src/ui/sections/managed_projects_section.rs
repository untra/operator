use crate::ui::status_panel::{
    ActionSet, SectionHealth, SectionId, StatusIcon, StatusSection, StatusSnapshot, TreeRow,
};

/// Managed Projects section — mirrors the VS Code extension's `ManagedProjectsSection`.
/// Visible once Git is configured; lists the projects operator can assign work to.
pub struct ManagedProjectsSection;

impl StatusSection for ManagedProjectsSection {
    fn section_id(&self) -> SectionId {
        SectionId::ManagedProjects
    }

    fn label(&self) -> &'static str {
        "Managed Projects"
    }

    fn prerequisites(&self) -> &[SectionId] {
        &[SectionId::Git]
    }

    fn health(&self, snapshot: &StatusSnapshot) -> SectionHealth {
        if snapshot.managed_projects.is_empty() {
            SectionHealth::Yellow
        } else {
            SectionHealth::Green
        }
    }

    fn description(&self, snapshot: &StatusSnapshot) -> String {
        let count = snapshot.managed_projects.len();
        if count == 0 {
            "No projects configured".into()
        } else {
            format!("{count} project{}", if count == 1 { "" } else { "s" })
        }
    }

    fn children(&self, snapshot: &StatusSnapshot) -> Vec<TreeRow> {
        snapshot
            .managed_projects
            .iter()
            .map(|proj| TreeRow {
                section_id: SectionId::ManagedProjects,
                id: proj.name.clone(),
                depth: 1,
                label: proj.name.clone(),
                description: if proj.exists {
                    String::new()
                } else {
                    "missing".into()
                },
                icon: if proj.exists {
                    StatusIcon::Folder
                } else {
                    StatusIcon::Warning
                },
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
    use crate::ui::status_panel::{ManagedProjectInfo, WrapperConnectionStatus};

    fn base_snapshot() -> StatusSnapshot {
        StatusSnapshot {
            working_dir: "/test".into(),
            config_file_found: true,
            config_path: "operator.toml".into(),
            tickets_dir: ".tickets".into(),
            tickets_dir_exists: true,
            wrapper_type: "tmux".into(),
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

    fn snapshot_with(managed_projects: Vec<ManagedProjectInfo>) -> StatusSnapshot {
        StatusSnapshot {
            managed_projects,
            ..base_snapshot()
        }
    }

    #[test]
    fn test_projects_prerequisite_is_git() {
        assert_eq!(ManagedProjectsSection.prerequisites(), &[SectionId::Git]);
    }

    #[test]
    fn test_projects_health_yellow_when_empty() {
        let snap = snapshot_with(vec![]);
        assert_eq!(ManagedProjectsSection.health(&snap), SectionHealth::Yellow);
    }

    #[test]
    fn test_projects_health_green_when_present() {
        let snap = snapshot_with(vec![ManagedProjectInfo {
            name: "operator".into(),
            exists: true,
        }]);
        assert_eq!(ManagedProjectsSection.health(&snap), SectionHealth::Green);
    }

    #[test]
    fn test_projects_description_counts_projects() {
        assert_eq!(
            ManagedProjectsSection.description(&snapshot_with(vec![])),
            "No projects configured"
        );
        let snap = snapshot_with(vec![ManagedProjectInfo {
            name: "operator".into(),
            exists: true,
        }]);
        assert_eq!(ManagedProjectsSection.description(&snap), "1 project");
    }

    #[test]
    fn test_projects_children_flag_missing_dirs() {
        let snap = snapshot_with(vec![
            ManagedProjectInfo {
                name: "present".into(),
                exists: true,
            },
            ManagedProjectInfo {
                name: "gone".into(),
                exists: false,
            },
        ]);
        let children = ManagedProjectsSection.children(&snap);
        assert_eq!(children.len(), 2);
        assert_eq!(children[0].label, "present");
        assert_eq!(children[0].description, "");
        assert!(matches!(children[0].icon, StatusIcon::Folder));
        assert_eq!(children[1].label, "gone");
        assert_eq!(children[1].description, "missing");
        assert!(matches!(children[1].icon, StatusIcon::Warning));
    }
}

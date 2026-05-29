use crate::ui::status_panel::{
    ActionMeta, ActionSet, SectionHealth, SectionId, StatusAction, StatusIcon, StatusSection,
    StatusSnapshot, TreeRow,
};

pub struct KanbanSection;

impl StatusSection for KanbanSection {
    fn section_id(&self) -> SectionId {
        SectionId::Kanban
    }

    fn label(&self) -> &'static str {
        "Kanban"
    }

    fn prerequisites(&self) -> &[SectionId] {
        &[SectionId::Connections]
    }

    fn health(&self, snapshot: &StatusSnapshot) -> SectionHealth {
        if snapshot.kanban_providers.is_empty() {
            SectionHealth::Yellow
        } else {
            SectionHealth::Green
        }
    }

    fn description(&self, snapshot: &StatusSnapshot) -> String {
        snapshot
            .kanban_providers
            .first()
            .map(|p| p.provider_type.clone())
            .unwrap_or_else(|| "No provider connected".into())
    }

    fn children(&self, snapshot: &StatusSnapshot) -> Vec<TreeRow> {
        if snapshot.kanban_providers.is_empty() {
            return vec![
                TreeRow {
                    section_id: SectionId::Kanban,
                    id: "configure-jira".into(),
                    depth: 1,
                    label: "Configure Jira".into(),
                    description: "Connect to Jira Cloud".into(),
                    icon: StatusIcon::Plug,
                    is_header: false,
                    actions: ActionSet::primary(StatusAction::ConfigureKanbanProvider {
                        provider: "jira".into(),
                    }),
                    health: SectionHealth::Gray,
                },
                TreeRow {
                    section_id: SectionId::Kanban,
                    id: "configure-linear".into(),
                    depth: 1,
                    label: "Configure Linear".into(),
                    description: "Connect to Linear".into(),
                    icon: StatusIcon::Plug,
                    is_header: false,
                    actions: ActionSet::primary(StatusAction::ConfigureKanbanProvider {
                        provider: "linear".into(),
                    }),
                    health: SectionHealth::Gray,
                },
            ];
        }

        snapshot
            .kanban_providers
            .iter()
            .map(|provider| TreeRow {
                section_id: SectionId::Kanban,
                id: provider.domain.clone(),
                depth: 1,
                label: provider.provider_type.clone(),
                description: provider.domain.clone(),
                icon: StatusIcon::Plug,
                is_header: false,
                actions: ActionSet {
                    primary: StatusAction::None,
                    back: StatusAction::None,
                    special: StatusAction::None,
                    special_meta: None,
                    refresh: StatusAction::RefreshSection(SectionId::Kanban),
                    refresh_meta: Some(ActionMeta {
                        title: "Sync",
                        tooltip: "Re-check kanban provider connection",
                    }),
                },
                health: SectionHealth::Gray,
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rest::RestApiStatus;
    use crate::ui::status_panel::{
        DelegatorInfo, KanbanProviderInfo, LlmToolInfo, WrapperConnectionStatus,
    };

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
            default_llm_tool: None,
            default_llm_model: None,
            delegators: vec![],
            model_servers: vec![],
            issue_types: vec![],
            managed_projects: vec![],
            git_provider: None,
            git_token_set: false,
            git_branch_format: None,
            git_use_worktrees: false,
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

    #[test]
    fn test_kanban_health_yellow_when_no_providers() {
        let section = KanbanSection;
        let snap = base_snapshot();
        assert_eq!(section.health(&snap), SectionHealth::Yellow);
    }

    #[test]
    fn test_kanban_health_green_when_providers_configured() {
        let section = KanbanSection;
        let mut snap = base_snapshot();
        snap.kanban_providers.push(KanbanProviderInfo {
            provider_type: "jira".into(),
            domain: "myteam.atlassian.net".into(),
        });
        assert_eq!(section.health(&snap), SectionHealth::Green);
    }

    #[test]
    fn test_kanban_description_no_provider() {
        let section = KanbanSection;
        let snap = base_snapshot();
        assert_eq!(section.description(&snap), "No provider connected");
    }

    #[test]
    fn test_kanban_description_with_provider() {
        let section = KanbanSection;
        let mut snap = base_snapshot();
        snap.kanban_providers.push(KanbanProviderInfo {
            provider_type: "Linear".into(),
            domain: "myteam".into(),
        });
        assert_eq!(section.description(&snap), "Linear");
    }

    #[test]
    fn test_kanban_children_empty_shows_configure_options() {
        let section = KanbanSection;
        let snap = base_snapshot();
        let children = section.children(&snap);

        assert_eq!(children.len(), 2);
        assert_eq!(children[0].label, "Configure Jira");
        assert_eq!(children[0].description, "Connect to Jira Cloud");
        assert_eq!(
            children[0].actions.primary,
            StatusAction::ConfigureKanbanProvider {
                provider: "jira".into()
            }
        );
        assert_eq!(children[1].label, "Configure Linear");
        assert_eq!(children[1].description, "Connect to Linear");
        assert_eq!(
            children[1].actions.primary,
            StatusAction::ConfigureKanbanProvider {
                provider: "linear".into()
            }
        );
    }

    #[test]
    fn test_kanban_children_with_providers_shows_provider_rows() {
        let section = KanbanSection;
        let mut snap = base_snapshot();
        snap.kanban_providers.push(KanbanProviderInfo {
            provider_type: "jira".into(),
            domain: "myteam.atlassian.net".into(),
        });
        let children = section.children(&snap);

        assert_eq!(children.len(), 1);
        assert_eq!(children[0].label, "jira");
        assert_eq!(children[0].description, "myteam.atlassian.net");
        assert_eq!(children[0].actions.primary, StatusAction::None);
    }
}

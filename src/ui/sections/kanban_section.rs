use crate::api::providers::kanban::KanbanProviderType;
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
        // A row per connected provider, followed by a "Configure" row for every
        // supported provider not yet connected. Both the configure list and the
        // ordering come from the canonical `KanbanProviderType::ALL` catalog, so
        // the TUI, the web `/#/kanban` view, and VS Code onboarding stay aligned.
        let mut rows: Vec<TreeRow> = snapshot
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
            .collect();

        for provider in KanbanProviderType::ALL {
            let already_connected = snapshot
                .kanban_providers
                .iter()
                .any(|p| p.provider_type == provider.slug());
            if already_connected {
                continue;
            }
            rows.push(TreeRow {
                section_id: SectionId::Kanban,
                id: format!("configure-{}", provider.slug()),
                depth: 1,
                label: format!("Configure {}", provider.display_name()),
                description: provider.connect_blurb().to_string(),
                icon: StatusIcon::Plug,
                is_header: false,
                actions: ActionSet::primary(StatusAction::ConfigureKanbanProvider {
                    provider: provider.slug().to_string(),
                }),
                health: SectionHealth::Gray,
            });
        }

        rows
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
            operator_inside_wrapper: false,
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
    fn test_kanban_children_empty_shows_all_three_configure_options() {
        let section = KanbanSection;
        let snap = base_snapshot();
        let children = section.children(&snap);

        // Every supported provider is offered when none are connected.
        assert_eq!(children.len(), 3);
        assert_eq!(children[0].label, "Configure Jira Cloud");
        assert_eq!(children[0].description, "Connect to Jira Cloud");
        assert_eq!(
            children[0].actions.primary,
            StatusAction::ConfigureKanbanProvider {
                provider: "jira".into()
            }
        );
        assert_eq!(children[1].label, "Configure Linear");
        assert_eq!(
            children[1].actions.primary,
            StatusAction::ConfigureKanbanProvider {
                provider: "linear".into()
            }
        );
        assert_eq!(children[2].label, "Configure GitHub Projects");
        assert_eq!(children[2].description, "Connect to GitHub Projects");
        assert_eq!(
            children[2].actions.primary,
            StatusAction::ConfigureKanbanProvider {
                provider: "github".into()
            }
        );
    }

    #[test]
    fn test_kanban_children_with_one_provider_still_offers_the_rest() {
        let section = KanbanSection;
        let mut snap = base_snapshot();
        snap.kanban_providers.push(KanbanProviderInfo {
            provider_type: "jira".into(),
            domain: "myteam.atlassian.net".into(),
        });
        let children = section.children(&snap);

        // The connected provider row, plus "add" rows for the two not yet connected.
        assert_eq!(children.len(), 3);
        assert_eq!(children[0].label, "jira");
        assert_eq!(children[0].description, "myteam.atlassian.net");
        assert_eq!(children[0].actions.primary, StatusAction::None);

        let configure: Vec<_> = children
            .iter()
            .filter_map(|r| match &r.actions.primary {
                StatusAction::ConfigureKanbanProvider { provider } => Some(provider.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(configure, vec!["linear", "github"]);
    }
}

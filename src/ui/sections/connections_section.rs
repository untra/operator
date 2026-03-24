use crate::backstage::ServerStatus;
use crate::rest::RestApiStatus;
use crate::ui::status_panel::{
    ActionMeta, ActionSet, SectionHealth, SectionId, StatusAction, StatusIcon, StatusSection,
    StatusSnapshot, TreeRow, WrapperConnectionStatus,
};

pub struct ConnectionsSection;

impl StatusSection for ConnectionsSection {
    fn section_id(&self) -> SectionId {
        SectionId::Connections
    }

    fn label(&self) -> &'static str {
        "Connections"
    }

    fn prerequisites(&self) -> &[SectionId] {
        &[SectionId::Configuration]
    }

    fn health(&self, snapshot: &StatusSnapshot) -> SectionHealth {
        let api_ok = matches!(snapshot.api_status, RestApiStatus::Running { .. });
        let api_starting = matches!(snapshot.api_status, RestApiStatus::Starting);
        let wrapper_ok = snapshot.wrapper_connection_status.is_connected();

        // When backstage is hidden, health is based on API + wrapper only
        if !snapshot.backstage_display {
            return match (api_ok, wrapper_ok) {
                (true, true) => SectionHealth::Green,
                _ if api_starting => SectionHealth::Yellow,
                (true, false) | (false, true) => SectionHealth::Yellow,
                (false, false) => SectionHealth::Red,
            };
        }

        // When backstage is displayed, include it in health
        let bs_ok = matches!(snapshot.backstage_status, ServerStatus::Running { .. });
        let bs_starting = matches!(snapshot.backstage_status, ServerStatus::Starting);
        let all_ok = api_ok && bs_ok && wrapper_ok;
        let any_starting = api_starting || bs_starting;

        if all_ok {
            SectionHealth::Green
        } else if any_starting || api_ok || bs_ok || wrapper_ok {
            SectionHealth::Yellow
        } else {
            SectionHealth::Red
        }
    }

    fn description(&self, snapshot: &StatusSnapshot) -> String {
        let api_ok = matches!(snapshot.api_status, RestApiStatus::Running { .. });
        let api_starting = matches!(snapshot.api_status, RestApiStatus::Starting);
        let wrapper_ok = snapshot.wrapper_connection_status.is_connected();

        if api_starting {
            return "Starting...".into();
        }
        if api_ok && wrapper_ok {
            return "Connected".into();
        }
        if !api_ok && !wrapper_ok {
            return "Disconnected".into();
        }
        "Partial".into()
    }

    fn children(&self, snapshot: &StatusSnapshot) -> Vec<TreeRow> {
        let mut rows = vec![
            // 1. Operator API
            TreeRow {
                section_id: SectionId::Connections,
                depth: 1,
                label: "Operator API".into(),
                description: match &snapshot.api_status {
                    RestApiStatus::Running { port } => format!(":{port}"),
                    RestApiStatus::Starting => "Starting...".into(),
                    RestApiStatus::Stopping => "Stopping...".into(),
                    RestApiStatus::Stopped => "Stopped".into(),
                    RestApiStatus::Error(e) => format!("Error: {e}"),
                },
                icon: match &snapshot.api_status {
                    RestApiStatus::Running { .. } => StatusIcon::Check,
                    RestApiStatus::Starting => StatusIcon::Warning,
                    _ => StatusIcon::Cross,
                },
                is_header: false,
                actions: ActionSet {
                    primary: match &snapshot.api_status {
                        RestApiStatus::Running { port } => {
                            StatusAction::OpenSwagger { port: *port }
                        }
                        RestApiStatus::Stopped | RestApiStatus::Error(_) => StatusAction::StartApi,
                        _ => StatusAction::None,
                    },
                    back: StatusAction::None,
                    special: match &snapshot.api_status {
                        RestApiStatus::Running { port } => {
                            StatusAction::OpenSwagger { port: *port }
                        }
                        _ => StatusAction::None,
                    },
                    special_meta: Some(ActionMeta {
                        title: "Docs",
                        tooltip: "Open Swagger API documentation",
                    }),
                    refresh: StatusAction::StartApi,
                    refresh_meta: Some(ActionMeta {
                        title: "Start",
                        tooltip: "Start or restart the Operator API server",
                    }),
                },
                health: SectionHealth::Gray,
            },
        ];

        // 2. Backstage (conditionally displayed)
        if snapshot.backstage_display {
            rows.push(TreeRow {
                section_id: SectionId::Connections,
                depth: 1,
                label: "Backstage".into(),
                description: format!("{:?}", snapshot.backstage_status),
                icon: if matches!(snapshot.backstage_status, ServerStatus::Running { .. }) {
                    StatusIcon::Check
                } else {
                    StatusIcon::Cross
                },
                is_header: false,
                actions: ActionSet::primary(StatusAction::ToggleWebServers),
                health: SectionHealth::Gray,
            });
        }

        rows
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::status_panel::{DelegatorInfo, KanbanProviderInfo, LlmToolInfo};

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
            backstage_status: ServerStatus::Stopped,
            backstage_display: false,
            kanban_providers: vec![],
            llm_tools: vec![],
            default_llm_tool: None,
            default_llm_model: None,
            delegators: vec![],
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
        }
    }

    #[test]
    fn test_connections_tmux_connected_green_health() {
        let section = ConnectionsSection;
        let snap = base_snapshot();
        // API running + tmux connected = Green
        assert_eq!(section.health(&snap), SectionHealth::Green);
    }

    #[test]
    fn test_connections_startup_grace_yellow_not_red() {
        let section = ConnectionsSection;
        let mut snap = base_snapshot();
        snap.api_status = RestApiStatus::Starting;
        // API starting + tmux connected should be Yellow, not Red
        assert_eq!(section.health(&snap), SectionHealth::Yellow);
    }

    #[test]
    fn test_connections_startup_grace_both_down_is_red() {
        let section = ConnectionsSection;
        let mut snap = base_snapshot();
        snap.api_status = RestApiStatus::Stopped;
        snap.wrapper_connection_status = WrapperConnectionStatus::Tmux {
            available: false,
            server_running: false,
            version: None,
        };
        assert_eq!(section.health(&snap), SectionHealth::Red);
    }

    #[test]
    fn test_connections_api_running_opens_swagger() {
        let section = ConnectionsSection;
        let snap = base_snapshot();
        let children = section.children(&snap);
        let api_row = children.iter().find(|r| r.label == "Operator API").unwrap();
        assert_eq!(
            api_row.actions.primary,
            StatusAction::OpenSwagger { port: 7008 }
        );
    }

    #[test]
    fn test_connections_api_stopped_starts_api() {
        let section = ConnectionsSection;
        let mut snap = base_snapshot();
        snap.api_status = RestApiStatus::Stopped;
        let children = section.children(&snap);
        let api_row = children.iter().find(|r| r.label == "Operator API").unwrap();
        assert_eq!(api_row.actions.primary, StatusAction::StartApi);
    }

    #[test]
    fn test_connections_backstage_hidden_by_default() {
        let section = ConnectionsSection;
        let snap = base_snapshot();
        let children = section.children(&snap);
        assert!(
            !children.iter().any(|r| r.label == "Backstage"),
            "Backstage should be hidden when backstage_display is false"
        );
    }

    #[test]
    fn test_connections_backstage_shown_when_display_true() {
        let section = ConnectionsSection;
        let mut snap = base_snapshot();
        snap.backstage_display = true;
        let children = section.children(&snap);
        assert!(
            children.iter().any(|r| r.label == "Backstage"),
            "Backstage should be shown when backstage_display is true"
        );
    }
}

use crate::rest::RestApiStatus;
use crate::ui::status_panel::{
    ActionMeta, ActionSet, McpHttpStatus, SectionHealth, SectionId, StatusAction, StatusIcon,
    StatusSection, StatusSnapshot, TreeRow, WrapperConnectionStatus,
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

        match (api_ok, wrapper_ok) {
            (true, true) => SectionHealth::Green,
            _ if api_starting => SectionHealth::Yellow,
            (true, false) | (false, true) => SectionHealth::Yellow,
            (false, false) => SectionHealth::Red,
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

        // 2. Web UI (when embed-ui feature is compiled in)
        if snapshot.embed_ui_available {
            rows.push(TreeRow {
                section_id: SectionId::Connections,
                depth: 1,
                label: "Web UI".into(),
                description: match &snapshot.api_status {
                    RestApiStatus::Running { port } => format!(":{port}"),
                    RestApiStatus::Starting => "Starting...".into(),
                    _ => "API stopped".into(),
                },
                icon: match &snapshot.api_status {
                    RestApiStatus::Running { .. } => StatusIcon::Check,
                    RestApiStatus::Starting => StatusIcon::Warning,
                    _ => StatusIcon::Cross,
                },
                is_header: false,
                actions: ActionSet {
                    primary: match &snapshot.api_status {
                        RestApiStatus::Running { port } => StatusAction::OpenWebUi { port: *port },
                        RestApiStatus::Stopped | RestApiStatus::Error(_) => StatusAction::StartApi,
                        _ => StatusAction::None,
                    },
                    back: StatusAction::None,
                    special: StatusAction::None,
                    special_meta: None,
                    refresh: StatusAction::None,
                    refresh_meta: None,
                },
                health: SectionHealth::Gray,
            });
        }

        // 3. MCP (always shown). Status reflects HTTP mount + stdio advertise + session count.
        rows.push(TreeRow {
            section_id: SectionId::Connections,
            depth: 1,
            label: "MCP".into(),
            description: match (
                &snapshot.mcp_http_status,
                snapshot.mcp_stdio_advertised,
                snapshot.mcp_active_sessions,
            ) {
                (McpHttpStatus::Mounted { port }, true, n) if n > 0 => {
                    format!(":{port} + stdio · {n} sessions")
                }
                (McpHttpStatus::Mounted { port }, true, _) => format!(":{port} + stdio"),
                (McpHttpStatus::Mounted { port }, false, n) if n > 0 => {
                    format!(":{port} · {n} sessions")
                }
                (McpHttpStatus::Mounted { port }, false, _) => format!(":{port} (HTTP only)"),
                (McpHttpStatus::NotMounted, true, _) => "stdio only".into(),
                (McpHttpStatus::NotMounted, false, _) => "Disabled".into(),
            },
            icon: match (&snapshot.mcp_http_status, snapshot.mcp_stdio_advertised) {
                (McpHttpStatus::Mounted { .. }, _) | (_, true) => StatusIcon::Check,
                _ => StatusIcon::Cross,
            },
            is_header: false,
            actions: ActionSet {
                primary: StatusAction::WriteAndOpenMcpClientConfig {
                    client: "claude-code".to_string(),
                },
                back: StatusAction::None,
                special: StatusAction::ToggleMcpHttp,
                special_meta: Some(ActionMeta {
                    title: "HTTP",
                    tooltip: "Toggle the MCP HTTP transport (restart required)",
                }),
                refresh: StatusAction::OpenMcpDocs,
                refresh_meta: Some(ActionMeta {
                    title: "Docs",
                    tooltip: "Open MCP setup docs in browser",
                }),
            },
            health: SectionHealth::Gray,
        });

        // 3. ACP (always shown). Status reflects whether operator
        //    advertises itself as an ACP agent. Active session count is
        //    always 0 in v1 (editor-spawned ACP runs out-of-process).
        rows.push(TreeRow {
            section_id: SectionId::Connections,
            depth: 1,
            label: "ACP".into(),
            description: if snapshot.acp_stdio_advertised {
                if snapshot.acp_active_sessions > 0 {
                    format!("stdio · {} sessions", snapshot.acp_active_sessions)
                } else {
                    "stdio ready".into()
                }
            } else {
                "Disabled".into()
            },
            icon: if snapshot.acp_stdio_advertised {
                StatusIcon::Check
            } else {
                StatusIcon::Cross
            },
            is_header: false,
            actions: ActionSet {
                primary: StatusAction::WriteAndOpenAcpEditorConfig {
                    editor: "zed".to_string(),
                },
                back: StatusAction::None,
                special: StatusAction::WriteAndOpenAcpEditorConfig {
                    editor: "jetbrains".to_string(),
                },
                special_meta: Some(ActionMeta {
                    title: "JBrn",
                    tooltip: "Write JetBrains ACP registry snippet",
                }),
                refresh: StatusAction::OpenAcpDocs,
                refresh_meta: Some(ActionMeta {
                    title: "Docs",
                    tooltip: "Open ACP setup docs in browser",
                }),
            },
            health: SectionHealth::Gray,
        });

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
            kanban_providers: vec![],
            llm_tools: vec![],
            default_llm_tool: None,
            default_llm_model: None,
            delegators: vec![],
            model_servers: vec![],
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
            mcp_http_status: McpHttpStatus::Mounted { port: 7008 },
            mcp_stdio_advertised: true,
            mcp_active_sessions: 0,
            acp_stdio_advertised: true,
            acp_active_sessions: 0,
            embed_ui_available: true,
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
    fn test_connections_acp_row_present_when_advertised() {
        let section = ConnectionsSection;
        let snap = base_snapshot();
        let children = section.children(&snap);
        let acp_row = children
            .iter()
            .find(|r| r.label == "ACP")
            .expect("ACP row must be in the connections section");
        assert!(matches!(acp_row.icon, StatusIcon::Check));
        assert_eq!(acp_row.description, "stdio ready");
        assert_eq!(
            acp_row.actions.primary,
            StatusAction::WriteAndOpenAcpEditorConfig {
                editor: "zed".to_string()
            }
        );
        assert_eq!(acp_row.actions.refresh, StatusAction::OpenAcpDocs);
    }

    #[test]
    fn test_connections_acp_row_disabled_when_flag_off() {
        let section = ConnectionsSection;
        let mut snap = base_snapshot();
        snap.acp_stdio_advertised = false;
        let children = section.children(&snap);
        let acp_row = children.iter().find(|r| r.label == "ACP").unwrap();
        assert!(matches!(acp_row.icon, StatusIcon::Cross));
        assert_eq!(acp_row.description, "Disabled");
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
    fn test_connections_web_ui_row_present_when_embed_ui_available() {
        let section = ConnectionsSection;
        let snap = base_snapshot();
        let children = section.children(&snap);
        let web_ui_row = children
            .iter()
            .find(|r| r.label == "Web UI")
            .expect("Web UI row must be present when embed_ui_available is true");
        assert!(matches!(web_ui_row.icon, StatusIcon::Check));
        assert_eq!(web_ui_row.description, ":7008");
        assert_eq!(
            web_ui_row.actions.primary,
            StatusAction::OpenWebUi { port: 7008 }
        );
    }

    #[test]
    fn test_connections_web_ui_row_absent_when_embed_ui_unavailable() {
        let section = ConnectionsSection;
        let mut snap = base_snapshot();
        snap.embed_ui_available = false;
        let children = section.children(&snap);
        assert!(
            !children.iter().any(|r| r.label == "Web UI"),
            "Web UI row should be hidden when embed_ui_available is false"
        );
    }

    #[test]
    fn test_connections_web_ui_row_starts_api_when_stopped() {
        let section = ConnectionsSection;
        let mut snap = base_snapshot();
        snap.api_status = RestApiStatus::Stopped;
        let children = section.children(&snap);
        let web_ui_row = children.iter().find(|r| r.label == "Web UI").unwrap();
        assert!(matches!(web_ui_row.icon, StatusIcon::Cross));
        assert_eq!(web_ui_row.description, "API stopped");
        assert_eq!(web_ui_row.actions.primary, StatusAction::StartApi);
    }

    #[test]
    fn test_connections_mcp_row_always_present() {
        let section = ConnectionsSection;
        let snap = base_snapshot();
        let children = section.children(&snap);
        assert!(
            children.iter().any(|r| r.label == "MCP"),
            "MCP row should always be present"
        );
    }

    #[test]
    fn test_connections_mcp_row_disabled_description() {
        let section = ConnectionsSection;
        let mut snap = base_snapshot();
        snap.mcp_http_status = McpHttpStatus::NotMounted;
        snap.mcp_stdio_advertised = false;
        let children = section.children(&snap);
        let mcp_row = children.iter().find(|r| r.label == "MCP").unwrap();
        assert_eq!(mcp_row.description, "Disabled");
    }

    #[test]
    fn test_connections_mcp_row_stdio_only_description() {
        let section = ConnectionsSection;
        let mut snap = base_snapshot();
        snap.mcp_http_status = McpHttpStatus::NotMounted;
        snap.mcp_stdio_advertised = true;
        let children = section.children(&snap);
        let mcp_row = children.iter().find(|r| r.label == "MCP").unwrap();
        assert_eq!(mcp_row.description, "stdio only");
    }

    #[test]
    fn test_connections_mcp_row_actions() {
        let section = ConnectionsSection;
        let snap = base_snapshot();
        let children = section.children(&snap);
        let mcp_row = children.iter().find(|r| r.label == "MCP").unwrap();
        // Primary writes the claude-code snippet by default.
        assert_eq!(
            mcp_row.actions.primary,
            StatusAction::WriteAndOpenMcpClientConfig {
                client: "claude-code".to_string()
            }
        );
        assert_eq!(mcp_row.actions.special, StatusAction::ToggleMcpHttp);
        assert_eq!(mcp_row.actions.refresh, StatusAction::OpenMcpDocs);
    }
}

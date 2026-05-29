use crate::ui::status_panel::{
    ActionMeta, ActionSet, SectionHealth, SectionId, StatusAction, StatusIcon, StatusSection,
    StatusSnapshot, TreeRow,
};

pub struct DelegatorSection;

impl StatusSection for DelegatorSection {
    fn section_id(&self) -> SectionId {
        SectionId::Delegators
    }

    fn label(&self) -> &'static str {
        "Delegators"
    }

    fn prerequisites(&self) -> &[SectionId] {
        &[SectionId::LlmTools]
    }

    fn health(&self, snapshot: &StatusSnapshot) -> SectionHealth {
        if snapshot.delegators.is_empty() {
            SectionHealth::Yellow
        } else {
            SectionHealth::Green
        }
    }

    fn description(&self, snapshot: &StatusSnapshot) -> String {
        let count = snapshot.delegators.len();
        if count == 0 {
            "None configured".into()
        } else {
            format!("{count} delegator{}", if count == 1 { "" } else { "s" })
        }
    }

    fn children(&self, snapshot: &StatusSnapshot) -> Vec<TreeRow> {
        if snapshot.delegators.is_empty() {
            return vec![TreeRow {
                section_id: SectionId::Delegators,
                id: "add-delegator".into(),
                depth: 1,
                label: "Add delegator".into(),
                description: "Edit config to configure a delegator".into(),
                icon: StatusIcon::Tool,
                is_header: false,
                actions: ActionSet::primary(StatusAction::EditFile(snapshot.config_path.clone())),
                health: SectionHealth::Gray,
            }];
        }

        snapshot
            .delegators
            .iter()
            .map(|d| {
                let label = d.display_name.as_deref().unwrap_or(&d.name).to_string();
                let yolo_flag = if d.yolo { " · yolo" } else { "" };
                let server_suffix = d
                    .model_server
                    .as_deref()
                    .map(|s| format!(" @ {s}"))
                    .unwrap_or_default();
                let description =
                    format!("{}:{}{}{}", d.llm_tool, d.model, yolo_flag, server_suffix);

                TreeRow {
                    section_id: SectionId::Delegators,
                    id: d.name.clone(),
                    depth: 1,
                    label,
                    description,
                    icon: StatusIcon::Tool,
                    is_header: false,
                    actions: ActionSet {
                        primary: StatusAction::None,
                        back: StatusAction::None,
                        special: StatusAction::EditFile(snapshot.config_path.clone()),
                        special_meta: Some(ActionMeta {
                            title: "Config",
                            tooltip: "Edit delegator configuration",
                        }),
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
    use crate::rest::RestApiStatus;
    use crate::ui::status_panel::{DelegatorInfo, WrapperConnectionStatus};

    fn snapshot_with(delegators: Vec<DelegatorInfo>) -> StatusSnapshot {
        StatusSnapshot {
            working_dir: "/test".into(),
            config_file_found: true,
            config_path: "operator.toml".into(),
            tickets_dir: ".tickets".into(),
            tickets_dir_exists: true,
            wrapper_type: "tmux".into(),
            operator_version: "0.1.30".into(),
            api_status: RestApiStatus::Stopped,
            kanban_providers: vec![],
            llm_tools: vec![],
            default_llm_tool: None,
            default_llm_model: None,
            delegators,
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
                version: None,
            },
            env_editor: String::new(),
            env_visual: String::new(),
            mcp_http_status: crate::ui::status_panel::McpHttpStatus::NotMounted,
            mcp_stdio_advertised: true,
            mcp_active_sessions: 0,
            acp_stdio_advertised: true,
            acp_active_sessions: 0,
            embed_ui_available: true,
        }
    }

    fn delegator(name: &str, tool: &str, model: &str, server: Option<&str>) -> DelegatorInfo {
        DelegatorInfo {
            name: name.into(),
            display_name: None,
            llm_tool: tool.into(),
            model: model.into(),
            yolo: false,
            model_server: server.map(String::from),
        }
    }

    #[test]
    fn test_empty_delegators_shows_add_row() {
        let snap = snapshot_with(vec![]);
        let rows = DelegatorSection.children(&snap);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].label, "Add delegator");
        assert!(matches!(
            &rows[0].actions.primary,
            StatusAction::EditFile(_)
        ));
    }

    #[test]
    fn test_description_includes_server_when_set() {
        let snap = snapshot_with(vec![delegator(
            "codex-qwen",
            "codex",
            "qwen2.5-coder",
            Some("ollama-local"),
        )]);
        let rows = DelegatorSection.children(&snap);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].description, "codex:qwen2.5-coder @ ollama-local");
    }

    #[test]
    fn test_description_omits_server_when_default() {
        let snap = snapshot_with(vec![delegator("claude-opus", "claude", "opus", None)]);
        let rows = DelegatorSection.children(&snap);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].description, "claude:opus");
    }
}

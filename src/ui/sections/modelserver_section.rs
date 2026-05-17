use crate::ui::status_panel::{
    ActionMeta, ActionSet, SectionHealth, SectionId, StatusAction, StatusIcon, StatusSection,
    StatusSnapshot, TreeRow,
};

pub struct ModelServerSection;

impl StatusSection for ModelServerSection {
    fn section_id(&self) -> SectionId {
        SectionId::ModelServers
    }

    fn label(&self) -> &'static str {
        "Model Servers"
    }

    fn prerequisites(&self) -> &[SectionId] {
        &[SectionId::LlmTools]
    }

    fn health(&self, snapshot: &StatusSnapshot) -> SectionHealth {
        if snapshot.model_servers.iter().any(|s| s.user_declared) {
            SectionHealth::Green
        } else {
            SectionHealth::Gray
        }
    }

    fn description(&self, snapshot: &StatusSnapshot) -> String {
        let declared = snapshot
            .model_servers
            .iter()
            .filter(|s| s.user_declared)
            .count();
        if declared == 0 {
            "builtins only".into()
        } else {
            format!("{declared} declared")
        }
    }

    fn children(&self, snapshot: &StatusSnapshot) -> Vec<TreeRow> {
        snapshot
            .model_servers
            .iter()
            .map(|s| {
                let label = s.display_name.clone().unwrap_or_else(|| s.name.clone());
                let base = s
                    .base_url
                    .as_deref()
                    .map(truncate_base_url)
                    .unwrap_or_default();
                let description = if base.is_empty() {
                    if s.user_declared {
                        s.kind.clone()
                    } else {
                        format!("{} · builtin", s.kind)
                    }
                } else {
                    format!("{} · {}", s.kind, base)
                };

                let action = if s.user_declared {
                    ActionSet {
                        primary: StatusAction::None,
                        back: StatusAction::None,
                        special: StatusAction::EditFile(snapshot.config_path.clone()),
                        special_meta: Some(ActionMeta {
                            title: "Config",
                            tooltip: "Edit model server configuration",
                        }),
                        refresh: StatusAction::None,
                        refresh_meta: None,
                    }
                } else {
                    ActionSet {
                        primary: StatusAction::None,
                        back: StatusAction::None,
                        special: StatusAction::None,
                        special_meta: None,
                        refresh: StatusAction::None,
                        refresh_meta: None,
                    }
                };

                TreeRow {
                    section_id: SectionId::ModelServers,
                    depth: 1,
                    label,
                    description,
                    icon: StatusIcon::Tool,
                    is_header: false,
                    actions: action,
                    health: if s.user_declared {
                        SectionHealth::Green
                    } else {
                        SectionHealth::Gray
                    },
                }
            })
            .collect()
    }
}

fn truncate_base_url(url: &str) -> String {
    const MAX: usize = 40;
    if url.len() <= MAX {
        url.to_string()
    } else {
        format!("{}…", &url[..MAX.saturating_sub(1)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backstage::ServerStatus;
    use crate::rest::RestApiStatus;
    use crate::ui::status_panel::{ModelServerInfo, WrapperConnectionStatus};

    fn snapshot_with_servers(servers: Vec<ModelServerInfo>) -> StatusSnapshot {
        StatusSnapshot {
            working_dir: "/test".into(),
            config_file_found: true,
            config_path: "operator.toml".into(),
            tickets_dir: ".tickets".into(),
            tickets_dir_exists: true,
            wrapper_type: "tmux".into(),
            operator_version: "0.1.30".into(),
            api_status: RestApiStatus::Stopped,
            backstage_status: ServerStatus::Stopped,
            backstage_display: false,
            kanban_providers: vec![],
            llm_tools: vec![],
            default_llm_tool: None,
            default_llm_model: None,
            delegators: vec![],
            model_servers: servers,
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
        }
    }

    fn builtin(name: &str, kind: &str) -> ModelServerInfo {
        ModelServerInfo {
            name: name.into(),
            kind: kind.into(),
            base_url: None,
            display_name: None,
            user_declared: false,
        }
    }

    fn declared(name: &str, kind: &str, base: &str) -> ModelServerInfo {
        ModelServerInfo {
            name: name.into(),
            kind: kind.into(),
            base_url: Some(base.into()),
            display_name: None,
            user_declared: true,
        }
    }

    #[test]
    fn test_description_builtins_only() {
        let snapshot = snapshot_with_servers(vec![
            builtin("anthropic-api", "anthropic-api"),
            builtin("openai-api", "openai-api"),
        ]);
        let section = ModelServerSection;
        assert_eq!(section.description(&snapshot), "builtins only");
        assert!(matches!(section.health(&snapshot), SectionHealth::Gray));
    }

    #[test]
    fn test_description_counts_declared() {
        let snapshot = snapshot_with_servers(vec![
            builtin("anthropic-api", "anthropic-api"),
            declared("ollama-local", "ollama", "http://localhost:11434"),
            declared("vllm-gpu", "openai-compat", "http://gpu:8000"),
        ]);
        let section = ModelServerSection;
        assert_eq!(section.description(&snapshot), "2 declared");
        assert!(matches!(section.health(&snapshot), SectionHealth::Green));
    }

    #[test]
    fn test_children_render_base_url_for_declared() {
        let snapshot = snapshot_with_servers(vec![declared(
            "ollama-local",
            "ollama",
            "http://localhost:11434",
        )]);
        let rows = ModelServerSection.children(&snapshot);
        assert_eq!(rows.len(), 1);
        assert!(rows[0].description.contains("ollama"));
        assert!(rows[0].description.contains("localhost:11434"));
    }

    #[test]
    fn test_children_mark_builtin_as_such() {
        let snapshot = snapshot_with_servers(vec![builtin("anthropic-api", "anthropic-api")]);
        let rows = ModelServerSection.children(&snapshot);
        assert_eq!(rows.len(), 1);
        assert!(rows[0].description.contains("builtin"));
    }
}

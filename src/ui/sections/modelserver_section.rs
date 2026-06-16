use crate::api::providers::model_server::{ModelProviderClass, ModelServerKind};
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
        let mut rows: Vec<TreeRow> = snapshot
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
                    id: s.name.clone(),
                    depth: 1,
                    label,
                    description,
                    icon: StatusIcon::Tool,
                    brand_icon: ModelServerKind::from_slug(&s.kind)
                        .and_then(|k| k.brand_icon())
                        .map(str::to_string),
                    is_header: false,
                    actions: action,
                    health: if s.user_declared {
                        SectionHealth::Green
                    } else {
                        SectionHealth::Gray
                    },
                }
            })
            .collect();

        // Catalog "Add <kind>" rows for the addable (non-builtin) kinds, derived
        // from ModelServerKind::ALL so the options can't drift from the other
        // surfaces. The vendor builtins always exist, so they aren't offered here.
        // Multiple servers of the same kind are allowed, so these are always shown.
        // Rows are grouped under a category header (the *Model Provider* vertical)
        // so the catalog reads the same way as the README/docs/web surfaces.
        let mut last_category: Option<ModelProviderClass> = None;
        for kind in ModelServerKind::ALL {
            if kind.is_builtin() {
                continue;
            }
            let category = kind.provider_class();
            if last_category != Some(category) {
                rows.push(TreeRow {
                    section_id: SectionId::ModelServers,
                    id: format!("category-{}", category.slug()),
                    depth: 1,
                    label: category.display_name().to_string(),
                    description: String::new(),
                    icon: StatusIcon::Tool,
                    brand_icon: None,
                    is_header: true,
                    actions: ActionSet::none(),
                    health: SectionHealth::Gray,
                });
                last_category = Some(category);
            }
            rows.push(TreeRow {
                section_id: SectionId::ModelServers,
                id: format!("add-{}", kind.slug()),
                depth: 2,
                label: format!("Add {}", kind.display_name()),
                description: kind.connect_blurb().to_string(),
                icon: StatusIcon::Tool,
                brand_icon: kind.brand_icon().map(str::to_string),
                is_header: false,
                actions: ActionSet::primary(StatusAction::ConfigureModelServer {
                    kind: kind.slug().to_string(),
                }),
                health: SectionHealth::Gray,
            });
        }

        rows
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
            operator_inside_wrapper: false,
            operator_version: "0.1.30".into(),
            api_status: RestApiStatus::Stopped,
            kanban_providers: vec![],
            llm_tools: vec![],
            default_llm_tool: None,
            default_llm_model: None,
            delegators: vec![],
            model_servers: servers,
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
        // First row is the declared server; catalog "Add <kind>" rows follow.
        let server_row = &rows[0];
        assert_eq!(server_row.id, "ollama-local");
        assert!(server_row.description.contains("ollama"));
        assert!(server_row.description.contains("localhost:11434"));
    }

    #[test]
    fn test_children_mark_builtin_as_such() {
        let snapshot = snapshot_with_servers(vec![builtin("anthropic-api", "anthropic-api")]);
        let rows = ModelServerSection.children(&snapshot);
        // 1 server row + the addable-kind catalog rows.
        assert!(rows[0].description.contains("builtin"));
    }

    #[test]
    fn test_children_offer_add_rows_for_non_builtin_kinds() {
        let snapshot = snapshot_with_servers(vec![builtin("anthropic-api", "anthropic-api")]);
        let rows = ModelServerSection.children(&snapshot);

        let add_rows: Vec<&TreeRow> = rows.iter().filter(|r| r.id.starts_with("add-")).collect();
        // ollama, openrouter, openai-compat, lmstudio — the addable kinds.
        assert_eq!(add_rows.len(), 4);
        assert!(add_rows
            .iter()
            .any(|r| matches!(&r.actions.primary, StatusAction::ConfigureModelServer { kind } if kind == "ollama")));
        assert!(add_rows
            .iter()
            .any(|r| matches!(&r.actions.primary, StatusAction::ConfigureModelServer { kind } if kind == "openrouter")));
        // No add-row for vendor builtins.
        assert!(!add_rows.iter().any(|r| r.id == "add-anthropic-api"));

        // The addable kinds (all gateways) sit under a "Gateways" header.
        let header = rows
            .iter()
            .find(|r| r.is_header && r.id == "category-gateway")
            .expect("gateway provider-class header present");
        assert_eq!(header.label, "Gateways");
    }

    #[test]
    fn test_configure_model_server_web_url_points_at_setup_page() {
        let action = StatusAction::ConfigureModelServer {
            kind: "ollama".to_string(),
        };
        let url = action.web_url().expect("ollama kind has a setup url");
        assert!(url.starts_with("http"));
    }
}

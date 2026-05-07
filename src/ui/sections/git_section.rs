use crate::ui::status_panel::{
    ActionMeta, ActionSet, SectionHealth, SectionId, StatusAction, StatusIcon, StatusSection,
    StatusSnapshot, TreeRow,
};

pub struct GitSection;

impl StatusSection for GitSection {
    fn section_id(&self) -> SectionId {
        SectionId::Git
    }

    fn label(&self) -> &'static str {
        "Git"
    }

    fn prerequisites(&self) -> &[SectionId] {
        &[SectionId::Connections]
    }

    fn health(&self, snapshot: &StatusSnapshot) -> SectionHealth {
        match (&snapshot.git_provider, snapshot.git_token_set) {
            (Some(_), true) => SectionHealth::Green,
            (Some(_), false) => SectionHealth::Yellow,
            (None, _) => SectionHealth::Red,
        }
    }

    fn description(&self, snapshot: &StatusSnapshot) -> String {
        snapshot
            .git_provider
            .clone()
            .unwrap_or_else(|| "Not configured".into())
    }

    fn children(&self, snapshot: &StatusSnapshot) -> Vec<TreeRow> {
        match &snapshot.git_provider {
            None => {
                vec![
                    TreeRow {
                        section_id: SectionId::Git,
                        depth: 1,
                        label: "Configure GitHub".into(),
                        description: "Set up GitHub".into(),
                        icon: StatusIcon::Plug,
                        is_header: false,
                        actions: ActionSet::primary(StatusAction::ConfigureGitProvider {
                            provider: "github".into(),
                        }),
                        health: SectionHealth::Gray,
                    },
                    TreeRow {
                        section_id: SectionId::Git,
                        depth: 1,
                        label: "Configure GitLab".into(),
                        description: "Set up GitLab".into(),
                        icon: StatusIcon::Plug,
                        is_header: false,
                        actions: ActionSet::primary(StatusAction::ConfigureGitProvider {
                            provider: "gitlab".into(),
                        }),
                        health: SectionHealth::Gray,
                    },
                ]
            }
            Some(provider) => {
                let provider_lower = provider.to_lowercase();

                let mut rows = vec![
                    TreeRow {
                        section_id: SectionId::Git,
                        depth: 1,
                        label: "Provider".into(),
                        description: provider.clone(),
                        icon: StatusIcon::Branch,
                        is_header: false,
                        actions: ActionSet {
                            primary: StatusAction::None,
                            back: StatusAction::None,
                            special: StatusAction::EditFile(snapshot.config_path.clone()),
                            special_meta: Some(ActionMeta {
                                title: "Config",
                                tooltip: "Edit git provider configuration",
                            }),
                            refresh: StatusAction::None,
                            refresh_meta: None,
                        },
                        health: SectionHealth::Gray,
                    },
                    TreeRow {
                        section_id: SectionId::Git,
                        depth: 1,
                        label: "Token".into(),
                        description: if snapshot.git_token_set {
                            "Set".into()
                        } else {
                            "Not set".into()
                        },
                        icon: if snapshot.git_token_set {
                            StatusIcon::Key
                        } else {
                            StatusIcon::Warning
                        },
                        is_header: false,
                        actions: ActionSet::primary(if snapshot.git_token_set {
                            StatusAction::None
                        } else {
                            StatusAction::ConfigureGitProvider {
                                provider: provider_lower,
                            }
                        }),
                        health: SectionHealth::Gray,
                    },
                ];

                if let Some(ref fmt) = snapshot.git_branch_format {
                    rows.push(TreeRow {
                        section_id: SectionId::Git,
                        depth: 1,
                        label: "Branch Format".into(),
                        description: fmt.clone(),
                        icon: StatusIcon::Branch,
                        is_header: false,
                        actions: ActionSet::none(),
                        health: SectionHealth::Gray,
                    });
                }

                rows.push(TreeRow {
                    section_id: SectionId::Git,
                    depth: 1,
                    label: "Worktrees".into(),
                    description: if snapshot.git_use_worktrees {
                        "Enabled".into()
                    } else {
                        "Disabled".into()
                    },
                    icon: StatusIcon::Branch,
                    is_header: false,
                    actions: ActionSet::none(),
                    health: SectionHealth::Gray,
                });

                rows
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backstage::ServerStatus;
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
            backstage_status: ServerStatus::Stopped,
            backstage_display: false,
            kanban_providers: vec![],
            llm_tools: vec![],
            delegators: vec![],
            model_servers: vec![],
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
        }
    }

    #[test]
    fn test_git_health_red_when_no_provider() {
        let section = GitSection;
        let snap = base_snapshot();
        assert_eq!(section.health(&snap), SectionHealth::Red);
    }

    #[test]
    fn test_git_health_yellow_when_provider_no_token() {
        let section = GitSection;
        let mut snap = base_snapshot();
        snap.git_provider = Some("GitHub".into());
        assert_eq!(section.health(&snap), SectionHealth::Yellow);
    }

    #[test]
    fn test_git_health_green_when_provider_and_token() {
        let section = GitSection;
        let mut snap = base_snapshot();
        snap.git_provider = Some("GitHub".into());
        snap.git_token_set = true;
        assert_eq!(section.health(&snap), SectionHealth::Green);
    }

    #[test]
    fn test_git_unconfigured_shows_provider_options() {
        let section = GitSection;
        let snap = base_snapshot();
        let children = section.children(&snap);

        assert_eq!(children.len(), 2);
        assert_eq!(children[0].label, "Configure GitHub");
        assert_eq!(children[0].description, "Set up GitHub");
        assert_eq!(
            children[0].actions.primary,
            StatusAction::ConfigureGitProvider {
                provider: "github".into()
            }
        );
        assert_eq!(children[1].label, "Configure GitLab");
        assert_eq!(children[1].description, "Set up GitLab");
        assert_eq!(
            children[1].actions.primary,
            StatusAction::ConfigureGitProvider {
                provider: "gitlab".into()
            }
        );
    }

    #[test]
    fn test_git_configured_shows_provider_token_worktrees() {
        let section = GitSection;
        let mut snap = base_snapshot();
        snap.git_provider = Some("GitHub".into());
        snap.git_token_set = true;
        let children = section.children(&snap);

        assert_eq!(children[0].label, "Provider");
        assert_eq!(children[0].description, "GitHub");
        assert_eq!(children[1].label, "Token");
        assert_eq!(children[1].description, "Set");
        assert!(matches!(children[1].icon, StatusIcon::Key));
        // Last row is Worktrees (no branch format set)
        let last = children.last().unwrap();
        assert_eq!(last.label, "Worktrees");
    }

    #[test]
    fn test_git_branch_format_shown_when_set() {
        let section = GitSection;
        let mut snap = base_snapshot();
        snap.git_provider = Some("GitHub".into());
        snap.git_branch_format = Some("feature/{ticket}".into());
        let children = section.children(&snap);

        let fmt_row = children.iter().find(|r| r.label == "Branch Format");
        assert!(fmt_row.is_some());
        assert_eq!(fmt_row.unwrap().description, "feature/{ticket}");
    }

    #[test]
    fn test_git_branch_format_hidden_when_unset() {
        let section = GitSection;
        let mut snap = base_snapshot();
        snap.git_provider = Some("GitHub".into());
        snap.git_branch_format = None;
        let children = section.children(&snap);

        assert!(
            !children.iter().any(|r| r.label == "Branch Format"),
            "Branch Format row should be hidden when git_branch_format is None"
        );
    }

    #[test]
    fn test_git_token_clickable_when_not_set() {
        let section = GitSection;
        let mut snap = base_snapshot();
        snap.git_provider = Some("GitHub".into());
        snap.git_token_set = false;
        let children = section.children(&snap);

        let token_row = children.iter().find(|r| r.label == "Token").unwrap();
        assert_eq!(token_row.description, "Not set");
        assert!(matches!(token_row.icon, StatusIcon::Warning));
        assert_eq!(
            token_row.actions.primary,
            StatusAction::ConfigureGitProvider {
                provider: "github".into()
            }
        );
    }

    #[test]
    fn test_git_token_not_clickable_when_set() {
        let section = GitSection;
        let mut snap = base_snapshot();
        snap.git_provider = Some("GitHub".into());
        snap.git_token_set = true;
        let children = section.children(&snap);

        let token_row = children.iter().find(|r| r.label == "Token").unwrap();
        assert_eq!(token_row.description, "Set");
        assert!(matches!(token_row.icon, StatusIcon::Key));
        assert_eq!(token_row.actions.primary, StatusAction::None);
    }
}

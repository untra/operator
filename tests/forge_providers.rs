use operator::config::{Config, GitProviderConfig};
use operator::integrations::{all_integrations, SupportStatus, Vertical};
use operator::types::pr::GitProvider;

#[test]
fn test_forgejo_git_factory_preserves_provider_identity() {
    let config = Config::default();
    let service = operator::api::pr_service::pr_service_for(GitProvider::Forgejo, &config.git)
        .expect("Forgejo must support PR operations");
    assert_eq!(service.provider_name(), "forgejo");
}

#[test]
fn test_forgejo_git_onboarding_uses_its_own_token() {
    let mut config = Config::default();
    operator::services::git_onboarding::configure_git_provider(
        &mut config,
        "forgejo",
        "CUSTOM_FORGEJO_TOKEN",
    )
    .unwrap();
    assert_eq!(config.git.provider, Some(GitProviderConfig::Forgejo));
    assert_eq!(config.git.forgejo.token_env, "CUSTOM_FORGEJO_TOKEN");
    assert_eq!(config.git.gitea.token_env, "GITEA_TOKEN");
}

#[test]
fn test_forge_catalog_maturity_matches_capabilities() {
    let entries = all_integrations();
    for (vertical, slug, status) in [
        (Vertical::Git, "forgejo", SupportStatus::Alpha),
        (Vertical::Git, "gitea", SupportStatus::Alpha),
        (Vertical::Kanban, "gitea", SupportStatus::Alpha),
        (Vertical::Kanban, "forgejo", SupportStatus::Proto),
    ] {
        let entry = entries
            .iter()
            .find(|e| e.vertical == vertical && e.slug == slug)
            .unwrap();
        assert_eq!(entry.status, status);
        assert!(entry.docs_path.is_some());
    }
}

#[test]
fn test_forge_project_config_roundtrip_and_proto_rejection() {
    let raw = r#"
[gitea.work]
base_url = "https://git.example"
repositories = ["team/repo"]
[forgejo.work]
base_url = "https://codeberg.org"
"#;
    let config = Config {
        kanban: toml::from_str(raw).unwrap(),
        ..Default::default()
    };
    assert!(!config.kanban.gitea["work"].enabled);
    assert_eq!(
        config.kanban.gitea["work"].api_key_env,
        "OPERATOR_GITEA_TOKEN"
    );
    assert_eq!(
        config.kanban.forgejo["work"].api_key_env,
        "OPERATOR_FORGEJO_TOKEN"
    );
    let mut restored: Config = toml::from_str(&toml::to_string(&config).unwrap()).unwrap();
    restored.kanban.validate_forge_projects().unwrap();
    restored.kanban.forgejo.get_mut("work").unwrap().enabled = true;
    assert!(restored
        .kanban
        .validate_forge_projects()
        .unwrap_err()
        .to_string()
        .contains("Proto"));
}

#[test]
fn test_gitea_repeated_import_and_cross_board_card_do_not_duplicate_tickets() {
    use operator::api::providers::kanban::ExternalIssue;
    use operator::config::ProjectSyncConfig;
    use operator::services::kanban_sync::KanbanSyncService;
    let root = tempfile::tempdir().unwrap();
    let mut config = Config::default();
    config.paths.tickets = root.path().to_string_lossy().into_owned();
    let service = KanbanSyncService::new(&config);
    let issue: ExternalIssue = serde_json::from_value(serde_json::json!({
        "id": "https://git.example/team/repo/issues/7",
        "key": "https://git.example/team/repo/issues/7",
        "summary": "Imported task", "description": "details", "kanban_issue_types": [],
        "status": "Todo", "url": "https://git.example/team/repo/issues/7"
    }))
    .unwrap();
    let sync = ProjectSyncConfig {
        ticket_project: Some("repo".into()),
        ..Default::default()
    };
    let first = service.apply_issues(
        "gitea",
        "https://git.example/team/repo/projects/1",
        &sync,
        vec![issue.clone()],
    );
    assert_eq!(first.created.len(), 1, "{:?}", first.errors);
    let again = service.apply_issues(
        "gitea",
        "https://git.example/team/repo/projects/2",
        &sync,
        vec![issue],
    );
    assert!(again.created.is_empty());
    assert_eq!(again.skipped.len(), 1);
    assert_eq!(
        std::fs::read_dir(root.path().join("queue"))
            .unwrap()
            .count(),
        1
    );
}

#[test]
fn test_forgejo_git_hosts_are_independent_from_gitea() {
    use operator::api::gitea_service::GiteaService;
    use operator::config::ForgejoConfig;
    let default = GiteaService::forgejo(ForgejoConfig::default());
    assert_eq!(default.base_url().unwrap().host_str(), Some("codeberg.org"));
    let configured = GiteaService::forgejo(ForgejoConfig {
        host: Some("https://forge.example".into()),
        ..Default::default()
    });
    assert_eq!(
        configured.base_url().unwrap().as_str(),
        "https://forge.example/"
    );
}

#[test]
fn test_gitea_factory_never_falls_back_to_a_different_instance() {
    use operator::api::providers::kanban::get_provider_from_config;
    let config = Config {
        kanban: toml::from_str(
            r#"
[gitea.work]
enabled = true
base_url = "https://first.example"
repositories = ["team/repo"]
[gitea.work.projects."https://first.example/team/repo/projects/1"]
sync_user_id = "4"
"#,
        )
        .unwrap(),
        ..Default::default()
    };
    assert!(get_provider_from_config(
        &config.kanban,
        "gitea",
        "https://second.example/team/repo/projects/1"
    )
    .is_err());
    assert!(get_provider_from_config(&config.kanban, "forgejo", "1").is_err());
}

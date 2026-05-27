use super::*;

#[test]
fn test_default_preset_is_dev_kanban() {
    assert_eq!(CollectionPreset::default(), CollectionPreset::DevKanban);
}

#[test]
fn test_templates_config_default_uses_dev_kanban() {
    let config = TemplatesConfig::default();
    assert_eq!(config.preset, CollectionPreset::DevKanban);
}

#[test]
fn test_dev_kanban_has_three_issue_types() {
    let types = CollectionPreset::DevKanban.issue_types();
    assert_eq!(types.len(), 3);
    assert!(types.contains(&"TASK".to_string()));
    assert!(types.contains(&"FEAT".to_string()));
    assert!(types.contains(&"FIX".to_string()));
}

#[test]
fn test_delegator_serde_roundtrip() {
    let delegator = Delegator {
        name: "claude-opus-auto".to_string(),
        llm_tool: "claude".to_string(),
        model: "opus".to_string(),
        display_name: Some("Claude Opus Auto".to_string()),
        model_properties: std::collections::HashMap::new(),
        launch_config: Some(DelegatorLaunchConfig {
            yolo: true,
            permission_mode: Some("delegate".to_string()),
            flags: vec!["--verbose".to_string()],
            ..Default::default()
        }),
        model_server: None,
    };

    let json = serde_json::to_string(&delegator).unwrap();
    let parsed: Delegator = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.name, "claude-opus-auto");
    assert_eq!(parsed.llm_tool, "claude");
    assert_eq!(parsed.model, "opus");
    assert!(parsed.launch_config.unwrap().yolo);
    assert!(parsed.model_server.is_none());
}

#[test]
fn test_model_server_toml_roundtrip() {
    let toml_str = r#"
        name = "ollama-local"
        kind = "ollama"
        base_url = "http://localhost:11434"
        display_name = "Ollama (local)"
    "#;
    let server: ModelServer = toml::from_str(toml_str).unwrap();
    assert_eq!(server.name, "ollama-local");
    assert_eq!(server.kind, "ollama");
    assert_eq!(server.base_url.as_deref(), Some("http://localhost:11434"));
    assert_eq!(server.display_name.as_deref(), Some("Ollama (local)"));
    assert!(server.extra_env.is_empty());
    assert!(server.api_key_env.is_none());
}

#[test]
fn test_delegator_with_model_server_ref_roundtrip() {
    let toml_str = r#"
        name = "codex-local-qwen"
        llm_tool = "codex"
        model = "qwen2.5-coder"
        model_server = "ollama-local"
    "#;
    let d: Delegator = toml::from_str(toml_str).unwrap();
    assert_eq!(d.name, "codex-local-qwen");
    assert_eq!(d.model_server.as_deref(), Some("ollama-local"));
}

#[test]
fn test_delegator_without_model_server_field_still_parses() {
    let toml_str = r#"
        name = "claude-opus-auto"
        llm_tool = "claude"
        model = "opus"
    "#;
    let d: Delegator = toml::from_str(toml_str).unwrap();
    assert_eq!(d.name, "claude-opus-auto");
    assert!(d.model_server.is_none());
}

#[test]
fn test_implicit_model_server_for_known_tools() {
    assert_eq!(
        implicit_model_server_for_tool("claude").kind,
        "anthropic-api"
    );
    assert_eq!(implicit_model_server_for_tool("codex").kind, "openai-api");
    assert_eq!(implicit_model_server_for_tool("gemini").kind, "google-api");
    assert_eq!(implicit_model_server_for_tool("unknown").kind, "openai-api");
}

#[test]
fn test_config_without_model_servers_field_still_parses() {
    let toml_str = r#"
        [agents]
        max_parallel = 1
        cores_reserved = 0
        health_check_interval = 5
        [notifications]
        enabled = false
        [queue]
        auto_assign = true
        priority_order = []
        poll_interval_ms = 1000
        [paths]
        tickets = ".tickets"
        projects = "."
        state = ".tickets/operator"
        worktrees = ".worktrees"
        [ui]
        refresh_rate_ms = 100
        completed_history_hours = 1
        summary_max_length = 40
        [launch]
        confirm_autonomous = false
        confirm_paired = false
        launch_delay_ms = 0
        [templates]
    "#;
    let cfg: Config = toml::from_str(toml_str).unwrap();
    assert!(cfg.model_servers.is_empty());
}

#[test]
fn test_skill_directories_override_default() {
    let override_config = SkillDirectoriesOverride::default();
    assert!(override_config.global.is_empty());
    assert!(override_config.project.is_empty());
}

#[test]
fn test_session_wrapper_type_cmux_display() {
    assert_eq!(SessionWrapperType::Cmux.to_string(), "cmux");
}

#[test]
fn test_session_wrapper_type_cmux_serde_roundtrip() {
    let json = serde_json::to_string(&SessionWrapperType::Cmux).unwrap();
    let parsed: SessionWrapperType = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, SessionWrapperType::Cmux);
}

#[test]
fn test_sessions_cmux_config_defaults() {
    let config = SessionsCmuxConfig::default();
    assert_eq!(
        config.binary_path,
        "/Applications/cmux.app/Contents/Resources/bin/cmux"
    );
    assert!(config.require_in_cmux);
    assert_eq!(config.placement, CmuxPlacementPolicy::Auto);
}

#[test]
fn test_cmux_placement_policy_display() {
    assert_eq!(CmuxPlacementPolicy::Auto.to_string(), "auto");
    assert_eq!(CmuxPlacementPolicy::Workspace.to_string(), "workspace");
    assert_eq!(CmuxPlacementPolicy::Window.to_string(), "window");
}

#[test]
fn test_config_deserialize_with_cmux_wrapper() {
    let toml_str = r#"
        wrapper = "cmux"
        [cmux]
        binary_path = "/usr/local/bin/cmux"
        require_in_cmux = false
        placement = "window"
    "#;
    let config: SessionsConfig = toml::from_str(toml_str).unwrap();
    assert_eq!(config.wrapper, SessionWrapperType::Cmux);
    assert_eq!(config.cmux.binary_path, "/usr/local/bin/cmux");
    assert!(!config.cmux.require_in_cmux);
    assert_eq!(config.cmux.placement, CmuxPlacementPolicy::Window);
}

#[test]
fn test_devops_kanban_has_five_issue_types() {
    let types = CollectionPreset::DevopsKanban.issue_types();
    assert_eq!(types.len(), 5);
    assert!(types.contains(&"TASK".to_string()));
    assert!(types.contains(&"FEAT".to_string()));
    assert!(types.contains(&"FIX".to_string()));
    assert!(types.contains(&"SPIKE".to_string()));
    assert!(types.contains(&"INV".to_string()));
}

// --- effective_max_agents tests ---

#[test]
fn test_effective_max_agents_never_returns_zero() {
    let mut config = Config::default();
    config.agents.max_parallel = 0;
    config.agents.cores_reserved = 100;
    assert!(config.effective_max_agents() >= 1);
}

#[test]
fn test_effective_max_agents_respects_max_parallel() {
    let mut config = Config::default();
    config.agents.max_parallel = 2;
    config.agents.cores_reserved = 0;
    assert!(config.effective_max_agents() <= 2);
}

#[test]
fn test_effective_max_agents_reserves_cores() {
    let config = Config::default();
    let cpu_count = sysinfo::System::new_all().cpus().len();
    let effective = config.effective_max_agents();
    assert!(effective <= cpu_count.saturating_sub(config.agents.cores_reserved));
}

// --- effective_max_agents_per_repo tests ---

#[test]
fn test_effective_max_agents_per_repo_default() {
    let config = Config::default();
    assert_eq!(config.effective_max_agents_per_repo(), 1);
}

#[test]
fn test_effective_max_agents_per_repo_clamps_zero() {
    let mut config = Config::default();
    config.agents.max_agents_per_repo = 0;
    assert_eq!(config.effective_max_agents_per_repo(), 1);
}

#[test]
fn test_effective_max_agents_per_repo_custom() {
    let mut config = Config::default();
    config.agents.max_agents_per_repo = 3;
    assert_eq!(config.effective_max_agents_per_repo(), 3);
}

// --- Path resolution tests ---

#[test]
fn test_tickets_path_absolute_passthrough() {
    let mut config = Config::default();
    config.paths.tickets = "/absolute/path/tickets".to_string();
    assert_eq!(
        config.tickets_path(),
        std::path::PathBuf::from("/absolute/path/tickets")
    );
}

#[test]
fn test_tickets_path_relative_resolves() {
    let config = Config::default();
    let path = config.tickets_path();
    assert!(path.is_absolute());
    assert!(path.ends_with(".tickets"));
}

#[test]
fn test_projects_path_absolute_passthrough() {
    let mut config = Config::default();
    config.paths.projects = "/my/projects".to_string();
    assert_eq!(
        config.projects_path(),
        std::path::PathBuf::from("/my/projects")
    );
}

#[test]
fn test_state_path_relative_resolves() {
    let config = Config::default();
    let path = config.state_path();
    assert!(path.is_absolute());
    assert!(path.ends_with("operator"));
}

// --- priority_index tests ---

#[test]
fn test_priority_index_known_types() {
    let config = Config::default();
    assert_eq!(config.priority_index("INV"), 0);
    assert_eq!(config.priority_index("FIX"), 1);
    assert_eq!(config.priority_index("TASK"), 2);
    assert_eq!(config.priority_index("FEAT"), 3);
    assert_eq!(config.priority_index("SPIKE"), 4);
}

#[test]
fn test_priority_index_unknown_returns_max() {
    let config = Config::default();
    assert_eq!(config.priority_index("UNKNOWN"), usize::MAX);
}

#[test]
fn test_priority_index_empty_order() {
    let mut config = Config::default();
    config.queue.priority_order.clear();
    assert_eq!(config.priority_index("INV"), usize::MAX);
}

#[test]
fn test_upsert_jira_project_inserts_new_workspace() {
    let mut kanban = KanbanConfig::default();
    kanban.upsert_jira_project(
        "acme.atlassian.net",
        "user@acme.com",
        "OPERATOR_JIRA_API_KEY",
        "PROJ",
        "acct-123",
    );

    let ws = kanban
        .jira
        .get("acme.atlassian.net")
        .expect("workspace should be inserted");
    assert!(ws.enabled);
    assert_eq!(ws.email, "user@acme.com");
    assert_eq!(ws.api_key_env, "OPERATOR_JIRA_API_KEY");

    let project = ws.projects.get("PROJ").expect("project should exist");
    assert_eq!(project.sync_user_id, "acct-123");
}

#[test]
fn test_upsert_jira_project_adds_to_existing_workspace_without_clobber() {
    let mut kanban = KanbanConfig::default();
    // Seed with an existing workspace and project
    kanban.upsert_jira_project(
        "acme.atlassian.net",
        "user@acme.com",
        "OPERATOR_JIRA_API_KEY",
        "EXISTING",
        "acct-existing",
    );

    // Add a second project to the same workspace
    kanban.upsert_jira_project(
        "acme.atlassian.net",
        "user@acme.com",
        "OPERATOR_JIRA_API_KEY",
        "NEWONE",
        "acct-new",
    );

    let ws = kanban.jira.get("acme.atlassian.net").unwrap();
    assert_eq!(ws.projects.len(), 2, "both projects should be preserved");
    assert_eq!(ws.projects["EXISTING"].sync_user_id, "acct-existing");
    assert_eq!(ws.projects["NEWONE"].sync_user_id, "acct-new");
}

#[test]
fn test_upsert_jira_project_replaces_existing_project_entry() {
    let mut kanban = KanbanConfig::default();
    kanban.upsert_jira_project(
        "acme.atlassian.net",
        "user@acme.com",
        "OPERATOR_JIRA_API_KEY",
        "PROJ",
        "acct-old",
    );
    // Upsert same project with new sync_user_id
    kanban.upsert_jira_project(
        "acme.atlassian.net",
        "user@acme.com",
        "OPERATOR_JIRA_API_KEY",
        "PROJ",
        "acct-new",
    );

    let ws = kanban.jira.get("acme.atlassian.net").unwrap();
    assert_eq!(ws.projects.len(), 1);
    assert_eq!(ws.projects["PROJ"].sync_user_id, "acct-new");
}

#[test]
fn test_upsert_linear_project_inserts_new_workspace() {
    let mut kanban = KanbanConfig::default();
    kanban.upsert_linear_project(
        "myworkspace",
        "OPERATOR_LINEAR_API_KEY",
        "ENG",
        "user-uuid-1",
    );

    let ws = kanban.linear.get("myworkspace").unwrap();
    assert!(ws.enabled);
    assert_eq!(ws.api_key_env, "OPERATOR_LINEAR_API_KEY");
    assert_eq!(ws.projects["ENG"].sync_user_id, "user-uuid-1");
}

#[test]
fn test_upsert_linear_project_adds_to_existing_workspace_without_clobber() {
    let mut kanban = KanbanConfig::default();
    kanban.upsert_linear_project("myworkspace", "OPERATOR_LINEAR_API_KEY", "ENG", "user-a");
    kanban.upsert_linear_project("myworkspace", "OPERATOR_LINEAR_API_KEY", "DESIGN", "user-b");

    let ws = kanban.linear.get("myworkspace").unwrap();
    assert_eq!(ws.projects.len(), 2);
    assert_eq!(ws.projects["ENG"].sync_user_id, "user-a");
    assert_eq!(ws.projects["DESIGN"].sync_user_id, "user-b");
}

#[test]
fn test_upsert_jira_does_not_touch_other_workspaces() {
    let mut kanban = KanbanConfig::default();
    kanban.upsert_jira_project(
        "first.atlassian.net",
        "u1@first.com",
        "OPERATOR_JIRA_API_KEY",
        "FIRST",
        "acct-1",
    );
    kanban.upsert_jira_project(
        "second.atlassian.net",
        "u2@second.com",
        "OPERATOR_JIRA_SECOND_API_KEY",
        "SECOND",
        "acct-2",
    );

    assert_eq!(kanban.jira.len(), 2);
    assert_eq!(kanban.jira["first.atlassian.net"].email, "u1@first.com");
    assert_eq!(
        kanban.jira["second.atlassian.net"].api_key_env,
        "OPERATOR_JIRA_SECOND_API_KEY"
    );
}

#[test]
fn test_upsert_project_jira() {
    use crate::api::providers::kanban::{
        DiscoveredProject, KanbanProviderType, ValidatedWorkspace, WorkspaceExtra,
    };

    let mut kanban = KanbanConfig::default();
    let ws = ValidatedWorkspace {
        provider_kind: KanbanProviderType::Jira,
        workspace_key: "acme.atlassian.net".to_string(),
        workspace_display_name: "Acme Corp".to_string(),
        sync_user_id: "acct-123".to_string(),
        sync_user_display_name: "Alice".to_string(),
        api_key_env: "OPERATOR_JIRA_API_KEY".to_string(),
        prefetched_projects: None,
        extra: WorkspaceExtra::Jira {
            email: "alice@acme.com".to_string(),
        },
    };
    let project = DiscoveredProject {
        workspace_key: "acme.atlassian.net".to_string(),
        project_key: "PROJ".to_string(),
        project_display_name: "My Project".to_string(),
        provider_url: None,
        provider_native_id: None,
    };

    kanban.upsert_project(&ws, &project);

    let entry = kanban
        .jira
        .get("acme.atlassian.net")
        .expect("workspace should be created");
    assert!(entry.enabled);
    assert_eq!(entry.email, "alice@acme.com");
    assert_eq!(entry.api_key_env, "OPERATOR_JIRA_API_KEY");
    let proj = entry.projects.get("PROJ").expect("project should exist");
    assert_eq!(proj.sync_user_id, "acct-123");
}

#[test]
fn test_upsert_project_linear() {
    use crate::api::providers::kanban::{
        DiscoveredProject, KanbanProviderType, ValidatedWorkspace, WorkspaceExtra,
    };

    let mut kanban = KanbanConfig::default();
    let ws = ValidatedWorkspace {
        provider_kind: KanbanProviderType::Linear,
        workspace_key: "acme".to_string(),
        workspace_display_name: "Acme Inc".to_string(),
        sync_user_id: "user-uuid-1".to_string(),
        sync_user_display_name: "Bob".to_string(),
        api_key_env: "OPERATOR_LINEAR_API_KEY".to_string(),
        prefetched_projects: None,
        extra: WorkspaceExtra::Linear,
    };
    let project = DiscoveredProject {
        workspace_key: "acme".to_string(),
        project_key: "ENG".to_string(),
        project_display_name: "Engineering".to_string(),
        provider_url: None,
        provider_native_id: None,
    };

    kanban.upsert_project(&ws, &project);

    let entry = kanban
        .linear
        .get("acme")
        .expect("workspace should be created");
    assert!(entry.enabled);
    assert_eq!(entry.api_key_env, "OPERATOR_LINEAR_API_KEY");
    let proj = entry.projects.get("ENG").expect("project should exist");
    assert_eq!(proj.sync_user_id, "user-uuid-1");
}

#[test]
fn test_upsert_project_github() {
    use crate::api::providers::kanban::{
        DiscoveredProject, KanbanProviderType, ValidatedWorkspace, WorkspaceExtra,
    };

    let mut kanban = KanbanConfig::default();
    let ws = ValidatedWorkspace {
        provider_kind: KanbanProviderType::Github,
        workspace_key: "my-org".to_string(),
        workspace_display_name: "github.com".to_string(),
        sync_user_id: "12345678".to_string(),
        sync_user_display_name: "octocat".to_string(),
        api_key_env: "OPERATOR_GITHUB_TOKEN".to_string(),
        prefetched_projects: None,
        extra: WorkspaceExtra::Github,
    };
    let project = DiscoveredProject {
        workspace_key: "my-org".to_string(),
        project_key: "PVT_abc".to_string(),
        project_display_name: "My Board".to_string(),
        provider_url: None,
        provider_native_id: None,
    };

    kanban.upsert_project(&ws, &project);

    let entry = kanban
        .github
        .get("my-org")
        .expect("workspace should be created");
    assert!(entry.enabled);
    assert_eq!(entry.api_key_env, "OPERATOR_GITHUB_TOKEN");
    let proj = entry.projects.get("PVT_abc").expect("project should exist");
    assert_eq!(proj.sync_user_id, "12345678");
}

#[test]
fn test_relay_config_default_auto_inject_is_false() {
    let config = Config::default();
    assert!(!config.relay.auto_inject_mcp);
}

// --- ExternalMcpServer tests ---

#[test]
fn test_mcp_external_servers_defaults_to_empty() {
    let config = McpConfig::default();
    assert!(config.external_servers.is_empty());
}

#[test]
fn test_mcp_config_without_external_servers_still_parses() {
    let toml_str = r"
        http_enabled = true
        stdio_advertised = false
        expose_ticket_write_tools = true
    ";
    let config: McpConfig = toml::from_str(toml_str).unwrap();
    assert!(config.http_enabled);
    assert!(!config.stdio_advertised);
    assert!(config.expose_ticket_write_tools);
    assert!(config.external_servers.is_empty());
}

#[test]
fn test_external_mcp_server_static_config_roundtrip() {
    let toml_str = r#"
        [[external_servers]]
        name = "my-tools"
        command = "/usr/local/bin/my-mcp-server"
        args = ["--stdio"]
        env = { API_KEY = "${MY_TOOLS_API_KEY}" }
    "#;
    let config: McpConfig = toml::from_str(toml_str).unwrap();
    assert_eq!(config.external_servers.len(), 1);
    let server = &config.external_servers[0];
    assert_eq!(server.name, "my-tools");
    assert_eq!(server.command, "/usr/local/bin/my-mcp-server");
    assert_eq!(server.args, vec!["--stdio"]);
    assert_eq!(server.env.get("API_KEY").unwrap(), "${MY_TOOLS_API_KEY}");
    assert!(server.enabled);
    assert!(server.discover_from.is_none());
}

#[test]
fn test_external_mcp_server_sidecar_config_roundtrip() {
    let toml_str = r#"
        [[external_servers]]
        name = "kanbots"
        command = ""
        discover_from = ".kanbots/active-session.json"
    "#;
    let config: McpConfig = toml::from_str(toml_str).unwrap();
    assert_eq!(config.external_servers.len(), 1);
    let server = &config.external_servers[0];
    assert_eq!(server.name, "kanbots");
    assert_eq!(server.command, "");
    assert_eq!(
        server.discover_from.as_deref(),
        Some(".kanbots/active-session.json")
    );
    assert!(server.enabled);
}

#[test]
fn test_external_mcp_server_disabled() {
    let toml_str = r#"
        [[external_servers]]
        name = "disabled-server"
        command = "some-binary"
        enabled = false
    "#;
    let config: McpConfig = toml::from_str(toml_str).unwrap();
    assert!(!config.external_servers[0].enabled);
}

#[test]
fn test_external_mcp_server_multiple_servers() {
    let toml_str = r#"
        [[external_servers]]
        name = "kanbots"
        command = ""
        discover_from = ".kanbots/active-session.json"

        [[external_servers]]
        name = "my-tools"
        command = "/usr/local/bin/my-mcp"
        args = ["--port", "9090"]
    "#;
    let config: McpConfig = toml::from_str(toml_str).unwrap();
    assert_eq!(config.external_servers.len(), 2);
    assert_eq!(config.external_servers[0].name, "kanbots");
    assert_eq!(config.external_servers[1].name, "my-tools");
}

#[test]
fn test_external_mcp_server_json_serde_roundtrip() {
    let server = ExternalMcpServer {
        name: "test".to_string(),
        command: "/bin/test".to_string(),
        args: vec!["--flag".to_string()],
        env: std::collections::HashMap::from([("KEY".to_string(), "val".to_string())]),
        enabled: true,
        discover_from: Some("/tmp/sidecar.json".to_string()),
    };
    let json = serde_json::to_string(&server).unwrap();
    let parsed: ExternalMcpServer = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.name, "test");
    assert_eq!(parsed.command, "/bin/test");
    assert_eq!(parsed.args, vec!["--flag"]);
    assert_eq!(parsed.env.get("KEY").unwrap(), "val");
    assert!(parsed.enabled);
    assert_eq!(parsed.discover_from.as_deref(), Some("/tmp/sidecar.json"));
}

#[test]
fn test_delegator_launch_config_operator_relay_defaults_to_none() {
    let toml_str = r#"
        name = "test-delegator"
        llm_tool = "claude"
        model = "opus"
        [launch_config]
        yolo = false
    "#;
    let d: Delegator = toml::from_str(toml_str).unwrap();
    assert!(d.launch_config.as_ref().unwrap().operator_relay.is_none());
}

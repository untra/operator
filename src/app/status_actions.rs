use anyhow::Result;

use crate::config::SessionWrapperType;
use crate::ui::status_panel::StatusAction;
use crate::ui::with_suspended_tui;

use super::git_onboarding;
use super::{App, AppTerminal};

/// Open a URL in the default browser.
pub(super) fn open_in_browser(url: &str) -> std::io::Result<()> {
    let opener = if cfg!(target_os = "macos") {
        "open"
    } else if cfg!(target_os = "windows") {
        "cmd"
    } else {
        "xdg-open"
    };

    if cfg!(target_os = "windows") {
        std::process::Command::new(opener)
            .args(["/C", "start", url])
            .spawn()?;
    } else {
        std::process::Command::new(opener).arg(url).spawn()?;
    }
    Ok(())
}

impl App {
    /// Execute an action from the status panel.
    pub(super) fn execute_status_action(
        &mut self,
        action: StatusAction,
        terminal: &mut AppTerminal,
    ) -> Result<()> {
        match action {
            StatusAction::ToggleSection(_) => {
                // Already handled by dashboard.status_action()
            }
            StatusAction::OpenDirectory(path) => {
                if let Err(e) = open_in_browser(&path) {
                    self.dashboard.set_status(&format!("Failed to open: {e}"));
                }
            }
            StatusAction::EditFile(path) => {
                let cmd = self.dashboard.editor_config.file_editor().to_string();
                with_suspended_tui(terminal, || {
                    let (prog, args) = crate::editors::EditorConfig::split_command(&cmd);
                    let result = std::process::Command::new(prog)
                        .args(&args)
                        .arg(&path)
                        .status();
                    if let Err(e) = result {
                        tracing::warn!("Failed to open editor: {}", e);
                    }
                    Ok(())
                })?;
            }
            StatusAction::OpenUrl(url) => {
                if let Err(e) = open_in_browser(&url) {
                    self.dashboard
                        .set_status(&format!("Failed to open URL: {e}"));
                }
            }
            StatusAction::StartApi => {
                if !self.rest_api_server.is_running() {
                    if let Err(e) = self.rest_api_server.start() {
                        self.dashboard
                            .set_status(&format!("Failed to start API: {e}"));
                    } else {
                        self.dashboard.set_status("Starting API server...");
                    }
                }
            }
            StatusAction::OpenSwagger { port } => {
                let url = format!("http://localhost:{port}/swagger-ui/");
                if let Err(e) = open_in_browser(&url) {
                    self.dashboard
                        .set_status(&format!("Failed to open Swagger: {e}"));
                }
            }
            StatusAction::RestartWrapperConnection => {
                self.restart_wrapper_connection();
            }
            StatusAction::ToggleBackstage => {
                self.toggle_backstage(terminal)?;
            }
            StatusAction::OpenWebUi { port } => {
                let url = format!("http://localhost:{port}/");
                if let Err(e) = open_in_browser(&url) {
                    self.dashboard
                        .set_status(&format!("Failed to open web UI: {e}"));
                }
            }
            StatusAction::OpenWebUiAt { port, route } => {
                let url = format!("http://localhost:{port}/#{route}");
                if let Err(e) = open_in_browser(&url) {
                    self.dashboard
                        .set_status(&format!("Failed to open web UI: {e}"));
                }
            }
            StatusAction::SetDefaultLlm { tool_name, model } => {
                self.set_default_llm(&tool_name, &model);
            }
            StatusAction::ConfigureKanbanProvider { provider } => {
                let url = match provider.as_str() {
                    "jira" => "https://id.atlassian.com/manage-profile/security/api-tokens",
                    "linear" => "https://linear.app/settings/api",
                    _ => return Ok(()),
                };
                if let Err(e) = open_in_browser(url) {
                    self.dashboard
                        .set_status(&format!("Failed to open {provider} setup: {e}"));
                } else {
                    self.dashboard.set_status(&format!(
                        "Opened {provider} API key page — add credentials to config.toml"
                    ));
                }
            }
            StatusAction::ConfigureGitProvider { provider } => {
                match git_onboarding::resolve_onboarding(&provider) {
                    Some(git_onboarding::OnboardingStep::InstallCli {
                        install_url,
                        provider_display,
                    }) => {
                        if let Err(e) = open_in_browser(&install_url) {
                            self.dashboard.set_status(&format!(
                                "Failed to open {provider_display} setup: {e}"
                            ));
                        } else {
                            self.dashboard
                                .set_status(&format!("Opened {provider_display} CLI install page"));
                        }
                    }
                    Some(git_onboarding::OnboardingStep::CollectToken {
                        pat_url,
                        provider,
                        provider_display,
                        placeholder,
                    }) => {
                        let _ = open_in_browser(&pat_url);
                        self.git_token_dialog.show(
                            &provider,
                            &provider_display,
                            &pat_url,
                            &placeholder,
                        );
                    }
                    Some(git_onboarding::OnboardingStep::AutoConfigured {
                        username,
                        token,
                        provider,
                        provider_display,
                    }) => {
                        match git_onboarding::complete_git_onboarding(
                            &mut self.config,
                            &provider,
                            &token,
                        ) {
                            Ok(()) => {
                                self.dashboard.update_config(&self.config);
                                self.refresh_data()?;
                                self.dashboard.set_status(&format!(
                                    "{provider_display} connected as {username}"
                                ));
                            }
                            Err(e) => {
                                self.dashboard.set_status(&format!("Git setup failed: {e}"));
                            }
                        }
                    }
                    None => {
                        self.dashboard.set_status("Unsupported git provider");
                    }
                }
            }
            StatusAction::RefreshSection(_section_id) => {
                self.refresh_data()?;
            }
            StatusAction::ResetConfig => {
                // TODO: implement double-confirm dialog (type working dir name to confirm)
                self.dashboard
                    .set_status("Config reset requires confirmation — not yet implemented");
            }
            StatusAction::ReloadConfig => match crate::config::Config::load(None) {
                Ok(new_config) => {
                    self.config = new_config;
                    self.dashboard.update_config(&self.config);
                    self.refresh_data()?;
                    self.dashboard.set_status("Configuration reloaded");
                }
                Err(e) => {
                    self.dashboard
                        .set_status(&format!("Failed to reload config: {e}"));
                }
            },
            StatusAction::ToggleMcpHttp => {
                self.config.mcp.http_enabled = !self.config.mcp.http_enabled;
                self.dashboard.update_config(&self.config);
                self.dashboard.set_status(if self.config.mcp.http_enabled {
                    "MCP HTTP enabled — restart the API to mount routes"
                } else {
                    "MCP HTTP disabled — restart the API to unmount routes"
                });
            }
            StatusAction::WriteAndOpenMcpClientConfig { client } => {
                let cwd = std::env::current_dir().unwrap_or_default();
                let Some(snippet) = crate::mcp::client_configs::snippet_for(&client, &cwd) else {
                    self.dashboard
                        .set_status(&format!("Unknown MCP client: {client}"));
                    return Ok(());
                };
                let dir = self.config.tickets_path().join("operator/mcp");
                if let Err(e) = std::fs::create_dir_all(&dir) {
                    self.dashboard
                        .set_status(&format!("Failed to create {}: {e}", dir.display()));
                    return Ok(());
                }
                let path = dir.join(format!("{client}.json"));
                let body = serde_json::to_string_pretty(&snippet).unwrap_or_default();
                if let Err(e) = std::fs::write(&path, body) {
                    self.dashboard
                        .set_status(&format!("Failed to write {}: {e}", path.display()));
                    return Ok(());
                }
                let cmd = self.dashboard.editor_config.file_editor().to_string();
                with_suspended_tui(terminal, || {
                    let (prog, args) = crate::editors::EditorConfig::split_command(&cmd);
                    let result = std::process::Command::new(prog)
                        .args(&args)
                        .arg(&path)
                        .status();
                    if let Err(e) = result {
                        tracing::warn!("Failed to open editor: {}", e);
                    }
                    Ok(())
                })?;
            }
            StatusAction::OpenMcpDocs => {
                if let Err(e) = open_in_browser("https://operator.untra.io/mcp/") {
                    self.dashboard
                        .set_status(&format!("Failed to open MCP docs: {e}"));
                }
            }
            StatusAction::WriteAndOpenAcpEditorConfig { editor } => {
                let Some(snippet) = crate::acp::client_configs::snippet_for(&editor) else {
                    self.dashboard
                        .set_status(&format!("Unknown ACP editor: {editor}"));
                    return Ok(());
                };
                let dir = self.config.tickets_path().join("operator/acp");
                if let Err(e) = std::fs::create_dir_all(&dir) {
                    self.dashboard
                        .set_status(&format!("Failed to create {}: {e}", dir.display()));
                    return Ok(());
                }
                // Text-format editors (emacs elisp, kiro TOML) deserialise into
                // a JSON string; everything else is a structured Value.
                let (extension, body) = match snippet {
                    serde_json::Value::String(s) => {
                        let ext = if editor == "emacs" { "el" } else { "toml" };
                        (ext, s)
                    }
                    other => (
                        "json",
                        serde_json::to_string_pretty(&other).unwrap_or_default(),
                    ),
                };
                let path = dir.join(format!("{editor}.{extension}"));
                if let Err(e) = std::fs::write(&path, body) {
                    self.dashboard
                        .set_status(&format!("Failed to write {}: {e}", path.display()));
                    return Ok(());
                }
                let cmd = self.dashboard.editor_config.file_editor().to_string();
                with_suspended_tui(terminal, || {
                    let (prog, args) = crate::editors::EditorConfig::split_command(&cmd);
                    let result = std::process::Command::new(prog)
                        .args(&args)
                        .arg(&path)
                        .status();
                    if let Err(e) = result {
                        tracing::warn!("Failed to open editor: {}", e);
                    }
                    Ok(())
                })?;
            }
            StatusAction::OpenAcpDocs => {
                if let Err(e) = open_in_browser("https://operator.untra.io/acp/") {
                    self.dashboard
                        .set_status(&format!("Failed to open ACP docs: {e}"));
                }
            }
            StatusAction::None => {}
        }
        Ok(())
    }

    /// Attempt to restart the session wrapper connection.
    /// After attempting restart, immediately re-checks connection status
    /// so the UI reflects the result without waiting for the next periodic refresh.
    fn restart_wrapper_connection(&mut self) {
        match self.config.sessions.wrapper {
            SessionWrapperType::Tmux => {
                let socket = &self.config.sessions.tmux.socket_name;
                match std::process::Command::new("tmux")
                    .args(["-L", socket, "start-server"])
                    .status()
                {
                    Ok(status) if status.success() => {
                        // Re-check connection status immediately
                        let wrapper_status = self.check_tmux_status();
                        self.dashboard
                            .update_wrapper_connection_status(wrapper_status.clone());
                        if wrapper_status.is_connected() {
                            self.dashboard.set_status("tmux server connected");
                        } else {
                            self.dashboard
                                .set_status("tmux server started (no sessions)");
                        }
                    }
                    Ok(_) => {
                        self.dashboard.set_status("Failed to start tmux server");
                    }
                    Err(e) => {
                        self.dashboard.set_status(&format!("tmux not found: {e}"));
                    }
                }
            }
            SessionWrapperType::Vscode => {
                self.dashboard
                    .set_status("Webhook managed by VS Code extension");
            }
            SessionWrapperType::Cmux => {
                self.dashboard
                    .set_status("Start operator inside cmux to connect");
            }
            SessionWrapperType::Zellij => {
                self.dashboard
                    .set_status("Start operator inside zellij to connect");
            }
        }
    }

    fn set_default_llm(&mut self, tool_name: &str, model: &str) {
        self.config.llm_tools.default_tool = Some(tool_name.to_string());
        self.config.llm_tools.default_model = Some(model.to_string());
        if let Err(e) = self.config.save() {
            self.dashboard
                .set_status(&format!("Failed to save config: {e}"));
            return;
        }
        self.dashboard.update_config(&self.config);
        self.dashboard
            .set_status(&format!("Default LLM set to {tool_name}:{model}"));
    }

    /// Open the embedded web UI in the default browser.
    pub(super) fn open_web_ui(&mut self) -> Result<()> {
        if self.rest_api_server.is_running() {
            let port = self.config.rest_api.port;
            let url = format!("http://localhost:{port}/");
            if let Err(e) = open_in_browser(&url) {
                self.dashboard
                    .set_status(&format!("Failed to open browser: {e}"));
            } else {
                self.dashboard
                    .set_status(&format!("Opened web UI at {url}"));
            }
        } else {
            self.dashboard
                .set_status("API not running — web UI unavailable");
        }
        Ok(())
    }

    /// Toggle the Backstage server.
    pub(super) fn toggle_backstage(&mut self, terminal: &mut AppTerminal) -> Result<()> {
        let backstage_running = self.backstage_server.is_running();
        let rest_running = self.rest_api_server.is_running();

        if backstage_running && rest_running {
            // Both running - stop both
            self.rest_api_server.stop();
            if let Err(e) = self.backstage_server.stop() {
                tracing::error!("Backstage stop failed: {}", e);
            }
        } else {
            // Show yellow "Starting" indicator immediately for feedback
            use crate::backstage::ServerStatus;
            self.dashboard
                .update_backstage_status(ServerStatus::Starting);
            terminal.draw(|f| self.dashboard.render(f))?;

            // Start both if not running
            if !rest_running {
                if let Err(e) = self.rest_api_server.start() {
                    tracing::error!("REST API start failed: {}", e);
                }
            }
            if !backstage_running {
                if let Err(e) = self.backstage_server.start() {
                    tracing::error!("Backstage start failed: {}", e);
                }
            }
            // Wait for server to be ready before opening browser
            if self.backstage_server.is_running() {
                match self.backstage_server.wait_for_ready(25000) {
                    Ok(()) => {
                        if let Err(e) = self.backstage_server.open_browser() {
                            tracing::warn!("Failed to open browser: {}", e);
                        }
                    }
                    Err(e) => {
                        tracing::error!("Server not ready: {}", e);
                    }
                }
            }
        }
        Ok(())
    }
}

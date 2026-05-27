#![allow(dead_code)]

use std::path::Path;
use std::time::Instant;

use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};

use super::in_progress_panel::InProgressPanel;
use super::panels::{CompletedPanel, HeaderBar, QueuePanel, StatusBar};
use super::status_panel::{
    DelegatorInfo, KanbanProviderInfo, LlmToolInfo, StatusPanel, StatusSnapshot,
    WrapperConnectionStatus,
};
use crate::backstage::ServerStatus;
use crate::config::{Config, GitProviderConfig, SessionWrapperType};
use crate::editors::EditorConfig;
use crate::queue::Ticket;
use crate::rest::RestApiStatus;
use crate::state::{AgentState, CompletedTicket, OrphanSession};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusedPanel {
    Status,
    Queue,
    InProgress,
    Completed,
}

pub struct Dashboard {
    pub status_panel: StatusPanel,
    pub queue_panel: QueuePanel,
    pub in_progress_panel: InProgressPanel,
    pub completed_panel: CompletedPanel,
    pub focused: FocusedPanel,
    pub paused: bool,
    pub max_agents: usize,
    /// Backstage server status
    pub backstage_status: ServerStatus,
    /// REST API server status
    pub rest_api_status: RestApiStatus,
    /// Wrapper display name for header bar
    pub wrapper_name: &'static str,
    /// Exit confirmation mode (first Ctrl+C pressed)
    pub exit_confirmation_mode: bool,
    /// Version update available (if notification should be shown)
    pub update_available_version: Option<String>,
    /// Transient status message (auto-dismissed after 5s)
    pub status_message: Option<String>,
    /// When the status message was set
    pub status_message_at: Option<Instant>,
    /// Cached wrapper connection status (updated periodically)
    pub wrapper_connection_status: WrapperConnectionStatus,
    /// Config snapshot for status panel
    config: Config,
    /// Resolved editor environment variables
    pub editor_config: EditorConfig,
    /// Active MCP SSE sessions, updated by the app each tick via `update_mcp_active_sessions`.
    pub mcp_active_sessions: usize,
    /// ACP agent advertisement + active-session count. Updated by `App` on
    /// construction; refreshed by `update_acp_status` if it changes later.
    pub acp_status: crate::acp::AcpAgentStatus,
}

impl Dashboard {
    pub fn new(config: &Config) -> Self {
        let mut dashboard = Self {
            status_panel: StatusPanel::new(config.ui.panel_names.status.clone()),
            queue_panel: QueuePanel::new(config.ui.panel_names.queue.clone()),
            in_progress_panel: InProgressPanel::new(config.ui.panel_names.in_progress.clone()),
            completed_panel: CompletedPanel::new(config.ui.panel_names.completed.clone()),
            focused: FocusedPanel::Status,
            paused: false,
            max_agents: config.effective_max_agents(),
            wrapper_name: config.sessions.wrapper.display_name(),
            backstage_status: ServerStatus::Stopped,
            rest_api_status: RestApiStatus::Stopped,
            exit_confirmation_mode: false,
            update_available_version: None,
            status_message: None,
            status_message_at: None,
            wrapper_connection_status: Self::initial_wrapper_status(config),
            config: config.clone(),
            editor_config: EditorConfig::detect(config.sessions.wrapper),
            mcp_active_sessions: 0,
            acp_status: crate::acp::AcpAgentServer::from_config(config).status(),
        };
        dashboard.compute_initial_focus();
        dashboard
    }

    /// Determine the best panel to focus on startup.
    ///
    /// Priority:
    /// 1. Status panel — if any section needs attention (Yellow/Red), focus there
    ///    and select the first section that needs attention
    /// 2. In Progress — if there are active agents
    /// 3. Queue — default fallback
    pub fn compute_initial_focus(&mut self) {
        let snapshot = self.build_status_snapshot();
        if self.status_panel.has_attention_needed(&snapshot) {
            self.focused = FocusedPanel::Status;
            self.status_panel.focus_first_attention(&snapshot);
        } else if !self.in_progress_panel.agents.is_empty() {
            self.focused = FocusedPanel::InProgress;
        } else {
            self.focused = FocusedPanel::Queue;
        }
    }

    pub fn update_backstage_status(&mut self, status: ServerStatus) {
        self.backstage_status = status;
    }

    pub fn update_rest_api_status(&mut self, status: RestApiStatus) {
        self.rest_api_status = status;
    }

    /// Update the active MCP SSE session count. Called each tick by the app
    /// from `rest_api_server.api_state().map(|s| s.mcp_sessions.try_lock()...)`.
    pub fn update_mcp_active_sessions(&mut self, count: usize) {
        self.mcp_active_sessions = count;
    }

    pub fn update_exit_confirmation_mode(&mut self, mode: bool) {
        self.exit_confirmation_mode = mode;
    }

    pub fn update_available_version(&mut self, version: Option<String>) {
        self.update_available_version = version;
    }

    /// Set a transient status message (auto-dismissed after 5 seconds)
    pub fn set_status(&mut self, msg: &str) {
        self.status_message = Some(msg.to_string());
        self.status_message_at = Some(Instant::now());
    }

    /// Clear status message if it has expired (5 second TTL)
    pub fn clear_expired_status(&mut self) {
        if let Some(at) = self.status_message_at {
            if at.elapsed() > std::time::Duration::from_secs(5) {
                self.status_message = None;
                self.status_message_at = None;
            }
        }
    }

    pub fn update_config(&mut self, config: &Config) {
        self.config = config.clone();
    }

    pub fn expand_and_focus_section(&mut self, section_id: super::status_panel::SectionId) {
        let snapshot = self.build_status_snapshot();
        self.status_panel
            .tree_state
            .expanded
            .insert(section_id, true);
        // Find the header row for the section and select it
        let rows = self.status_panel.flatten(&snapshot);
        for (i, row) in rows.iter().enumerate() {
            if row.is_header && row.section_id == section_id {
                self.status_panel.tree_state.selected = i;
                break;
            }
        }
    }

    pub fn update_queue(&mut self, tickets: Vec<Ticket>) {
        self.queue_panel.tickets = tickets;
    }

    pub fn update_agents(&mut self, agents: Vec<AgentState>) {
        // All agents go to the unified in_progress_panel
        self.in_progress_panel.agents = agents;
    }

    pub fn update_completed(&mut self, tickets: Vec<CompletedTicket>) {
        self.completed_panel.tickets = tickets;
    }

    pub fn update_orphan_sessions(&mut self, orphans: Vec<OrphanSession>) {
        self.in_progress_panel.orphan_sessions = orphans;
    }

    /// Create initial wrapper connection status based on config.
    fn initial_wrapper_status(config: &Config) -> WrapperConnectionStatus {
        match config.sessions.wrapper {
            SessionWrapperType::Tmux => WrapperConnectionStatus::Tmux {
                available: false,
                server_running: false,
                version: None,
            },
            SessionWrapperType::Vscode => WrapperConnectionStatus::Vscode {
                webhook_running: false,
                port: Some(config.sessions.vscode.webhook_port),
            },
            SessionWrapperType::Cmux => WrapperConnectionStatus::Cmux {
                binary_available: false,
                in_cmux: std::env::var("CMUX_WORKSPACE_ID").is_ok(),
            },
            SessionWrapperType::Zellij => WrapperConnectionStatus::Zellij {
                binary_available: false,
                in_zellij: std::env::var("ZELLIJ").is_ok(),
            },
        }
    }

    /// Update the cached wrapper connection status.
    pub fn update_wrapper_connection_status(&mut self, status: WrapperConnectionStatus) {
        self.wrapper_connection_status = status;
    }

    /// Build a status snapshot from current config and runtime state
    fn build_status_snapshot(&self) -> StatusSnapshot {
        let config = &self.config;

        // Working directory is where the operator process runs from
        let working_dir = std::env::current_dir()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_default();
        let config_path = Config::operator_config_path()
            .to_string_lossy()
            .into_owned();
        let tickets_dir = config.paths.tickets.clone();
        let tickets_dir_exists = Path::new(&tickets_dir).exists();

        // Build kanban provider info from jira + linear configs
        let mut kanban_providers: Vec<KanbanProviderInfo> = Vec::new();
        for domain in config.kanban.jira.keys() {
            kanban_providers.push(KanbanProviderInfo {
                provider_type: "jira".to_string(),
                domain: domain.clone(),
            });
        }
        for slug in config.kanban.linear.keys() {
            kanban_providers.push(KanbanProviderInfo {
                provider_type: "linear".to_string(),
                domain: slug.clone(),
            });
        }

        // Build LLM tool info from detected tools
        let llm_tools: Vec<LlmToolInfo> = config
            .llm_tools
            .detected
            .iter()
            .map(|t| LlmToolInfo {
                name: t.name.clone(),
                version: t.version.clone(),
                model_aliases: t.model_aliases.clone(),
            })
            .collect();

        // Build delegator info
        let delegators: Vec<DelegatorInfo> = config
            .delegators
            .iter()
            .map(|d| DelegatorInfo {
                name: d.name.clone(),
                display_name: d.display_name.clone(),
                llm_tool: d.llm_tool.clone(),
                model: d.model.clone(),
                yolo: d.launch_config.as_ref().is_some_and(|lc| lc.yolo),
                model_server: d.model_server.clone(),
            })
            .collect();

        // Build model server info — implicit builtins plus any user-declared.
        let mut model_servers: Vec<crate::ui::status_panel::ModelServerInfo> = config
            .model_servers
            .iter()
            .map(|s| crate::ui::status_panel::ModelServerInfo {
                name: s.name.clone(),
                kind: s.kind.clone(),
                base_url: s.base_url.clone(),
                display_name: s.display_name.clone(),
                user_declared: true,
            })
            .collect();
        for tool in ["claude", "codex", "gemini"] {
            let implicit = crate::config::implicit_model_server_for_tool(tool);
            if !model_servers.iter().any(|s| s.name == implicit.name) {
                model_servers.push(crate::ui::status_panel::ModelServerInfo {
                    name: implicit.name,
                    kind: implicit.kind,
                    base_url: implicit.base_url,
                    display_name: implicit.display_name,
                    user_declared: false,
                });
            }
        }

        // Git config
        let git_provider = config.git.provider.as_ref().map(|p| format!("{p:?}"));
        let git_token_set = match config.git.provider {
            Some(GitProviderConfig::GitLab) => std::env::var(&config.git.gitlab.token_env).is_ok(),
            // GitHub is the default for all other providers (including None)
            _ => std::env::var(&config.git.github.token_env).is_ok(),
        };

        StatusSnapshot {
            working_dir,
            config_file_found: true, // We have a config if we're running
            config_path,
            tickets_dir,
            tickets_dir_exists,
            wrapper_type: config.sessions.wrapper.display_name().to_string(),
            operator_version: env!("CARGO_PKG_VERSION").to_string(),
            api_status: self.rest_api_status.clone(),
            backstage_status: self.backstage_status.clone(),
            backstage_display: config.backstage.display,
            kanban_providers,
            llm_tools,
            default_llm_tool: config.llm_tools.default_tool.clone(),
            default_llm_model: config.llm_tools.default_model.clone(),
            delegators,
            model_servers,
            git_provider,
            git_token_set,
            git_branch_format: Some(config.git.branch_format.clone()),
            git_use_worktrees: config.git.use_worktrees,
            update_available_version: self.update_available_version.clone(),
            wrapper_connection_status: self.wrapper_connection_status.clone(),
            env_editor: self.editor_config.editor.clone(),
            env_visual: self.editor_config.visual.clone(),
            mcp_http_status: if config.mcp.http_enabled {
                match &self.rest_api_status {
                    RestApiStatus::Running { port } => {
                        crate::ui::status_panel::McpHttpStatus::Mounted { port: *port }
                    }
                    _ => crate::ui::status_panel::McpHttpStatus::NotMounted,
                }
            } else {
                crate::ui::status_panel::McpHttpStatus::NotMounted
            },
            mcp_stdio_advertised: config.mcp.stdio_advertised,
            mcp_active_sessions: self.mcp_active_sessions,
            acp_stdio_advertised: self.acp_status.is_advertised(),
            acp_active_sessions: self.acp_status.active_sessions(),
            embed_ui_available: cfg!(feature = "embed-ui"),
        }
    }

    pub fn render(&mut self, frame: &mut Frame) {
        let snapshot = self.build_status_snapshot();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2), // Header
                Constraint::Min(10),   // Main content
                Constraint::Length(2), // Status bar
            ])
            .split(frame.area());

        // Header
        let header = HeaderBar {
            version: env!("CARGO_PKG_VERSION"),
            wrapper_name: self.wrapper_name,
        };
        header.render(frame, chunks[0]);

        // Main content - 4 columns: Status | Queue | In Progress | Completed
        // Focused panel gets 40% width, others get 20%
        let (s, q, ip, c) = match self.focused {
            FocusedPanel::Status => (40, 20, 20, 20),
            FocusedPanel::Queue => (20, 40, 20, 20),
            FocusedPanel::InProgress => (20, 20, 40, 20),
            FocusedPanel::Completed => (20, 20, 20, 40),
        };
        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(s),
                Constraint::Percentage(q),
                Constraint::Percentage(ip),
                Constraint::Percentage(c),
            ])
            .split(chunks[1]);

        // Render panels
        self.status_panel.render(
            frame,
            main_chunks[0],
            self.focused == FocusedPanel::Status,
            &snapshot,
        );

        self.queue_panel
            .render(frame, main_chunks[1], self.focused == FocusedPanel::Queue);

        self.in_progress_panel.render(
            frame,
            main_chunks[2],
            self.focused == FocusedPanel::InProgress,
            self.max_agents,
        );

        self.completed_panel.render(
            frame,
            main_chunks[3],
            self.focused == FocusedPanel::Completed,
        );

        // Status bar — show dynamic hints when status panel is focused
        let row_hints = if self.focused == FocusedPanel::Status {
            let snapshot = self.build_status_snapshot();
            self.status_panel.current_row_hints(&snapshot)
        } else {
            None
        };
        let status = StatusBar {
            paused: self.paused,
            agent_count: self.in_progress_panel.agents.len(),
            max_agents: self.max_agents,
            backstage_status: self.backstage_status.clone(),
            rest_api_status: self.rest_api_status.clone(),
            backstage_display: self.config.backstage.display,
            embed_ui_available: cfg!(feature = "embed-ui"),
            exit_confirmation_mode: self.exit_confirmation_mode,
            update_available_version: self.update_available_version.clone(),
            status_message: self.status_message.clone(),
            row_hints,
        };
        status.render(frame, chunks[2]);
    }

    pub fn focus_next(&mut self) {
        self.focused = match self.focused {
            FocusedPanel::Status => FocusedPanel::Queue,
            FocusedPanel::Queue => FocusedPanel::InProgress,
            FocusedPanel::InProgress => FocusedPanel::Completed,
            FocusedPanel::Completed => FocusedPanel::Status,
        };
    }

    pub fn focus_prev(&mut self) {
        self.focused = match self.focused {
            FocusedPanel::Status => FocusedPanel::Completed,
            FocusedPanel::Queue => FocusedPanel::Status,
            FocusedPanel::InProgress => FocusedPanel::Queue,
            FocusedPanel::Completed => FocusedPanel::InProgress,
        };
    }

    pub fn select_next(&mut self) {
        match self.focused {
            FocusedPanel::Status => {
                let snapshot = self.build_status_snapshot();
                self.status_panel.select_next(&snapshot);
            }
            FocusedPanel::Queue => {
                let len = self.queue_panel.tickets.len();
                if len > 0 {
                    let i = self.queue_panel.state.selected().map_or(0, |i| {
                        if i >= len - 1 {
                            0
                        } else {
                            i + 1
                        }
                    });
                    self.queue_panel.state.select(Some(i));
                }
            }
            FocusedPanel::InProgress => {
                let len = self.in_progress_panel.total_items();
                if len > 0 {
                    let i = self.in_progress_panel.state.selected().map_or(0, |i| {
                        if i >= len - 1 {
                            0
                        } else {
                            i + 1
                        }
                    });
                    self.in_progress_panel.state.select(Some(i));
                }
            }
            FocusedPanel::Completed => {}
        }
    }

    pub fn select_prev(&mut self) {
        match self.focused {
            FocusedPanel::Status => {
                let snapshot = self.build_status_snapshot();
                self.status_panel.select_prev(&snapshot);
            }
            FocusedPanel::Queue => {
                let len = self.queue_panel.tickets.len();
                if len > 0 {
                    let i = self.queue_panel.state.selected().map_or(0, |i| {
                        if i == 0 {
                            len - 1
                        } else {
                            i - 1
                        }
                    });
                    self.queue_panel.state.select(Some(i));
                }
            }
            FocusedPanel::InProgress => {
                let len = self.in_progress_panel.total_items();
                if len > 0 {
                    let i = self.in_progress_panel.state.selected().map_or(0, |i| {
                        if i == 0 {
                            len - 1
                        } else {
                            i - 1
                        }
                    });
                    self.in_progress_panel.state.select(Some(i));
                }
            }
            FocusedPanel::Completed => {}
        }
    }

    /// Get the action for the currently selected status panel row.
    /// Section toggles are handled internally by the status panel.
    pub fn status_action(
        &mut self,
        button: super::status_panel::ActionButton,
    ) -> super::status_panel::StatusAction {
        let snapshot = self.build_status_snapshot();
        self.status_panel.action_for_current(&snapshot, button)
    }

    pub fn selected_ticket(&self) -> Option<&Ticket> {
        if self.focused == FocusedPanel::Queue {
            self.queue_panel
                .state
                .selected()
                .and_then(|i| self.queue_panel.tickets.get(i))
        } else {
            None
        }
    }

    pub fn selected_agent(&self) -> Option<&AgentState> {
        match self.focused {
            FocusedPanel::InProgress => self
                .in_progress_panel
                .state
                .selected()
                .and_then(|i| self.in_progress_panel.agents.get(i)),
            _ => None,
        }
    }

    /// Get the selected orphan session (from `in_progress` panel, below the fold)
    pub fn selected_orphan(&self) -> Option<&OrphanSession> {
        self.in_progress_panel.selected_orphan()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    fn make_test_dashboard() -> Dashboard {
        // Minimal config for testing
        let config = Config::default();
        Dashboard::new(&config)
    }

    #[test]
    fn test_set_status_stores_message() {
        let mut dashboard = make_test_dashboard();
        assert!(dashboard.status_message.is_none());

        dashboard.set_status("Test message");
        assert_eq!(dashboard.status_message.as_deref(), Some("Test message"));
        assert!(dashboard.status_message_at.is_some());
    }

    #[test]
    fn test_clear_expired_status_keeps_fresh_message() {
        let mut dashboard = make_test_dashboard();
        dashboard.set_status("Fresh message");

        dashboard.clear_expired_status();
        assert!(dashboard.status_message.is_some());
    }

    #[test]
    fn test_clear_expired_status_clears_old_message() {
        let mut dashboard = make_test_dashboard();
        dashboard.status_message = Some("Old message".to_string());
        // Set timestamp to 6 seconds ago
        dashboard.status_message_at =
            Some(Instant::now().checked_sub(Duration::from_secs(6)).unwrap());

        dashboard.clear_expired_status();
        assert!(dashboard.status_message.is_none());
        assert!(dashboard.status_message_at.is_none());
    }

    #[test]
    fn test_clear_expired_status_noop_when_no_message() {
        let mut dashboard = make_test_dashboard();
        dashboard.clear_expired_status();
        assert!(dashboard.status_message.is_none());
    }

    #[test]
    fn test_version_matches_cargo_toml() {
        // env! is evaluated at compile time from Cargo.toml
        let version = env!("CARGO_PKG_VERSION");
        assert!(!version.is_empty(), "Version should not be empty");

        // Verify semver format (major.minor.patch)
        let parts: Vec<&str> = version.split('.').collect();
        assert!(
            parts.len() >= 2,
            "Version should have at least major.minor format"
        );

        // All parts should be numeric (except possible pre-release suffix)
        for part in parts.iter().take(3) {
            let numeric_part: &str = part.split('-').next().unwrap_or(part);
            assert!(
                numeric_part.parse::<u32>().is_ok(),
                "Version component '{part}' should be numeric"
            );
        }
    }

    #[test]
    fn test_focus_next_cycles_through_all_panels() {
        let mut dashboard = make_test_dashboard();
        dashboard.focused = FocusedPanel::Status;

        dashboard.focus_next();
        assert_eq!(dashboard.focused, FocusedPanel::Queue);
        dashboard.focus_next();
        assert_eq!(dashboard.focused, FocusedPanel::InProgress);
        dashboard.focus_next();
        assert_eq!(dashboard.focused, FocusedPanel::Completed);
        dashboard.focus_next();
        assert_eq!(dashboard.focused, FocusedPanel::Status);
    }

    #[test]
    fn test_focus_prev_cycles_through_all_panels() {
        let mut dashboard = make_test_dashboard();
        dashboard.focused = FocusedPanel::Status;

        dashboard.focus_prev();
        assert_eq!(dashboard.focused, FocusedPanel::Completed);
        dashboard.focus_prev();
        assert_eq!(dashboard.focused, FocusedPanel::InProgress);
        dashboard.focus_prev();
        assert_eq!(dashboard.focused, FocusedPanel::Queue);
        dashboard.focus_prev();
        assert_eq!(dashboard.focused, FocusedPanel::Status);
    }

    #[test]
    fn test_update_agents_no_partition() {
        let mut dashboard = make_test_dashboard();
        // All agents should go to in_progress_panel without splitting
        let agents = vec![];
        dashboard.update_agents(agents);
        assert!(dashboard.in_progress_panel.agents.is_empty());
    }
}

use anyhow::Result;
use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::fs;
use std::io;
use std::time::Duration;

use crate::agents::tmux::SystemTmuxClient;
use crate::agents::{
    AgentTicketCreator, AssessTicketCreator, LaunchOptions, Launcher, SessionMonitor,
    TicketSessionSync,
};
use crate::api::Capabilities;
use crate::backstage::scaffold::{BackstageScaffold, ScaffoldOptions};
use crate::backstage::BackstageServer;
use crate::config::Config;
use crate::notifications;
use crate::queue::{Queue, TicketCreator};
use crate::rest::RestApiServer;
use crate::setup::filter_schema_fields;
use crate::state::State;
use crate::templates::TemplateType;
use crate::ui::create_dialog::{CreateDialog, CreateDialogResult};
use crate::ui::dialogs::HelpDialog;
use crate::ui::projects_dialog::{ProjectAction, ProjectsDialog, ProjectsDialogResult};
use crate::ui::session_preview::SessionPreview;
use crate::ui::setup::{DetectedToolInfo, SetupResult, SetupScreen};
use crate::ui::{
    with_suspended_tui, ConfirmDialog, ConfirmSelection, Dashboard, SessionRecoveryDialog,
    SessionRecoverySelection,
};
use std::sync::Arc;

/// Type alias for the terminal used by the app
type AppTerminal = Terminal<CrosstermBackend<io::Stdout>>;

pub struct App {
    config: Config,
    dashboard: Dashboard,
    confirm_dialog: ConfirmDialog,
    help_dialog: HelpDialog,
    create_dialog: CreateDialog,
    projects_dialog: ProjectsDialog,
    setup_screen: Option<SetupScreen>,
    should_quit: bool,
    /// Message to print on exit (for unimplemented features)
    exit_message: Option<String>,
    /// Session health monitor
    session_monitor: SessionMonitor,
    /// Session preview dialog
    session_preview: SessionPreview,
    /// Ticket-session synchronizer
    ticket_sync: TicketSessionSync,
    /// API capabilities (rate limits, PR status, etc.)
    capabilities: Capabilities,
    /// Flag indicating rate limit sync is in progress
    rate_limit_syncing: bool,
    /// Last sync status message for display
    sync_status_message: Option<String>,
    /// Backstage server lifecycle manager
    backstage_server: BackstageServer,
    /// REST API server lifecycle manager
    rest_api_server: RestApiServer,
    /// Exit confirmation mode (first Ctrl+C pressed)
    exit_confirmation_mode: bool,
    /// Time when exit confirmation mode was entered
    exit_confirmation_time: Option<std::time::Instant>,
    /// Start web servers on launch (--web flag)
    start_web_on_launch: bool,
    /// Session recovery dialog for handling dead tmux sessions
    session_recovery_dialog: SessionRecoveryDialog,
}

impl App {
    pub fn new(mut config: Config, start_web: bool) -> Result<Self> {
        // Run LLM tool detection on first startup
        if !config.llm_tools.detection_complete {
            tracing::info!("Detecting LLM CLI tools...");
            config.llm_tools = crate::llm::detect_all_tools();

            // Log detected tools
            for tool in &config.llm_tools.detected {
                tracing::info!(
                    tool = %tool.name,
                    version = %tool.version,
                    path = %tool.path,
                    "LLM tool detected"
                );
            }

            // Log available providers
            for provider in &config.llm_tools.providers {
                tracing::debug!(
                    tool = %provider.tool,
                    model = %provider.model,
                    "LLM provider available"
                );
            }

            // Save the detection results to config
            if let Err(e) = config.save() {
                tracing::warn!("Failed to save LLM detection results: {}", e);
            }
        }

        let dashboard = Dashboard::new(&config);

        // Check if tickets directory exists
        let tickets_path = config.tickets_path();
        let needs_setup = !tickets_path.join("queue").exists();

        // For setup screen, we discover projects dynamically
        // After setup, we use the projects list from config
        let (setup_screen, projects_for_dialog) = if needs_setup {
            // Discover projects by tool for the setup screen display
            let projects_by_tool =
                crate::projects::discover_projects_by_tool(&config.projects_path());
            let discovered_projects = config.discover_projects();

            // Build detected tool info for display
            let detected_tools: Vec<DetectedToolInfo> = config
                .llm_tools
                .detected
                .iter()
                .map(|t| DetectedToolInfo {
                    name: t.name.clone(),
                    version: t.version.clone(),
                    model_count: t.model_aliases.len(),
                })
                .collect();

            let setup = SetupScreen::new(
                tickets_path.to_string_lossy().to_string(),
                detected_tools,
                projects_by_tool,
            );
            // Projects will be saved to config during initialize_tickets()
            (Some(setup), discovered_projects)
        } else {
            // Use projects from saved config
            (None, config.projects.clone())
        };

        // Create dialog with projects
        let mut create_dialog = CreateDialog::new();
        create_dialog.set_projects(projects_for_dialog.clone());

        // Projects dialog with projects
        let mut projects_dialog = ProjectsDialog::new();
        projects_dialog.set_projects(projects_for_dialog);
        projects_dialog.set_projects_path(config.projects_path());

        // Initialize session monitor
        let session_monitor = SessionMonitor::new(&config);

        // Initialize ticket-session sync with custom tmux config if available
        let tmux_client: Arc<dyn crate::agents::TmuxClient> = if config.tmux.config_generated {
            let config_path = config.tmux_config_path();
            if config_path.exists() {
                Arc::new(SystemTmuxClient::with_config(config_path))
            } else {
                Arc::new(SystemTmuxClient::new())
            }
        } else {
            Arc::new(SystemTmuxClient::new())
        };
        let ticket_sync = TicketSessionSync::new(&config, tmux_client);

        // Initialize API capabilities from environment
        let capabilities = Capabilities::from_env();
        if capabilities.has_ai() {
            tracing::info!(
                provider = capabilities.ai_provider_name(),
                "AI provider configured"
            );
        }
        if capabilities.has_repo() {
            tracing::info!(
                provider = capabilities.repo_provider_name(),
                "Repo provider configured"
            );
        }

        // Initialize Backstage server lifecycle manager using compiled binary mode
        let backstage_server = BackstageServer::with_compiled_binary(
            config.state_path(),
            config.backstage.release_url.clone(),
            config.backstage.local_binary_path.clone(),
            config.backstage.port,
        )
        .map_err(|e| anyhow::anyhow!("Failed to initialize backstage server: {}", e))?;

        // Initialize REST API server lifecycle manager
        let rest_api_server = RestApiServer::new(config.clone(), config.rest_api.port);

        Ok(Self {
            config,
            dashboard,
            confirm_dialog: ConfirmDialog::new(),
            help_dialog: HelpDialog::new(),
            create_dialog,
            projects_dialog,
            setup_screen,
            should_quit: false,
            exit_message: None,
            session_monitor,
            session_preview: SessionPreview::new(),
            ticket_sync,
            capabilities,
            rate_limit_syncing: false,
            sync_status_message: None,
            backstage_server,
            rest_api_server,
            exit_confirmation_mode: false,
            exit_confirmation_time: None,
            start_web_on_launch: start_web,
            session_recovery_dialog: SessionRecoveryDialog::new(),
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        // Reconcile state with actual tmux sessions on startup
        self.reconcile_sessions()?;

        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // Initial data load
        self.refresh_data()?;

        // Start web servers if --web flag was passed
        if self.start_web_on_launch {
            if let Err(e) = self.rest_api_server.start() {
                tracing::error!("REST API start failed: {}", e);
            }
            if let Err(e) = self.backstage_server.start() {
                tracing::error!("Backstage start failed: {}", e);
            }
            // Wait for server to be ready then open browser
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

        // Main loop
        let tick_rate = Duration::from_millis(self.config.ui.refresh_rate_ms);

        while !self.should_quit {
            if self.exit_confirmation_mode {
                if let Some(start_time) = self.exit_confirmation_time {
                    if start_time.elapsed() > Duration::from_secs(1) {
                        self.exit_confirmation_mode = false;
                        self.exit_confirmation_time = None;
                    }
                }
            }

            // Update dashboard with server statuses and exit confirmation mode
            self.dashboard
                .update_rest_api_status(self.rest_api_server.status());
            self.dashboard
                .update_exit_confirmation_mode(self.exit_confirmation_mode);

            // Draw
            terminal.draw(|f| {
                if let Some(ref mut setup) = self.setup_screen {
                    setup.render(f);
                } else {
                    self.dashboard.render(f);
                    self.confirm_dialog.render(f);
                    self.help_dialog.render(f);
                    self.create_dialog.render(f);
                    self.projects_dialog.render(f);
                    self.session_preview.render(f);
                    self.session_recovery_dialog.render(f);
                }
            })?;

            // Handle events
            if event::poll(tick_rate)? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        // Handle Ctrl+C for graceful shutdown
                        if key.code == KeyCode::Char('c')
                            && key.modifiers.contains(KeyModifiers::CONTROL)
                        {
                            self.handle_ctrl_c();
                        } else {
                            // Reset exit confirmation if any other key is pressed
                            if self.exit_confirmation_mode {
                                self.exit_confirmation_mode = false;
                                self.exit_confirmation_time = None;
                            }
                            self.handle_key(key.code, &mut terminal).await?;
                        }
                    }
                }
            }

            // Refresh data periodically
            self.refresh_data()?;

            // Run health checks if it's time
            self.run_health_checks()?;

            // Run periodic ticket-session sync
            self.run_periodic_sync()?;
        }

        // Restore terminal
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        // Check for exit message (unimplemented features)
        if let Some(message) = &self.exit_message {
            eprintln!("{}", message);
            std::process::exit(1);
        }

        Ok(())
    }

    fn refresh_data(&mut self) -> Result<()> {
        // Load queue
        let queue = Queue::new(&self.config)?;
        let tickets = queue.list_by_priority()?;
        self.dashboard.update_queue(tickets);

        // Load state
        let state = State::load(&self.config)?;
        self.dashboard.paused = state.paused;

        // Update agents
        let agents: Vec<_> = state.agents.clone();
        self.dashboard.update_agents(agents);

        // Update completed
        let completed: Vec<_> = state
            .recent_completions(self.config.ui.completed_history_hours)
            .into_iter()
            .cloned()
            .collect();
        self.dashboard.update_completed(completed);

        // Update Backstage server status
        self.backstage_server.refresh_status();
        self.dashboard
            .update_backstage_status(self.backstage_server.status());

        Ok(())
    }

    /// Reconcile state with actual tmux sessions on startup
    fn reconcile_sessions(&self) -> Result<()> {
        let result = self.session_monitor.reconcile_on_startup()?;

        if result.active > 0 {
            tracing::info!(
                active = result.active,
                "Found active agent sessions from previous run"
            );
        }

        if !result.orphaned.is_empty() {
            tracing::warn!(
                orphaned = result.orphaned.len(),
                "Found orphaned agents (sessions no longer exist)"
            );

            // Notify about orphaned sessions
            if self.config.notifications.enabled {
                for session in &result.orphaned {
                    notifications::send(
                        "Orphaned Agent Found",
                        session,
                        "This agent's session was not found on startup.",
                        self.config.notifications.sound,
                    )?;
                }
            }
        }

        if !result.stale_sessions.is_empty() {
            tracing::warn!(
                stale = result.stale_sessions.len(),
                "Found stale tmux sessions with no matching agent"
            );

            // Auto-cleanup stale sessions
            let killed = self
                .session_monitor
                .cleanup_stale_sessions(&result.stale_sessions)?;
            if killed > 0 {
                tracing::info!(killed = killed, "Cleaned up stale tmux sessions");
            }
        }

        Ok(())
    }

    /// Run session health checks and handle orphaned sessions
    fn run_health_checks(&mut self) -> Result<()> {
        // Only check if it's time
        if !self.session_monitor.should_check() {
            return Ok(());
        }

        let result = self.session_monitor.check_health()?;

        // Send notifications for orphaned sessions
        if !result.orphaned.is_empty() {
            tracing::warn!(
                orphaned = result.orphaned.len(),
                "Detected orphaned agent sessions"
            );

            if self.config.notifications.enabled {
                for session in &result.orphaned {
                    notifications::send(
                        "Agent Session Lost",
                        session,
                        "The tmux session for this agent has terminated unexpectedly.",
                        self.config.notifications.sound,
                    )?;
                }
            }
        }

        // Log content changes at debug level
        if !result.changed.is_empty() {
            tracing::debug!(
                changed = result.changed.len(),
                "Agent sessions with content changes"
            );
        }

        // Detect and update orphan sessions for display
        if let Ok(orphans) = self.session_monitor.detect_orphan_sessions() {
            self.dashboard.update_orphan_sessions(orphans);
        }

        Ok(())
    }

    /// Run periodic ticket-session sync
    fn run_periodic_sync(&mut self) -> Result<()> {
        if !self.ticket_sync.should_sync() {
            return Ok(());
        }

        self.execute_sync()
    }

    /// Run manual sync (triggered by 'S' key)
    fn run_manual_sync(&mut self) -> Result<()> {
        self.ticket_sync.force_sync();
        self.execute_sync()
    }

    /// Sync rate limits from AI provider
    async fn sync_rate_limits(&mut self) {
        if !self.capabilities.has_ai() {
            self.sync_status_message = Some("No AI provider configured".to_string());
            return;
        }

        self.rate_limit_syncing = true;
        self.sync_status_message = Some("Syncing rate limits...".to_string());

        match self.capabilities.sync_rate_limits().await {
            Ok(info) => {
                let summary = info.summary();
                self.sync_status_message = Some(format!("Rate limits: {}", summary));

                // Update dashboard with rate limit info
                self.dashboard.update_rate_limit(Some(info));

                // Check for providers needing token refresh
                let needs_refresh = self.capabilities.providers_needing_refresh();
                if !needs_refresh.is_empty() {
                    tracing::warn!(
                        providers = ?needs_refresh,
                        "Providers need token refresh (persistent 401 errors)"
                    );
                }
            }
            Err(e) => {
                self.sync_status_message = Some(format!("Rate limit sync failed: {}", e));
                tracing::warn!("Rate limit sync failed: {}", e);

                // Check if token needs refresh
                if e.needs_token_refresh() {
                    self.sync_status_message = Some(format!(
                        "{} token expired - please refresh",
                        e.provider_name()
                    ));
                }
            }
        }

        self.rate_limit_syncing = false;
    }

    /// Execute the sync and handle results
    fn execute_sync(&mut self) -> Result<()> {
        use crate::queue::Queue;

        let mut state = State::load(&self.config)?;
        let queue = Queue::new(&self.config)?;

        // Run health check to get current session states
        let health_result = self.session_monitor.check_health()?;

        // Run the sync
        let result = self
            .ticket_sync
            .sync_all(&mut state, &queue, &health_result)?;

        // Log results
        if result.synced > 0 {
            tracing::debug!(
                synced = result.synced,
                awaiting = result.moved_to_awaiting.len(),
                timed_out = result.timed_out.len(),
                "Ticket-session sync completed"
            );
        }

        // Send notifications for tickets that moved to awaiting
        if self.config.notifications.enabled {
            for ticket_id in &result.moved_to_awaiting {
                notifications::send(
                    "Agent Awaiting Input",
                    ticket_id,
                    "The agent is waiting for user input.",
                    self.config.notifications.sound,
                )?;
            }

            for ticket_id in &result.timed_out {
                notifications::send(
                    "Step Timed Out",
                    ticket_id,
                    "The agent step has timed out and is now awaiting input.",
                    self.config.notifications.sound,
                )?;
            }
        }

        // Log any errors
        for error in &result.errors {
            tracing::warn!("Sync error: {}", error);
        }

        Ok(())
    }

    async fn handle_key(&mut self, key: KeyCode, terminal: &mut AppTerminal) -> Result<()> {
        // Setup screen takes absolute priority
        if let Some(ref mut setup) = self.setup_screen {
            match key {
                KeyCode::Char('i') | KeyCode::Char('I') => {
                    if setup.confirm_selected {
                        self.initialize_tickets()?;
                        self.setup_screen = None;
                        self.refresh_data()?;
                    }
                }
                KeyCode::Enter => {
                    match setup.confirm() {
                        SetupResult::Initialize => {
                            self.initialize_tickets()?;
                            self.setup_screen = None;
                            self.refresh_data()?;
                        }
                        SetupResult::Cancel => {
                            self.should_quit = true;
                        }
                        SetupResult::ExitUnimplemented(message) => {
                            self.exit_message = Some(message);
                            self.should_quit = true;
                        }
                        SetupResult::Continue => {
                            // Moved to next step - stay in setup
                        }
                    }
                }
                KeyCode::Char('c') | KeyCode::Char('C') => {
                    self.should_quit = true;
                }
                KeyCode::Esc => {
                    match setup.go_back() {
                        SetupResult::Cancel => {
                            self.should_quit = true;
                        }
                        _ => {
                            // Moved to previous step - stay in setup
                        }
                    }
                }
                KeyCode::Tab | KeyCode::Left | KeyCode::Right => {
                    setup.toggle_selection();
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    setup.select_prev();
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    setup.select_next();
                }
                KeyCode::Char(' ') => {
                    setup.toggle_selection();
                }
                _ => {}
            }
            return Ok(());
        }

        // Help dialog takes priority
        if self.help_dialog.visible {
            self.help_dialog.visible = false;
            return Ok(());
        }

        // Session preview handling
        if self.session_preview.visible {
            match key {
                KeyCode::Esc | KeyCode::Char('q') => {
                    self.session_preview.hide();
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.session_preview.scroll_up();
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.session_preview.scroll_down(30); // Approximate viewport height
                }
                KeyCode::PageUp => {
                    self.session_preview.page_up(30);
                }
                KeyCode::PageDown => {
                    self.session_preview.page_down(30);
                }
                KeyCode::Char('g') => {
                    self.session_preview.scroll_to_top();
                }
                KeyCode::Char('G') => {
                    self.session_preview.scroll_to_bottom(30);
                }
                _ => {}
            }
            return Ok(());
        }

        // Create dialog handling
        if self.create_dialog.visible {
            if let Some(result) = self.create_dialog.handle_key(key) {
                self.create_ticket(result, terminal)?;
            }
            return Ok(());
        }

        // Projects dialog handling
        if self.projects_dialog.visible {
            if let Some(result) = self.projects_dialog.handle_key(key) {
                self.execute_project_action(result)?;
            }
            return Ok(());
        }

        // Confirm dialog handling
        if self.confirm_dialog.visible {
            // Check if options are focused for different key behavior
            if self.confirm_dialog.is_options_focused() {
                match key {
                    // Down or Enter moves focus back to buttons
                    KeyCode::Down | KeyCode::Enter => {
                        self.confirm_dialog.focus_buttons();
                    }
                    // Up/k navigates between options (provider <-> project)
                    KeyCode::Up | KeyCode::Char('k') => {
                        self.confirm_dialog.prev_option();
                    }
                    // j also navigates between options
                    KeyCode::Char('j') => {
                        self.confirm_dialog.next_option();
                    }
                    // Left/Right cycles the current option's value
                    KeyCode::Right | KeyCode::Tab => {
                        self.confirm_dialog.cycle_current_option();
                    }
                    KeyCode::Left => {
                        self.confirm_dialog.cycle_current_option_prev();
                    }
                    // Escape closes dialog
                    KeyCode::Esc => {
                        self.confirm_dialog.hide();
                    }
                    // Direct shortcuts still work
                    KeyCode::Char('m') | KeyCode::Char('M') => {
                        self.confirm_dialog.cycle_provider();
                    }
                    KeyCode::Char('p') | KeyCode::Char('P') => {
                        self.confirm_dialog.cycle_project();
                    }
                    KeyCode::Char('d') | KeyCode::Char('D') => {
                        self.confirm_dialog.toggle_docker();
                    }
                    KeyCode::Char('a') | KeyCode::Char('A') => {
                        self.confirm_dialog.toggle_yolo();
                    }
                    _ => {}
                }
            } else {
                // Buttons focused (default behavior)
                match key {
                    KeyCode::Char('y') | KeyCode::Char('Y') => {
                        self.launch_confirmed().await?;
                    }
                    KeyCode::Char('v') | KeyCode::Char('V') => {
                        self.view_ticket(terminal)?;
                    }
                    KeyCode::Char('e') | KeyCode::Char('E') => {
                        self.edit_ticket(terminal)?;
                    }
                    KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                        self.confirm_dialog.hide();
                    }
                    // Up moves focus to options section (if options available)
                    KeyCode::Up => {
                        self.confirm_dialog.focus_options();
                    }
                    // Launch options shortcuts: M = provider, P = project, D = docker, A = auto
                    KeyCode::Char('m') | KeyCode::Char('M') => {
                        self.confirm_dialog.cycle_provider();
                    }
                    KeyCode::Char('p') | KeyCode::Char('P') => {
                        self.confirm_dialog.cycle_project();
                    }
                    KeyCode::Char('d') | KeyCode::Char('D') => {
                        self.confirm_dialog.toggle_docker();
                    }
                    KeyCode::Char('a') | KeyCode::Char('A') => {
                        self.confirm_dialog.toggle_yolo();
                    }
                    KeyCode::Tab | KeyCode::Right => {
                        self.confirm_dialog.select_next();
                    }
                    KeyCode::Left => {
                        self.confirm_dialog.select_prev();
                    }
                    KeyCode::Enter => match self.confirm_dialog.selection {
                        ConfirmSelection::Yes => {
                            self.launch_confirmed().await?;
                        }
                        ConfirmSelection::View => {
                            self.view_ticket(terminal)?;
                        }
                        ConfirmSelection::No => {
                            self.confirm_dialog.hide();
                        }
                    },
                    _ => {}
                }
            }
            return Ok(());
        }

        // Session recovery dialog handling
        if self.session_recovery_dialog.visible {
            match key {
                KeyCode::Char('r') | KeyCode::Char('R') => {
                    if self.session_recovery_dialog.has_session_id() {
                        self.handle_session_recovery(SessionRecoverySelection::ResumeSession)
                            .await?;
                    }
                }
                KeyCode::Char('s') | KeyCode::Char('S') => {
                    self.handle_session_recovery(SessionRecoverySelection::StartFresh)
                        .await?;
                }
                KeyCode::Char('q') | KeyCode::Char('Q') => {
                    self.handle_session_recovery(SessionRecoverySelection::ReturnToQueue)
                        .await?;
                }
                KeyCode::Esc => {
                    self.session_recovery_dialog.hide();
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.session_recovery_dialog.select_prev();
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.session_recovery_dialog.select_next();
                }
                KeyCode::Enter => {
                    let selection = self.session_recovery_dialog.selection;
                    self.handle_session_recovery(selection).await?;
                }
                _ => {}
            }
            return Ok(());
        }

        // Normal mode
        match key {
            KeyCode::Char('q') => {
                // Stop servers if running before exiting
                if self.rest_api_server.is_running() || self.backstage_server.is_running() {
                    self.rest_api_server.stop();
                    let _ = self.backstage_server.stop();
                }
                self.should_quit = true;
            }
            KeyCode::Char('?') => {
                self.help_dialog.toggle();
            }
            KeyCode::Tab => {
                self.dashboard.focus_next();
            }
            KeyCode::BackTab => {
                self.dashboard.focus_prev();
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.dashboard.select_prev();
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.dashboard.select_next();
            }
            KeyCode::Char('L') | KeyCode::Char('l') => {
                self.try_launch()?;
            }
            KeyCode::Enter => {
                // Enter key behavior depends on focused panel
                match self.dashboard.focused {
                    crate::ui::dashboard::FocusedPanel::Queue => {
                        self.try_launch()?;
                    }
                    crate::ui::dashboard::FocusedPanel::Agents
                    | crate::ui::dashboard::FocusedPanel::Awaiting => {
                        self.attach_to_session(terminal)?;
                    }
                    crate::ui::dashboard::FocusedPanel::Completed => {
                        // No action on completed panel
                    }
                }
            }
            KeyCode::Char('P') | KeyCode::Char('p') => {
                self.pause_queue()?;
            }
            KeyCode::Char('R') | KeyCode::Char('r') => {
                self.resume_queue()?;
            }
            KeyCode::Char('Q') => {
                self.dashboard.focused = crate::ui::dashboard::FocusedPanel::Queue;
            }
            KeyCode::Char('A') | KeyCode::Char('a') => {
                self.dashboard.focused = crate::ui::dashboard::FocusedPanel::Agents;
            }
            KeyCode::Char('C') => {
                self.create_dialog.show();
            }
            KeyCode::Char('J') => {
                self.projects_dialog.show();
            }
            KeyCode::Char('v') | KeyCode::Char('V') => {
                self.show_session_preview()?;
            }
            KeyCode::Char('S') => {
                // Sync both ticket-session state and rate limits
                self.run_manual_sync()?;
                self.sync_rate_limits().await;
            }
            KeyCode::Char('W') | KeyCode::Char('w') => {
                // Toggle both REST API and Backstage servers together
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
                    // Polls /health every 500ms, up to 50 times (25 seconds)
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
            }
            _ => {}
        }

        Ok(())
    }

    /// Handle Ctrl+C for graceful two-stage exit
    fn handle_ctrl_c(&mut self) {
        if self.exit_confirmation_mode {
            // Second Ctrl+C - exit immediately
            self.should_quit = true;
        } else {
            // First Ctrl+C - stop servers and enter confirmation mode
            self.rest_api_server.stop();
            let _ = self.backstage_server.stop();

            self.exit_confirmation_mode = true;
            self.exit_confirmation_time = Some(std::time::Instant::now());
        }
    }

    fn try_launch(&mut self) -> Result<()> {
        // Check if we can launch
        let state = State::load(&self.config)?;
        let running_count = state.running_agents().len();

        if running_count >= self.config.effective_max_agents() {
            // Could show an error dialog here
            return Ok(());
        }

        if self.dashboard.paused {
            // Could show an error dialog here
            return Ok(());
        }

        // Get selected ticket
        if let Some(ticket) = self.dashboard.selected_ticket().cloned() {
            // Check if project is already busy
            if state.is_project_busy(&ticket.project) {
                // Could show an error dialog here
                return Ok(());
            }

            // Configure dialog with available options from config
            self.confirm_dialog.configure(
                self.config.llm_tools.providers.clone(),
                self.config.projects.clone(),
                self.config.launch.docker.enabled,
                self.config.launch.yolo.enabled,
            );

            // Show confirmation
            self.confirm_dialog.show(ticket);
        }

        Ok(())
    }

    async fn launch_confirmed(&mut self) -> Result<()> {
        if let Some(ticket) = self.confirm_dialog.ticket.take() {
            let launcher = Launcher::new(&self.config)?;

            // Build launch options from dialog state
            // Only set project_override if it differs from the ticket's original project
            let project_override = if self.confirm_dialog.is_project_overridden() {
                self.confirm_dialog.selected_project_name().cloned()
            } else {
                None
            };

            let options = LaunchOptions {
                provider: self.confirm_dialog.selected_provider().cloned(),
                docker_mode: self.confirm_dialog.docker_selected,
                yolo_mode: self.confirm_dialog.yolo_selected,
                project_override,
            };

            launcher.launch_with_options(&ticket, options).await?;
            self.confirm_dialog.hide();
            self.refresh_data()?;
        }
        Ok(())
    }

    /// View ticket file in $VISUAL or with `open` command
    fn view_ticket(&mut self, terminal: &mut AppTerminal) -> Result<()> {
        let Some(filepath) = self.confirm_dialog.ticket_filepath() else {
            return Ok(());
        };

        with_suspended_tui(terminal, || {
            // Try $VISUAL first, then fall back to `open` (macOS)
            let result = if let Ok(visual) = std::env::var("VISUAL") {
                std::process::Command::new(&visual).arg(&filepath).status()
            } else {
                std::process::Command::new("open").arg(&filepath).status()
            };

            if let Err(e) = result {
                tracing::warn!("Failed to open file: {}", e);
            }

            Ok(())
        })
    }

    /// Edit ticket file in $EDITOR
    fn edit_ticket(&mut self, terminal: &mut AppTerminal) -> Result<()> {
        let Some(filepath) = self.confirm_dialog.ticket_filepath() else {
            return Ok(());
        };

        let Ok(editor) = std::env::var("EDITOR") else {
            // No EDITOR set, do nothing
            return Ok(());
        };

        with_suspended_tui(terminal, || {
            let result = std::process::Command::new(&editor).arg(&filepath).status();

            if let Err(e) = result {
                tracing::warn!("Failed to open editor: {}", e);
            }

            Ok(())
        })
    }

    fn pause_queue(&mut self) -> Result<()> {
        let mut state = State::load(&self.config)?;
        state.set_paused(true)?;
        self.dashboard.paused = true;
        Ok(())
    }

    fn resume_queue(&mut self) -> Result<()> {
        let mut state = State::load(&self.config)?;
        state.set_paused(false)?;
        self.dashboard.paused = false;
        Ok(())
    }

    /// Show session preview for the selected agent
    fn show_session_preview(&mut self) -> Result<()> {
        use crate::agents::tmux::{SystemTmuxClient, TmuxClient};
        use crate::ui::dashboard::FocusedPanel;

        // Only works when agents or awaiting panel is focused
        let agent = match self.dashboard.focused {
            FocusedPanel::Agents => self.dashboard.selected_running_agent().cloned(),
            FocusedPanel::Awaiting => self.dashboard.selected_awaiting_agent().cloned(),
            _ => None,
        };

        let Some(agent) = agent else {
            return Ok(());
        };

        // Check if agent has a session
        let Some(ref session_name) = agent.session_name else {
            self.session_preview.show(
                &agent,
                Err("This agent does not have an attached tmux session.".to_string()),
            );
            return Ok(());
        };

        // Capture the session content
        // Use custom tmux config if it has been generated and exists
        let tmux: Box<dyn TmuxClient> = if self.config.tmux.config_generated {
            let config_path = self.config.tmux_config_path();
            if config_path.exists() {
                Box::new(SystemTmuxClient::with_config(config_path))
            } else {
                Box::new(SystemTmuxClient::new())
            }
        } else {
            Box::new(SystemTmuxClient::new())
        };

        let content = tmux
            .capture_pane(session_name, false)
            .map_err(|e| format!("Failed to capture session: {}", e));

        self.session_preview.show(&agent, content);

        Ok(())
    }

    /// Attach to the selected agent's tmux session
    ///
    /// Suspends the TUI, runs `tmux attach`, and restores the TUI when the user detaches.
    fn attach_to_session(&mut self, terminal: &mut AppTerminal) -> Result<()> {
        use crate::agents::tmux::TmuxClient;
        use crate::ui::dashboard::FocusedPanel;

        // Get the selected agent or orphan session based on focused panel
        let session_name = match self.dashboard.focused {
            FocusedPanel::Agents => {
                // Check if an orphan session is selected
                if let Some(orphan) = self.dashboard.selected_orphan() {
                    Some(orphan.session_name.clone())
                } else {
                    // Otherwise get selected running agent's session
                    self.dashboard
                        .selected_running_agent()
                        .and_then(|a| a.session_name.clone())
                }
            }
            FocusedPanel::Awaiting => self
                .dashboard
                .selected_awaiting_agent()
                .and_then(|a| a.session_name.clone()),
            _ => None,
        };

        let Some(session_name) = session_name else {
            return Ok(());
        };

        // Create tmux client (with custom config if available)
        let tmux: Box<dyn TmuxClient> = if self.config.tmux.config_generated {
            let config_path = self.config.tmux_config_path();
            if config_path.exists() {
                Box::new(SystemTmuxClient::with_config(config_path))
            } else {
                Box::new(SystemTmuxClient::new())
            }
        } else {
            Box::new(SystemTmuxClient::new())
        };

        tracing::info!(session = %session_name, "Attaching to tmux session");

        // Suspend TUI and attach to session, capturing any error
        let attach_result = with_suspended_tui(terminal, || Ok(tmux.attach_session(&session_name)));

        match attach_result {
            Ok(Ok(())) => {
                tracing::info!(session = %session_name, "Detached from tmux session");
            }
            Ok(Err(e)) => {
                let error_str = e.to_string();
                if error_str.contains("exit code: Some(1)") {
                    // Session no longer exists - show recovery dialog
                    tracing::warn!(
                        session = %session_name,
                        "Tmux session not found, showing recovery dialog"
                    );
                    self.show_session_recovery_dialog(&session_name)?;
                } else {
                    tracing::warn!(
                        session = %session_name,
                        error = %e,
                        "Failed to attach to session"
                    );
                }
            }
            Err(e) => {
                tracing::warn!(
                    session = %session_name,
                    error = %e,
                    "Error during TUI suspension"
                );
            }
        }

        // Refresh data after returning
        self.refresh_data()?;

        Ok(())
    }

    /// Show the session recovery dialog for a dead tmux session
    fn show_session_recovery_dialog(&mut self, session_name: &str) -> Result<()> {
        // Find the agent by session name
        let state = State::load(&self.config)?;
        let agent = state.agent_by_session(session_name);

        let Some(agent) = agent else {
            tracing::warn!(session = %session_name, "No agent found for session");
            return Ok(());
        };

        let ticket_id = agent.ticket_id.clone();
        let current_step = agent.current_step.clone();

        // Load the ticket to get session data
        let queue = Queue::new(&self.config)?;
        let ticket = queue.get_in_progress_ticket(&ticket_id)?;

        let Some(ticket) = ticket else {
            tracing::warn!(ticket = %ticket_id, "Ticket not found in in-progress");
            return Ok(());
        };

        // Get the step name (current_step from agent or step from ticket)
        let step = current_step.unwrap_or_else(|| ticket.step.clone()).clone();
        let step = if step.is_empty() {
            "initial".to_string()
        } else {
            step
        };

        // Look up Claude session ID for this step
        let claude_session_id = ticket.get_session_id(&step).cloned();

        // Show the recovery dialog
        self.session_recovery_dialog.show(
            ticket.id.clone(),
            session_name.to_string(),
            step,
            claude_session_id,
        );

        Ok(())
    }

    /// Handle a session recovery dialog selection
    async fn handle_session_recovery(&mut self, selection: SessionRecoverySelection) -> Result<()> {
        let ticket_id = self.session_recovery_dialog.ticket_id.clone();
        let session_name = self.session_recovery_dialog.session_name.clone();
        let claude_session_id = self.session_recovery_dialog.claude_session_id.clone();

        self.session_recovery_dialog.hide();

        match selection {
            SessionRecoverySelection::ResumeSession => {
                // Relaunch with resume flag
                self.relaunch_ticket(&ticket_id, &session_name, claude_session_id)
                    .await?;
            }
            SessionRecoverySelection::StartFresh => {
                // Relaunch without resume flag
                self.relaunch_ticket(&ticket_id, &session_name, None)
                    .await?;
            }
            SessionRecoverySelection::ReturnToQueue => {
                // Move ticket back to queue, remove agent from state
                self.return_ticket_to_queue(&ticket_id, &session_name)?;
            }
            SessionRecoverySelection::Cancel => {
                // Do nothing, dialog already hidden
            }
        }

        self.refresh_data()?;
        Ok(())
    }

    /// Relaunch a ticket with optional session resume
    async fn relaunch_ticket(
        &mut self,
        ticket_id: &str,
        old_session_name: &str,
        resume_session_id: Option<String>,
    ) -> Result<()> {
        use crate::agents::{Launcher, RelaunchOptions};

        // Load ticket from in-progress
        let queue = Queue::new(&self.config)?;
        let ticket = queue
            .get_in_progress_ticket(ticket_id)?
            .ok_or_else(|| anyhow::anyhow!("Ticket not found: {}", ticket_id))?;

        // Remove old agent state
        let mut state = State::load(&self.config)?;
        state.remove_agent_by_session(old_session_name)?;

        // Relaunch with the launcher
        let launcher = Launcher::new(&self.config)?;
        let options = RelaunchOptions {
            launch_options: LaunchOptions::default(),
            resume_session_id,
        };

        launcher.relaunch(&ticket, options).await?;

        Ok(())
    }

    /// Return a ticket to the queue and clean up agent state
    fn return_ticket_to_queue(&mut self, ticket_id: &str, session_name: &str) -> Result<()> {
        // Load ticket
        let queue = Queue::new(&self.config)?;
        let ticket = queue
            .get_in_progress_ticket(ticket_id)?
            .ok_or_else(|| anyhow::anyhow!("Ticket not found: {}", ticket_id))?;

        // Move ticket back to queue
        queue.return_to_queue(&ticket)?;

        // Remove agent from state
        let mut state = State::load(&self.config)?;
        state.remove_agent_by_session(session_name)?;

        // Send notification
        if self.config.notifications.enabled {
            notifications::send(
                "Ticket Returned to Queue",
                &ticket.project,
                &format!("{} - {}", ticket.id, ticket.summary),
                self.config.notifications.sound,
            )?;
        }

        Ok(())
    }

    /// Initialize the tickets directory with default templates and save config
    fn initialize_tickets(&mut self) -> Result<()> {
        let tickets_path = self.config.tickets_path();

        // Create directories
        fs::create_dir_all(tickets_path.join("queue"))?;
        fs::create_dir_all(tickets_path.join("in-progress"))?;
        fs::create_dir_all(tickets_path.join("completed"))?;
        fs::create_dir_all(tickets_path.join("templates"))?;
        fs::create_dir_all(tickets_path.join("operator"))?;

        // Get selected issuetype collection and configured fields from setup screen
        let (selected_preset, selected_collection, task_fields) = self
            .setup_screen
            .as_ref()
            .map(|s| (s.preset(), s.collection(), s.configured_task_fields()))
            .unwrap_or_else(|| {
                (
                    crate::config::CollectionPreset::Simple,
                    vec!["TASK".to_string()],
                    vec!["priority".to_string(), "context".to_string()],
                )
            });

        // Update config with selected preset and collection
        self.config.templates.preset = selected_preset;
        if selected_preset == crate::config::CollectionPreset::Custom {
            self.config.templates.collection = selected_collection.clone();
        } else {
            self.config.templates.collection.clear();
        }

        // Write template files (only for selected types)
        for template_type in TemplateType::all() {
            let type_str = template_type.as_str();
            if !selected_collection.contains(&type_str.to_string()) {
                continue;
            }

            let filename = match template_type {
                TemplateType::Feature => "feature.md",
                TemplateType::Fix => "fix.md",
                TemplateType::Task => "task.md",
                TemplateType::Spike => "spike.md",
                TemplateType::Investigation => "investigation.md",
                TemplateType::Assess => "assess.md",
                TemplateType::Sync => "sync.md",
                TemplateType::Init => "init.md",
            };
            let filepath = tickets_path.join("templates").join(filename);
            fs::write(&filepath, template_type.template_content())?;

            // Also write the JSON schema (with field filtering applied)
            let schema_filename = match template_type {
                TemplateType::Feature => "feature.json",
                TemplateType::Fix => "fix.json",
                TemplateType::Task => "task.json",
                TemplateType::Spike => "spike.json",
                TemplateType::Investigation => "investigation.json",
                TemplateType::Assess => "assess.json",
                TemplateType::Sync => "sync.json",
                TemplateType::Init => "init.json",
            };
            let schema_filepath = tickets_path.join("templates").join(schema_filename);
            let filtered_schema = filter_schema_fields(template_type.schema(), &task_fields)?;
            fs::write(&schema_filepath, filtered_schema)?;
        }

        // Generate tmux configuration files
        self.generate_tmux_config()?;

        // Discover projects (one-time scan during setup)
        let discovered_projects = self.config.discover_projects();

        // Update config with discovered projects and save
        self.config.projects = discovered_projects.clone();
        self.config.save()?;

        // Update the create dialog with discovered projects
        self.create_dialog.set_projects(discovered_projects.clone());

        // Create startup tickets based on user selections
        let startup_tickets = self
            .setup_screen
            .as_ref()
            .map(|s| s.selected_startup_tickets())
            .unwrap_or_default();

        if !startup_tickets.is_empty() {
            let projects_path = self.config.projects_path();
            for project in &discovered_projects {
                let project_path = projects_path.join(project);

                // ASSESS or PROJECT-INIT creates assess tickets
                if startup_tickets.contains(&"assess".to_string())
                    || startup_tickets.contains(&"project_init".to_string())
                {
                    match AssessTicketCreator::create_assess_ticket(
                        &project_path,
                        project,
                        &self.config,
                    ) {
                        Ok(result) => {
                            tracing::info!(
                                ticket_id = %result.ticket_id,
                                project = %project,
                                "Created ASSESS startup ticket"
                            );
                        }
                        Err(e) => {
                            tracing::warn!(project = %project, error = %e, "Failed to create ASSESS ticket");
                        }
                    }
                }

                // AGENT-SETUP or PROJECT-INIT creates agent tickets
                if startup_tickets.contains(&"agent_setup".to_string())
                    || startup_tickets.contains(&"project_init".to_string())
                {
                    match AgentTicketCreator::create_agent_tickets(
                        &project_path,
                        project,
                        &self.config,
                    ) {
                        Ok(result) => {
                            if !result.created.is_empty() {
                                tracing::info!(
                                    created = ?result.created,
                                    project = %project,
                                    "Created AGENT-SETUP startup tickets"
                                );
                            }
                        }
                        Err(e) => {
                            tracing::warn!(project = %project, error = %e, "Failed to create AGENT-SETUP tickets");
                        }
                    }
                }
            }
        }

        // Generate Backstage scaffold
        let backstage_path = self.config.backstage_path();
        if !BackstageScaffold::exists(&backstage_path) {
            let options = ScaffoldOptions::from_config(&self.config);
            let scaffold = BackstageScaffold::new(backstage_path, options);
            match scaffold.generate() {
                Ok(result) => {
                    tracing::info!(
                        created = result.created.len(),
                        skipped = result.skipped.len(),
                        "Generated Backstage scaffold: {}",
                        result.summary()
                    );
                }
                Err(e) => {
                    tracing::warn!("Failed to generate Backstage scaffold: {}", e);
                }
            }
        }

        Ok(())
    }

    /// Generate custom tmux config and status script
    fn generate_tmux_config(&mut self) -> Result<()> {
        use crate::agents::{generate_status_script, generate_tmux_conf};

        let state_path = self.config.state_path();
        let tmux_conf_path = self.config.tmux_config_path();
        let status_script_path = self.config.tmux_status_script_path();

        // Generate tmux.conf
        let tmux_conf_content = generate_tmux_conf(&status_script_path, &state_path);
        fs::write(&tmux_conf_path, tmux_conf_content)?;

        // Generate status script
        let status_script_content = generate_status_script();
        fs::write(&status_script_path, status_script_content)?;

        // Make status script executable
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&status_script_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&status_script_path, perms)?;
        }

        // Mark config as generated
        self.config.tmux.config_generated = true;

        tracing::info!(
            tmux_conf = %tmux_conf_path.display(),
            status_script = %status_script_path.display(),
            "Generated tmux configuration files"
        );

        Ok(())
    }

    /// Create a new ticket from the dialog result
    fn create_ticket(
        &mut self,
        dialog_result: CreateDialogResult,
        terminal: &mut AppTerminal,
    ) -> Result<()> {
        let config = self.config.clone();

        let result = with_suspended_tui(terminal, || {
            let creator = TicketCreator::new(&config);
            // Use the new method that accepts pre-filled values
            creator.create_ticket_with_values(dialog_result.template_type, &dialog_result.values)
        });

        // Handle result after TUI is restored
        match result {
            Ok(_) => {
                self.refresh_data()?;
            }
            Err(e) => {
                tracing::error!("Failed to create ticket: {}", e);
            }
        }

        Ok(())
    }

    /// Execute a project action (e.g., generating operator agents)
    fn execute_project_action(&mut self, result: ProjectsDialogResult) -> Result<()> {
        match result.action {
            ProjectAction::AddOperatorAgents => {
                // Create TASK tickets for missing operator agents
                let ticket_result = AgentTicketCreator::create_agent_tickets(
                    &result.project_path,
                    &result.project,
                    &self.config,
                );

                // Update dialog with result
                match ticket_result {
                    Ok(agent_result) => {
                        self.projects_dialog.set_creation_result(Ok(agent_result));
                    }
                    Err(e) => {
                        self.projects_dialog.set_creation_result(Err(e.to_string()));
                    }
                }
            }
            ProjectAction::AssessProject => {
                // Create ASSESS ticket for Backstage catalog assessment
                let ticket_result = AssessTicketCreator::create_assess_ticket(
                    &result.project_path,
                    &result.project,
                    &self.config,
                );

                // Convert to AgentTicketResult format for display
                match ticket_result {
                    Ok(assess_result) => {
                        use crate::agents::AgentTicketResult;
                        let agent_result = AgentTicketResult {
                            created: vec![assess_result.ticket_id],
                            skipped: vec![],
                            errors: vec![],
                        };
                        self.projects_dialog.set_creation_result(Ok(agent_result));
                    }
                    Err(e) => {
                        self.projects_dialog.set_creation_result(Err(e.to_string()));
                    }
                }
            }
        }

        Ok(())
    }
}

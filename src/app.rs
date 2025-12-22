use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::fs;
use std::io;
use std::time::Duration;

use crate::agents::tmux::SystemTmuxClient;
use crate::agents::{AgentTicketCreator, Launcher, SessionMonitor, TicketSessionSync};
use crate::api::Capabilities;
use crate::config::Config;
use crate::notifications;
use crate::queue::{Queue, TicketCreator};
use crate::state::State;
use crate::templates::TemplateType;
use crate::ui::create_dialog::{CreateDialog, CreateDialogResult};
use crate::ui::dialogs::HelpDialog;
use crate::ui::projects_dialog::{ProjectAction, ProjectsDialog, ProjectsDialogResult};
use crate::ui::session_preview::SessionPreview;
use crate::ui::setup::{SetupResult, SetupScreen};
use crate::ui::{ConfirmDialog, ConfirmSelection, Dashboard};
use std::sync::Arc;

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
}

impl App {
    pub fn new(config: Config) -> Result<Self> {
        let dashboard = Dashboard::new(&config);

        // Check if tickets directory exists
        let tickets_path = config.tickets_path();
        let needs_setup = !tickets_path.join("queue").exists();

        // For setup screen, we discover projects dynamically
        // After setup, we use the projects list from config
        let (setup_screen, projects_for_dialog) = if needs_setup {
            // Discover projects for the setup screen display
            let discovered_projects = config.discover_projects();
            let setup = SetupScreen::new(
                tickets_path.to_string_lossy().to_string(),
                discovered_projects.clone(),
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

        // Initialize ticket-session sync
        let tmux_client: Arc<dyn crate::agents::TmuxClient> = Arc::new(SystemTmuxClient::new());
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

        // Main loop
        let tick_rate = Duration::from_millis(self.config.ui.refresh_rate_ms);

        while !self.should_quit {
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
                }
            })?;

            // Handle events
            if event::poll(tick_rate)? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        self.handle_key(key.code).await?;
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

    async fn handle_key(&mut self, key: KeyCode) -> Result<()> {
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
                self.create_ticket(result)?;
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
            match key {
                KeyCode::Char('y') | KeyCode::Char('Y') => {
                    self.launch_confirmed().await?;
                }
                KeyCode::Char('v') | KeyCode::Char('V') => {
                    self.view_ticket()?;
                }
                KeyCode::Char('e') | KeyCode::Char('E') => {
                    self.edit_ticket()?;
                }
                KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                    self.confirm_dialog.hide();
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
                        self.view_ticket()?;
                    }
                    ConfirmSelection::No => {
                        self.confirm_dialog.hide();
                    }
                },
                _ => {}
            }
            return Ok(());
        }

        // Normal mode
        match key {
            KeyCode::Char('q') => {
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
            KeyCode::Char('L') | KeyCode::Char('l') | KeyCode::Enter => {
                self.try_launch()?;
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
            _ => {}
        }

        Ok(())
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

            // Show confirmation
            self.confirm_dialog.show(ticket);
        }

        Ok(())
    }

    async fn launch_confirmed(&mut self) -> Result<()> {
        if let Some(ticket) = self.confirm_dialog.ticket.take() {
            let launcher = Launcher::new(&self.config)?;
            launcher.launch(&ticket).await?;
            self.confirm_dialog.hide();
            self.refresh_data()?;
        }
        Ok(())
    }

    /// View ticket file in $VISUAL or with `open` command
    fn view_ticket(&mut self) -> Result<()> {
        let Some(filepath) = self.confirm_dialog.ticket_filepath() else {
            return Ok(());
        };

        // Temporarily exit TUI
        disable_raw_mode()?;
        execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;

        // Try $VISUAL first, then fall back to `open` (macOS)
        let result = if let Ok(visual) = std::env::var("VISUAL") {
            std::process::Command::new(&visual).arg(&filepath).status()
        } else {
            std::process::Command::new("open").arg(&filepath).status()
        };

        if let Err(e) = result {
            tracing::warn!("Failed to open file: {}", e);
        }

        // Restore TUI
        enable_raw_mode()?;
        execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;

        Ok(())
    }

    /// Edit ticket file in $EDITOR
    fn edit_ticket(&mut self) -> Result<()> {
        let Some(filepath) = self.confirm_dialog.ticket_filepath() else {
            return Ok(());
        };

        let Ok(editor) = std::env::var("EDITOR") else {
            // No EDITOR set, do nothing
            return Ok(());
        };

        // Temporarily exit TUI
        disable_raw_mode()?;
        execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;

        let result = std::process::Command::new(&editor).arg(&filepath).status();

        if let Err(e) = result {
            tracing::warn!("Failed to open editor: {}", e);
        }

        // Restore TUI
        enable_raw_mode()?;
        execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;

        Ok(())
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
        let tmux = SystemTmuxClient::new();
        let content = tmux
            .capture_pane(session_name, false)
            .map_err(|e| format!("Failed to capture session: {}", e));

        self.session_preview.show(&agent, content);

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

        // Get selected issuetype collection from setup screen
        let selected_collection = self
            .setup_screen
            .as_ref()
            .map(|s| s.collection())
            .unwrap_or_default();

        // Update config with selected collection
        self.config.templates.collection = selected_collection.clone();

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
            };
            let filepath = tickets_path.join("templates").join(filename);
            fs::write(&filepath, template_type.template_content())?;

            // Also write the JSON schema
            let schema_filename = match template_type {
                TemplateType::Feature => "feature.json",
                TemplateType::Fix => "fix.json",
                TemplateType::Task => "task.json",
                TemplateType::Spike => "spike.json",
                TemplateType::Investigation => "investigation.json",
            };
            let schema_filepath = tickets_path.join("templates").join(schema_filename);
            fs::write(&schema_filepath, template_type.schema())?;
        }

        // Discover projects (one-time scan during setup)
        let discovered_projects = self.config.discover_projects();

        // Update config with discovered projects and save
        self.config.projects = discovered_projects.clone();
        self.config.save()?;

        // Update the create dialog with discovered projects
        self.create_dialog.set_projects(discovered_projects);

        Ok(())
    }

    /// Create a new ticket from the dialog result
    fn create_ticket(&mut self, dialog_result: CreateDialogResult) -> Result<()> {
        // Need to temporarily exit TUI to open editor
        disable_raw_mode()?;
        execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;

        let creator = TicketCreator::new(&self.config);
        // Use the new method that accepts pre-filled values
        let result =
            creator.create_ticket_with_values(dialog_result.template_type, &dialog_result.values);

        // Restore TUI
        enable_raw_mode()?;
        execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;

        // Handle result after TUI is restored
        if let Err(e) = result {
            tracing::error!("Failed to create ticket: {}", e);
        } else {
            self.refresh_data()?;
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
        }

        Ok(())
    }
}

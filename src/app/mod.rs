use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::collections::HashMap;
use std::io;
use std::time::Duration;
use tokio::sync::{mpsc, RwLock};

use crate::agents::tmux::SystemTmuxClient;
use crate::agents::{SessionMonitor, TicketSessionSync};
use crate::backstage::BackstageServer;
use crate::config::Config;
use crate::issuetypes::IssueTypeRegistry;
use crate::notifications::NotificationService;
use crate::rest::RestApiServer;
use crate::services::{KanbanSyncService, PrMonitorService, PrStatusEvent, TrackedPr};
use crate::ui::create_dialog::CreateDialog;
use crate::ui::dialogs::HelpDialog;
use crate::ui::projects_dialog::ProjectsDialog;
use crate::ui::session_preview::SessionPreview;
use crate::ui::setup::{DetectedToolInfo, SetupScreen};
use crate::ui::{
    CollectionSwitchDialog, ConfirmDialog, Dashboard, KanbanView, SessionRecoveryDialog,
    SyncConfirmDialog, TerminalGuard,
};
use std::sync::Arc;

mod agents;
mod data_sync;
mod kanban;
mod keyboard;
mod pr_workflow;
mod review;
mod session;
mod tickets;

#[cfg(test)]
mod tests;

/// Type alias for the terminal used by the app
type AppTerminal = Terminal<CrosstermBackend<io::Stdout>>;

pub struct App {
    pub(crate) config: Config,
    pub(crate) dashboard: Dashboard,
    pub(crate) confirm_dialog: ConfirmDialog,
    pub(crate) help_dialog: HelpDialog,
    pub(crate) create_dialog: CreateDialog,
    pub(crate) projects_dialog: ProjectsDialog,
    pub(crate) setup_screen: Option<SetupScreen>,
    pub(crate) should_quit: bool,
    /// Message to print on exit (for unimplemented features)
    pub(crate) exit_message: Option<String>,
    /// Session health monitor
    pub(crate) session_monitor: SessionMonitor,
    /// Session preview dialog
    pub(crate) session_preview: SessionPreview,
    /// Ticket-session synchronizer
    pub(crate) ticket_sync: TicketSessionSync,
    /// Last sync status message for display
    pub(crate) sync_status_message: Option<String>,
    /// Backstage server lifecycle manager
    pub(crate) backstage_server: BackstageServer,
    /// REST API server lifecycle manager
    pub(crate) rest_api_server: RestApiServer,
    /// Exit confirmation mode (first Ctrl+C pressed)
    pub(crate) exit_confirmation_mode: bool,
    /// Time when exit confirmation mode was entered
    pub(crate) exit_confirmation_time: Option<std::time::Instant>,
    /// Start web servers on launch (--web flag)
    pub(crate) start_web_on_launch: bool,
    /// Session recovery dialog for handling dead tmux sessions
    pub(crate) session_recovery_dialog: SessionRecoveryDialog,
    /// Collection switch dialog for changing active issue type collection
    pub(crate) collection_dialog: CollectionSwitchDialog,
    /// Kanban providers view for syncing external issues
    pub(crate) kanban_view: KanbanView,
    /// Kanban sync confirmation dialog
    pub(crate) sync_confirm_dialog: SyncConfirmDialog,
    /// Kanban sync service
    pub(crate) kanban_sync_service: KanbanSyncService,
    /// Issue type registry for dynamic issue types
    pub(crate) issue_type_registry: IssueTypeRegistry,
    /// Receiver for PR status events from the background monitor
    pub(crate) pr_event_rx: mpsc::UnboundedReceiver<PrStatusEvent>,
    /// Shared access to tracked PRs (for adding new PRs from sync)
    pub(crate) pr_tracked: Arc<RwLock<HashMap<String, TrackedPr>>>,
    /// Shutdown signal sender for PR monitor
    pub(crate) pr_shutdown_tx: Option<mpsc::Sender<()>>,
    /// Notification service for dispatching events to integrations
    pub(crate) notification_service: NotificationService,
    /// Shared tmux client for agent operations (switching, etc.)
    pub(crate) tmux_client: Arc<dyn crate::agents::TmuxClient>,
    /// Latest version available (if update notification shown)
    pub(crate) update_available_version: Option<String>,
    /// Time when update notification was first shown
    pub(crate) update_notification_shown_at: Option<std::time::Instant>,
    /// Receiver for version check results
    pub(crate) version_rx: mpsc::UnboundedReceiver<String>,
    /// True if REST API port was in use at startup (another instance may be running)
    pub(crate) api_port_conflict: bool,
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
        let ticket_sync = TicketSessionSync::new(&config, Arc::clone(&tmux_client));

        // Initialize Backstage server lifecycle manager using compiled binary mode
        let backstage_server = BackstageServer::with_compiled_binary(
            config.state_path(),
            config.backstage.release_url.clone(),
            config.backstage.local_binary_path.clone(),
            config.backstage.port,
        )
        .map_err(|e| anyhow::anyhow!("Failed to initialize backstage server: {e}"))?;

        // Initialize REST API server lifecycle manager
        let rest_api_server = RestApiServer::new(config.clone(), config.rest_api.port);

        // Initialize issue type registry
        let mut issue_type_registry = IssueTypeRegistry::new();
        if let Err(e) = issue_type_registry.load_all(&config.tickets_path()) {
            tracing::warn!("Failed to load issue types: {}", e);
        }

        // Activate configured collection if specified
        if let Some(ref active) = config.templates.active_collection {
            if let Err(e) = issue_type_registry.activate_collection(active) {
                tracing::warn!("Failed to activate collection '{}': {}", active, e);
            }
        }

        // Initialize notification service
        let notification_service = NotificationService::from_config(&config)?;

        // Initialize PR monitor channels (monitor will be spawned in run())
        let (pr_event_tx, pr_event_rx) = mpsc::unbounded_channel();
        let (pr_shutdown_tx, pr_shutdown_rx) = mpsc::channel(1);

        // Initialize version check channel
        let (version_tx, version_rx) = mpsc::unbounded_channel();

        // Create PR monitor service and get shared access to tracked PRs
        let mut pr_monitor = PrMonitorService::new(pr_event_tx)
            .with_poll_interval(Duration::from_secs(config.api.pr_check_interval_secs))
            .with_shutdown(pr_shutdown_rx);
        let pr_tracked = pr_monitor.tracked_prs();

        // Spawn PR monitor as background task
        tokio::spawn(async move {
            if let Err(e) = pr_monitor.run().await {
                tracing::error!("PR monitor error: {}", e);
            }
        });

        // Spawn background version check
        if config.version_check.enabled {
            let check_config = config.version_check.clone();
            let tx = version_tx;
            tokio::spawn(async move {
                if let Some(new_version) = crate::version::check_for_updates(&check_config).await {
                    let _ = tx.send(new_version);
                }
            });
        }

        let kanban_sync_service = KanbanSyncService::new(&config);

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
            sync_status_message: None,
            backstage_server,
            rest_api_server,
            exit_confirmation_mode: false,
            exit_confirmation_time: None,
            start_web_on_launch: start_web,
            session_recovery_dialog: SessionRecoveryDialog::new(),
            collection_dialog: CollectionSwitchDialog::new(),
            kanban_view: KanbanView::new(),
            sync_confirm_dialog: SyncConfirmDialog::new(),
            kanban_sync_service,
            issue_type_registry,
            pr_event_rx,
            pr_tracked,
            pr_shutdown_tx: Some(pr_shutdown_tx),
            notification_service,
            update_available_version: None,
            update_notification_shown_at: None,
            version_rx,
            api_port_conflict: false,
            tmux_client,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        // Reconcile state with actual tmux sessions on startup
        self.reconcile_sessions()?;

        // Terminal guard handles setup and cleanup on drop
        // This ensures terminal is restored even on early returns via `?` or panics
        let _terminal_guard = TerminalGuard::new()?;

        let stdout = io::stdout();
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // Initial data load
        self.refresh_data()?;

        // Always try to start REST API (unless disabled in config)
        if self.config.rest_api.enabled {
            // Check if port is already in use (another operator instance may be running)
            if self.rest_api_server.is_port_in_use().await {
                self.api_port_conflict = true;
                tracing::warn!(
                    port = self.config.rest_api.port,
                    "REST API port is already in use. Another operator instance may be running from this .tickets/ directory."
                );
            } else if let Err(e) = self.rest_api_server.start() {
                tracing::error!("REST API start failed: {}", e);
            }
        }

        // Start Backstage web server if --web flag was passed
        if self.start_web_on_launch {
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

            // Update dashboard with version notification status
            let version_to_show = if self.update_notification_shown_at.is_some() {
                self.update_available_version.clone()
            } else {
                None
            };
            self.dashboard.update_available_version(version_to_show);

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
                    self.collection_dialog.render(f);
                    if self.kanban_view.visible {
                        self.kanban_view.render(f, f.area());
                    }
                    self.sync_confirm_dialog.render(f);
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
                            self.handle_ctrl_c().await;
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

            // Check for PR status events (non-blocking)
            self.handle_pr_events().await?;

            // Process any pending PR creations
            self.process_pending_pr_creations().await?;

            // Check for version check results (non-blocking)
            if let Ok(new_version) = self.version_rx.try_recv() {
                self.update_available_version = Some(new_version);
                self.update_notification_shown_at = Some(std::time::Instant::now());
            }

            // Auto-dismiss version notification after 6 seconds
            if let Some(shown_at) = self.update_notification_shown_at {
                if shown_at.elapsed() > Duration::from_secs(6) {
                    self.update_notification_shown_at = None;
                }
            }
        }

        // Terminal cleanup is handled by _terminal_guard drop

        // Check for exit message (unimplemented features)
        if let Some(message) = &self.exit_message {
            eprintln!("{message}");
            std::process::exit(1);
        }

        Ok(())
    }
}

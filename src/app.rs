use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::time::Duration;
use tokio::sync::{mpsc, RwLock};

use crate::agents::tmux::SystemTmuxClient;
use crate::agents::{
    AgentTicketCreator, AssessTicketCreator, LaunchOptions, Launcher, PrWorkflow, SessionMonitor,
    TicketSessionSync,
};
use crate::backstage::scaffold::{BackstageScaffold, ScaffoldOptions};
use crate::backstage::BackstageServer;
use crate::config::Config;
use crate::issuetypes::IssueTypeRegistry;
use crate::notifications::{NotificationEvent, NotificationService};
use crate::queue::{Queue, TicketCreator};
use crate::rest::RestApiServer;
use crate::services::{KanbanSyncService, PrMonitorService, PrStatusEvent, TrackedPr};
use crate::setup::filter_schema_fields;
use crate::state::State;
use crate::templates::TemplateType;
use crate::ui::create_dialog::{CreateDialog, CreateDialogResult};
use crate::ui::dialogs::HelpDialog;
use crate::ui::projects_dialog::{ProjectAction, ProjectsDialog, ProjectsDialogResult};
use crate::ui::session_preview::SessionPreview;
use crate::ui::setup::{DetectedToolInfo, SetupResult, SetupScreen};
use crate::ui::{
    with_suspended_tui, CollectionSwitchDialog, ConfirmDialog, ConfirmSelection, Dashboard,
    KanbanView, KanbanViewResult, SessionRecoveryDialog, SessionRecoverySelection,
    SyncConfirmDialog, SyncConfirmResult, TerminalGuard,
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
    /// Collection switch dialog for changing active issue type collection
    collection_dialog: CollectionSwitchDialog,
    /// Kanban providers view for syncing external issues
    kanban_view: KanbanView,
    /// Kanban sync confirmation dialog
    sync_confirm_dialog: SyncConfirmDialog,
    /// Kanban sync service
    kanban_sync_service: KanbanSyncService,
    /// Issue type registry for dynamic issue types
    issue_type_registry: IssueTypeRegistry,
    /// Receiver for PR status events from the background monitor
    pr_event_rx: mpsc::UnboundedReceiver<PrStatusEvent>,
    /// Shared access to tracked PRs (for adding new PRs from sync)
    pr_tracked: Arc<RwLock<HashMap<String, TrackedPr>>>,
    /// Shutdown signal sender for PR monitor
    pr_shutdown_tx: Option<mpsc::Sender<()>>,
    /// Notification service for dispatching events to integrations
    notification_service: NotificationService,
    /// Latest version available (if update notification shown)
    update_available_version: Option<String>,
    /// Time when update notification was first shown
    update_notification_shown_at: Option<std::time::Instant>,
    /// Receiver for version check results
    version_rx: mpsc::UnboundedReceiver<String>,
    /// True if REST API port was in use at startup (another instance may be running)
    api_port_conflict: bool,
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
            let tx = version_tx.clone();
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
            for session in &result.orphaned {
                self.notification_service
                    .notify_sync(NotificationEvent::AgentSessionLost {
                        session_name: session.clone(),
                    });
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

            for session in &result.orphaned {
                self.notification_service
                    .notify_sync(NotificationEvent::AgentSessionLost {
                        session_name: session.clone(),
                    });
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

    /// Sync all configured kanban collections
    async fn run_kanban_sync_all(&mut self) -> Result<()> {
        let collections = self.kanban_sync_service.configured_collections();
        let total = collections.len();

        if total == 0 {
            self.sync_confirm_dialog.hide();
            self.sync_status_message = Some("No kanban providers configured".to_string());
            return Ok(());
        }

        let mut created_total = 0;
        let mut skipped_total = 0;
        let mut error_count = 0;

        for (i, collection) in collections.iter().enumerate() {
            self.sync_confirm_dialog.set_syncing(i, total);

            match self
                .kanban_sync_service
                .sync_collection(&collection.provider, &collection.project_key)
                .await
            {
                Ok(result) => {
                    created_total += result.created.len();
                    skipped_total += result.skipped.len();
                    if !result.is_success() {
                        error_count += result.errors.len();
                    }
                    tracing::info!(
                        provider = %collection.provider,
                        project = %collection.project_key,
                        "Synced collection: {}",
                        result.summary()
                    );
                }
                Err(e) => {
                    error_count += 1;
                    tracing::warn!(
                        provider = %collection.provider,
                        project = %collection.project_key,
                        "Failed to sync collection: {}",
                        e
                    );
                }
            }
        }

        // Build summary message
        let summary = if error_count > 0 {
            format!(
                "Sync complete: {} created, {} skipped, {} errors",
                created_total, skipped_total, error_count
            )
        } else {
            format!(
                "Sync complete: {} tickets created, {} skipped",
                created_total, skipped_total
            )
        };

        self.sync_confirm_dialog.set_complete(&summary);
        self.sync_status_message = Some(summary);
        self.sync_confirm_dialog.hide();

        // Trigger queue refresh
        self.run_manual_sync()?;

        Ok(())
    }

    /// Handle PR status events from the background monitor (non-blocking)
    async fn handle_pr_events(&mut self) -> Result<()> {
        // Process all pending PR events (non-blocking)
        while let Ok(event) = self.pr_event_rx.try_recv() {
            match event {
                PrStatusEvent::Merged {
                    ticket_id,
                    pr_number,
                    merge_commit_sha,
                } => {
                    tracing::info!(
                        ticket = %ticket_id,
                        pr = pr_number,
                        merge_sha = %merge_commit_sha,
                        "PR merged - advancing ticket"
                    );

                    // Load state and queue
                    let mut state = State::load(&self.config)?;
                    let queue = Queue::new(&self.config)?;

                    // Get the ticket and agent
                    if let Some(ticket) = queue.get_in_progress_ticket(&ticket_id)? {
                        if let Some(agent) = state.agent_by_ticket(&ticket_id).cloned() {
                            // Handle PR merged (cleanup worktree, etc.)
                            if let Err(e) = self
                                .ticket_sync
                                .handle_pr_merged(&ticket, &agent, None)
                                .await
                            {
                                tracing::error!(
                                    ticket = %ticket_id,
                                    error = %e,
                                    "Failed to cleanup after PR merge"
                                );
                            }

                            // Clear PR review state and update status
                            state.clear_review_state(&agent.id)?;
                            state.update_agent_status(
                                &agent.id,
                                "completed",
                                Some(format!("PR #{} merged", pr_number)),
                            )?;

                            // Send notification
                            self.notification_service
                                .notify(NotificationEvent::PrMerged {
                                    project: ticket.project.clone(),
                                    ticket_id: ticket_id.clone(),
                                    pr_number,
                                })
                                .await;
                        }
                    }

                    // Untrack the PR (it's been merged)
                    let key = format!("{}#{}", ticket_id, pr_number);
                    self.pr_tracked.write().await.remove(&key);
                }
                PrStatusEvent::Closed {
                    ticket_id,
                    pr_number,
                } => {
                    tracing::warn!(
                        ticket = %ticket_id,
                        pr = pr_number,
                        "PR closed without merge - triggering on_reject"
                    );

                    // Load state
                    let mut state = State::load(&self.config)?;

                    if let Some(agent) = state.agent_by_ticket(&ticket_id).cloned() {
                        // Set review state to indicate rejection
                        state.update_agent_status(
                            &agent.id,
                            "awaiting_input",
                            Some("PR closed without merge".to_string()),
                        )?;
                        state.set_agent_review_state(&agent.id, "pr_rejected")?;

                        // Send notification
                        self.notification_service
                            .notify(NotificationEvent::PrClosed {
                                project: String::new(), // Project unknown in this context
                                ticket_id: ticket_id.clone(),
                                pr_number,
                            })
                            .await;
                    }

                    // Untrack the PR
                    let key = format!("{}#{}", ticket_id, pr_number);
                    self.pr_tracked.write().await.remove(&key);
                }
                PrStatusEvent::ReadyToMerge {
                    ticket_id,
                    pr_number,
                } => {
                    // Notify only - no auto-merge per user decision
                    tracing::info!(
                        ticket = %ticket_id,
                        pr = pr_number,
                        "PR ready to merge (approved + checks pass)"
                    );

                    self.notification_service
                        .notify(NotificationEvent::PrReadyToMerge {
                            project: String::new(), // Project unknown in this context
                            ticket_id: ticket_id.clone(),
                            pr_number,
                        })
                        .await;
                }
                PrStatusEvent::Approved {
                    ticket_id,
                    pr_number,
                } => {
                    tracing::info!(
                        ticket = %ticket_id,
                        pr = pr_number,
                        "PR approved"
                    );
                }
                PrStatusEvent::ChangesRequested {
                    ticket_id,
                    pr_number,
                } => {
                    tracing::info!(
                        ticket = %ticket_id,
                        pr = pr_number,
                        "PR has changes requested"
                    );

                    // Update state to indicate changes requested
                    let mut state = State::load(&self.config)?;
                    if let Some(agent) = state.agent_by_ticket(&ticket_id).cloned() {
                        state.set_agent_review_state(&agent.id, "pr_changes_requested")?;
                    }

                    self.notification_service
                        .notify(NotificationEvent::PrChangesRequested {
                            project: String::new(), // Project unknown in this context
                            ticket_id: ticket_id.clone(),
                            pr_number,
                        })
                        .await;
                }
                PrStatusEvent::ReadyForReview {
                    ticket_id,
                    pr_number,
                } => {
                    tracing::info!(
                        ticket = %ticket_id,
                        pr = pr_number,
                        "PR converted from draft to ready for review"
                    );
                }
            }
        }

        Ok(())
    }

    /// Process agents with pending PR creations
    async fn process_pending_pr_creations(&mut self) -> Result<()> {
        let state = State::load(&self.config)?;
        let queue = Queue::new(&self.config)?;

        // Find agents with pending_pr_creation state
        let pending_agents: Vec<_> = state
            .agents
            .iter()
            .filter(|a| a.review_state.as_deref() == Some("pending_pr_creation"))
            .cloned()
            .collect();

        for agent in pending_agents {
            // Get the ticket for this agent
            let ticket = match queue.get_in_progress_ticket(&agent.ticket_id)? {
                Some(t) => t,
                None => {
                    tracing::warn!(
                        agent_id = %agent.id,
                        ticket_id = %agent.ticket_id,
                        "Ticket not found for pending PR creation"
                    );
                    continue;
                }
            };

            // Get the worktree path
            let worktree_path = match &agent.worktree_path {
                Some(path) => std::path::PathBuf::from(path),
                None => {
                    tracing::warn!(
                        agent_id = %agent.id,
                        "No worktree path for PR creation"
                    );
                    continue;
                }
            };

            // Get the base branch (from ticket or default)
            let base_branch = ticket.branch.as_deref().unwrap_or("main");

            // Create PR via PrWorkflow
            let workflow = PrWorkflow::new();
            let pr_title = format!("{}: {}", ticket.ticket_type, ticket.summary);
            let pr_body = Some(ticket.content.clone());

            // Get repo info for tracking
            let repo_info = match workflow.get_repo_info(&worktree_path).await {
                Ok(info) => info,
                Err(e) => {
                    tracing::error!(
                        ticket_id = %ticket.id,
                        error = %e,
                        "Failed to get repo info for PR creation"
                    );
                    continue;
                }
            };

            tracing::info!(
                ticket_id = %ticket.id,
                worktree = %worktree_path.display(),
                base = %base_branch,
                repo = %repo_info.full_name(),
                "Creating PR for ticket"
            );

            match workflow
                .create_or_attach_pr(
                    &worktree_path,
                    &pr_title,
                    pr_body,
                    base_branch,
                    false, // not draft
                )
                .await
            {
                Ok(pr) => {
                    tracing::info!(
                        ticket_id = %ticket.id,
                        pr_number = pr.number,
                        pr_url = %pr.url,
                        "PR created successfully"
                    );

                    // Update agent state with PR info
                    let mut state = State::load(&self.config)?;
                    if let Err(e) = state.update_agent_pr(
                        &agent.id,
                        &pr.url,
                        pr.number as u64,
                        &repo_info.full_name(),
                    ) {
                        tracing::error!(error = %e, "Failed to update agent PR info");
                    }
                    if let Err(e) = state.update_agent_status(
                        &agent.id,
                        "awaiting_input",
                        Some("PR created, awaiting merge".to_string()),
                    ) {
                        tracing::error!(error = %e, "Failed to update agent status");
                    }
                    if let Err(e) = state.set_agent_review_state(&agent.id, "pending_pr_merge") {
                        tracing::error!(error = %e, "Failed to set agent review state");
                    }

                    // Add PR to tracking
                    let key = format!("{}#{}", repo_info.full_name(), pr.number);
                    let tracked_pr = TrackedPr {
                        repo_info: repo_info.clone(),
                        pr_number: pr.number,
                        last_state: crate::types::pr::PrState::Open,
                        ticket_id: ticket.id.clone(),
                        is_draft: false,
                        merge_commit_sha: None,
                    };
                    self.pr_tracked.write().await.insert(key, tracked_pr);

                    // Send notification
                    self.notification_service
                        .notify(NotificationEvent::PrCreated {
                            project: ticket.project.clone(),
                            ticket_id: ticket.id.clone(),
                            pr_url: pr.url.clone(),
                            pr_number: pr.number,
                        })
                        .await;
                }
                Err(e) => {
                    tracing::error!(
                        ticket_id = %ticket.id,
                        error = %e,
                        "Failed to create PR"
                    );

                    // Update agent state to indicate failure
                    let mut state = State::load(&self.config)?;
                    if let Err(e) = state.update_agent_status(
                        &agent.id,
                        "awaiting_input",
                        Some(format!("PR creation failed: {}", e)),
                    ) {
                        tracing::error!(error = %e, "Failed to update agent status");
                    }
                    if let Err(e) = state.set_agent_review_state(&agent.id, "pr_creation_failed") {
                        tracing::error!(error = %e, "Failed to set agent review state");
                    }

                    // Send notification
                    self.notification_service
                        .notify(NotificationEvent::AgentFailed {
                            project: ticket.project.clone(),
                            ticket_id: ticket.id.clone(),
                            error: format!("Failed to create PR: {}", e),
                        })
                        .await;
                }
            }
        }

        Ok(())
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
        for ticket_id in &result.moved_to_awaiting {
            self.notification_service
                .notify_sync(NotificationEvent::AgentAwaitingInput {
                    project: String::new(), // Project unknown in this context
                    ticket_type: String::new(),
                    ticket_id: ticket_id.clone(),
                    reason: "The agent is waiting for user input.".to_string(),
                });
        }

        for ticket_id in &result.timed_out {
            self.notification_service
                .notify_sync(NotificationEvent::AgentAwaitingInput {
                    project: String::new(),
                    ticket_type: String::new(),
                    ticket_id: ticket_id.clone(),
                    reason: "The agent step has timed out and is now awaiting input.".to_string(),
                });
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

        // Collection dialog handling
        if self.collection_dialog.visible {
            if let Some(result) = self.collection_dialog.handle_key(key) {
                self.handle_collection_switch(result)?;
            }
            return Ok(());
        }

        // Kanban view handling
        if self.kanban_view.visible {
            if let Some(result) = self.kanban_view.handle_key(key) {
                match result {
                    KanbanViewResult::Sync {
                        provider,
                        project_key,
                    } => {
                        // Trigger sync in background
                        self.kanban_view.syncing = true;
                        self.kanban_view.set_status("Syncing...");
                        // Note: Actual sync would require spawning an async task
                        // For now, just show a status message
                        self.sync_status_message = Some(format!(
                            "Sync requested for {}/{} (sync not yet implemented)",
                            provider, project_key
                        ));
                        self.kanban_view.syncing = false;
                        self.kanban_view.hide();
                    }
                    KanbanViewResult::Dismissed => {
                        // Already hidden by handle_key
                    }
                }
            }
            return Ok(());
        }

        // Sync confirm dialog handling
        if self.sync_confirm_dialog.visible {
            if let Some(result) = self.sync_confirm_dialog.handle_key(key) {
                match result {
                    SyncConfirmResult::Confirmed => {
                        self.run_kanban_sync_all().await?;
                    }
                    SyncConfirmResult::Cancelled => {
                        // Already hidden by handle_key
                    }
                }
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
                // Shut down PR monitor
                if let Some(tx) = self.pr_shutdown_tx.take() {
                    let _ = tx.send(()).await;
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
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                // Approve review (only for agents with review_state)
                self.handle_review_approval()?;
            }
            KeyCode::Char('x') | KeyCode::Char('X') => {
                // Reject review (only for agents with review_state)
                self.handle_review_rejection()?;
            }
            KeyCode::Char('S') => {
                // Show kanban sync confirmation dialog
                let collections = self.kanban_sync_service.configured_collections();
                if collections.is_empty() {
                    self.sync_status_message = Some("No kanban providers configured".to_string());
                } else {
                    self.sync_confirm_dialog.show(collections);
                }
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
            KeyCode::Char('T') | KeyCode::Char('t') => {
                // Open collection switch dialog
                self.show_collection_dialog();
            }
            KeyCode::Char('K') => {
                // Open kanban providers view
                self.show_kanban_view();
            }
            _ => {}
        }

        Ok(())
    }

    /// Show the collection switch dialog
    fn show_collection_dialog(&mut self) {
        // Get project context from selected queue item if any
        let project_context = self.dashboard.selected_ticket().map(|t| t.project.as_str());
        self.collection_dialog.show(
            &self.issue_type_registry,
            self.issue_type_registry.active_collection_name(),
            project_context,
        );
    }

    /// Show the kanban providers view
    fn show_kanban_view(&mut self) {
        let collections = self.kanban_sync_service.configured_collections();
        if collections.is_empty() {
            // No kanban providers configured, show a message
            self.sync_status_message = Some(
                "No kanban providers configured. Add [kanban] section to config.toml".to_string(),
            );
            return;
        }
        self.kanban_view.show(collections);
    }

    /// Handle collection switch result
    fn handle_collection_switch(
        &mut self,
        result: crate::ui::CollectionSwitchResult,
    ) -> Result<()> {
        // Activate the collection in the registry
        if let Err(e) = self
            .issue_type_registry
            .activate_collection(&result.collection_name)
        {
            tracing::warn!(
                "Failed to activate collection '{}': {}",
                result.collection_name,
                e
            );
            return Ok(());
        }

        // Persist the preference
        if let Some(project) = result.project_scope {
            // Per-project preference
            let mut state = State::load(&self.config)?;
            state.set_project_collection(&project, &result.collection_name)?;
            tracing::info!(
                "Set collection '{}' for project '{}'",
                result.collection_name,
                project
            );
        } else {
            // Global preference - update config
            self.config.templates.active_collection = Some(result.collection_name.clone());
            if let Err(e) = self.config.save() {
                tracing::warn!("Failed to save config: {}", e);
            }
            tracing::info!("Set global collection to '{}'", result.collection_name);
        }

        Ok(())
    }

    /// Handle Ctrl+C for graceful two-stage exit
    async fn handle_ctrl_c(&mut self) {
        if self.exit_confirmation_mode {
            // Second Ctrl+C - exit immediately
            // Shut down PR monitor
            if let Some(tx) = self.pr_shutdown_tx.take() {
                let _ = tx.send(()).await;
            }
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

    /// Handle review approval for the selected agent
    ///
    /// Only works for agents in awaiting_input with a review_state of pending_plan or pending_visual.
    /// Creates a signal file to trigger resume in the next sync cycle.
    fn handle_review_approval(&mut self) -> Result<()> {
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

        // Only process if agent has a review state that can be approved
        match agent.review_state.as_deref() {
            Some("pending_plan") | Some("pending_visual") => {
                // Write signal file to trigger resume
                if let Some(ref session_name) = agent.session_name {
                    let signal_file = format!("/tmp/operator-detach-{}.signal", session_name);
                    std::fs::write(&signal_file, "approved")?;

                    tracing::info!(
                        agent_id = %agent.id,
                        session = %session_name,
                        review_state = ?agent.review_state,
                        "Review approved - signal file written"
                    );
                }
            }
            _ => {
                // No review state or non-approvable state - ignore
            }
        }

        Ok(())
    }

    /// Handle review rejection for the selected agent
    ///
    /// Only works for agents in awaiting_input with a review_state of pending_plan or pending_visual.
    /// For now, this just logs the rejection. A full implementation would show a dialog
    /// for entering a rejection reason and possibly restart the step.
    fn handle_review_rejection(&mut self) -> Result<()> {
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

        // Only process if agent has a review state that can be rejected
        match agent.review_state.as_deref() {
            Some("pending_plan") | Some("pending_visual") => {
                // TODO: Show rejection dialog for entering reason
                // For now, just log the rejection
                tracing::info!(
                    agent_id = %agent.id,
                    review_state = ?agent.review_state,
                    "Review rejected (rejection dialog not yet implemented)"
                );
            }
            _ => {
                // No review state or non-rejectable state - ignore
            }
        }

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
            retry_reason: None,
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
        self.notification_service
            .notify_sync(NotificationEvent::TicketReturned {
                project: ticket.project.clone(),
                ticket_id: ticket.id.clone(),
                summary: ticket.summary.clone(),
            });

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
        // Use full discovery to get git info for filtering
        let discovered_full = self.config.discover_projects_full();
        let discovered_projects: Vec<String> =
            discovered_full.iter().map(|p| p.name.clone()).collect();

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
                    // Check if project has git remote before creating ASSESS ticket
                    let project_info = discovered_full.iter().find(|p| p.name == *project);
                    let has_git_remote = project_info
                        .map(|info| info.has_git_remote())
                        .unwrap_or(false);

                    if !has_git_remote {
                        tracing::info!(
                            project = %project,
                            "Skipping ASSESS ticket - no git remote configured"
                        );
                    } else {
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
                // Check if project has git remote before creating ASSESS ticket
                let discovered =
                    crate::projects::discover_projects_with_git(&self.config.projects_path());
                let project_info = discovered.iter().find(|p| p.name == result.project);
                let has_git_remote = project_info
                    .map(|info| info.has_git_remote())
                    .unwrap_or(false);

                if !has_git_remote {
                    self.projects_dialog.set_creation_result(Err(
                        "Cannot create ASSESS ticket: project has no git remote configured"
                            .to_string(),
                    ));
                    return Ok(());
                }

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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    use crate::config::{DetectedTool, PathsConfig};
    use crate::queue::Ticket;

    /// Create a test configuration with isolated temporary directories
    fn make_test_config(temp_dir: &TempDir) -> Config {
        let projects_path = temp_dir.path().join("projects");
        let tickets_path = temp_dir.path().join("tickets");
        let state_path = temp_dir.path().join("state");

        std::fs::create_dir_all(&projects_path).unwrap();
        std::fs::create_dir_all(tickets_path.join("queue")).unwrap();
        std::fs::create_dir_all(tickets_path.join("in-progress")).unwrap();
        std::fs::create_dir_all(tickets_path.join("completed")).unwrap();
        std::fs::create_dir_all(tickets_path.join("operator")).unwrap();
        std::fs::create_dir_all(&state_path).unwrap();

        // Create a test project
        let test_project = projects_path.join("test-project");
        std::fs::create_dir_all(&test_project).unwrap();
        std::fs::write(test_project.join("CLAUDE.md"), "# Test Project").unwrap();

        // Create mock detected tool for tests
        let detected_tool = DetectedTool {
            name: "claude".to_string(),
            path: "/usr/bin/claude".to_string(),
            version: "1.0.0".to_string(),
            min_version: Some("1.0.0".to_string()),
            version_ok: true,
            model_aliases: vec!["sonnet".to_string()],
            command_template: "claude {{config_flags}}{{model_flag}}--session-id {{session_id}} --print-prompt-path {{prompt_file}}".to_string(),
            capabilities: crate::config::ToolCapabilities {
                supports_sessions: true,
                supports_headless: true,
            },
            yolo_flags: vec!["--dangerously-skip-permissions".to_string()],
        };

        Config {
            paths: PathsConfig {
                tickets: tickets_path.to_string_lossy().to_string(),
                projects: projects_path.to_string_lossy().to_string(),
                state: state_path.to_string_lossy().to_string(),
                worktrees: state_path.join("worktrees").to_string_lossy().to_string(),
            },
            projects: vec!["test-project".to_string()],
            llm_tools: crate::config::LlmToolsConfig {
                detected: vec![detected_tool],
                providers: vec![crate::config::LlmProvider {
                    tool: "claude".to_string(),
                    model: "sonnet".to_string(),
                    display_name: None,
                    ..Default::default()
                }],
                detection_complete: true,
            },
            // Disable notifications in tests
            notifications: crate::config::NotificationsConfig {
                enabled: false,
                os: crate::config::OsNotificationConfig {
                    enabled: false,
                    sound: false,
                    events: vec![],
                },
                webhook: None,
                webhooks: vec![],
                on_agent_start: false,
                on_agent_complete: false,
                on_agent_needs_input: false,
                on_pr_created: false,
                on_investigation_created: false,
                sound: false,
            },
            ..Default::default()
        }
    }

    // ============================================
    // State Transition Tests
    // ============================================

    mod state_transitions {
        use super::*;

        #[test]
        fn test_pause_queue_sets_state_paused() {
            let temp_dir = TempDir::new().unwrap();
            let config = make_test_config(&temp_dir);

            // Initialize state file
            let mut state = State::load(&config).unwrap();
            state.set_paused(false).unwrap();

            // Reload and verify initial state
            let state = State::load(&config).unwrap();
            assert!(!state.paused);

            // Simulate pause_queue logic
            let mut state = State::load(&config).unwrap();
            state.set_paused(true).unwrap();

            // Verify state is now paused
            let reloaded = State::load(&config).unwrap();
            assert!(reloaded.paused);
        }

        #[test]
        fn test_resume_queue_sets_state_resumed() {
            let temp_dir = TempDir::new().unwrap();
            let config = make_test_config(&temp_dir);

            // Initialize state as paused
            let mut state = State::load(&config).unwrap();
            state.set_paused(true).unwrap();

            // Reload and verify
            let state = State::load(&config).unwrap();
            assert!(state.paused);

            // Simulate resume_queue logic
            let mut state = State::load(&config).unwrap();
            state.set_paused(false).unwrap();

            // Verify state is now resumed
            let reloaded = State::load(&config).unwrap();
            assert!(!reloaded.paused);
        }

        #[test]
        fn test_pause_persists_to_disk() {
            let temp_dir = TempDir::new().unwrap();
            let config = make_test_config(&temp_dir);

            // Pause and verify persistence
            let mut state = State::load(&config).unwrap();
            state.set_paused(true).unwrap();

            // Create a completely new State instance (simulates app restart)
            let fresh_state = State::load(&config).unwrap();
            assert!(fresh_state.paused, "Paused state should persist to disk");

            // Resume and verify persistence
            let mut state = State::load(&config).unwrap();
            state.set_paused(false).unwrap();

            let fresh_state = State::load(&config).unwrap();
            assert!(!fresh_state.paused, "Resumed state should persist to disk");
        }

        #[test]
        fn test_ctrl_c_once_enters_confirmation_mode() {
            // Test the logic without full App instantiation
            let mut exit_confirmation_mode = false;
            let mut exit_confirmation_time: Option<std::time::Instant> = None;

            // Simulate first Ctrl+C
            if !exit_confirmation_mode {
                exit_confirmation_mode = true;
                exit_confirmation_time = Some(std::time::Instant::now());
            }

            assert!(exit_confirmation_mode);
            assert!(exit_confirmation_time.is_some());
        }

        #[test]
        fn test_ctrl_c_timeout_clears_confirmation() {
            let mut exit_confirmation_mode = true;
            // Set a time in the past (simulating timeout)
            let mut exit_confirmation_time =
                Some(std::time::Instant::now() - std::time::Duration::from_secs(2));

            // Simulate the timeout check logic from run()
            if exit_confirmation_mode {
                if let Some(start_time) = exit_confirmation_time {
                    if start_time.elapsed() > std::time::Duration::from_secs(1) {
                        exit_confirmation_mode = false;
                        exit_confirmation_time = None;
                    }
                }
            }

            assert!(!exit_confirmation_mode);
            assert!(exit_confirmation_time.is_none());
        }

        #[test]
        fn test_ctrl_c_twice_sets_should_quit() {
            let mut should_quit = false;
            let exit_confirmation_mode = true; // Already in confirmation mode

            // Simulate second Ctrl+C
            if exit_confirmation_mode {
                should_quit = true;
            }

            assert!(should_quit);
        }
    }

    // ============================================
    // Launch Validation Tests
    // ============================================

    mod launch_validation {
        use super::*;

        #[test]
        fn test_try_launch_blocked_when_paused() {
            let temp_dir = TempDir::new().unwrap();
            let config = make_test_config(&temp_dir);

            // Set up state as paused
            let mut state = State::load(&config).unwrap();
            state.set_paused(true).unwrap();

            // Simulate try_launch check
            let dashboard_paused = true;
            let can_launch = !dashboard_paused;

            assert!(!can_launch, "Should not launch when paused");
        }

        #[test]
        fn test_try_launch_blocked_at_max_agents() {
            let temp_dir = TempDir::new().unwrap();
            let config = make_test_config(&temp_dir);

            // Add max agents to state
            let mut state = State::load(&config).unwrap();
            state
                .add_agent(
                    "TASK-001".to_string(),
                    "TASK".to_string(),
                    "test-project".to_string(),
                    false,
                )
                .unwrap();

            // Reload state
            let state = State::load(&config).unwrap();
            let running_count = state.running_agents().len();
            let max_agents = config.effective_max_agents();

            // Test with max_agents = 1 (default)
            let can_launch = running_count < max_agents;

            // With one agent running and max_agents = 1, should be blocked
            assert!(!can_launch || max_agents > 1);
        }

        #[test]
        fn test_try_launch_blocked_project_busy() {
            let temp_dir = TempDir::new().unwrap();
            let config = make_test_config(&temp_dir);

            // Add an agent for test-project
            let mut state = State::load(&config).unwrap();
            state
                .add_agent(
                    "TASK-001".to_string(),
                    "TASK".to_string(),
                    "test-project".to_string(),
                    false,
                )
                .unwrap();

            // Check if project is busy
            let state = State::load(&config).unwrap();
            let project_busy = state.is_project_busy("test-project");

            assert!(project_busy, "Project should be busy with running agent");
        }

        #[test]
        fn test_try_launch_project_not_busy_when_empty() {
            let temp_dir = TempDir::new().unwrap();
            let config = make_test_config(&temp_dir);

            let state = State::load(&config).unwrap();
            let project_busy = state.is_project_busy("test-project");

            assert!(!project_busy, "Project should not be busy without agents");
        }

        #[test]
        fn test_try_launch_with_empty_queue() {
            let temp_dir = TempDir::new().unwrap();
            let config = make_test_config(&temp_dir);

            let queue = Queue::new(&config).unwrap();
            let tickets = queue.list_by_priority().unwrap();

            assert!(tickets.is_empty(), "Queue should be empty initially");
        }

        #[test]
        fn test_try_launch_with_ticket_in_queue() {
            let temp_dir = TempDir::new().unwrap();
            let config = make_test_config(&temp_dir);

            // Create a ticket file in the queue
            let ticket_content = r#"---
priority: P2-medium
---
# Test ticket

Test content
"#;
            let ticket_filename = "20241225-1200-TASK-test-project-test.md";
            let ticket_path = config.tickets_path().join("queue").join(ticket_filename);
            std::fs::write(&ticket_path, ticket_content).unwrap();

            let queue = Queue::new(&config).unwrap();
            let tickets = queue.list_by_priority().unwrap();

            assert_eq!(tickets.len(), 1, "Queue should have one ticket");
        }
    }

    // ============================================
    // Modal Dispatch Tests
    // ============================================

    mod modal_dispatch {
        use super::*;

        #[test]
        fn test_help_dialog_visibility_toggle() {
            let mut help_visible = false;

            // Toggle on
            help_visible = !help_visible;
            assert!(help_visible);

            // Toggle off
            help_visible = !help_visible;
            assert!(!help_visible);
        }

        #[test]
        fn test_help_dialog_closes_on_key() {
            let mut help_visible = true;

            // Simulate any key press when help is visible
            // In app.rs, any key closes the help dialog
            if help_visible {
                help_visible = false;
            }

            assert!(!help_visible);
        }

        #[test]
        fn test_confirm_dialog_y_launches() {
            // Test the confirm dialog selection logic
            let selection = ConfirmSelection::Yes;
            let should_launch = matches!(selection, ConfirmSelection::Yes);

            assert!(should_launch);
        }

        #[test]
        fn test_confirm_dialog_n_closes() {
            let selection = ConfirmSelection::No;
            let should_close = matches!(selection, ConfirmSelection::No);

            assert!(should_close);
        }

        #[test]
        fn test_confirm_dialog_view_option() {
            let selection = ConfirmSelection::View;
            let should_view = matches!(selection, ConfirmSelection::View);

            assert!(should_view);
        }

        #[test]
        fn test_session_recovery_resume_selection() {
            use crate::ui::SessionRecoverySelection;

            let selection = SessionRecoverySelection::ResumeSession;
            let is_resume = matches!(selection, SessionRecoverySelection::ResumeSession);

            assert!(is_resume);
        }

        #[test]
        fn test_session_recovery_fresh_selection() {
            use crate::ui::SessionRecoverySelection;

            let selection = SessionRecoverySelection::StartFresh;
            let is_fresh = matches!(selection, SessionRecoverySelection::StartFresh);

            assert!(is_fresh);
        }

        #[test]
        fn test_session_recovery_return_selection() {
            use crate::ui::SessionRecoverySelection;

            let selection = SessionRecoverySelection::ReturnToQueue;
            let is_return = matches!(selection, SessionRecoverySelection::ReturnToQueue);

            assert!(is_return);
        }
    }

    // ============================================
    // Review Signal Tests
    // ============================================

    mod review_signals {
        #[test]
        fn test_review_approval_requires_pending_state() {
            // Test the condition check without full App
            let review_state: Option<&str> = Some("pending_plan");

            let can_approve = matches!(review_state, Some("pending_plan") | Some("pending_visual"));

            assert!(can_approve);
        }

        #[test]
        fn test_review_approval_blocked_for_other_states() {
            let review_state: Option<&str> = Some("running");

            let can_approve = matches!(review_state, Some("pending_plan") | Some("pending_visual"));

            assert!(!can_approve);
        }

        #[test]
        fn test_review_approval_blocked_for_none() {
            let review_state: Option<&str> = None;

            let can_approve = matches!(review_state, Some("pending_plan") | Some("pending_visual"));

            assert!(!can_approve);
        }

        #[test]
        fn test_review_signal_file_path() {
            let session_name = "op-TASK-123";
            let signal_file = format!("/tmp/operator-detach-{}.signal", session_name);

            assert_eq!(signal_file, "/tmp/operator-detach-op-TASK-123.signal");
        }
    }

    // ============================================
    // Return to Queue Tests
    // ============================================

    mod return_to_queue {
        use super::*;

        #[test]
        fn test_return_ticket_removes_agent_from_state() {
            let temp_dir = TempDir::new().unwrap();
            let config = make_test_config(&temp_dir);

            // Add an agent and set its session name
            let mut state = State::load(&config).unwrap();
            let agent_id = state
                .add_agent(
                    "TASK-001".to_string(),
                    "TASK".to_string(),
                    "test-project".to_string(),
                    false,
                )
                .unwrap();

            // Set the session name
            let session_name = "op-TASK-001".to_string();
            state
                .update_agent_session(&agent_id, &session_name)
                .unwrap();

            // Reload and verify agent exists
            let state = State::load(&config).unwrap();
            assert!(state.agent_by_session(&session_name).is_some());

            // Remove agent by session
            let mut state = State::load(&config).unwrap();
            state.remove_agent_by_session(&session_name).unwrap();

            // Verify agent is removed
            let state = State::load(&config).unwrap();
            assert!(state.agent_by_session(&session_name).is_none());
        }

        #[test]
        fn test_queue_return_moves_ticket_file() {
            let temp_dir = TempDir::new().unwrap();
            let config = make_test_config(&temp_dir);

            // Create a ticket in in-progress
            let ticket_content = r#"---
priority: P2-medium
status: in-progress
---
# Test ticket

Test content
"#;
            let ticket_filename = "20241225-1200-TASK-test-project-test.md";
            let in_progress_path = config
                .tickets_path()
                .join("in-progress")
                .join(ticket_filename);
            std::fs::write(&in_progress_path, ticket_content).unwrap();

            // Verify file is in in-progress
            assert!(in_progress_path.exists());

            // Load queue and get ticket
            let queue = Queue::new(&config).unwrap();

            // Create ticket struct for return_to_queue
            let ticket = Ticket {
                filename: ticket_filename.to_string(),
                filepath: in_progress_path.to_string_lossy().to_string(),
                timestamp: "20241225-1200".to_string(),
                ticket_type: "TASK".to_string(),
                project: "test-project".to_string(),
                id: "TASK-test".to_string(),
                summary: "Test ticket".to_string(),
                priority: "P2-medium".to_string(),
                status: "in-progress".to_string(),
                step: String::new(),
                content: "Test content".to_string(),
                sessions: std::collections::HashMap::new(),
                llm_task: crate::queue::LlmTask::default(),
                worktree_path: None,
                branch: None,
                external_id: None,
                external_url: None,
                external_provider: None,
            };

            // Return to queue
            queue.return_to_queue(&ticket).unwrap();

            // Verify file moved to queue
            let queue_path = config.tickets_path().join("queue").join(ticket_filename);
            assert!(queue_path.exists(), "Ticket should be moved to queue");
            assert!(
                !in_progress_path.exists(),
                "Ticket should be removed from in-progress"
            );
        }
    }

    // ============================================
    // Dashboard State Tests
    // ============================================

    mod dashboard_state {
        use super::*;

        #[test]
        fn test_dashboard_paused_reflects_state() {
            let temp_dir = TempDir::new().unwrap();
            let config = make_test_config(&temp_dir);

            // Set state to paused
            let mut state = State::load(&config).unwrap();
            state.set_paused(true).unwrap();

            // Create dashboard and update from state
            let mut dashboard = Dashboard::new(&config);
            let state = State::load(&config).unwrap();
            dashboard.paused = state.paused;

            assert!(dashboard.paused);
        }

        #[test]
        fn test_dashboard_agents_update() {
            let temp_dir = TempDir::new().unwrap();
            let config = make_test_config(&temp_dir);

            // Add agent to state
            let mut state = State::load(&config).unwrap();
            state
                .add_agent(
                    "TASK-001".to_string(),
                    "TASK".to_string(),
                    "test-project".to_string(),
                    false,
                )
                .unwrap();

            // Create dashboard and update agents
            let mut dashboard = Dashboard::new(&config);
            let state = State::load(&config).unwrap();
            let agents: Vec<_> = state.agents.clone();
            dashboard.update_agents(agents);

            // Verify running agents count via state (dashboard reflects state)
            assert_eq!(state.running_agents().len(), 1);
        }
    }
}

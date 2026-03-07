use anyhow::Result;
use crossterm::event::KeyCode;

use crate::ui::setup::SetupResult;
use crate::ui::{ConfirmSelection, KanbanViewResult, SessionRecoverySelection, SyncConfirmResult};

use super::{App, AppTerminal};

impl App {
    pub(super) async fn handle_key(
        &mut self,
        key: KeyCode,
        terminal: &mut AppTerminal,
    ) -> Result<()> {
        // Setup screen takes absolute priority
        if let Some(ref mut setup) = self.setup_screen {
            match key {
                KeyCode::Char('i' | 'I') => {
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
                KeyCode::Char('c' | 'C') => {
                    self.should_quit = true;
                }
                KeyCode::Esc => {
                    if let SetupResult::Cancel = setup.go_back() {
                        self.should_quit = true;
                    } else {
                        // Moved to previous step - stay in setup
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
                    KeyCode::Char('m' | 'M') => {
                        self.confirm_dialog.cycle_provider();
                    }
                    KeyCode::Char('p' | 'P') => {
                        self.confirm_dialog.cycle_project();
                    }
                    KeyCode::Char('d' | 'D') => {
                        self.confirm_dialog.toggle_docker();
                    }
                    KeyCode::Char('a' | 'A') => {
                        self.confirm_dialog.toggle_yolo();
                    }
                    _ => {}
                }
            } else {
                // Buttons focused (default behavior)
                match key {
                    KeyCode::Char('y' | 'Y') => {
                        self.launch_confirmed().await?;
                    }
                    KeyCode::Char('v' | 'V') => {
                        self.view_ticket(terminal)?;
                    }
                    KeyCode::Char('e' | 'E') => {
                        self.edit_ticket(terminal)?;
                    }
                    KeyCode::Char('n' | 'N') | KeyCode::Esc => {
                        self.confirm_dialog.hide();
                    }
                    // Up moves focus to options section (if options available)
                    KeyCode::Up => {
                        self.confirm_dialog.focus_options();
                    }
                    // Launch options shortcuts: M = provider, P = project, D = docker, A = auto
                    KeyCode::Char('m' | 'M') => {
                        self.confirm_dialog.cycle_provider();
                    }
                    KeyCode::Char('p' | 'P') => {
                        self.confirm_dialog.cycle_project();
                    }
                    KeyCode::Char('d' | 'D') => {
                        self.confirm_dialog.toggle_docker();
                    }
                    KeyCode::Char('a' | 'A') => {
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
                KeyCode::Char('r' | 'R') => {
                    if self.session_recovery_dialog.has_session_id() {
                        self.handle_session_recovery(SessionRecoverySelection::ResumeSession)
                            .await?;
                    }
                }
                KeyCode::Char('s' | 'S') => {
                    self.handle_session_recovery(SessionRecoverySelection::StartFresh)
                        .await?;
                }
                KeyCode::Char('q' | 'Q') => {
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
                            "Sync requested for {provider}/{project_key} (sync not yet implemented)"
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
            KeyCode::Char('L' | 'l') => {
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
            KeyCode::Char('P' | 'p') => {
                self.pause_queue()?;
            }
            KeyCode::Char('R' | 'r') => {
                self.resume_queue()?;
            }
            KeyCode::Char('Q') => {
                self.dashboard.focused = crate::ui::dashboard::FocusedPanel::Queue;
            }
            KeyCode::Char('A' | 'a') => {
                self.dashboard.focused = crate::ui::dashboard::FocusedPanel::Agents;
            }
            KeyCode::Char('C') => {
                self.create_dialog.show();
            }
            KeyCode::Char('J') => {
                self.projects_dialog.show();
            }
            KeyCode::Char('v' | 'V') => {
                self.show_session_preview()?;
            }
            KeyCode::Char('y' | 'Y') => {
                // Approve review (only for agents with review_state)
                self.handle_review_approval()?;
            }
            KeyCode::Char('x' | 'X') => {
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
            KeyCode::Char('W' | 'w') => {
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
            KeyCode::Char('T' | 't') => {
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

    /// Handle Ctrl+C for graceful two-stage exit
    pub(super) async fn handle_ctrl_c(&mut self) {
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
}

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::ui::setup::SetupResult;
use crate::ui::status_panel::ActionButton;
use crate::ui::{ConfirmSelection, KanbanViewResult, SessionRecoverySelection, SyncConfirmResult};

use super::git_onboarding;
use super::{App, AppTerminal};

impl App {
    pub(super) async fn handle_key(
        &mut self,
        key: KeyEvent,
        terminal: &mut AppTerminal,
    ) -> Result<()> {
        let code = key.code;
        let mods = key.modifiers;

        // Setup screen takes absolute priority
        if let Some(ref mut setup) = self.setup_screen {
            match code {
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
            match code {
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
            if let Some(result) = self.create_dialog.handle_key(code) {
                self.create_ticket(result, terminal)?;
            }
            return Ok(());
        }

        // Projects dialog handling
        if self.projects_dialog.visible {
            if let Some(result) = self.projects_dialog.handle_key(code) {
                self.execute_project_action(result)?;
            }
            return Ok(());
        }

        // Confirm dialog handling
        if self.confirm_dialog.visible {
            // Check if options are focused for different key behavior
            if self.confirm_dialog.is_options_focused() {
                match code {
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
                match code {
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
            match code {
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
            if let Some(result) = self.collection_dialog.handle_key(code) {
                self.handle_collection_switch(result)?;
            }
            return Ok(());
        }

        // Kanban view handling
        if self.kanban_view.visible {
            if let Some(result) = self.kanban_view.handle_key(code) {
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

        // Git token dialog handling
        if self.git_token_dialog.visible {
            match code {
                KeyCode::Esc => {
                    self.git_token_dialog.hide();
                }
                KeyCode::Enter => {
                    let token = self.git_token_dialog.token().to_string();
                    if token.is_empty() {
                        self.git_token_dialog.set_error("Token cannot be empty");
                    } else {
                        let provider = self.git_token_dialog.provider.clone();
                        let provider_display = self.git_token_dialog.provider_display.clone();
                        match git_onboarding::validate_token(&provider, &token) {
                            Ok(username) => {
                                match git_onboarding::complete_git_onboarding(
                                    &mut self.config,
                                    &provider,
                                    &token,
                                ) {
                                    Ok(()) => {
                                        self.git_token_dialog.hide();
                                        self.dashboard.update_config(&self.config);
                                        self.refresh_data()?;
                                        self.dashboard.set_status(&format!(
                                            "{provider_display} connected as {username}"
                                        ));
                                    }
                                    Err(e) => {
                                        self.git_token_dialog
                                            .set_error(&format!("Failed to save config: {e}"));
                                    }
                                }
                            }
                            Err(e) => {
                                self.git_token_dialog
                                    .set_error(&format!("Token validation failed: {e}"));
                            }
                        }
                    }
                }
                KeyCode::Char(c) => {
                    self.git_token_dialog.handle_char(c);
                }
                KeyCode::Backspace => {
                    self.git_token_dialog.handle_backspace();
                }
                KeyCode::Delete => {
                    self.git_token_dialog.handle_delete();
                }
                KeyCode::Left => {
                    self.git_token_dialog.cursor_left();
                }
                KeyCode::Right => {
                    self.git_token_dialog.cursor_right();
                }
                KeyCode::Home => {
                    self.git_token_dialog.cursor_home();
                }
                KeyCode::End => {
                    self.git_token_dialog.cursor_end();
                }
                _ => {}
            }
            return Ok(());
        }

        // Sync confirm dialog handling
        if self.sync_confirm_dialog.visible {
            if let Some(result) = self.sync_confirm_dialog.handle_key(code) {
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
        match code {
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
            KeyCode::Char('L') => {
                self.try_launch()?;
            }
            KeyCode::Char('h') | KeyCode::Left => {
                self.dashboard.focus_prev();
            }
            KeyCode::Char('l') | KeyCode::Right => {
                self.dashboard.focus_next();
            }
            KeyCode::Enter => {
                // Enter key behavior depends on focused panel and modifiers
                match self.dashboard.focused {
                    crate::ui::dashboard::FocusedPanel::Status => {
                        let button = if mods.contains(KeyModifiers::SHIFT) {
                            ActionButton::X
                        } else if mods.contains(KeyModifiers::CONTROL) {
                            ActionButton::Y
                        } else {
                            ActionButton::A
                        };
                        let action = self.dashboard.status_action(button);
                        self.execute_status_action(action, terminal)?;
                    }
                    crate::ui::dashboard::FocusedPanel::Queue => {
                        if mods.contains(KeyModifiers::SHIFT) {
                            self.auto_launch().await?;
                        } else {
                            self.try_launch()?;
                        }
                    }
                    crate::ui::dashboard::FocusedPanel::InProgress => {
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
                self.dashboard.focused = crate::ui::dashboard::FocusedPanel::InProgress;
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
                self.toggle_web_servers(terminal)?;
            }
            KeyCode::Char('T' | 't') => {
                // Open collection switch dialog
                self.show_collection_dialog();
            }
            KeyCode::Char('K') => {
                // Open kanban providers view
                self.show_kanban_view();
            }
            KeyCode::Char('F') => {
                // Focus agent's cmux window (cmux power-user action)
                self.focus_agent_window()?;
            }
            KeyCode::Esc | KeyCode::Backspace
                if self.dashboard.focused == crate::ui::dashboard::FocusedPanel::Status =>
            {
                // B-action: go back / collapse section in status panel
                let action = self.dashboard.status_action(ActionButton::B);
                self.execute_status_action(action, terminal)?;
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

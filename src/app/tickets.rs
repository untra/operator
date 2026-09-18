use anyhow::Result;

use crate::agents::{AgentTicketCreator, AssessTicketCreator};
use crate::auth::store::AuthStore;
use crate::queue::TicketCreator;
use crate::state::State;
use crate::ui::create_dialog::CreateDialogResult;
use crate::ui::projects_dialog::{ProjectAction, ProjectsDialogResult};
use crate::ui::with_suspended_tui;

use super::{App, AppTerminal};

impl App {
    /// Build setup options from the wizard's collected choices.
    fn setup_options(&self) -> crate::setup::SetupOptions {
        let Some(screen) = self.setup_screen.as_ref() else {
            return crate::setup::SetupOptions::default();
        };

        let hosted: Vec<crate::setup::FetchedCollection> = screen
            .selected_hosted_collections()
            .into_iter()
            .map(|r| (r.manifest.clone(), r.files.clone(), r.icon_svg.clone()))
            .collect();
        let active_collection = match hosted.as_slice() {
            [single] => Some(single.0.id.clone()),
            _ => None,
        };

        crate::setup::SetupOptions {
            preset: screen.preset(),
            task_fields: screen.configured_task_fields(),
            use_worktrees: screen.use_worktrees,
            wrapper: Some(screen.selected_wrapper),
            acceptance_criteria: Some(screen.acceptance_criteria_text.clone()),
            custom_collection: screen.collection(),
            active_collection,
            hosted_collections: hosted,
            model_servers: screen.declared_model_servers(),
            execution_target: Some(screen.selected_execution_target()),
            ..Default::default()
        }
    }

    /// Initialize the tickets directory with default templates and save config
    pub(super) fn initialize_tickets(&mut self) -> Result<()> {
        let options = self.setup_options();
        if let Some(screen) = self.setup_screen.as_ref() {
            if self.config.profile_registry.is_some() {
                crate::profiles::rename_registered(&mut self.config, &screen.configuration_name)?;
            } else {
                crate::profiles::validate_name(&screen.configuration_name)?;
                self.config
                    .profile
                    .name
                    .clone_from(&screen.configuration_name);
            }
        }
        let result = crate::setup::initialize_workspace(&mut self.config, &options)?;
        let discovered_full = result.discovered;
        let discovered_projects = self.config.projects.clone();

        // Create the admin account before the config is written.
        if let Some(password) = self
            .setup_screen
            .as_ref()
            .and_then(|s| s.admin_password.as_deref())
        {
            let store = AuthStore::open(&self.config.auth_state_path())?;
            persist_admin_password(&store, Some(password))?;
        }

        self.config.save()?;

        // Reload the issue type registry so the chosen collection is active
        // without requiring a restart (mirrors App::new's load path).
        let mut registry = crate::startup::templates::load_registry(&self.config.tickets_path());
        if let Some(ref active) = self.config.templates.active_collection {
            if let Err(e) = registry.activate_collection(active) {
                tracing::warn!("Failed to activate collection '{}': {}", active, e);
            }
        }
        self.issue_type_registry = registry;

        // Update the create dialog with discovered projects
        self.create_dialog.set_projects(discovered_projects.clone());

        // Create startup tickets based on user selections
        let startup_tickets = self
            .setup_screen
            .as_ref()
            .map(super::super::ui::setup::SetupScreen::selected_startup_tickets)
            .unwrap_or_default();

        if !startup_tickets.is_empty() {
            let projects_path = self.config.projects_path();
            for project in &discovered_projects {
                let project_path = projects_path.join(project);

                // ASSESS or PROJECT_INIT creates assess tickets
                if startup_tickets.contains(&"assess".to_string())
                    || startup_tickets.contains(&"project_init".to_string())
                {
                    // Check if project has git remote before creating ASSESS ticket
                    let project_info = discovered_full.iter().find(|p| p.name == *project);
                    let has_git_remote = project_info
                        .is_some_and(super::super::projects::DiscoveredProject::has_git_remote);

                    if has_git_remote {
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
                    } else {
                        tracing::info!(
                            project = %project,
                            "Skipping ASSESS ticket - no git remote configured"
                        );
                    }
                }

                // AGENT_SETUP or PROJECT_INIT creates agent tickets
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
                                    "Created AGENT_SETUP startup tickets"
                                );
                            }
                        }
                        Err(e) => {
                            tracing::warn!(project = %project, error = %e, "Failed to create AGENT_SETUP tickets");
                        }
                    }
                }
            }
        }

        crate::startup::mark_workspace_initialized(&self.config)?;

        Ok(())
    }

    /// Create a new ticket from the dialog result
    pub(super) fn create_ticket(
        &mut self,
        dialog_result: CreateDialogResult,
        terminal: &mut AppTerminal,
    ) -> Result<()> {
        let config = self.config.clone();

        let editor_cmd = self.dashboard.editor_config.file_editor().to_string();
        let result = with_suspended_tui(terminal, || {
            let creator = TicketCreator::new(&config);
            creator.create_ticket_with_values(
                dialog_result.template_type,
                &dialog_result.values,
                &editor_cmd,
            )
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
    pub(super) fn execute_project_action(&mut self, result: ProjectsDialogResult) -> Result<()> {
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
                    .is_some_and(super::super::projects::DiscoveredProject::has_git_remote);

                if !has_git_remote {
                    self.projects_dialog.set_creation_result(Err(
                        "Cannot create ASSESS ticket: project has no git remote configured"
                            .to_string(),
                    ));
                    return Ok(());
                }

                // Create ASSESS ticket for catalog assessment
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

    pub(super) fn pause_queue(&mut self) -> Result<()> {
        let mut state = State::load(&self.config)?;
        state.set_paused(true)?;
        self.dashboard.paused = true;
        Ok(())
    }

    pub(super) fn resume_queue(&mut self) -> Result<()> {
        let mut state = State::load(&self.config)?;
        state.set_paused(false)?;
        self.dashboard.paused = false;
        Ok(())
    }

    /// View ticket file in $VISUAL or with `open` command
    pub(super) fn view_ticket(&mut self, terminal: &mut AppTerminal) -> Result<()> {
        let Some(filepath) = self.confirm_dialog.ticket_filepath() else {
            return Ok(());
        };

        let visual = self.dashboard.editor_config.visual.clone();
        with_suspended_tui(terminal, || {
            let result = if visual.is_empty() {
                std::process::Command::new("open").arg(&filepath).status()
            } else {
                let (prog, args) = crate::editors::EditorConfig::split_command(&visual);
                std::process::Command::new(prog)
                    .args(&args)
                    .arg(&filepath)
                    .status()
            };

            if let Err(e) = result {
                tracing::warn!("Failed to open file: {}", e);
            }

            Ok(())
        })
    }

    /// Edit ticket file in $EDITOR
    pub(super) fn edit_ticket(&mut self, terminal: &mut AppTerminal) -> Result<()> {
        let Some(filepath) = self.confirm_dialog.ticket_filepath() else {
            return Ok(());
        };

        let editor = self.dashboard.editor_config.editor.clone();
        if editor.is_empty() {
            return Ok(());
        }

        with_suspended_tui(terminal, || {
            let (prog, args) = crate::editors::EditorConfig::split_command(&editor);
            let result = std::process::Command::new(prog)
                .args(&args)
                .arg(&filepath)
                .status();

            if let Err(e) = result {
                tracing::warn!("Failed to open editor: {}", e);
            }

            Ok(())
        })
    }
}

/// Create the admin account from the wizard's optional password.
///
/// `None` means the operator skipped the step, which is not a failure.
///
/// A `false` return from `create_admin` may mean an account already existed.
fn persist_admin_password(store: &AuthStore, password: Option<&str>) -> Result<()> {
    let Some(password) = password else {
        return Ok(());
    };

    if store.create_admin(password, false)? {
        store.audit("admin created", Some("via setup wizard"), true)?;
    } else {
        tracing::info!("admin account already exists; setup wizard password not applied");
    }
    Ok(())
}

#[cfg(test)]
mod admin_password_tests {
    use super::*;

    #[test]
    fn test_skipped_password_creates_no_account() {
        use crate::rest::dto::auth::BootstrapState;

        let store = AuthStore::in_memory().unwrap();
        persist_admin_password(&store, None).unwrap();

        assert_eq!(
            store.bootstrap_state().unwrap(),
            BootstrapState::Uninitialized,
            "skipping the step must leave the deployment unbootstrapped"
        );
    }

    #[test]
    fn test_password_creates_a_usable_admin_account() {
        use crate::rest::dto::auth::BootstrapState;

        let store = AuthStore::in_memory().unwrap();
        persist_admin_password(&store, Some("a properly long password")).unwrap();

        assert_eq!(store.bootstrap_state().unwrap(), BootstrapState::Complete);
        assert!(store
            .verify_admin_password("a properly long password")
            .unwrap());
    }

    #[test]
    fn test_existing_admin_is_left_alone_rather_than_failing() {
        // A server that bootstrapped between wizard start and finish must not
        // make initialization fail, and must keep its own password.
        let store = AuthStore::in_memory().unwrap();
        store.create_admin("the original password", false).unwrap();

        persist_admin_password(&store, Some("the wizard password")).unwrap();

        assert!(store
            .verify_admin_password("the original password")
            .unwrap());
        assert!(!store.verify_admin_password("the wizard password").unwrap());
    }

    #[test]
    fn test_invalid_password_surfaces_as_an_error() {
        // The wizard validates first, so this only happens if that check is bypassed
        let store = AuthStore::in_memory().unwrap();
        assert!(persist_admin_password(&store, Some("short")).is_err());
    }

    #[test]
    fn test_creation_is_audited() {
        let store = AuthStore::in_memory().unwrap();
        persist_admin_password(&store, Some("a properly long password")).unwrap();

        let recent = store.recent_audit(10).unwrap();
        let entry = recent
            .iter()
            .find(|(_, event, _, _)| event == "admin created")
            .expect("account creation should be audited");
        assert_eq!(entry.2.as_deref(), Some("via setup wizard"));
        assert!(entry.3, "recorded as a success");
    }
}

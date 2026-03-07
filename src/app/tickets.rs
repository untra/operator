use anyhow::Result;
use std::fs;

use crate::agents::{generate_status_script, generate_tmux_conf};
use crate::agents::{AgentTicketCreator, AssessTicketCreator};
use crate::backstage::scaffold::{BackstageScaffold, ScaffoldOptions};
use crate::queue::TicketCreator;
use crate::setup::filter_schema_fields;
use crate::state::State;
use crate::templates::TemplateType;
use crate::ui::create_dialog::CreateDialogResult;
use crate::ui::projects_dialog::{ProjectAction, ProjectsDialogResult};
use crate::ui::with_suspended_tui;

use super::{App, AppTerminal};

impl App {
    /// Initialize the tickets directory with default templates and save config
    pub(super) fn initialize_tickets(&mut self) -> Result<()> {
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
            .map(super::super::ui::setup::SetupScreen::selected_startup_tickets)
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
    pub(super) fn generate_tmux_config(&mut self) -> Result<()> {
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
    pub(super) fn create_ticket(
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
    pub(super) fn edit_ticket(&mut self, terminal: &mut AppTerminal) -> Result<()> {
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
}

//! Agent launcher for starting Claude agents in tmux sessions
//!
//! This module handles launching agents with appropriate permissions,
//! prompt generation, and session management.

#![allow(dead_code)]

mod cmux_session;
pub mod interpolation;
mod llm_command;
mod options;
mod prompt;
mod step_config;
mod tmux_session;
pub mod worktree_setup;
mod zellij_session;

#[cfg(test)]
mod tests;

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result};

use crate::agents::cmux::{CmuxClient, SystemCmuxClient};
use crate::agents::tmux::{sanitize_session_name, SystemTmuxClient, TmuxClient, TmuxError};
use crate::agents::zellij::{SystemZellijClient, ZellijClient};
use crate::api::kanban_sync::KanbanBidirectionalSync;
use crate::config::{Config, SessionWrapperType};
use crate::notifications;
use crate::queue::{Queue, Ticket};
use crate::state::State;

use cmux_session::{launch_in_cmux_with_options, launch_in_cmux_with_relaunch_options};
pub use options::{LaunchOptions, RelaunchOptions};
use prompt::generate_prompt;
use tmux_session::{launch_in_tmux_with_options, launch_in_tmux_with_relaunch_options};
use worktree_setup::setup_worktree_for_ticket;
use zellij_session::{launch_in_zellij_with_options, launch_in_zellij_with_relaunch_options};

use self::interpolation::PromptInterpolator;
use self::llm_command::{
    apply_yolo_flags, build_docker_command, build_llm_command_with_permissions_for_tool,
    get_default_model,
};
use self::prompt::{
    generate_session_uuid, get_agent_prompt, get_template_prompt, write_prompt_file,
};

/// Session name prefix for operator-managed tmux sessions
pub const SESSION_PREFIX: &str = "op-";

/// Apply delegator prompt prefix/suffix wrapping to a generated prompt
fn apply_prompt_wrapping(prompt: String, options: &LaunchOptions) -> String {
    match (&options.prompt_prefix, &options.prompt_suffix) {
        (Some(pre), Some(suf)) => format!("{pre}\n\n{prompt}\n\n{suf}"),
        (Some(pre), None) => format!("{pre}\n\n{prompt}"),
        (None, Some(suf)) => format!("{prompt}\n\n{suf}"),
        (None, None) => prompt,
    }
}

/// Result of preparing a launch without executing it
///
/// Contains all the information needed to launch an agent in any wrapper
/// (tmux, VS Code terminal, or standalone terminal).
#[derive(Debug, Clone)]
pub struct PreparedLaunch {
    /// Agent ID assigned to this launch
    pub agent_id: String,
    /// Ticket ID being launched
    pub ticket_id: String,
    /// Working directory (worktree path or project path)
    pub working_directory: PathBuf,
    /// Command to execute in the terminal
    pub command: String,
    /// Terminal/session name (e.g., "op-FEAT-123")
    pub terminal_name: String,
    /// Session UUID for the LLM tool
    pub session_id: String,
    /// Whether a worktree was created
    pub worktree_created: bool,
    /// Branch name (if worktree was created)
    pub branch: Option<String>,
    /// Which session wrapper was used: "tmux", "vscode", "cmux", or "zellij"
    pub session_wrapper: Option<String>,
    /// Session window reference ID (e.g. cmux window, tmux session)
    pub session_window_ref: Option<String>,
    /// Session context reference (e.g. cmux workspace, zellij session)
    pub session_context_ref: Option<String>,
}

/// Minimum required tmux version
pub const MIN_TMUX_VERSION: (u32, u32) = (2, 1);

pub struct Launcher {
    config: Config,
    tmux: Arc<dyn TmuxClient>,
    cmux: Option<Arc<dyn CmuxClient>>,
    zellij: Option<Arc<dyn ZellijClient>>,
}

impl Launcher {
    /// Create a new Launcher with the system tmux client
    ///
    /// Uses custom tmux config if it has been generated and exists.
    /// Also creates a cmux client if the wrapper type is Cmux.
    pub fn new(config: &Config) -> Result<Self> {
        // Use custom tmux config if it exists
        let tmux: Arc<dyn TmuxClient> = if config.tmux.config_generated {
            let config_path = config.tmux_config_path();
            if config_path.exists() {
                Arc::new(SystemTmuxClient::with_config(config_path))
            } else {
                Arc::new(SystemTmuxClient::new())
            }
        } else {
            Arc::new(SystemTmuxClient::new())
        };

        // Create cmux client if wrapper type is Cmux
        let cmux: Option<Arc<dyn CmuxClient>> =
            if config.sessions.wrapper == SessionWrapperType::Cmux {
                Some(Arc::new(SystemCmuxClient::from_config(
                    &config.sessions.cmux,
                )))
            } else {
                None
            };

        // Create zellij client if wrapper type is Zellij
        let zellij: Option<Arc<dyn ZellijClient>> =
            if config.sessions.wrapper == SessionWrapperType::Zellij {
                Some(Arc::new(SystemZellijClient::new()))
            } else {
                None
            };

        Ok(Self {
            config: config.clone(),
            tmux,
            cmux,
            zellij,
        })
    }

    /// Create a new Launcher with a custom tmux client (for testing)
    pub fn with_tmux_client(config: &Config, tmux: Arc<dyn TmuxClient>) -> Result<Self> {
        Ok(Self {
            config: config.clone(),
            tmux,
            cmux: None,
            zellij: None,
        })
    }

    /// Create a new Launcher with a custom cmux client (for testing)
    pub fn with_cmux_client(config: &Config, cmux: Arc<dyn CmuxClient>) -> Result<Self> {
        Ok(Self {
            config: config.clone(),
            tmux: Arc::new(SystemTmuxClient::new()),
            cmux: Some(cmux),
            zellij: None,
        })
    }

    /// Create a new Launcher with a custom zellij client (for testing)
    pub fn with_zellij_client(config: &Config, zellij: Arc<dyn ZellijClient>) -> Result<Self> {
        Ok(Self {
            config: config.clone(),
            tmux: Arc::new(SystemTmuxClient::new()),
            cmux: None,
            zellij: Some(zellij),
        })
    }

    /// Collect all LLM tools needed across a ticket's steps (for multi-tool skill deployment).
    ///
    /// When steps specify different agents via the `agent` field, skills need to be
    /// deployed for all tools so they're available when agent switching occurs.
    fn collect_tools_for_ticket(&self, ticket: &Ticket, primary_tool: &str) -> Vec<String> {
        let mut tools = vec![primary_tool.to_string()];

        if let Some(template) = ticket.template_schema() {
            for step in &template.steps {
                if let Some(agent_name) = crate::templates::step_type::effective_agent(step) {
                    if let Some(delegator) =
                        self.config.delegators.iter().find(|d| d.name == agent_name)
                    {
                        if !tools.contains(&delegator.llm_tool) {
                            tools.push(delegator.llm_tool.clone());
                        }
                    }
                }
            }
        }

        tools
    }

    /// Check if tmux is available and meets minimum version requirements
    pub fn check_tmux(&self) -> Result<(), TmuxError> {
        let version = self.tmux.check_available()?;

        if !version.meets_minimum(MIN_TMUX_VERSION.0, MIN_TMUX_VERSION.1) {
            return Err(TmuxError::VersionTooOld(
                version.raw,
                format!("{}.{}", MIN_TMUX_VERSION.0, MIN_TMUX_VERSION.1),
            ));
        }

        tracing::info!(
            version = %version.raw,
            "tmux is available"
        );

        Ok(())
    }

    /// Launch a Claude agent in a tmux session for the given ticket
    pub async fn launch(&self, ticket: &Ticket) -> Result<String> {
        self.launch_with_options(ticket, LaunchOptions::default())
            .await
    }

    /// Launch an agent with specific launch options
    pub async fn launch_with_options(
        &self,
        ticket: &Ticket,
        options: LaunchOptions,
    ) -> Result<String> {
        // Clone ticket so we can update worktree info
        let mut ticket = ticket.clone();

        // Move ticket to in-progress
        let queue = Queue::new(&self.config)?;
        queue.claim_ticket(&ticket)?;

        // Best-effort: notify upstream kanban that ticket is now in-progress.
        let ks = KanbanBidirectionalSync::new(Arc::new(self.config.clone()));
        let ticket_clone = ticket.clone();
        tokio::spawn(async move {
            ks.on_ticket_claimed(&ticket_clone).await;
        });

        // Get project path (use override if provided)
        let project_path = if let Some(ref override_project) = options.project_override {
            PathBuf::from(self.get_project_path_for(override_project)?)
        } else {
            PathBuf::from(self.get_project_path(&ticket)?)
        };

        // Setup worktree for per-ticket isolation (if project is a git repo)
        let working_dir = setup_worktree_for_ticket(
            &self.config,
            &mut ticket,
            &project_path,
            options.use_worktrees_override,
        )
        .await
        .context("Failed to setup worktree for ticket")?;

        // Deploy operator skills for all tools this ticket may use across steps
        let primary_tool = options
            .provider
            .as_ref()
            .map_or("claude", |p| p.tool.as_str());
        let tools = self.collect_tools_for_ticket(&ticket, primary_tool);
        let tool_refs: Vec<&str> = tools.iter().map(std::string::String::as_str).collect();
        if let Err(e) = crate::llm::deploy_skills(&working_dir, &project_path, &tool_refs) {
            tracing::warn!(error = %e, "Failed to deploy skills (non-fatal)");
        }

        let working_dir_str = working_dir.to_string_lossy().to_string();

        // Dispatch multi-agent step types before single-agent launch.
        // Worktree/skills are already set up above and shared across sub-agents.
        if let Some(ref step) = ticket.current_step_schema() {
            match step.step_type {
                crate::templates::schema::StepTypeTag::MultiModel => {
                    return self
                        .launch_multi_model(&ticket, step, &working_dir_str, &options)
                        .await;
                }
                crate::templates::schema::StepTypeTag::MultiPrompt => {
                    return self
                        .launch_multi_prompt(&ticket, step, &working_dir_str, &options)
                        .await;
                }
                crate::templates::schema::StepTypeTag::Matrixed => {
                    return self
                        .launch_matrixed(&ticket, step, &working_dir_str, &options)
                        .await;
                }
                _ => {}
            }
        }

        // Single-agent path: generate prompt and launch one sub-agent.
        let initial_prompt = generate_prompt(&self.config, &ticket);
        let initial_prompt = apply_prompt_wrapping(initial_prompt, &options);

        let (agent_id, _session) = self
            .launch_one_sub_agent(&ticket, &working_dir_str, &initial_prompt, &options)
            .await?;
        Ok(agent_id)
    }

    /// Dispatch a single sub-agent launch: wrapper dispatch, state registration,
    /// worktree-path persistence, step recording, and start-up notification.
    /// Returns `(agent_id, session_name)`.
    async fn launch_one_sub_agent(
        &self,
        ticket: &Ticket,
        working_dir_str: &str,
        initial_prompt: &str,
        options: &LaunchOptions,
    ) -> Result<(String, String)> {
        // Dispatch based on session wrapper type
        let (session_name, wrapper_name, cmux_refs) =
            if self.config.sessions.wrapper == SessionWrapperType::Cmux {
                let cmux = self.cmux.as_ref().ok_or_else(|| {
                    anyhow::anyhow!("cmux client not initialized but wrapper type is cmux")
                })?;
                let result = launch_in_cmux_with_options(
                    &self.config,
                    cmux,
                    ticket,
                    working_dir_str,
                    initial_prompt,
                    options,
                )?;
                (
                    result.session_name,
                    "cmux",
                    Some((result.window_ref, result.workspace_ref)),
                )
            } else if self.config.sessions.wrapper == SessionWrapperType::Zellij {
                let zellij = self.zellij.as_ref().ok_or_else(|| {
                    anyhow::anyhow!("zellij client not initialized but wrapper type is zellij")
                })?;
                let result = launch_in_zellij_with_options(
                    &self.config,
                    zellij,
                    ticket,
                    working_dir_str,
                    initial_prompt,
                    options,
                )?;
                (result.session_name, "zellij", None)
            } else {
                // Tmux (default) and Vscode (uses prepare_launch path)
                let name = launch_in_tmux_with_options(
                    &self.config,
                    &self.tmux,
                    ticket,
                    working_dir_str,
                    initial_prompt,
                    options,
                )?;
                (name, "tmux", None)
            };

        // Determine tool name from options or default
        let llm_tool = options
            .provider
            .as_ref()
            .map(|p| p.tool.clone())
            .or_else(|| {
                self.config
                    .llm_tools
                    .detected
                    .first()
                    .map(|t| t.name.clone())
            });

        // Update state with launch options
        let mut state = State::load(&self.config)?;
        let agent_id = state.add_agent_with_options(
            ticket.id.clone(),
            ticket.ticket_type.clone(),
            ticket.project.clone(),
            ticket.is_paired(),
            llm_tool,
            Some(options.launch_mode_string()),
        )?;

        // Store session name in state for later recovery
        state.update_agent_session(&agent_id, &session_name)?;

        // Store session wrapper type
        state.update_agent_session_wrapper(&agent_id, wrapper_name)?;

        // Store session refs if applicable (e.g. cmux window + workspace)
        if let Some((window_ref, workspace_ref)) = cmux_refs {
            state.update_agent_session_refs(
                &agent_id,
                Some(&window_ref),
                Some(&workspace_ref),
                None,
            )?;
        }

        // Store worktree path in state (if one was created)
        if let Some(ref worktree_path) = ticket.worktree_path {
            state.update_agent_worktree_path(&agent_id, worktree_path)?;
        }

        // Set the current step in state
        if !ticket.step.is_empty() {
            state.update_agent_step(&agent_id, &ticket.step)?;
        }

        // Send notification
        if self.config.notifications.enabled && self.config.notifications.on_agent_start {
            let mode_suffix = match (options.docker_mode, options.yolo_mode) {
                (true, true) => " [docker-yolo]",
                (true, false) => " [docker]",
                (false, true) => " [yolo]",
                (false, false) => "",
            };
            let worktree_suffix = if ticket.worktree_path.is_some() {
                " [worktree]"
            } else {
                ""
            };
            // TODO: Migrate to NotificationService when Launcher has access to it
            #[allow(deprecated)]
            notifications::send(
                "Agent Started",
                &format!(
                    "{} - {} ({}: {}){}{}",
                    ticket.project,
                    ticket.ticket_type,
                    wrapper_name,
                    session_name,
                    mode_suffix,
                    worktree_suffix
                ),
                &ticket.summary,
                self.config.notifications.sound,
            )?;
        }

        Ok((agent_id, session_name))
    }

    /// Build per-sub-agent launch options from a base and a delegator name.
    ///
    /// Copies the base options, overrides `provider`/`delegator_name` from the
    /// delegator, and applies the delegator's `launch_config`. Session suffix
    /// is set to the `variant_key` so parallel sub-agents don't collide.
    fn sub_agent_options(
        &self,
        base: &LaunchOptions,
        delegator_name: &str,
        variant_key: &str,
    ) -> Result<LaunchOptions> {
        let delegator = self
            .config
            .delegators
            .iter()
            .find(|d| d.name == delegator_name)
            .ok_or_else(|| {
                anyhow::anyhow!("Delegator '{delegator_name}' not found in config.delegators")
            })?;

        let mut opts = base.clone();
        opts.provider = Some(crate::agents::delegator_resolution::delegator_to_provider(
            delegator,
        ));
        opts.delegator_name = Some(delegator.name.clone());
        crate::agents::delegator_resolution::apply_delegator_launch_config(
            &mut opts,
            &delegator.launch_config,
        );
        opts.session_suffix = Some(variant_key.to_string());
        Ok(opts)
    }

    /// Render a prompt template with the ticket's handlebars context.
    fn render_variant_prompt(
        &self,
        template: &str,
        ticket: &Ticket,
        working_dir_str: &str,
    ) -> Result<String> {
        let interpolator = self::interpolation::PromptInterpolator::new();
        let ctx = interpolator.build_context(&self.config, ticket, working_dir_str)?;
        interpolator.render(template, &ctx)
    }

    /// Compute the budget for new sub-agent launches based on `max_parallel`.
    fn available_slots(&self) -> Result<usize> {
        let state = State::load(&self.config)?;
        let running = state.running_agents().len();
        let cap = self.config.effective_max_agents();
        Ok(cap.saturating_sub(running))
    }

    /// Fan out a `multi_model` step: N delegators, same prompt for all.
    ///
    /// Launches up to `available_slots()` sub-agents immediately; any that
    /// don't fit are stored in `pending_launches` and drip-launched by the
    /// sync loop as slots free up. Returns the `agent_id` of the first
    /// sub-agent launched (or the `group_id` if nothing launched).
    async fn launch_multi_model(
        &self,
        ticket: &Ticket,
        step: &crate::templates::schema::StepSchema,
        working_dir_str: &str,
        base_options: &LaunchOptions,
    ) -> Result<String> {
        let cfg = step.multi_model_config.as_ref().ok_or_else(|| {
            anyhow::anyhow!(
                "multi_model step '{}' is missing multi_model_config",
                step.name
            )
        })?;

        let base_prompt = generate_prompt(&self.config, ticket);
        let base_prompt = apply_prompt_wrapping(base_prompt, base_options);

        let pending: Vec<crate::state::PendingSubAgent> = cfg
            .delegators
            .iter()
            .map(|d| crate::state::PendingSubAgent {
                delegator_name: d.clone(),
                prompt: base_prompt.clone(),
                variant_key: d.clone(),
            })
            .collect();

        self.launch_group(
            ticket,
            step,
            working_dir_str,
            base_options,
            "multi_model",
            pending,
        )
        .await
    }

    /// Fan out a `multi_prompt` step: N prompt variations, one delegator.
    async fn launch_multi_prompt(
        &self,
        ticket: &Ticket,
        step: &crate::templates::schema::StepSchema,
        working_dir_str: &str,
        base_options: &LaunchOptions,
    ) -> Result<String> {
        let cfg = step.multi_prompt_config.as_ref().ok_or_else(|| {
            anyhow::anyhow!(
                "multi_prompt step '{}' is missing multi_prompt_config",
                step.name
            )
        })?;

        // Resolve the delegator for all variations
        let delegator_name = cfg
            .agent
            .clone()
            .or_else(|| {
                crate::templates::step_type::effective_agent(step)
                    .map(std::string::ToString::to_string)
            })
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "multi_prompt step '{}' has no agent configured \
                     (set multi_prompt_config.agent or step.agent)",
                    step.name
                )
            })?;

        let mut pending = Vec::with_capacity(cfg.prompt_variations.len());
        for (i, tpl) in cfg.prompt_variations.iter().enumerate() {
            let rendered = self.render_variant_prompt(tpl, ticket, working_dir_str)?;
            let full_prompt = apply_prompt_wrapping(rendered, base_options);
            pending.push(crate::state::PendingSubAgent {
                delegator_name: delegator_name.clone(),
                prompt: full_prompt,
                variant_key: i.to_string(),
            });
        }

        self.launch_group(
            ticket,
            step,
            working_dir_str,
            base_options,
            "multi_prompt",
            pending,
        )
        .await
    }

    /// Fan out a `matrixed` step: N delegators x M prompt variations.
    async fn launch_matrixed(
        &self,
        ticket: &Ticket,
        step: &crate::templates::schema::StepSchema,
        working_dir_str: &str,
        base_options: &LaunchOptions,
    ) -> Result<String> {
        let cfg = step.matrixed_config.as_ref().ok_or_else(|| {
            anyhow::anyhow!("matrixed step '{}' is missing matrixed_config", step.name)
        })?;

        let mut pending = Vec::with_capacity(cfg.delegators.len() * cfg.prompt_variations.len());
        for delegator in &cfg.delegators {
            for (j, tpl) in cfg.prompt_variations.iter().enumerate() {
                let rendered = self.render_variant_prompt(tpl, ticket, working_dir_str)?;
                let full_prompt = apply_prompt_wrapping(rendered, base_options);
                pending.push(crate::state::PendingSubAgent {
                    delegator_name: delegator.clone(),
                    prompt: full_prompt,
                    variant_key: format!("{delegator}:{j}"),
                });
            }
        }

        self.launch_group(
            ticket,
            step,
            working_dir_str,
            base_options,
            "matrixed",
            pending,
        )
        .await
    }

    /// Shared fan-out implementation: create the group, launch as many
    /// sub-agents as slots allow, queue the rest.
    async fn launch_group(
        &self,
        ticket: &Ticket,
        step: &crate::templates::schema::StepSchema,
        working_dir_str: &str,
        base_options: &LaunchOptions,
        step_type_name: &str,
        pending: Vec<crate::state::PendingSubAgent>,
    ) -> Result<String> {
        if pending.is_empty() {
            anyhow::bail!(
                "multi-agent step '{}' produced zero sub-agents — check config",
                step.name
            );
        }

        // Create the group with everything in pending_launches.
        let group_id = {
            let mut state = State::load(&self.config)?;
            state.create_multi_agent_group(&ticket.id, &step.name, step_type_name, pending)?
        };

        // Drip-launch up to available_slots() sub-agents right now.
        self.launch_pending_sub_agents(ticket, &group_id, working_dir_str, base_options)
            .await?;

        // Return the first launched agent_id (or group_id if nothing launched yet).
        let state = State::load(&self.config)?;
        let group = state
            .multi_agent_groups
            .iter()
            .find(|g| g.group_id == group_id)
            .ok_or_else(|| anyhow::anyhow!("group '{group_id}' not found after creation"))?;
        Ok(group
            .agent_ids
            .first()
            .cloned()
            .unwrap_or_else(|| group_id.clone()))
    }

    /// Launch as many pending sub-agents as current slot budget allows.
    ///
    /// Called during initial fan-out AND by the sync loop when slots free up.
    pub async fn launch_pending_sub_agents(
        &self,
        ticket: &Ticket,
        group_id: &str,
        working_dir_str: &str,
        base_options: &LaunchOptions,
    ) -> Result<()> {
        loop {
            let budget = self.available_slots()?;
            if budget == 0 {
                break;
            }
            let next = {
                let state = State::load(&self.config)?;
                state.next_pending_for_group(group_id)
            };
            let Some(next) = next else {
                break;
            };

            let variant_key = next.variant_key.clone();
            let delegator_name = next.delegator_name.clone();
            let prompt = next.prompt.clone();

            let sub_opts = self.sub_agent_options(base_options, &delegator_name, &variant_key)?;
            let (agent_id, _session) = self
                .launch_one_sub_agent(ticket, working_dir_str, &prompt, &sub_opts)
                .await?;

            let mut state = State::load(&self.config)?;
            state.mark_launched(group_id, &variant_key, &agent_id)?;
        }
        Ok(())
    }

    /// Prepare a launch without executing it
    ///
    /// This method does everything needed to launch an agent (claim ticket,
    /// setup worktree, generate prompt, build command, register agent) but
    /// returns the command and details instead of executing in tmux.
    ///
    /// Use this for launching via VS Code terminals or other wrappers.
    pub async fn prepare_launch(
        &self,
        ticket: &Ticket,
        options: LaunchOptions,
    ) -> Result<PreparedLaunch> {
        // Clone ticket so we can update worktree info
        let mut ticket = ticket.clone();

        // Move ticket to in-progress
        let queue = Queue::new(&self.config)?;
        queue.claim_ticket(&ticket)?;

        // Best-effort: notify upstream kanban that ticket is now in-progress.
        let ks = KanbanBidirectionalSync::new(Arc::new(self.config.clone()));
        let ticket_clone = ticket.clone();
        tokio::spawn(async move {
            ks.on_ticket_claimed(&ticket_clone).await;
        });

        // Get project path (use override if provided)
        let project_path = if let Some(ref override_project) = options.project_override {
            PathBuf::from(self.get_project_path_for(override_project)?)
        } else {
            PathBuf::from(self.get_project_path(&ticket)?)
        };

        // Setup worktree for per-ticket isolation (if project is a git repo)
        let working_dir = setup_worktree_for_ticket(&self.config, &mut ticket, &project_path, None)
            .await
            .context("Failed to setup worktree for ticket")?;

        // Deploy operator skills for all tools this ticket may use across steps
        let primary_tool = options
            .provider
            .as_ref()
            .map_or("claude", |p| p.tool.as_str());
        let tools = self.collect_tools_for_ticket(&ticket, primary_tool);
        let tool_refs: Vec<&str> = tools.iter().map(std::string::String::as_str).collect();
        if let Err(e) = crate::llm::deploy_skills(&working_dir, &project_path, &tool_refs) {
            tracing::warn!(error = %e, "Failed to deploy skills (non-fatal)");
        }

        let worktree_created = ticket.worktree_path.is_some();
        let branch = ticket.branch.clone();
        let working_dir_str = working_dir.to_string_lossy().to_string();

        // Generate terminal/session name
        let terminal_name = format!("{}{}", SESSION_PREFIX, sanitize_session_name(&ticket.id));

        // Generate session UUID
        let session_uuid = generate_session_uuid();

        // Get the step name (use "initial" if not set)
        let step_name = if ticket.step.is_empty() {
            "initial".to_string()
        } else {
            ticket.step.clone()
        };

        // Store the session UUID in the ticket file (now in in-progress)
        let ticket_in_progress_path = self
            .config
            .tickets_path()
            .join("in-progress")
            .join(&ticket.filename);
        if ticket_in_progress_path.exists() {
            if let Ok(mut updated_ticket) = Ticket::from_file(&ticket_in_progress_path) {
                if let Err(e) = updated_ticket.set_session_id(&step_name, &session_uuid) {
                    tracing::warn!(
                        error = %e,
                        ticket = %ticket.id,
                        step = %step_name,
                        "Failed to store session UUID in ticket"
                    );
                }
            }
        }

        // Get the model and tool from options or use defaults
        let (tool_name, model) = if let Some(ref provider) = options.provider {
            (provider.tool.clone(), provider.model.clone())
        } else {
            let default_tool = self
                .config
                .llm_tools
                .detected
                .first()
                .map_or_else(|| "claude".to_string(), |t| t.name.clone());
            let default_model =
                get_default_model(&self.config).unwrap_or_else(|| "sonnet".to_string());
            (default_tool, default_model)
        };

        // Build the full prompt using the interpolation engine
        let initial_prompt = generate_prompt(&self.config, &ticket);
        let initial_prompt = apply_prompt_wrapping(initial_prompt, &options);
        let full_prompt = if get_template_prompt(&ticket.ticket_type).is_some() {
            let interpolator = PromptInterpolator::new();
            match interpolator.build_launch_prompt(&self.config, &ticket, &working_dir_str) {
                Ok(prompt) => prompt,
                Err(e) => {
                    tracing::warn!(
                        error = %e,
                        ticket = %ticket.id,
                        "Failed to build interpolated prompt, falling back to initial prompt"
                    );
                    initial_prompt
                }
            }
        } else if let Some(agent_prompt) = get_agent_prompt(&ticket.ticket_type) {
            let ticket_path = format!("../.tickets/in-progress/{}", ticket.filename);
            let message = format!(
                "use the {} agent to implement the ticket at {}",
                ticket.ticket_type.to_lowercase(),
                ticket_path
            );
            format!("{agent_prompt}\n---\n{message}")
        } else {
            initial_prompt
        };

        // Write prompt to file
        let prompt_file = write_prompt_file(&self.config, &session_uuid, &full_prompt)?;

        // Build command using the detected tool's template (with permissions)
        let mut llm_cmd = build_llm_command_with_permissions_for_tool(
            &self.config,
            &tool_name,
            &model,
            &session_uuid,
            &prompt_file,
            Some(&ticket),
            Some(&working_dir_str),
            options.operator_relay,
        )?;

        // Apply YOLO flags if enabled
        if options.yolo_mode {
            llm_cmd = apply_yolo_flags(&self.config, &llm_cmd, &tool_name);
        }

        // Apply extra flags from delegator launch_config
        if !options.extra_flags.is_empty() {
            llm_cmd = format!("{} {}", llm_cmd, options.extra_flags.join(" "));
        }

        // Wrap in docker command if docker mode is enabled
        if options.docker_mode {
            llm_cmd = build_docker_command(&self.config, &llm_cmd, &working_dir_str)?;
        }

        // Determine tool name from options or default
        let llm_tool = options
            .provider
            .as_ref()
            .map(|p| p.tool.clone())
            .or_else(|| {
                self.config
                    .llm_tools
                    .detected
                    .first()
                    .map(|t| t.name.clone())
            });

        // Update state with launch
        let mut state = State::load(&self.config)?;
        let agent_id = state.add_agent_with_options(
            ticket.id.clone(),
            ticket.ticket_type.clone(),
            ticket.project.clone(),
            ticket.is_paired(),
            llm_tool,
            Some(options.launch_mode_string()),
        )?;

        // Store session name in state for later recovery
        state.update_agent_session(&agent_id, &terminal_name)?;

        // Store worktree path in state (if one was created)
        if let Some(ref worktree_path) = ticket.worktree_path {
            state.update_agent_worktree_path(&agent_id, worktree_path)?;
        }

        // Set the current step in state
        if !ticket.step.is_empty() {
            state.update_agent_step(&agent_id, &ticket.step)?;
        }

        tracing::info!(
            terminal = %terminal_name,
            session_uuid = %session_uuid,
            project = %ticket.project,
            ticket = %ticket.id,
            step = %step_name,
            tool = %tool_name,
            launch_mode = %options.launch_mode_string(),
            working_dir = %working_dir_str,
            "Prepared agent launch"
        );

        Ok(PreparedLaunch {
            agent_id,
            ticket_id: ticket.id.clone(),
            working_directory: working_dir,
            command: llm_cmd,
            terminal_name,
            session_id: session_uuid,
            worktree_created,
            branch,
            session_wrapper: None,
            session_window_ref: None,
            session_context_ref: None,
        })
    }

    /// Prepare a relaunch for an in-progress ticket without executing it
    ///
    /// Similar to `prepare_launch()` but does NOT claim the ticket (it's already in-progress).
    /// Use this when relaunching a ticket that's already being worked on.
    pub async fn prepare_relaunch(
        &self,
        ticket: &Ticket,
        options: RelaunchOptions,
    ) -> Result<PreparedLaunch> {
        // Clone ticket so we can update worktree info if needed
        let mut ticket = ticket.clone();

        // Get project path (use override if provided)
        let project_path =
            if let Some(ref override_project) = options.launch_options.project_override {
                PathBuf::from(self.get_project_path_for(override_project)?)
            } else {
                PathBuf::from(self.get_project_path(&ticket)?)
            };

        let worktree_override = options.launch_options.use_worktrees_override;

        // Get working directory (reuse existing worktree or create new one)
        let working_dir = if let Some(ref worktree_path) = ticket.worktree_path {
            let path = PathBuf::from(worktree_path);
            if path.exists() {
                // Reuse existing worktree
                path
            } else {
                // Worktree was deleted, recreate it
                setup_worktree_for_ticket(
                    &self.config,
                    &mut ticket,
                    &project_path,
                    worktree_override,
                )
                .await
                .context("Failed to recreate worktree for ticket")?
            }
        } else {
            // No worktree yet, try to create one
            setup_worktree_for_ticket(&self.config, &mut ticket, &project_path, worktree_override)
                .await
                .context("Failed to setup worktree for ticket")?
        };

        // Deploy operator skills for all tools this ticket may use across steps
        let primary_tool = options
            .launch_options
            .provider
            .as_ref()
            .map_or("claude", |p| p.tool.as_str());
        let tools = self.collect_tools_for_ticket(&ticket, primary_tool);
        let tool_refs: Vec<&str> = tools.iter().map(std::string::String::as_str).collect();
        if let Err(e) = crate::llm::deploy_skills(&working_dir, &project_path, &tool_refs) {
            tracing::warn!(error = %e, "Failed to deploy skills (non-fatal)");
        }

        let worktree_created = ticket.worktree_path.is_some();
        let branch = ticket.branch.clone();
        let working_dir_str = working_dir.to_string_lossy().to_string();

        // Generate terminal/session name
        let terminal_name = format!("{}{}", SESSION_PREFIX, sanitize_session_name(&ticket.id));

        // Generate session UUID (or use existing for resume)
        let session_uuid = options
            .resume_session_id
            .clone()
            .unwrap_or_else(generate_session_uuid);

        // Get the step name (use "initial" if not set)
        let step_name = if ticket.step.is_empty() {
            "initial".to_string()
        } else {
            ticket.step.clone()
        };

        // Store the session UUID in the ticket file
        let ticket_in_progress_path = self
            .config
            .tickets_path()
            .join("in-progress")
            .join(&ticket.filename);
        if ticket_in_progress_path.exists() {
            if let Ok(mut updated_ticket) = Ticket::from_file(&ticket_in_progress_path) {
                if let Err(e) = updated_ticket.set_session_id(&step_name, &session_uuid) {
                    tracing::warn!(
                        error = %e,
                        ticket = %ticket.id,
                        step = %step_name,
                        "Failed to store session UUID in ticket"
                    );
                }
            }
        }

        // Get the model and tool from options or use defaults
        let (tool_name, model) = if let Some(ref provider) = options.launch_options.provider {
            (provider.tool.clone(), provider.model.clone())
        } else {
            let default_tool = self
                .config
                .llm_tools
                .detected
                .first()
                .map_or_else(|| "claude".to_string(), |t| t.name.clone());
            let default_model =
                get_default_model(&self.config).unwrap_or_else(|| "sonnet".to_string());
            (default_tool, default_model)
        };

        // Build the full prompt using the interpolation engine
        let initial_prompt = generate_prompt(&self.config, &ticket);
        let initial_prompt = apply_prompt_wrapping(initial_prompt, &options.launch_options);
        let mut full_prompt = if get_template_prompt(&ticket.ticket_type).is_some() {
            let interpolator = PromptInterpolator::new();
            match interpolator.build_launch_prompt(&self.config, &ticket, &working_dir_str) {
                Ok(prompt) => prompt,
                Err(e) => {
                    tracing::warn!(
                        error = %e,
                        ticket = %ticket.id,
                        "Failed to build interpolated prompt, falling back to initial prompt"
                    );
                    initial_prompt
                }
            }
        } else if let Some(agent_prompt) = get_agent_prompt(&ticket.ticket_type) {
            let ticket_path = format!("../.tickets/in-progress/{}", ticket.filename);
            let message = format!(
                "use the {} agent to implement the ticket at {}",
                ticket.ticket_type.to_lowercase(),
                ticket_path
            );
            format!("{agent_prompt}\n---\n{message}")
        } else {
            initial_prompt
        };

        // Append retry reason if provided
        if let Some(ref retry_reason) = options.retry_reason {
            full_prompt = format!(
                "{full_prompt}\n\n---\n## Relaunch Context\n\nThis ticket is being relaunched. Previous attempt feedback:\n{retry_reason}"
            );
        }

        // Write prompt to file
        let prompt_file = write_prompt_file(&self.config, &session_uuid, &full_prompt)?;

        // Build command using the detected tool's template (with permissions)
        let mut llm_cmd = build_llm_command_with_permissions_for_tool(
            &self.config,
            &tool_name,
            &model,
            &session_uuid,
            &prompt_file,
            Some(&ticket),
            Some(&working_dir_str),
            options.launch_options.operator_relay,
        )?;

        // Apply YOLO flags if enabled
        if options.launch_options.yolo_mode {
            llm_cmd = apply_yolo_flags(&self.config, &llm_cmd, &tool_name);
        }

        // Apply extra flags from delegator launch_config
        if !options.launch_options.extra_flags.is_empty() {
            llm_cmd = format!(
                "{} {}",
                llm_cmd,
                options.launch_options.extra_flags.join(" ")
            );
        }

        // Wrap in docker command if docker mode is enabled
        if options.launch_options.docker_mode {
            llm_cmd = build_docker_command(&self.config, &llm_cmd, &working_dir_str)?;
        }

        // Determine tool name from options or default
        let llm_tool = options
            .launch_options
            .provider
            .as_ref()
            .map(|p| p.tool.clone())
            .or_else(|| {
                self.config
                    .llm_tools
                    .detected
                    .first()
                    .map(|t| t.name.clone())
            });

        // Update state with launch
        let mut state = State::load(&self.config)?;
        let agent_id = state.add_agent_with_options(
            ticket.id.clone(),
            ticket.ticket_type.clone(),
            ticket.project.clone(),
            ticket.is_paired(),
            llm_tool,
            Some(options.launch_options.launch_mode_string()),
        )?;

        // Store session name in state for later recovery
        state.update_agent_session(&agent_id, &terminal_name)?;

        // Store worktree path in state (if one was created)
        if let Some(ref worktree_path) = ticket.worktree_path {
            state.update_agent_worktree_path(&agent_id, worktree_path)?;
        }

        // Set the current step in state
        if !ticket.step.is_empty() {
            state.update_agent_step(&agent_id, &ticket.step)?;
        }

        tracing::info!(
            terminal = %terminal_name,
            session_uuid = %session_uuid,
            project = %ticket.project,
            ticket = %ticket.id,
            step = %step_name,
            tool = %tool_name,
            launch_mode = %options.launch_options.launch_mode_string(),
            working_dir = %working_dir_str,
            relaunch = true,
            has_retry_reason = %options.retry_reason.is_some(),
            "Prepared agent relaunch"
        );

        Ok(PreparedLaunch {
            agent_id,
            ticket_id: ticket.id.clone(),
            working_directory: working_dir,
            command: llm_cmd,
            terminal_name,
            session_id: session_uuid,
            worktree_created,
            branch,
            session_wrapper: None,
            session_window_ref: None,
            session_context_ref: None,
        })
    }

    /// Relaunch an existing in-progress ticket (does NOT claim from queue)
    ///
    /// Used when a tmux session died but the ticket is still in progress.
    /// Can optionally resume from an existing Claude session ID.
    pub async fn relaunch(&self, ticket: &Ticket, options: RelaunchOptions) -> Result<String> {
        // Clone ticket so we can update worktree info if needed
        let mut ticket = ticket.clone();

        // Get working directory (use existing worktree, or setup new one)
        let project_path = PathBuf::from(self.get_project_path(&ticket)?);
        let worktree_override = options.launch_options.use_worktrees_override;
        let working_dir = if let Some(ref worktree_path) = ticket.worktree_path {
            let path = PathBuf::from(worktree_path);
            if path.exists() {
                // Reuse existing worktree
                path
            } else {
                // Worktree was deleted, recreate it
                setup_worktree_for_ticket(
                    &self.config,
                    &mut ticket,
                    &project_path,
                    worktree_override,
                )
                .await
                .context("Failed to recreate worktree for ticket")?
            }
        } else {
            // No worktree yet, try to create one
            setup_worktree_for_ticket(&self.config, &mut ticket, &project_path, worktree_override)
                .await
                .context("Failed to setup worktree for ticket")?
        };

        // Deploy operator skills for all tools this ticket may use across steps
        let primary_tool = options
            .launch_options
            .provider
            .as_ref()
            .map_or("claude", |p| p.tool.as_str());
        let tools = self.collect_tools_for_ticket(&ticket, primary_tool);
        let tool_refs: Vec<&str> = tools.iter().map(std::string::String::as_str).collect();
        if let Err(e) = crate::llm::deploy_skills(&working_dir, &project_path, &tool_refs) {
            tracing::warn!(error = %e, "Failed to deploy skills (non-fatal)");
        }

        let working_dir_str = working_dir.to_string_lossy().to_string();

        // Generate the initial prompt for the agent
        let initial_prompt = generate_prompt(&self.config, &ticket);
        let initial_prompt = apply_prompt_wrapping(initial_prompt, &options.launch_options);

        // Dispatch based on session wrapper type
        let (session_name, wrapper_name, cmux_refs) =
            if self.config.sessions.wrapper == SessionWrapperType::Cmux {
                let cmux = self.cmux.as_ref().ok_or_else(|| {
                    anyhow::anyhow!("cmux client not initialized but wrapper type is cmux")
                })?;
                let result = launch_in_cmux_with_relaunch_options(
                    &self.config,
                    cmux,
                    &ticket,
                    &working_dir_str,
                    &initial_prompt,
                    &options,
                )?;
                (
                    result.session_name,
                    "cmux",
                    Some((result.window_ref, result.workspace_ref)),
                )
            } else if self.config.sessions.wrapper == SessionWrapperType::Zellij {
                let zellij = self.zellij.as_ref().ok_or_else(|| {
                    anyhow::anyhow!("zellij client not initialized but wrapper type is zellij")
                })?;
                let result = launch_in_zellij_with_relaunch_options(
                    &self.config,
                    zellij,
                    &ticket,
                    &working_dir_str,
                    &initial_prompt,
                    &options,
                )?;
                (result.session_name, "zellij", None)
            } else {
                let name = launch_in_tmux_with_relaunch_options(
                    &self.config,
                    &self.tmux,
                    &ticket,
                    &working_dir_str,
                    &initial_prompt,
                    &options,
                )?;
                (name, "tmux", None)
            };

        // Determine tool name from options or default
        let llm_tool = options
            .launch_options
            .provider
            .as_ref()
            .map(|p| p.tool.clone())
            .or_else(|| {
                self.config
                    .llm_tools
                    .detected
                    .first()
                    .map(|t| t.name.clone())
            });

        // Update state with new agent
        let mut state = State::load(&self.config)?;
        let agent_id = state.add_agent_with_options(
            ticket.id.clone(),
            ticket.ticket_type.clone(),
            ticket.project.clone(),
            ticket.is_paired(),
            llm_tool,
            Some(options.launch_options.launch_mode_string()),
        )?;

        // Store session name in state for later recovery
        state.update_agent_session(&agent_id, &session_name)?;

        // Store session wrapper type
        state.update_agent_session_wrapper(&agent_id, wrapper_name)?;

        // Store session refs if applicable (e.g. cmux window + workspace)
        if let Some((window_ref, workspace_ref)) = cmux_refs {
            state.update_agent_session_refs(
                &agent_id,
                Some(&window_ref),
                Some(&workspace_ref),
                None,
            )?;
        }

        // Store worktree path in state (if one was created)
        if let Some(ref worktree_path) = ticket.worktree_path {
            state.update_agent_worktree_path(&agent_id, worktree_path)?;
        }

        // Set the current step in state
        if !ticket.step.is_empty() {
            state.update_agent_step(&agent_id, &ticket.step)?;
        }

        // Send notification
        if self.config.notifications.enabled && self.config.notifications.on_agent_start {
            let mode_suffix = if options.resume_session_id.is_some() {
                " [resumed]"
            } else {
                " [restarted]"
            };
            let worktree_suffix = if ticket.worktree_path.is_some() {
                " [worktree]"
            } else {
                ""
            };
            // TODO: Migrate to NotificationService when Launcher has access to it
            #[allow(deprecated)]
            notifications::send(
                "Agent Relaunched",
                &format!(
                    "{} - {} ({}: {}){}{}",
                    ticket.project,
                    ticket.ticket_type,
                    wrapper_name,
                    session_name,
                    mode_suffix,
                    worktree_suffix
                ),
                &ticket.summary,
                self.config.notifications.sound,
            )?;
        }

        Ok(agent_id)
    }

    fn get_project_path(&self, ticket: &Ticket) -> Result<String> {
        self.get_project_path_for(&ticket.project)
    }

    /// Get project path for a given project name
    fn get_project_path_for(&self, project: &str) -> Result<String> {
        let projects_root = self.config.projects_path();

        let project_path = if project == "global" {
            // Global tickets use the root directory
            projects_root
        } else {
            projects_root.join(project)
        };

        if !project_path.exists() {
            anyhow::bail!("Project path does not exist: {}", project_path.display());
        }

        Ok(project_path.to_string_lossy().to_string())
    }

    /// List all operator tmux sessions
    pub fn list_sessions(&self) -> Result<Vec<String>> {
        match self.tmux.list_sessions(Some(SESSION_PREFIX)) {
            Ok(sessions) => Ok(sessions.into_iter().map(|s| s.name).collect()),
            Err(TmuxError::NotInstalled) => {
                tracing::warn!("tmux not installed, returning empty session list");
                Ok(Vec::new())
            }
            Err(e) => {
                tracing::warn!(error = %e, "Failed to list tmux sessions");
                Ok(Vec::new())
            }
        }
    }

    /// Kill a specific operator tmux session
    pub fn kill_session(&self, session_name: &str) -> Result<()> {
        self.tmux
            .kill_session(session_name)
            .context("Failed to kill tmux session")?;
        Ok(())
    }

    /// Capture the current content of a session's pane
    pub fn capture_session_content(&self, session_name: &str) -> Result<String> {
        self.tmux
            .capture_pane(session_name, false)
            .context("Failed to capture pane content")
    }

    /// Check if a session is still alive
    pub fn session_alive(&self, session_name: &str) -> bool {
        matches!(self.tmux.session_exists(session_name), Ok(true))
    }

    /// Attach to a tmux session (returns the command to run)
    pub fn attach_command(session_name: &str) -> String {
        format!("tmux attach -t {session_name}")
    }
}

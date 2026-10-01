#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

//! Ticket-session synchronization for keeping ticket metadata in sync with tmux sessions.
//!
//! This module provides periodic and manual synchronization between:
//! - Ticket files (in .tickets/in-progress/)
//! - Agent state (in state.json)
//! - Tmux sessions (op-{ticket-id})

use std::sync::Arc;
use std::time::{Duration, Instant};

use std::path::PathBuf;

use anyhow::{Context, Result};

use super::monitor::{HealthCheckResult, SessionMonitor};
use super::tmux::TmuxClient;
use super::visual_review::VisualReviewHandler;
use crate::agents::judge::{
    default_judge_factory, judging_step, run_judge, JudgeFactory, JudgingStep,
};
use crate::agents::launcher::worktree_setup::cleanup_ticket_worktree;
use crate::agents::ProofResult;
use crate::config::Config;
use crate::llm::native::JudgeVerdict;
use crate::queue::{Queue, StepAdvanceResult, Ticket};
use crate::state::{AgentState, MultiAgentGroup, State};
use crate::steps::manager::StepManager;
use crate::templates::schema::{ReviewType, StepSchema};
use crate::templates::step_type;

/// Status message for a finished proof run - mirrors the strings the
/// `complete_step` proof hook (`rest/routes/launch.rs`) produces, so the
/// message reads the same regardless of which path ran the proof.
fn proof_result_message(result: &ProofResult, proof_ref: &str) -> String {
    if result.timed_out {
        format!(
            "Proof FAILED (timeout, exit {}) - awaiting review ({proof_ref})",
            result.exit_code
        )
    } else if result.passed {
        format!("Proof passed - awaiting review ({proof_ref})")
    } else {
        format!(
            "Proof FAILED (exit {}) - awaiting review ({proof_ref})",
            result.exit_code
        )
    }
}

/// Read and format the message for an already-persisted `result.json`.
/// Returns `None` on any read/parse failure so callers fall back to a
/// generic message.
fn read_proof_result_message(result_path: &std::path::Path, proof_ref: &str) -> Option<String> {
    let contents = std::fs::read_to_string(result_path).ok()?;
    let result: ProofResult = serde_json::from_str(&contents).ok()?;
    Some(proof_result_message(&result, proof_ref))
}

/// Deterministic aggregation of a finished group's outputs by step type.
fn aggregate_outputs(
    group: &MultiAgentGroup,
    step_schema: Option<&StepSchema>,
) -> serde_json::Value {
    let outputs = &group.individual_outputs;
    match group.step_type.as_str() {
        "multi_model" => step_schema
            .and_then(|s| s.multi_model_config.as_ref())
            .map_or(serde_json::Value::Null, |cfg| {
                step_type::aggregate_multi_model(outputs, cfg)
            }),
        "multi_prompt" => step_schema
            .and_then(|s| s.multi_prompt_config.as_ref())
            .map_or(serde_json::Value::Null, |cfg| {
                step_type::aggregate_multi_prompt(outputs, cfg)
            }),
        "matrixed" => step_schema
            .and_then(|s| s.matrixed_config.as_ref())
            .map_or(serde_json::Value::Null, |cfg| {
                step_type::aggregate_matrixed(outputs, cfg, &group.step_name)
            }),
        other => {
            tracing::warn!(
                step_type = other,
                "unknown multi-agent step_type, skipping aggregation"
            );
            serde_json::Value::Null
        }
    }
}

fn note_history(ticket: &mut Ticket, message: &str, result: &mut SyncResult) {
    if let Err(e) = ticket.append_history(&format!(
        "- **{}** - {message}",
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
    )) {
        result
            .errors
            .push(format!("Failed to add history for {}: {e}", ticket.id));
    }
}

/// Result of a sync cycle
#[derive(Debug, Default)]
pub struct SyncResult {
    /// Number of tickets synced
    pub synced: usize,
    /// Tickets moved to AWAITING status
    pub moved_to_awaiting: Vec<String>,
    /// Tickets that timed out
    pub timed_out: Vec<String>,
    /// Tickets that completed their step
    pub completed: Vec<String>,
    /// Agent IDs that resumed from awaiting state
    pub resumed: Vec<String>,
    /// Errors encountered during sync
    pub errors: Vec<String>,
}

/// Action taken for a single ticket during sync
#[derive(Debug, Clone, PartialEq)]
pub enum SyncAction {
    /// No changes needed
    NoChange,
    /// Status was updated
    UpdatedStatus(String),
    /// Ticket moved to AWAITING
    MovedToAwaiting,
    /// Step completed, ready for next step
    StepCompleted,
    /// Step timed out
    TimedOut,
    /// Session is hung (no content change)
    Hung,
    /// Resumed from awaiting state (user returned and LLM is active)
    ResumedFromAwaiting,
}

/// Ticket-session synchronizer
pub struct TicketSessionSync {
    config: Config,
    tmux: Arc<dyn TmuxClient>,
    last_sync: Instant,
    sync_interval: Duration,
    judge_factory: JudgeFactory,
}

impl TicketSessionSync {
    /// Create a new sync manager
    pub fn new(config: &Config, tmux: Arc<dyn TmuxClient>) -> Self {
        Self {
            config: config.clone(),
            tmux,
            last_sync: Instant::now()
                .checked_sub(Duration::from_secs(config.agents.sync_interval))
                .unwrap_or_else(Instant::now),
            sync_interval: Duration::from_secs(config.agents.sync_interval),
            judge_factory: default_judge_factory(),
        }
    }

    #[cfg(test)]
    fn with_judge_factory(mut self, judge_factory: JudgeFactory) -> Self {
        self.judge_factory = judge_factory;
        self
    }

    /// Check if it's time to run a sync
    pub fn should_sync(&self) -> bool {
        self.last_sync.elapsed() >= self.sync_interval
    }

    /// Force a sync now (resets the timer)
    pub fn force_sync(&mut self) {
        // Set last_sync to a time in the past to trigger immediate sync
        self.last_sync = Instant::now()
            .checked_sub(self.sync_interval)
            .unwrap_or_else(Instant::now);
    }

    /// Sync all in-progress tickets with their sessions
    pub fn sync_all(
        &mut self,
        state: &mut State,
        queue: &Queue,
        health_result: &HealthCheckResult,
    ) -> Result<SyncResult> {
        self.last_sync = Instant::now();
        let mut result = SyncResult::default();

        // Get all in-progress tickets
        let tickets = queue.list_in_progress()?;

        for mut ticket in tickets {
            // If this ticket has an active multi-agent group, route sub-agent
            // completions through group-aware aggregation instead of the
            // per-agent single-agent path.
            if state.get_group_for_ticket(&ticket.id).is_some() {
                if let Err(e) =
                    self.sync_multi_agent_ticket(&mut ticket, state, health_result, &mut result)
                {
                    result.errors.push(format!(
                        "Failed to sync multi-agent ticket {}: {e}",
                        ticket.id
                    ));
                }
                result.synced += 1;
                continue;
            }

            // Find the corresponding agent (single-agent path)
            if let Some(agent) = state.agent_by_ticket(&ticket.id) {
                let agent_id = agent.id.clone();
                let session_name = agent.session_name.clone().unwrap_or_default();
                let worktree_path = agent.worktree_path.clone();

                // Determine the sync action based on health check results
                let action = self.determine_action(&ticket, &session_name, health_result);

                match action {
                    SyncAction::NoChange => {}
                    SyncAction::MovedToAwaiting => {
                        // Get the ticket's current step review type
                        let review_type = ticket
                            .current_step_schema()
                            .map_or(ReviewType::None, |s| s.review_type);

                        let step_display = ticket.current_step_display_name();

                        // Handle based on review type
                        match review_type {
                            ReviewType::None => {
                                // Standard awaiting_input (agent is stuck)
                                state.update_agent_status(&agent_id, "awaiting_input", None)?;
                            }
                            ReviewType::Plan => {
                                // Agent completed step, awaiting plan review
                                state.update_agent_status(
                                    &agent_id,
                                    "awaiting_input",
                                    Some("Awaiting plan approval".to_string()),
                                )?;
                                state.set_agent_review_state(&agent_id, "pending_plan")?;
                                tracing::info!(
                                    ticket_id = %ticket.id,
                                    step = %step_display,
                                    "Step awaiting plan review"
                                );
                            }
                            ReviewType::Visual => {
                                // Trigger visual review flow
                                if let Some(ref visual_config) =
                                    ticket.current_step_schema().and_then(|s| s.visual_config)
                                {
                                    state.update_agent_status(
                                        &agent_id,
                                        "awaiting_input",
                                        Some(format!("Visual review: {}", visual_config.url)),
                                    )?;
                                    state.set_agent_review_state(&agent_id, "pending_visual")?;

                                    // Open browser (fire and forget)
                                    let _ = VisualReviewHandler::open_browser(&visual_config.url);

                                    tracing::info!(
                                        ticket_id = %ticket.id,
                                        step = %step_display,
                                        url = %visual_config.url,
                                        "Opened browser for visual review"
                                    );
                                } else {
                                    // Visual review without config - treat as plan review
                                    state.update_agent_status(
                                        &agent_id,
                                        "awaiting_input",
                                        Some("Visual review (no config)".to_string()),
                                    )?;
                                    state.set_agent_review_state(&agent_id, "pending_visual")?;
                                }
                            }
                            ReviewType::Pr => {
                                // Trigger PR creation flow
                                // PR creation is async - set state and let background task handle it
                                state.update_agent_status(
                                    &agent_id,
                                    "awaiting_input",
                                    Some("Creating PR...".to_string()),
                                )?;
                                state.set_agent_review_state(&agent_id, "pending_pr_creation")?;
                                tracing::info!(
                                    ticket_id = %ticket.id,
                                    step = %step_display,
                                    "Step requires PR review - awaiting PR creation"
                                );
                            }
                            ReviewType::Proof => {
                                self.handle_proof_awaiting(
                                    &ticket,
                                    &agent_id,
                                    &step_display,
                                    &worktree_path,
                                    state,
                                )?;
                            }
                        }

                        // Add history entry to ticket
                        if let Err(e) = ticket.add_awaiting_entry(&step_display) {
                            result.errors.push(format!(
                                "Failed to add history entry for {}: {}",
                                ticket.id, e
                            ));
                        }

                        // Reset silence flag after handling
                        let _ = self.tmux.reset_silence_flag(&session_name);

                        result.moved_to_awaiting.push(ticket.id.clone());
                        tracing::info!(
                            ticket_id = %ticket.id,
                            step = %step_display,
                            review_type = ?review_type,
                            "Ticket moved to AWAITING"
                        );
                    }
                    SyncAction::TimedOut => {
                        // Timeout is treated as AWAITING with timeout note
                        state.update_agent_status(
                            &agent_id,
                            "awaiting_input",
                            Some("Step timed out".to_string()),
                        )?;

                        // Add timeout entry to history
                        let step_display = ticket.current_step_display_name();
                        if let Err(e) = ticket.append_history(&format!(
                            "- **{}** - Step \"{}\" timed out after {} minutes",
                            chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                            step_display,
                            self.config.agents.step_timeout / 60
                        )) {
                            result.errors.push(format!(
                                "Failed to add timeout entry for {}: {}",
                                ticket.id, e
                            ));
                        }

                        result.timed_out.push(ticket.id.clone());
                        tracing::warn!(
                            ticket_id = %ticket.id,
                            step = %step_display,
                            "Step timed out"
                        );
                    }
                    SyncAction::UpdatedStatus(new_status) => {
                        state.update_agent_status(&agent_id, &new_status, None)?;
                    }
                    SyncAction::StepCompleted => {
                        let step_display = ticket.current_step_display_name();

                        match ticket.advance_step() {
                            Ok(StepAdvanceResult::Advanced { step, switch_agent }) => {
                                state.update_agent_step(&agent_id, &step)?;

                                if let Err(e) = ticket.append_history(&format!(
                                    "- **{}** - Step \"{}\" completed (artifact detected), advancing to \"{}\"",
                                    chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                                    step_display,
                                    step,
                                )) {
                                    result.errors.push(format!(
                                        "Failed to add history for {}: {}",
                                        ticket.id, e
                                    ));
                                }

                                if let Some(ref delegator_name) = switch_agent {
                                    state.set_agent_review_state(
                                        &agent_id,
                                        &format!("switching_agent:{delegator_name}"),
                                    )?;
                                }

                                result.completed.push(ticket.id.clone());
                                tracing::info!(
                                    ticket_id = %ticket.id,
                                    from_step = %step_display,
                                    to_step = %step,
                                    switch_agent = ?switch_agent,
                                    "Step completed via artifact detection, advanced to next step"
                                );
                            }
                            Ok(StepAdvanceResult::FinalStep) => {
                                state.update_agent_status(
                                    &agent_id,
                                    "completing",
                                    Some("All steps completed".to_string()),
                                )?;
                                // Coder targets: stop the finished workspace
                                // (never delete). Best-effort by design.
                                if let Some(agent) =
                                    state.agents.iter().find(|a| a.id == agent_id).cloned()
                                {
                                    crate::agents::launcher::coder::stop_on_complete_for_agent(
                                        &self.config,
                                        &agent,
                                    );
                                }
                                result.completed.push(ticket.id.clone());
                                tracing::info!(
                                    ticket_id = %ticket.id,
                                    step = %step_display,
                                    "Final step completed via artifact detection"
                                );
                            }
                            Err(e) => {
                                result.errors.push(format!(
                                    "Failed to advance step for {}: {}",
                                    ticket.id, e
                                ));
                            }
                        }
                    }
                    SyncAction::Hung => {
                        // Session is hung but not necessarily awaiting input
                        tracing::warn!(
                            ticket_id = %ticket.id,
                            "Session appears hung (no content changes)"
                        );
                    }
                    SyncAction::ResumedFromAwaiting => {
                        // Agent resumed from awaiting state (content changed while awaiting)
                        state.update_agent_status(&agent_id, "running", None)?;
                        // Clear any review state since agent is now active
                        state.set_agent_review_state(&agent_id, "")?;
                        result.resumed.push(ticket.id.clone());
                        tracing::info!(
                            ticket_id = %ticket.id,
                            "Agent resumed from awaiting state - now running"
                        );
                    }
                }

                result.synced += 1;
            }
        }

        Ok(result)
    }

    /// Gate a Proof step at `MovedToAwaiting`
    fn handle_proof_awaiting(
        &self,
        ticket: &Ticket,
        agent_id: &str,
        step_display: &str,
        worktree_path: &Option<String>,
        state: &mut State,
    ) -> Result<()> {
        let Some(proof_config) = ticket.current_step_schema().and_then(|s| s.proof_config) else {
            state.update_agent_status(
                agent_id,
                "awaiting_input",
                Some("Proof review (no config)".to_string()),
            )?;
            state.set_agent_review_state(agent_id, "pending_proof")?;
            return Ok(());
        };

        let worktree_root = worktree_path
            .clone()
            .map(PathBuf::from)
            .unwrap_or_else(|| self.config.projects_path().join(&ticket.project));
        let proof_ref = format!(".proof/{}/{}", ticket.id, ticket.step);
        let result_path =
            crate::agents::ProofRunner::result_path(&worktree_root, &ticket.id, &ticket.step);
        let already_ran = result_path.exists();

        let message = if already_ran {
            // Already ran (e.g. via complete_step) - never rerun.
            read_proof_result_message(&result_path, &proof_ref)
                .unwrap_or_else(|| "Proof review: awaiting approval".to_string())
        } else {
            "Proof review: awaiting approval".to_string()
        };

        state.update_agent_status(agent_id, "awaiting_input", Some(message))?;
        state.set_agent_review_state(agent_id, "pending_proof")?;

        if !already_ran {
            self.spawn_proof_run(ticket, agent_id, proof_config, worktree_root, proof_ref);
        }

        tracing::info!(
            ticket_id = %ticket.id,
            step = %step_display,
            "Step awaiting proof review"
        );
        Ok(())
    }

    /// Fire-and-forget the proof run. `sync_all` doesn't hold a live `State`
    /// handle across await points, so the spawned task loads/saves its own
    /// copy on completion - the same pattern `launch.rs`'s proof hook uses.
    fn spawn_proof_run(
        &self,
        ticket: &Ticket,
        agent_id: &str,
        proof_config: crate::templates::schema::ProofReviewConfig,
        worktree_root: PathBuf,
        proof_ref: String,
    ) {
        let config = self.config.clone();
        let ticket_id = ticket.id.clone();
        let step_name = ticket.step.clone();
        let agent_id = agent_id.to_string();
        let context = serde_json::json!({"ticket_id": ticket_id, "step": step_name});

        tokio::spawn(async move {
            let runner = crate::agents::ProofRunner::with_context(context);
            let message = match runner
                .run(&proof_config, &worktree_root, &ticket_id, &step_name)
                .await
            {
                Ok(result) => proof_result_message(&result, &proof_ref),
                Err(e) => {
                    tracing::warn!(
                        ticket_id = %ticket_id,
                        step = %step_name,
                        error = %e,
                        "Proof runner error"
                    );
                    "Proof runner error - awaiting review".to_string()
                }
            };
            match State::load(&config) {
                Ok(mut app_state) => {
                    if let Err(e) =
                        app_state.update_agent_status(&agent_id, "awaiting_input", Some(message))
                    {
                        tracing::warn!(
                            agent_id = %agent_id,
                            error = %e,
                            "Failed to persist proof result status"
                        );
                    }
                }
                Err(e) => tracing::warn!(
                    error = %e,
                    "Failed to load state for proof result status"
                ),
            }
        });
    }

    /// Sync a multi-agent ticket: iterate each sub-agent in the group,
    /// collect their outputs, and when all `expected_total` sub-agents
    /// have reported, aggregate + write artifact + advance the step once.
    fn sync_multi_agent_ticket(
        &mut self,
        ticket: &mut Ticket,
        state: &mut State,
        health_result: &HealthCheckResult,
        result: &mut SyncResult,
    ) -> Result<()> {
        use crate::state::MultiAgentPhase;

        // Snapshot the group so we can iterate without holding a borrow on state.
        let group = state
            .get_group_for_ticket(&ticket.id)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("group vanished while syncing ticket {}", ticket.id))?;

        let group_id = group.group_id.clone();
        let step_name = group.step_name.clone();

        match group.phase {
            MultiAgentPhase::FanOut => {}
            MultiAgentPhase::Voting => {
                let step_schema = ticket.current_step_schema();
                return self.sync_judging_group(
                    ticket,
                    state,
                    &group,
                    step_schema.as_ref(),
                    result,
                );
            }
            MultiAgentPhase::Complete | MultiAgentPhase::Failed => return Ok(()),
        }

        // Process each launched sub-agent's health-check action.
        let agent_ids = group.agent_ids.clone();
        for agent_id in &agent_ids {
            let session_name = state
                .agents
                .iter()
                .find(|a| &a.id == agent_id)
                .and_then(|a| a.session_name.clone())
                .unwrap_or_default();

            let action = self.determine_action(ticket, &session_name, health_result);
            match action {
                SyncAction::StepCompleted => {
                    // Read this sub-agent's output file (keyed by the
                    // agent_id the REST handler used when writing it).
                    let output = StepManager::read_agent_step_output(ticket, &step_name, agent_id);
                    let all_done = state.record_agent_output(agent_id, output)?;

                    // Mark this sub-agent as completing so we stop polling it.
                    state.update_agent_status(
                        agent_id,
                        "completing",
                        Some("sub-agent complete".to_string()),
                    )?;
                    let _ = self.tmux.reset_silence_flag(&session_name);

                    if all_done {
                        // Re-fetch the now-fully-populated group to aggregate.
                        let finished =
                            state
                                .get_group_for_ticket(&ticket.id)
                                .cloned()
                                .ok_or_else(|| {
                                    anyhow::anyhow!("group {group_id} missing after all_done")
                                })?;

                        let step_schema = ticket.current_step_schema();
                        self.aggregate_or_start_judging(
                            ticket,
                            state,
                            &finished,
                            step_schema.as_ref(),
                            result,
                        )?;
                        // Other sub-agents (if any) were already completing;
                        // the group is now judging or finalized, exit the loop.
                        break;
                    }
                }
                SyncAction::TimedOut | SyncAction::Hung => {
                    tracing::warn!(
                        ticket_id = %ticket.id,
                        agent_id = %agent_id,
                        action = ?action,
                        "Multi-agent sub-agent stuck (will not advance step)"
                    );
                }
                SyncAction::UpdatedStatus(new_status) => {
                    state.update_agent_status(agent_id, &new_status, None)?;
                }
                // Awaiting / resumed / no-change: ignore at group level for v1.
                SyncAction::NoChange
                | SyncAction::MovedToAwaiting
                | SyncAction::ResumedFromAwaiting => {}
            }
        }

        Ok(())
    }

    /// All sub-agents reported. Start the LLM judge when the step asks for
    /// model-based selection and a judge is usable; otherwise finalize now
    /// with the deterministic rule.
    fn aggregate_or_start_judging(
        &mut self,
        ticket: &mut Ticket,
        state: &mut State,
        group: &MultiAgentGroup,
        step_schema: Option<&StepSchema>,
        result: &mut SyncResult,
    ) -> Result<()> {
        let plan = match (step_schema, ticket.worktree_path.clone()) {
            (Some(schema), Some(worktree)) => {
                let render = |t: &str| {
                    StepManager::render_ticket_template(t, ticket).unwrap_or_else(|_| t.to_string())
                };
                step_type::judge_plan(schema, &group.individual_outputs, &render)
                    .map(|plan| (plan, worktree))
            }
            _ => None,
        };
        let Some((plan, worktree)) = plan else {
            return self.finalize_group(ticket, state, group, step_schema, None, result);
        };

        let judge = match (self.judge_factory)(&self.config) {
            Ok(Some(judge)) => judge,
            Ok(None) => {
                return self.finalize_group(ticket, state, group, step_schema, None, result)
            }
            Err(e) => {
                note_history(
                    ticket,
                    &format!("Judge unavailable ({e}); used deterministic selection"),
                    result,
                );
                return self.finalize_group(ticket, state, group, step_schema, None, result);
            }
        };
        let Ok(runtime) = tokio::runtime::Handle::try_current() else {
            note_history(
                ticket,
                "Judge unavailable (no async runtime); used deterministic selection",
                result,
            );
            return self.finalize_group(ticket, state, group, step_schema, None, result);
        };

        let attempt = state.begin_judging(&group.group_id, judge.timeout_secs)?;
        let step_name = group.step_name.clone();
        let ticket_id = ticket.id.clone();
        tracing::info!(
            ticket_id = %ticket_id,
            step = %step_name,
            attempt = %attempt.attempt_id,
            "Multi-agent step judging"
        );
        runtime.spawn(async move {
            let outcome = run_judge(judge.llm, plan, attempt.timeout_secs).await;
            if let Err(e) = StepManager::write_judge_outcome(
                &worktree,
                &step_name,
                &attempt.attempt_id,
                &outcome,
            ) {
                // The sync loop's deadline fallback still finalizes the group.
                tracing::warn!(ticket_id = %ticket_id, error = %e, "Failed to write judge outcome");
            }
        });
        Ok(())
    }

    /// Poll a judging group: finalize on a verdict, on a failure, or once the
    /// attempt's deadline passes (covers a daemon restart mid-judge).
    fn sync_judging_group(
        &mut self,
        ticket: &mut Ticket,
        state: &mut State,
        group: &MultiAgentGroup,
        step_schema: Option<&StepSchema>,
        result: &mut SyncResult,
    ) -> Result<()> {
        let step = match group.judge_attempt.as_ref() {
            Some(attempt) => judging_step(
                attempt,
                StepManager::read_judge_outcome(ticket, &group.step_name, &attempt.attempt_id),
                chrono::Utc::now(),
            ),
            None => JudgingStep::Fallback("judging phase without an attempt".to_string()),
        };
        match step {
            JudgingStep::Wait => Ok(()),
            JudgingStep::Apply(verdict) => {
                self.finalize_group(ticket, state, group, step_schema, Some(&verdict), result)
            }
            JudgingStep::Fallback(reason) => {
                note_history(
                    ticket,
                    &format!("Judge fell back to deterministic selection: {reason}"),
                    result,
                );
                self.finalize_group(ticket, state, group, step_schema, None, result)
            }
        }
    }

    /// Aggregate (applying the judge's pick if any), write the step artifact,
    /// complete the group, advance the ticket once, and retire the sub-agents.
    fn finalize_group(
        &mut self,
        ticket: &mut Ticket,
        state: &mut State,
        group: &MultiAgentGroup,
        step_schema: Option<&StepSchema>,
        judged: Option<&JudgeVerdict>,
        result: &mut SyncResult,
    ) -> Result<()> {
        let mut aggregated = aggregate_outputs(group, step_schema);
        if let (Some(verdict), Some(schema)) = (judged, step_schema) {
            let message = if step_type::apply_judge_verdict(
                &mut aggregated,
                schema,
                verdict.winner_index,
                &verdict.rationale,
            ) {
                format!(
                    "Judge selected candidate {}: {}",
                    verdict.winner_index, verdict.rationale
                )
            } else {
                format!(
                    "Judge verdict {} did not fit the step; used deterministic selection",
                    verdict.winner_index
                )
            };
            note_history(ticket, &message, result);
        }

        // Persist the aggregated artifact for the next step to read.
        StepManager::write_step_output_artifact(ticket, &group.step_name, &aggregated)?;

        // Mark the group complete with the aggregated result.
        state.complete_group(&group.group_id, aggregated)?;

        // Advance the ticket's step exactly once for the group.
        let step_display = ticket.current_step_display_name();
        match ticket.advance_step() {
            Ok(StepAdvanceResult::Advanced { step, .. }) => {
                note_history(
                    ticket,
                    &format!(
                        "Multi-agent step \"{step_display}\" completed, advancing to \"{step}\""
                    ),
                    result,
                );
                tracing::info!(
                    ticket_id = %ticket.id,
                    step = %step_display,
                    next = %step,
                    "Multi-agent step aggregated, advanced"
                );
            }
            Ok(StepAdvanceResult::FinalStep) => {
                tracing::info!(
                    ticket_id = %ticket.id,
                    step = %step_display,
                    "Multi-agent final step completed"
                );
            }
            Err(e) => {
                result
                    .errors
                    .push(format!("Failed to advance step for {}: {e}", ticket.id));
            }
        }

        // Remove all sub-agent records now that the group is done.
        // Coder targets: stop each finished workspace first.
        for aid in &group.agent_ids {
            if let Some(agent) = state.agents.iter().find(|a| &a.id == aid).cloned() {
                crate::agents::launcher::coder::stop_on_complete_for_agent(&self.config, &agent);
            }
            state.remove_agent(aid)?;
        }
        state.cleanup_finished_groups()?;

        result.completed.push(ticket.id.clone());
        Ok(())
    }

    /// Determine what sync action to take for a ticket based on health check results
    fn determine_action(
        &self,
        ticket: &Ticket,
        session_name: &str,
        health_result: &HealthCheckResult,
    ) -> SyncAction {
        // Check if timed out (takes priority)
        if health_result.timed_out.iter().any(|s| s == session_name) {
            return SyncAction::TimedOut;
        }

        // Check if resumed from awaiting (before checking awaiting_input)
        if health_result.resumed.iter().any(|s| s == session_name) {
            return SyncAction::ResumedFromAwaiting;
        }

        // Check if awaiting input (via hooks, patterns, or silence)
        if health_result
            .awaiting_input
            .iter()
            .any(|s| s == session_name)
        {
            // If artifacts are ready, this is a step completion (positive signal)
            if health_result
                .artifact_ready
                .iter()
                .any(|s| s == session_name)
            {
                return SyncAction::StepCompleted;
            }
            return SyncAction::MovedToAwaiting;
        }

        // Check if orphaned
        if health_result.orphaned.iter().any(|s| s == session_name) {
            return SyncAction::UpdatedStatus("orphaned".to_string());
        }

        SyncAction::NoChange
    }

    /// Sync a single ticket (useful for manual sync of specific ticket)
    pub fn sync_ticket(
        &self,
        ticket: &mut Ticket,
        state: &mut State,
        health_result: &HealthCheckResult,
    ) -> Result<SyncAction> {
        if let Some(agent) = state.agent_by_ticket(&ticket.id) {
            let agent_id = agent.id.clone();
            let session_name = agent.session_name.clone().unwrap_or_default();

            let action = self.determine_action(ticket, &session_name, health_result);

            match &action {
                SyncAction::MovedToAwaiting => {
                    state.update_agent_status(&agent_id, "awaiting_input", None)?;
                    let step_display = ticket.current_step_display_name();
                    ticket.add_awaiting_entry(&step_display)?;
                    let _ = self.tmux.reset_silence_flag(&session_name);
                }
                SyncAction::TimedOut => {
                    state.update_agent_status(
                        &agent_id,
                        "awaiting_input",
                        Some("Step timed out".to_string()),
                    )?;
                    let step_display = ticket.current_step_display_name();
                    ticket.append_history(&format!(
                        "- **{}** - Step \"{}\" timed out after {} minutes",
                        chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                        step_display,
                        self.config.agents.step_timeout / 60
                    ))?;
                }
                SyncAction::UpdatedStatus(status) => {
                    state.update_agent_status(&agent_id, status, None)?;
                }
                _ => {}
            }

            return Ok(action);
        }

        Ok(SyncAction::NoChange)
    }

    /// Get time until next scheduled sync
    pub fn time_until_next_sync(&self) -> Duration {
        let elapsed = self.last_sync.elapsed();
        if elapsed >= self.sync_interval {
            Duration::ZERO
        } else {
            self.sync_interval.saturating_sub(elapsed)
        }
    }

    /// Check for detach signals and process agent returns from awaiting state
    /// Returns a list of agent IDs that have resumed
    pub fn check_detach_signals(&self, state: &mut State) -> Result<Vec<String>> {
        use sha2::{Digest, Sha256};
        use std::fs;
        use std::path::Path;

        let mut resumed_agents = Vec::new();

        // Get agents in awaiting_input status
        let awaiting_agents: Vec<_> = state
            .agents
            .iter()
            .filter(|a| a.status == "awaiting_input")
            .cloned()
            .collect();

        for agent in awaiting_agents {
            let Some(ref session_name) = agent.session_name else {
                continue;
            };

            // Check for signal file
            let signal_file = format!("/tmp/operator-detach-{session_name}.signal");
            let signal_path = Path::new(&signal_file);

            if signal_path.exists() {
                // Signal found - user has detached from session
                let _ = fs::remove_file(signal_path);

                // Handle based on review state
                match agent.review_state.as_deref() {
                    Some("pending_plan" | "pending_visual" | "pending_proof") => {
                        // Review was approved via TUI (signal file written by approval handler)
                        // Resume without needing content change
                        state.update_agent_status(&agent.id, "running", None)?;
                        state.clear_review_state(&agent.id)?;
                        resumed_agents.push(agent.id.clone());

                        tracing::info!(
                            agent_id = %agent.id,
                            session = %session_name,
                            review_state = ?agent.review_state,
                            "Agent resumed from review - approved"
                        );
                    }
                    Some("pending_pr_merge" | "pending_pr_creation") => {
                        // PR review doesn't use signal files for resume
                        // PR merge is detected by PrMonitorService
                        // Skip - signal file may have been created accidentally
                        tracing::debug!(
                            agent_id = %agent.id,
                            "Ignoring signal file for PR review - resume via PR monitor"
                        );
                    }
                    _ => {
                        // Standard awaiting_input (no review state or unknown state)
                        // Original behavior: check content change
                        if let Ok(content) = self.tmux.capture_pane(session_name, false) {
                            let mut hasher = Sha256::new();
                            hasher.update(content.as_bytes());
                            let new_hash = crate::agents::hex_encode(&hasher.finalize());

                            // Compare with stored hash
                            let content_changed = agent.content_hash.as_ref() != Some(&new_hash);

                            if content_changed {
                                // Content changed - LLM is working, resume agent
                                state.update_agent_status(&agent.id, "running", None)?;
                                state.update_agent_content_hash(&agent.id, &new_hash)?;
                                resumed_agents.push(agent.id.clone());

                                tracing::info!(
                                    agent_id = %agent.id,
                                    session = %session_name,
                                    "Agent resumed from awaiting state - content changed"
                                );
                            }
                        }
                    }
                }
            }
        }

        Ok(resumed_agents)
    }

    /// Handle a PR that has been merged - cleanup worktree and complete ticket
    ///
    /// This should be called when `PrStatusEvent::Merged` is received.
    ///
    /// # Arguments
    /// * `ticket` - The ticket whose PR was merged
    /// * `agent` - The agent state for this ticket
    /// * `cleanup_script` - Optional cleanup script from `ProjectRepo`
    ///
    /// # Cleanup steps:
    /// 1. Run `cleanup_script` in worktree (if provided)
    /// 2. Remove worktree via `WorktreeManager`
    /// 3. Delete local branch (remote branch already merged)
    pub async fn handle_pr_merged(
        &self,
        ticket: &Ticket,
        agent: &AgentState,
        cleanup_script: Option<&str>,
    ) -> Result<()> {
        let worktree_path = if let Some(path) = &agent.worktree_path {
            PathBuf::from(path)
        } else {
            tracing::debug!(
                ticket_id = %ticket.id,
                "No worktree to cleanup for merged PR"
            );
            return Ok(());
        };

        if !worktree_path.exists() {
            tracing::warn!(
                ticket_id = %ticket.id,
                worktree = %worktree_path.display(),
                "Worktree path doesn't exist, skipping cleanup"
            );
            return Ok(());
        }

        // Get the main repo path (worktree's parent repo)
        let repo_path = self.get_main_repo_path(ticket)?;

        tracing::info!(
            ticket_id = %ticket.id,
            worktree = %worktree_path.display(),
            "Cleaning up worktree for merged PR"
        );

        // Perform cleanup:
        // - Run cleanup_script (if any)
        // - Remove worktree
        // - Delete local branch (prune_branch=true)
        // - Don't delete remote branch (already merged)
        cleanup_ticket_worktree(
            &self.config,
            &worktree_path,
            &repo_path,
            cleanup_script,
            true,  // prune_branch - delete local branch
            false, // delete_remote_branch - already merged
        )
        .await
        .context("Failed to cleanup worktree for merged PR")?;

        tracing::info!(
            ticket_id = %ticket.id,
            "Worktree cleanup complete for merged PR"
        );

        Ok(())
    }

    /// Get the main repository path for a ticket's project
    fn get_main_repo_path(&self, ticket: &Ticket) -> Result<PathBuf> {
        let projects_root = self.config.projects_path();
        let project_path = if ticket.project == "global" {
            projects_root
        } else {
            projects_root.join(&ticket.project)
        };

        if !project_path.exists() {
            anyhow::bail!("Project path does not exist: {}", project_path.display());
        }

        Ok(project_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agents::tmux::MockTmuxClient;
    use crate::config::PathsConfig;
    use tempfile::TempDir;

    fn make_test_config(temp_dir: &TempDir) -> Config {
        let projects_path = temp_dir.path().join("projects");
        let tickets_path = temp_dir.path().join("tickets");
        let state_path = temp_dir.path().join("state");
        std::fs::create_dir_all(&projects_path).unwrap();
        std::fs::create_dir_all(tickets_path.join("queue")).unwrap();
        std::fs::create_dir_all(tickets_path.join("in-progress")).unwrap();
        std::fs::create_dir_all(tickets_path.join("completed")).unwrap();
        std::fs::create_dir_all(&state_path).unwrap();

        let mut config = Config {
            paths: PathsConfig {
                tickets: tickets_path.to_string_lossy().to_string(),
                projects: projects_path.to_string_lossy().to_string(),
                state: state_path.to_string_lossy().to_string(),
                worktrees: state_path.join("worktrees").to_string_lossy().to_string(),
            },
            ..Default::default()
        };
        config.agents.sync_interval = 1;
        config.agents.step_timeout = 60;
        config
    }

    #[test]
    fn test_should_sync_initially() {
        let temp_dir = TempDir::new().unwrap();
        let config = make_test_config(&temp_dir);
        let mock = Arc::new(MockTmuxClient::new());

        let sync = TicketSessionSync::new(&config, mock);
        // Should be ready to sync immediately after creation
        assert!(sync.should_sync());
    }

    #[test]
    fn test_determine_action_no_change() {
        let temp_dir = TempDir::new().unwrap();
        let config = make_test_config(&temp_dir);
        let mock = Arc::new(MockTmuxClient::new());

        let sync = TicketSessionSync::new(&config, mock);

        // Empty health result means no action needed
        let health = HealthCheckResult::default();

        // Create a minimal ticket for testing
        let ticket = Ticket {
            filename: "test.md".to_string(),
            filepath: "/tmp/test.md".to_string(),
            timestamp: "20241221-1430".to_string(),
            ticket_type: "FEAT".to_string(),
            project: "test".to_string(),
            id: "FEAT-123".to_string(),
            summary: "Test ticket".to_string(),
            priority: "P2-medium".to_string(),
            status: "running".to_string(),
            step: "plan".to_string(),
            content: "# Test".to_string(),
            sessions: std::collections::HashMap::new(),
            step_delegators: std::collections::HashMap::new(),
            llm_task: crate::queue::LlmTask::default(),
            worktree_path: None,
            branch: None,
            external_id: None,
            external_url: None,
            external_provider: None,
            collection: None,
        };

        let action = sync.determine_action(&ticket, "op-FEAT-123", &health);
        assert_eq!(action, SyncAction::NoChange);
    }

    #[test]
    fn test_determine_action_awaiting() {
        let temp_dir = TempDir::new().unwrap();
        let config = make_test_config(&temp_dir);
        let mock = Arc::new(MockTmuxClient::new());

        let sync = TicketSessionSync::new(&config, mock);

        let mut health = HealthCheckResult::default();
        health.awaiting_input.push("op-FEAT-123".to_string());

        let ticket = Ticket {
            filename: "test.md".to_string(),
            filepath: "/tmp/test.md".to_string(),
            timestamp: "20241221-1430".to_string(),
            ticket_type: "FEAT".to_string(),
            project: "test".to_string(),
            id: "FEAT-123".to_string(),
            summary: "Test ticket".to_string(),
            priority: "P2-medium".to_string(),
            status: "running".to_string(),
            step: "plan".to_string(),
            content: "# Test".to_string(),
            sessions: std::collections::HashMap::new(),
            step_delegators: std::collections::HashMap::new(),
            llm_task: crate::queue::LlmTask::default(),
            worktree_path: None,
            branch: None,
            external_id: None,
            external_url: None,
            external_provider: None,
            collection: None,
        };

        let action = sync.determine_action(&ticket, "op-FEAT-123", &health);
        assert_eq!(action, SyncAction::MovedToAwaiting);
    }

    #[test]
    fn test_determine_action_timeout() {
        let temp_dir = TempDir::new().unwrap();
        let config = make_test_config(&temp_dir);
        let mock = Arc::new(MockTmuxClient::new());

        let sync = TicketSessionSync::new(&config, mock);

        let mut health = HealthCheckResult::default();
        health.timed_out.push("op-FEAT-456".to_string());

        let ticket = Ticket {
            filename: "test.md".to_string(),
            filepath: "/tmp/test.md".to_string(),
            timestamp: "20241221-1430".to_string(),
            ticket_type: "FEAT".to_string(),
            project: "test".to_string(),
            id: "FEAT-456".to_string(),
            summary: "Test ticket".to_string(),
            priority: "P2-medium".to_string(),
            status: "running".to_string(),
            step: "implement".to_string(),
            content: "# Test".to_string(),
            sessions: std::collections::HashMap::new(),
            step_delegators: std::collections::HashMap::new(),
            llm_task: crate::queue::LlmTask::default(),
            worktree_path: None,
            branch: None,
            external_id: None,
            external_url: None,
            external_provider: None,
            collection: None,
        };

        let action = sync.determine_action(&ticket, "op-FEAT-456", &health);
        assert_eq!(action, SyncAction::TimedOut);
    }

    #[test]
    fn test_timeout_takes_priority_over_awaiting() {
        let temp_dir = TempDir::new().unwrap();
        let config = make_test_config(&temp_dir);
        let mock = Arc::new(MockTmuxClient::new());

        let sync = TicketSessionSync::new(&config, mock);

        let mut health = HealthCheckResult::default();
        // Both timeout and awaiting for same session
        health.timed_out.push("op-FEAT-789".to_string());
        health.awaiting_input.push("op-FEAT-789".to_string());

        let ticket = Ticket {
            filename: "test.md".to_string(),
            filepath: "/tmp/test.md".to_string(),
            timestamp: "20241221-1430".to_string(),
            ticket_type: "FEAT".to_string(),
            project: "test".to_string(),
            id: "FEAT-789".to_string(),
            summary: "Test ticket".to_string(),
            priority: "P2-medium".to_string(),
            status: "running".to_string(),
            step: "test".to_string(),
            content: "# Test".to_string(),
            sessions: std::collections::HashMap::new(),
            step_delegators: std::collections::HashMap::new(),
            llm_task: crate::queue::LlmTask::default(),
            worktree_path: None,
            branch: None,
            external_id: None,
            external_url: None,
            external_provider: None,
            collection: None,
        };

        let action = sync.determine_action(&ticket, "op-FEAT-789", &health);
        // Timeout should take priority
        assert_eq!(action, SyncAction::TimedOut);
    }

    #[test]
    fn test_determine_action_awaiting_with_artifact_ready() {
        let temp_dir = TempDir::new().unwrap();
        let config = make_test_config(&temp_dir);
        let mock = Arc::new(MockTmuxClient::new());

        let sync = TicketSessionSync::new(&config, mock);

        let mut health = HealthCheckResult::default();
        health.awaiting_input.push("op-FEAT-123".to_string());
        health.artifact_ready.push("op-FEAT-123".to_string());

        let ticket = Ticket {
            filename: "test.md".to_string(),
            filepath: "/tmp/test.md".to_string(),
            timestamp: "20241221-1430".to_string(),
            ticket_type: "FEAT".to_string(),
            project: "test".to_string(),
            id: "FEAT-123".to_string(),
            summary: "Test ticket".to_string(),
            priority: "P2-medium".to_string(),
            status: "running".to_string(),
            step: "plan".to_string(),
            content: "# Test".to_string(),
            sessions: std::collections::HashMap::new(),
            step_delegators: std::collections::HashMap::new(),
            llm_task: crate::queue::LlmTask::default(),
            worktree_path: None,
            branch: None,
            external_id: None,
            external_url: None,
            external_provider: None,
            collection: None,
        };

        let action = sync.determine_action(&ticket, "op-FEAT-123", &health);
        assert_eq!(action, SyncAction::StepCompleted);
    }

    #[test]
    fn test_determine_action_awaiting_without_artifact_ready() {
        let temp_dir = TempDir::new().unwrap();
        let config = make_test_config(&temp_dir);
        let mock = Arc::new(MockTmuxClient::new());

        let sync = TicketSessionSync::new(&config, mock);

        let mut health = HealthCheckResult::default();
        health.awaiting_input.push("op-FEAT-123".to_string());
        // artifact_ready is empty - agent is idle but no artifacts found

        let ticket = Ticket {
            filename: "test.md".to_string(),
            filepath: "/tmp/test.md".to_string(),
            timestamp: "20241221-1430".to_string(),
            ticket_type: "FEAT".to_string(),
            project: "test".to_string(),
            id: "FEAT-123".to_string(),
            summary: "Test ticket".to_string(),
            priority: "P2-medium".to_string(),
            status: "running".to_string(),
            step: "plan".to_string(),
            content: "# Test".to_string(),
            sessions: std::collections::HashMap::new(),
            step_delegators: std::collections::HashMap::new(),
            llm_task: crate::queue::LlmTask::default(),
            worktree_path: None,
            branch: None,
            external_id: None,
            external_url: None,
            external_provider: None,
            collection: None,
        };

        let action = sync.determine_action(&ticket, "op-FEAT-123", &health);
        assert_eq!(action, SyncAction::MovedToAwaiting);
    }

    #[test]
    fn test_check_detach_signals_no_awaiting_agents() {
        let temp_dir = TempDir::new().unwrap();
        let config = make_test_config(&temp_dir);
        let mock = Arc::new(MockTmuxClient::new());

        let sync = TicketSessionSync::new(&config, mock);
        let mut state = State::load(&config).unwrap();

        // No agents at all
        let resumed = sync.check_detach_signals(&mut state).unwrap();
        assert!(resumed.is_empty());
    }

    #[test]
    fn test_check_detach_signals_no_signal_file() {
        let temp_dir = TempDir::new().unwrap();
        let config = make_test_config(&temp_dir);
        let mock = Arc::new(MockTmuxClient::new());

        let sync = TicketSessionSync::new(&config, mock);
        let mut state = State::load(&config).unwrap();

        // Add an awaiting agent
        let agent_id = state
            .add_agent(
                "FEAT-123".to_string(),
                "FEAT".to_string(),
                "test".to_string(),
                false,
            )
            .unwrap();
        state
            .update_agent_session(&agent_id, "op-FEAT-123")
            .unwrap();
        state
            .update_agent_status(&agent_id, "awaiting_input", None)
            .unwrap();
        state
            .update_agent_content_hash(&agent_id, "old-hash")
            .unwrap();

        // No signal file exists
        let resumed = sync.check_detach_signals(&mut state).unwrap();
        assert!(resumed.is_empty());
        // Agent should still be awaiting
        let agent = state.agent_by_ticket("FEAT-123").unwrap();
        assert_eq!(agent.status, "awaiting_input");
    }

    #[test]
    fn test_check_detach_signals_content_changed() {
        use sha2::{Digest, Sha256};
        use std::fs;

        let temp_dir = TempDir::new().unwrap();
        let config = make_test_config(&temp_dir);
        let mock = Arc::new(MockTmuxClient::new());

        // Create session with content
        mock.create_session("op-FEAT-detach", "/tmp").unwrap();
        mock.send_keys("op-FEAT-detach", "New content from LLM", false)
            .unwrap();

        let sync = TicketSessionSync::new(&config, Arc::clone(&mock) as Arc<dyn TmuxClient>);
        let mut state = State::load(&config).unwrap();

        // Calculate hash of OLD content (different from what's in the session)
        let mut old_hasher = Sha256::new();
        old_hasher.update(b"Old content");
        let old_hash = crate::agents::hex_encode(&old_hasher.finalize());

        // Add an awaiting agent with old content hash
        let agent_id = state
            .add_agent(
                "FEAT-detach".to_string(),
                "FEAT".to_string(),
                "test".to_string(),
                false,
            )
            .unwrap();
        state
            .update_agent_session(&agent_id, "op-FEAT-detach")
            .unwrap();
        state
            .update_agent_status(&agent_id, "awaiting_input", None)
            .unwrap();
        state
            .update_agent_content_hash(&agent_id, &old_hash)
            .unwrap();

        // Create signal file
        let signal_file = "/tmp/operator-detach-op-FEAT-detach.signal";
        fs::write(signal_file, "").unwrap();

        // Run check
        let resumed = sync.check_detach_signals(&mut state).unwrap();

        // Clean up signal file if it still exists
        let _ = fs::remove_file(signal_file);

        // Agent should have resumed
        assert_eq!(resumed.len(), 1);
        assert_eq!(resumed[0], agent_id);
        let agent = state.agent_by_ticket("FEAT-detach").unwrap();
        assert_eq!(agent.status, "running");
    }

    #[test]
    fn test_check_detach_signals_content_unchanged() {
        use sha2::{Digest, Sha256};
        use std::fs;

        let temp_dir = TempDir::new().unwrap();
        let config = make_test_config(&temp_dir);
        let mock = Arc::new(MockTmuxClient::new());

        // Create session with specific content
        mock.create_session("op-FEAT-unchanged", "/tmp").unwrap();
        mock.send_keys("op-FEAT-unchanged", "Same content", false)
            .unwrap();

        // Get the content hash that will match
        let content = mock.capture_pane("op-FEAT-unchanged", false).unwrap();
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        let same_hash = crate::agents::hex_encode(&hasher.finalize());

        let sync = TicketSessionSync::new(&config, Arc::clone(&mock) as Arc<dyn TmuxClient>);
        let mut state = State::load(&config).unwrap();

        // Add an awaiting agent with SAME content hash
        let agent_id = state
            .add_agent(
                "FEAT-unchanged".to_string(),
                "FEAT".to_string(),
                "test".to_string(),
                false,
            )
            .unwrap();
        state
            .update_agent_session(&agent_id, "op-FEAT-unchanged")
            .unwrap();
        state
            .update_agent_status(&agent_id, "awaiting_input", None)
            .unwrap();
        state
            .update_agent_content_hash(&agent_id, &same_hash)
            .unwrap();

        // Create signal file
        let signal_file = "/tmp/operator-detach-op-FEAT-unchanged.signal";
        fs::write(signal_file, "").unwrap();

        // Run check
        let resumed = sync.check_detach_signals(&mut state).unwrap();

        // Clean up signal file if it still exists
        let _ = fs::remove_file(signal_file);

        // Agent should NOT have resumed (content unchanged)
        assert!(resumed.is_empty());
        let agent = state.agent_by_ticket("FEAT-unchanged").unwrap();
        assert_eq!(agent.status, "awaiting_input");
    }

    #[test]
    fn test_sync_action_resumed_from_awaiting_variant() {
        // Test that the ResumedFromAwaiting variant exists and works
        let action = SyncAction::ResumedFromAwaiting;
        assert_eq!(action, SyncAction::ResumedFromAwaiting);

        // Test that it's different from other actions
        assert_ne!(action, SyncAction::NoChange);
        assert_ne!(action, SyncAction::MovedToAwaiting);
    }

    // ── multi-agent judge phase ─────────────────────────────────────

    mod judge_phase {
        use super::*;
        use crate::agents::judge::ConfiguredJudge;
        use crate::llm::native::fake::FakeNativeLlm;
        use crate::llm::native::{JudgeOutcome, NativeLlmError};
        use crate::state::MultiAgentPhase;

        struct Fixture {
            _dir: TempDir,
            config: Config,
            worktree: String,
            ticket: Ticket,
            state: State,
            group_id: String,
        }

        fn multi_model_schema() -> StepSchema {
            serde_json::from_value(serde_json::json!({
                "name": "review",
                "prompt": "Review it",
                "outputs": [],
                "type": "multi_model",
                "multi_model_config": {
                    "delegators": ["a", "b"],
                    "voting_strategy": "majority",
                    "voting_mode": "single_judge"
                }
            }))
            .unwrap()
        }

        fn fixture() -> Fixture {
            let dir = TempDir::new().unwrap();
            let config = make_test_config(&dir);
            let worktree = dir.path().join("wt").to_string_lossy().to_string();
            std::fs::create_dir_all(&worktree).unwrap();
            let ticket_path = dir.path().join("FEAT-1.md");
            std::fs::write(&ticket_path, "# FEAT-1").unwrap();
            let ticket = Ticket {
                filename: "FEAT-1.md".to_string(),
                filepath: ticket_path.to_string_lossy().to_string(),
                timestamp: "20241221-1430".to_string(),
                ticket_type: "FEAT".to_string(),
                project: "test".to_string(),
                id: "FEAT-1".to_string(),
                summary: "t".to_string(),
                priority: "P2-medium".to_string(),
                status: "running".to_string(),
                step: "review".to_string(),
                content: "# FEAT-1".to_string(),
                sessions: std::collections::HashMap::new(),
                step_delegators: std::collections::HashMap::new(),
                llm_task: crate::queue::LlmTask::default(),
                worktree_path: Some(worktree.clone()),
                branch: None,
                external_id: None,
                external_url: None,
                external_provider: None,
                collection: None,
            };
            let mut state = State::load(&config).unwrap();
            let group_id = state
                .create_multi_agent_group("FEAT-1", "review", "multi_model", Vec::new())
                .unwrap();
            let group = state
                .multi_agent_groups
                .iter_mut()
                .find(|g| g.group_id == group_id)
                .unwrap();
            group
                .individual_outputs
                .insert("a".to_string(), serde_json::json!("answer A"));
            group
                .individual_outputs
                .insert("b".to_string(), serde_json::json!("answer B"));
            state.save().unwrap();
            Fixture {
                _dir: dir,
                config,
                worktree,
                ticket,
                state,
                group_id,
            }
        }

        fn sync_with(config: &Config, factory: JudgeFactory) -> TicketSessionSync {
            TicketSessionSync::new(config, Arc::new(MockTmuxClient::new()))
                .with_judge_factory(factory)
        }

        fn judge_returning(result: Result<JudgeVerdict, NativeLlmError>) -> JudgeFactory {
            Arc::new(move |_| {
                Ok(Some(ConfiguredJudge {
                    llm: Arc::new(FakeNativeLlm(result.clone())),
                    timeout_secs: 5,
                }))
            })
        }

        fn group(f: &Fixture) -> MultiAgentGroup {
            f.state
                .multi_agent_groups
                .iter()
                .find(|g| g.group_id == f.group_id)
                .cloned()
                .unwrap()
        }

        fn artifact(f: &Fixture) -> serde_json::Value {
            let path = std::path::Path::new(&f.worktree).join(".tickets/steps/review.output.json");
            serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
        }

        fn verdict(i: usize) -> JudgeVerdict {
            JudgeVerdict {
                winner_index: i,
                rationale: "more thorough".to_string(),
            }
        }

        #[test]
        fn no_judge_configured_finalizes_deterministically() {
            let mut f = fixture();
            let mut sync = sync_with(&f.config, Arc::new(|_| Ok(None)));
            let schema = multi_model_schema();
            let mut result = SyncResult::default();
            let g = group(&f);

            sync.aggregate_or_start_judging(
                &mut f.ticket,
                &mut f.state,
                &g,
                Some(&schema),
                &mut result,
            )
            .unwrap();

            assert_eq!(artifact(&f)["winner_delegator"], "a");
            assert!(artifact(&f).get("judge").is_none());
            assert!(f.state.multi_agent_groups.is_empty());
            assert_eq!(result.completed, vec!["FEAT-1".to_string()]);
        }

        #[test]
        fn unusable_judge_finalizes_with_history_note() {
            let mut f = fixture();
            let mut sync = sync_with(
                &f.config,
                Arc::new(|_| Err(NativeLlmError::Config("no key".to_string()))),
            );
            let schema = multi_model_schema();
            let mut result = SyncResult::default();
            let g = group(&f);

            sync.aggregate_or_start_judging(
                &mut f.ticket,
                &mut f.state,
                &g,
                Some(&schema),
                &mut result,
            )
            .unwrap();

            assert_eq!(artifact(&f)["winner_delegator"], "a");
            assert!(f.ticket.content.contains("Judge unavailable"));
            assert!(f.state.multi_agent_groups.is_empty());
        }

        #[tokio::test]
        async fn judge_verdict_is_applied_on_next_tick() {
            let mut f = fixture();
            let mut sync = sync_with(&f.config, judge_returning(Ok(verdict(1))));
            let schema = multi_model_schema();
            let mut result = SyncResult::default();
            let g = group(&f);

            sync.aggregate_or_start_judging(
                &mut f.ticket,
                &mut f.state,
                &g,
                Some(&schema),
                &mut result,
            )
            .unwrap();
            let judging = group(&f);
            assert_eq!(judging.phase, MultiAgentPhase::Voting);
            let attempt = judging.judge_attempt.clone().unwrap();

            // Let the spawned judge task write its outcome.
            for _ in 0..100 {
                if StepManager::read_judge_outcome(&f.ticket, "review", &attempt.attempt_id)
                    .is_some()
                {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }

            sync.sync_judging_group(
                &mut f.ticket,
                &mut f.state,
                &judging,
                Some(&schema),
                &mut result,
            )
            .unwrap();

            let out = artifact(&f);
            assert_eq!(out["winner_delegator"], "b");
            assert_eq!(out["value"], "answer B");
            assert_eq!(out["judge"]["rationale"], "more thorough");
            assert!(f.ticket.content.contains("Judge selected candidate 1"));
            assert!(f.state.multi_agent_groups.is_empty());
        }

        #[test]
        fn judging_group_waits_for_outcome_and_ignores_stale_attempts() {
            let mut f = fixture();
            let mut sync = sync_with(&f.config, Arc::new(|_| Ok(None)));
            let schema = multi_model_schema();
            f.state.begin_judging(&f.group_id, 60).unwrap();
            StepManager::write_judge_outcome(
                &f.worktree,
                "review",
                "some-older-attempt",
                &JudgeOutcome::Verdict(verdict(1)),
            )
            .unwrap();
            let mut result = SyncResult::default();
            let g = group(&f);

            sync.sync_judging_group(&mut f.ticket, &mut f.state, &g, Some(&schema), &mut result)
                .unwrap();

            assert_eq!(group(&f).phase, MultiAgentPhase::Voting);
            assert!(result.completed.is_empty());
        }

        #[test]
        fn judge_failure_falls_back_with_history_note() {
            let mut f = fixture();
            let mut sync = sync_with(&f.config, Arc::new(|_| Ok(None)));
            let schema = multi_model_schema();
            let attempt = f.state.begin_judging(&f.group_id, 60).unwrap();
            StepManager::write_judge_outcome(
                &f.worktree,
                "review",
                &attempt.attempt_id,
                &JudgeOutcome::Failed {
                    reason: "HTTP 401".to_string(),
                },
            )
            .unwrap();
            let mut result = SyncResult::default();
            let g = group(&f);

            sync.sync_judging_group(&mut f.ticket, &mut f.state, &g, Some(&schema), &mut result)
                .unwrap();

            assert_eq!(artifact(&f)["winner_delegator"], "a");
            assert!(f.ticket.content.contains("fell back"));
            assert!(f.ticket.content.contains("HTTP 401"));
            assert!(f.state.multi_agent_groups.is_empty());
        }

        #[test]
        fn orphaned_attempt_past_deadline_falls_back() {
            // e.g. the daemon restarted while judging: nothing will write the file
            let mut f = fixture();
            let mut sync = sync_with(&f.config, Arc::new(|_| Ok(None)));
            let schema = multi_model_schema();
            f.state.begin_judging(&f.group_id, 60).unwrap();
            let g = f
                .state
                .multi_agent_groups
                .iter_mut()
                .find(|g| g.group_id == f.group_id)
                .unwrap();
            g.judge_attempt.as_mut().unwrap().started_at =
                chrono::Utc::now() - chrono::Duration::hours(1);
            let g = g.clone();
            let mut result = SyncResult::default();

            sync.sync_judging_group(&mut f.ticket, &mut f.state, &g, Some(&schema), &mut result)
                .unwrap();

            assert_eq!(artifact(&f)["winner_delegator"], "a");
            assert!(f.ticket.content.contains("no verdict by deadline"));
            assert!(f.state.multi_agent_groups.is_empty());
        }
    }
}

use anyhow::Result;

use crate::agents::PrWorkflow;
use crate::notifications::NotificationEvent;
use crate::queue::Queue;
use crate::services::{PrStatusEvent, TrackedPr};
use crate::state::State;

use super::App;

impl App {
    /// Handle PR status events from the background monitor (non-blocking)
    pub(super) async fn handle_pr_events(&mut self) -> Result<()> {
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
                                Some(format!("PR #{pr_number} merged")),
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
                    let key = format!("{ticket_id}#{pr_number}");
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
                    let key = format!("{ticket_id}#{pr_number}");
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
    #[allow(clippy::cognitive_complexity)] // PR workflow has inherent branching complexity
    pub(super) async fn process_pending_pr_creations(&mut self) -> Result<()> {
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
            let ticket = if let Some(t) = queue.get_in_progress_ticket(&agent.ticket_id)? {
                t
            } else {
                tracing::warn!(
                    agent_id = %agent.id,
                    ticket_id = %agent.ticket_id,
                    "Ticket not found for pending PR creation"
                );
                continue;
            };

            // Get the worktree path
            let worktree_path = if let Some(path) = &agent.worktree_path {
                std::path::PathBuf::from(path)
            } else {
                tracing::warn!(
                    agent_id = %agent.id,
                    "No worktree path for PR creation"
                );
                continue;
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
                        Some(format!("PR creation failed: {e}")),
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
                            error: format!("Failed to create PR: {e}"),
                        })
                        .await;
                }
            }
        }

        Ok(())
    }
}

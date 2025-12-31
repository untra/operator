//! PR Monitor Service - Background polling for PR status changes.
//!
//! Supports multiple Git providers through the PrService trait:
//! - Polls every 60 seconds for active PRs
//! - Detects state changes (merged, changes requested, approved)
//! - Triggers callbacks on status changes

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, error, info, instrument, warn};

use crate::api::{GitHubService, PrService};
use crate::types::pr::{GitProvider, PrState, RepoInfo};

/// Default poll interval (60 seconds, matching vibe-kanban)
const DEFAULT_POLL_INTERVAL: Duration = Duration::from_secs(60);

/// Tracked PR information
#[derive(Debug, Clone)]
pub struct TrackedPr {
    /// Repository info (provider-agnostic)
    pub repo_info: RepoInfo,
    /// PR number
    pub pr_number: i64,
    /// Last known state
    pub last_state: PrState,
    /// Associated ticket ID (for routing callbacks)
    pub ticket_id: String,
    /// Whether PR is a draft
    pub is_draft: bool,
    /// Merge commit SHA (if merged)
    pub merge_commit_sha: Option<String>,
}

/// Event emitted when a PR status changes
#[derive(Debug, Clone)]
pub enum PrStatusEvent {
    /// PR was merged
    Merged {
        ticket_id: String,
        pr_number: i64,
        merge_commit_sha: String,
    },
    /// PR was closed without merge
    Closed { ticket_id: String, pr_number: i64 },
    /// PR received approval
    #[allow(dead_code)] // Emitted when review detection is added
    Approved { ticket_id: String, pr_number: i64 },
    /// PR had changes requested
    #[allow(dead_code)] // Emitted when review detection is added
    ChangesRequested { ticket_id: String, pr_number: i64 },
    /// PR is ready to merge (approved + checks pass)
    ReadyToMerge { ticket_id: String, pr_number: i64 },
    /// PR was converted from draft to ready
    ReadyForReview { ticket_id: String, pr_number: i64 },
}

/// Background service that monitors PR status
pub struct PrMonitorService {
    /// PR service for API calls (provider-agnostic)
    pr_service: Arc<dyn PrService>,
    /// Poll interval
    poll_interval: Duration,
    /// Currently tracked PRs (keyed by "owner/repo#number")
    tracked_prs: Arc<RwLock<HashMap<String, TrackedPr>>>,
    /// Channel to send status events
    event_tx: mpsc::UnboundedSender<PrStatusEvent>,
    /// Shutdown signal
    shutdown_rx: Option<mpsc::Receiver<()>>,
}

impl PrMonitorService {
    /// Create a new PR monitor service with default GitHub provider
    pub fn new(event_tx: mpsc::UnboundedSender<PrStatusEvent>) -> Self {
        Self::with_service(Arc::new(GitHubService::new()), event_tx)
    }

    /// Create a new PR monitor service with a custom provider
    pub fn with_service(
        pr_service: Arc<dyn PrService>,
        event_tx: mpsc::UnboundedSender<PrStatusEvent>,
    ) -> Self {
        Self {
            pr_service,
            poll_interval: DEFAULT_POLL_INTERVAL,
            tracked_prs: Arc::new(RwLock::new(HashMap::new())),
            event_tx,
            shutdown_rx: None,
        }
    }

    /// Create with custom poll interval
    pub fn with_poll_interval(mut self, interval: Duration) -> Self {
        self.poll_interval = interval;
        self
    }

    /// Set shutdown receiver
    pub fn with_shutdown(mut self, rx: mpsc::Receiver<()>) -> Self {
        self.shutdown_rx = Some(rx);
        self
    }

    /// Get a clone of the tracked PRs map for external access
    pub fn tracked_prs(&self) -> Arc<RwLock<HashMap<String, TrackedPr>>> {
        self.tracked_prs.clone()
    }

    /// Generate a key for a tracked PR
    fn pr_key(repo_info: &RepoInfo, pr_number: i64) -> String {
        format!("{}#{}", repo_info.full_name(), pr_number)
    }

    /// Start tracking a PR
    #[instrument(skip(self))]
    pub async fn track_pr(
        &self,
        repo_info: RepoInfo,
        pr_number: i64,
        ticket_id: String,
    ) -> Result<()> {
        // Fetch current state
        let pr = self
            .pr_service
            .get_pr(&repo_info, pr_number)
            .await
            .context("Failed to fetch initial PR state")?;

        let tracked = TrackedPr {
            repo_info: repo_info.clone(),
            pr_number,
            last_state: pr.state,
            ticket_id,
            is_draft: pr.is_draft,
            merge_commit_sha: pr.merge_commit_sha,
        };

        let key = Self::pr_key(&repo_info, pr_number);
        let mut tracked_prs = self.tracked_prs.write().await;
        tracked_prs.insert(key.clone(), tracked);

        info!("Now tracking PR {}", key);
        Ok(())
    }

    /// Stop tracking a PR
    #[instrument(skip(self))]
    pub async fn untrack_pr(&self, repo_info: &RepoInfo, pr_number: i64) {
        let key = Self::pr_key(repo_info, pr_number);
        let mut tracked_prs = self.tracked_prs.write().await;
        if tracked_prs.remove(&key).is_some() {
            info!("Stopped tracking PR {}", key);
        }
    }

    /// Run the monitor loop
    #[instrument(skip(self))]
    pub async fn run(&mut self) -> Result<()> {
        info!(
            "PR monitor service started, poll interval: {:?}",
            self.poll_interval
        );

        let mut interval = tokio::time::interval(self.poll_interval);

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    if let Err(e) = self.poll_all_prs().await {
                        error!("Error polling PRs: {}", e);
                    }
                }
                _ = async {
                    if let Some(ref mut rx) = self.shutdown_rx {
                        rx.recv().await
                    } else {
                        std::future::pending::<Option<()>>().await
                    }
                } => {
                    info!("PR monitor service shutting down");
                    break;
                }
            }
        }

        Ok(())
    }

    /// Poll all tracked PRs for status changes
    async fn poll_all_prs(&self) -> Result<()> {
        let tracked_prs = self.tracked_prs.read().await;
        let prs: Vec<TrackedPr> = tracked_prs.values().cloned().collect();
        drop(tracked_prs); // Release read lock

        debug!("Polling {} tracked PRs", prs.len());

        for tracked in prs {
            if let Err(e) = self.poll_single_pr(&tracked).await {
                warn!(
                    "Error polling PR {}#{}: {}",
                    tracked.repo_info.full_name(),
                    tracked.pr_number,
                    e
                );
            }
        }

        Ok(())
    }

    /// Poll a single PR and handle status changes
    async fn poll_single_pr(&self, tracked: &TrackedPr) -> Result<()> {
        let pr = self
            .pr_service
            .get_pr(&tracked.repo_info, tracked.pr_number)
            .await
            .context("Failed to fetch PR")?;

        // Check for state changes
        let mut events = Vec::new();

        // Check for merge
        if pr.state == PrState::Merged && tracked.last_state != PrState::Merged {
            events.push(PrStatusEvent::Merged {
                ticket_id: tracked.ticket_id.clone(),
                pr_number: tracked.pr_number,
                merge_commit_sha: pr.merge_commit_sha.clone().unwrap_or_default(),
            });
        }

        // Check for close without merge
        if pr.state == PrState::Closed && tracked.last_state == PrState::Open {
            events.push(PrStatusEvent::Closed {
                ticket_id: tracked.ticket_id.clone(),
                pr_number: tracked.pr_number,
            });
        }

        // Check for draft -> ready conversion
        if tracked.is_draft && !pr.is_draft && pr.state == PrState::Open {
            events.push(PrStatusEvent::ReadyForReview {
                ticket_id: tracked.ticket_id.clone(),
                pr_number: tracked.pr_number,
            });
        }

        // Check if ready to merge (only for open, non-draft PRs)
        if pr.state == PrState::Open && !pr.is_draft {
            let ready = self
                .pr_service
                .is_ready_to_merge(&tracked.repo_info, tracked.pr_number)
                .await
                .unwrap_or(false);

            if ready {
                events.push(PrStatusEvent::ReadyToMerge {
                    ticket_id: tracked.ticket_id.clone(),
                    pr_number: tracked.pr_number,
                });
            }
        }

        // Update tracked state
        if pr.state != tracked.last_state || pr.is_draft != tracked.is_draft {
            let key = Self::pr_key(&tracked.repo_info, tracked.pr_number);
            let mut tracked_prs = self.tracked_prs.write().await;
            if let Some(t) = tracked_prs.get_mut(&key) {
                t.last_state = pr.state;
                t.is_draft = pr.is_draft;
                t.merge_commit_sha = pr.merge_commit_sha;
            }
        }

        // Send events
        for event in events {
            debug!("Emitting PR event: {:?}", event);
            if self.event_tx.send(event).is_err() {
                warn!("Event receiver dropped, stopping monitor");
                return Err(anyhow::anyhow!("Event receiver dropped"));
            }
        }

        Ok(())
    }

    /// Get current count of tracked PRs
    #[allow(dead_code)] // Utility method for future use
    pub async fn tracked_count(&self) -> usize {
        self.tracked_prs.read().await.len()
    }

    /// Check if a specific PR is being tracked
    #[allow(dead_code)] // Utility method for future use
    pub async fn is_tracking(&self, repo_info: &RepoInfo, pr_number: i64) -> bool {
        let key = Self::pr_key(repo_info, pr_number);
        self.tracked_prs.read().await.contains_key(&key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pr_key_format() {
        let repo = RepoInfo {
            provider: GitProvider::GitHub,
            owner: "owner".to_string(),
            repo_name: "repo".to_string(),
        };
        assert_eq!(PrMonitorService::pr_key(&repo, 42), "owner/repo#42");
    }

    #[tokio::test]
    async fn test_create_service() {
        let (tx, _rx) = mpsc::unbounded_channel();
        let service = PrMonitorService::new(tx);
        assert_eq!(service.tracked_count().await, 0);
    }

    #[tokio::test]
    async fn test_poll_interval_config() {
        let (tx, _rx) = mpsc::unbounded_channel();
        let service = PrMonitorService::new(tx).with_poll_interval(Duration::from_secs(30));
        assert_eq!(service.poll_interval, Duration::from_secs(30));
    }
}

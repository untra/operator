//! Shutdown recovery reconciliation, run once before the API accepts launches.
//!
//! Operator marks agents on the way down (`ShutdownRecovery`); this decides what
//! those marks mean on the way back up. It deliberately does not probe remote
//! hosts: a Coder workspace or SSH session can outlive Operator while its
//! callback path is down, so unreachable is reported as unresolved rather than
//! guessed either way.

use crate::config::Config;
use crate::state::{ShutdownRecovery, State};

const LOCAL_MESSAGE: &str = "Interrupted by Operator shutdown; retry explicitly";
const REMOTE_MESSAGE: &str = "Remote work is awaiting reconciliation after Operator restart";

/// Statuses that mean an agent still holds work.
fn is_active(status: &str) -> bool {
    matches!(status, "running" | "awaiting_input" | "completing")
}

/// Reconcile shutdown markers. Returns how many agent records changed.
pub fn reconcile(config: &Config) -> anyhow::Result<usize> {
    State::mutate(config, |state| {
        let mut changed = 0;
        for agent in &mut state.agents {
            match agent.shutdown_recovery {
                // Remote work was left running on purpose. Keep it flagged so a
                // relaunch of the same ticket is refused until an operator says
                // whether it survived.
                Some(ShutdownRecovery::RemoteAwaitingReconciliation) => {
                    if is_active(&agent.status) {
                        agent.last_message = Some(REMOTE_MESSAGE.to_string());
                    } else {
                        // It reported a terminal status after the mark was
                        // written, so there is nothing left to reconcile.
                        agent.shutdown_recovery = None;
                    }
                    changed += 1;
                }
                // The local session is gone with the process. Leave it failed
                // and retryable; never auto-relaunch.
                Some(ShutdownRecovery::InterruptedLocal) => {
                    if is_active(&agent.status) {
                        agent.status = "failed".to_string();
                        agent.last_message = Some(LOCAL_MESSAGE.to_string());
                    }
                    changed += 1;
                }
                None => {}
            }
        }
        changed
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config(temp: &tempfile::TempDir) -> Config {
        let mut config = Config::default();
        config.paths.state = temp.path().join("state").display().to_string();
        config.paths.tickets = temp.path().join("tickets").display().to_string();
        config
    }

    fn seed(config: &Config, ticket: &str, launch_mode: Option<&str>) -> String {
        State::mutate(config, |state| {
            state
                .add_agent_with_options(
                    ticket.to_string(),
                    "FEAT".to_string(),
                    "project".to_string(),
                    false,
                    None,
                    launch_mode.map(str::to_string),
                )
                .unwrap()
        })
        .unwrap()
    }

    fn mark(config: &Config, id: &str, recovery: ShutdownRecovery, status: &str) {
        State::mutate(config, |state| {
            let agent = state.agents.iter_mut().find(|a| a.id == id).unwrap();
            agent.shutdown_recovery = Some(recovery);
            agent.status = status.to_string();
        })
        .unwrap();
    }

    fn agent(config: &Config, id: &str) -> crate::state::AgentState {
        State::load(config)
            .unwrap()
            .agents
            .into_iter()
            .find(|a| a.id == id)
            .unwrap()
    }

    /// Regression: the local arm mutated the record without flagging the write,
    /// so the status change was silently discarded.
    #[test]
    fn test_interrupted_local_work_persists_as_failed_and_retryable() {
        let temp = tempfile::TempDir::new().unwrap();
        let config = config(&temp);
        let id = seed(&config, "FEAT-1", None);
        mark(&config, &id, ShutdownRecovery::InterruptedLocal, "running");

        assert_eq!(reconcile(&config).unwrap(), 1);

        let agent = agent(&config, &id);
        assert_eq!(agent.status, "failed");
        assert_eq!(agent.last_message.as_deref(), Some(LOCAL_MESSAGE));
        assert_eq!(
            agent.shutdown_recovery,
            Some(ShutdownRecovery::InterruptedLocal),
            "the marker stays so an explicit retry can clear it"
        );
    }

    #[test]
    fn test_remote_work_stays_unresolved_and_is_never_marked_completed() {
        let temp = tempfile::TempDir::new().unwrap();
        let config = config(&temp);
        let id = seed(&config, "FEAT-2", Some("coder"));
        mark(
            &config,
            &id,
            ShutdownRecovery::RemoteAwaitingReconciliation,
            "running",
        );

        reconcile(&config).unwrap();

        let agent = agent(&config, &id);
        assert_eq!(
            agent.status, "running",
            "surviving remote work must not be failed or completed on our say-so"
        );
        assert_eq!(agent.last_message.as_deref(), Some(REMOTE_MESSAGE));
        assert_eq!(
            agent.shutdown_recovery,
            Some(ShutdownRecovery::RemoteAwaitingReconciliation)
        );
    }

    /// A completion recorded during downtime is authoritative; clear the mark.
    #[test]
    fn test_remote_work_that_completed_during_downtime_clears_its_marker() {
        let temp = tempfile::TempDir::new().unwrap();
        let config = config(&temp);
        let id = seed(&config, "FEAT-3", Some("coder"));
        mark(
            &config,
            &id,
            ShutdownRecovery::RemoteAwaitingReconciliation,
            "completed",
        );

        reconcile(&config).unwrap();

        let agent = agent(&config, &id);
        assert_eq!(agent.status, "completed");
        assert_eq!(agent.shutdown_recovery, None);
    }

    #[test]
    fn test_unmarked_agents_are_left_alone() {
        let temp = tempfile::TempDir::new().unwrap();
        let config = config(&temp);
        let id = seed(&config, "FEAT-4", None);

        assert_eq!(reconcile(&config).unwrap(), 0);
        assert_eq!(agent(&config, &id).shutdown_recovery, None);
    }

    /// State written before the marker existed must still load and reconcile.
    #[test]
    fn test_state_without_the_marker_field_reconciles_cleanly() {
        let temp = tempfile::TempDir::new().unwrap();
        let config = config(&temp);
        let id = seed(&config, "FEAT-5", None);

        let path = std::path::PathBuf::from(&config.paths.state).join("state.json");
        let raw = std::fs::read_to_string(&path).unwrap();
        assert!(
            !raw.contains("shutdown_recovery"),
            "an unmarked agent must not serialize the field at all"
        );
        std::fs::write(&path, raw).unwrap();

        assert_eq!(reconcile(&config).unwrap(), 0);
        assert_eq!(agent(&config, &id).shutdown_recovery, None);
    }
}

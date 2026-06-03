//! ACP agent status/count handle for the dashboard.
//!
//! Unlike [`crate::rest::server::RestApiServer`], this is **not** a listener
//! lifecycle — editor-spawned `operator acp` runs in a separate stdio
//! subprocess that the TUI never hosts. [`AcpAgentServer`] just records
//! whether ACP is advertised in the dashboard and how many sessions are
//! currently active (always `0` in v1, since out-of-process ACP runs don't
//! report back to the TUI).
//!
//! When a shared file/socket bridge is added later, `active_sessions` can be
//! populated from there without changing this handle's shape.

use std::sync::{Arc, Mutex};

use crate::config::Config;

/// Coarse status reported to the dashboard.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AcpAgentStatus {
    /// `[acp].stdio_advertised = false` — operator is intentionally not
    /// advertising itself as an ACP agent.
    Disabled,
    /// Advertised. No active sessions are visible to the TUI (the editor
    /// runs `operator acp` out-of-process in v1, so the count is always 0
    /// here). The dashboard can still surface "ready" and offer config
    /// snippets.
    Advertised { active_sessions: usize },
}

impl AcpAgentStatus {
    pub fn is_advertised(&self) -> bool {
        matches!(self, AcpAgentStatus::Advertised { .. })
    }

    pub fn active_sessions(&self) -> usize {
        match self {
            AcpAgentStatus::Advertised { active_sessions } => *active_sessions,
            AcpAgentStatus::Disabled => 0,
        }
    }
}

use serde::{Deserialize, Serialize};

/// Status handle wired into `App` and read by the dashboard. Shape mirrors
/// the lock-protected pattern of [`crate::rest::server::RestApiServer`] so
/// future TUI-launched listeners can slot in without changing call sites.
#[derive(Debug, Clone)]
pub struct AcpAgentServer {
    status: Arc<Mutex<AcpAgentStatus>>,
}

impl AcpAgentServer {
    /// Construct from a config snapshot. Honors `config.acp.stdio_advertised`.
    pub fn from_config(config: &Config) -> Self {
        let status = if config.acp.stdio_advertised {
            AcpAgentStatus::Advertised { active_sessions: 0 }
        } else {
            AcpAgentStatus::Disabled
        };
        Self {
            status: Arc::new(Mutex::new(status)),
        }
    }

    /// Current status (cloned out of the mutex).
    pub fn status(&self) -> AcpAgentStatus {
        self.status.lock().unwrap().clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_config_advertised_default() {
        let config = Config::default();
        let server = AcpAgentServer::from_config(&config);
        assert!(server.status().is_advertised());
        assert_eq!(server.status().active_sessions(), 0);
    }

    #[test]
    fn test_from_config_disabled_when_flag_off() {
        let mut config = Config::default();
        config.acp.stdio_advertised = false;
        let server = AcpAgentServer::from_config(&config);
        assert!(!server.status().is_advertised());
        assert_eq!(server.status(), AcpAgentStatus::Disabled);
    }
}

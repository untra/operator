//! Relay hub and channel client for multi-agent peer-to-peer communication.
//!
//! The hub runs embedded in operator's process lifetime. Existing TypeScript
//! claude-relay channels connect unchanged (wire-compatible protocol).
//!
//! See `docs/relay/` for architecture documentation.

// Re-export the shared relay crate so existing `crate::relay::*` paths continue to work.
#[cfg(unix)]
pub use operator_relay::hub;
pub use operator_relay::socket_path;

/// The socket of the hub this process started, if any.
///
/// Agents need the path in their own environment, but writing it into
/// *Operator's* environment with `set_var` made a launch detail into
/// process-global state - and, under Rust 2024, a data race. The hub is a
/// server-wide resource, so one process-wide slot is the honest shape; each
/// launcher reads it and exports it into the child it spawns.
static ACTIVE_HUB_SOCKET: std::sync::OnceLock<std::path::PathBuf> = std::sync::OnceLock::new();

/// Record the hub this process is serving. First writer wins; a second hub
/// would not be reachable at a different path anyway.
pub fn set_active_hub_socket(path: std::path::PathBuf) {
    let _ = ACTIVE_HUB_SOCKET.set(path);
}

/// The hub socket to hand a launched agent.
///
/// Prefers the hub this process started, then `$RELAY_HUB_SOCKET` so an
/// externally managed claude-relay hub is still found.
pub fn active_hub_socket() -> Option<String> {
    if let Some(path) = ACTIVE_HUB_SOCKET.get() {
        return Some(path.to_string_lossy().into_owned());
    }
    std::env::var("RELAY_HUB_SOCKET").ok()
}

#[cfg(test)]
mod tests {
    /// The hub socket must reach agents through their own environment, never by
    /// mutating Operator's. `set_var` here was a process-global write from a
    /// launch path, and the thing standing between one server and several
    /// configurations sharing one environment.
    #[test]
    fn the_hub_socket_is_not_written_into_the_process_environment() {
        const SOURCES: &[(&str, &str)] = &[
            ("src/app/mod.rs", include_str!("../app/mod.rs")),
            (
                "src/agents/launcher/tmux_session.rs",
                include_str!("../agents/launcher/tmux_session.rs"),
            ),
            (
                "src/agents/launcher/cmux_session.rs",
                include_str!("../agents/launcher/cmux_session.rs"),
            ),
            (
                "src/agents/launcher/zellij_session.rs",
                include_str!("../agents/launcher/zellij_session.rs"),
            ),
        ];
        for (name, source) in SOURCES {
            assert!(
                !source.contains("set_var(\"RELAY_HUB_SOCKET\""),
                "{name} writes RELAY_HUB_SOCKET into the process environment"
            );
        }
    }
}

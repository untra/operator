//! Hub socket path resolution, matching claude-relay's data-dir.ts priority order.

use std::path::PathBuf;

/// Returns the Unix socket path for the relay hub.
///
/// Priority: `$RELAY_HUB_SOCKET` → `$CLAUDE_PLUGIN_DATA/hub.sock` → `~/.claude-relay/hub.sock`
///
/// This matches claude-relay's `data-dir.ts` so existing deployments find the hub
/// at the same path regardless of whether the TS or Rust hub is running.
pub fn hub_socket_path() -> PathBuf {
    if let Ok(explicit) = std::env::var("RELAY_HUB_SOCKET") {
        return PathBuf::from(explicit);
    }
    if let Ok(data_dir) = std::env::var("CLAUDE_PLUGIN_DATA") {
        return PathBuf::from(data_dir).join("hub.sock");
    }
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".claude-relay")
        .join("hub.sock")
}

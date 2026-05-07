//! Relay hub and channel client for multi-agent peer-to-peer communication.
//!
//! The hub runs embedded in operator's process lifetime. Existing TypeScript
//! claude-relay channels connect unchanged (wire-compatible protocol).
//!
//! See `docs/relay/` for architecture documentation.

// Re-export the shared relay crate so existing `crate::relay::*` paths continue to work.
pub use operator_relay::hub;
pub use operator_relay::socket_path;

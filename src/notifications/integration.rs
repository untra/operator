//! Notification integration trait definition.

use anyhow::Result;
use async_trait::async_trait;

use super::NotificationEvent;

/// Trait for notification integrations.
///
/// Each integration (OS notifications, webhooks, etc.) implements this trait
/// to receive and handle notification events.
#[allow(dead_code)] // Used by main.rs binary via mod, not via lib crate
#[async_trait]
pub trait NotificationIntegration: Send + Sync {
    /// Integration name (for logging and config identification)
    fn name(&self) -> &str;

    /// Check if this integration handles the given event type.
    ///
    /// Returns true if this integration should receive the event.
    /// Used for per-integration event filtering.
    fn handles_event(&self, event: &NotificationEvent) -> bool;

    /// Check if this integration is enabled.
    fn is_enabled(&self) -> bool;

    /// Send a notification event.
    ///
    /// This is fire-and-forget - implementations should log errors
    /// but not fail the overall notification dispatch.
    async fn send(&self, event: &NotificationEvent) -> Result<()>;
}

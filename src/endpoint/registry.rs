//! Live uplink connections, keyed by endpoint id.
//!
//! The registry is the in-memory truth while an endpoint is connected; the
//! auth store keeps the durable record. Every connection carries a
//! *generation*: a newer socket for the same endpoint supersedes the older one,
//! and calls made with a superseded generation are ignored, so a slow-closing
//! old socket can never overwrite or disconnect the new one.

use std::collections::HashMap;
use std::sync::Mutex;

use chrono::{DateTime, Duration, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use ts_rs::TS;
use utoipa::ToSchema;

use super::protocol::{Capabilities, DownlinkFrame};

pub const HEARTBEAT_INTERVAL_SECS: u32 = 15;
const HEARTBEATS_MISSED_BEFORE_STALE: i64 = 3;
pub const HEARTBEAT_TTL: Duration =
    Duration::seconds(HEARTBEATS_MISSED_BEFORE_STALE * HEARTBEAT_INTERVAL_SECS as i64);
pub const OUTBOUND_BUFFER: usize = 256;

/// A WebSocket close code in the application range, with its reason.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CloseCode {
    pub code: u16,
    pub reason: &'static str,
}

pub const SUPERSEDED: CloseCode = CloseCode {
    code: 4000,
    reason: "superseded by a newer connection",
};
pub const INCOMPATIBLE_VERSION: CloseCode = CloseCode {
    code: 4001,
    reason: "incompatible protocol version",
};
pub const REVOKED: CloseCode = CloseCode {
    code: 4002,
    reason: "endpoint revoked",
};
pub const PROTOCOL_ERROR: CloseCode = CloseCode {
    code: 4003,
    reason: "protocol error",
};
pub const HELLO_TIMEOUT: CloseCode = CloseCode {
    code: 4004,
    reason: "no hello received in time",
};

/// What the per-connection writer task sends.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outbound {
    Frame(Box<DownlinkFrame>),
    Close(CloseCode),
}

pub type Generation = u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum Liveness {
    /// Connected and heartbeating within [`HEARTBEAT_TTL`].
    Live,
    /// Connected, but no heartbeat within [`HEARTBEAT_TTL`].
    Stale,
    /// No open uplink.
    Offline,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashCheck {
    Match,
    /// The heartbeat's capabilities hash differs from the last `Capabilities`
    /// frame: the client changed without telling us.
    Mismatch,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Snapshot {
    pub connected: bool,
    pub last_heartbeat: Option<DateTime<Utc>>,
    pub capabilities: Option<Capabilities>,
    pub capabilities_stale: bool,
}

struct Connection {
    generation: Generation,
    tx: mpsc::Sender<Outbound>,
}

#[derive(Default)]
struct Entry {
    connection: Option<Connection>,
    last_generation: Generation,
    last_heartbeat: Option<DateTime<Utc>>,
    capabilities: Option<Capabilities>,
    capabilities_stale: bool,
}

impl Entry {
    fn current(&mut self, generation: Generation) -> Option<&mut Self> {
        let live = self
            .connection
            .as_ref()
            .is_some_and(|c| c.generation == generation);
        live.then_some(self)
    }
}

#[derive(Default)]
pub struct EndpointRegistry {
    entries: Mutex<HashMap<String, Entry>>,
    stopping: std::sync::atomic::AtomicBool,
    stopped: tokio::sync::Notify,
}

impl EndpointRegistry {
    pub fn shutdown(&self) {
        self.stopping
            .store(true, std::sync::atomic::Ordering::Release);
        self.stopped.notify_waiters();
    }

    pub fn is_stopping(&self) -> bool {
        self.stopping.load(std::sync::atomic::Ordering::Acquire)
    }

    pub async fn stopped(&self) {
        let stopped = self.stopped.notified();
        tokio::pin!(stopped);
        stopped.as_mut().enable();
        if !self.is_stopping() {
            stopped.await;
        }
    }

    fn with<R>(&self, f: impl FnOnce(&mut HashMap<String, Entry>) -> R) -> R {
        let mut entries = self
            .entries
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        f(&mut entries)
    }

    fn with_current<R>(
        &self,
        id: &str,
        generation: Generation,
        f: impl FnOnce(&mut Entry) -> R,
    ) -> Option<R> {
        self.with(|entries| entries.get_mut(id)?.current(generation).map(f))
    }

    /// Register a new uplink, superseding (and closing) any existing one.
    pub fn connect(&self, id: &str, tx: mpsc::Sender<Outbound>) -> Generation {
        self.with(|entries| {
            let entry = entries.entry(id.to_string()).or_default();
            if let Some(old) = entry.connection.take() {
                // Best effort: a full or closed channel means the old writer is
                // already gone or about to be.
                let _ = old.tx.try_send(Outbound::Close(SUPERSEDED));
            }
            entry.last_generation += 1;
            entry.last_heartbeat = None;
            entry.connection = Some(Connection {
                generation: entry.last_generation,
                tx,
            });
            entry.last_generation
        })
    }

    #[cfg(test)]
    pub fn is_current(&self, id: &str, generation: Generation) -> bool {
        self.with_current(id, generation, |_| ()).is_some()
    }

    /// Drop the connection, but only if `generation` still owns it.
    pub fn disconnect(&self, id: &str, generation: Generation) {
        self.with_current(id, generation, |entry| entry.connection = None);
    }

    /// Record a heartbeat. `None` when `generation` is no longer current.
    pub fn heartbeat(
        &self,
        id: &str,
        generation: Generation,
        now: DateTime<Utc>,
        capabilities_hash: &str,
    ) -> Option<HashCheck> {
        self.with_current(id, generation, |entry| {
            entry.last_heartbeat = Some(now);
            let check = match &entry.capabilities {
                Some(known) if known.hash != capabilities_hash => HashCheck::Mismatch,
                _ => HashCheck::Match,
            };
            if check == HashCheck::Mismatch {
                entry.capabilities_stale = true;
            }
            check
        })
    }

    /// Replace capabilities; also counts as a heartbeat. False when stale.
    pub fn set_capabilities(
        &self,
        id: &str,
        generation: Generation,
        capabilities: Capabilities,
        now: DateTime<Utc>,
    ) -> bool {
        self.with_current(id, generation, |entry| {
            entry.capabilities = Some(capabilities);
            entry.capabilities_stale = false;
            entry.last_heartbeat = Some(now);
        })
        .is_some()
    }

    pub fn liveness(&self, id: &str, now: DateTime<Utc>) -> Liveness {
        self.with(|entries| match entries.get(id) {
            Some(Entry {
                connection: Some(_),
                last_heartbeat: Some(at),
                ..
            }) if now - *at <= HEARTBEAT_TTL => Liveness::Live,
            Some(Entry {
                connection: Some(_),
                ..
            }) => Liveness::Stale,
            _ => Liveness::Offline,
        })
    }

    /// Close the current uplink with `code`. False when none is open.
    pub fn close(&self, id: &str, code: CloseCode) -> bool {
        self.with(|entries| {
            let Some(connection) = entries.get_mut(id).and_then(|e| e.connection.take()) else {
                return false;
            };
            let _ = connection.tx.try_send(Outbound::Close(code));
            true
        })
    }

    pub fn snapshot(&self, id: &str) -> Option<Snapshot> {
        self.with(|entries| {
            entries.get(id).map(|entry| Snapshot {
                connected: entry.connection.is_some(),
                last_heartbeat: entry.last_heartbeat,
                capabilities: entry.capabilities.clone(),
                capabilities_stale: entry.capabilities_stale,
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::endpoint::protocol::Capabilities;
    use chrono::{TimeZone, Utc};
    use tokio::sync::mpsc;

    fn t(secs: i64) -> DateTime<Utc> {
        Utc.timestamp_opt(1_800_000_000 + secs, 0).unwrap()
    }

    fn channel() -> (mpsc::Sender<Outbound>, mpsc::Receiver<Outbound>) {
        mpsc::channel(OUTBOUND_BUFFER)
    }

    fn caps_v(hash: &str) -> Capabilities {
        Capabilities {
            hash: hash.into(),
            hostname: "pc".into(),
            os: "windows".into(),
            arch: "x86_64".into(),
            gpus: vec![],
            llm_tools: vec![],
            model_servers: vec![],
            projects: vec![],
            launch_enabled: false,
        }
    }

    fn caps() -> Capabilities {
        caps_v("h1")
    }

    #[test]
    fn a_new_connection_supersedes_the_old_one() {
        let reg = EndpointRegistry::default();
        let (tx1, mut rx1) = channel();
        let (tx2, mut rx2) = channel();
        let g1 = reg.connect("ep", tx1);
        let g2 = reg.connect("ep", tx2);
        assert_ne!(g1, g2);

        assert_eq!(rx1.try_recv().unwrap(), Outbound::Close(SUPERSEDED));
        assert!(!reg.is_current("ep", g1));
        assert!(reg.is_current("ep", g2));

        assert!(reg.heartbeat("ep", g1, t(0), "h").is_none());
        reg.disconnect("ep", g1);
        assert!(
            reg.is_current("ep", g2),
            "a stale disconnect must not drop the new socket"
        );
        assert!(rx2.try_recv().is_err());
    }

    #[test]
    fn liveness_follows_heartbeats_and_the_socket() {
        let reg = EndpointRegistry::default();
        assert_eq!(reg.liveness("ep", t(0)), Liveness::Offline);

        let (tx, _rx) = channel();
        let g = reg.connect("ep", tx);
        assert_eq!(
            reg.liveness("ep", t(0)),
            Liveness::Stale,
            "connected, no heartbeat yet"
        );

        assert_eq!(reg.heartbeat("ep", g, t(0), "h"), Some(HashCheck::Match));
        assert_eq!(reg.liveness("ep", t(0)), Liveness::Live);
        let ttl = HEARTBEAT_TTL.num_seconds();
        assert_eq!(reg.liveness("ep", t(ttl)), Liveness::Live);
        assert_eq!(reg.liveness("ep", t(ttl + 1)), Liveness::Stale);

        reg.disconnect("ep", g);
        assert_eq!(reg.liveness("ep", t(1)), Liveness::Offline);
    }

    #[test]
    fn heartbeat_hash_is_checked_against_the_last_capabilities() {
        let reg = EndpointRegistry::default();
        let (tx, _rx) = channel();
        let g = reg.connect("ep", tx);
        assert!(reg.set_capabilities("ep", g, caps(), t(0)));

        assert_eq!(reg.heartbeat("ep", g, t(1), "h1"), Some(HashCheck::Match));
        assert_eq!(
            reg.heartbeat("ep", g, t(2), "h2"),
            Some(HashCheck::Mismatch)
        );
        assert!(reg.snapshot("ep").unwrap().capabilities_stale);

        assert!(reg.set_capabilities("ep", g, caps_v("h2"), t(3)));
        let snap = reg.snapshot("ep").unwrap();
        assert!(!snap.capabilities_stale);
        assert_eq!(snap.capabilities, Some(caps_v("h2")));
        assert_eq!(snap.last_heartbeat, Some(t(3)));
    }

    #[test]
    fn stale_generations_cannot_write_capabilities() {
        let reg = EndpointRegistry::default();
        let (tx1, _rx1) = channel();
        let (tx2, _rx2) = channel();
        let g1 = reg.connect("ep", tx1);
        let _g2 = reg.connect("ep", tx2);
        assert!(!reg.set_capabilities("ep", g1, caps(), t(0)));
        assert!(reg.snapshot("ep").unwrap().capabilities.is_none());
    }

    #[test]
    fn close_reaches_only_the_current_socket_and_disconnects_it() {
        let reg = EndpointRegistry::default();
        let (tx, mut rx) = channel();
        let g = reg.connect("ep", tx);
        assert!(reg.close("ep", REVOKED));
        assert_eq!(rx.try_recv().unwrap(), Outbound::Close(REVOKED));
        assert!(!reg.is_current("ep", g));
        assert!(!reg.close("ep", REVOKED), "nothing left to close");
        assert!(!reg.close("other", REVOKED));
    }

    #[test]
    fn close_codes_are_distinct_application_codes() {
        let codes = [
            SUPERSEDED,
            INCOMPATIBLE_VERSION,
            REVOKED,
            PROTOCOL_ERROR,
            HELLO_TIMEOUT,
        ];
        let set: std::collections::BTreeSet<u16> = codes.iter().map(|c| c.code).collect();
        assert_eq!(set.len(), codes.len());
        assert!(codes.iter().all(|c| (4000..5000).contains(&c.code)));
    }
}

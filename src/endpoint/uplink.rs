//! One endpoint uplink: a pure per-connection state machine plus thin socket glue.
//!
//! [`Session`] decides what each inbound text frame means and returns
//! [`Action`]s; the glue performs them (writes, closes, store persistence).
//! Keeping the decisions pure lets every rule be tested without a socket.

use axum::extract::ws::{CloseFrame, Message, WebSocket};
use chrono::{DateTime, Duration, Utc};
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;

use crate::auth::store::AuthStore;
use crate::rest::state::ApiState;

use super::protocol::{is_compatible, Capabilities, DownlinkFrame, UplinkFrame, Welcome};
use super::registry::{
    CloseCode, EndpointRegistry, Generation, HashCheck, Outbound, HEARTBEAT_INTERVAL_SECS,
    HELLO_TIMEOUT, INCOMPATIBLE_VERSION, OUTBOUND_BUFFER, PROTOCOL_ERROR, REVOKED,
};

/// Minimum spacing between `last_seen_at` writes to the store; the registry
/// holds the precise value while connected.
pub const LAST_SEEN_PERSIST_EVERY: Duration = Duration::seconds(60);

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    Send(DownlinkFrame),
    Close(CloseCode),
    PersistCapabilities(Capabilities),
    PersistLastSeen,
}

pub struct Session {
    endpoint_id: String,
    generation: Generation,
    greeted: bool,
    last_persisted: Option<DateTime<Utc>>,
}

impl Session {
    pub fn new(endpoint_id: String, generation: Generation) -> Self {
        Self {
            endpoint_id,
            generation,
            greeted: false,
            last_persisted: None,
        }
    }

    pub fn endpoint_id(&self) -> &str {
        &self.endpoint_id
    }

    pub fn on_text(
        &mut self,
        registry: &EndpointRegistry,
        text: &str,
        now: DateTime<Utc>,
    ) -> Vec<Action> {
        let Ok(frame) = serde_json::from_str::<UplinkFrame>(text) else {
            return vec![Action::Close(PROTOCOL_ERROR)];
        };
        match (self.greeted, frame) {
            (false, UplinkFrame::Hello(hello)) => {
                if !is_compatible(hello.protocol_version) {
                    return vec![Action::Close(INCOMPATIBLE_VERSION)];
                }
                if !self.store_capabilities(registry, hello.capabilities.clone(), now) {
                    return vec![];
                }
                self.greeted = true;
                vec![
                    Action::PersistCapabilities(hello.capabilities),
                    Action::Send(DownlinkFrame::Welcome(Welcome {
                        endpoint_id: self.endpoint_id.clone(),
                        heartbeat_interval_secs: HEARTBEAT_INTERVAL_SECS,
                    })),
                ]
            }
            (false, _) | (true, UplinkFrame::Hello(_)) => vec![Action::Close(PROTOCOL_ERROR)],
            (true, UplinkFrame::Heartbeat(beat)) => {
                let check = registry.heartbeat(
                    &self.endpoint_id,
                    self.generation,
                    now,
                    &beat.capabilities_hash,
                );
                match check {
                    None => vec![],
                    Some(HashCheck::Mismatch) => {
                        tracing::debug!(
                            endpoint = %self.endpoint_id,
                            "heartbeat capabilities hash differs from the last capabilities frame"
                        );
                        self.persist_last_seen_if_due(now)
                    }
                    Some(HashCheck::Match) => self.persist_last_seen_if_due(now),
                }
            }
            (true, UplinkFrame::Capabilities(caps)) => {
                if self.store_capabilities(registry, caps.clone(), now) {
                    vec![Action::PersistCapabilities(caps)]
                } else {
                    vec![]
                }
            }
            (true, other) => {
                tracing::debug!(
                    endpoint = %self.endpoint_id,
                    frame = other.tag(),
                    "ignoring uplink frame not yet handled"
                );
                vec![]
            }
        }
    }

    fn store_capabilities(
        &mut self,
        registry: &EndpointRegistry,
        caps: Capabilities,
        now: DateTime<Utc>,
    ) -> bool {
        let stored = registry.set_capabilities(&self.endpoint_id, self.generation, caps, now);
        if stored {
            self.last_persisted = Some(now);
        }
        stored
    }

    fn persist_last_seen_if_due(&mut self, now: DateTime<Utc>) -> Vec<Action> {
        let due = self
            .last_persisted
            .is_none_or(|at| now - at >= LAST_SEEN_PERSIST_EVERY);
        if !due {
            return vec![];
        }
        self.last_persisted = Some(now);
        vec![Action::PersistLastSeen]
    }
}

/// How long a fresh socket may stay silent before its `Hello`.
pub const HELLO_WAIT: std::time::Duration = std::time::Duration::from_secs(10);

/// Drive one upgraded uplink socket until either side closes it.
///
/// All writes go through one channel to a single writer task, so frames from
/// the session and closes from the registry (supersede, revoke) never
/// interleave. Store writes are awaited in order, so an older capabilities
/// write cannot land after a newer one.
pub async fn run(socket: WebSocket, state: ApiState, endpoint_id: String) {
    let Ok(_permit) = state.begin_callback() else {
        return;
    };
    let (tx, rx) = mpsc::channel(OUTBOUND_BUFFER);
    let generation = state.endpoints.connect(&endpoint_id, tx.clone());
    let (sink, mut stream) = socket.split();
    let mut writer = tokio::spawn(write_outbound(sink, rx));
    let mut session = Session::new(endpoint_id.clone(), generation);

    let first = tokio::select! {
        () = state.endpoints.stopped() => Ok(None),
        result = tokio::time::timeout(HELLO_WAIT, stream.next()) => result,
    };
    let mut next = match first {
        Ok(message) => message,
        Err(_) => {
            let _ = tx.send(Outbound::Close(HELLO_TIMEOUT)).await;
            None
        }
    };

    while let Some(Ok(message)) = next {
        let text = match message {
            Message::Text(text) => text,
            Message::Close(_) => break,
            Message::Ping(_) | Message::Pong(_) => {
                next = read_next(&mut stream, &mut writer).await;
                continue;
            }
            Message::Binary(_) => {
                let _ = tx.send(Outbound::Close(PROTOCOL_ERROR)).await;
                break;
            }
        };
        let actions = session.on_text(&state.endpoints, text.as_str(), Utc::now());
        if !perform(&state, &session, &tx, actions).await {
            break;
        }
        next = tokio::select! {
            () = state.endpoints.stopped() => None,
            next = read_next(&mut stream, &mut writer) => next,
        };
    }

    state.endpoints.disconnect(&endpoint_id, generation);
    drop(tx);
    if state.endpoints.is_stopping() {
        writer.abort();
    }
    let _ = writer.await;
}

/// The next inbound message, or `None` once the writer has closed the socket.
async fn read_next(
    stream: &mut SplitStream<WebSocket>,
    writer: &mut tokio::task::JoinHandle<()>,
) -> Option<Result<Message, axum::Error>> {
    tokio::select! {
        message = stream.next() => message,
        _ = writer => None,
    }
}

async fn write_outbound(mut sink: SplitSink<WebSocket, Message>, mut rx: mpsc::Receiver<Outbound>) {
    while let Some(outbound) = rx.recv().await {
        match outbound {
            Outbound::Frame(frame) => {
                let Ok(text) = serde_json::to_string(frame.as_ref()) else {
                    continue;
                };
                if sink.send(Message::Text(text.into())).await.is_err() {
                    return;
                }
            }
            Outbound::Close(close) => {
                let _ = sink
                    .send(Message::Close(Some(CloseFrame {
                        code: close.code,
                        reason: close.reason.into(),
                    })))
                    .await;
                return;
            }
        }
    }
}

/// Perform `actions`; false when the session must end.
async fn perform(
    state: &ApiState,
    session: &Session,
    tx: &mpsc::Sender<Outbound>,
    actions: Vec<Action>,
) -> bool {
    for action in actions {
        match action {
            Action::Send(frame) => {
                if tx.send(Outbound::Frame(Box::new(frame))).await.is_err() {
                    return false;
                }
            }
            Action::Close(close) => {
                let _ = tx.send(Outbound::Close(close)).await;
                return false;
            }
            Action::PersistCapabilities(caps) => {
                persist(state, session.endpoint_id(), move |store, id, now| {
                    let json = serde_json::to_string(&caps)?;
                    store.record_endpoint_capabilities(id, &json, &caps.hash, now)?;
                    store.adopt_endpoint_hostname(id, &caps.hostname)?;
                    Ok(())
                })
                .await;
            }
            Action::PersistLastSeen => {
                let profile = state.config().profile.id;
                let live = persist(state, session.endpoint_id(), move |store, id, now| {
                    store.touch_endpoint(id, now)?;
                    Ok(store.endpoint_authority(id)?.is_some()
                        && store
                            .device_authority(id, profile)?
                            .is_some_and(|a| a.user.administrator))
                })
                .await;
                // Revocations that bypass the REST revoke (password reset,
                // refresh-token reuse) are caught here, within one interval.
                if live != Some(true)
                    || crate::licensing::require_premium(
                        &state.config(),
                        crate::licensing::PremiumFeature::RemoteTargets,
                    )
                    .is_err()
                {
                    let _ = tx.send(Outbound::Close(REVOKED)).await;
                    return false;
                }
            }
        }
    }
    true
}

async fn persist<T: Send + 'static>(
    state: &ApiState,
    endpoint_id: &str,
    work: impl FnOnce(&AuthStore, &str, DateTime<Utc>) -> anyhow::Result<T> + Send + 'static,
) -> Option<T> {
    let store = state.auth.store.clone();
    let id = endpoint_id.to_string();
    let result = tokio::task::spawn_blocking(move || work(&store, &id, Utc::now())).await;
    match result {
        Ok(Ok(value)) => Some(value),
        Ok(Err(error)) => {
            tracing::warn!(endpoint = %endpoint_id, %error, "endpoint persistence failed");
            None
        }
        Err(error) => {
            tracing::warn!(endpoint = %endpoint_id, %error, "endpoint persistence task failed");
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::endpoint::protocol::{
        fixtures, Capabilities, Heartbeat, Hello, TunnelEnd, UplinkFrame, PROTOCOL_VERSION,
    };
    use crate::endpoint::registry::{
        EndpointRegistry, Liveness, Outbound, INCOMPATIBLE_VERSION, OUTBOUND_BUFFER, PROTOCOL_ERROR,
    };
    use chrono::{TimeZone, Utc};
    use tokio::sync::mpsc;

    const EP: &str = "ep-1";

    fn t(secs: i64) -> DateTime<Utc> {
        Utc.timestamp_opt(1_800_000_000 + secs, 0).unwrap()
    }

    fn caps(hash: &str) -> Capabilities {
        Capabilities {
            hash: hash.into(),
            hostname: "GAMING-PC".into(),
            os: "windows".into(),
            arch: "x86_64".into(),
            gpus: vec![],
            llm_tools: vec![],
            model_servers: vec![],
            projects: vec![],
            launch_enabled: false,
        }
    }

    fn text(frame: &UplinkFrame) -> String {
        serde_json::to_string(frame).unwrap()
    }

    fn hello(version: u32) -> String {
        text(&UplinkFrame::Hello(Hello {
            protocol_version: version,
            client_version: "0.1.0".into(),
            capabilities: caps("h1"),
        }))
    }

    fn heartbeat(seq: u64, hash: &str) -> String {
        text(&UplinkFrame::Heartbeat(Heartbeat {
            seq,
            capabilities_hash: hash.into(),
        }))
    }

    struct Harness {
        registry: EndpointRegistry,
        session: Session,
        _rx: mpsc::Receiver<Outbound>,
    }

    fn connected() -> Harness {
        let registry = EndpointRegistry::default();
        let (tx, rx) = mpsc::channel(OUTBOUND_BUFFER);
        let generation = registry.connect(EP, tx);
        Harness {
            session: Session::new(EP.into(), generation),
            registry,
            _rx: rx,
        }
    }

    impl Harness {
        fn feed(&mut self, frame: &str, now: DateTime<Utc>) -> Vec<Action> {
            self.session.on_text(&self.registry, frame, now)
        }
    }

    fn welcomed() -> Harness {
        let mut h = connected();
        h.feed(&hello(PROTOCOL_VERSION), t(0));
        h
    }

    #[test]
    fn hello_is_welcomed_and_persisted() {
        let mut h = connected();
        let actions = h.feed(&hello(PROTOCOL_VERSION), t(0));
        assert_eq!(
            actions,
            vec![
                Action::PersistCapabilities(caps("h1")),
                Action::Send(DownlinkFrame::Welcome(Welcome {
                    endpoint_id: EP.into(),
                    heartbeat_interval_secs: HEARTBEAT_INTERVAL_SECS,
                })),
            ]
        );
        assert_eq!(h.registry.liveness(EP, t(0)), Liveness::Live);
    }

    #[test]
    fn an_incompatible_hello_is_closed() {
        let mut h = connected();
        assert_eq!(
            h.feed(&hello(PROTOCOL_VERSION + 1), t(0)),
            vec![Action::Close(INCOMPATIBLE_VERSION)]
        );
    }

    #[test]
    fn anything_but_hello_first_is_a_protocol_error() {
        for first in [
            heartbeat(1, "h1"),
            "not json".to_string(),
            r#"{"type":"nope"}"#.into(),
        ] {
            let mut h = connected();
            assert_eq!(
                h.feed(&first, t(0)),
                vec![Action::Close(PROTOCOL_ERROR)],
                "{first}"
            );
        }
    }

    #[test]
    fn a_second_hello_is_a_protocol_error() {
        let mut h = welcomed();
        assert_eq!(
            h.feed(&hello(PROTOCOL_VERSION), t(1)),
            vec![Action::Close(PROTOCOL_ERROR)]
        );
    }

    #[test]
    fn heartbeats_persist_last_seen_at_most_once_per_interval() {
        let mut h = welcomed();
        let every = LAST_SEEN_PERSIST_EVERY.num_seconds();
        assert_eq!(h.feed(&heartbeat(1, "h1"), t(every - 1)), vec![]);
        assert_eq!(
            h.feed(&heartbeat(2, "h1"), t(every)),
            vec![Action::PersistLastSeen]
        );
        assert_eq!(h.feed(&heartbeat(3, "h1"), t(every + 1)), vec![]);
        assert_eq!(h.registry.liveness(EP, t(every + 1)), Liveness::Live);
    }

    #[test]
    fn a_mismatched_heartbeat_marks_capabilities_stale_without_closing() {
        let mut h = welcomed();
        assert_eq!(h.feed(&heartbeat(1, "other"), t(1)), vec![]);
        assert!(h.registry.snapshot(EP).unwrap().capabilities_stale);
    }

    #[test]
    fn a_capabilities_frame_replaces_and_persists() {
        let mut h = welcomed();
        let frame = text(&UplinkFrame::Capabilities(caps("h2")));
        assert_eq!(
            h.feed(&frame, t(1)),
            vec![Action::PersistCapabilities(caps("h2"))]
        );
        assert_eq!(
            h.registry.snapshot(EP).unwrap().capabilities,
            Some(caps("h2"))
        );
    }

    #[test]
    fn part_two_frames_are_ignored_not_fatal() {
        let mut h = welcomed();
        for (name, frame) in fixtures().uplink {
            if matches!(
                frame,
                UplinkFrame::Hello(_) | UplinkFrame::Heartbeat(_) | UplinkFrame::Capabilities(_)
            ) {
                continue;
            }
            assert_eq!(h.feed(&text(&frame), t(1)), vec![], "{name}");
        }
        let end = text(&UplinkFrame::TunnelEnd(TunnelEnd { req_id: "r".into() }));
        assert_eq!(h.feed(&end, t(2)), vec![]);
    }

    #[test]
    fn a_superseded_session_does_nothing() {
        let mut h = welcomed();
        let (tx, _rx2) = mpsc::channel(OUTBOUND_BUFFER);
        h.registry.connect(EP, tx);
        assert_eq!(h.feed(&heartbeat(1, "h1"), t(1)), vec![]);
        let frame = text(&UplinkFrame::Capabilities(caps("h9")));
        assert_eq!(h.feed(&frame, t(2)), vec![]);
    }
}

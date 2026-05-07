//! Relay hub — tokio actor that owns the peer registry and routes ask/reply/broadcast messages.
//!
//! The hub runs embedded in operator's async runtime (lifetime = operator lifetime).
//! No idle-shutdown timer: the hub exits only when operator exits.
//!
//! Wire protocol is byte-compatible with claude-relay's hub, so existing TypeScript
//! channel binaries connect unchanged.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::mpsc;

use crate::protocol::{
    self, ClientMsg, ErrCode, PeerRecord, ServerMsg, MAX_LINE_LEN, PROTOCOL_VERSION,
};

// ── Public handle ────────────────────────────────────────────────────────────

/// Handle to the running relay hub.
pub struct RelayHub {
    cmd_tx: mpsc::Sender<HubCommand>,
    socket_path: PathBuf,
    accept_handle: tokio::task::JoinHandle<()>,
}

impl RelayHub {
    /// Start the relay hub on the given socket path.
    ///
    /// Performs stale-socket recovery: if the socket file exists but is not responsive,
    /// it is removed and a fresh listener is bound.
    pub async fn start(socket_path: PathBuf) -> anyhow::Result<Self> {
        // Create parent directory
        if let Some(parent) = socket_path.parent() {
            std::fs::create_dir_all(parent)?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let perms = std::fs::Permissions::from_mode(0o700);
                let _ = std::fs::set_permissions(parent, perms);
            }
        }

        // Stale socket recovery
        if socket_path.exists() {
            match tokio::time::timeout(
                Duration::from_millis(200),
                UnixStream::connect(&socket_path),
            )
            .await
            {
                Ok(Ok(_)) => {
                    return Err(anyhow::anyhow!(
                        "Relay hub socket {} is already in use (another hub may be running)",
                        socket_path.display()
                    ));
                }
                _ => {
                    // Stale socket — remove it
                    let _ = std::fs::remove_file(&socket_path);
                }
            }
        }

        let listener = UnixListener::bind(&socket_path)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o600);
            let _ = std::fs::set_permissions(&socket_path, perms);
        }

        let (cmd_tx, cmd_rx) = mpsc::channel(512);

        // Spawn actor
        let actor_cmd_tx = cmd_tx.clone();
        tokio::spawn(async move {
            run_actor(cmd_rx, actor_cmd_tx).await;
        });

        // Spawn accept loop
        let accept_cmd_tx = cmd_tx.clone();
        let accept_handle = tokio::spawn(async move {
            run_accept_loop(listener, accept_cmd_tx).await;
        });

        tracing::info!(socket = %socket_path.display(), "Relay hub started");

        Ok(Self {
            cmd_tx,
            socket_path,
            accept_handle,
        })
    }

    /// Gracefully shut down the hub.
    pub async fn shutdown(self) {
        self.accept_handle.abort();
        let _ = self.cmd_tx.send(HubCommand::Shutdown).await;
        tokio::time::sleep(Duration::from_millis(50)).await;
        let _ = std::fs::remove_file(&self.socket_path);
        tracing::info!("Relay hub shut down");
    }

    pub fn socket_path(&self) -> &Path {
        &self.socket_path
    }
}

// ── Actor commands ────────────────────────────────────────────────────────────

enum HubCommand {
    ClientConnected {
        conn_id: u64,
        write_tx: mpsc::Sender<ServerMsg>,
    },
    ClientLine {
        conn_id: u64,
        line: String,
    },
    ClientDisconnect {
        conn_id: u64,
    },
    TimeoutExpired {
        ask_id: String,
    },
    Shutdown,
}

// ── State ─────────────────────────────────────────────────────────────────────

struct PeerEntry {
    name: String,
    cwd: String,
    git_branch: String,
    last_seen: u64,
    /// Non-None for operator-managed peers: evict if `now - last_seen > lease_ms`.
    lease_ms: Option<u64>,
}

struct PendingAsk {
    caller: String,
    target: String,
    broadcast_id: Option<String>,
    thread_id: Option<String>,
    timeout_abort: tokio::task::AbortHandle,
}

struct HubState {
    name_to_id: HashMap<String, u64>,
    id_to_entry: HashMap<u64, PeerEntry>,
    pending: HashMap<String, PendingAsk>,
    senders: HashMap<u64, mpsc::Sender<ServerMsg>>,
    cmd_tx: mpsc::Sender<HubCommand>,
    default_timeout_ms: u64,
}

impl HubState {
    fn new(cmd_tx: mpsc::Sender<HubCommand>) -> Self {
        Self {
            name_to_id: HashMap::new(),
            id_to_entry: HashMap::new(),
            pending: HashMap::new(),
            senders: HashMap::new(),
            cmd_tx,
            default_timeout_ms: 120_000,
        }
    }

    fn send_to_id(&self, conn_id: u64, msg: ServerMsg) {
        if let Some(tx) = self.senders.get(&conn_id) {
            let _ = tx.try_send(msg);
        }
    }

    fn send_to_name(&self, name: &str, msg: ServerMsg) {
        if let Some(&conn_id) = self.name_to_id.get(name) {
            self.send_to_id(conn_id, msg);
        }
    }

    fn get_name(&self, conn_id: u64) -> Option<&str> {
        self.id_to_entry.get(&conn_id).map(|e| e.name.as_str())
    }

    fn peer_list(&self, exclude: Option<&str>) -> Vec<PeerRecord> {
        self.id_to_entry
            .values()
            .filter(|e| exclude.is_none_or(|n| e.name != n))
            .map(|e| PeerRecord {
                name: e.name.clone(),
                cwd: e.cwd.clone(),
                git_branch: e.git_branch.clone(),
                last_seen: e.last_seen,
            })
            .collect()
    }

    fn all_names(&self) -> Vec<String> {
        self.id_to_entry.values().map(|e| e.name.clone()).collect()
    }

    // ── Message dispatch ──────────────────────────────────────────────────────

    fn handle_line(&mut self, conn_id: u64, line: &str) {
        let Ok(raw) = serde_json::from_str::<serde_json::Value>(line) else {
            self.send_to_id(
                conn_id,
                ServerMsg::Err {
                    code: ErrCode::BadMsg,
                    message: None,
                    req_id: None,
                    ask_id: None,
                },
            );
            return;
        };
        let Ok(msg) = serde_json::from_value::<ClientMsg>(raw) else {
            self.send_to_id(
                conn_id,
                ServerMsg::Err {
                    code: ErrCode::BadMsg,
                    message: None,
                    req_id: None,
                    ask_id: None,
                },
            );
            return;
        };

        if let Some(entry) = self.id_to_entry.get_mut(&conn_id) {
            entry.last_seen = protocol::now_ms();
        }

        match msg {
            ClientMsg::Register {
                name,
                cwd,
                git_branch,
                protocol_version,
                lease_ms,
            } => {
                self.handle_register(conn_id, name, cwd, git_branch, &protocol_version, lease_ms);
            }
            ClientMsg::Rename { new_name, req_id } => {
                self.handle_rename(conn_id, new_name, req_id);
            }
            ClientMsg::ListPeers { req_id } => {
                self.handle_list_peers(conn_id, req_id);
            }
            ClientMsg::Ask {
                to,
                question,
                ask_id,
                timeout_ms,
                thread_id,
            } => {
                self.handle_ask(conn_id, to, question, ask_id, timeout_ms, thread_id);
            }
            ClientMsg::Reply { ask_id, text } => {
                self.handle_reply(conn_id, ask_id, text);
            }
            ClientMsg::Broadcast {
                question,
                broadcast_id,
                exclude_self,
            } => {
                self.handle_broadcast(conn_id, question, broadcast_id, exclude_self);
            }
        }
    }

    fn handle_register(
        &mut self,
        conn_id: u64,
        name: String,
        cwd: String,
        git_branch: String,
        protocol_version: &str,
        lease_ms: Option<u64>,
    ) {
        if protocol_version != PROTOCOL_VERSION {
            self.send_to_id(
                conn_id,
                ServerMsg::Err {
                    code: ErrCode::ProtocolMismatch,
                    message: Some(format!(
                        "expected {PROTOCOL_VERSION}, got {protocol_version}"
                    )),
                    req_id: None,
                    ask_id: None,
                },
            );
            return;
        }
        if self.id_to_entry.contains_key(&conn_id) {
            self.send_to_id(
                conn_id,
                ServerMsg::Err {
                    code: ErrCode::AlreadyRegistered,
                    message: None,
                    req_id: None,
                    ask_id: None,
                },
            );
            return;
        }
        if self.name_to_id.contains_key(&name) {
            self.send_to_id(
                conn_id,
                ServerMsg::Err {
                    code: ErrCode::NameTaken,
                    message: None,
                    req_id: None,
                    ask_id: None,
                },
            );
            return;
        }
        self.name_to_id.insert(name.clone(), conn_id);
        self.id_to_entry.insert(
            conn_id,
            PeerEntry {
                name,
                cwd,
                git_branch,
                last_seen: protocol::now_ms(),
                lease_ms,
            },
        );
        self.send_to_id(conn_id, ServerMsg::Ack { req_id: None });
    }

    fn handle_rename(&mut self, conn_id: u64, new_name: String, req_id: Option<String>) {
        let Some(entry) = self.id_to_entry.get(&conn_id) else {
            self.send_to_id(
                conn_id,
                ServerMsg::Err {
                    code: ErrCode::NotRegistered,
                    message: None,
                    req_id,
                    ask_id: None,
                },
            );
            return;
        };
        let old_name = entry.name.clone();
        let already_taken = self
            .name_to_id
            .get(&new_name)
            .is_some_and(|&id| id != conn_id);
        if already_taken {
            self.send_to_id(
                conn_id,
                ServerMsg::Err {
                    code: ErrCode::NameTaken,
                    message: None,
                    req_id,
                    ask_id: None,
                },
            );
            return;
        }
        // Update registry
        self.name_to_id.remove(&old_name);
        self.name_to_id.insert(new_name.clone(), conn_id);
        if let Some(entry) = self.id_to_entry.get_mut(&conn_id) {
            entry.name = new_name.clone();
        }
        // Update pending asks — must happen before ack (matches TS ordering)
        self.update_name_on_rename(&old_name, &new_name);
        self.send_to_id(conn_id, ServerMsg::Ack { req_id });
    }

    fn handle_list_peers(&self, conn_id: u64, req_id: Option<String>) {
        let self_name = self.get_name(conn_id).map(str::to_string);
        let peers = self.peer_list(self_name.as_deref());
        self.send_to_id(conn_id, ServerMsg::Peers { peers, req_id });
    }

    fn handle_ask(
        &mut self,
        conn_id: u64,
        to: String,
        question: String,
        ask_id: String,
        timeout_ms: Option<u64>,
        thread_id: Option<String>,
    ) {
        let Some(caller_ref) = self.get_name(conn_id) else {
            self.send_to_id(
                conn_id,
                ServerMsg::Err {
                    code: ErrCode::NotRegistered,
                    message: None,
                    req_id: None,
                    ask_id: None,
                },
            );
            return;
        };
        let caller = caller_ref.to_string();
        if !self.name_to_id.contains_key(&to) {
            self.send_to_id(
                conn_id,
                ServerMsg::Err {
                    code: ErrCode::PeerNotFound,
                    message: None,
                    req_id: None,
                    ask_id: Some(ask_id),
                },
            );
            return;
        }
        let timeout_ms = timeout_ms.unwrap_or(self.default_timeout_ms);
        // Internal thread_id for reply routing; generated if not provided.
        // Only forward the caller-provided value in IncomingAsk so receivers
        // don't see hub-internal UUIDs they didn't ask for.
        let internal_thread_id = thread_id
            .clone()
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

        let ask_id_clone = ask_id.clone();
        let cmd_tx = self.cmd_tx.clone();
        let timeout_task = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(timeout_ms)).await;
            let _ = cmd_tx
                .send(HubCommand::TimeoutExpired {
                    ask_id: ask_id_clone,
                })
                .await;
        });

        self.pending.insert(
            ask_id.clone(),
            PendingAsk {
                caller: caller.clone(),
                target: to.clone(),
                broadcast_id: None,
                thread_id: Some(internal_thread_id),
                timeout_abort: timeout_task.abort_handle(),
            },
        );

        self.send_to_name(
            &to,
            ServerMsg::IncomingAsk {
                from: caller,
                question,
                ask_id,
                broadcast_id: None,
                thread_id, // preserve caller-provided value (may be None)
            },
        );
    }

    fn handle_reply(&mut self, conn_id: u64, ask_id: String, text: String) {
        let Some(replier_ref) = self.get_name(conn_id) else {
            self.send_to_id(
                conn_id,
                ServerMsg::Err {
                    code: ErrCode::NotRegistered,
                    message: None,
                    req_id: None,
                    ask_id: None,
                },
            );
            return;
        };
        let replier = replier_ref.to_string();
        // Validate ask exists and replier is the target
        let valid = self
            .pending
            .get(&ask_id)
            .is_some_and(|a| a.target == replier);
        if !valid {
            self.send_to_id(
                conn_id,
                ServerMsg::Err {
                    code: ErrCode::UnknownAsk,
                    message: None,
                    req_id: None,
                    ask_id: None,
                },
            );
            return;
        }
        let ask = self.pending.remove(&ask_id).expect("validated above");
        ask.timeout_abort.abort();
        self.send_to_name(
            &ask.caller,
            ServerMsg::IncomingReply {
                from: replier,
                text,
                ask_id,
                broadcast_id: ask.broadcast_id,
                thread_id: ask.thread_id,
            },
        );
    }

    fn handle_broadcast(
        &mut self,
        conn_id: u64,
        question: String,
        broadcast_id: String,
        exclude_self: Option<bool>,
    ) {
        let Some(caller_ref) = self.get_name(conn_id) else {
            self.send_to_id(
                conn_id,
                ServerMsg::Err {
                    code: ErrCode::NotRegistered,
                    message: None,
                    req_id: None,
                    ask_id: None,
                },
            );
            return;
        };
        let caller = caller_ref.to_string();
        let exclude_self = exclude_self.unwrap_or(true);
        let thread_id = broadcast_id.clone();
        let targets: Vec<String> = self
            .all_names()
            .into_iter()
            .filter(|n| !exclude_self || n != &caller)
            .collect();
        let peer_count = targets.len() as u32;
        let timeout_ms = self.default_timeout_ms;
        let cmd_tx = self.cmd_tx.clone();

        for target in targets {
            let ask_id = format!("{broadcast_id}:{target}");
            let ask_id_clone = ask_id.clone();
            let cmd_tx_clone = cmd_tx.clone();
            let timeout_task = tokio::spawn(async move {
                tokio::time::sleep(Duration::from_millis(timeout_ms)).await;
                // Broadcast timeouts don't send errors — replies just stop arriving
                let _ = cmd_tx_clone
                    .send(HubCommand::TimeoutExpired {
                        ask_id: ask_id_clone,
                    })
                    .await;
            });

            self.pending.insert(
                ask_id.clone(),
                PendingAsk {
                    caller: caller.clone(),
                    target: target.clone(),
                    broadcast_id: Some(broadcast_id.clone()),
                    thread_id: Some(thread_id.clone()),
                    timeout_abort: timeout_task.abort_handle(),
                },
            );

            self.send_to_name(
                &target,
                ServerMsg::IncomingAsk {
                    from: caller.clone(),
                    question: question.clone(),
                    ask_id,
                    broadcast_id: Some(broadcast_id.clone()),
                    thread_id: Some(thread_id.clone()),
                },
            );
        }

        self.send_to_id(
            conn_id,
            ServerMsg::BroadcastAck {
                broadcast_id,
                peer_count,
            },
        );
    }

    fn handle_timeout(&mut self, ask_id: String) {
        if let Some(ask) = self.pending.remove(&ask_id) {
            // Only send error for direct asks, not broadcast asks
            if ask.broadcast_id.is_none() {
                self.send_to_name(
                    &ask.caller,
                    ServerMsg::Err {
                        code: ErrCode::Timeout,
                        message: None,
                        req_id: None,
                        ask_id: Some(ask_id),
                    },
                );
            }
        }
        // If not found: already resolved (reply/disconnect beat the timeout), ignore
    }

    fn handle_disconnect(&mut self, conn_id: u64) {
        self.senders.remove(&conn_id);

        let Some(entry) = self.id_to_entry.remove(&conn_id) else {
            return; // Already cleaned up
        };
        let name = entry.name;
        self.name_to_id.remove(&name);

        // Collect asks targeting this peer, then resolve them as peer_gone
        let peer_gone_asks: Vec<(String, String)> = self
            .pending
            .iter()
            .filter(|(_, ask)| ask.target == name)
            .map(|(ask_id, ask)| (ask_id.clone(), ask.caller.clone()))
            .collect();

        for (ask_id, caller) in peer_gone_asks {
            if let Some(ask) = self.pending.remove(&ask_id) {
                ask.timeout_abort.abort();
                self.send_to_name(
                    &caller,
                    ServerMsg::Err {
                        code: ErrCode::PeerGone,
                        message: None,
                        req_id: None,
                        ask_id: Some(ask_id),
                    },
                );
            }
        }
    }

    fn sweep_expired_leases(&mut self) {
        let now = protocol::now_ms();
        let expired: Vec<u64> = self
            .id_to_entry
            .iter()
            .filter(|(_, e)| {
                e.lease_ms
                    .is_some_and(|ms| now.saturating_sub(e.last_seen) > ms)
            })
            .map(|(id, _)| *id)
            .collect();
        for conn_id in expired {
            self.handle_disconnect(conn_id);
        }
    }

    /// Update caller/target strings in all pending asks after a rename.
    fn update_name_on_rename(&mut self, old: &str, new: &str) {
        for ask in self.pending.values_mut() {
            if ask.caller == old {
                ask.caller = new.to_string();
            }
            if ask.target == old {
                ask.target = new.to_string();
            }
        }
    }
}

// ── Actor task ────────────────────────────────────────────────────────────────

async fn run_actor(mut cmd_rx: mpsc::Receiver<HubCommand>, cmd_tx: mpsc::Sender<HubCommand>) {
    let mut state = HubState::new(cmd_tx);
    let mut sweep = tokio::time::interval(Duration::from_secs(1));
    sweep.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    loop {
        tokio::select! {
            Some(cmd) = cmd_rx.recv() => {
                match cmd {
                    HubCommand::ClientConnected { conn_id, write_tx } => {
                        state.senders.insert(conn_id, write_tx);
                    }
                    HubCommand::ClientLine { conn_id, line } => {
                        state.handle_line(conn_id, &line);
                    }
                    HubCommand::ClientDisconnect { conn_id } => {
                        state.handle_disconnect(conn_id);
                    }
                    HubCommand::TimeoutExpired { ask_id } => {
                        state.handle_timeout(ask_id);
                    }
                    HubCommand::Shutdown => break,
                }
            }
            _ = sweep.tick() => {
                state.sweep_expired_leases();
            }
        }
    }
}

// ── Accept loop ───────────────────────────────────────────────────────────────

async fn run_accept_loop(listener: UnixListener, cmd_tx: mpsc::Sender<HubCommand>) {
    let next_conn_id = Arc::new(AtomicU64::new(1));
    loop {
        match listener.accept().await {
            Ok((socket, _)) => {
                let conn_id = next_conn_id.fetch_add(1, Ordering::Relaxed);
                let cmd_tx = cmd_tx.clone();
                tokio::spawn(async move {
                    handle_connection(socket, conn_id, cmd_tx).await;
                });
            }
            Err(e) => {
                // Listener was closed (hub shutting down)
                tracing::debug!("Relay hub accept loop exiting: {e}");
                break;
            }
        }
    }
}

async fn handle_connection(socket: UnixStream, conn_id: u64, cmd_tx: mpsc::Sender<HubCommand>) {
    let (read_half, write_half) = tokio::io::split(socket);
    let (write_tx, mut write_rx) = mpsc::channel::<ServerMsg>(64);

    // Register the write channel with the actor BEFORE starting the reader.
    // mpsc is FIFO: actor sees ClientConnected before any ClientLine from this conn.
    if cmd_tx
        .send(HubCommand::ClientConnected { conn_id, write_tx })
        .await
        .is_err()
    {
        return;
    }

    // Writer task: serialize messages and send to socket
    tokio::spawn(async move {
        let mut write_half = write_half;
        while let Some(msg) = write_rx.recv().await {
            match serde_json::to_string(&msg) {
                Ok(mut line) => {
                    line.push('\n');
                    if write_half.write_all(line.as_bytes()).await.is_err() {
                        break;
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to serialize relay message: {e}");
                }
            }
        }
    });

    // Reader: line-by-line, forward to actor
    let mut reader = BufReader::new(read_half);
    let mut line = String::new();
    loop {
        line.clear();
        match reader.read_line(&mut line).await {
            Ok(0) => break, // EOF
            Ok(n) if n > MAX_LINE_LEN => {
                tracing::warn!(
                    conn_id,
                    "Relay line too long ({n} bytes), closing connection"
                );
                break;
            }
            Ok(_) => {
                let trimmed = line.trim_end_matches(['\n', '\r']).to_string();
                if trimmed.is_empty() {
                    continue;
                }
                if cmd_tx
                    .send(HubCommand::ClientLine {
                        conn_id,
                        line: trimmed,
                    })
                    .await
                    .is_err()
                {
                    break;
                }
            }
            Err(_) => break,
        }
    }

    let _ = cmd_tx.send(HubCommand::ClientDisconnect { conn_id }).await;
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use tokio::net::UnixStream;

    // ── Test helpers ──────────────────────────────────────────────────────────

    struct TestHub {
        hub: RelayHub,
        socket_path: PathBuf,
        _dir: tempfile::TempDir,
    }

    impl TestHub {
        async fn new() -> Self {
            let dir = tempfile::tempdir().unwrap();
            let socket_path = dir.path().join("hub.sock");
            let hub = RelayHub::start(socket_path.clone()).await.unwrap();
            Self {
                hub,
                socket_path,
                _dir: dir,
            }
        }
    }

    struct TestClient {
        reader: BufReader<tokio::io::ReadHalf<UnixStream>>,
        writer: tokio::io::WriteHalf<UnixStream>,
    }

    impl TestClient {
        async fn connect(path: &Path) -> Self {
            let stream = UnixStream::connect(path).await.unwrap();
            let (r, w) = tokio::io::split(stream);
            Self {
                reader: BufReader::new(r),
                writer: w,
            }
        }

        async fn send(&mut self, msg: &ClientMsg) {
            let mut line = serde_json::to_string(msg).unwrap();
            line.push('\n');
            self.writer.write_all(line.as_bytes()).await.unwrap();
        }

        async fn recv(&mut self) -> ServerMsg {
            let mut line = String::new();
            tokio::time::timeout(Duration::from_secs(2), self.reader.read_line(&mut line))
                .await
                .expect("recv timed out")
                .expect("recv error");
            serde_json::from_str(line.trim()).expect("invalid server msg")
        }

        async fn register(&mut self, name: &str) {
            self.send(&ClientMsg::Register {
                name: name.into(),
                cwd: "/".into(),
                git_branch: "main".into(),
                protocol_version: PROTOCOL_VERSION.into(),
                lease_ms: None,
            })
            .await;
            let resp = self.recv().await;
            assert!(
                matches!(resp, ServerMsg::Ack { .. }),
                "expected ack: {resp:?}"
            );
        }
    }

    // Give the hub a tick to process between sends
    async fn tick() {
        tokio::time::sleep(Duration::from_millis(10)).await;
    }

    // ── Tests ─────────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_register_ack() {
        let th = TestHub::new().await;
        let mut c = TestClient::connect(&th.socket_path).await;
        c.send(&ClientMsg::Register {
            name: "alice".into(),
            cwd: "/".into(),
            git_branch: "main".into(),
            protocol_version: PROTOCOL_VERSION.into(),
            lease_ms: None,
        })
        .await;
        let resp = c.recv().await;
        assert!(matches!(resp, ServerMsg::Ack { .. }));
        th.hub.shutdown().await;
    }

    #[tokio::test]
    async fn test_register_duplicate_name_taken() {
        let th = TestHub::new().await;
        let mut c1 = TestClient::connect(&th.socket_path).await;
        let mut c2 = TestClient::connect(&th.socket_path).await;

        c1.register("alice").await;
        c2.send(&ClientMsg::Register {
            name: "alice".into(),
            cwd: "/".into(),
            git_branch: "main".into(),
            protocol_version: PROTOCOL_VERSION.into(),
            lease_ms: None,
        })
        .await;
        let resp = c2.recv().await;
        assert!(
            matches!(
                resp,
                ServerMsg::Err {
                    code: ErrCode::NameTaken,
                    ..
                }
            ),
            "expected name_taken: {resp:?}"
        );
        th.hub.shutdown().await;
    }

    #[tokio::test]
    async fn test_register_already_registered() {
        let th = TestHub::new().await;
        let mut c = TestClient::connect(&th.socket_path).await;
        c.register("alice").await;
        c.send(&ClientMsg::Register {
            name: "alice2".into(),
            cwd: "/".into(),
            git_branch: "main".into(),
            protocol_version: PROTOCOL_VERSION.into(),
            lease_ms: None,
        })
        .await;
        let resp = c.recv().await;
        assert!(
            matches!(
                resp,
                ServerMsg::Err {
                    code: ErrCode::AlreadyRegistered,
                    ..
                }
            ),
            "expected already_registered: {resp:?}"
        );
        th.hub.shutdown().await;
    }

    #[tokio::test]
    async fn test_protocol_version_mismatch() {
        let th = TestHub::new().await;
        let mut c = TestClient::connect(&th.socket_path).await;
        c.send(&ClientMsg::Register {
            name: "alice".into(),
            cwd: "/".into(),
            git_branch: "main".into(),
            protocol_version: "99".into(),
            lease_ms: None,
        })
        .await;
        let resp = c.recv().await;
        assert!(
            matches!(
                resp,
                ServerMsg::Err {
                    code: ErrCode::ProtocolMismatch,
                    ..
                }
            ),
            "expected protocol_mismatch: {resp:?}"
        );
        th.hub.shutdown().await;
    }

    #[tokio::test]
    async fn test_rename_updates_name() {
        let th = TestHub::new().await;
        let mut c = TestClient::connect(&th.socket_path).await;
        c.register("alice").await;

        c.send(&ClientMsg::Rename {
            new_name: "bob".into(),
            req_id: Some("r1".into()),
        })
        .await;
        let resp = c.recv().await;
        assert!(
            matches!(resp, ServerMsg::Ack { req_id: Some(ref r), .. } if r == "r1"),
            "expected ack r1: {resp:?}"
        );

        // Old name should be gone: another client can register as "alice"
        let mut c2 = TestClient::connect(&th.socket_path).await;
        c2.register("alice").await; // would fail if old name still held

        th.hub.shutdown().await;
    }

    #[tokio::test]
    async fn test_rename_to_taken_name() {
        let th = TestHub::new().await;
        let mut c1 = TestClient::connect(&th.socket_path).await;
        let mut c2 = TestClient::connect(&th.socket_path).await;
        c1.register("alice").await;
        c2.register("bob").await;

        c1.send(&ClientMsg::Rename {
            new_name: "bob".into(),
            req_id: None,
        })
        .await;
        let resp = c1.recv().await;
        assert!(
            matches!(
                resp,
                ServerMsg::Err {
                    code: ErrCode::NameTaken,
                    ..
                }
            ),
            "expected name_taken: {resp:?}"
        );
        th.hub.shutdown().await;
    }

    #[tokio::test]
    async fn test_ask_delivers_incoming_ask_to_target() {
        let th = TestHub::new().await;
        let mut sender = TestClient::connect(&th.socket_path).await;
        let mut target = TestClient::connect(&th.socket_path).await;
        sender.register("alice").await;
        target.register("bob").await;

        sender
            .send(&ClientMsg::Ask {
                to: "bob".into(),
                question: "hello?".into(),
                ask_id: "a1".into(),
                timeout_ms: Some(5000),
                thread_id: None,
            })
            .await;

        let incoming = target.recv().await;
        assert!(
            matches!(
                incoming,
                ServerMsg::IncomingAsk { ref from, ref question, ref ask_id, .. }
                if from == "alice" && question == "hello?" && ask_id == "a1"
            ),
            "unexpected: {incoming:?}"
        );
        th.hub.shutdown().await;
    }

    #[tokio::test]
    async fn test_reply_delivers_incoming_reply_to_caller() {
        let th = TestHub::new().await;
        let mut caller = TestClient::connect(&th.socket_path).await;
        let mut target = TestClient::connect(&th.socket_path).await;
        caller.register("alice").await;
        target.register("bob").await;

        caller
            .send(&ClientMsg::Ask {
                to: "bob".into(),
                question: "ping?".into(),
                ask_id: "a2".into(),
                timeout_ms: Some(5000),
                thread_id: None,
            })
            .await;

        let _ = target.recv().await; // incoming_ask

        target
            .send(&ClientMsg::Reply {
                ask_id: "a2".into(),
                text: "pong!".into(),
            })
            .await;
        tick().await;

        let reply = caller.recv().await;
        assert!(
            matches!(
                reply,
                ServerMsg::IncomingReply { ref from, ref text, ref ask_id, .. }
                if from == "bob" && text == "pong!" && ask_id == "a2"
            ),
            "unexpected: {reply:?}"
        );
        th.hub.shutdown().await;
    }

    #[tokio::test]
    async fn test_ask_to_unknown_peer_returns_peer_not_found() {
        let th = TestHub::new().await;
        let mut c = TestClient::connect(&th.socket_path).await;
        c.register("alice").await;
        c.send(&ClientMsg::Ask {
            to: "nobody".into(),
            question: "hey".into(),
            ask_id: "a3".into(),
            timeout_ms: None,
            thread_id: None,
        })
        .await;
        let resp = c.recv().await;
        assert!(
            matches!(
                resp,
                ServerMsg::Err {
                    code: ErrCode::PeerNotFound,
                    ..
                }
            ),
            "expected peer_not_found: {resp:?}"
        );
        th.hub.shutdown().await;
    }

    #[tokio::test]
    async fn test_ask_timeout_sends_err_to_caller() {
        let th = TestHub::new().await;
        let mut caller = TestClient::connect(&th.socket_path).await;
        let mut target = TestClient::connect(&th.socket_path).await;
        caller.register("alice").await;
        target.register("bob").await;

        caller
            .send(&ClientMsg::Ask {
                to: "bob".into(),
                question: "waiting...".into(),
                ask_id: "a4".into(),
                timeout_ms: Some(100), // very short
                thread_id: None,
            })
            .await;

        let _ = target.recv().await; // incoming_ask (bob doesn't reply)

        // Caller should receive timeout error after 100ms
        let err = tokio::time::timeout(Duration::from_secs(2), caller.recv())
            .await
            .expect("timeout waiting for error");
        assert!(
            matches!(err, ServerMsg::Err { code: ErrCode::Timeout, ask_id: Some(ref id), .. } if id == "a4"),
            "expected timeout: {err:?}"
        );
        th.hub.shutdown().await;
    }

    #[tokio::test]
    async fn test_peer_disconnect_sends_peer_gone_to_caller() {
        let th = TestHub::new().await;
        let mut caller = TestClient::connect(&th.socket_path).await;
        caller.register("alice").await;

        // Connect target, register, send ask, then drop target
        {
            let mut target = TestClient::connect(&th.socket_path).await;
            target.register("bob").await;
            caller
                .send(&ClientMsg::Ask {
                    to: "bob".into(),
                    question: "you there?".into(),
                    ask_id: "a5".into(),
                    timeout_ms: Some(10_000),
                    thread_id: None,
                })
                .await;
            let _ = target.recv().await; // incoming_ask
                                         // target drops here, simulating disconnect
        }

        tick().await;

        let err = tokio::time::timeout(Duration::from_secs(2), caller.recv())
            .await
            .expect("timeout waiting for peer_gone");
        assert!(
            matches!(err, ServerMsg::Err { code: ErrCode::PeerGone, ask_id: Some(ref id), .. } if id == "a5"),
            "expected peer_gone: {err:?}"
        );
        th.hub.shutdown().await;
    }

    #[tokio::test]
    async fn test_broadcast_delivers_to_all_peers() {
        let th = TestHub::new().await;
        let mut broadcaster = TestClient::connect(&th.socket_path).await;
        let mut peer1 = TestClient::connect(&th.socket_path).await;
        let mut peer2 = TestClient::connect(&th.socket_path).await;
        broadcaster.register("alice").await;
        peer1.register("bob").await;
        peer2.register("carol").await;

        broadcaster
            .send(&ClientMsg::Broadcast {
                question: "everyone?".into(),
                broadcast_id: "bc1".into(),
                exclude_self: Some(true),
            })
            .await;

        let ack = broadcaster.recv().await;
        assert!(
            matches!(ack, ServerMsg::BroadcastAck { ref broadcast_id, peer_count: 2 } if broadcast_id == "bc1"),
            "expected broadcast_ack with peer_count=2: {ack:?}"
        );

        let ask1 = peer1.recv().await;
        let ask2 = peer2.recv().await;
        assert!(
            matches!(ask1, ServerMsg::IncomingAsk { .. }),
            "peer1: {ask1:?}"
        );
        assert!(
            matches!(ask2, ServerMsg::IncomingAsk { .. }),
            "peer2: {ask2:?}"
        );
        th.hub.shutdown().await;
    }

    #[tokio::test]
    async fn test_rename_updates_pending_ask_caller() {
        let th = TestHub::new().await;
        let mut caller = TestClient::connect(&th.socket_path).await;
        let mut target = TestClient::connect(&th.socket_path).await;
        caller.register("alice").await;
        target.register("bob").await;

        caller
            .send(&ClientMsg::Ask {
                to: "bob".into(),
                question: "q".into(),
                ask_id: "a6".into(),
                timeout_ms: Some(5000),
                thread_id: None,
            })
            .await;
        let _ = target.recv().await; // incoming_ask

        // alice renames to carol
        caller
            .send(&ClientMsg::Rename {
                new_name: "carol".into(),
                req_id: None,
            })
            .await;
        let _ = caller.recv().await; // ack

        // bob replies — should reach carol (formerly alice)
        target
            .send(&ClientMsg::Reply {
                ask_id: "a6".into(),
                text: "ok".into(),
            })
            .await;
        tick().await;

        let reply = caller.recv().await;
        assert!(
            matches!(reply, ServerMsg::IncomingReply { ref from, .. } if from == "bob"),
            "expected incoming_reply: {reply:?}"
        );
        th.hub.shutdown().await;
    }

    #[tokio::test]
    async fn test_stale_socket_recovery() {
        let dir = tempfile::tempdir().unwrap();
        let socket_path = dir.path().join("hub.sock");

        // First hub
        let hub1 = RelayHub::start(socket_path.clone()).await.unwrap();
        hub1.accept_handle.abort();
        drop(hub1.cmd_tx);
        tokio::time::sleep(Duration::from_millis(50)).await;
        // Leave the socket file without unlinking it (stale)

        // Second hub should recover the stale socket and start cleanly
        let hub2 = RelayHub::start(socket_path.clone()).await.unwrap();
        let mut c = TestClient::connect(&socket_path).await;
        c.register("test").await;
        hub2.shutdown().await;
    }

    #[tokio::test]
    async fn test_bad_json_returns_bad_msg() {
        let th = TestHub::new().await;
        let mut c = TestClient::connect(&th.socket_path).await;
        // Send raw bad JSON (not a valid ClientMsg)
        c.writer.write_all(b"not json at all\n").await.unwrap();
        let resp = c.recv().await;
        assert!(
            matches!(
                resp,
                ServerMsg::Err {
                    code: ErrCode::BadMsg,
                    ..
                }
            ),
            "expected bad_msg: {resp:?}"
        );
        th.hub.shutdown().await;
    }

    #[tokio::test]
    async fn test_lease_expires_evicts_peer() {
        let th = TestHub::new().await;
        let mut c1 = TestClient::connect(&th.socket_path).await;
        let mut observer = TestClient::connect(&th.socket_path).await;

        // Register with a 150ms lease
        c1.send(&ClientMsg::Register {
            name: "shortlived".into(),
            cwd: "/".into(),
            git_branch: "main".into(),
            protocol_version: PROTOCOL_VERSION.into(),
            lease_ms: Some(150),
        })
        .await;
        assert!(matches!(c1.recv().await, ServerMsg::Ack { .. }));
        observer.register("observer").await;

        // Verify peer is visible
        observer.send(&ClientMsg::ListPeers { req_id: None }).await;
        let peers = match observer.recv().await {
            ServerMsg::Peers { peers, .. } => peers,
            other => panic!("expected peers: {other:?}"),
        };
        assert!(peers.iter().any(|p| p.name == "shortlived"));

        // Wait for sweep to evict the peer (lease = 150ms, sweep runs every 1s)
        tokio::time::sleep(Duration::from_millis(1500)).await;

        observer.send(&ClientMsg::ListPeers { req_id: None }).await;
        let peers = match observer.recv().await {
            ServerMsg::Peers { peers, .. } => peers,
            other => panic!("expected peers: {other:?}"),
        };
        assert!(
            !peers.iter().any(|p| p.name == "shortlived"),
            "evicted peer still visible: {peers:?}"
        );
        th.hub.shutdown().await;
    }

    #[tokio::test]
    async fn test_lease_resets_on_message() {
        let th = TestHub::new().await;
        let mut c1 = TestClient::connect(&th.socket_path).await;
        let mut observer = TestClient::connect(&th.socket_path).await;

        // Register with a 400ms lease
        c1.send(&ClientMsg::Register {
            name: "active".into(),
            cwd: "/".into(),
            git_branch: "main".into(),
            protocol_version: PROTOCOL_VERSION.into(),
            lease_ms: Some(400),
        })
        .await;
        assert!(matches!(c1.recv().await, ServerMsg::Ack { .. }));
        observer.register("observer").await;

        // At 300ms (within lease), send a message to reset last_seen
        tokio::time::sleep(Duration::from_millis(300)).await;
        c1.send(&ClientMsg::ListPeers { req_id: None }).await;
        let _ = c1.recv().await; // consume Peers response

        // At 600ms total: original lease would have expired (400ms), but last_seen was reset at 300ms
        // so the new deadline is 300ms + 400ms = 700ms. Peer must still be alive at 600ms.
        tokio::time::sleep(Duration::from_millis(300)).await;

        observer.send(&ClientMsg::ListPeers { req_id: None }).await;
        let peers = match observer.recv().await {
            ServerMsg::Peers { peers, .. } => peers,
            other => panic!("expected peers: {other:?}"),
        };
        assert!(
            peers.iter().any(|p| p.name == "active"),
            "peer evicted early, should still be alive: {peers:?}"
        );
        th.hub.shutdown().await;
    }

    #[tokio::test]
    async fn test_list_peers_excludes_self() {
        let th = TestHub::new().await;
        let mut c1 = TestClient::connect(&th.socket_path).await;
        let mut c2 = TestClient::connect(&th.socket_path).await;
        c1.register("alice").await;
        c2.register("bob").await;

        c1.send(&ClientMsg::ListPeers { req_id: None }).await;
        let resp = c1.recv().await;
        if let ServerMsg::Peers { peers, .. } = resp {
            assert_eq!(peers.len(), 1, "should see only 1 peer (not self)");
            assert_eq!(peers[0].name, "bob");
        } else {
            panic!("expected peers: {resp:?}");
        }
        th.hub.shutdown().await;
    }
}

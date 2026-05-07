//! Bidirectional relay session for the relay-channel binary.
//!
//! Unlike the thin `RelayClient` (which drops the read half after registration),
//! `ChannelSession` keeps a persistent background reader task and routes every
//! incoming `ServerMsg` to the appropriate waiter:
//!
//! - `Peers` / `Ack` / `Err { req_id: Some(_) }` → resolve via `req_map`
//! - `IncomingReply` / `Err { ask_id: Some(_) }` → resolve via `ask_map`
//! - `BroadcastAck` → resolve via `bcast_map`
//! - `IncomingAsk` → pushed to the caller via `incoming_ask_tx`

use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::unix::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::UnixStream;
use tokio::sync::{mpsc, oneshot};

use crate::protocol::{ClientMsg, PeerRecord, ServerMsg, PROTOCOL_VERSION};

type ReqMap = Arc<Mutex<HashMap<String, oneshot::Sender<ServerMsg>>>>;
type BcastMap = Arc<Mutex<HashMap<String, oneshot::Sender<u32>>>>;

/// A connected relay peer with full bidirectional message routing.
pub struct ChannelSession {
    name: String,
    write_tx: mpsc::Sender<ClientMsg>,
    req_map: ReqMap,
    bcast_map: BcastMap,
}

impl ChannelSession {
    /// Connect to the hub, register, and return the session plus a channel for
    /// unsolicited incoming asks (which the MCP layer forwards to the LLM).
    pub async fn connect(
        socket_path: &Path,
        name: String,
        cwd: String,
        git_branch: String,
    ) -> anyhow::Result<(Self, mpsc::Receiver<ServerMsg>)> {
        let stream = UnixStream::connect(socket_path).await.map_err(|e| {
            anyhow::anyhow!(
                "Cannot connect to relay hub at {}: {e}",
                socket_path.display()
            )
        })?;

        // Use into_split so that dropping OwnedWriteHalf shuts down the socket write
        // direction, letting the hub detect the disconnect via EOF on its reader.
        let (read_half, write_half): (OwnedReadHalf, OwnedWriteHalf) = stream.into_split();
        let (write_tx, mut write_rx) = mpsc::channel::<ClientMsg>(32);

        // Writer task
        tokio::spawn(async move {
            let mut wh = write_half;
            while let Some(msg) = write_rx.recv().await {
                if let Ok(mut line) = serde_json::to_string(&msg) {
                    line.push('\n');
                    if wh.write_all(line.as_bytes()).await.is_err() {
                        break;
                    }
                }
            }
        });

        // Send Register
        let register_msg = ClientMsg::Register {
            name: name.clone(),
            cwd,
            git_branch,
            protocol_version: PROTOCOL_VERSION.into(),
            lease_ms: None,
        };
        write_tx
            .send(register_msg)
            .await
            .map_err(|_| anyhow::anyhow!("relay write channel closed before register"))?;

        // Read registration ack
        let mut reader = BufReader::new(read_half);
        let mut line = String::new();
        tokio::time::timeout(
            std::time::Duration::from_secs(5),
            reader.read_line(&mut line),
        )
        .await
        .map_err(|_| anyhow::anyhow!("Timed out waiting for hub registration ack"))?
        .map_err(|e| anyhow::anyhow!("I/O error reading registration ack: {e}"))?;

        let resp: serde_json::Value = serde_json::from_str(line.trim())
            .map_err(|e| anyhow::anyhow!("Invalid ack from hub: {e}"))?;
        if resp.get("type").and_then(|t| t.as_str()) != Some("ack") {
            let code = resp
                .get("code")
                .and_then(|c| c.as_str())
                .unwrap_or("unknown");
            return Err(anyhow::anyhow!("Registration failed: {code}"));
        }

        let req_map: ReqMap = Arc::new(Mutex::new(HashMap::new()));
        let bcast_map: BcastMap = Arc::new(Mutex::new(HashMap::new()));
        let (incoming_ask_tx, incoming_ask_rx) = mpsc::channel::<ServerMsg>(32);

        // Background reader task: route all subsequent ServerMsg
        {
            let req_map = req_map.clone();
            let bcast_map = bcast_map.clone();
            tokio::spawn(async move {
                let mut line = String::new();
                loop {
                    line.clear();
                    match reader.read_line(&mut line).await {
                        Ok(0) | Err(_) => break,
                        Ok(_) => {}
                    }
                    let Ok(msg) = serde_json::from_str::<ServerMsg>(line.trim()) else {
                        continue;
                    };
                    route_msg(msg, &req_map, &bcast_map, &incoming_ask_tx).await;
                }
            });
        }

        Ok((
            Self {
                name,
                write_tx,
                req_map,
                bcast_map,
            },
            incoming_ask_rx,
        ))
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    /// Send an ask fire-and-forget; the reply arrives as an `IncomingReply` notification.
    pub async fn send_ask(
        &self,
        to: String,
        question: String,
        ask_id: String,
        timeout_ms: Option<u64>,
        thread_id: Option<String>,
    ) -> anyhow::Result<()> {
        self.send(ClientMsg::Ask {
            to,
            question,
            ask_id,
            timeout_ms,
            thread_id,
        })
        .await
    }

    /// List all peers registered with the hub (excluding self).
    pub async fn list_peers(&self) -> anyhow::Result<Vec<PeerRecord>> {
        let req_id = uuid::Uuid::new_v4().to_string();
        let (tx, rx) = oneshot::channel();
        self.req_map.lock().unwrap().insert(req_id.clone(), tx);
        self.send(ClientMsg::ListPeers {
            req_id: Some(req_id),
        })
        .await?;
        match tokio::time::timeout(std::time::Duration::from_secs(10), rx)
            .await
            .map_err(|_| anyhow::anyhow!("list_peers timed out"))?
            .map_err(|_| anyhow::anyhow!("list_peers channel dropped"))?
        {
            ServerMsg::Peers { peers, .. } => Ok(peers),
            ServerMsg::Err { code, message, .. } => {
                Err(anyhow::anyhow!("list_peers error: {code:?} — {message:?}"))
            }
            other => Err(anyhow::anyhow!("unexpected list_peers response: {other:?}")),
        }
    }

    /// Reply to an incoming ask (fire-and-forget).
    pub fn reply(&self, ask_id: String, text: String) {
        let write_tx = self.write_tx.clone();
        tokio::spawn(async move {
            let _ = write_tx.send(ClientMsg::Reply { ask_id, text }).await;
        });
    }

    /// Broadcast a question to all peers and return the number of peers reached.
    pub async fn broadcast(&self, question: String) -> anyhow::Result<u32> {
        let broadcast_id = uuid::Uuid::new_v4().to_string();
        self.broadcast_with_id(question, broadcast_id, None).await
    }

    /// Broadcast with a caller-supplied ID and optional `exclude_self` override; returns peer count.
    /// `exclude_self: None` delegates to the hub default (exclude self).
    pub async fn broadcast_with_id(
        &self,
        question: String,
        broadcast_id: String,
        exclude_self: Option<bool>,
    ) -> anyhow::Result<u32> {
        let (tx, rx) = oneshot::channel::<u32>();
        self.bcast_map
            .lock()
            .unwrap()
            .insert(broadcast_id.clone(), tx);
        self.send(ClientMsg::Broadcast {
            question,
            broadcast_id,
            exclude_self,
        })
        .await?;
        tokio::time::timeout(std::time::Duration::from_secs(10), rx)
            .await
            .map_err(|_| anyhow::anyhow!("broadcast timed out"))?
            .map_err(|_| anyhow::anyhow!("broadcast channel dropped"))
    }

    /// Rename this peer on the hub.
    pub async fn rename(&self, new_name: String) -> anyhow::Result<()> {
        let req_id = uuid::Uuid::new_v4().to_string();
        let (tx, rx) = oneshot::channel();
        self.req_map.lock().unwrap().insert(req_id.clone(), tx);
        self.send(ClientMsg::Rename {
            new_name,
            req_id: Some(req_id),
        })
        .await?;
        match tokio::time::timeout(std::time::Duration::from_secs(5), rx)
            .await
            .map_err(|_| anyhow::anyhow!("rename timed out"))?
            .map_err(|_| anyhow::anyhow!("rename channel dropped"))?
        {
            ServerMsg::Ack { .. } => Ok(()),
            ServerMsg::Err { code, message, .. } => {
                Err(anyhow::anyhow!("rename error: {code:?} — {message:?}"))
            }
            other => Err(anyhow::anyhow!("unexpected rename response: {other:?}")),
        }
    }

    async fn send(&self, msg: ClientMsg) -> anyhow::Result<()> {
        self.write_tx
            .send(msg)
            .await
            .map_err(|_| anyhow::anyhow!("relay write channel closed"))
    }
}

async fn route_msg(
    msg: ServerMsg,
    req_map: &ReqMap,
    bcast_map: &BcastMap,
    incoming_ask_tx: &mpsc::Sender<ServerMsg>,
) {
    match &msg {
        ServerMsg::Peers {
            req_id: Some(id), ..
        }
        | ServerMsg::Ack { req_id: Some(id) } => {
            if let Some(tx) = req_map.lock().unwrap().remove(id) {
                let _ = tx.send(msg);
            }
        }
        ServerMsg::Err {
            req_id: Some(id), ..
        } => {
            if let Some(tx) = req_map.lock().unwrap().remove(id) {
                let _ = tx.send(msg);
            }
        }
        ServerMsg::IncomingReply { .. } | ServerMsg::IncomingAsk { .. } => {
            let _ = incoming_ask_tx.send(msg).await;
        }
        ServerMsg::Err {
            ask_id: Some(_), ..
        } => {
            let _ = incoming_ask_tx.send(msg).await;
        }
        ServerMsg::BroadcastAck {
            broadcast_id,
            peer_count,
        } => {
            let id = broadcast_id.clone();
            let count = *peer_count;
            if let Some(tx) = bcast_map.lock().unwrap().remove(&id) {
                let _ = tx.send(count);
            }
        }
        // Ack/Peers/Err without correlation ID — ignore
        _ => {}
    }
}

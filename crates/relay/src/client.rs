//! Thin relay client for connecting to the hub from opr8r or the relay-channel binary.
//!
//! Handles connection, registration, and rename. Does not manage reconnection —
//! that is the caller's responsibility for long-lived use cases.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tokio::sync::mpsc;

use crate::protocol::{ClientMsg, PROTOCOL_VERSION};

/// A connected relay peer. Stays registered until `close()` is called or dropped.
pub struct RelayClient {
    socket_path: PathBuf,
    name: Arc<Mutex<String>>,
    write_tx: mpsc::Sender<String>,
}

impl RelayClient {
    /// Connect to the hub, register with the given name, and return the client.
    ///
    /// Returns an error if the hub is not reachable or registration fails.
    pub async fn connect(
        socket_path: &Path,
        name: String,
        cwd: String,
        git_branch: String,
    ) -> anyhow::Result<Self> {
        let stream = UnixStream::connect(socket_path).await.map_err(|e| {
            anyhow::anyhow!(
                "Cannot connect to relay hub at {}: {e}",
                socket_path.display()
            )
        })?;

        let (read_half, write_half) = tokio::io::split(stream);
        let (write_tx, mut write_rx) = mpsc::channel::<String>(32);

        // Writer task
        tokio::spawn(async move {
            let mut write_half = write_half;
            while let Some(line) = write_rx.recv().await {
                if write_half.write_all(line.as_bytes()).await.is_err() {
                    break;
                }
            }
        });

        let client = Self {
            socket_path: socket_path.to_path_buf(),
            name: Arc::new(Mutex::new(name.clone())),
            write_tx,
        };

        // Register
        let register_msg = ClientMsg::Register {
            name,
            cwd,
            git_branch,
            protocol_version: PROTOCOL_VERSION.into(),
            lease_ms: None,
        };
        client.send_msg(&register_msg).await?;

        // Read ack (with timeout)
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

        Ok(client)
    }

    /// Send a rename to the hub.
    pub async fn rename(&self, new_name: String) -> anyhow::Result<()> {
        let msg = ClientMsg::Rename {
            new_name: new_name.clone(),
            req_id: None,
        };
        self.send_msg(&msg).await?;
        *self.name.lock().unwrap() = new_name;
        Ok(())
    }

    /// Current registered name.
    pub fn name(&self) -> String {
        self.name.lock().unwrap().clone()
    }

    pub fn socket_path(&self) -> &Path {
        &self.socket_path
    }

    /// Close the connection gracefully (drops the write channel, hub detects disconnect).
    pub fn close(self) {
        // Dropping write_tx closes the write channel; writer task exits; hub sees EOF
        drop(self.write_tx);
    }

    async fn send_msg(&self, msg: &ClientMsg) -> anyhow::Result<()> {
        let mut line = serde_json::to_string(msg)
            .map_err(|e| anyhow::anyhow!("Failed to serialize relay message: {e}"))?;
        line.push('\n');
        self.write_tx
            .send(line)
            .await
            .map_err(|_| anyhow::anyhow!("Relay write channel closed"))?;
        Ok(())
    }
}

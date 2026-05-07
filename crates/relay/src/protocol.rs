//! Wire protocol types for the relay hub, byte-compatible with claude-relay's protocol.ts.
//!
//! All serde field names and type discriminants match the TypeScript implementation exactly
//! so that existing claude-relay channels can connect to the Rust hub unchanged.

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

pub const PROTOCOL_VERSION: &str = "2";
pub const MAX_LINE_LEN: usize = 8 * 1024 * 1024; // 8MB

/// Current time as milliseconds since Unix epoch (matches JS `Date.now()`)
pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerRecord {
    pub name: String,
    pub cwd: String,
    pub git_branch: String,
    /// Milliseconds since Unix epoch (matches JS `Date.now()`)
    pub last_seen: u64,
}

/// Messages sent from a channel client to the hub.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMsg {
    Register {
        name: String,
        cwd: String,
        git_branch: String,
        protocol_version: String,
        /// Optional TTL in milliseconds for operator-managed peers.
        /// TS clients omit this field; serde treats absence as None (infinite lease).
        #[serde(skip_serializing_if = "Option::is_none")]
        lease_ms: Option<u64>,
    },
    Rename {
        new_name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        req_id: Option<String>,
    },
    ListPeers {
        #[serde(skip_serializing_if = "Option::is_none")]
        req_id: Option<String>,
    },
    Ask {
        to: String,
        question: String,
        ask_id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        timeout_ms: Option<u64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        thread_id: Option<String>,
    },
    Reply {
        ask_id: String,
        text: String,
    },
    Broadcast {
        question: String,
        broadcast_id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        exclude_self: Option<bool>,
    },
}

/// Error codes, matching the `ErrCode` enum in claude-relay's `protocol.ts`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrCode {
    PeerNotFound,
    PeerGone,
    Timeout,
    NameTaken,
    NotRegistered,
    AlreadyRegistered,
    UnknownAsk,
    BadMsg,
    HubUnreachable,
    BadArgs,
    ProtocolMismatch,
    Unexpected,
}

/// Messages sent from the hub to channel clients.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMsg {
    Ack {
        #[serde(skip_serializing_if = "Option::is_none")]
        req_id: Option<String>,
    },
    Err {
        code: ErrCode,
        #[serde(skip_serializing_if = "Option::is_none")]
        message: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        req_id: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        ask_id: Option<String>,
    },
    Peers {
        peers: Vec<PeerRecord>,
        #[serde(skip_serializing_if = "Option::is_none")]
        req_id: Option<String>,
    },
    IncomingAsk {
        from: String,
        question: String,
        ask_id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        broadcast_id: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        thread_id: Option<String>,
    },
    IncomingReply {
        from: String,
        text: String,
        ask_id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        broadcast_id: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        thread_id: Option<String>,
    },
    BroadcastAck {
        broadcast_id: String,
        peer_count: u32,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    fn roundtrip<T: Serialize + for<'de> Deserialize<'de>>(v: &T) -> T {
        let s = serde_json::to_string(v).unwrap();
        serde_json::from_str(&s).unwrap()
    }

    #[test]
    fn test_register_type_field() {
        let msg = ClientMsg::Register {
            name: "foo".into(),
            cwd: "/".into(),
            git_branch: "main".into(),
            protocol_version: PROTOCOL_VERSION.into(),
            lease_ms: None,
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"register\""), "got: {json}");
        assert!(
            !json.contains("lease_ms"),
            "None fields omitted, got: {json}"
        );
    }

    #[test]
    fn test_rename_type_field() {
        let msg = ClientMsg::Rename {
            new_name: "bar".into(),
            req_id: Some("r1".into()),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"rename\""), "got: {json}");
        assert!(json.contains("\"req_id\":\"r1\""), "got: {json}");
    }

    #[test]
    fn test_list_peers_type_field() {
        let msg = ClientMsg::ListPeers { req_id: None };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"list_peers\""), "got: {json}");
        assert!(!json.contains("req_id"), "None omitted, got: {json}");
    }

    #[test]
    fn test_ask_type_field() {
        let msg = ClientMsg::Ask {
            to: "b".into(),
            question: "q".into(),
            ask_id: "a1".into(),
            timeout_ms: None,
            thread_id: None,
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"ask\""), "got: {json}");
    }

    #[test]
    fn test_broadcast_type_field() {
        let msg = ClientMsg::Broadcast {
            question: "q".into(),
            broadcast_id: "b1".into(),
            exclude_self: Some(true),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"broadcast\""), "got: {json}");
    }

    #[test]
    fn test_ack_type_field() {
        let msg = ServerMsg::Ack { req_id: None };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"ack\""), "got: {json}");
    }

    #[test]
    fn test_err_type_field() {
        let msg = ServerMsg::Err {
            code: ErrCode::PeerNotFound,
            message: None,
            req_id: None,
            ask_id: None,
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"err\""), "got: {json}");
        assert!(json.contains("\"code\":\"peer_not_found\""), "got: {json}");
    }

    #[test]
    fn test_incoming_ask_type_field() {
        let msg = ServerMsg::IncomingAsk {
            from: "a".into(),
            question: "q".into(),
            ask_id: "x".into(),
            broadcast_id: None,
            thread_id: None,
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"incoming_ask\""), "got: {json}");
    }

    #[test]
    fn test_broadcast_ack_type_field() {
        let msg = ServerMsg::BroadcastAck {
            broadcast_id: "b1".into(),
            peer_count: 3,
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"broadcast_ack\""), "got: {json}");
        assert!(json.contains("\"peer_count\":3"), "got: {json}");
    }

    #[test]
    fn test_all_err_codes_roundtrip() {
        let codes = [
            ErrCode::PeerNotFound,
            ErrCode::PeerGone,
            ErrCode::Timeout,
            ErrCode::NameTaken,
            ErrCode::NotRegistered,
            ErrCode::AlreadyRegistered,
            ErrCode::UnknownAsk,
            ErrCode::BadMsg,
            ErrCode::HubUnreachable,
            ErrCode::BadArgs,
            ErrCode::ProtocolMismatch,
            ErrCode::Unexpected,
        ];
        for code in &codes {
            let rt = roundtrip(code);
            assert_eq!(&rt, code);
        }
    }

    #[test]
    fn test_peer_record_last_seen_is_u64() {
        let rec = PeerRecord {
            name: "x".into(),
            cwd: "/".into(),
            git_branch: "main".into(),
            last_seen: 1_700_000_000_000,
        };
        let json = serde_json::to_string(&rec).unwrap();
        assert!(json.contains("1700000000000"), "got: {json}");
        let rt: PeerRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(rt.last_seen, 1_700_000_000_000);
    }

    #[test]
    fn test_lease_ms_omitted_for_none() {
        let msg = ClientMsg::Register {
            name: "x".into(),
            cwd: "/".into(),
            git_branch: "main".into(),
            protocol_version: PROTOCOL_VERSION.into(),
            lease_ms: None,
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(
            !json.contains("lease_ms"),
            "None lease_ms must be omitted: {json}"
        );
    }

    #[test]
    fn test_lease_ms_present_when_some() {
        let msg = ClientMsg::Register {
            name: "x".into(),
            cwd: "/".into(),
            git_branch: "main".into(),
            protocol_version: PROTOCOL_VERSION.into(),
            lease_ms: Some(30_000),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"lease_ms\":30000"), "got: {json}");
    }

    #[test]
    fn test_ts_produced_register_deserializes() {
        // Simulate a TS-produced register message (no lease_ms field)
        let raw = r#"{"type":"register","name":"my-proj","cwd":"/home/user/proj","git_branch":"main","protocol_version":"2"}"#;
        let msg: ClientMsg = serde_json::from_str(raw).unwrap();
        match msg {
            ClientMsg::Register { name, lease_ms, .. } => {
                assert_eq!(name, "my-proj");
                assert_eq!(lease_ms, None);
            }
            _ => panic!("wrong variant"),
        }
    }
}

//! Endpoint uplink wire protocol: JSON text frames over one WebSocket.
//!
//! Every frame is an internally tagged object (`"type": "<snake_case>"`). Each
//! variant wraps a named struct so the generated C# stays one record per
//! payload. Binary payloads (tunnel bodies) travel base64-encoded.

use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;

/// Major protocol version. A client whose `Hello` carries a different value is
/// refused, so wire changes that are not additive must bump this.
pub const PROTOCOL_VERSION: u32 = 1;

pub fn is_compatible(version: u32) -> bool {
    version == PROTOCOL_VERSION
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, ToSchema, TS)]
#[ts(export)]
#[serde(rename_all = "snake_case")]
pub enum GpuVendor {
    Nvidia,
    Amd,
    Intel,
    Apple,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, ToSchema, TS)]
#[ts(export)]
pub struct GpuInfo {
    pub name: String,
    pub vendor: GpuVendor,
    pub vram_total_mb: u64,
    #[serde(default)]
    pub vram_used_mb: Option<u64>,
    #[serde(default)]
    pub driver: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, ToSchema, TS)]
#[ts(export)]
pub struct LlmToolInfo {
    pub name: String,
    #[serde(default)]
    pub version: Option<String>,
}

/// A local inference server the endpoint found. `kind` is a
/// `ModelServerKind` slug; only `exposed` servers accept tunnel requests.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, ToSchema, TS)]
#[ts(export)]
pub struct ModelServerInfo {
    pub name: String,
    pub kind: String,
    pub port: u16,
    #[serde(default)]
    pub models: Vec<String>,
    pub exposed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, ToSchema, TS)]
#[ts(export)]
pub struct ProjectInfo {
    pub name: String,
    pub path: String,
}

/// What the endpoint can do. `hash` is an opaque version tag the client
/// chooses (it changes whenever anything else here changes); heartbeats echo
/// it so the server notices a change it was not sent, without either side
/// having to hash JSON identically.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, ToSchema, TS)]
#[ts(export)]
pub struct Capabilities {
    pub hash: String,
    pub hostname: String,
    pub os: String,
    pub arch: String,
    #[serde(default)]
    pub gpus: Vec<GpuInfo>,
    #[serde(default)]
    pub llm_tools: Vec<LlmToolInfo>,
    #[serde(default)]
    pub model_servers: Vec<ModelServerInfo>,
    #[serde(default)]
    pub projects: Vec<ProjectInfo>,
    pub launch_enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, ToSchema, TS)]
#[ts(export)]
pub struct Hello {
    pub protocol_version: u32,
    pub client_version: String,
    pub capabilities: Capabilities,
}

/// `capabilities_hash` echoes [`Capabilities::hash`] as last sent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, ToSchema, TS)]
#[ts(export)]
pub struct Heartbeat {
    pub seq: u64,
    pub capabilities_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, ToSchema, TS)]
#[ts(export)]
pub struct Welcome {
    pub endpoint_id: String,
    pub heartbeat_interval_secs: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, ToSchema, TS)]
#[ts(export)]
pub struct TunnelRequest {
    pub req_id: String,
    pub model_server: String,
    pub method: String,
    pub path: String,
    #[serde(default)]
    pub headers: BTreeMap<String, String>,
    pub body_b64: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, ToSchema, TS)]
#[ts(export)]
pub struct TunnelHead {
    pub req_id: String,
    pub status: u16,
    #[serde(default)]
    pub headers: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, ToSchema, TS)]
#[ts(export)]
pub struct TunnelChunk {
    pub req_id: String,
    pub data_b64: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, ToSchema, TS)]
#[ts(export)]
pub struct TunnelEnd {
    pub req_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, ToSchema, TS)]
#[ts(export)]
pub struct TunnelError {
    pub req_id: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, ToSchema, TS)]
#[ts(export)]
pub struct TunnelCancel {
    pub req_id: String,
}

/// A ticket step to run on the endpoint. `env` values are literals or shell
/// references (`${VAR}`), never resolved secrets; `callback_token` is the only
/// credential and the client must keep it in the child's environment only.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, ToSchema, TS)]
#[ts(export)]
pub struct Launch {
    pub launch_id: String,
    pub ticket_id: String,
    pub step: String,
    pub project: String,
    pub branch: String,
    pub prompt: String,
    pub llm_command: Vec<String>,
    #[serde(default)]
    pub env: BTreeMap<String, String>,
    pub api_url: String,
    pub callback_token: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, ToSchema, TS)]
#[ts(export)]
pub struct LaunchAck {
    pub launch_id: String,
    pub accepted: bool,
    #[serde(default)]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, ToSchema, TS)]
#[ts(export)]
#[serde(rename_all = "snake_case")]
pub enum LaunchState {
    Started,
    Exited,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, ToSchema, TS)]
#[ts(export)]
pub struct LaunchStatus {
    pub launch_id: String,
    pub state: LaunchState,
    #[serde(default)]
    pub exit_code: Option<i32>,
    #[serde(default)]
    pub reason: Option<String>,
    #[serde(default)]
    pub log_tail: Vec<String>,
}

/// Client to server.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, ToSchema, TS)]
#[ts(export)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum UplinkFrame {
    Hello(Hello),
    Heartbeat(Heartbeat),
    Capabilities(Capabilities),
    TunnelHead(TunnelHead),
    TunnelChunk(TunnelChunk),
    TunnelEnd(TunnelEnd),
    TunnelError(TunnelError),
    LaunchAck(LaunchAck),
    LaunchStatus(LaunchStatus),
}

impl UplinkFrame {
    #[cfg(test)]
    pub const TAGS: &'static [&'static str] = &[
        "hello",
        "heartbeat",
        "capabilities",
        "tunnel_head",
        "tunnel_chunk",
        "tunnel_end",
        "tunnel_error",
        "launch_ack",
        "launch_status",
    ];

    pub fn tag(&self) -> &'static str {
        match self {
            Self::Hello(_) => "hello",
            Self::Heartbeat(_) => "heartbeat",
            Self::Capabilities(_) => "capabilities",
            Self::TunnelHead(_) => "tunnel_head",
            Self::TunnelChunk(_) => "tunnel_chunk",
            Self::TunnelEnd(_) => "tunnel_end",
            Self::TunnelError(_) => "tunnel_error",
            Self::LaunchAck(_) => "launch_ack",
            Self::LaunchStatus(_) => "launch_status",
        }
    }
}

/// Server to client.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, ToSchema, TS)]
#[ts(export)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DownlinkFrame {
    Welcome(Welcome),
    TunnelRequest(TunnelRequest),
    TunnelCancel(TunnelCancel),
    Launch(Launch),
}

impl DownlinkFrame {
    #[cfg(test)]
    pub const TAGS: &'static [&'static str] =
        &["welcome", "tunnel_request", "tunnel_cancel", "launch"];

    #[cfg(test)]
    pub fn tag(&self) -> &'static str {
        match self {
            Self::Welcome(_) => "welcome",
            Self::TunnelRequest(_) => "tunnel_request",
            Self::TunnelCancel(_) => "tunnel_cancel",
            Self::Launch(_) => "launch",
        }
    }
}

/// Named sample frames, one or more per variant. `generate_types` writes them
/// to `bindings/endpoint/fixtures/` so the C# client can prove it reads and
/// writes the same bytes.
pub struct Fixtures {
    pub uplink: Vec<(&'static str, UplinkFrame)>,
    pub downlink: Vec<(&'static str, DownlinkFrame)>,
}

fn sample_capabilities() -> Capabilities {
    Capabilities {
        hash: "sha256:00ff".into(),
        hostname: "GAMING-PC".into(),
        os: "windows".into(),
        arch: "x86_64".into(),
        gpus: vec![
            GpuInfo {
                name: "NVIDIA GeForce RTX 3090".into(),
                vendor: GpuVendor::Nvidia,
                vram_total_mb: 24_576,
                vram_used_mb: Some(1_024),
                driver: Some("566.36".into()),
            },
            GpuInfo {
                name: "NVIDIA GeForce RTX 4060 Ti".into(),
                vendor: GpuVendor::Nvidia,
                vram_total_mb: 8_192,
                vram_used_mb: None,
                driver: None,
            },
        ],
        llm_tools: vec![
            LlmToolInfo {
                name: "claude".into(),
                version: Some("2.1.0".into()),
            },
            LlmToolInfo {
                name: "lms".into(),
                version: None,
            },
        ],
        model_servers: vec![ModelServerInfo {
            name: "lmstudio".into(),
            kind: "lmstudio".into(),
            port: 1234,
            models: vec!["qwen3.8-27b".into()],
            exposed: true,
        }],
        projects: vec![ProjectInfo {
            name: "operator".into(),
            path: r"C:\src\operator".into(),
        }],
        launch_enabled: true,
    }
}

pub fn fixtures() -> Fixtures {
    let headers = BTreeMap::from([("content-type".to_string(), "text/event-stream".to_string())]);
    Fixtures {
        uplink: vec![
            (
                "hello",
                UplinkFrame::Hello(Hello {
                    protocol_version: PROTOCOL_VERSION,
                    client_version: "0.1.0".into(),
                    capabilities: sample_capabilities(),
                }),
            ),
            (
                "heartbeat",
                UplinkFrame::Heartbeat(Heartbeat {
                    seq: 42,
                    capabilities_hash: "sha256:00ff".into(),
                }),
            ),
            (
                "capabilities_minimal",
                UplinkFrame::Capabilities(Capabilities {
                    hash: "v2".into(),
                    hostname: "arm-laptop".into(),
                    os: "windows".into(),
                    arch: "aarch64".into(),
                    gpus: vec![],
                    llm_tools: vec![],
                    model_servers: vec![],
                    projects: vec![],
                    launch_enabled: false,
                }),
            ),
            (
                "tunnel_head",
                UplinkFrame::TunnelHead(TunnelHead {
                    req_id: "r1".into(),
                    status: 200,
                    headers,
                }),
            ),
            (
                "tunnel_chunk",
                UplinkFrame::TunnelChunk(TunnelChunk {
                    req_id: "r1".into(),
                    data_b64: "ZGF0YTog+/8KCg==".into(),
                }),
            ),
            (
                "tunnel_end",
                UplinkFrame::TunnelEnd(TunnelEnd {
                    req_id: "r1".into(),
                }),
            ),
            (
                "tunnel_error",
                UplinkFrame::TunnelError(TunnelError {
                    req_id: "r2".into(),
                    reason: "model server not exposed".into(),
                }),
            ),
            (
                "launch_ack",
                UplinkFrame::LaunchAck(LaunchAck {
                    launch_id: "l1".into(),
                    accepted: false,
                    reason: Some("launch disabled".into()),
                }),
            ),
            (
                "launch_status_exited",
                UplinkFrame::LaunchStatus(LaunchStatus {
                    launch_id: "l1".into(),
                    state: LaunchState::Exited,
                    exit_code: Some(0),
                    reason: None,
                    log_tail: vec!["done".into()],
                }),
            ),
        ],
        downlink: vec![
            (
                "welcome",
                DownlinkFrame::Welcome(Welcome {
                    endpoint_id: "ep_01".into(),
                    heartbeat_interval_secs: 15,
                }),
            ),
            (
                "tunnel_request",
                DownlinkFrame::TunnelRequest(TunnelRequest {
                    req_id: "r1".into(),
                    model_server: "lmstudio".into(),
                    method: "POST".into(),
                    path: "/v1/chat/completions".into(),
                    headers: BTreeMap::from([(
                        "content-type".to_string(),
                        "application/json".to_string(),
                    )]),
                    body_b64: "e30=".into(),
                }),
            ),
            (
                "tunnel_cancel",
                DownlinkFrame::TunnelCancel(TunnelCancel {
                    req_id: "r1".into(),
                }),
            ),
            (
                "launch",
                DownlinkFrame::Launch(Launch {
                    launch_id: "l1".into(),
                    ticket_id: "FEAT-1234".into(),
                    step: "plan".into(),
                    project: "operator".into(),
                    branch: "feat/FEAT-1234".into(),
                    prompt: "Plan the change \u{2192} r\u{e9}sum\u{e9} <b>&'quotes'</b>".into(),
                    llm_command: vec!["claude".into(), "--print".into()],
                    env: BTreeMap::from([(
                        "OPENAI_API_KEY".to_string(),
                        "${OPERATOR_LM_STUDIO_TOKEN}".to_string(),
                    )]),
                    api_url: "https://operator.example/api".into(),
                    callback_token: "cb.token".into(),
                }),
            ),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn round_trip_uplink(frame: &UplinkFrame) {
        let json = serde_json::to_string(frame).unwrap();
        let back: UplinkFrame = serde_json::from_str(&json).unwrap();
        assert_eq!(&back, frame, "{json}");
    }

    fn round_trip_downlink(frame: &DownlinkFrame) {
        let json = serde_json::to_string(frame).unwrap();
        let back: DownlinkFrame = serde_json::from_str(&json).unwrap();
        assert_eq!(&back, frame, "{json}");
    }

    #[test]
    fn every_fixture_round_trips() {
        let fixtures = fixtures();
        assert!(!fixtures.uplink.is_empty() && !fixtures.downlink.is_empty());
        fixtures
            .uplink
            .iter()
            .for_each(|(_, f)| round_trip_uplink(f));
        fixtures
            .downlink
            .iter()
            .for_each(|(_, f)| round_trip_downlink(f));
    }

    #[test]
    fn fixtures_cover_every_variant() {
        let fixtures = fixtures();
        let uplink: std::collections::BTreeSet<_> =
            fixtures.uplink.iter().map(|(_, f)| f.tag()).collect();
        let downlink: std::collections::BTreeSet<_> =
            fixtures.downlink.iter().map(|(_, f)| f.tag()).collect();
        assert_eq!(uplink, UplinkFrame::TAGS.iter().copied().collect());
        assert_eq!(downlink, DownlinkFrame::TAGS.iter().copied().collect());
    }

    #[test]
    fn frames_are_tagged_by_type() {
        let json = serde_json::to_value(UplinkFrame::Heartbeat(Heartbeat {
            seq: 7,
            capabilities_hash: "abc".into(),
        }))
        .unwrap();
        assert_eq!(json["type"], "heartbeat");
        assert_eq!(json["seq"], 7);
    }

    #[test]
    fn launch_state_serializes_snake_case() {
        assert_eq!(
            serde_json::to_value(LaunchState::Exited).unwrap(),
            serde_json::json!("exited")
        );
    }

    #[test]
    fn protocol_compatibility_is_major_only() {
        assert!(is_compatible(PROTOCOL_VERSION));
        assert!(!is_compatible(PROTOCOL_VERSION + 1));
        assert!(!is_compatible(0));
    }

    #[test]
    fn fixtures_exercise_json_escaping_differences() {
        let json: String = fixtures()
            .downlink
            .iter()
            .map(|(_, f)| serde_json::to_string(f).unwrap())
            .chain(
                fixtures()
                    .uplink
                    .iter()
                    .map(|(_, f)| serde_json::to_string(f).unwrap()),
            )
            .collect();
        for needle in ["+", "<", ">", "&", "'", "\u{e9}", "\u{2192}"] {
            assert!(json.contains(needle), "no fixture carries {needle:?} raw");
        }
    }

    #[test]
    fn unknown_frame_type_is_rejected() {
        assert!(serde_json::from_str::<UplinkFrame>(r#"{"type":"nope"}"#).is_err());
    }
}

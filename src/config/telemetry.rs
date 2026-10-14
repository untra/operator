use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

const MIB: u64 = 1024 * 1024;
const DEFAULT_INGEST_MAX_BODY_BYTES: u64 = 64 * MIB;
const DEFAULT_INGEST_QUEUE_CAPACITY: usize = 1024;
const DEFAULT_WINDOW_MAX_SESSIONS: usize = 64;
const DEFAULT_WINDOW_RECORDS_PER_SESSION: usize = 2048;
const DEFAULT_WINDOW_GRACE_SECS: u64 = 300;

/// OTLP telemetry: ingest from agents, a bounded per-session window, and export upstream.
///
/// Off by default. Operator never configures a destination of its own.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export)]
#[serde(deny_unknown_fields)]
pub struct TelemetryConfig {
    /// Accept OTLP from agent sessions on `/otlp/v1/*` and inject agent OTLP env
    #[serde(default)]
    pub enabled: bool,
    /// Hash personal data and drop content and unapproved signal names, while secrets are protected in both modes.
    #[serde(default)]
    pub enable_redaction: bool,
    /// Inbound OTLP limits and agent environment injection.
    #[serde(default)]
    pub ingest: TelemetryIngestConfig,
    /// Upstream OTLP export.
    #[serde(default)]
    pub export: TelemetryExportConfig,
    /// Bounded in-memory session window.
    #[serde(default)]
    pub window: TelemetryWindowConfig,
}

/// Limits incoming OTLP requests and controls agent environment injection.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export)]
pub struct TelemetryIngestConfig {
    /// Maximum request body size, enforced both compressed and decompressed
    #[serde(default = "default_ingest_max_body_bytes")]
    pub max_body_bytes: u64,
    /// Bounded ingest queue; a full queue answers 503 with Retry-After
    #[serde(default = "default_ingest_queue_capacity")]
    pub queue_capacity: usize,
    /// Export OTLP endpoint, token and trace context into launched agent sessions
    #[serde(default = "default_true")]
    pub inject_agent_env: bool,
}

impl Default for TelemetryIngestConfig {
    fn default() -> Self {
        Self {
            max_body_bytes: default_ingest_max_body_bytes(),
            queue_capacity: default_ingest_queue_capacity(),
            inject_agent_env: true,
        }
    }
}

/// Selects the OTLP HTTP payload encoding.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export)]
pub enum OtlpProtocol {
    /// OTLP/HTTP protobuf (`http/protobuf`).
    #[default]
    #[serde(rename = "http/protobuf")]
    HttpProtobuf,
    /// OTLP/HTTP JSON (`http/json`).
    #[serde(rename = "http/json")]
    HttpJson,
}

/// Selects compression for outgoing OTLP requests.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export)]
#[serde(rename_all = "lowercase")]
pub enum OtlpCompression {
    /// gzip request compression.
    #[default]
    Gzip,
    /// Send the OTLP body uncompressed.
    None,
}

/// Describes the user-configured upstream OTLP destination.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export)]
#[serde(deny_unknown_fields)]
pub struct TelemetryExportConfig {
    /// Upstream OTLP base URL; falls back to the standard `OTEL_EXPORTER_OTLP_*` variables
    #[serde(default)]
    pub endpoint: Option<String>,
    /// OTLP encoding. Default is `http/protobuf`.
    #[serde(default)]
    pub protocol: Option<OtlpProtocol>,
    /// Name of an environment variable holding OTLP headers (`key=value,...`)
    #[serde(default)]
    pub headers_env: Option<String>,
    /// Request compression. Default is `gzip`.
    #[serde(default)]
    pub compression: Option<OtlpCompression>,
    /// Upstream export timeout in seconds (default: 10).
    #[serde(default)]
    pub timeout_secs: Option<u64>,
    /// Export Operator's own traces, logs and metrics alongside forwarded agent telemetry
    #[serde(default = "default_true")]
    pub self_telemetry: bool,
}

impl Default for TelemetryExportConfig {
    fn default() -> Self {
        Self {
            endpoint: None,
            protocol: None,
            headers_env: None,
            compression: None,
            timeout_secs: None,
            self_telemetry: true,
        }
    }
}

/// Bounds session retention in the in-memory telemetry window.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export)]
#[serde(deny_unknown_fields)]
pub struct TelemetryWindowConfig {
    /// Sessions retained; the least recently updated is evicted first
    #[serde(default = "default_window_max_sessions")]
    pub max_sessions: usize,
    /// Records retained per session; the oldest are evicted first
    #[serde(default = "default_window_records_per_session")]
    pub records_per_session: usize,
    /// Seconds an ended session stays readable before eviction
    #[serde(default = "default_window_grace_secs")]
    pub grace_secs: u64,
}

impl Default for TelemetryWindowConfig {
    fn default() -> Self {
        Self {
            max_sessions: default_window_max_sessions(),
            records_per_session: default_window_records_per_session(),
            grace_secs: default_window_grace_secs(),
        }
    }
}

fn default_true() -> bool {
    true
}

fn default_ingest_max_body_bytes() -> u64 {
    DEFAULT_INGEST_MAX_BODY_BYTES
}

fn default_ingest_queue_capacity() -> usize {
    DEFAULT_INGEST_QUEUE_CAPACITY
}

fn default_window_max_sessions() -> usize {
    DEFAULT_WINDOW_MAX_SESSIONS
}

fn default_window_records_per_session() -> usize {
    DEFAULT_WINDOW_RECORDS_PER_SESSION
}

fn default_window_grace_secs() -> u64 {
    DEFAULT_WINDOW_GRACE_SECS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_table_yields_off_by_default_with_spec_defaults() {
        let cfg: TelemetryConfig = toml::from_str("").unwrap();
        assert!(!cfg.enabled);
        assert!(!cfg.enable_redaction);
        assert_eq!(cfg.ingest.max_body_bytes, 64 * MIB);
        assert!(cfg.ingest.inject_agent_env);
        assert_eq!(cfg.export.endpoint, None);
        assert_eq!(cfg.export.protocol, None);
        assert_eq!(cfg.export.compression, None);
        assert_eq!(cfg.export.timeout_secs, None);
        assert!(cfg.export.self_telemetry);
    }

    #[test]
    fn test_protocol_uses_otel_spelling() {
        let cfg: TelemetryExportConfig = toml::from_str(r#"protocol = "http/json""#).unwrap();
        assert_eq!(cfg.protocol, Some(OtlpProtocol::HttpJson));
        assert!(toml::from_str::<TelemetryExportConfig>(r#"protocol = "grpc""#).is_err());
    }

    #[test]
    fn test_redaction_is_independent_of_telemetry_and_rejects_obsolete_settings() {
        let cfg: TelemetryConfig = toml::from_str("enable_redaction = true").unwrap();
        assert!(cfg.enable_redaction);
        assert!(!cfg.enabled);
        for (input, key) in [
            ("allow_content = true", "allow_content"),
            (
                "[export]\nmax_redaction_class = \"internal\"",
                "max_redaction_class",
            ),
            (
                "[window]\nmax_redaction_class = \"personal\"",
                "max_redaction_class",
            ),
        ] {
            let error = toml::from_str::<TelemetryConfig>(input)
                .unwrap_err()
                .to_string();
            assert!(error.contains(key), "{error}");
            assert!(error.contains("unknown field"), "{error}");
        }
    }
}

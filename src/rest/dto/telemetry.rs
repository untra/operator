//! Telemetry session window DTOs.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;

/// OTLP signal a record arrived on.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema, JsonSchema, TS,
)]
#[ts(export)]
#[serde(rename_all = "lowercase")]
pub enum TelemetrySignal {
    Traces,
    Metrics,
    Logs,
}

/// One span, log record or metric from an agent session, after the shared ingest policy.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct TelemetryRecord {
    /// Identifies the OTLP signal that supplied this record.
    pub signal: TelemetrySignal,
    /// Span name, log event name, or metric name
    pub name: String,
    /// Nanoseconds since the Unix epoch, as a decimal string (OTLP JSON convention)
    pub time_unix_nano: String,
    /// Lowercase hex trace id, when the record is trace-correlated
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
    /// Lowercase hex span id, when the record is trace-correlated
    #[serde(skip_serializing_if = "Option::is_none")]
    pub span_id: Option<String>,
    /// Record attributes merged over resource attributes
    #[schema(value_type = Object)]
    #[ts(type = "Record<string, unknown>")]
    pub attributes: serde_json::Map<String, serde_json::Value>,
    /// Log body retained with secret scrubbing only when optional redaction is disabled
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Option<Object>)]
    #[ts(type = "unknown")]
    pub body: Option<serde_json::Value>,
}

/// The bounded in-memory telemetry window for one agent session.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct TelemetrySessionWindow {
    /// Identifies the agent session whose sanitized records are retained.
    pub session_id: String,
    /// Whether the session has ended and is within its grace period
    pub ended: bool,
    /// Records evicted from this session because the ring was full
    pub evicted_records: u64,
    /// Oldest first
    pub records: Vec<TelemetryRecord>,
}

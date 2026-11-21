use serde::Serialize;
use serde_json::{json, Value};

pub const SPAN_KIND_INTERNAL: i32 = 1;
pub const STATUS_CODE_OK: i32 = 1;
pub const STATUS_CODE_ERROR: i32 = 2;
pub const SEVERITY_INFO: i32 = 9;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum AnyValue {
    StringValue(String),
    IntValue(#[serde(serialize_with = "decimal")] i64),
    BoolValue(bool),
    DoubleValue(f64),
}

fn decimal<T: std::fmt::Display, S: serde::Serializer>(
    value: &T,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&value.to_string())
}

#[derive(Debug, Clone, Serialize)]
pub struct KeyValue {
    pub key: String,
    pub value: AnyValue,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Span {
    pub trace_id: String,
    pub span_id: String,
    pub parent_span_id: String,
    pub name: String,
    pub kind: i32,
    #[serde(serialize_with = "decimal")]
    pub start_time_unix_nano: u64,
    #[serde(serialize_with = "decimal")]
    pub end_time_unix_nano: u64,
    pub attributes: Vec<KeyValue>,
    pub status: Status,
}
#[derive(Debug, Serialize)]
pub struct Status {
    pub code: i32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogRecord {
    #[serde(serialize_with = "decimal")]
    pub time_unix_nano: u64,
    pub severity_number: i32,
    pub event_name: String,
    pub trace_id: String,
    pub span_id: String,
    pub attributes: Vec<KeyValue>,
}

pub fn traces_request(resource: Vec<KeyValue>, scope: &str, spans: Vec<Span>) -> Value {
    json!({"resourceSpans": [{"resource": {"attributes": resource}, "scopeSpans": [{"scope": {"name": scope}, "spans": spans}]}]})
}
pub fn logs_request(resource: Vec<KeyValue>, scope: &str, logs: Vec<LogRecord>) -> Value {
    json!({"resourceLogs": [{"resource": {"attributes": resource}, "scopeLogs": [{"scope": {"name": scope}, "logRecords": logs}]}]})
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_ints_are_decimal_strings() {
        assert_eq!(
            serde_json::to_value(AnyValue::IntValue(i64::MAX)).unwrap(),
            json!({"intValue": i64::MAX.to_string()})
        );
    }
}

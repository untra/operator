//! OTLP/HTTP request decoding and response encoding (protobuf or JSON, optional gzip).

use std::io::Read;

use axum::http::{header, HeaderMap};
use opentelemetry_proto::tonic::collector::logs::v1::{
    ExportLogsServiceRequest, ExportLogsServiceResponse,
};
use opentelemetry_proto::tonic::collector::metrics::v1::{
    ExportMetricsServiceRequest, ExportMetricsServiceResponse,
};
use opentelemetry_proto::tonic::collector::trace::v1::{
    ExportTraceServiceRequest, ExportTraceServiceResponse,
};
use prost::Message;

use super::otlp::OtlpPayload;
use crate::rest::dto::TelemetrySignal;

pub const CONTENT_TYPE_PROTOBUF: &str = "application/x-protobuf";
pub const CONTENT_TYPE_JSON: &str = "application/json";
const ENCODING_GZIP: &str = "gzip";
const ENCODING_IDENTITY: &str = "identity";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Encoding {
    Protobuf,
    Json,
}

impl Encoding {
    pub const fn content_type(self) -> &'static str {
        match self {
            Self::Protobuf => CONTENT_TYPE_PROTOBUF,
            Self::Json => CONTENT_TYPE_JSON,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum WireError {
    UnsupportedMediaType(String),
    TooLarge,
    Malformed(String),
}

/// The request's body encoding and whether it is gzip-compressed.
pub fn negotiate(headers: &HeaderMap) -> Result<(Encoding, bool), WireError> {
    let content_type = headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default();
    let media = content_type.split(';').next().unwrap_or_default().trim();
    let encoding = if media.eq_ignore_ascii_case(CONTENT_TYPE_PROTOBUF) {
        Encoding::Protobuf
    } else if media.eq_ignore_ascii_case(CONTENT_TYPE_JSON) {
        Encoding::Json
    } else {
        return Err(WireError::UnsupportedMediaType(format!(
            "content type `{content_type}` is not {CONTENT_TYPE_PROTOBUF} or {CONTENT_TYPE_JSON}"
        )));
    };

    let content_encoding = headers
        .get(header::CONTENT_ENCODING)
        .and_then(|v| v.to_str().ok())
        .unwrap_or(ENCODING_IDENTITY)
        .trim();
    let gzip = if content_encoding.eq_ignore_ascii_case(ENCODING_GZIP) {
        true
    } else if content_encoding.eq_ignore_ascii_case(ENCODING_IDENTITY) {
        false
    } else {
        return Err(WireError::UnsupportedMediaType(format!(
            "content encoding `{content_encoding}` is not {ENCODING_GZIP}"
        )));
    };
    Ok((encoding, gzip))
}

/// Decompress with the expanded size bounded, so a small gzip bomb cannot exhaust memory.
pub fn inflate(body: &[u8], gzip: bool, max_bytes: u64) -> Result<Vec<u8>, WireError> {
    if !gzip {
        return Ok(body.to_vec());
    }
    let mut out = Vec::new();
    flate2::read::GzDecoder::new(body)
        .take(max_bytes.saturating_add(1))
        .read_to_end(&mut out)
        .map_err(|e| WireError::Malformed(format!("invalid gzip body: {e}")))?;
    if out.len() as u64 > max_bytes {
        return Err(WireError::TooLarge);
    }
    Ok(out)
}

pub fn decode(
    signal: TelemetrySignal,
    encoding: Encoding,
    bytes: &[u8],
) -> Result<OtlpPayload, WireError> {
    fn parse<T: Message + Default + serde::de::DeserializeOwned>(
        encoding: Encoding,
        bytes: &[u8],
    ) -> Result<T, WireError> {
        match encoding {
            Encoding::Protobuf => T::decode(bytes).map_err(|e| WireError::Malformed(e.to_string())),
            Encoding::Json => {
                serde_json::from_slice(bytes).map_err(|e| WireError::Malformed(e.to_string()))
            }
        }
    }
    Ok(match signal {
        TelemetrySignal::Traces => {
            OtlpPayload::Traces(parse::<ExportTraceServiceRequest>(encoding, bytes)?)
        }
        TelemetrySignal::Metrics => {
            OtlpPayload::Metrics(parse::<ExportMetricsServiceRequest>(encoding, bytes)?)
        }
        TelemetrySignal::Logs => {
            OtlpPayload::Logs(parse::<ExportLogsServiceRequest>(encoding, bytes)?)
        }
    })
}

/// A full-success export response; an absent `partial_success` means nothing was rejected.
pub fn success(signal: TelemetrySignal, encoding: Encoding) -> Vec<u8> {
    fn encode<T: Message + serde::Serialize>(encoding: Encoding, value: &T) -> Vec<u8> {
        match encoding {
            Encoding::Protobuf => value.encode_to_vec(),
            Encoding::Json => serde_json::to_vec(value).expect("OTLP response serializes"),
        }
    }
    match signal {
        TelemetrySignal::Traces => encode(encoding, &ExportTraceServiceResponse::default()),
        TelemetrySignal::Metrics => encode(encoding, &ExportMetricsServiceResponse::default()),
        TelemetrySignal::Logs => encode(encoding, &ExportLogsServiceResponse::default()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;
    use std::io::Write;

    fn headers(content_type: &str, encoding: Option<&str>) -> HeaderMap {
        let mut h = HeaderMap::new();
        h.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_str(content_type).unwrap(),
        );
        if let Some(e) = encoding {
            h.insert(header::CONTENT_ENCODING, HeaderValue::from_str(e).unwrap());
        }
        h
    }

    fn gzip(bytes: &[u8]) -> Vec<u8> {
        let mut enc = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        enc.write_all(bytes).unwrap();
        enc.finish().unwrap()
    }

    #[test]
    fn test_negotiate_accepts_protobuf_json_and_gzip() {
        assert_eq!(
            negotiate(&headers("application/x-protobuf", None)),
            Ok((Encoding::Protobuf, false))
        );
        assert_eq!(
            negotiate(&headers("application/json; charset=utf-8", Some("gzip"))),
            Ok((Encoding::Json, true))
        );
        assert!(matches!(
            negotiate(&headers("text/plain", None)),
            Err(WireError::UnsupportedMediaType(_))
        ));
        assert!(matches!(
            negotiate(&headers("application/json", Some("br"))),
            Err(WireError::UnsupportedMediaType(_))
        ));
    }

    #[test]
    fn test_inflate_bounds_the_expanded_size() {
        let body = gzip(&[b'a'; 1000]);
        assert_eq!(inflate(&body, true, 1000).unwrap().len(), 1000);
        assert_eq!(inflate(&body, true, 999), Err(WireError::TooLarge));
        assert!(matches!(
            inflate(b"not gzip", true, 1000),
            Err(WireError::Malformed(_))
        ));
    }

    #[test]
    fn test_protobuf_and_json_decode_to_the_same_payload() {
        let json = br#"{"resourceLogs":[{"resource":{"attributes":[{"key":"service.name","value":{"stringValue":"claude"}}]},"scopeLogs":[{"logRecords":[{"eventName":"claude_code.api_request","traceId":"0102030405060708090a0b0c0d0e0f10"}]}]}]}"#;
        let from_json = decode(TelemetrySignal::Logs, Encoding::Json, json).unwrap();
        let OtlpPayload::Logs(req) = &from_json else {
            unreachable!()
        };
        let from_proto = decode(
            TelemetrySignal::Logs,
            Encoding::Protobuf,
            &req.encode_to_vec(),
        )
        .unwrap();
        assert_eq!(from_json, from_proto);
        assert_eq!(
            req.resource_logs[0].scope_logs[0].log_records[0].trace_id,
            (1u8..=16).collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_malformed_body_is_rejected() {
        assert!(matches!(
            decode(TelemetrySignal::Traces, Encoding::Protobuf, &[0xff, 0xff]),
            Err(WireError::Malformed(_))
        ));
    }
}

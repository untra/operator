//! Resolve the upstream OTLP destination once, from config then standard env.

use std::fmt;
use std::time::Duration;

use crate::config::{OtlpCompression, OtlpProtocol, TelemetryExportConfig};
use crate::rest::dto::TelemetrySignal;

const ENV_SDK_DISABLED: &str = "OTEL_SDK_DISABLED";
const ENV_ENDPOINT: &str = "OTEL_EXPORTER_OTLP_ENDPOINT";
const ENV_PROTOCOL: &str = "OTEL_EXPORTER_OTLP_PROTOCOL";
const ENV_HEADERS: &str = "OTEL_EXPORTER_OTLP_HEADERS";
const ENV_TIMEOUT: &str = "OTEL_EXPORTER_OTLP_TIMEOUT";
const ENV_COMPRESSION: &str = "OTEL_EXPORTER_OTLP_COMPRESSION";

const GRPC_FALLBACK_WARNING: &str = "OTLP gRPC export is not supported; using http/protobuf";

/// Signal, path suffix, and the `OTEL_*` infix (`TRACES`, `METRICS`, `LOGS`).
const SIGNAL_PATH: [(TelemetrySignal, &str, &str); 3] = [
    (TelemetrySignal::Traces, "v1/traces", "TRACES"),
    (TelemetrySignal::Metrics, "v1/metrics", "METRICS"),
    (TelemetrySignal::Logs, "v1/logs", "LOGS"),
];

/// One upstream URL for a single signal.
#[derive(Clone, PartialEq, Eq)]
pub struct SignalEndpoint {
    pub url: String,
}

impl fmt::Debug for SignalEndpoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SignalEndpoint")
            .field("url", &self.url)
            .finish()
    }
}

/// Export destination fixed for the process lifetime.
pub struct ResolvedExport {
    pub traces: Option<SignalEndpoint>,
    pub metrics: Option<SignalEndpoint>,
    pub logs: Option<SignalEndpoint>,
    pub protocol: OtlpProtocol,
    pub compression: OtlpCompression,
    pub timeout: Duration,
    pub headers: Vec<(String, String)>,
    pub warnings: Vec<String>,
}

impl fmt::Debug for ResolvedExport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let names: Vec<&str> = self.headers.iter().map(|(name, _)| name.as_str()).collect();
        f.debug_struct("ResolvedExport")
            .field("traces", &self.traces)
            .field("metrics", &self.metrics)
            .field("logs", &self.logs)
            .field("protocol", &self.protocol)
            .field("compression", &self.compression)
            .field("timeout", &self.timeout)
            .field("header_names", &names)
            .field("warnings", &self.warnings)
            .finish()
    }
}

impl ResolvedExport {
    pub fn endpoint(&self, signal: TelemetrySignal) -> Option<&SignalEndpoint> {
        match signal {
            TelemetrySignal::Traces => self.traces.as_ref(),
            TelemetrySignal::Metrics => self.metrics.as_ref(),
            TelemetrySignal::Logs => self.logs.as_ref(),
        }
    }

    pub fn is_active(&self) -> bool {
        self.traces.is_some() || self.metrics.is_some() || self.logs.is_some()
    }
}

/// Resolve `config` against `env`. Explicit `config.endpoint` beats env URLs.
/// `OTEL_SDK_DISABLED=true` disables env-derived export only.
pub fn resolve(config: &TelemetryExportConfig, env: &dyn Fn(&str) -> Option<String>) -> ResolvedExport {
    let defaults = TelemetryExportConfig::default();
    let mut warnings = Vec::new();
    let sdk_disabled = env(ENV_SDK_DISABLED)
        .as_deref()
        .is_some_and(|v| v.eq_ignore_ascii_case("true"));
    let config_endpoint = config.endpoint.as_deref().filter(|v| !v.is_empty());
    let env_export_allowed = config_endpoint.is_some() || !sdk_disabled;

    let base_env = env(ENV_ENDPOINT).filter(|v| !v.is_empty());
    let mut traces = None;
    let mut metrics = None;
    let mut logs = None;
    if env_export_allowed {
        for (signal, suffix, infix) in SIGNAL_PATH {
            if exporter_disabled(env, infix) {
                continue;
            }
            let url = endpoint_for(config_endpoint, env, suffix, infix, base_env.as_deref());
            let slot = match signal {
                TelemetrySignal::Traces => &mut traces,
                TelemetrySignal::Metrics => &mut metrics,
                TelemetrySignal::Logs => &mut logs,
            };
            *slot = url.map(|url| SignalEndpoint { url });
        }
    }

    let protocol = if config_endpoint.is_some() {
        config.protocol
    } else {
        protocol_from_env(env(ENV_PROTOCOL).as_deref(), &mut warnings)
    };

    let headers = parse_headers(config, env, &mut warnings);

    let timeout = if config.timeout_secs == defaults.timeout_secs {
        timeout_from_env(env(ENV_TIMEOUT).as_deref(), config.timeout_secs, &mut warnings)
    } else {
        Duration::from_secs(config.timeout_secs)
    };

    let compression = if config.compression == defaults.compression {
        compression_from_env(env(ENV_COMPRESSION).as_deref(), &mut warnings)
    } else {
        config.compression
    };

    ResolvedExport {
        traces,
        metrics,
        logs,
        protocol,
        compression,
        timeout,
        headers,
        warnings,
    }
}

pub fn resolve_from_process(config: &TelemetryExportConfig) -> ResolvedExport {
    resolve(config, &|key| std::env::var(key).ok())
}

fn exporter_disabled(env: &dyn Fn(&str) -> Option<String>, infix: &str) -> bool {
    let name = format!("OTEL_{infix}_EXPORTER");
    env(&name)
        .as_deref()
        .is_some_and(|v| v.eq_ignore_ascii_case("none"))
}

fn endpoint_for(
    config_endpoint: Option<&str>,
    env: &dyn Fn(&str) -> Option<String>,
    suffix: &str,
    infix: &str,
    base_env: Option<&str>,
) -> Option<String> {
    if let Some(base) = config_endpoint {
        return Some(join_signal(base, suffix));
    }
    let specific = format!("OTEL_EXPORTER_OTLP_{infix}_ENDPOINT");
    if let Some(url) = env(&specific).filter(|v| !v.is_empty()) {
        return Some(url);
    }
    base_env.map(|base| join_signal(base, suffix))
}

fn join_signal(base: &str, suffix: &str) -> String {
    format!("{}/{}", base.trim().trim_end_matches('/'), suffix)
}

fn protocol_from_env(raw: Option<&str>, warnings: &mut Vec<String>) -> OtlpProtocol {
    match raw.map(str::trim) {
        None | Some("") | Some("http/protobuf") => OtlpProtocol::HttpProtobuf,
        Some("http/json") => OtlpProtocol::HttpJson,
        Some("grpc") => {
            warnings.push(GRPC_FALLBACK_WARNING.to_string());
            OtlpProtocol::HttpProtobuf
        }
        Some(other) => {
            warnings.push(format!(
                "unrecognized OTLP protocol `{other}`; using http/protobuf"
            ));
            OtlpProtocol::HttpProtobuf
        }
    }
}

fn timeout_from_env(raw: Option<&str>, default_secs: u64, warnings: &mut Vec<String>) -> Duration {
    let Some(raw) = raw.map(str::trim).filter(|v| !v.is_empty()) else {
        return Duration::from_secs(default_secs);
    };
    match raw.parse::<u64>() {
        Ok(ms) => Duration::from_millis(ms),
        Err(_) => {
            warnings.push("OTLP timeout is not a millisecond count; using the configured timeout".into());
            Duration::from_secs(default_secs)
        }
    }
}

fn compression_from_env(raw: Option<&str>, warnings: &mut Vec<String>) -> OtlpCompression {
    let Some(raw) = raw.map(str::trim).filter(|value| !value.is_empty()) else {
        return OtlpCompression::Gzip;
    };
    match raw.to_ascii_lowercase().as_str() {
        "gzip" => OtlpCompression::Gzip,
        "none" => OtlpCompression::None,
        other => {
            warnings.push(format!("unrecognized OTLP compression `{other}`; using gzip"));
            OtlpCompression::Gzip
        }
    }
}

fn parse_headers(
    config: &TelemetryExportConfig,
    env: &dyn Fn(&str) -> Option<String>,
    warnings: &mut Vec<String>,
) -> Vec<(String, String)> {
    let raw = if let Some(name) = config.headers_env.as_deref().filter(|v| !v.is_empty()) {
        match env(name) {
            Some(raw) => raw,
            None => {
                warnings.push(format!("OTLP headers environment variable `{name}` is unset"));
                return Vec::new();
            }
        }
    } else {
        match env(ENV_HEADERS) {
            Some(raw) => raw,
            None => return Vec::new(),
        }
    };
    let mut headers = Vec::new();
    for part in raw.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        let Some((key, value)) = part.split_once('=') else {
            warnings.push("skipped malformed OTLP header".into());
            continue;
        };
        let key = key.trim();
        if key.is_empty() {
            warnings.push("skipped malformed OTLP header".into());
            continue;
        }
        headers.push((key.to_string(), percent_decode(value.trim())));
    }
    headers
}

fn percent_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Some(decoded) = std::str::from_utf8(&bytes[i + 1..i + 3])
                .ok()
                .and_then(|hex| u8::from_str_radix(hex, 16).ok())
            {
                out.push(decoded);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn env(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> {
        let map: HashMap<String, String> = pairs.iter().map(|(k, v)| ((*k).into(), (*v).into())).collect();
        move |k| map.get(k).cloned()
    }

    #[test]
    fn test_off_without_config_or_env() {
        assert!(!resolve(&TelemetryExportConfig::default(), &env(&[])).is_active());
    }

    #[test]
    fn test_base_env_endpoint_appends_signal_paths() {
        let r = resolve(
            &TelemetryExportConfig::default(),
            &env(&[("OTEL_EXPORTER_OTLP_ENDPOINT", "http://c:4318/")]),
        );
        assert_eq!(
            r.endpoint(TelemetrySignal::Traces).unwrap().url,
            "http://c:4318/v1/traces"
        );
        assert_eq!(
            r.endpoint(TelemetrySignal::Logs).unwrap().url,
            "http://c:4318/v1/logs"
        );
    }

    #[test]
    fn test_signal_env_endpoint_is_used_as_is_and_beats_base() {
        let r = resolve(
            &TelemetryExportConfig::default(),
            &env(&[
                ("OTEL_EXPORTER_OTLP_ENDPOINT", "http://base"),
                ("OTEL_EXPORTER_OTLP_METRICS_ENDPOINT", "http://m/custom"),
            ]),
        );
        assert_eq!(
            r.endpoint(TelemetrySignal::Metrics).unwrap().url,
            "http://m/custom"
        );
        assert_eq!(
            r.endpoint(TelemetrySignal::Traces).unwrap().url,
            "http://base/v1/traces"
        );
    }

    #[test]
    fn test_config_endpoint_beats_env() {
        let cfg = TelemetryExportConfig {
            endpoint: Some("http://cfg".into()),
            ..Default::default()
        };
        let r = resolve(
            &cfg,
            &env(&[("OTEL_EXPORTER_OTLP_TRACES_ENDPOINT", "http://env")]),
        );
        assert_eq!(
            r.endpoint(TelemetrySignal::Traces).unwrap().url,
            "http://cfg/v1/traces"
        );
    }

    #[test]
    fn test_exporter_none_disables_one_signal() {
        let r = resolve(
            &TelemetryExportConfig::default(),
            &env(&[
                ("OTEL_EXPORTER_OTLP_ENDPOINT", "http://c"),
                ("OTEL_LOGS_EXPORTER", "none"),
            ]),
        );
        assert!(r.endpoint(TelemetrySignal::Logs).is_none());
        assert!(r.endpoint(TelemetrySignal::Traces).is_some());
    }

    #[test]
    fn test_exporter_none_disables_a_configured_endpoint() {
        let cfg = TelemetryExportConfig {
            endpoint: Some("http://cfg".into()),
            ..Default::default()
        };
        let r = resolve(&cfg, &env(&[("OTEL_TRACES_EXPORTER", "none")]));
        assert!(r.endpoint(TelemetrySignal::Traces).is_none());
        assert!(r.endpoint(TelemetrySignal::Logs).is_some());
    }

    #[test]
    fn test_sdk_disabled_turns_env_export_off() {
        let r = resolve(
            &TelemetryExportConfig::default(),
            &env(&[
                ("OTEL_EXPORTER_OTLP_ENDPOINT", "http://c"),
                ("OTEL_SDK_DISABLED", "TRUE"),
            ]),
        );
        assert!(!r.is_active());
    }

    #[test]
    fn test_sdk_disabled_keeps_explicit_config_endpoint() {
        let cfg = TelemetryExportConfig {
            endpoint: Some("http://cfg".into()),
            ..Default::default()
        };
        let r = resolve(&cfg, &env(&[("OTEL_SDK_DISABLED", "true")]));
        assert_eq!(
            r.endpoint(TelemetrySignal::Traces).unwrap().url,
            "http://cfg/v1/traces"
        );
    }

    #[test]
    fn test_grpc_warns_and_falls_back_to_protobuf() {
        let r = resolve(
            &TelemetryExportConfig::default(),
            &env(&[
                ("OTEL_EXPORTER_OTLP_ENDPOINT", "http://c"),
                ("OTEL_EXPORTER_OTLP_PROTOCOL", "grpc"),
            ]),
        );
        assert_eq!(r.protocol, OtlpProtocol::HttpProtobuf);
        assert_eq!(r.warnings.len(), 1);
    }

    #[test]
    fn test_headers_are_parsed_decoded_and_never_debug_printed() {
        let r = resolve(
            &TelemetryExportConfig::default(),
            &env(&[
                ("OTEL_EXPORTER_OTLP_ENDPOINT", "http://c"),
                (
                    "OTEL_EXPORTER_OTLP_HEADERS",
                    "Authorization=Bearer%20s3cret,x-team=a",
                ),
            ]),
        );
        assert_eq!(r.headers[0], ("Authorization".into(), "Bearer s3cret".into()));
        assert!(!format!("{r:?}").contains("s3cret"));
    }

    #[test]
    fn test_headers_env_indirection_wins_over_standard_variable() {
        let cfg = TelemetryExportConfig {
            headers_env: Some("MY_HDRS".into()),
            ..Default::default()
        };
        let r = resolve(
            &cfg,
            &env(&[
                ("OTEL_EXPORTER_OTLP_ENDPOINT", "http://c"),
                ("MY_HDRS", "k=1"),
                ("OTEL_EXPORTER_OTLP_HEADERS", "k=2"),
            ]),
        );
        assert_eq!(r.headers, vec![("k".into(), "1".into())]);
    }

    #[test]
    fn test_timeout_env_is_milliseconds() {
        let r = resolve(
            &TelemetryExportConfig::default(),
            &env(&[
                ("OTEL_EXPORTER_OTLP_ENDPOINT", "http://c"),
                ("OTEL_EXPORTER_OTLP_TIMEOUT", "2500"),
            ]),
        );
        assert_eq!(r.timeout, Duration::from_millis(2500));
    }

    #[test]
    fn test_explicit_timeout_beats_env() {
        let cfg = TelemetryExportConfig {
            timeout_secs: 4,
            ..Default::default()
        };
        let r = resolve(&cfg, &env(&[("OTEL_EXPORTER_OTLP_TIMEOUT", "2500")]));
        assert_eq!(r.timeout, Duration::from_secs(4));
    }
}

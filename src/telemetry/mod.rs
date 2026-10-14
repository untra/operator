//! OTLP ingest from agent sessions: sanitize, queue, and keep a bounded per-session window.
//!
//! Trusted enrichment and one policy pass run before queueing; every sink consumes that sanitized payload.

pub mod export_config;
pub mod forward;
pub mod otlp;
pub mod self_export;
pub mod sink;
pub mod window;
pub mod wire;

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

use operator_telemetry::{RedactionPolicy, Redactor};
use tokio::sync::mpsc;

use crate::config::TelemetryConfig;
use crate::rest::dto::{TelemetrySessionWindow, TelemetrySignal};

pub const fn signal_index(signal: TelemetrySignal) -> usize {
    match signal {
        TelemetrySignal::Traces => 0,
        TelemetrySignal::Metrics => 1,
        TelemetrySignal::Logs => 2,
    }
}
use otlp::{OtlpPayload, TrustedContext};
use window::{SessionWindow, WindowLimits};

/// Routes under this prefix are OTLP, never the SPA bundle.
pub const OTLP_PATH_PREFIX: &str = "/otlp/";

/// Seconds a client should wait after a 503 from a full ingest queue.
pub const INGEST_RETRY_AFTER_SECS: u64 = 1;

/// A sanitized payload awaiting the sinks.
#[derive(Debug)]
pub struct IngestBatch {
    session_id: Option<String>,
    payload: OtlpPayload,
}

#[derive(Debug, Default)]
pub struct TelemetryStats {
    received: [AtomicU64; 3],
    dropped_queue_full: [AtomicU64; 3],
    redacted: [[AtomicU64; 3]; 3],
}

impl TelemetryStats {
    pub fn received(&self, signal: TelemetrySignal) -> u64 {
        self.received[signal_index(signal)].load(Ordering::Relaxed)
    }
    pub fn dropped_queue_full(&self, signal: TelemetrySignal) -> u64 {
        self.dropped_queue_full[signal_index(signal)].load(Ordering::Relaxed)
    }
    #[cfg(test)]
    pub fn redacted(&self, signal: TelemetrySignal) -> u64 {
        self.redacted[signal_index(signal)]
            .iter()
            .map(|v| v.load(Ordering::Relaxed))
            .sum()
    }
    pub fn redaction_counts(&self, signal: TelemetrySignal) -> [u64; 3] {
        std::array::from_fn(|i| self.redacted[signal_index(signal)][i].load(Ordering::Relaxed))
    }
    pub fn record_redaction(
        &self,
        signal: TelemetrySignal,
        stats: operator_telemetry::RedactionStats,
    ) {
        for (slot, value) in self.redacted[signal_index(signal)].iter().zip([
            stats.dropped,
            stats.hashed,
            stats.scrubbed,
        ]) {
            slot.fetch_add(value, Ordering::Relaxed);
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct QueueFull;

pub struct TelemetryHub {
    sink: Option<Arc<dyn sink::ExportSink>>,
    redactor: Arc<Redactor>,
    policy: RedactionPolicy,
    window: Arc<Mutex<SessionWindow>>,
    queue_capacity: usize,
    queue: OnceLock<mpsc::Sender<IngestBatch>>,
    pub stats: Arc<TelemetryStats>,
}

impl TelemetryHub {
    /// Policies and capacities are fixed here; changing them takes a restart.
    pub fn new(
        config: &TelemetryConfig,
        key: &[u8; 32],
        sink: Option<Arc<dyn sink::ExportSink>>,
    ) -> anyhow::Result<Self> {
        Ok(Self {
            sink,
            redactor: Arc::new(Redactor::new(key)?),
            policy: RedactionPolicy {
                enable_redaction: config.enable_redaction,
            },
            window: Arc::new(Mutex::new(SessionWindow::new(WindowLimits {
                max_sessions: config.window.max_sessions,
                records_per_session: config.window.records_per_session,
                grace: Duration::from_secs(config.window.grace_secs),
            }))),
            queue_capacity: config.ingest.queue_capacity.max(1),
            queue: OnceLock::new(),
            stats: Arc::new(TelemetryStats::default()),
        })
    }

    /// Replace client correlation with the trusted context, then apply the ingest policy.
    pub fn sanitize(
        &self,
        mut payload: OtlpPayload,
        context: Option<&TrustedContext>,
    ) -> IngestBatch {
        payload.enrich(context);
        let signal = payload.signal();
        let count = payload.record_count();
        let (payload, stats) = payload.redacted(&self.redactor, self.policy);
        self.stats.received[signal_index(signal)].fetch_add(count, Ordering::Relaxed);
        self.stats.record_redaction(signal, stats);
        IngestBatch {
            session_id: payload.session_id(),
            payload,
        }
    }

    /// Queue without waiting; a full queue is the caller's 503.
    ///
    /// The worker starts on first use so it runs on the serving runtime.
    pub fn enqueue(&self, batch: IngestBatch) -> Result<(), QueueFull> {
        let sender = self.queue.get_or_init(|| self.spawn_worker());
        sender.try_send(batch).map_err(|error| {
            let payload = error.into_inner().payload;
            self.stats.dropped_queue_full[signal_index(payload.signal())]
                .fetch_add(payload.record_count(), Ordering::Relaxed);
            QueueFull
        })
    }

    pub fn session_window(&self, session_id: &str) -> Option<TelemetrySessionWindow> {
        lock(&self.window).get(session_id, Instant::now())
    }

    /// Begin the session's grace period, e.g. when its step completes.
    pub fn end_session(&self, session_id: &str) {
        lock(&self.window).end(session_id, Instant::now());
    }

    fn spawn_worker(&self) -> mpsc::Sender<IngestBatch> {
        let (tx, mut rx) = mpsc::channel::<IngestBatch>(self.queue_capacity);
        let window = Arc::clone(&self.window);
        let sink = self.sink.clone();
        tokio::spawn(async move {
            while let Some(batch) = rx.recv().await {
                deliver_to_window(&window, &batch);
                if let Some(sink) = &sink {
                    sink.submit(batch.payload);
                }
            }
        });
        tx
    }
}

fn deliver_to_window(window: &Mutex<SessionWindow>, batch: &IngestBatch) {
    let Some(session_id) = &batch.session_id else {
        return;
    };
    lock(window).insert(session_id, window::records(&batch.payload), Instant::now());
}

/// A poisoned window only means a panic mid-insert; its data is still bounded and usable.
fn lock(window: &Mutex<SessionWindow>) -> std::sync::MutexGuard<'_, SessionWindow> {
    window
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

#[cfg(test)]
mod tests {
    use super::*;
    use opentelemetry_proto::tonic::collector::logs::v1::ExportLogsServiceRequest;
    use opentelemetry_proto::tonic::common::v1::{any_value, AnyValue, KeyValue};
    use opentelemetry_proto::tonic::logs::v1::{LogRecord, ResourceLogs, ScopeLogs};
    use operator_telemetry::attr;

    fn kv(key: &str, value: &str) -> KeyValue {
        KeyValue {
            key: key.to_string(),
            value: Some(AnyValue {
                value: Some(any_value::Value::StringValue(value.to_string())),
            }),
            ..KeyValue::default()
        }
    }

    fn logs(attributes: Vec<KeyValue>) -> OtlpPayload {
        OtlpPayload::Logs(ExportLogsServiceRequest {
            resource_logs: vec![ResourceLogs {
                scope_logs: vec![ScopeLogs {
                    log_records: vec![LogRecord {
                        event_name: "operator.step.started".to_string(),
                        attributes,
                        ..LogRecord::default()
                    }],
                    ..ScopeLogs::default()
                }],
                ..ResourceLogs::default()
            }],
        })
    }

    fn context(session: &str) -> TrustedContext {
        TrustedContext {
            ticket_id: Some("FEAT-1".into()),
            step: Some("build".into()),
            session_id: Some(session.into()),
            profile_id: None,
        }
    }

    fn hub(config: &TelemetryConfig) -> TelemetryHub {
        TelemetryHub::new(config, &[3; 32], None).unwrap()
    }

    #[tokio::test]
    async fn test_sanitized_payload_reaches_sink_and_window() {
        #[derive(Default)]
        struct RecordingSink(Mutex<Vec<OtlpPayload>>);
        impl sink::ExportSink for RecordingSink {
            fn submit(&self, payload: OtlpPayload) {
                self.0.lock().unwrap().push(payload);
            }
        }
        let sink = Arc::new(RecordingSink::default());
        let hub = TelemetryHub::new(
            &TelemetryConfig::default(),
            &[3; 32],
            Some(Arc::clone(&sink) as Arc<dyn sink::ExportSink>),
        )
        .unwrap();
        hub.enqueue(hub.sanitize(
            logs(vec![kv(attr::OPERATOR_AUTH_TOKEN, "secret")]),
            Some(&context("s")),
        ))
        .unwrap();
        tokio::time::timeout(Duration::from_secs(2), async {
            while sink.0.lock().unwrap().is_empty() {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        assert!(hub.session_window("s").is_some());
        assert!(!format!("{:?}", sink.0.lock().unwrap()[0]).contains("secret"));
        assert_eq!(hub.stats.received(TelemetrySignal::Logs), 1);
    }

    #[tokio::test]
    async fn test_ingest_reaches_window_with_secret_protection_when_redaction_disabled() {
        let hub = hub(&TelemetryConfig::default());
        let batch = hub.sanitize(
            logs(vec![
                kv(attr::OPERATOR_SESSION_ID, "spoofed"),
                kv("user.name", "alice"),
                kv(attr::OPERATOR_AUTH_TOKEN, "secret"),
            ]),
            Some(&context("sess-1")),
        );
        assert_eq!(batch.session_id.as_deref(), Some("sess-1"));
        hub.enqueue(batch).unwrap();

        let window = loop {
            if let Some(w) = hub.session_window("sess-1") {
                break w;
            }
            tokio::task::yield_now().await;
        };
        assert!(hub.session_window("spoofed").is_none());
        let attrs = &window.records[0].attributes;
        assert_eq!(
            attrs["user.name"], "alice",
            "optional redaction is disabled by default"
        );
        assert!(!attrs.contains_key(attr::OPERATOR_AUTH_TOKEN));
        assert_eq!(attrs[attr::OPERATOR_TICKET_ID], "FEAT-1");
    }

    #[tokio::test]
    async fn test_personal_data_is_hashed_once_before_queue_and_window() {
        let config = TelemetryConfig {
            enable_redaction: true,
            ..TelemetryConfig::default()
        };
        let hub = hub(&config);
        let batch = hub.sanitize(
            logs(vec![kv("user.name", "alice"), kv("unknown", "private")]),
            Some(&context("s")),
        );
        let before = window::records(&batch.payload);
        let hash = before[0].attributes["user.name"].clone();
        assert!(hash.as_str().unwrap().starts_with("hmac:"));
        assert!(!before[0].attributes.contains_key("unknown"));
        assert_eq!(before[0].attributes[attr::OPERATOR_REDACTED_COUNT], 2);
        hub.enqueue(batch).unwrap();
        let window = tokio::time::timeout(Duration::from_secs(2), async {
            loop {
                if let Some(w) = hub.session_window("s") {
                    break w;
                }
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        assert_eq!(window.records[0].attributes["user.name"], hash);
        assert_eq!(
            window.records[0].attributes[attr::OPERATOR_REDACTED_COUNT],
            2
        );
        assert_eq!(hub.stats.redacted(TelemetrySignal::Logs), 2);
    }

    #[tokio::test]
    async fn test_full_queue_rejects_without_blocking_and_counts_the_drop() {
        let mut config = TelemetryConfig::default();
        config.ingest.queue_capacity = 1;
        let hub = hub(&config);
        let (tx, _rx) = mpsc::channel(1);
        hub.queue.set(tx).unwrap();

        let first = hub.sanitize(logs(vec![]), Some(&context("s")));
        let second = hub.sanitize(logs(vec![]), Some(&context("s")));
        hub.enqueue(first).unwrap();
        assert_eq!(hub.enqueue(second), Err(QueueFull));
        assert_eq!(hub.stats.dropped_queue_full(TelemetrySignal::Logs), 1);
    }
}

pub mod events;

pub mod metrics;

pub mod agent_env;

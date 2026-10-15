use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant};

use opentelemetry::logs::LoggerProvider;
use opentelemetry::metrics::MeterProvider;
use opentelemetry::trace::TracerProvider;
use opentelemetry::KeyValue;
use opentelemetry_proto::tonic::collector::{
    logs::v1::ExportLogsServiceRequest, trace::v1::ExportTraceServiceRequest,
};
use opentelemetry_proto::transform::{
    common::tonic::ResourceAttributesWithSchema, logs::tonic::group_logs_by_resource_and_scope,
    trace::tonic::group_spans_by_resource_and_scope,
};
use opentelemetry_sdk::{
    error::OTelSdkResult,
    logs::{LogBatch, LogExporter, SdkLogger, SdkLoggerProvider},
    metrics::{
        data::ResourceMetrics, exporter::PushMetricExporter, PeriodicReader, SdkMeterProvider,
        Temporality,
    },
    trace::{SdkTracerProvider, SpanData, SpanExporter},
    Resource,
};
use operator_telemetry::{RedactionPolicy, Redactor};
use tracing_subscriber::{layer::Filter, registry::LookupSpan, Layer};

use super::{
    export_config,
    forward::{Forwarder, ForwarderLimits},
    otlp::OtlpPayload,
    sink::ExportSink,
    TelemetryStats,
};
use crate::{auth::store::AuthStore, config::Config};

static ACTIVE: OnceLock<SelfExport> = OnceLock::new();
pub const EXCLUDED_TARGETS: &[&str] = &[
    "reqwest",
    "hyper",
    "hyper_util",
    "h2",
    "rustls",
    "opentelemetry",
    "operator::telemetry::forward",
];

pub fn active() -> Option<&'static SelfExport> {
    ACTIVE.get()
}

struct RedactingExporter {
    sink: Arc<dyn ExportSink>,
    redactor: Arc<Redactor>,
    policy: RedactionPolicy,
    resource: ResourceAttributesWithSchema,
    stats: Arc<TelemetryStats>,
}

impl std::fmt::Debug for RedactingExporter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("RedactingExporter")
    }
}

impl RedactingExporter {
    fn submit(&self, payload: OtlpPayload) {
        let signal = payload.signal();
        let (payload, stats) = payload.redacted(&self.redactor, self.policy);
        self.stats.record_redaction(signal, stats);
        self.sink.submit(payload);
    }
}

impl SpanExporter for RedactingExporter {
    async fn export(&self, batch: Vec<SpanData>) -> OTelSdkResult {
        self.submit(OtlpPayload::Traces(ExportTraceServiceRequest {
            resource_spans: group_spans_by_resource_and_scope(batch, &self.resource),
        }));
        Ok(())
    }
    fn set_resource(&mut self, resource: &Resource) {
        self.resource = resource.into();
    }
}

impl LogExporter for RedactingExporter {
    async fn export(&self, batch: LogBatch<'_>) -> OTelSdkResult {
        self.submit(OtlpPayload::Logs(ExportLogsServiceRequest {
            resource_logs: group_logs_by_resource_and_scope(&batch, &self.resource),
        }));
        Ok(())
    }
    fn set_resource(&mut self, resource: &Resource) {
        self.resource = resource.into();
    }
}

impl PushMetricExporter for RedactingExporter {
    async fn export(&self, metrics: &ResourceMetrics) -> OTelSdkResult {
        self.submit(OtlpPayload::Metrics(metrics.into()));
        Ok(())
    }
    fn force_flush(&self) -> OTelSdkResult {
        Ok(())
    }
    fn shutdown_with_timeout(&self, _: Duration) -> OTelSdkResult {
        Ok(())
    }
    fn temporality(&self) -> Temporality {
        Temporality::Cumulative
    }
}

pub struct SelfExport {
    tracer: Option<SdkTracerProvider>,
    logger_provider: Option<SdkLoggerProvider>,
    logger: Option<SdkLogger>,
    meter: Option<SdkMeterProvider>,
    forwarder: Arc<Forwarder>,
    timeout: Duration,
    stopped: std::sync::atomic::AtomicBool,
    pub stats: Arc<TelemetryStats>,
}

impl SelfExport {
    pub fn start(config: &Config) -> anyhow::Result<Option<&'static Self>> {
        if let Some(active) = active() {
            return Ok(Some(active));
        }
        let export = export_config::resolve_from_process(&config.telemetry.export);
        for warning in &export.warnings {
            eprintln!("telemetry: {warning}");
        }
        let sdk =
            export_config::sdk_enabled(&config.telemetry.export, &|key| std::env::var(key).ok());
        if !export.is_active() || !(sdk || config.telemetry.enabled) {
            return Ok(None);
        }
        let key = AuthStore::open(&config.auth_state_path())?.load_or_create_telemetry_key()?;
        let redactor = Arc::new(Redactor::new(&key)?);
        let traces = sdk && export.traces.is_some();
        let logs = sdk && export.logs.is_some();
        let metrics = sdk && export.metrics.is_some();
        let timeout = export.timeout;
        let forwarder = Forwarder::start(export, ForwarderLimits::default())?;
        let stats = Arc::new(TelemetryStats::default());
        let resource = Resource::builder()
            .with_service_name(
                std::env::var("OTEL_SERVICE_NAME").unwrap_or_else(|_| "operator".into()),
            )
            .with_attributes([
                KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
                KeyValue::new(
                    operator_telemetry::attr::OPERATOR_PROFILE_ID,
                    config.profile.id.to_string(),
                ),
            ])
            .build();
        let exporter = || RedactingExporter {
            sink: Arc::clone(&forwarder) as Arc<dyn ExportSink>,
            redactor: Arc::clone(&redactor),
            policy: RedactionPolicy {
                enable_redaction: config.telemetry.enable_redaction,
            },
            resource: (&resource).into(),
            stats: Arc::clone(&stats),
        };
        let meter = metrics.then(|| {
            SdkMeterProvider::builder()
                .with_resource(resource.clone())
                .with_reader(PeriodicReader::builder(exporter()).build())
                .build()
        });
        if let Some(provider) = &meter {
            opentelemetry::global::set_meter_provider(provider.clone());
        }
        let tracer = traces.then(|| {
            SdkTracerProvider::builder()
                .with_resource(resource.clone())
                .with_batch_exporter(exporter())
                .build()
        });
        let logger_provider = logs.then(|| {
            SdkLoggerProvider::builder()
                .with_resource(resource.clone())
                .with_batch_exporter(exporter())
                .build()
        });
        let logger = logger_provider.as_ref().map(|p| p.logger("operator"));
        let instance = Self {
            tracer,
            logger_provider,
            logger,
            meter,
            forwarder,
            timeout,
            stopped: std::sync::atomic::AtomicBool::new(false),
            stats,
        };
        if let Some(provider) = &instance.tracer {
            opentelemetry::global::set_tracer_provider(provider.clone());
        }
        if let Some(provider) = &instance.meter {
            opentelemetry::global::set_meter_provider(provider.clone());
        }
        super::sink::install(Arc::clone(&instance.forwarder) as Arc<dyn ExportSink>);
        if let Some(provider) = instance.meter_provider() {
            super::metrics::register(
                &provider.meter("operator"),
                config.clone(),
                instance.forward_stats(),
                Arc::clone(&instance.stats),
            );
        }
        let _ = ACTIVE.set(instance);
        Ok(active())
    }

    pub fn tracing_layers<S>(&self) -> Vec<Box<dyn Layer<S> + Send + Sync>>
    where
        S: tracing::Subscriber + for<'a> LookupSpan<'a> + Send + Sync,
    {
        let mut layers = Vec::new();
        if let Some(provider) = &self.tracer {
            layers.push(
                tracing_opentelemetry::layer()
                    .with_tracer(provider.tracer("operator"))
                    .with_filter(ExportFilter)
                    .boxed(),
            );
        }
        if let Some(provider) = &self.logger_provider {
            layers.push(
                opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge::new(provider)
                    .with_filter(ExportFilter)
                    .boxed(),
            );
        }
        layers
    }

    pub fn logger(&self) -> Option<&SdkLogger> {
        self.logger.as_ref()
    }
    pub fn meter_provider(&self) -> Option<&SdkMeterProvider> {
        self.meter.as_ref()
    }
    pub fn forward_stats(&self) -> Arc<super::forward::ForwardStats> {
        self.forwarder.stats()
    }
    pub fn shutdown(&self) {
        if self.stopped.swap(true, std::sync::atomic::Ordering::AcqRel) {
            return;
        }
        let deadline = Instant::now() + self.timeout;
        let remaining = || deadline.saturating_duration_since(Instant::now());
        let tracer = self.tracer.clone();
        let logger = self.logger_provider.clone();
        let meter = self.meter.clone();
        let (done_tx, done_rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let remaining = || deadline.saturating_duration_since(Instant::now());
            if let Some(provider) = tracer {
                let _ = provider.shutdown_with_timeout(remaining());
            }
            if let Some(provider) = logger {
                let _ = provider.shutdown_with_timeout(remaining());
            }
            if let Some(provider) = meter {
                let _ = provider.shutdown_with_timeout(remaining());
            }
            let _ = done_tx.send(());
        });
        let _ = done_rx.recv_timeout(remaining());
        self.forwarder.shutdown(remaining());
    }
}

struct ExportFilter;
impl<S> Filter<S> for ExportFilter {
    fn enabled(
        &self,
        metadata: &tracing::Metadata<'_>,
        _: &tracing_subscriber::layer::Context<'_, S>,
    ) -> bool {
        !EXCLUDED_TARGETS
            .iter()
            .any(|target| metadata.target().starts_with(target))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use opentelemetry::logs::{LogRecord, Logger};
    use opentelemetry::trace::{Span, Tracer};
    use std::sync::Mutex;
    use tracing_subscriber::prelude::*;

    #[derive(Default)]
    struct RecordingSink(Mutex<Vec<OtlpPayload>>);
    impl ExportSink for RecordingSink {
        fn submit(&self, payload: OtlpPayload) {
            self.0.lock().unwrap().push(payload);
        }
    }
    fn exporter(sink: Arc<RecordingSink>) -> RedactingExporter {
        RedactingExporter {
            sink,
            redactor: Arc::new(Redactor::new(&[3; 32]).unwrap()),
            policy: RedactionPolicy {
                enable_redaction: true,
            },
            resource: ResourceAttributesWithSchema::default(),
            stats: Arc::new(TelemetryStats::default()),
        }
    }
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_self_export_reaches_local_upstream_on_shutdown() {
        use axum::{body::Bytes, routing::post, Router};
        use prost::Message;
        let captured = Arc::new(Mutex::new(Vec::new()));
        let received = Arc::clone(&captured);
        let app = Router::new().route(
            "/v1/traces",
            post(move |body: Bytes| async move {
                received.lock().unwrap().push(body);
                axum::http::StatusCode::OK
            }),
        );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let endpoint = format!("http://{}", listener.local_addr().unwrap());
        let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        let mut config = Config::default();
        config.telemetry.export.endpoint = Some(endpoint);
        config.telemetry.export.compression = Some(crate::config::OtlpCompression::None);
        let forwarder = Forwarder::start(
            export_config::resolve(&config.telemetry.export, &|_| None),
            ForwarderLimits::default(),
        )
        .unwrap();
        let exporter = RedactingExporter {
            sink: Arc::clone(&forwarder) as Arc<dyn ExportSink>,
            redactor: Arc::new(Redactor::new(&[3; 32]).unwrap()),
            policy: RedactionPolicy {
                enable_redaction: true,
            },
            resource: ResourceAttributesWithSchema::default(),
            stats: Arc::new(TelemetryStats::default()),
        };
        let provider = SdkTracerProvider::builder()
            .with_batch_exporter(exporter)
            .build();
        tracing::subscriber::with_default(
            tracing_subscriber::registry()
                .with(tracing_opentelemetry::layer().with_tracer(provider.tracer("test"))),
            || {
                let span = tracing::info_span!("operator.agent.launch");
                let _entered = span.enter();
                tracing::info!("step started");
            },
        );
        provider.shutdown().unwrap();
        forwarder.shutdown(Duration::from_secs(3));
        let batches = captured.lock().unwrap();
        assert_eq!(batches.len(), 1);
        let request = ExportTraceServiceRequest::decode(batches[0].clone()).unwrap();
        assert_eq!(
            request.resource_spans[0].scope_spans[0].spans[0].name,
            "operator.agent.launch"
        );
        server.abort();
    }

    #[test]
    fn test_span_exporter_sanitizes_before_submission() {
        let sink = Arc::new(RecordingSink::default());
        let provider = SdkTracerProvider::builder()
            .with_simple_exporter(exporter(Arc::clone(&sink)))
            .build();
        let mut span = provider.tracer("test").start("unapproved");
        span.set_attribute(KeyValue::new("user.name", "alice"));
        span.set_attribute(KeyValue::new("operator.auth.token", "secret"));
        span.end();
        provider.force_flush().unwrap();
        let payloads = sink.0.lock().unwrap();
        let OtlpPayload::Traces(request) = &payloads[0] else {
            panic!("expected traces")
        };
        let span = &request.resource_spans[0].scope_spans[0].spans[0];
        assert_eq!(span.name, "operator.redacted.span");
        assert!(!format!("{span:?}").contains("alice"));
        assert!(!span
            .attributes
            .iter()
            .any(|a| a.key == "operator.auth.token"));
    }
    #[test]
    fn test_exporter_diagnostics_do_not_reenter_logs() {
        let sink = Arc::new(RecordingSink::default());
        let provider = SdkLoggerProvider::builder()
            .with_simple_exporter(exporter(Arc::clone(&sink)))
            .build();
        let bridge =
            opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge::new(&provider)
                .with_filter(ExportFilter);
        tracing::subscriber::with_default(tracing_subscriber::registry().with(bridge), || {
            tracing::info!(target: "hyper", "transport");
            tracing::info!(target: "operator::telemetry::forward", "export");
            tracing::info!(target: "opentelemetry_sdk", "sdk");
            tracing::info!(target: "operator::queue", "application");
        });
        provider.force_flush().unwrap();
        assert_eq!(
            sink.0
                .lock()
                .unwrap()
                .iter()
                .map(OtlpPayload::record_count)
                .sum::<u64>(),
            1
        );
    }
    #[test]
    fn test_log_exporter_removes_body_and_secret_attribute() {
        let sink = Arc::new(RecordingSink::default());
        let provider = SdkLoggerProvider::builder()
            .with_simple_exporter(exporter(Arc::clone(&sink)))
            .build();
        let logger = provider.logger("test");
        let mut record = logger.create_log_record();
        record.set_body("private body".into());
        record.add_attribute("operator.auth.token", "secret");
        logger.emit(record);
        provider.force_flush().unwrap();
        let payloads = sink.0.lock().unwrap();
        let OtlpPayload::Logs(request) = &payloads[0] else {
            panic!("expected logs")
        };
        let record = &request.resource_logs[0].scope_logs[0].log_records[0];
        assert!(record.body.is_none());
        assert!(!record
            .attributes
            .iter()
            .any(|a| a.key == "operator.auth.token"));
    }
}

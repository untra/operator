use std::sync::{Arc, Mutex};

use opentelemetry::metrics::{Meter, MeterProvider};
use opentelemetry::KeyValue;
use operator_telemetry::{attr, MetricName, SIGNALS};

use super::{
    forward::{DropReason, ForwardStats},
    TelemetryStats,
};
use crate::{config::Config, rest::dto::TelemetrySignal};

static HUB_STATS: Mutex<Vec<std::sync::Weak<TelemetryStats>>> = Mutex::new(Vec::new());
const SIGNALS_ALL: [TelemetrySignal; 3] = [
    TelemetrySignal::Traces,
    TelemetrySignal::Metrics,
    TelemetrySignal::Logs,
];

pub fn attach_hub_stats(stats: &Arc<TelemetryStats>) {
    let mut hubs = HUB_STATS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    hubs.retain(|hub| hub.strong_count() > 0);
    if !hubs.iter().any(|hub| hub.ptr_eq(&Arc::downgrade(stats))) {
        hubs.push(Arc::downgrade(stats));
    }
}

fn attributes(signal: TelemetrySignal, reason: &'static str) -> [KeyValue; 2] {
    let signal = match signal {
        TelemetrySignal::Traces => "traces",
        TelemetrySignal::Metrics => "metrics",
        TelemetrySignal::Logs => "logs",
    };
    [
        KeyValue::new(attr::OPERATOR_TELEMETRY_SIGNAL, signal),
        KeyValue::new(attr::OPERATOR_TELEMETRY_REASON, reason),
    ]
}

pub fn register(
    meter: &Meter,
    config: Config,
    forward: Arc<ForwardStats>,
    own: Arc<TelemetryStats>,
) {
    for counter in crate::auth::metrics::Counter::ALL {
        let spec = SIGNALS
            .iter()
            .find(|spec| spec.name == counter.name().as_str())
            .expect("auth metric");
        meter
            .u64_observable_counter(counter.name().as_str())
            .with_unit(spec.unit.unwrap_or_default())
            .with_description(spec.description)
            .with_callback(move |observer| observer.observe(counter.value(), &[]))
            .build();
    }
    let state_path = config.state_path().join("state.json");
    meter
        .u64_observable_gauge(MetricName::OperatorAgentsActive.as_str())
        .with_callback(move |observer| {
            if let Ok(bytes) = std::fs::read(&state_path) {
                if let Ok(state) = serde_json::from_slice::<crate::state::State>(&bytes) {
                    observer.observe(state.running_agents().len() as u64, &[]);
                }
            }
        })
        .build();
    meter
        .u64_observable_gauge(MetricName::OperatorQueueDepth.as_str())
        .with_callback(move |observer| {
            if let Ok(queue) =
                crate::queue::Queue::new(&config).and_then(|queue| queue.list_queue())
            {
                observer.observe(queue.len() as u64, &[]);
            }
        })
        .build();
    for name in [
        MetricName::OperatorTelemetryReceived,
        MetricName::OperatorTelemetryDropped,
        MetricName::OperatorTelemetryForwarded,
        MetricName::OperatorTelemetryRedacted,
    ] {
        let forward = Arc::clone(&forward);
        let own = Arc::clone(&own);
        let spec = SIGNALS
            .iter()
            .find(|spec| spec.name == name.as_str())
            .expect("catalog metric");
        meter
            .u64_observable_counter(name.as_str())
            .with_unit(spec.unit.unwrap_or_default())
            .with_description(spec.description)
            .with_callback(move |observer| {
                let mut stats: Vec<_> = HUB_STATS
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .iter()
                    .filter_map(std::sync::Weak::upgrade)
                    .collect();
                stats.push(Arc::clone(&own));
                for signal in SIGNALS_ALL {
                    match name {
                        MetricName::OperatorTelemetryReceived => observer.observe(
                            stats.iter().map(|s| s.received(signal)).sum(),
                            &attributes(signal, "accepted"),
                        ),
                        MetricName::OperatorTelemetryForwarded => observer
                            .observe(forward.forwarded(signal), &attributes(signal, "accepted")),
                        MetricName::OperatorTelemetryDropped => {
                            for reason in DropReason::ALL {
                                let ingest = if matches!(reason, DropReason::QueueFull) {
                                    stats.iter().map(|s| s.dropped_queue_full(signal)).sum()
                                } else {
                                    0
                                };
                                observer.observe(
                                    forward.dropped(signal, reason) + ingest,
                                    &attributes(signal, reason.as_str()),
                                );
                            }
                        }
                        MetricName::OperatorTelemetryRedacted => {
                            for (i, reason) in ["dropped", "hashed", "scrubbed"].iter().enumerate()
                            {
                                observer.observe(
                                    stats.iter().map(|s| s.redaction_counts(signal)[i]).sum(),
                                    &attributes(signal, reason),
                                );
                            }
                        }
                        _ => unreachable!("registered counter"),
                    }
                }
            })
            .build();
    }
}

pub fn record_step_duration(seconds: f64, profile: &str) {
    if let Some(provider) =
        super::self_export::active().and_then(super::self_export::SelfExport::meter_provider)
    {
        provider
            .meter("operator")
            .f64_histogram(MetricName::OperatorStepDuration.as_str())
            .with_unit("s")
            .build()
            .record(
                seconds,
                &[KeyValue::new(attr::OPERATOR_PROFILE_ID, profile.to_owned())],
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use opentelemetry_proto::tonic::collector::metrics::v1::ExportMetricsServiceRequest;
    use opentelemetry_proto::tonic::metrics::v1::{metric::Data, number_data_point::Value};
    use opentelemetry_sdk::{
        error::OTelSdkResult,
        metrics::{
            data::ResourceMetrics, exporter::PushMetricExporter, PeriodicReader, SdkMeterProvider,
            Temporality,
        },
    };
    use std::time::Duration;

    struct Capture(Arc<Mutex<Vec<ExportMetricsServiceRequest>>>);
    impl PushMetricExporter for Capture {
        async fn export(&self, metrics: &ResourceMetrics) -> OTelSdkResult {
            self.0.lock().unwrap().push(metrics.into());
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
    #[test]
    fn test_metrics_collect_state_and_classified_counters() {
        let tmp = tempfile::TempDir::new().unwrap();
        let mut config = Config::default();
        config.paths.state = tmp.path().join("state").to_string_lossy().into_owned();
        config.paths.tickets = tmp.path().join("tickets").to_string_lossy().into_owned();
        crate::state::State::load(&config).unwrap().save().unwrap();
        let captured = Arc::new(Mutex::new(Vec::new()));
        let provider = SdkMeterProvider::builder()
            .with_reader(PeriodicReader::builder(Capture(Arc::clone(&captured))).build())
            .build();
        let forwarder = super::super::forward::Forwarder::start(
            super::super::export_config::resolve(&config.telemetry.export, &|_| None),
            super::super::forward::ForwarderLimits::default(),
        )
        .unwrap();
        let stats = Arc::new(TelemetryStats::default());
        stats.received[super::super::signal_index(TelemetrySignal::Logs)]
            .store(3, std::sync::atomic::Ordering::Relaxed);
        stats.record_redaction(
            TelemetrySignal::Logs,
            operator_telemetry::RedactionStats {
                hashed: 7,
                ..Default::default()
            },
        );
        for counter in crate::auth::metrics::Counter::ALL {
            counter.increment();
        }
        register(&provider.meter("test"), config, forwarder.stats(), stats);
        provider.force_flush().unwrap();
        let batches = captured.lock().unwrap();
        let metrics: Vec<_> = batches
            .iter()
            .flat_map(|batch| &batch.resource_metrics)
            .flat_map(|resource| &resource.scope_metrics)
            .flat_map(|scope| &scope.metrics)
            .collect();
        for metric in &metrics {
            assert!(SIGNALS
                .iter()
                .any(|spec| spec.kind == "metric" && spec.name == metric.name));
        }
        for counter in crate::auth::metrics::Counter::ALL {
            let metric = metrics
                .iter()
                .find(|metric| metric.name == counter.name().as_str())
                .unwrap();
            let Some(Data::Sum(sum)) = &metric.data else {
                panic!("auth counter")
            };
            assert_eq!(sum.data_points.len(), 1);
            assert!(sum.data_points[0].attributes.is_empty());
            assert!(matches!(sum.data_points[0].value, Some(Value::AsInt(value)) if value >= 1));
        }
        for name in [
            MetricName::OperatorQueueDepth,
            MetricName::OperatorAgentsActive,
        ] {
            let metric = metrics.iter().find(|m| m.name == name.as_str()).unwrap();
            let Some(Data::Gauge(gauge)) = &metric.data else {
                panic!("gauge")
            };
            assert_eq!(gauge.data_points[0].value, Some(Value::AsInt(0)));
        }
        let received = metrics
            .iter()
            .find(|m| m.name == MetricName::OperatorTelemetryReceived.as_str())
            .unwrap();
        let Some(Data::Sum(sum)) = &received.data else {
            panic!("counter")
        };
        assert!(sum
            .data_points
            .iter()
            .any(|p| matches!(p.value, Some(Value::AsInt(v)) if v >= 3)));
        drop(batches);
        provider.shutdown().unwrap();
        forwarder.shutdown(Duration::from_secs(1));
    }
}

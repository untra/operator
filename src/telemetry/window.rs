//! Bounded in-memory telemetry per agent session.
//!
//! Operator is a relay, not a store: each session keeps a ring of its newest
//! records, the least recently updated session is evicted past the session
//! cap, and an ended session stays readable only for a grace period.

use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

use opentelemetry_proto::tonic::common::v1::KeyValue;

use super::otlp::{any_to_json, OtlpPayload};
use crate::rest::dto::{TelemetryRecord, TelemetrySessionWindow, TelemetrySignal};

#[derive(Debug, Clone, Copy)]
pub struct WindowLimits {
    pub max_sessions: usize,
    pub records_per_session: usize,
    pub grace: Duration,
}

#[derive(Debug)]
struct Session {
    records: VecDeque<TelemetryRecord>,
    evicted: u64,
    touched: Instant,
    ended_at: Option<Instant>,
}

#[derive(Debug)]
pub struct SessionWindow {
    limits: WindowLimits,
    sessions: HashMap<String, Session>,
}

impl SessionWindow {
    pub fn new(limits: WindowLimits) -> Self {
        Self {
            limits,
            sessions: HashMap::new(),
        }
    }

    pub fn insert(&mut self, session_id: &str, records: Vec<TelemetryRecord>, now: Instant) {
        if records.is_empty() || self.limits.records_per_session == 0 {
            return;
        }
        let session = self
            .sessions
            .entry(session_id.to_string())
            .or_insert_with(|| Session {
                records: VecDeque::new(),
                evicted: 0,
                touched: now,
                ended_at: None,
            });
        session.touched = now;
        for record in records {
            if session.records.len() == self.limits.records_per_session {
                session.records.pop_front();
                session.evicted += 1;
            }
            session.records.push_back(record);
        }
        self.prune(now);
    }

    /// Start the grace period; later records are still accepted until it expires.
    pub fn end(&mut self, session_id: &str, now: Instant) {
        let session = self
            .sessions
            .entry(session_id.to_owned())
            .or_insert_with(|| Session {
                records: VecDeque::new(),
                evicted: 0,
                touched: now,
                ended_at: None,
            });
        session.ended_at.get_or_insert(now);
        self.prune(now);
    }

    pub fn get(&mut self, session_id: &str, now: Instant) -> Option<TelemetrySessionWindow> {
        self.prune(now);
        self.sessions
            .get(session_id)
            .map(|s| TelemetrySessionWindow {
                session_id: session_id.to_string(),
                ended: s.ended_at.is_some(),
                evicted_records: s.evicted,
                records: s.records.iter().cloned().collect(),
            })
    }

    fn prune(&mut self, now: Instant) {
        let grace = self.limits.grace;
        self.sessions.retain(|_, s| {
            s.ended_at
                .is_none_or(|ended| now.duration_since(ended) < grace)
        });
        while self.sessions.len() > self.limits.max_sessions {
            let Some(oldest) = self
                .sessions
                .iter()
                .min_by_key(|(_, s)| s.touched)
                .map(|(id, _)| id.clone())
            else {
                break;
            };
            self.sessions.remove(&oldest);
        }
    }
}

/// Flatten a sanitized payload into records, resource attributes underneath record attributes.
pub fn records(payload: &OtlpPayload) -> Vec<TelemetryRecord> {
    let mut out = Vec::new();
    match payload {
        OtlpPayload::Traces(req) => {
            for rs in &req.resource_spans {
                let resource = resource_attrs(rs.resource.as_ref());
                for span in rs.scope_spans.iter().flat_map(|s| &s.spans) {
                    out.push(TelemetryRecord {
                        signal: TelemetrySignal::Traces,
                        name: span.name.clone(),
                        time_unix_nano: span.start_time_unix_nano.to_string(),
                        trace_id: hex(&span.trace_id),
                        span_id: hex(&span.span_id),
                        attributes: merged(resource, &span.attributes),
                        body: None,
                    });
                }
            }
        }
        OtlpPayload::Logs(req) => {
            for rl in &req.resource_logs {
                let resource = resource_attrs(rl.resource.as_ref());
                for log in rl.scope_logs.iter().flat_map(|s| &s.log_records) {
                    let time = if log.time_unix_nano == 0 {
                        log.observed_time_unix_nano
                    } else {
                        log.time_unix_nano
                    };
                    out.push(TelemetryRecord {
                        signal: TelemetrySignal::Logs,
                        name: log.event_name.clone(),
                        time_unix_nano: time.to_string(),
                        trace_id: hex(&log.trace_id),
                        span_id: hex(&log.span_id),
                        attributes: merged(resource, &log.attributes),
                        body: log.body.as_ref().map(any_to_json),
                    });
                }
            }
        }
        OtlpPayload::Metrics(req) => {
            for rm in &req.resource_metrics {
                let resource = resource_attrs(rm.resource.as_ref());
                for metric in rm.scope_metrics.iter().flat_map(|s| &s.metrics) {
                    out.push(TelemetryRecord {
                        signal: TelemetrySignal::Metrics,
                        name: metric.name.clone(),
                        time_unix_nano: metric_time(metric).to_string(),
                        trace_id: None,
                        span_id: None,
                        attributes: merged(resource, &metric.metadata),
                        body: None,
                    });
                }
            }
        }
    }
    out
}

/// Latest data point time, since a metric record summarizes all of its points.
fn metric_time(metric: &opentelemetry_proto::tonic::metrics::v1::Metric) -> u64 {
    use opentelemetry_proto::tonic::metrics::v1::metric::Data;
    let times: Vec<u64> = match &metric.data {
        Some(Data::Gauge(g)) => g.data_points.iter().map(|p| p.time_unix_nano).collect(),
        Some(Data::Sum(s)) => s.data_points.iter().map(|p| p.time_unix_nano).collect(),
        Some(Data::Histogram(h)) => h.data_points.iter().map(|p| p.time_unix_nano).collect(),
        Some(Data::ExponentialHistogram(h)) => {
            h.data_points.iter().map(|p| p.time_unix_nano).collect()
        }
        Some(Data::Summary(s)) => s.data_points.iter().map(|p| p.time_unix_nano).collect(),
        None => Vec::new(),
    };
    times.into_iter().max().unwrap_or_default()
}

fn resource_attrs(
    resource: Option<&opentelemetry_proto::tonic::resource::v1::Resource>,
) -> &[KeyValue] {
    resource.map_or(&[], |r| r.attributes.as_slice())
}

fn merged(
    resource: &[KeyValue],
    record: &[KeyValue],
) -> serde_json::Map<String, serde_json::Value> {
    resource
        .iter()
        .chain(record)
        .map(|kv| {
            (
                kv.key.clone(),
                kv.value
                    .as_ref()
                    .map_or(serde_json::Value::Null, any_to_json),
            )
        })
        .collect()
}

fn hex(bytes: &[u8]) -> Option<String> {
    (!bytes.is_empty()).then(|| super::otlp::hex(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(name: &str) -> TelemetryRecord {
        TelemetryRecord {
            signal: TelemetrySignal::Logs,
            name: name.to_string(),
            time_unix_nano: "0".to_string(),
            trace_id: None,
            span_id: None,
            attributes: serde_json::Map::new(),
            body: None,
        }
    }

    fn window(max_sessions: usize, per_session: usize) -> SessionWindow {
        SessionWindow::new(WindowLimits {
            max_sessions,
            records_per_session: per_session,
            grace: Duration::from_secs(10),
        })
    }

    fn names(w: &mut SessionWindow, id: &str, now: Instant) -> Vec<String> {
        w.get(id, now)
            .map(|s| s.records.into_iter().map(|r| r.name).collect())
            .unwrap_or_default()
    }

    #[test]
    fn test_end_before_first_record_preserves_terminal_state() {
        let mut window = SessionWindow::new(WindowLimits {
            max_sessions: 2,
            records_per_session: 2,
            grace: Duration::from_secs(1),
        });
        let now = Instant::now();
        window.end("late", now);
        assert!(window.get("late", now).unwrap().ended);
        assert!(window.get("late", now + Duration::from_secs(2)).is_none());
    }

    #[test]
    fn test_ring_keeps_newest_and_counts_evictions() {
        let now = Instant::now();
        let mut w = window(4, 2);
        w.insert("s", vec![record("a"), record("b"), record("c")], now);
        assert_eq!(names(&mut w, "s", now), vec!["b", "c"]);
        assert_eq!(w.get("s", now).unwrap().evicted_records, 1);
    }

    #[test]
    fn test_least_recently_updated_session_is_evicted() {
        let t0 = Instant::now();
        let mut w = window(2, 8);
        w.insert("old", vec![record("a")], t0);
        w.insert("mid", vec![record("b")], t0 + Duration::from_secs(1));
        w.insert("old", vec![record("c")], t0 + Duration::from_secs(2));
        w.insert("new", vec![record("d")], t0 + Duration::from_secs(3));
        let now = t0 + Duration::from_secs(3);
        assert!(w.get("mid", now).is_none());
        assert_eq!(names(&mut w, "old", now), vec!["a", "c"]);
    }

    #[test]
    fn test_ended_session_is_readable_only_during_grace() {
        let t0 = Instant::now();
        let mut w = window(4, 8);
        w.insert("s", vec![record("a")], t0);
        w.end("s", t0);
        assert!(w.get("s", t0 + Duration::from_secs(9)).unwrap().ended);
        assert!(w.get("s", t0 + Duration::from_secs(10)).is_none());
    }
}

//! Redaction and trusted enrichment over OTLP wire types.
//!
//! All attributes and text are inspected before queueing, without encoding away credentials.

use opentelemetry_proto::tonic::collector::logs::v1::ExportLogsServiceRequest;
use opentelemetry_proto::tonic::collector::metrics::v1::ExportMetricsServiceRequest;
use opentelemetry_proto::tonic::collector::trace::v1::ExportTraceServiceRequest;
use opentelemetry_proto::tonic::common::v1::any_value::Value as Any;
use opentelemetry_proto::tonic::common::v1::{AnyValue, KeyValue};
#[cfg(test)]
use opentelemetry_proto::tonic::common::v1::{ArrayValue, KeyValueList};
use opentelemetry_proto::tonic::metrics::v1::metric::Data;
use opentelemetry_proto::tonic::metrics::v1::{Exemplar, Metric};
use opentelemetry_proto::tonic::resource::v1::Resource;
use operator_telemetry::{
    attr, EventName, RedactionPolicy, RedactionStats, Redactor, SpanName, SIGNALS,
};

use crate::rest::dto::TelemetrySignal;
use serde_json::{Number, Value};

/// Correlation attributes only the server may assert; client values are discarded.
pub const TRUSTED_KEYS: &[&str] = &[
    attr::OPERATOR_TICKET_ID,
    attr::OPERATOR_STEP_NAME,
    attr::OPERATOR_SESSION_ID,
    attr::OPERATOR_PROFILE_ID,
];

/// Correlation asserted by a verified agent token.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TrustedContext {
    /// Identifies the ticket asserted by the verified token.
    pub ticket_id: Option<String>,
    /// Names the workflow step asserted by the verified token.
    pub step: Option<String>,
    /// Identifies the session asserted by the verified token.
    pub session_id: Option<String>,
    /// Identifies the owning configuration profile asserted by the verified token.
    pub profile_id: Option<String>,
}

impl TrustedContext {
    fn attributes(&self) -> Vec<KeyValue> {
        [
            (attr::OPERATOR_TICKET_ID, &self.ticket_id),
            (attr::OPERATOR_STEP_NAME, &self.step),
            (attr::OPERATOR_SESSION_ID, &self.session_id),
            (attr::OPERATOR_PROFILE_ID, &self.profile_id),
        ]
        .into_iter()
        .filter_map(|(key, value)| value.as_ref().map(|v| string_kv(key, v)))
        .collect()
    }
}

/// One OTLP export request of any signal.
#[derive(Debug, Clone, PartialEq)]
pub enum OtlpPayload {
    Traces(ExportTraceServiceRequest),
    Metrics(ExportMetricsServiceRequest),
    Logs(ExportLogsServiceRequest),
}

/// Callbacks for [`walk`]. Resource attributes are visited after their children.
trait Visitor {
    fn attributes(&mut self, list: &mut Vec<KeyValue>);
    fn body(&mut self, _body: &mut Option<AnyValue>) {}
    fn text(&mut self, _text: &mut String) {}
    fn name(&mut self, _name: &mut String, _event: bool) {}
    fn metric(&mut self, _metric: &mut Metric) -> bool {
        true
    }
    fn begin_resource(&mut self) {}
    fn end_resource(&mut self, resource: &mut Resource);
}

impl OtlpPayload {
    /// A copy with `policy` applied everywhere and `operator.redacted.count` set per resource.
    pub fn redacted(&self, redactor: &Redactor, policy: RedactionPolicy) -> (Self, RedactionStats) {
        let mut out = self.clone();
        let mut pass = RedactPass {
            redactor,
            policy,
            resource: RedactionStats::default(),
            total: RedactionStats::default(),
        };
        walk(&mut out, &mut pass);
        (out, pass.total)
    }

    /// Discard client-asserted correlation anywhere and assert the trusted context on each resource.
    ///
    /// With no trusted context (an admin credential), client correlation is kept.
    pub fn enrich(&mut self, context: Option<&TrustedContext>) {
        if let Some(context) = context {
            walk(
                self,
                &mut EnrichPass {
                    trusted: context.attributes(),
                },
            );
        }
    }

    /// The session id asserted on the first resource that carries one.
    pub fn session_id(&self) -> Option<String> {
        let resources: Vec<&Option<Resource>> = match self {
            Self::Traces(req) => req.resource_spans.iter().map(|r| &r.resource).collect(),
            Self::Logs(req) => req.resource_logs.iter().map(|r| &r.resource).collect(),
            Self::Metrics(req) => req.resource_metrics.iter().map(|r| &r.resource).collect(),
        };
        resources
            .into_iter()
            .flatten()
            .find_map(|r| string_attr(&r.attributes, attr::OPERATOR_SESSION_ID))
    }

    pub fn signal(&self) -> TelemetrySignal {
        match self {
            Self::Traces(_) => TelemetrySignal::Traces,
            Self::Metrics(_) => TelemetrySignal::Metrics,
            Self::Logs(_) => TelemetrySignal::Logs,
        }
    }

    /// Spans, log records, or metric data points.
    pub fn record_count(&self) -> u64 {
        match self {
            Self::Traces(req) => req
                .resource_spans
                .iter()
                .flat_map(|resource| &resource.scope_spans)
                .map(|scope| scope.spans.len() as u64)
                .sum(),
            Self::Logs(req) => req
                .resource_logs
                .iter()
                .flat_map(|resource| &resource.scope_logs)
                .map(|scope| scope.log_records.len() as u64)
                .sum(),
            Self::Metrics(req) => req
                .resource_metrics
                .iter()
                .flat_map(|resource| &resource.scope_metrics)
                .flat_map(|scope| &scope.metrics)
                .map(metric_point_count)
                .sum(),
        }
    }
}

fn metric_point_count(metric: &Metric) -> u64 {
    match &metric.data {
        Some(Data::Gauge(gauge)) => gauge.data_points.len() as u64,
        Some(Data::Sum(sum)) => sum.data_points.len() as u64,
        Some(Data::Histogram(histogram)) => histogram.data_points.len() as u64,
        Some(Data::ExponentialHistogram(histogram)) => histogram.data_points.len() as u64,
        Some(Data::Summary(summary)) => summary.data_points.len() as u64,
        None => 0,
    }
}

fn walk(payload: &mut OtlpPayload, v: &mut impl Visitor) {
    match payload {
        OtlpPayload::Traces(req) => {
            for rs in &mut req.resource_spans {
                v.begin_resource();
                v.text(&mut rs.schema_url);
                for ss in &mut rs.scope_spans {
                    v.text(&mut ss.schema_url);
                    if let Some(scope) = &mut ss.scope {
                        v.text(&mut scope.name);
                        v.text(&mut scope.version);
                        v.attributes(&mut scope.attributes);
                    }
                    for span in &mut ss.spans {
                        v.name(&mut span.name, false);
                        v.text(&mut span.trace_state);
                        v.attributes(&mut span.attributes);
                        for event in &mut span.events {
                            v.name(&mut event.name, true);
                            v.attributes(&mut event.attributes);
                        }
                        for link in &mut span.links {
                            v.text(&mut link.trace_state);
                            v.attributes(&mut link.attributes);
                        }
                        if let Some(status) = &mut span.status {
                            v.text(&mut status.message);
                        }
                    }
                }
                v.end_resource(rs.resource.get_or_insert_with(Resource::default));
            }
        }
        OtlpPayload::Logs(req) => {
            for rl in &mut req.resource_logs {
                v.begin_resource();
                v.text(&mut rl.schema_url);
                for sl in &mut rl.scope_logs {
                    v.text(&mut sl.schema_url);
                    if let Some(scope) = &mut sl.scope {
                        v.text(&mut scope.name);
                        v.text(&mut scope.version);
                        v.attributes(&mut scope.attributes);
                    }
                    for record in &mut sl.log_records {
                        if !record.event_name.is_empty() {
                            v.name(&mut record.event_name, true);
                        }
                        v.text(&mut record.severity_text);
                        v.attributes(&mut record.attributes);
                        v.body(&mut record.body);
                    }
                }
                v.end_resource(rl.resource.get_or_insert_with(Resource::default));
            }
        }
        OtlpPayload::Metrics(req) => {
            for rm in &mut req.resource_metrics {
                v.begin_resource();
                v.text(&mut rm.schema_url);
                for sm in &mut rm.scope_metrics {
                    v.text(&mut sm.schema_url);
                    if let Some(scope) = &mut sm.scope {
                        v.text(&mut scope.name);
                        v.text(&mut scope.version);
                        v.attributes(&mut scope.attributes);
                    }
                    sm.metrics.retain_mut(|metric| v.metric(metric));
                    for metric in &mut sm.metrics {
                        walk_metric(metric, v);
                    }
                }
                v.end_resource(rm.resource.get_or_insert_with(Resource::default));
            }
        }
    }
}

fn walk_metric(metric: &mut Metric, v: &mut impl Visitor) {
    v.attributes(&mut metric.metadata);
    let exemplars = |list: &mut Vec<Exemplar>, v: &mut dyn FnMut(&mut Vec<KeyValue>)| {
        for e in list {
            v(&mut e.filtered_attributes);
        }
    };
    let mut visit = |list: &mut Vec<KeyValue>| v.attributes(list);
    match &mut metric.data {
        Some(Data::Gauge(g)) => {
            for p in &mut g.data_points {
                visit(&mut p.attributes);
                exemplars(&mut p.exemplars, &mut visit);
            }
        }
        Some(Data::Sum(s)) => {
            for p in &mut s.data_points {
                visit(&mut p.attributes);
                exemplars(&mut p.exemplars, &mut visit);
            }
        }
        Some(Data::Histogram(h)) => {
            for p in &mut h.data_points {
                visit(&mut p.attributes);
                exemplars(&mut p.exemplars, &mut visit);
            }
        }
        Some(Data::ExponentialHistogram(h)) => {
            for p in &mut h.data_points {
                visit(&mut p.attributes);
                exemplars(&mut p.exemplars, &mut visit);
            }
        }
        Some(Data::Summary(s)) => {
            for p in &mut s.data_points {
                visit(&mut p.attributes);
            }
        }
        None => {}
    }
}

struct RedactPass<'a> {
    redactor: &'a Redactor,
    policy: RedactionPolicy,
    resource: RedactionStats,
    total: RedactionStats,
}

impl Visitor for RedactPass<'_> {
    fn attributes(&mut self, list: &mut Vec<KeyValue>) {
        let mut kept = Vec::with_capacity(list.len());
        for kv in list.drain(..) {
            if kv.key == attr::OPERATOR_REDACTED_COUNT {
                continue;
            }
            if kv.key_strindex != 0
                || kv
                    .value
                    .as_ref()
                    .is_none_or(|v| self.unsafe_attribute_value(v))
            {
                self.resource.dropped += 1;
                continue;
            }
            let json = kv.value.as_ref().map_or(Value::Null, policy_json);
            let result = self.redactor.redact_attribute(&kv.key, &json, self.policy);
            let hashed = result.stats.hashed > 0;
            add(&mut self.resource, result.stats);
            match result.value {
                None => {}
                Some(Value::String(hash)) if hashed => kept.push(string_kv(&kv.key, &hash)),
                Some(_) => kept.push(kv),
            }
        }
        *list = kept;
    }

    fn body(&mut self, body: &mut Option<AnyValue>) {
        let Some(mut value) = body.take() else {
            return;
        };
        if self.policy.enable_redaction {
            self.resource.dropped += 1;
        } else if self.scrub_value(&mut value) {
            *body = Some(value);
        }
    }

    fn text(&mut self, text: &mut String) {
        if text.is_empty() {
            return;
        }
        if self.policy.enable_redaction {
            text.clear();
            self.resource.dropped += 1;
        } else {
            let (scrubbed, count) = self.redactor.scrub_text(text);
            *text = scrubbed;
            self.resource.scrubbed += count;
        }
    }

    fn name(&mut self, name: &mut String, event: bool) {
        if !self.policy.enable_redaction {
            self.text(name);
            return;
        }
        let known = if event {
            EventName::ALL.iter().any(|n| n.as_str() == name)
        } else {
            SpanName::ALL.iter().any(|n| n.as_str() == name)
        };
        if !known {
            *name = if event {
                EventName::OperatorRedactedEvent.as_str()
            } else {
                SpanName::OperatorRedactedSpan.as_str()
            }
            .to_string();
            self.resource.scrubbed += 1;
        }
    }

    fn metric(&mut self, metric: &mut Metric) -> bool {
        if !self.policy.enable_redaction {
            self.text(&mut metric.name);
            self.text(&mut metric.description);
            self.text(&mut metric.unit);
            return true;
        }
        let Some(spec) = SIGNALS
            .iter()
            .find(|s| s.kind == "metric" && s.name == metric.name)
        else {
            self.resource.dropped += 1;
            return false;
        };
        for (text, approved) in [
            (&mut metric.description, spec.description),
            (&mut metric.unit, spec.unit.unwrap_or_default()),
        ] {
            if text != approved {
                *text = approved.to_string();
                self.resource.scrubbed += 1;
            }
        }
        true
    }

    fn begin_resource(&mut self) {
        self.resource = RedactionStats::default();
    }

    fn end_resource(&mut self, resource: &mut Resource) {
        self.attributes(&mut resource.attributes);
        if self.policy.enable_redaction {
            self.resource.dropped += resource.entity_refs.len() as u64;
            resource.entity_refs.clear();
        } else {
            for entity in &mut resource.entity_refs {
                self.text(&mut entity.schema_url);
                self.text(&mut entity.r#type);
                for key in entity
                    .id_keys
                    .iter_mut()
                    .chain(&mut entity.description_keys)
                {
                    self.text(key);
                }
            }
        }
        let count = self.resource.total();
        resource.attributes.push(KeyValue {
            key: attr::OPERATOR_REDACTED_COUNT.to_string(),
            value: Some(AnyValue {
                value: Some(Any::IntValue(i64::try_from(count).unwrap_or(i64::MAX))),
            }),
            ..KeyValue::default()
        });
        add(&mut self.total, self.resource);
    }
}

impl RedactPass<'_> {
    fn unsafe_attribute_value(&self, value: &AnyValue) -> bool {
        match &value.value {
            Some(Any::StringValue(text)) => self.redactor.contains_secret_text(text),
            Some(Any::BytesValue(bytes)) => std::str::from_utf8(bytes)
                .map_or(true, |text| self.redactor.contains_secret_text(text)),
            Some(Any::StringValueStrindex(_)) => true,
            Some(Any::ArrayValue(array)) => {
                array.values.iter().any(|v| self.unsafe_attribute_value(v))
            }
            Some(Any::KvlistValue(map)) => map.values.iter().any(|kv| {
                kv.key_strindex != 0
                    || self.redactor.is_secret_key(&kv.key)
                    || kv
                        .value
                        .as_ref()
                        .is_some_and(|v| self.unsafe_attribute_value(v))
            }),
            _ => false,
        }
    }

    fn scrub_value(&mut self, value: &mut AnyValue) -> bool {
        match &mut value.value {
            Some(Any::StringValue(text)) => self.text(text),
            Some(Any::BytesValue(bytes)) => {
                let Ok(text) = std::str::from_utf8(bytes) else {
                    self.resource.dropped += 1;
                    return false;
                };
                let (scrubbed, count) = self.redactor.scrub_text(text);
                if count != 0 {
                    *bytes = scrubbed.into_bytes();
                }
                self.resource.scrubbed += count;
            }
            Some(Any::StringValueStrindex(_)) => {
                self.resource.dropped += 1;
                return false;
            }
            Some(Any::ArrayValue(array)) => array.values.retain_mut(|v| self.scrub_value(v)),
            Some(Any::KvlistValue(map)) => map.values.retain_mut(|kv| {
                if kv.key_strindex != 0 || self.redactor.is_secret_key(&kv.key) {
                    self.resource.dropped += 1;
                    false
                } else {
                    kv.value.as_mut().is_none_or(|v| self.scrub_value(v))
                }
            }),
            _ => {}
        }
        true
    }
}

fn policy_json(value: &AnyValue) -> Value {
    match &value.value {
        Some(Any::BytesValue(_)) => Value::Null,
        Some(Any::ArrayValue(array)) => {
            Value::Array(array.values.iter().map(policy_json).collect())
        }
        Some(Any::KvlistValue(map)) => Value::Object(
            map.values
                .iter()
                .map(|kv| {
                    (
                        kv.key.clone(),
                        kv.value.as_ref().map_or(Value::Null, policy_json),
                    )
                })
                .collect(),
        ),
        _ => any_to_json(value),
    }
}

struct EnrichPass {
    trusted: Vec<KeyValue>,
}

impl Visitor for EnrichPass {
    fn attributes(&mut self, list: &mut Vec<KeyValue>) {
        list.retain(|kv| !TRUSTED_KEYS.contains(&kv.key.as_str()));
    }

    fn end_resource(&mut self, resource: &mut Resource) {
        self.attributes(&mut resource.attributes);
        resource.attributes.extend(self.trusted.iter().cloned());
    }
}

fn add(total: &mut RedactionStats, more: RedactionStats) {
    total.dropped += more.dropped;
    total.hashed += more.hashed;
    total.scrubbed += more.scrubbed;
}

fn string_kv(key: &str, value: &str) -> KeyValue {
    KeyValue {
        key: key.to_string(),
        value: Some(AnyValue {
            value: Some(Any::StringValue(value.to_string())),
        }),
        ..KeyValue::default()
    }
}

pub fn string_attr(list: &[KeyValue], key: &str) -> Option<String> {
    list.iter()
        .find(|kv| kv.key == key)
        .and_then(|kv| match kv.value.as_ref()?.value.as_ref()? {
            Any::StringValue(s) => Some(s.clone()),
            _ => None,
        })
}

/// Converts sanitized wire values for window display, representing bytes as lowercase hex.
pub fn any_to_json(value: &AnyValue) -> Value {
    match &value.value {
        None | Some(Any::StringValueStrindex(_)) => Value::Null,
        Some(Any::StringValue(s)) => Value::String(s.clone()),
        Some(Any::BoolValue(b)) => Value::Bool(*b),
        Some(Any::IntValue(i)) => Value::Number((*i).into()),
        Some(Any::DoubleValue(d)) => Number::from_f64(*d).map_or(Value::Null, Value::Number),
        Some(Any::ArrayValue(a)) => Value::Array(a.values.iter().map(any_to_json).collect()),
        Some(Any::KvlistValue(kv)) => Value::Object(
            kv.values
                .iter()
                .map(|kv| {
                    (
                        kv.key.clone(),
                        kv.value.as_ref().map_or(Value::Null, any_to_json),
                    )
                })
                .collect(),
        ),
        Some(Any::BytesValue(bytes)) => Value::String(hex(bytes)),
    }
}

/// Lowercase hex, the OTLP JSON spelling of trace and span ids.
pub fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write;
    bytes
        .iter()
        .fold(String::with_capacity(bytes.len() * 2), |mut out, b| {
            let _ = write!(out, "{b:02x}");
            out
        })
}

#[cfg(test)]
fn json_to_any(value: &Value) -> AnyValue {
    let value = match value {
        Value::Null => None,
        Value::Bool(b) => Some(Any::BoolValue(*b)),
        Value::Number(n) => n
            .as_i64()
            .map(Any::IntValue)
            .or_else(|| n.as_f64().map(Any::DoubleValue)),
        Value::String(s) => Some(Any::StringValue(s.clone())),
        Value::Array(values) => Some(Any::ArrayValue(ArrayValue {
            values: values.iter().map(json_to_any).collect(),
        })),
        Value::Object(map) => Some(Any::KvlistValue(KeyValueList {
            values: map
                .iter()
                .map(|(k, v)| KeyValue {
                    key: k.clone(),
                    value: Some(json_to_any(v)),
                    ..KeyValue::default()
                })
                .collect(),
        })),
    };
    AnyValue { value }
}

#[cfg(test)]
mod tests {
    use super::*;
    use opentelemetry_proto::tonic::logs::v1::{LogRecord, ResourceLogs, ScopeLogs};
    use opentelemetry_proto::tonic::trace::v1::{ResourceSpans, ScopeSpans, Span};

    const KEY: [u8; 32] = [7; 32];

    fn kv(key: &str, value: &str) -> KeyValue {
        string_kv(key, value)
    }

    fn keys(list: &[KeyValue]) -> Vec<&str> {
        list.iter().map(|kv| kv.key.as_str()).collect()
    }

    fn traces(resource: Vec<KeyValue>, span: Vec<KeyValue>) -> OtlpPayload {
        OtlpPayload::Traces(ExportTraceServiceRequest {
            resource_spans: vec![ResourceSpans {
                resource: Some(Resource {
                    attributes: resource,
                    ..Resource::default()
                }),
                scope_spans: vec![ScopeSpans {
                    spans: vec![Span {
                        name: "opr8r.step".to_string(),
                        attributes: span,
                        ..Span::default()
                    }],
                    ..ScopeSpans::default()
                }],
                ..ResourceSpans::default()
            }],
        })
    }

    fn span_attrs(p: &OtlpPayload) -> &[KeyValue] {
        let OtlpPayload::Traces(req) = p else {
            unreachable!()
        };
        &req.resource_spans[0].scope_spans[0].spans[0].attributes
    }

    fn resource_attrs(p: &OtlpPayload) -> &[KeyValue] {
        let OtlpPayload::Traces(req) = p else {
            unreachable!()
        };
        &req.resource_spans[0].resource.as_ref().unwrap().attributes
    }

    fn policy(enable_redaction: bool) -> RedactionPolicy {
        RedactionPolicy { enable_redaction }
    }

    #[test]
    fn test_enrich_replaces_spoofed_correlation_everywhere() {
        let mut p = traces(
            vec![
                kv(attr::OPERATOR_SESSION_ID, "spoofed"),
                kv("service.name", "claude"),
            ],
            vec![kv(attr::OPERATOR_TICKET_ID, "OTHER-1")],
        );
        p.enrich(Some(&TrustedContext {
            ticket_id: Some("FEAT-1".into()),
            step: Some("build".into()),
            session_id: Some("sess-1".into()),
            profile_id: None,
        }));
        assert!(span_attrs(&p).is_empty());
        assert_eq!(p.session_id().as_deref(), Some("sess-1"));
        assert_eq!(
            string_attr(resource_attrs(&p), attr::OPERATOR_TICKET_ID).as_deref(),
            Some("FEAT-1")
        );
        assert_eq!(
            resource_attrs(&p)
                .iter()
                .filter(|kv| kv.key == attr::OPERATOR_SESSION_ID)
                .count(),
            1
        );
    }

    #[test]
    fn test_enrich_without_context_keeps_admin_supplied_correlation() {
        let mut p = traces(vec![kv(attr::OPERATOR_SESSION_ID, "s")], vec![]);
        p.enrich(None);
        assert_eq!(p.session_id().as_deref(), Some("s"));
    }

    #[test]
    fn test_redaction_drops_secrets_unknowns_and_counts_per_resource() {
        let redactor = Redactor::new(&KEY).unwrap();
        let p = traces(
            vec![kv("service.name", "claude")],
            vec![
                kv(attr::OPERATOR_AUTH_TOKEN, "tok"),
                kv("unknown.vendor.field", "prompt text"),
                kv(attr::OPERATOR_LLM_TOOL, "claude"),
                kv(attr::OPERATOR_REDACTED_COUNT, "999"),
            ],
        );
        let (out, stats) = p.redacted(&redactor, policy(true));
        assert_eq!(keys(span_attrs(&out)), vec![attr::OPERATOR_LLM_TOOL]);
        assert_eq!(stats.dropped, 2);
        let count = resource_attrs(&out)
            .iter()
            .find(|kv| kv.key == attr::OPERATOR_REDACTED_COUNT)
            .and_then(|kv| kv.value.clone());
        assert_eq!(
            count.and_then(|v| v.value),
            Some(Any::IntValue(2)),
            "client-sent counts are replaced by the authoritative total"
        );
    }

    #[test]
    fn test_personal_enabled_is_hashed_stably() {
        let redactor = Redactor::new(&KEY).unwrap();
        let p = traces(vec![], vec![kv("user.name", "alice")]);
        let (first, _) = p.redacted(&redactor, policy(true));
        let (second, _) = p.redacted(&redactor, policy(true));
        let hashed = string_attr(span_attrs(&first), "user.name").unwrap();
        assert!(hashed.starts_with("hmac:"), "{hashed}");
        assert_eq!(span_attrs(&first), span_attrs(&second));
    }

    #[test]
    fn test_log_body_is_dropped_when_enabled_and_scrubbed_when_disabled() {
        let redactor = Redactor::new(&KEY).unwrap();
        let p = OtlpPayload::Logs(ExportLogsServiceRequest {
            resource_logs: vec![ResourceLogs {
                scope_logs: vec![ScopeLogs {
                    log_records: vec![LogRecord {
                        body: Some(json_to_any(&Value::String(
                            "token ghp_0123456789abcdefghijABCDEFGHIJ012345 leaked".into(),
                        ))),
                        ..LogRecord::default()
                    }],
                    ..ScopeLogs::default()
                }],
                ..ResourceLogs::default()
            }],
        });
        let body = |p: &OtlpPayload| {
            let OtlpPayload::Logs(req) = p else {
                unreachable!()
            };
            req.resource_logs[0].scope_logs[0].log_records[0]
                .body
                .clone()
        };

        let (dropped, _) = p.redacted(&redactor, policy(true));
        assert_eq!(body(&dropped), None);

        let (scrubbed, stats) = p.redacted(&redactor, policy(false));
        let text = any_to_json(&body(&scrubbed).unwrap());
        assert_eq!(text, Value::String("token [REDACTED] leaked".into()));
        assert_eq!(stats.scrubbed, 1);
    }

    #[test]
    fn test_json_round_trip_preserves_scalar_types() {
        for v in [
            serde_json::json!(1),
            serde_json::json!(1.5),
            serde_json::json!(true),
            serde_json::json!({"a": [1, "x"]}),
        ] {
            assert_eq!(any_to_json(&json_to_any(&v)), v);
        }
    }
    #[test]
    fn test_names_and_all_trace_text_follow_the_selected_mode() {
        use opentelemetry_proto::tonic::common::v1::InstrumentationScope;
        use opentelemetry_proto::tonic::trace::v1::{
            span::{Event, Link},
            Status,
        };
        let text = "private sk-ant-fixture";
        let mut payload = traces(
            vec![kv("unknown", "private")],
            vec![kv("user.name", "alice")],
        );
        let OtlpPayload::Traces(req) = &mut payload else {
            unreachable!()
        };
        let resource = &mut req.resource_spans[0];
        resource.schema_url = text.into();
        let scope = &mut resource.scope_spans[0];
        scope.schema_url = text.into();
        scope.scope = Some(InstrumentationScope {
            name: text.into(),
            version: text.into(),
            attributes: vec![kv("unknown", "private")],
            ..Default::default()
        });
        let span = &mut scope.spans[0];
        span.name = text.into();
        span.trace_state = text.into();
        span.trace_id = vec![1; 16];
        span.span_id = vec![2; 8];
        span.status = Some(Status {
            message: text.into(),
            code: 2,
        });
        span.events = vec![Event {
            name: text.into(),
            attributes: vec![kv("unknown", "private")],
            ..Default::default()
        }];
        span.links = vec![Link {
            trace_state: text.into(),
            attributes: vec![kv("unknown", "private")],
            ..Default::default()
        }];
        let redactor = Redactor::new(&KEY).unwrap();
        for enabled in [false, true] {
            let (out, _) = payload.redacted(&redactor, policy(enabled));
            let OtlpPayload::Traces(req) = &out else {
                unreachable!()
            };
            let resource = &req.resource_spans[0];
            let scope = &resource.scope_spans[0];
            let span = &scope.spans[0];
            assert_eq!(span.trace_id, vec![1; 16]);
            assert_eq!(span.span_id, vec![2; 8]);
            assert_eq!(span.status.as_ref().unwrap().code, 2);
            assert_eq!(
                span.name,
                if enabled {
                    "operator.redacted.span"
                } else {
                    "private [REDACTED]"
                }
            );
            assert_eq!(
                span.events[0].name,
                if enabled {
                    "operator.redacted.event"
                } else {
                    "private [REDACTED]"
                }
            );
            for field in [
                &resource.schema_url,
                &scope.schema_url,
                &scope.scope.as_ref().unwrap().name,
                &scope.scope.as_ref().unwrap().version,
                &span.trace_state,
                &span.links[0].trace_state,
                &span.status.as_ref().unwrap().message,
            ] {
                assert_eq!(field, if enabled { "" } else { "private [REDACTED]" });
            }
            for attrs in [
                &scope.scope.as_ref().unwrap().attributes,
                &span.events[0].attributes,
                &span.links[0].attributes,
            ] {
                assert_eq!(attrs.is_empty(), enabled);
            }
            assert!(!serde_json::to_string(req)
                .unwrap()
                .contains("sk-ant-fixture"));
        }
    }

    fn log_payload(record: LogRecord) -> OtlpPayload {
        OtlpPayload::Logs(ExportLogsServiceRequest {
            resource_logs: vec![ResourceLogs {
                scope_logs: vec![ScopeLogs {
                    log_records: vec![record],
                    ..Default::default()
                }],
                ..Default::default()
            }],
        })
    }

    fn log_record(payload: &OtlpPayload) -> &LogRecord {
        let OtlpPayload::Logs(req) = payload else {
            unreachable!()
        };
        &req.resource_logs[0].scope_logs[0].log_records[0]
    }

    #[test]
    fn test_log_names_severity_and_empty_event_names() {
        let redactor = Redactor::new(&KEY).unwrap();
        for name in ["", "operator.step.started", "private sk-ant-fixture"] {
            let payload = log_payload(LogRecord {
                event_name: name.into(),
                severity_text: "sk-ant-fixture".into(),
                severity_number: 9,
                ..Default::default()
            });
            for enabled in [false, true] {
                let (out, _) = payload.redacted(&redactor, policy(enabled));
                let log = log_record(&out);
                assert_eq!(log.severity_number, 9);
                assert_eq!(log.severity_text, if enabled { "" } else { "[REDACTED]" });
                let expected = if name == "private sk-ant-fixture" {
                    if enabled {
                        "operator.redacted.event"
                    } else {
                        "private [REDACTED]"
                    }
                } else {
                    name
                };
                assert_eq!(log.event_name, expected);
            }
        }
    }

    fn wire_kv(key: &str, value: Any) -> KeyValue {
        KeyValue {
            key: key.into(),
            value: Some(AnyValue { value: Some(value) }),
            ..Default::default()
        }
    }

    #[test]
    fn test_secret_inspection_sees_bytes_duplicate_keys_and_unresolved_indices() {
        let nested = Any::KvlistValue(KeyValueList {
            values: vec![kv("same", "sk-ant-fixture"), kv("same", "safe")],
        });
        let attrs = vec![
            wire_kv("unknown.duplicate", nested),
            wire_kv("unknown.bytes", Any::BytesValue(b"sk-ant-fixture".to_vec())),
            wire_kv("unknown.opaque", Any::BytesValue(vec![0xff])),
            wire_kv("unknown.index", Any::StringValueStrindex(1)),
            KeyValue {
                key_strindex: 1,
                ..kv("service.name", "safe")
            },
        ];
        let payload = log_payload(LogRecord {
            attributes: attrs,
            ..Default::default()
        });
        for enabled in [false, true] {
            let (out, stats) = payload.redacted(&Redactor::new(&KEY).unwrap(), policy(enabled));
            assert!(log_record(&out).attributes.is_empty());
            assert_eq!(stats.dropped, 5);
        }
    }

    #[test]
    fn test_disabled_body_preserves_wire_types_and_duplicate_entries() {
        let entries = vec![
            wire_kv("same", Any::StringValue("sk-ant-fixture".into())),
            wire_kv("same", Any::StringValue("safe".into())),
            wire_kv("float", Any::DoubleValue(1.0)),
            wire_kv("large", Any::IntValue(i64::MAX)),
            wire_kv("bytes", Any::BytesValue(b"safe".to_vec())),
            wire_kv("scrub_bytes", Any::BytesValue(b"sk-ant-fixture".to_vec())),
            wire_kv("bool", Any::BoolValue(true)),
            wire_kv(
                "array",
                Any::ArrayValue(ArrayValue {
                    values: vec![AnyValue {
                        value: Some(Any::IntValue(7)),
                    }],
                }),
            ),
            wire_kv("opaque", Any::BytesValue(vec![0xff])),
            wire_kv("indexed", Any::StringValueStrindex(1)),
            kv(attr::OPERATOR_AUTH_TOKEN, "unrecognized-token"),
        ];
        let payload = log_payload(LogRecord {
            body: Some(AnyValue {
                value: Some(Any::KvlistValue(KeyValueList {
                    values: entries.clone(),
                })),
            }),
            ..Default::default()
        });
        let (out, stats) = payload.redacted(&Redactor::new(&KEY).unwrap(), policy(false));
        let Some(Any::KvlistValue(body)) = &log_record(&out).body.as_ref().unwrap().value else {
            unreachable!()
        };
        assert_eq!(body.values.len(), 8);
        assert_eq!(body.values[0], kv("same", "[REDACTED]"));
        assert_eq!(body.values[1..5], entries[1..5]);
        assert_eq!(
            body.values[5],
            wire_kv("scrub_bytes", Any::BytesValue(b"[REDACTED]".to_vec()))
        );
        assert_eq!(body.values[6..8], entries[6..8]);
        assert_eq!(stats.dropped, 3);
        assert_eq!(stats.scrubbed, 2);
        let (out, stats) = payload.redacted(&Redactor::new(&KEY).unwrap(), policy(true));
        assert!(log_record(&out).body.is_none());
        assert_eq!(stats.dropped, 1);
    }

    #[test]
    fn test_enabled_wire_types_cannot_masquerade_as_public_strings() {
        let payload = traces(
            vec![],
            vec![
                wire_kv("service.name", Any::BytesValue(b"private".to_vec())),
                wire_kv(
                    "service.version",
                    Any::KvlistValue(KeyValueList {
                        values: vec![kv("prompt", "private")],
                    }),
                ),
                wire_kv("operator.exit_code", Any::StringValue("private".into())),
            ],
        );
        let (out, stats) = payload.redacted(&Redactor::new(&KEY).unwrap(), policy(true));
        assert!(span_attrs(&out).is_empty());
        assert_eq!(stats.dropped, 3);
    }

    #[test]
    fn test_metric_names_descriptions_units_and_all_point_kinds() {
        use opentelemetry_proto::tonic::metrics::v1::{
            ExponentialHistogram, ExponentialHistogramDataPoint, Gauge, Histogram,
            HistogramDataPoint, NumberDataPoint, ResourceMetrics, ScopeMetrics, Sum, Summary,
            SummaryDataPoint,
        };
        let attrs = || {
            vec![
                kv("unknown", "private"),
                kv(attr::OPERATOR_AUTH_TOKEN, "opaque"),
                kv("user.name", "alice"),
            ]
        };
        let exemplars = || {
            vec![Exemplar {
                filtered_attributes: attrs(),
                ..Default::default()
            }]
        };
        let points = || {
            vec![NumberDataPoint {
                attributes: attrs(),
                exemplars: exemplars(),
                ..Default::default()
            }]
        };
        let kinds = vec![
            Data::Gauge(Gauge {
                data_points: points(),
            }),
            Data::Sum(Sum {
                data_points: points(),
                ..Default::default()
            }),
            Data::Histogram(Histogram {
                data_points: vec![HistogramDataPoint {
                    attributes: attrs(),
                    exemplars: exemplars(),
                    ..Default::default()
                }],
                ..Default::default()
            }),
            Data::ExponentialHistogram(ExponentialHistogram {
                data_points: vec![ExponentialHistogramDataPoint {
                    attributes: attrs(),
                    exemplars: exemplars(),
                    ..Default::default()
                }],
                ..Default::default()
            }),
            Data::Summary(Summary {
                data_points: vec![SummaryDataPoint {
                    attributes: attrs(),
                    ..Default::default()
                }],
            }),
        ];
        let approved = SIGNALS
            .iter()
            .find(|s| s.name == "operator.agents.active")
            .unwrap();
        for data in kinds {
            let mut metrics = vec![Metric {
                name: approved.name.into(),
                description: "sk-ant-fixture".into(),
                unit: "private".into(),
                metadata: attrs(),
                data: Some(data),
            }];
            metrics.push(Metric {
                name: "unknown.sk-ant-fixture".into(),
                metadata: attrs(),
                ..Default::default()
            });
            let payload = OtlpPayload::Metrics(ExportMetricsServiceRequest {
                resource_metrics: vec![ResourceMetrics {
                    resource: Some(Resource {
                        attributes: attrs(),
                        ..Default::default()
                    }),
                    scope_metrics: vec![ScopeMetrics {
                        metrics,
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
            });
            for enabled in [false, true] {
                let (out, stats) = payload.redacted(&Redactor::new(&KEY).unwrap(), policy(enabled));
                let OtlpPayload::Metrics(req) = &out else {
                    unreachable!()
                };
                let metrics = &req.resource_metrics[0].scope_metrics[0].metrics;
                assert_eq!(metrics.len(), if enabled { 1 } else { 2 });
                assert_eq!(
                    metrics[0].description,
                    if enabled {
                        approved.description
                    } else {
                        "[REDACTED]"
                    }
                );
                assert_eq!(
                    metrics[0].unit,
                    if enabled {
                        approved.unit.unwrap()
                    } else {
                        "private"
                    }
                );
                let serialized = serde_json::to_string(req).unwrap();
                assert!(!serialized.contains("opaque"));
                assert!(!serialized.contains("sk-ant-fixture"));
                assert_eq!(serialized.contains("alice"), !enabled);
                assert_eq!(serialized.contains("private"), !enabled);
                assert!(stats.dropped > 0);
            }
        }
        let payload = OtlpPayload::Metrics(ExportMetricsServiceRequest {
            resource_metrics: vec![ResourceMetrics {
                scope_metrics: vec![ScopeMetrics {
                    metrics: vec![Metric {
                        name: "unknown".into(),
                        metadata: attrs(),
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }],
        });
        let (_, stats) = payload.redacted(&Redactor::new(&KEY).unwrap(), policy(true));
        assert_eq!(
            stats.dropped, 1,
            "a removed metric counts once without visiting its fields"
        );
    }

    #[test]
    fn test_signal_and_record_count() {
        use opentelemetry_proto::tonic::metrics::v1::{
            Gauge, NumberDataPoint, ResourceMetrics, ScopeMetrics,
        };
        use opentelemetry_proto::tonic::trace::v1::Span;

        let mut traces = traces(vec![], vec![]);
        let OtlpPayload::Traces(req) = &mut traces else {
            unreachable!();
        };
        req.resource_spans[0].scope_spans[0]
            .spans
            .push(Span::default());
        assert_eq!(traces.signal(), TelemetrySignal::Traces);
        assert_eq!(traces.record_count(), 2);

        let logs = OtlpPayload::Logs(ExportLogsServiceRequest {
            resource_logs: vec![opentelemetry_proto::tonic::logs::v1::ResourceLogs {
                scope_logs: vec![opentelemetry_proto::tonic::logs::v1::ScopeLogs {
                    log_records: vec![opentelemetry_proto::tonic::logs::v1::LogRecord::default()],
                    ..Default::default()
                }],
                ..Default::default()
            }],
        });
        assert_eq!(logs.signal(), TelemetrySignal::Logs);
        assert_eq!(logs.record_count(), 1);

        let metrics = OtlpPayload::Metrics(ExportMetricsServiceRequest {
            resource_metrics: vec![ResourceMetrics {
                scope_metrics: vec![ScopeMetrics {
                    metrics: vec![Metric {
                        data: Some(Data::Gauge(Gauge {
                            data_points: vec![
                                NumberDataPoint::default(),
                                NumberDataPoint::default(),
                                NumberDataPoint::default(),
                            ],
                        })),
                        ..Metric::default()
                    }],
                    ..ScopeMetrics::default()
                }],
                ..ResourceMetrics::default()
            }],
        });
        assert_eq!(metrics.signal(), TelemetrySignal::Metrics);
        assert_eq!(metrics.record_count(), 3);
    }
}

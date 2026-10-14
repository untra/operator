use operator_telemetry::{
    attr,
    otlp_json::{self, AnyValue, KeyValue, LogRecord, Span, Status},
    traceparent::TraceParent,
    EventName, SpanName,
};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub const SEND_TIMEOUT: Duration = Duration::from_secs(2);
const FLUSH_TIMEOUT: Duration = Duration::from_secs(4);
const SEND_ATTEMPTS: usize = 2;
const ENDPOINT_ENV: &str = "OTEL_EXPORTER_OTLP_ENDPOINT";
const TOKEN_ENV: &str = "OPERATOR_OTLP_TOKEN";

#[derive(Clone)]
pub struct StepTelemetry {
    client: reqwest::Client,
    endpoint: String,
    token: String,
    parent: Option<TraceParent>,
    span: TraceParent,
    started: u64,
    verbose: bool,
}
fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos()
        .min(u128::from(u64::MAX)) as u64
}
fn string(key: &str, value: &str) -> KeyValue {
    KeyValue {
        key: key.into(),
        value: AnyValue::StringValue(value.into()),
    }
}
fn int(key: &str, value: i64) -> KeyValue {
    KeyValue {
        key: key.into(),
        value: AnyValue::IntValue(value),
    }
}

impl StepTelemetry {
    pub fn from_env(verbose: bool) -> Option<Self> {
        Self::from_values(&|key| std::env::var(key).ok(), verbose)
    }
    fn from_values(env: &dyn Fn(&str) -> Option<String>, verbose: bool) -> Option<Self> {
        let endpoint = env(ENDPOINT_ENV).filter(|v| !v.is_empty())?;
        let url = reqwest::Url::parse(&endpoint).ok()?;
        if !matches!(url.scheme(), "http" | "https") {
            return None;
        }
        let token = env(TOKEN_ENV).filter(|v| !v.is_empty())?;
        let parent = env("TRACEPARENT").as_deref().and_then(TraceParent::parse);
        let span = TraceParent {
            trace_id: parent.map_or_else(|| *uuid::Uuid::new_v4().as_bytes(), |p| p.trace_id),
            span_id: uuid::Uuid::new_v4().as_bytes()[..8].try_into().ok()?,
            sampled: parent.is_none_or(|p| p.sampled),
        };
        Some(Self {
            client: crate::http_client::client_builder()
                .timeout(SEND_TIMEOUT)
                .build()
                .ok()?,
            endpoint: endpoint.trim_end_matches('/').to_owned(),
            token,
            parent,
            span,
            started: now(),
            verbose,
        })
    }
    pub fn child_traceparent(&self) -> String {
        self.span.to_string()
    }
    fn event(&self, name: EventName, attributes: Vec<KeyValue>) -> serde_json::Value {
        otlp_json::logs_request(
            vec![string("service.name", "opr8r")],
            "opr8r",
            vec![LogRecord {
                time_unix_nano: now(),
                severity_number: otlp_json::SEVERITY_INFO,
                event_name: name.as_str().into(),
                trace_id: self.span.trace_id_hex(),
                span_id: self.span.span_id_hex(),
                attributes,
            }],
        )
    }
    pub async fn step_started(&self, ticket: &str, step: &str) {
        self.send(
            "logs",
            &self.event(
                EventName::OperatorStepStarted,
                vec![
                    string(attr::OPERATOR_TICKET_ID, ticket),
                    string(attr::OPERATOR_STEP_NAME, step),
                ],
            ),
        )
        .await;
    }
    fn completion(
        &self,
        ticket: &str,
        step: &str,
        exit_code: i32,
        duration: u64,
        output: Option<&crate::api::OperatorOutput>,
    ) -> (serde_json::Value, serde_json::Value) {
        let mut attributes = vec![
            string(attr::OPERATOR_TICKET_ID, ticket),
            string(attr::OPERATOR_STEP_NAME, step),
            int(attr::OPERATOR_EXIT_CODE, i64::from(exit_code)),
            int(
                attr::OPERATOR_DURATION,
                duration.min(i64::MAX as u64) as i64,
            ),
        ];
        if let Some(output) = output {
            attributes.push(string(attr::OPERATOR_OUTPUT_STATUS, &output.status));
            attributes.push(KeyValue {
                key: attr::OPERATOR_OUTPUT_EXIT_SIGNAL.into(),
                value: AnyValue::BoolValue(output.exit_signal),
            });
            for (key, value) in [
                (
                    attr::OPERATOR_OUTPUT_CONFIDENCE,
                    output.confidence.map(i64::from),
                ),
                (
                    attr::OPERATOR_OUTPUT_FILES_MODIFIED,
                    output.files_modified.map(i64::from),
                ),
                (
                    attr::OPERATOR_OUTPUT_ERROR_COUNT,
                    output.error_count.map(i64::from),
                ),
                (
                    attr::OPERATOR_OUTPUT_TASKS_COMPLETED,
                    output.tasks_completed.map(i64::from),
                ),
                (
                    attr::OPERATOR_OUTPUT_TASKS_REMAINING,
                    output.tasks_remaining.map(i64::from),
                ),
            ] {
                if let Some(value) = value {
                    attributes.push(int(key, value));
                }
            }
            if let Some(status) = &output.tests_status {
                attributes.push(string(attr::OPERATOR_OUTPUT_TESTS_STATUS, status));
            }
        }
        let logs = self.event(EventName::OperatorStepCompleted, attributes.clone());
        let traces = otlp_json::traces_request(
            vec![string("service.name", "opr8r")],
            "opr8r",
            vec![Span {
                trace_id: self.span.trace_id_hex(),
                span_id: self.span.span_id_hex(),
                parent_span_id: self
                    .parent
                    .map_or_else(String::new, TraceParent::span_id_hex),
                name: SpanName::Opr8rStep.as_str().into(),
                kind: otlp_json::SPAN_KIND_INTERNAL,
                start_time_unix_nano: self.started,
                end_time_unix_nano: now(),
                attributes,
                status: Status {
                    code: if exit_code == 0 {
                        otlp_json::STATUS_CODE_OK
                    } else {
                        otlp_json::STATUS_CODE_ERROR
                    },
                },
            }],
        );
        (traces, logs)
    }
    pub async fn step_completed(
        &self,
        ticket: &str,
        step: &str,
        exit_code: i32,
        duration: u64,
        output: Option<&crate::api::OperatorOutput>,
    ) {
        let (traces, logs) = self.completion(ticket, step, exit_code, duration, output);
        let _ = tokio::time::timeout(FLUSH_TIMEOUT, async {
            tokio::join!(self.send("traces", &traces), self.send("logs", &logs));
        })
        .await;
    }
    async fn send(&self, signal: &str, body: &serde_json::Value) {
        for _ in 0..SEND_ATTEMPTS {
            let sent = tokio::time::timeout(
                SEND_TIMEOUT,
                self.client
                    .post(format!("{}/v1/{signal}", self.endpoint))
                    .bearer_auth(&self.token)
                    .json(body)
                    .send(),
            )
            .await;
            if sent
                .is_ok_and(|response| response.is_ok_and(|response| response.status().is_success()))
            {
                return;
            }
        }
        if self.verbose {
            eprintln!("[opr8r] telemetry delivery failed");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_requires_endpoint_and_token() {
        assert!(StepTelemetry::from_values(&|_| None, false).is_none());
        assert!(StepTelemetry::from_values(
            &|key| (key == ENDPOINT_ENV).then(|| "http://local/otlp".into()),
            false
        )
        .is_none());
    }
    #[test]
    fn test_completion_preserves_parent_and_excludes_content() {
        let parent = TraceParent {
            trace_id: [1; 16],
            span_id: [2; 8],
            sampled: true,
        };
        let telemetry = StepTelemetry::from_values(
            &|key| match key {
                ENDPOINT_ENV => Some("http://local/otlp".into()),
                TOKEN_ENV => Some("token".into()),
                "TRACEPARENT" => Some(parent.to_string()),
                _ => None,
            },
            false,
        )
        .unwrap();
        let child = TraceParent::parse(&telemetry.child_traceparent()).unwrap();
        assert_eq!(child.trace_id, parent.trace_id);
        assert_ne!(child.span_id, parent.span_id);
        let output = crate::api::OperatorOutput {
            status: "complete".into(),
            exit_signal: true,
            confidence: Some(90),
            summary: Some("PRIVATE".into()),
            recommendation: Some("PRIVATE".into()),
            blockers: Some(vec!["PRIVATE".into()]),
            ..Default::default()
        };
        let (traces, logs) = telemetry.completion("t", "s", 0, 1, Some(&output));
        assert!(!traces.to_string().contains("PRIVATE"));
        assert!(!logs.to_string().contains("PRIVATE"));
        assert!(logs.to_string().contains(attr::OPERATOR_OUTPUT_CONFIDENCE));
    }
    #[tokio::test]
    async fn test_slow_endpoint_respects_shared_flush_deadline() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let endpoint = format!("http://{}", listener.local_addr().unwrap());
        let task = tokio::spawn(async move {
            let mut connections = Vec::new();
            loop {
                connections.push(listener.accept().await.unwrap());
            }
        });
        let telemetry = StepTelemetry::from_values(
            &|key| match key {
                ENDPOINT_ENV => Some(endpoint.clone()),
                TOKEN_ENV => Some("token".into()),
                _ => None,
            },
            false,
        )
        .unwrap();
        let start = std::time::Instant::now();
        telemetry.step_completed("t", "s", 0, 1, None).await;
        assert!(start.elapsed() < Duration::from_millis(4500));
        task.abort();
    }
}

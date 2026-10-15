use std::collections::{BTreeMap, HashMap};

use opentelemetry::trace::{Span, TraceContextExt, Tracer};
use operator_telemetry::{attr, traceparent::TraceParent, SpanName};
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};

use crate::config::Config;

pub const OTLP_PATH: &str = "/otlp";
pub const TOKEN_ENV: &str = "OPERATOR_OTLP_TOKEN";
pub const ENDPOINT_ENV: &str = "OTEL_EXPORTER_OTLP_ENDPOINT";
pub const PROPAGATE_ENV: &str = "CLAUDE_CODE_PROPAGATE_TRACEPARENT";
const RESOURCE_ENV: &str = "OTEL_RESOURCE_ATTRIBUTES";
const HEADERS_ENV: &str = "OTEL_EXPORTER_OTLP_HEADERS";
const PROTOCOL_ENV: &str = "OTEL_EXPORTER_OTLP_PROTOCOL";
const PROTOCOL: &str = "http/protobuf";

pub fn inherited_otel_env() -> Vec<String> {
    let mut names = Vec::new();
    for signal in ["", "TRACES_", "METRICS_", "LOGS_"] {
        for field in [
            "ENDPOINT",
            "HEADERS",
            "PROTOCOL",
            "TIMEOUT",
            "COMPRESSION",
            "CERTIFICATE",
            "CLIENT_KEY",
            "CLIENT_CERTIFICATE",
        ] {
            names.push(format!("OTEL_EXPORTER_OTLP_{signal}{field}"));
        }
    }
    names.extend(
        [
            RESOURCE_ENV,
            "OTEL_SERVICE_NAME",
            "OTEL_SDK_DISABLED",
            "OTEL_TRACES_EXPORTER",
            "OTEL_METRICS_EXPORTER",
            "OTEL_LOGS_EXPORTER",
            "TRACEPARENT",
            "TRACESTATE",
            TOKEN_ENV,
            PROPAGATE_ENV,
            "BETA_TRACING_ENDPOINT",
            "GEMINI_TELEMETRY_OTLP_ENDPOINT",
            "GEMINI_TELEMETRY_OUTFILE",
        ]
        .map(str::to_owned),
    );
    names
}

pub fn step_rotation_env(
    token: &str,
    parent: TraceParent,
    ticket: &str,
    step: &str,
    session: &str,
    profile: &str,
) -> BTreeMap<String, String> {
    let resource = [
        (attr::OPERATOR_TICKET_ID, ticket),
        (attr::OPERATOR_STEP_NAME, step),
        (attr::OPERATOR_SESSION_ID, session),
        (attr::OPERATOR_PROFILE_ID, profile),
    ]
    .map(|(key, value)| format!("{key}={}", utf8_percent_encode(value, NON_ALPHANUMERIC)))
    .join(",");
    BTreeMap::from([
        ("TRACEPARENT".into(), parent.to_string()),
        (TOKEN_ENV.into(), token.into()),
        (
            HEADERS_ENV.into(),
            format!(
                "Authorization={}",
                utf8_percent_encode(&format!("Bearer {token}"), NON_ALPHANUMERIC)
            ),
        ),
        (RESOURCE_ENV.into(), resource),
    ])
}

pub fn launch_span(
    ticket: &str,
    step: &str,
    session: &str,
    parent: Option<TraceParent>,
) -> TraceParent {
    let mut context = opentelemetry::Context::new();
    if let Some(parent) = parent {
        context = context.with_remote_span_context(opentelemetry::trace::SpanContext::new(
            opentelemetry::trace::TraceId::from_bytes(parent.trace_id),
            opentelemetry::trace::SpanId::from_bytes(parent.span_id),
            if parent.sampled {
                opentelemetry::trace::TraceFlags::SAMPLED
            } else {
                opentelemetry::trace::TraceFlags::default()
            },
            true,
            opentelemetry::trace::TraceState::default(),
        ));
    }
    let mut span = opentelemetry::global::tracer("operator")
        .start_with_context(SpanName::OperatorAgentLaunch.as_str(), &context);
    for (key, value) in [
        (attr::OPERATOR_TICKET_ID, ticket),
        (attr::OPERATOR_STEP_NAME, step),
        (attr::OPERATOR_SESSION_ID, session),
    ] {
        span.set_attribute(opentelemetry::KeyValue::new(key, value.to_owned()));
    }
    let result = if span.span_context().is_valid()
        && parent.is_none_or(|parent| span.span_context().span_id().to_bytes() != parent.span_id)
    {
        TraceParent {
            trace_id: span.span_context().trace_id().to_bytes(),
            span_id: span.span_context().span_id().to_bytes(),
            sampled: span.span_context().is_sampled(),
        }
    } else {
        TraceParent {
            trace_id: parent.map_or_else(|| *uuid::Uuid::new_v4().as_bytes(), |p| p.trace_id),
            span_id: uuid::Uuid::new_v4().as_bytes()[..8]
                .try_into()
                .expect("span id"),
            sampled: parent.is_none_or(|p| p.sampled),
        }
    };
    span.end();
    result
}

pub fn render_tool_env(
    map: &BTreeMap<String, String>,
    endpoint: &str,
) -> anyhow::Result<BTreeMap<String, String>> {
    map.iter()
        .map(|(key, value)| {
            anyhow::ensure!(
                !key.is_empty()
                    && key.bytes().enumerate().all(|(i, b)| b == b'_'
                        || b.is_ascii_alphabetic()
                        || (i > 0 && b.is_ascii_digit())),
                "invalid telemetry environment name"
            );
            let value = value
                .replace("{{otlp_endpoint}}", endpoint)
                .replace("{{otlp_protocol}}", PROTOCOL);
            anyhow::ensure!(
                !value.contains("{{") && !value.contains("}}"),
                "unknown telemetry placeholder"
            );
            Ok((key.clone(), value))
        })
        .collect()
}

pub fn launch_env(
    config: &Config,
    context: (&str, &str, &str),
    api_url: &str,
    tool: &str,
    model_server: bool,
) -> HashMap<String, String> {
    if !config.telemetry.enabled || !config.telemetry.ingest.inject_agent_env {
        return HashMap::new();
    }
    let (ticket, step, session) = context;
    let result = (|| -> anyhow::Result<HashMap<String, String>> {
        let token = crate::auth::callback::mint_telemetry(config, ticket, step, session)?;
        let endpoint = format!("{}{OTLP_PATH}", api_url.trim_end_matches('/'));
        let mut env = BTreeMap::new();
        if let Some(tool) = crate::llm::tool_config::load_all_tool_configs()
            .into_iter()
            .find(|t| t.tool_name == tool)
        {
            env.extend(render_tool_env(&tool.telemetry_env, &endpoint)?);
        }
        env.extend(step_rotation_env(
            &token,
            launch_span(ticket, step, session, None),
            ticket,
            step,
            session,
            &config.profile.id.to_string(),
        ));
        env.insert(ENDPOINT_ENV.into(), endpoint);
        env.insert(PROTOCOL_ENV.into(), PROTOCOL.into());
        if tool == "claude" && model_server {
            env.insert(PROPAGATE_ENV.into(), "1".into());
        }
        Ok(env.into_iter().collect())
    })();
    result.unwrap_or_else(|_| {
        tracing::warn!("agent telemetry initialization failed");
        HashMap::new()
    })
}

pub fn retarget(env: &mut HashMap<String, String>, api_url: &str) {
    if env.contains_key(ENDPOINT_ENV) {
        let endpoint = format!("{}{OTLP_PATH}", api_url.trim_end_matches('/'));
        env.insert(ENDPOINT_ENV.into(), endpoint.clone());
        if let Some(value) = env.get_mut("GEMINI_TELEMETRY_OTLP_ENDPOINT") {
            *value = endpoint;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_rotation_encodes_values_and_preserves_context() {
        let parent = TraceParent {
            trace_id: [1; 16],
            span_id: [2; 8],
            sampled: true,
        };
        let env = step_rotation_env("token", parent, "t", "a,b=c", "s", "p");
        assert_eq!(env[HEADERS_ENV], "Authorization=Bearer%20token");
        assert!(env[RESOURCE_ENV].contains("a%2Cb%3Dc"));
        assert_eq!(TraceParent::parse(&env["TRACEPARENT"]), Some(parent));
    }
    #[test]
    fn test_template_rejects_unknown_and_invalid_names() {
        for (name, value) in [("A", "{{token}}"), ("A;evil", "value")] {
            assert!(render_tool_env(
                &BTreeMap::from([(name.into(), value.into())]),
                "http://local/otlp"
            )
            .is_err());
        }
    }
    #[test]
    fn test_launch_disabled_does_not_create_credentials() {
        assert!(launch_env(
            &Config::default(),
            ("t", "s", "a"),
            "http://local",
            "claude",
            true
        )
        .is_empty());
    }
}

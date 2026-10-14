---
layout: doc
title: Telemetry
---

Operator accepts OTLP/HTTP on its existing REST port when explicitly enabled.
Telemetry is off by default and Operator never selects a destination of its own.
Operator provides authenticated ingestion, a bounded session window, upstream
forwarding, self-export, and agent environment injection.

## Configuration

```toml
[telemetry]
enabled = true
enable_redaction = false
```

These switches are independent and both default to false. Set
`enable_redaction = true` to hash personal attributes and remove content before
queueing or storing telemetry. The environment override is
`OPERATOR_TELEMETRY__ENABLE_REDACTION=true`.

With redaction disabled, personal data, prompts, code, tool arguments, unknown
attributes, and log bodies can be retained. Secret-class attributes are always
dropped, matching attribute values are removed, and retained text is scrubbed in
both modes. Regex scrubbing cannot recognize every credential format.

The [OTLP Event Catalog](/telemetry/catalog/) defines the five classes, fixed
policy matrix, signal names, and redaction counting rules. There are no per-sink
thresholds or content opt-in switches. Obsolete `allow_content` and
`max_redaction_class` settings are rejected as unknown fields.

## Sanitization boundary

Every accepted payload is enriched from trusted authentication context and then
sanitized once before entering the bounded queue. The session window and
forwarder use that same sanitized payload. Personal hashes use a persisted
installation key; they correlate across restarts but change when the key changes.
Hashes are pseudonyms, not anonymization or unique identifiers.

With redaction enabled, unapproved span and event names become
`operator.redacted.span` and `operator.redacted.event`; unnamed log events stay
unnamed. Unknown metrics are dropped. Auxiliary text is removed and known metric
descriptions and units come from the catalog. Trace IDs, span IDs, timestamps,
and numeric protocol fields retain their protocol meaning.

## Ingest and session access

Send protobuf or OTLP JSON, optionally gzip-compressed, to
`/otlp/v1/traces`, `/otlp/v1/metrics`, or `/otlp/v1/logs`.
Ingest credentials use the `otlp-ingest` audience and pin ticket, step, and session
context; ordinary callback tokens are not ingest credentials. An ingest token
cannot read the API. Administrator credentials may also ingest.

The server overwrites client-supplied Operator correlation attributes and rejects
malformed or oversized requests. A full queue returns 503 with `Retry-After`.
Authorized API readers can inspect
`GET /api/v1/telemetry/sessions/{session_id}`. Records are bounded in memory and
may be evicted; Operator is not a telemetry storage backend.

## Export and self-export

Configure an upstream explicitly or use standard environment variables:

```toml
[telemetry.export]
endpoint = "http://collector:4318"
headers_env = "MY_OTLP_HEADERS"
self_telemetry = true
```

For each signal, an explicit config endpoint wins; otherwise a signal-specific
`OTEL_EXPORTER_OTLP_TRACES_ENDPOINT`, `OTEL_EXPORTER_OTLP_METRICS_ENDPOINT`, or
`OTEL_EXPORTER_OTLP_LOGS_ENDPOINT` is used unchanged. The generic
`OTEL_EXPORTER_OTLP_ENDPOINT` has `/v1/<signal>` appended. With no destination,
export remains off. Configuration is resolved once at startup.

Explicit `protocol`, `compression`, and `timeout_secs` override environment values,
including when equal to their defaults. When omitted, use
`OTEL_EXPORTER_OTLP_PROTOCOL`, `OTEL_EXPORTER_OTLP_COMPRESSION`, and
`OTEL_EXPORTER_OTLP_TIMEOUT` (milliseconds), with effective defaults of
`http/protobuf`, `gzip`, and ten seconds. `grpc` warns and falls back to
`http/protobuf`. Headers come from `headers_env` when configured, otherwise
`OTEL_EXPORTER_OTLP_HEADERS`; values are percent-decoded.

`OTEL_TRACES_EXPORTER=none`, `OTEL_METRICS_EXPORTER=none`, and
`OTEL_LOGS_EXPORTER=none` disable their respective signals. Standard endpoint
environment variables enable Operator self-export without enabling ingest.
`OTEL_SDK_DISABLED=true` always disables Operator's SDK providers; explicitly
configured forwarding remains available when ingest is enabled. Setting
`self_telemetry=false` disables self-export independently of forwarding.

Operator's own spans, logs, notification events, auth audit events, and metrics
pass through the same sanitization policy before export. Console/file formatting
is separate: `logging.format = "json"` emits JSON lines with trace and span IDs
when the event belongs to a traced span. `OPERATOR_LOGGING__FORMAT=json` selects
that format through the environment.

The forwarder uses bounded queues and retries network failures and HTTP
429/502/503/504 with jitter and bounded `Retry-After` delays. Retry overflow drops
the oldest batch. Forwarding failure does not block ingest; ingest's own full
queue still returns 503. Shutdown flushes within the configured export budget.
The catalog defines queue depth, active agent, step-duration, and per-signal
received/dropped/forwarded/redacted metrics.

## Agent environment and exec chains

When `telemetry.enabled` and `telemetry.ingest.inject_agent_env` are true,
launches receive a target-reachable `<OPERATOR_API_URL>/otlp` endpoint,
OTLP protocol, bearer header, resource correlation attributes, `TRACEPARENT`,
and `OPERATOR_OTLP_TOKEN`. Local, Docker, SSH, and Coder targets use their own
reachable API URL. Inherited exporter credentials, destinations, resource
attributes, and trace context are scrubbed before injecting the agent values.

Each chained step receives a fresh ingest token and trace context through
`next_env`. The callback credential is also rotated for the next step. A failed
telemetry credential rotation clears the old ingest credential. Session retention
starts only when automatic chaining ends, including `opr8r --no-auto-proceed`;
late records retain terminal state.

opr8r exports a started event, a completed step span, and a completed event with
classified scalar output fields. It never exports captured stdout/stderr,
summaries, recommendations, or blockers. Completion is reported before flushing
telemetry; concurrent completion sends share a four-second deadline, with at most
two two-second attempts each. Telemetry failure does not change the exit status.

## Agent CLIs

Vendor contracts checked October 2026:

| CLI | Variables | Contract |
| --- | --- | --- |
| Claude | `CLAUDE_CODE_ENABLE_TELEMETRY=1`, `OTEL_{TRACES,METRICS,LOGS}_EXPORTER=otlp`, `CLAUDE_CODE_ENHANCED_TELEMETRY_BETA=1` | [Monitoring](https://code.claude.com/docs/en/monitoring-usage) |
| Claude gateway | `CLAUDE_CODE_PROPAGATE_TRACEPARENT=1` when a model server is resolved | [Trace propagation](https://code.claude.com/docs/en/monitoring-usage#traces-beta) |
| Gemini | `GEMINI_TELEMETRY_ENABLED=true`, `GEMINI_TELEMETRY_TARGET=local`, `GEMINI_TELEMETRY_OTLP_ENDPOINT`, `GEMINI_TELEMETRY_OTLP_PROTOCOL=http`, `GEMINI_TELEMETRY_LOG_PROMPTS=false` | [Telemetry configuration](https://geminicli.com/docs/cli/telemetry/) |

Claude non-interactive `-p` sessions accept inbound context, permitting
`operator.agent.launch -> opr8r.step -> claude_code.* -> model-server gateway`.
Interactive Claude sessions ignore inbound context; correlate those by trusted
Operator resource attributes. Gemini's HTTP exporter may use OTLP JSON, which
Operator accepts alongside protobuf. Codex and Grok receive only standard
variables; no vendor-specific telemetry contract is asserted.

## Kubernetes

The Helm chart configures existing collectors; it never installs one. Use
`telemetry.export.endpoint` for self-export, and separately enable
`telemetry.enabled` for agent ingest. Header credentials reference an existing
Secret. Pod name and namespace are supplied through the Downward API. Optional
collector egress restricts traffic to specified peers and a TCP port. See the
[chart configuration](https://github.com/untra/operator/tree/main/charts/operator).

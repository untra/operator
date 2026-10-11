---
layout: doc
title: Telemetry
---

Operator accepts OTLP/HTTP on its existing REST port when explicitly enabled.
Telemetry is off by default and Operator never selects a destination of its own.
The current implementation provides authenticated ingestion and a bounded session
window; upstream forwarding and agent environment injection are later phases.

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
sanitized once before entering the bounded queue. The session window and future
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

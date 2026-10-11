# Operator telemetry registry

This registry is the source of truth for telemetry names, attributes, redaction
classes, and secret patterns. The server applies this policy to incoming OTLP
before its bounded queue and session window; exporters and agent wiring follow
in later phases.

## Layout and generation

- `registry_manifest.yaml`: version and pinned OTel semantic-convention dependency.
- `model/`: Weaver semantic-convention groups. Attribute definitions carry
  `annotations: {operator: {redaction_class: internal}}`; references inherit that
  classification. Define an attribute once, then use `ref` on signals.
- `redaction/classes.yaml`: fixed classes and actions.
- `redaction/secret_patterns.yaml`: reviewed Rust regex patterns.

`crates/telemetry/build.rs` validates local YAML and generates Rust and Markdown
into Cargo's `OUT_DIR`. Normal builds are offline with respect to the registry;
they never resolve the upstream registry. The reviewed third-party definitions
are local, explicit classifications, not an import-wide permission grant.

Run `cargo run --bin generate_types` to publish `docs/telemetry/catalog.md` and
export the `RedactionClass` TypeScript type. Run
`cargo run --bin generate_types -- --check` to detect stale artifacts without
rewriting them. Do not edit generated Rust, bindings, or the catalog manually.

Install Weaver 0.27.0 and run `make telemetry-registry` to validate the full
registry and pinned upstream dependency. Set `OPERATOR_WEAVER` to an explicit
binary path if needed. The script stages only model files and the manifest in a
temporary directory: Weaver scans YAML recursively, while `redaction/` contains
Operator policy data rather than semantic-convention groups. Cargo validates
that policy data separately. Weaver validation may fetch its pinned dependency.

## Adding a signal

1. Add attribute definitions with types, descriptions, examples, development
   stability, and a redaction class. Unknown attributes default to content.
2. Add an event, span, or metric using attribute references. Events cannot
   reference secret-class attributes. Metrics require an instrument and unit.
3. Add tests for its names, classifications, and any new secret patterns.
4. Run `make telemetry`, regenerate the catalog, and review the generated diff.

The parser deliberately supports only the constructs used here. Unsupported
fields, imports, inheritance, and reference overrides fail the build instead of
being ignored. Add support with validation tests before using another construct.
Definitions and generated identifiers must be unique. All class actions are
validated against the security contract, so changing YAML cannot weaken it.

## Redaction contract

`telemetry.enable_redaction` defaults to false and is independent of
`telemetry.enabled`, which also defaults to false.
The [generated catalog](../docs/telemetry/catalog.md) defines the fixed five-class
matrix from `redaction/classes.yaml`; there are no per-sink thresholds.

Secrets are always removed, even with optional redaction disabled. Secret matches
in attribute keys or nested values drop the whole attribute; retained free text
is scrubbed. Pattern matching is a best-effort backstop, not a guarantee that all
credential formats are recognized.

When enabled, personal attributes are HMAC-pseudonymized and content (including
unknown attributes and log bodies) is dropped. Only catalog signal names survive:
unknown spans and events use catalog placeholders, and unknown metrics are dropped.
Unclassified auxiliary text is removed and retained metrics use catalog metadata.
Catalog attribute value types are enforced. When disabled, personal data and
content are retained, subject to mandatory secret protection.

The OTLP adapter visits resources, scopes, records, events, links, data points,
and exemplars. It checks byte values as UTF-8 before encoding, drops opaque bytes
and unsupported string-table references, and inspects duplicate map entries
before conversion can hide a secret. Retained OTLP values keep their wire types.

Enrichment precedes one sanitization pass before queueing. The window and future
forwarder consume the same sanitized payload; never hash it again or trust an
incoming `hmac:` prefix. Incoming `operator.redacted.count` is discarded and
replaced with authoritative per-resource dropped, hashed, and scrubbed totals.
The generated catalog documents counting rules.

## Installation key and features

`Redactor::new` accepts a caller-supplied 32-byte key and retains an HMAC state,
without a `Debug` implementation. The server persists the key through its authentication store. The crate performs no key I/O. HMAC input is compact JSON with
recursively sorted object keys, preserving JSON types and array order. Equal
values under one installation key correlate across fields; key rotation changes
the hashes. Truncation is for pseudonymous correlation, not unique identifiers.

- Default: serde class enum, constants, name enums, catalog metadata.
- `schema`: schemars and ts-rs derives.
- `redact`: regex, HMAC, SHA-256, and the JSON value adapter.

There are no OTel SDK, HTTP, runtime, or filesystem dependencies in core.
Build dependencies parse and validate the registry; opr8r does not enable
runtime redaction dependencies. HMAC 0.12 uses SHA-256 0.10 to match its digest
API; this is independent of Operator's existing SHA-256 0.11 use.

## Upstream references

- [Weaver syntax](https://github.com/open-telemetry/weaver/blob/main/schemas/semconv-syntax.md)
- [Claude Code monitoring](https://code.claude.com/docs/en/monitoring-usage)
- [GenAI conventions](https://opentelemetry.io/docs/specs/semconv/gen-ai/)

Claude Code event names use `claude_code.*`, but many attributes do not have
that prefix. This registry intentionally covers an explicit subset; newly added
vendor fields remain content until reviewed.

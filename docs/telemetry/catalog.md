---
layout: doc
title: OTLP Event Catalog
---

<!-- AUTO-GENERATED FROM telemetry/ - DO NOT EDIT MANUALLY -->
<!-- Regenerate with: cargo run --bin generate_types -->

The catalog defines the classifications and approved signal names used by Operator OTLP ingestion.

## Redaction policy

Telemetry and optional redaction are independently disabled by default. Set `telemetry.enable_redaction = true` to hash personal attributes and drop content. Secret attributes and attributes containing detected credentials are dropped in both modes; retained free text is scrubbed. Unknown attributes are content, not a sixth class.

| Class | Redaction disabled | Redaction enabled |
| --- | --- | --- |
| public | keep | keep |
| internal | keep | keep |
| personal | keep | hash |
| content | keep | drop |
| secret | drop | drop |

Hashing uses installation-key HMAC-SHA256 as `hmac:<hex16>`. The policy runs once before queueing; all sinks receive that sanitized payload. Configuration changes require a restart.

With redaction enabled, unknown span and event names become `operator.redacted.span` and `operator.redacted.event`; unknown metrics are dropped. Absent log event names stay absent. Bodies and auxiliary text are content. Retained metrics use catalog descriptions and units. Attribute types must match the catalog.

In both modes, opaque non-UTF-8 bytes and unresolved string-table references are removed. Secret detection is pattern-based and cannot recognize every credential or establish that arbitrary metadata strings are safe.

`operator.redacted.count` is server-authored: each removed attribute, body, nested body entry, auxiliary field, or metric counts once, each hash counts once, and each non-overlapping secret substitution counts once. A replaced signal name or metric description/unit counts once per field; a removed entity reference counts once. Removing a metric does not additionally count its fields. Client counts are discarded.

See [Telemetry](/telemetry/) for configuration and ingest access.

## Attributes

| Attribute | Type | Class | Description | Source |
| --- | --- | --- | --- | --- |
| `cost_usd` | double | public | Cost usd. | `telemetry/model/third_party/claude_code.yaml` |
| `duration_ms` | int | public | Duration ms. | `telemetry/model/third_party/claude_code.yaml` |
| `error` | string | content | Error. | `telemetry/model/third_party/claude_code.yaml` |
| `gen_ai.conversation.id` | string | internal | Gen ai conversation id. | `telemetry/model/third_party/gen_ai.yaml` |
| `gen_ai.input.messages` | any | content | Gen ai input messages. | `telemetry/model/third_party/gen_ai.yaml` |
| `gen_ai.operation.name` | string | public | Gen ai operation name. | `telemetry/model/third_party/gen_ai.yaml` |
| `gen_ai.output.messages` | any | content | Gen ai output messages. | `telemetry/model/third_party/gen_ai.yaml` |
| `gen_ai.provider.name` | string | public | Gen ai provider name. | `telemetry/model/third_party/gen_ai.yaml` |
| `gen_ai.request.model` | string | public | Gen ai request model. | `telemetry/model/third_party/gen_ai.yaml` |
| `gen_ai.response.id` | string | internal | Gen ai response id. | `telemetry/model/third_party/gen_ai.yaml` |
| `gen_ai.response.model` | string | public | Gen ai response model. | `telemetry/model/third_party/gen_ai.yaml` |
| `gen_ai.system` | string | public | Gen ai system. | `telemetry/model/third_party/gen_ai.yaml` |
| `gen_ai.system_instructions` | any | content | Gen ai system instructions. | `telemetry/model/third_party/gen_ai.yaml` |
| `gen_ai.tool.call.arguments` | any | content | Gen ai tool call arguments. | `telemetry/model/third_party/gen_ai.yaml` |
| `gen_ai.tool.call.result` | any | content | Gen ai tool call result. | `telemetry/model/third_party/gen_ai.yaml` |
| `gen_ai.usage.input_tokens` | int | public | Gen ai usage input tokens. | `telemetry/model/third_party/gen_ai.yaml` |
| `gen_ai.usage.output_tokens` | int | public | Gen ai usage output tokens. | `telemetry/model/third_party/gen_ai.yaml` |
| `http.request.header.authorization` | string | secret | Http request header authorization. | `telemetry/model/attributes/resource.yaml` |
| `http.request.header.cookie` | string | secret | Http request header cookie. | `telemetry/model/attributes/resource.yaml` |
| `input_tokens` | int | public | Input tokens. | `telemetry/model/third_party/claude_code.yaml` |
| `k8s.namespace.name` | string | internal | K8s namespace name. | `telemetry/model/attributes/resource.yaml` |
| `k8s.pod.name` | string | internal | K8s pod name. | `telemetry/model/attributes/resource.yaml` |
| `model` | string | public | Model. | `telemetry/model/third_party/claude_code.yaml` |
| `operator.agent.id` | string | internal | Operator agent id. | `telemetry/model/attributes/operator.yaml` |
| `operator.auth.detail` | string | content | Operator auth detail. | `telemetry/model/attributes/operator.yaml` |
| `operator.auth.succeeded` | boolean | public | Operator auth succeeded. | `telemetry/model/attributes/operator.yaml` |
| `operator.auth.token` | string | secret | Operator auth token. | `telemetry/model/attributes/operator.yaml` |
| `operator.duration` | double | public | Operator duration. | `telemetry/model/attributes/operator.yaml` |
| `operator.error` | string | content | Operator error. | `telemetry/model/attributes/operator.yaml` |
| `operator.exit_code` | int | public | Operator exit code. | `telemetry/model/attributes/operator.yaml` |
| `operator.investigation.severity` | string | public | Operator investigation severity. | `telemetry/model/attributes/operator.yaml` |
| `operator.investigation.source` | string | public | Operator investigation source. | `telemetry/model/attributes/operator.yaml` |
| `operator.issuetype.key` | string | internal | Operator issuetype key. | `telemetry/model/attributes/operator.yaml` |
| `operator.launch.mode` | string | public | Operator launch mode. | `telemetry/model/attributes/operator.yaml` |
| `operator.llm_tool` | string | public | Operator llm tool. | `telemetry/model/attributes/operator.yaml` |
| `operator.output.confidence` | int | public | Operator output confidence. | `telemetry/model/attributes/operator.yaml` |
| `operator.output.error_count` | int | public | Operator output error count. | `telemetry/model/attributes/operator.yaml` |
| `operator.output.exit_signal` | boolean | public | Operator output exit signal. | `telemetry/model/attributes/operator.yaml` |
| `operator.output.files_modified` | int | public | Operator output files modified. | `telemetry/model/attributes/operator.yaml` |
| `operator.output.recommendation` | string | content | Operator output recommendation. | `telemetry/model/attributes/operator.yaml` |
| `operator.output.status` | string | public | Operator output status. | `telemetry/model/attributes/operator.yaml` |
| `operator.output.summary` | string | content | Operator output summary. | `telemetry/model/attributes/operator.yaml` |
| `operator.output.tasks_completed` | int | public | Operator output tasks completed. | `telemetry/model/attributes/operator.yaml` |
| `operator.output.tasks_remaining` | int | public | Operator output tasks remaining. | `telemetry/model/attributes/operator.yaml` |
| `operator.output.tests_status` | string | public | Operator output tests status. | `telemetry/model/attributes/operator.yaml` |
| `operator.pr.number` | int | internal | Operator pr number. | `telemetry/model/attributes/operator.yaml` |
| `operator.pr.url` | string | internal | Operator pr url. | `telemetry/model/attributes/operator.yaml` |
| `operator.profile.id` | string | internal | Operator profile id. | `telemetry/model/attributes/operator.yaml` |
| `operator.project` | string | internal | Operator project. | `telemetry/model/attributes/operator.yaml` |
| `operator.reason` | string | content | Operator reason. | `telemetry/model/attributes/operator.yaml` |
| `operator.redacted.count` | int | public | Operator redacted count. | `telemetry/model/attributes/operator.yaml` |
| `operator.session.id` | string | internal | Operator session id. | `telemetry/model/attributes/operator.yaml` |
| `operator.session.name` | string | internal | Operator session name. | `telemetry/model/attributes/operator.yaml` |
| `operator.step.name` | string | internal | Operator step name. | `telemetry/model/attributes/operator.yaml` |
| `operator.summary` | string | content | Operator summary. | `telemetry/model/attributes/operator.yaml` |
| `operator.telemetry.reason` | string | public | Operator telemetry reason. | `telemetry/model/attributes/operator.yaml` |
| `operator.telemetry.signal` | string | public | Operator telemetry signal. | `telemetry/model/attributes/operator.yaml` |
| `operator.ticket.id` | string | internal | Operator ticket id. | `telemetry/model/attributes/operator.yaml` |
| `organization.id` | string | internal | Organization id. | `telemetry/model/third_party/claude_code.yaml` |
| `output_tokens` | int | public | Output tokens. | `telemetry/model/third_party/claude_code.yaml` |
| `process.executable.path` | string | personal | Process executable path. | `telemetry/model/attributes/resource.yaml` |
| `process.owner` | string | personal | Process owner. | `telemetry/model/attributes/resource.yaml` |
| `prompt` | string | content | Prompt. | `telemetry/model/third_party/claude_code.yaml` |
| `service.instance.id` | string | internal | Service instance id. | `telemetry/model/attributes/resource.yaml` |
| `service.name` | string | public | Service name. | `telemetry/model/attributes/resource.yaml` |
| `service.version` | string | public | Service version. | `telemetry/model/attributes/resource.yaml` |
| `session.id` | string | internal | Session id. | `telemetry/model/third_party/claude_code.yaml` |
| `telemetry.sdk.language` | string | public | Telemetry sdk language. | `telemetry/model/attributes/resource.yaml` |
| `telemetry.sdk.name` | string | public | Telemetry sdk name. | `telemetry/model/attributes/resource.yaml` |
| `telemetry.sdk.version` | string | public | Telemetry sdk version. | `telemetry/model/attributes/resource.yaml` |
| `terminal.type` | string | public | Terminal type. | `telemetry/model/third_party/claude_code.yaml` |
| `tool_name` | string | public | Tool name. | `telemetry/model/third_party/claude_code.yaml` |
| `tool_parameters` | string | content | Tool parameters. | `telemetry/model/third_party/claude_code.yaml` |
| `tool_result` | string | content | Tool result. | `telemetry/model/third_party/claude_code.yaml` |
| `user.email` | string | personal | User email. | `telemetry/model/attributes/resource.yaml` |
| `user.id` | string | personal | User id. | `telemetry/model/attributes/resource.yaml` |
| `user.name` | string | personal | User name. | `telemetry/model/attributes/resource.yaml` |

## Events

### `claude_code.api_error`

Claude code api error.

Source: `telemetry/model/third_party/claude_code.yaml`.

Attributes: `session.id`, `user.email`, `model`, `error`.

### `claude_code.api_request`

Claude code api request.

Source: `telemetry/model/third_party/claude_code.yaml`.

Attributes: `session.id`, `user.email`, `model`, `input_tokens`, `output_tokens`, `cost_usd`, `duration_ms`.

### `claude_code.tool_decision`

Claude code tool decision.

Source: `telemetry/model/third_party/claude_code.yaml`.

Attributes: `session.id`, `user.email`, `tool_name`.

### `claude_code.tool_result`

Claude code tool result.

Source: `telemetry/model/third_party/claude_code.yaml`.

Attributes: `session.id`, `user.email`, `tool_name`, `tool_parameters`, `tool_result`, `duration_ms`.

### `claude_code.user_prompt`

Claude code user prompt.

Source: `telemetry/model/third_party/claude_code.yaml`.

Attributes: `session.id`, `user.email`, `prompt`.

### `operator.agent.awaiting_input`

Operator agent awaiting input.

Source: `telemetry/model/events/agent.yaml`.

Attributes: `operator.ticket.id`, `operator.session.id`, `operator.agent.id`, `operator.step.name`, `operator.project`, `operator.profile.id`, `operator.issuetype.key`, `operator.llm_tool`, `operator.reason`.

### `operator.agent.completed`

Operator agent completed.

Source: `telemetry/model/events/agent.yaml`.

Attributes: `operator.ticket.id`, `operator.session.id`, `operator.agent.id`, `operator.step.name`, `operator.project`, `operator.profile.id`, `operator.issuetype.key`, `operator.llm_tool`, `operator.pr.url`, `operator.duration`.

### `operator.agent.failed`

Operator agent failed.

Source: `telemetry/model/events/agent.yaml`.

Attributes: `operator.ticket.id`, `operator.session.id`, `operator.agent.id`, `operator.step.name`, `operator.project`, `operator.profile.id`, `operator.issuetype.key`, `operator.llm_tool`, `operator.error`.

### `operator.agent.session_lost`

Operator agent session lost.

Source: `telemetry/model/events/agent.yaml`.

Attributes: `operator.ticket.id`, `operator.session.id`, `operator.agent.id`, `operator.step.name`, `operator.project`, `operator.profile.id`, `operator.issuetype.key`, `operator.llm_tool`, `operator.session.name`.

### `operator.agent.started`

Operator agent started.

Source: `telemetry/model/events/agent.yaml`.

Attributes: `operator.ticket.id`, `operator.session.id`, `operator.agent.id`, `operator.step.name`, `operator.project`, `operator.profile.id`, `operator.issuetype.key`, `operator.llm_tool`, `operator.session.name`, `operator.launch.mode`.

### `operator.auth.access_key_created`

Operator auth access key created.

Source: `telemetry/model/events/auth.yaml`.

Attributes: `operator.auth.succeeded`, `operator.auth.detail`.

### `operator.auth.access_key_exchange`

Operator auth access key exchange.

Source: `telemetry/model/events/auth.yaml`.

Attributes: `operator.auth.succeeded`, `operator.auth.detail`.

### `operator.auth.access_key_revoked`

Operator auth access key revoked.

Source: `telemetry/model/events/auth.yaml`.

Attributes: `operator.auth.succeeded`, `operator.auth.detail`.

### `operator.auth.admin_created`

Operator auth admin created.

Source: `telemetry/model/events/auth.yaml`.

Attributes: `operator.auth.succeeded`, `operator.auth.detail`.

### `operator.auth.bootstrap`

Operator auth bootstrap.

Source: `telemetry/model/events/auth.yaml`.

Attributes: `operator.auth.succeeded`, `operator.auth.detail`.

### `operator.auth.device_approved`

Operator auth device approved.

Source: `telemetry/model/events/auth.yaml`.

Attributes: `operator.auth.succeeded`, `operator.auth.detail`.

### `operator.auth.device_code_issued`

Operator auth device code issued.

Source: `telemetry/model/events/auth.yaml`.

Attributes: `operator.auth.succeeded`, `operator.auth.detail`.

### `operator.auth.login`

Operator auth login.

Source: `telemetry/model/events/auth.yaml`.

Attributes: `operator.auth.succeeded`, `operator.auth.detail`.

### `operator.auth.logout`

Operator auth logout.

Source: `telemetry/model/events/auth.yaml`.

Attributes: `operator.auth.succeeded`, `operator.auth.detail`.

### `operator.auth.password_reset`

Operator auth password reset.

Source: `telemetry/model/events/auth.yaml`.

Attributes: `operator.auth.succeeded`, `operator.auth.detail`.

### `operator.auth.refresh_token_reuse`

Operator auth refresh token reuse.

Source: `telemetry/model/events/auth.yaml`.

Attributes: `operator.auth.succeeded`, `operator.auth.detail`.

### `operator.auth.session_revoked`

Operator auth session revoked.

Source: `telemetry/model/events/auth.yaml`.

Attributes: `operator.auth.succeeded`, `operator.auth.detail`.

### `operator.investigation.created`

Operator investigation created.

Source: `telemetry/model/events/ticket.yaml`.

Attributes: `operator.ticket.id`, `operator.session.id`, `operator.agent.id`, `operator.step.name`, `operator.project`, `operator.profile.id`, `operator.issuetype.key`, `operator.llm_tool`, `operator.investigation.source`, `operator.investigation.severity`, `operator.summary`.

### `operator.pr.changes_requested`

Operator pr changes requested.

Source: `telemetry/model/events/pr.yaml`.

Attributes: `operator.ticket.id`, `operator.session.id`, `operator.agent.id`, `operator.step.name`, `operator.project`, `operator.profile.id`, `operator.issuetype.key`, `operator.llm_tool`, `operator.pr.number`, `operator.pr.url`.

### `operator.pr.closed`

Operator pr closed.

Source: `telemetry/model/events/pr.yaml`.

Attributes: `operator.ticket.id`, `operator.session.id`, `operator.agent.id`, `operator.step.name`, `operator.project`, `operator.profile.id`, `operator.issuetype.key`, `operator.llm_tool`, `operator.pr.number`, `operator.pr.url`.

### `operator.pr.created`

Operator pr created.

Source: `telemetry/model/events/pr.yaml`.

Attributes: `operator.ticket.id`, `operator.session.id`, `operator.agent.id`, `operator.step.name`, `operator.project`, `operator.profile.id`, `operator.issuetype.key`, `operator.llm_tool`, `operator.pr.number`, `operator.pr.url`.

### `operator.pr.merged`

Operator pr merged.

Source: `telemetry/model/events/pr.yaml`.

Attributes: `operator.ticket.id`, `operator.session.id`, `operator.agent.id`, `operator.step.name`, `operator.project`, `operator.profile.id`, `operator.issuetype.key`, `operator.llm_tool`, `operator.pr.number`, `operator.pr.url`.

### `operator.pr.ready_to_merge`

Operator pr ready to merge.

Source: `telemetry/model/events/pr.yaml`.

Attributes: `operator.ticket.id`, `operator.session.id`, `operator.agent.id`, `operator.step.name`, `operator.project`, `operator.profile.id`, `operator.issuetype.key`, `operator.llm_tool`, `operator.pr.number`, `operator.pr.url`.

### `operator.redacted.event`

Event whose original name is not approved by the catalog.

Source: `telemetry/model/events/redaction.yaml`.

Attributes: `operator.redacted.count`.

### `operator.step.completed`

Operator step completed.

Source: `telemetry/model/events/step.yaml`.

Attributes: `operator.ticket.id`, `operator.session.id`, `operator.agent.id`, `operator.step.name`, `operator.project`, `operator.profile.id`, `operator.issuetype.key`, `operator.llm_tool`, `operator.exit_code`, `operator.duration`, `operator.output.status`, `operator.output.exit_signal`, `operator.output.confidence`, `operator.output.files_modified`, `operator.output.tests_status`, `operator.output.error_count`, `operator.output.tasks_completed`, `operator.output.tasks_remaining`, `operator.output.summary`, `operator.output.recommendation`.

### `operator.step.started`

Operator step started.

Source: `telemetry/model/events/step.yaml`.

Attributes: `operator.ticket.id`, `operator.session.id`, `operator.agent.id`, `operator.step.name`, `operator.project`, `operator.profile.id`, `operator.issuetype.key`, `operator.llm_tool`.

### `operator.ticket.returned`

Operator ticket returned.

Source: `telemetry/model/events/ticket.yaml`.

Attributes: `operator.ticket.id`, `operator.session.id`, `operator.agent.id`, `operator.step.name`, `operator.project`, `operator.profile.id`, `operator.issuetype.key`, `operator.llm_tool`, `operator.summary`.


## Spans

### `operator.agent.launch`

Operator agent launch.

Source: `telemetry/model/spans/operator.yaml`.

Span kind: `internal`.

Attributes: `operator.ticket.id`, `operator.session.id`, `operator.agent.id`, `operator.step.name`, `operator.project`, `operator.profile.id`, `operator.issuetype.key`, `operator.llm_tool`.

### `operator.otlp.ingest`

Operator otlp ingest.

Source: `telemetry/model/spans/operator.yaml`.

Span kind: `server`.

Attributes: `operator.ticket.id`, `operator.session.id`, `operator.agent.id`, `operator.step.name`, `operator.project`, `operator.profile.id`, `operator.issuetype.key`, `operator.llm_tool`.

### `operator.redacted.span`

Span whose original name is not approved by the catalog.

Source: `telemetry/model/spans/operator.yaml`.

Span kind: `internal`.

Attributes: `operator.redacted.count`.

### `opr8r.step`

Opr8r step.

Source: `telemetry/model/spans/operator.yaml`.

Span kind: `internal`.

Attributes: `operator.ticket.id`, `operator.session.id`, `operator.agent.id`, `operator.step.name`, `operator.project`, `operator.profile.id`, `operator.issuetype.key`, `operator.llm_tool`.


## Metrics

### `operator.agents.active`

Operator agents active.

Source: `telemetry/model/metrics/operator.yaml`.

Instrument: `gauge`. Unit: `{agent}`.

Attributes: `operator.profile.id`.

### `operator.queue.depth`

Operator queue depth.

Source: `telemetry/model/metrics/operator.yaml`.

Instrument: `gauge`. Unit: `{ticket}`.

Attributes: `operator.profile.id`.

### `operator.step.duration`

Operator step duration.

Source: `telemetry/model/metrics/operator.yaml`.

Instrument: `histogram`. Unit: `s`.

Attributes: `operator.profile.id`.

### `operator.telemetry.dropped`

Operator telemetry dropped.

Source: `telemetry/model/metrics/operator.yaml`.

Instrument: `counter`. Unit: `{record}`.

Attributes: `operator.telemetry.signal`, `operator.telemetry.reason`.

### `operator.telemetry.forwarded`

Operator telemetry forwarded.

Source: `telemetry/model/metrics/operator.yaml`.

Instrument: `counter`. Unit: `{record}`.

Attributes: `operator.telemetry.signal`, `operator.telemetry.reason`.

### `operator.telemetry.received`

Operator telemetry received.

Source: `telemetry/model/metrics/operator.yaml`.

Instrument: `counter`. Unit: `{record}`.

Attributes: `operator.telemetry.signal`, `operator.telemetry.reason`.

### `operator.telemetry.redacted`

Operator telemetry redacted.

Source: `telemetry/model/metrics/operator.yaml`.

Instrument: `counter`. Unit: `{record}`.

Attributes: `operator.telemetry.signal`, `operator.telemetry.reason`.


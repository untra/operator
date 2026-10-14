//! OTLP/HTTP ingest for agent sessions, and the per-session telemetry window.
//!
//! Ingest lives outside `/api/v1` at the paths an OTLP exporter derives from
//! `OTEL_EXPORTER_OTLP_ENDPOINT=<operator>/otlp`. It answers 404 while
//! telemetry is disabled in the live configuration.

use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;

use crate::rest::dto::{PrincipalKind, TelemetrySessionWindow, TelemetrySignal};
use crate::rest::error::ApiError;
use crate::rest::middleware::auth::Authenticated;
use crate::rest::state::ApiState;
use crate::telemetry::otlp::TrustedContext;
use crate::telemetry::wire::{self, WireError};
use crate::telemetry::INGEST_RETRY_AFTER_SECS;

const DISABLED: &str = "telemetry is not enabled";

/// Ingest OTLP traces
#[utoipa::path(
    post,
    path = "/otlp/v1/traces",
    tag = "Telemetry",
    operation_id = "telemetry_ingest_traces",
    request_body(content = Vec<u8>, content_type = "application/x-protobuf", description = "ExportTraceServiceRequest as protobuf or OTLP JSON, optionally gzip-encoded"),
    responses(
        (status = 200, description = "ExportTraceServiceResponse in the request's encoding"),
        (status = 400, description = "Malformed body"),
        (status = 404, description = "Telemetry is disabled"),
        (status = 413, description = "Body exceeds the ingest limit, compressed or expanded"),
        (status = 415, description = "Unsupported content type or encoding"),
        (status = 503, description = "Ingest queue is full; retry after the Retry-After delay"),
    )
)]
pub async fn ingest_traces(
    State(state): State<ApiState>,
    Authenticated(principal): Authenticated,
    headers: HeaderMap,
    body: Body,
) -> Response {
    ingest(TelemetrySignal::Traces, &state, &principal, &headers, body).await
}

/// Ingest OTLP metrics
#[utoipa::path(
    post,
    path = "/otlp/v1/metrics",
    tag = "Telemetry",
    operation_id = "telemetry_ingest_metrics",
    request_body(content = Vec<u8>, content_type = "application/x-protobuf", description = "ExportMetricsServiceRequest as protobuf or OTLP JSON, optionally gzip-encoded"),
    responses(
        (status = 200, description = "ExportMetricsServiceResponse in the request's encoding"),
        (status = 400, description = "Malformed body"),
        (status = 404, description = "Telemetry is disabled"),
        (status = 413, description = "Body exceeds the ingest limit, compressed or expanded"),
        (status = 415, description = "Unsupported content type or encoding"),
        (status = 503, description = "Ingest queue is full; retry after the Retry-After delay"),
    )
)]
pub async fn ingest_metrics(
    State(state): State<ApiState>,
    Authenticated(principal): Authenticated,
    headers: HeaderMap,
    body: Body,
) -> Response {
    ingest(TelemetrySignal::Metrics, &state, &principal, &headers, body).await
}

/// Ingest OTLP logs
#[utoipa::path(
    post,
    path = "/otlp/v1/logs",
    tag = "Telemetry",
    operation_id = "telemetry_ingest_logs",
    request_body(content = Vec<u8>, content_type = "application/x-protobuf", description = "ExportLogsServiceRequest as protobuf or OTLP JSON, optionally gzip-encoded"),
    responses(
        (status = 200, description = "ExportLogsServiceResponse in the request's encoding"),
        (status = 400, description = "Malformed body"),
        (status = 404, description = "Telemetry is disabled"),
        (status = 413, description = "Body exceeds the ingest limit, compressed or expanded"),
        (status = 415, description = "Unsupported content type or encoding"),
        (status = 503, description = "Ingest queue is full; retry after the Retry-After delay"),
    )
)]
pub async fn ingest_logs(
    State(state): State<ApiState>,
    Authenticated(principal): Authenticated,
    headers: HeaderMap,
    body: Body,
) -> Response {
    ingest(TelemetrySignal::Logs, &state, &principal, &headers, body).await
}

/// Read an agent session's telemetry window
#[utoipa::path(
    get,
    path = "/api/v1/telemetry/sessions/{session_id}",
    tag = "Telemetry",
    operation_id = "telemetry_get_session",
    params(("session_id" = String, Path, description = "Agent session id")),
    responses(
        (status = 200, description = "The session's retained records, oldest first", body = TelemetrySessionWindow),
        (status = 404, description = "Telemetry is disabled, or the session has no retained records"),
    )
)]
pub async fn get_session(
    State(state): State<ApiState>,
    Path(session_id): Path<String>,
) -> Result<Json<TelemetrySessionWindow>, ApiError> {
    let hub = state
        .telemetry()?
        .ok_or_else(|| ApiError::NotFound(DISABLED.to_string()))?;
    hub.session_window(&session_id).map(Json).ok_or_else(|| {
        ApiError::NotFound(format!("no telemetry retained for session {session_id}"))
    })
}

#[tracing::instrument(name = "operator.otlp.ingest", skip_all, fields(otel.kind = "server"))]
async fn ingest(
    signal: TelemetrySignal,
    state: &ApiState,
    principal: &crate::auth::scope::Principal,
    headers: &HeaderMap,
    body: Body,
) -> Response {
    let hub = match state.telemetry() {
        Ok(Some(hub)) => hub,
        Ok(None) => return ApiError::NotFound(DISABLED.to_string()).into_response(),
        Err(e) => return e.into_response(),
    };
    let (encoding, gzip) = match wire::negotiate(headers) {
        Ok(negotiated) => negotiated,
        Err(e) => return wire_error(e),
    };
    let max = state.config().telemetry.ingest.max_body_bytes;
    let compressed =
        match axum::body::to_bytes(body, usize::try_from(max).unwrap_or(usize::MAX)).await {
            Ok(bytes) => bytes,
            Err(e) if is_length_limit(&e) => return wire_error(WireError::TooLarge),
            Err(e) => return wire_error(WireError::Malformed(format!("reading body: {e}"))),
        };
    let payload = match wire::inflate(&compressed, gzip, max)
        .and_then(|bytes| wire::decode(signal, encoding, &bytes))
    {
        Ok(payload) => payload,
        Err(e) => return wire_error(e),
    };

    let context = trusted_context(state, principal);
    let batch = hub.sanitize(payload, context.as_ref());
    if hub.enqueue(batch).is_err() {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            [(header::RETRY_AFTER, INGEST_RETRY_AFTER_SECS.to_string())],
            "telemetry ingest queue is full",
        )
            .into_response();
    }
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, encoding.content_type())],
        wire::success(signal, encoding),
    )
        .into_response()
}

/// Correlation from an agent token; an admin credential asserts none.
fn trusted_context(
    state: &ApiState,
    principal: &crate::auth::scope::Principal,
) -> Option<TrustedContext> {
    (principal.kind == PrincipalKind::TelemetryIngest).then(|| TrustedContext {
        ticket_id: principal.ticket_id.clone(),
        step: principal.step.clone(),
        session_id: principal.agent_session_id.clone(),
        profile_id: Some(state.config().profile.id.to_string()),
    })
}

fn is_length_limit(error: &axum::Error) -> bool {
    let mut source: Option<&(dyn std::error::Error + 'static)> = Some(error);
    while let Some(e) = source {
        if e.is::<http_body_util::LengthLimitError>() {
            return true;
        }
        source = e.source();
    }
    false
}

fn wire_error(error: WireError) -> Response {
    match error {
        WireError::UnsupportedMediaType(message) => {
            (StatusCode::UNSUPPORTED_MEDIA_TYPE, message).into_response()
        }
        WireError::TooLarge => (
            StatusCode::PAYLOAD_TOO_LARGE,
            "OTLP body exceeds the ingest limit",
        )
            .into_response(),
        WireError::Malformed(message) => (StatusCode::BAD_REQUEST, message).into_response(),
    }
}

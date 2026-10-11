//! Upstream OTLP forwarder. One thread, one current-thread runtime, bounded queues.

use std::collections::VecDeque;
use std::io::Write;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use flate2::write::GzEncoder;
use flate2::Compression;
use prost::Message;
use tokio::sync::{mpsc, watch, OwnedSemaphorePermit, Semaphore};

use super::export_config::ResolvedExport;
use super::otlp::OtlpPayload;
use super::signal_index;
use super::sink::ExportSink;
use crate::config::{OtlpCompression, OtlpProtocol};
use crate::http_client::client_builder;
use crate::rest::dto::TelemetrySignal;
use crate::telemetry::wire::Encoding;

pub const RETRYABLE: [u16; 4] = [429, 502, 503, 504];
const MAX_BATCH_RESOURCES: usize = 64;
const FLUSH_INTERVAL: Duration = Duration::from_secs(1);
const DEFAULT_QUEUE_CAPACITY: usize = 1024;
const DEFAULT_RETRY_CAPACITY: usize = 256;
const DEFAULT_MAX_ATTEMPTS: u32 = 5;
const DEFAULT_BASE_BACKOFF: Duration = Duration::from_millis(500);
const DEFAULT_MAX_BACKOFF: Duration = Duration::from_secs(30);
const USER_AGENT: &str = concat!("operator/", env!("CARGO_PKG_VERSION"), " OTel-OTLP-Exporter-Rust");
const HEADER_CONTENT_ENCODING: &str = "content-encoding";
const GZIP: &str = "gzip";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DropReason {
    QueueFull,
    RetryOverflow,
    Rejected,
    RetriesExhausted,
}

impl DropReason {
    pub const ALL: [Self; 4] = [
        Self::QueueFull,
        Self::RetryOverflow,
        Self::Rejected,
        Self::RetriesExhausted,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::QueueFull => "queue_full",
            Self::RetryOverflow => "retry_overflow",
            Self::Rejected => "rejected",
            Self::RetriesExhausted => "retries_exhausted",
        }
    }

    const fn index(self) -> usize {
        match self {
            Self::QueueFull => 0,
            Self::RetryOverflow => 1,
            Self::Rejected => 2,
            Self::RetriesExhausted => 3,
        }
    }
}

pub struct ForwardStats {
    forwarded: [AtomicU64; 3],
    dropped: [[AtomicU64; 4]; 3],
}

impl ForwardStats {
    fn new() -> Self {
        Self {
            forwarded: [AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0)],
            dropped: [
                [
                    AtomicU64::new(0),
                    AtomicU64::new(0),
                    AtomicU64::new(0),
                    AtomicU64::new(0),
                ],
                [
                    AtomicU64::new(0),
                    AtomicU64::new(0),
                    AtomicU64::new(0),
                    AtomicU64::new(0),
                ],
                [
                    AtomicU64::new(0),
                    AtomicU64::new(0),
                    AtomicU64::new(0),
                    AtomicU64::new(0),
                ],
            ],
        }
    }

    pub fn forwarded(&self, signal: TelemetrySignal) -> u64 {
        self.forwarded[signal_index(signal)].load(Ordering::Relaxed)
    }

    pub fn dropped(&self, signal: TelemetrySignal, reason: DropReason) -> u64 {
        self.dropped[signal_index(signal)][reason.index()].load(Ordering::Relaxed)
    }

    fn add_forwarded(&self, signal: TelemetrySignal, records: u64) {
        self.forwarded[signal_index(signal)].fetch_add(records, Ordering::Relaxed);
    }

    fn add_dropped(&self, signal: TelemetrySignal, reason: DropReason, records: u64) {
        self.dropped[signal_index(signal)][reason.index()].fetch_add(records, Ordering::Relaxed);
    }
}

pub struct ForwarderLimits {
    pub queue_capacity: usize,
    pub retry_capacity: usize,
    pub max_attempts: u32,
    pub base_backoff: Duration,
    pub max_backoff: Duration,
    pub flush_interval: Duration,
}

impl Default for ForwarderLimits {
    fn default() -> Self {
        Self {
            queue_capacity: DEFAULT_QUEUE_CAPACITY,
            retry_capacity: DEFAULT_RETRY_CAPACITY,
            max_attempts: DEFAULT_MAX_ATTEMPTS,
            base_backoff: DEFAULT_BASE_BACKOFF,
            max_backoff: DEFAULT_MAX_BACKOFF,
            flush_interval: FLUSH_INTERVAL,
        }
    }
}

pub struct Forwarder {
    tx: mpsc::Sender<Queued>,
    stats: Arc<ForwardStats>,
    slots: Arc<Semaphore>,
    shutdown: watch::Sender<bool>,
    thread: Mutex<Option<JoinHandle<()>>>,
    done: std::sync::mpsc::Receiver<()>,
    retained: Arc<Mutex<Vec<String>>>,
}

struct Queued {
    payload: OtlpPayload,
    _permit: OwnedSemaphorePermit,
}

struct Pending {
    payload: OtlpPayload,
    resources: usize,
    started: Instant,
    permits: Vec<OwnedSemaphorePermit>,
}

struct RetryItem {
    payload: OtlpPayload,
    attempt: u32,
    ready_at: Instant,
}

impl Forwarder {
    pub fn start(export: ResolvedExport, limits: ForwarderLimits) -> anyhow::Result<Arc<Self>> {
        let capacity = limits.queue_capacity.max(1);
        let (tx, rx) = mpsc::channel(capacity);
        let (shutdown, shutdown_rx) = watch::channel(false);
        let (done_tx, done) = std::sync::mpsc::channel();
        let stats = Arc::new(ForwardStats::new());
        let slots = Arc::new(Semaphore::new(capacity));
        let retained = Arc::new(Mutex::new(Vec::new()));
        let stats_worker = Arc::clone(&stats);
        let retained_worker = Arc::clone(&retained);
        let thread = std::thread::Builder::new()
            .name("otlp-forwarder".into())
            .spawn(move || {
                let runtime = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("otlp forwarder runtime");
                runtime.block_on(worker(
                    rx,
                    export,
                    limits,
                    stats_worker,
                    shutdown_rx,
                    retained_worker,
                ));
                let _ = done_tx.send(());
            })?;
        Ok(Arc::new(Self {
            tx,
            stats,
            slots,
            shutdown,
            thread: Mutex::new(Some(thread)),
            done,
            retained,
        }))
    }

    pub fn stats(&self) -> Arc<ForwardStats> {
        Arc::clone(&self.stats)
    }

    /// Stop accepting work and return once the worker exits, or once `deadline` passes.
    pub fn shutdown(&self, deadline: Duration) {
        let _ = self.shutdown.send(true);
        let _ = self.done.recv_timeout(deadline);
        if let Some(handle) = self
            .thread
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take()
        {
            let (tx, rx) = std::sync::mpsc::channel();
            std::thread::spawn(move || {
                let _ = handle.join();
                let _ = tx.send(());
            });
            let _ = rx.recv_timeout(deadline);
        }
    }

    pub fn retained_span_names(&self) -> Vec<String> {
        self.retained
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }
}

impl ExportSink for Forwarder {
    fn submit(&self, payload: OtlpPayload) {
        let Ok(permit) = self.slots.clone().try_acquire_owned() else {
            self.stats
                .add_dropped(payload.signal(), DropReason::QueueFull, 1);
            return;
        };
        if let Err(error) = self.tx.try_send(Queued {
            payload,
            _permit: permit,
        }) {
            let queued = error.into_inner();
            self.stats
                .add_dropped(queued.payload.signal(), DropReason::QueueFull, 1);
        }
    }
}

/// `None` when `status` is not retryable. `jitter` is in `[0, 1)`.
pub fn retry_delay(
    status: u16,
    retry_after: Option<&str>,
    attempt: u32,
    limits: &ForwarderLimits,
    jitter: f64,
) -> Option<Duration> {
    if !RETRYABLE.contains(&status) {
        return None;
    }
    if let Some(seconds) = retry_after.and_then(|value| value.trim().parse::<u64>().ok()) {
        return Some(Duration::from_secs(seconds).min(limits.max_backoff));
    }
    let factor = 1u32.checked_shl(attempt).unwrap_or(u32::MAX);
    let grown = limits.base_backoff.saturating_mul(factor);
    let capped = grown.min(limits.max_backoff);
    let scaled = 0.5 + (jitter.clamp(0.0, 0.999) / 2.0);
    Some(scale_duration(capped, scaled))
}

fn scale_duration(duration: Duration, factor: f64) -> Duration {
    Duration::from_secs_f64(duration.as_secs_f64() * factor)
}

async fn worker(
    mut rx: mpsc::Receiver<Queued>,
    export: ResolvedExport,
    limits: ForwarderLimits,
    stats: Arc<ForwardStats>,
    mut shutdown: watch::Receiver<bool>,
    retained: Arc<Mutex<Vec<String>>>,
) {
    let client = match client_builder()
        .timeout(export.timeout)
        .user_agent(USER_AGENT)
        .build()
    {
        Ok(client) => client,
        Err(_) => return,
    };
    let mut pending: [Option<Pending>; 3] = [None, None, None];
    let mut retries: [VecDeque<RetryItem>; 3] = [
        VecDeque::new(),
        VecDeque::new(),
        VecDeque::new(),
    ];
    let mut tick = tokio::time::interval(flush_tick(&limits));
    loop {
        tokio::select! {
            biased;
            changed = shutdown.changed() => {
                if changed.is_err() || *shutdown.borrow() {
                    break;
                }
            }
            _ = tick.tick() => {
                flush_pending(&client, &export, &limits, &stats, &mut pending, &mut retries, &retained).await;
                drive_retries(&client, &export, &limits, &stats, &mut retries, &retained).await;
            }
            message = rx.recv() => {
                let Some(queued) = message else { break };
                accept(&client, &export, &limits, &stats, &mut pending, &mut retries, &retained, queued).await;
            }
        }
    }
}

fn flush_tick(limits: &ForwarderLimits) -> Duration {
    if limits.flush_interval.is_zero() {
        Duration::from_millis(1)
    } else {
        limits.flush_interval
    }
}

async fn accept(
    client: &reqwest::Client,
    export: &ResolvedExport,
    limits: &ForwarderLimits,
    stats: &ForwardStats,
    pending: &mut [Option<Pending>; 3],
    retries: &mut [VecDeque<RetryItem>; 3],
    retained: &Mutex<Vec<String>>,
    queued: Queued,
) {
    let signal = queued.payload.signal();
    if export.endpoint(signal).is_none() {
        return;
    }
    let index = signal_index(signal);
    let resources = resource_count(&queued.payload);
    match &mut pending[index] {
        Some(batch) => {
            merge(&mut batch.payload, queued.payload);
            batch.resources += resources;
            batch.permits.push(queued._permit);
        }
        empty => {
            *empty = Some(Pending {
                payload: queued.payload,
                resources,
                started: Instant::now(),
                permits: vec![queued._permit],
            });
        }
    }
    let due = pending[index].as_ref().is_some_and(|batch| {
        batch.resources >= MAX_BATCH_RESOURCES || batch.started.elapsed() >= limits.flush_interval
    });
    if due {
        let batch = pending[index].take();
        if let Some(batch) = batch {
            send_batch(client, export, limits, stats, retries, retained, batch.payload, 0).await;
        }
    }
}

async fn flush_pending(
    client: &reqwest::Client,
    export: &ResolvedExport,
    limits: &ForwarderLimits,
    stats: &ForwardStats,
    pending: &mut [Option<Pending>; 3],
    retries: &mut [VecDeque<RetryItem>; 3],
    retained: &Mutex<Vec<String>>,
) {
    for batch in pending.iter_mut() {
        if let Some(batch) = batch.take() {
            send_batch(client, export, limits, stats, retries, retained, batch.payload, 0).await;
        }
    }
}

async fn drive_retries(
    client: &reqwest::Client,
    export: &ResolvedExport,
    limits: &ForwarderLimits,
    stats: &ForwardStats,
    retries: &mut [VecDeque<RetryItem>; 3],
    retained: &Mutex<Vec<String>>,
) {
    for index in 0..retries.len() {
        loop {
            let ready = retries[index]
                .front()
                .is_some_and(|item| item.ready_at <= Instant::now());
            if !ready {
                break;
            }
            let Some(item) = retries[index].pop_front() else {
                break;
            };
            remember(retained, retries);
            send_or_retry(
                client,
                export,
                limits,
                stats,
                &mut retries[index],
                retained,
                item.payload,
                item.attempt,
            )
            .await;
        }
    }
}

async fn send_batch(
    client: &reqwest::Client,
    export: &ResolvedExport,
    limits: &ForwarderLimits,
    stats: &ForwardStats,
    retries: &mut [VecDeque<RetryItem>; 3],
    retained: &Mutex<Vec<String>>,
    payload: OtlpPayload,
    attempt: u32,
) {
    let index = signal_index(payload.signal());
    send_or_retry(
        client,
        export,
        limits,
        stats,
        &mut retries[index],
        retained,
        payload,
        attempt,
    )
    .await;
}

async fn send_or_retry(
    client: &reqwest::Client,
    export: &ResolvedExport,
    limits: &ForwarderLimits,
    stats: &ForwardStats,
    queue: &mut VecDeque<RetryItem>,
    retained: &Mutex<Vec<String>>,
    payload: OtlpPayload,
    attempt: u32,
) {
    let signal = payload.signal();
    let records = payload.record_count();
    let outcome = post(client, export, limits, attempt, &payload).await;
    match outcome {
        Attempt::Sent => stats.add_forwarded(signal, records),
        Attempt::Rejected => stats.add_dropped(signal, DropReason::Rejected, records),
        Attempt::Retry { delay } => {
            if attempt + 1 >= limits.max_attempts {
                stats.add_dropped(signal, DropReason::RetriesExhausted, records);
            } else {
                enqueue_retry(queue, stats, signal, limits.retry_capacity, RetryItem {
                    payload,
                    attempt: attempt + 1,
                    ready_at: Instant::now() + delay,
                });
            }
        }
    }
    remember(retained, std::slice::from_ref(queue));
}

fn enqueue_retry(
    queue: &mut VecDeque<RetryItem>,
    stats: &ForwardStats,
    signal: TelemetrySignal,
    capacity: usize,
    item: RetryItem,
) {
    while queue.len() >= capacity.max(1) {
        queue.pop_front();
        stats.add_dropped(signal, DropReason::RetryOverflow, 1);
    }
    queue.push_back(item);
}

fn remember(retained: &Mutex<Vec<String>>, queues: &[VecDeque<RetryItem>]) {
    let mut names = Vec::new();
    for queue in queues {
        for item in queue {
            names.extend(span_names(&item.payload));
        }
    }
    *retained.lock().unwrap_or_else(std::sync::PoisonError::into_inner) = names;
}

enum Attempt {
    Sent,
    Rejected,
    Retry { delay: Duration },
}

async fn post(
    client: &reqwest::Client,
    export: &ResolvedExport,
    limits: &ForwarderLimits,
    attempt: u32,
    payload: &OtlpPayload,
) -> Attempt {
    let Some(endpoint) = export.endpoint(payload.signal()) else {
        return Attempt::Rejected;
    };
    let encoding = match export.protocol {
        OtlpProtocol::HttpProtobuf => Encoding::Protobuf,
        OtlpProtocol::HttpJson => Encoding::Json,
    };
    let Ok(body) = encode(payload, encoding) else {
        return Attempt::Rejected;
    };
    let gzip = export.compression == OtlpCompression::Gzip;
    let Ok(body) = compress(body, gzip) else {
        return Attempt::Rejected;
    };
    let mut request = client
        .post(&endpoint.url)
        .header(reqwest::header::CONTENT_TYPE, encoding.content_type())
        .body(body);
    if gzip {
        request = request.header(HEADER_CONTENT_ENCODING, GZIP);
    }
    for (name, value) in &export.headers {
        request = request.header(name, value);
    }
    match request.send().await {
        Ok(response) => classify(
            response.status().as_u16(),
            response.headers().get(reqwest::header::RETRY_AFTER),
            limits,
            attempt,
        ),
        Err(_) => Attempt::Retry {
            delay: retry_delay(503, None, attempt, limits, jitter()).unwrap_or(limits.base_backoff),
        },
    }
}

fn classify(
    status: u16,
    retry_after: Option<&reqwest::header::HeaderValue>,
    limits: &ForwarderLimits,
    attempt: u32,
) -> Attempt {
    if (200..300).contains(&status) {
        return Attempt::Sent;
    }
    let header = retry_after.and_then(|value| value.to_str().ok());
    match retry_delay(status, header, attempt, limits, jitter()) {
        Some(delay) => Attempt::Retry { delay },
        None => Attempt::Rejected,
    }
}

fn jitter() -> f64 {
    rand::random::<f64>().clamp(0.0, 0.999)
}

fn encode(payload: &OtlpPayload, encoding: Encoding) -> anyhow::Result<Vec<u8>> {
    Ok(match (payload, encoding) {
        (OtlpPayload::Traces(request), Encoding::Protobuf) => request.encode_to_vec(),
        (OtlpPayload::Metrics(request), Encoding::Protobuf) => request.encode_to_vec(),
        (OtlpPayload::Logs(request), Encoding::Protobuf) => request.encode_to_vec(),
        (OtlpPayload::Traces(request), Encoding::Json) => serde_json::to_vec(request)?,
        (OtlpPayload::Metrics(request), Encoding::Json) => serde_json::to_vec(request)?,
        (OtlpPayload::Logs(request), Encoding::Json) => serde_json::to_vec(request)?,
    })
}

fn compress(body: Vec<u8>, gzip: bool) -> std::io::Result<Vec<u8>> {
    if !gzip {
        return Ok(body);
    }
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&body)?;
    encoder.finish()
}

fn resource_count(payload: &OtlpPayload) -> usize {
    match payload {
        OtlpPayload::Traces(request) => request.resource_spans.len(),
        OtlpPayload::Metrics(request) => request.resource_metrics.len(),
        OtlpPayload::Logs(request) => request.resource_logs.len(),
    }
}

fn merge(into: &mut OtlpPayload, from: OtlpPayload) {
    match (into, from) {
        (OtlpPayload::Traces(into), OtlpPayload::Traces(from)) => {
            into.resource_spans.extend(from.resource_spans);
        }
        (OtlpPayload::Metrics(into), OtlpPayload::Metrics(from)) => {
            into.resource_metrics.extend(from.resource_metrics);
        }
        (OtlpPayload::Logs(into), OtlpPayload::Logs(from)) => {
            into.resource_logs.extend(from.resource_logs);
        }
        _ => {}
    }
}

impl Drop for Forwarder {
    fn drop(&mut self) {
        self.shutdown(Duration::from_secs(2));
    }
}

fn span_names(payload: &OtlpPayload) -> Vec<String> {
    let OtlpPayload::Traces(request) = payload else {
        return Vec::new();
    };
    request
        .resource_spans
        .iter()
        .flat_map(|resource| &resource.scope_spans)
        .flat_map(|scope| &scope.spans)
        .map(|span| span.name.clone())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Bytes;
    use axum::extract::State;
    use axum::http::{HeaderMap, StatusCode};
    use axum::routing::post;
    use axum::Router;
    use flate2::read::GzDecoder;
    use opentelemetry_proto::tonic::collector::trace::v1::ExportTraceServiceRequest;
    use opentelemetry_proto::tonic::trace::v1::{ResourceSpans, ScopeSpans, Span};
    use std::io::Read;
    use std::sync::Arc;

    fn limits_now() -> ForwarderLimits {
        ForwarderLimits {
            flush_interval: Duration::ZERO,
            ..ForwarderLimits::default()
        }
    }

    fn traces(name: &str) -> OtlpPayload {
        OtlpPayload::Traces(ExportTraceServiceRequest {
            resource_spans: vec![ResourceSpans {
                scope_spans: vec![ScopeSpans {
                    spans: vec![Span {
                        name: name.into(),
                        ..Span::default()
                    }],
                    ..ScopeSpans::default()
                }],
                ..ResourceSpans::default()
            }],
        })
    }

    #[derive(Clone)]
    struct Script {
        statuses: Arc<Mutex<VecDeque<(StatusCode, Option<String>)>>>,
        fallback: (StatusCode, Option<String>),
        hits: Arc<Mutex<Vec<Hit>>>,
        hang: bool,
    }

    struct Hit {
        path: String,
        headers: HeaderMap,
        body: Bytes,
    }

    async fn serve(script: Script) -> String {
        let app = Router::new().route("/v1/traces", post(record)).with_state(script);
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        tokio::spawn(async move {
            let _ = axum::serve(listener, app).await;
        });
        format!("http://{address}")
    }

    async fn record(State(script): State<Script>, headers: HeaderMap, body: Bytes) -> axum::response::Response {
        script.hits.lock().unwrap().push(Hit {
            path: "/v1/traces".to_string(),
            headers,
            body,
        });
        if script.hang {
            std::future::pending::<()>().await;
        }
        let (status, retry_after) = script
            .statuses
            .lock()
            .unwrap()
            .pop_front()
            .unwrap_or(script.fallback);
        let mut response = axum::response::Response::builder().status(status);
        if let Some(value) = retry_after {
            response = response.header("retry-after", value);
        }
        response.body(axum::body::Body::empty()).unwrap()
    }

    async fn wait_hits(hits: &Mutex<Vec<Hit>>, count: usize) {
        tokio::time::timeout(Duration::from_secs(5), async {
            while hits.lock().unwrap().len() < count {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
    }

    fn export_at(base: &str, timeout: Duration) -> ResolvedExport {
        ResolvedExport {
            traces: Some(super::super::export_config::SignalEndpoint {
                url: format!("{base}/v1/traces"),
            }),
            metrics: None,
            logs: None,
            protocol: OtlpProtocol::HttpProtobuf,
            compression: OtlpCompression::Gzip,
            timeout,
            headers: vec![("x-team".into(), "red".into())],
            warnings: Vec::new(),
        }
    }

    #[test]
    fn test_retry_delay_only_for_retryable_statuses() {
        let limits = ForwarderLimits::default();
        assert!(retry_delay(400, None, 0, &limits, 0.0).is_none());
        assert!(retry_delay(500, None, 0, &limits, 0.0).is_none());
        for status in RETRYABLE {
            assert!(retry_delay(status, None, 0, &limits, 0.0).is_some());
        }
    }

    #[test]
    fn test_retry_after_seconds_wins_and_is_capped() {
        let limits = ForwarderLimits::default();
        assert_eq!(
            retry_delay(503, Some("3"), 0, &limits, 0.9),
            Some(Duration::from_secs(3))
        );
        assert_eq!(
            retry_delay(429, Some("99999"), 0, &limits, 0.0),
            Some(limits.max_backoff)
        );
    }

    #[test]
    fn test_backoff_grows_and_stays_within_jitter_bounds() {
        let limits = ForwarderLimits::default();
        let first = retry_delay(502, None, 0, &limits, 0.0).unwrap();
        let later = retry_delay(502, None, 3, &limits, 0.0).unwrap();
        assert!(later > first);
        assert!(retry_delay(502, None, 30, &limits, 0.999).unwrap() <= limits.max_backoff);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_forwards_gzip_protobuf_with_headers_and_user_agent() {
        let hits = Arc::new(Mutex::new(Vec::new()));
        let base = serve(Script {
            statuses: Arc::new(Mutex::new(VecDeque::from([(StatusCode::OK, None)]))),
            fallback: (StatusCode::OK, None),
            hits: Arc::clone(&hits),
            hang: false,
        })
        .await;
        let forwarder = Forwarder::start(export_at(&base, Duration::from_secs(2)), limits_now()).unwrap();
        let payload = traces("handle_request");
        forwarder.submit(payload.clone());
        wait_hits(&hits, 1).await;
        let hit = &hits.lock().unwrap()[0];
        assert_eq!(hit.path, "/v1/traces");
        assert_eq!(
            hit.headers.get("content-encoding").unwrap().to_str().unwrap(),
            "gzip"
        );
        assert_eq!(
            hit.headers.get("content-type").unwrap().to_str().unwrap(),
            "application/x-protobuf"
        );
        assert!(hit
            .headers
            .get("user-agent")
            .unwrap()
            .to_str()
            .unwrap()
            .starts_with("operator/"));
        assert_eq!(hit.headers.get("x-team").unwrap().to_str().unwrap(), "red");
        let mut decoder = GzDecoder::new(hit.body.as_ref());
        let mut raw = Vec::new();
        decoder.read_to_end(&mut raw).unwrap();
        let decoded = ExportTraceServiceRequest::decode(raw.as_slice()).unwrap();
        let OtlpPayload::Traces(expected) = payload else { unreachable!() };
        assert_eq!(decoded, expected);
        assert_eq!(forwarder.stats().forwarded(TelemetrySignal::Traces), 1);
        forwarder.shutdown(Duration::from_secs(2));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_retries_503_honoring_retry_after_then_succeeds() {
        let hits = Arc::new(Mutex::new(Vec::new()));
        let base = serve(Script {
            statuses: Arc::new(Mutex::new(VecDeque::from([
                (StatusCode::SERVICE_UNAVAILABLE, Some("0".into())),
                (StatusCode::OK, None),
            ]))),
            fallback: (StatusCode::OK, None),
            hits: Arc::clone(&hits),
            hang: false,
        })
        .await;
        let forwarder = Forwarder::start(export_at(&base, Duration::from_secs(2)), limits_now()).unwrap();
        forwarder.submit(traces("retry"));
        wait_hits(&hits, 2).await;
        assert_eq!(forwarder.stats().forwarded(TelemetrySignal::Traces), 1);
        assert_eq!(forwarder.stats().dropped(TelemetrySignal::Traces, DropReason::Rejected), 0);
        forwarder.shutdown(Duration::from_secs(2));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_400_is_not_retried_and_counts_rejected() {
        let hits = Arc::new(Mutex::new(Vec::new()));
        let base = serve(Script {
            statuses: Arc::new(Mutex::new(VecDeque::from([(StatusCode::BAD_REQUEST, None)]))),
            fallback: (StatusCode::BAD_REQUEST, None),
            hits: Arc::clone(&hits),
            hang: false,
        })
        .await;
        let forwarder = Forwarder::start(export_at(&base, Duration::from_secs(2)), limits_now()).unwrap();
        forwarder.submit(traces("bad"));
        wait_hits(&hits, 1).await;
        tokio::time::sleep(Duration::from_millis(50)).await;
        assert_eq!(hits.lock().unwrap().len(), 1);
        assert_eq!(forwarder.stats().dropped(TelemetrySignal::Traces, DropReason::Rejected), 1);
        forwarder.shutdown(Duration::from_secs(2));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_retry_queue_overflow_drops_oldest() {
        let hits = Arc::new(Mutex::new(Vec::new()));
        let base = serve(Script {
            statuses: Arc::new(Mutex::new(VecDeque::new())),
            fallback: (StatusCode::SERVICE_UNAVAILABLE, Some("60".into())),
            hits: Arc::clone(&hits),
            hang: false,
        })
        .await;
        let limits = ForwarderLimits {
            retry_capacity: 1,
            ..limits_now()
        };
        let forwarder = Forwarder::start(export_at(&base, Duration::from_secs(2)), limits).unwrap();
        for name in ["oldest", "middle", "newest"] {
            forwarder.submit(traces(name));
        }
        wait_hits(&hits, 3).await;
        assert!(forwarder.stats().dropped(TelemetrySignal::Traces, DropReason::RetryOverflow) >= 1);
        assert_eq!(forwarder.retained_span_names(), vec!["newest".to_string()]);
        forwarder.shutdown(Duration::from_secs(2));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_full_submit_queue_drops_without_blocking() {
        let hits = Arc::new(Mutex::new(Vec::new()));
        let base = serve(Script {
            statuses: Arc::new(Mutex::new(VecDeque::new())),
            fallback: (StatusCode::OK, None),
            hits: Arc::clone(&hits),
            hang: true,
        })
        .await;
        let limits = ForwarderLimits {
            queue_capacity: 1,
            ..limits_now()
        };
        let forwarder = Forwarder::start(export_at(&base, Duration::from_secs(30)), limits).unwrap();
        forwarder.submit(traces("held"));
        let started = Instant::now();
        forwarder.submit(traces("dropped"));
        assert!(started.elapsed() < Duration::from_secs(1));
        assert_eq!(forwarder.stats().dropped(TelemetrySignal::Traces, DropReason::QueueFull), 1);
        forwarder.shutdown(Duration::from_millis(300));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_shutdown_returns_within_deadline_when_upstream_hangs() {
        let hits = Arc::new(Mutex::new(Vec::new()));
        let base = serve(Script {
            statuses: Arc::new(Mutex::new(VecDeque::new())),
            fallback: (StatusCode::OK, None),
            hits: Arc::clone(&hits),
            hang: true,
        })
        .await;
        let forwarder = Forwarder::start(export_at(&base, Duration::from_secs(30)), limits_now()).unwrap();
        forwarder.submit(traces("hang"));
        wait_hits(&hits, 1).await;
        let started = Instant::now();
        forwarder.shutdown(Duration::from_millis(300));
        assert!(started.elapsed() < Duration::from_secs(2));
    }
}

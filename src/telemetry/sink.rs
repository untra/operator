//! Process-wide export sink. Producers submit without waiting.

use std::sync::{Arc, OnceLock};

use super::otlp::OtlpPayload;

static SINK: OnceLock<Arc<dyn ExportSink>> = OnceLock::new();

pub trait ExportSink: Send + Sync {
    /// Never blocks. A full queue drops and counts.
    fn submit(&self, payload: OtlpPayload);
}

/// Install the process sink. Returns false when one is already installed.
pub fn install(sink: Arc<dyn ExportSink>) -> bool {
    SINK.set(sink).is_ok()
}

pub fn installed() -> Option<Arc<dyn ExportSink>> {
    SINK.get().map(Arc::clone)
}

/// A fixed transformation selected by an attribute classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RedactionAction {
    /// Retains a value after mandatory secret protection.
    Keep,
    /// Removes a value from the sanitized payload.
    Drop,
    /// Replaces a personal value with an installation-key pseudonym.
    Hash,
}

/// Describes a catalog attribute and its privacy classification.
#[derive(Debug)]
pub struct AttributeSpec {
    /// Names the catalog entry using its emitted identifier.
    pub name: &'static str,
    /// Selects the fixed redaction behavior for this attribute.
    pub class: RedactionClass,
    /// Declares the value shape accepted when redaction is enabled.
    pub value_type: &'static str,
    /// Describes the entry for catalog consumers.
    pub description: &'static str,
    /// Locates the definition relative to the telemetry registry.
    pub source: &'static str,
}

/// Describes an approved event, span, or metric.
#[derive(Debug)]
pub struct SignalSpec {
    /// Names the catalog entry using its emitted identifier.
    pub name: &'static str,
    /// Distinguishes event, span, and metric definitions.
    pub kind: &'static str,
    /// Describes the entry for catalog consumers.
    pub description: &'static str,
    /// Lists attributes associated with this signal.
    pub attributes: &'static [&'static str],
    /// Locates the definition relative to the telemetry registry.
    pub source: &'static str,
    /// Provides the catalog measurement unit for metrics.
    pub unit: Option<&'static str>,
}

include!(concat!(env!("OUT_DIR"), "/catalog.rs"));

pub const CATALOG_MARKDOWN: &str = include_str!(concat!(env!("OUT_DIR"), "/catalog.md"));

#[cfg(feature = "redact")]
mod redact;
#[cfg(feature = "redact")]
pub use redact::{RedactionPolicy, RedactionResult, RedactionStats, Redactor};

#[cfg(feature = "otlp-json")]
pub mod otlp_json;
pub mod traceparent;

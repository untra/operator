use hmac::{Hmac, Mac};
use regex::Regex;
use serde_json::Value;
use sha2::Sha256;

use crate::{class_of, RedactionAction, RedactionClass, ATTRIBUTES, SECRET_PATTERNS};

const HASH_BYTES: usize = 8;
const REDACTED: &str = "[REDACTED]";

/// Controls optional privacy filtering while mandatory secret protection remains active.
#[derive(Debug, Clone, Copy, Default)]
pub struct RedactionPolicy {
    /// Enables personal pseudonymization and content removal, defaulting to false.
    pub enable_redaction: bool,
}

/// Counts transformations applied during one sanitization pass.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RedactionStats {
    /// Counts removed attributes, bodies, body entries, auxiliary fields, or entire metrics.
    pub dropped: u64,
    /// Counts personal attributes replaced with keyed pseudonyms.
    pub hashed: u64,
    /// Counts secret substitutions and approved signal-name or metric-metadata replacements.
    pub scrubbed: u64,
}

impl RedactionStats {
    /// Returns the authoritative number of transformations in this pass.
    pub const fn total(self) -> u64 {
        self.dropped + self.hashed + self.scrubbed
    }
}

/// Contains a sanitized value and its transformation counts.
#[derive(Debug, Clone, PartialEq)]
pub struct RedactionResult {
    /// Holds the retained value or indicates that it was removed.
    pub value: Option<Value>,
    /// Records the operations used to sanitize this value.
    pub stats: RedactionStats,
}

/// Applies the registry policy with a caller-supplied installation key.
pub struct Redactor {
    mac: Hmac<Sha256>,
    secrets: Regex,
}

impl Redactor {
    /// Compiles the shipped patterns and initializes keyed pseudonymization without persisting the key.
    pub fn new(key: &[u8; 32]) -> Result<Self, regex::Error> {
        Ok(Self {
            mac: Hmac::<Sha256>::new_from_slice(key).expect("HMAC accepts 32-byte keys"),
            secrets: Regex::new(&SECRET_PATTERNS.join("|"))?,
        })
    }

    /// Returns an independently sanitized attribute without modifying the input.
    pub fn redact_attribute(
        &self,
        key: &str,
        value: &Value,
        policy: RedactionPolicy,
    ) -> RedactionResult {
        if self.is_secret_key(key) || self.contains_secret(value) {
            return dropped();
        }
        let action = class_of(key).action(policy.enable_redaction);
        if action == RedactionAction::Drop
            || (policy.enable_redaction && !matches_catalog_type(key, value))
        {
            return dropped();
        }
        if action == RedactionAction::Hash {
            return RedactionResult {
                value: Some(Value::String(self.hash(value))),
                stats: RedactionStats {
                    hashed: 1,
                    ..RedactionStats::default()
                },
            };
        }
        RedactionResult {
            value: Some(value.clone()),
            stats: RedactionStats::default(),
        }
    }

    /// Removes content bodies when enabled and otherwise scrubs credentials recursively.
    pub fn redact_body(&self, value: &Value, policy: RedactionPolicy) -> RedactionResult {
        if policy.enable_redaction {
            return dropped();
        }
        let mut stats = RedactionStats::default();
        let value = self.scrub(value, &mut stats);
        RedactionResult {
            value: Some(value),
            stats,
        }
    }

    /// Identifies cataloged secret keys and keys containing recognizable credentials.
    pub fn is_secret_key(&self, key: &str) -> bool {
        class_of(key) == RedactionClass::Secret || self.contains_secret_text(key)
    }

    /// Detects recognizable credentials in text without encoding or transforming it.
    pub fn contains_secret_text(&self, text: &str) -> bool {
        self.secrets.is_match(text)
    }

    /// Replaces detected credentials with a fixed marker and returns the replacement count.
    pub fn scrub_text(&self, text: &str) -> (String, u64) {
        (
            self.secrets.replace_all(text, REDACTED).into_owned(),
            self.secrets.find_iter(text).count() as u64,
        )
    }

    fn contains_secret(&self, value: &Value) -> bool {
        match value {
            Value::String(s) => self.secrets.is_match(s),
            Value::Array(values) => values.iter().any(|v| self.contains_secret(v)),
            Value::Object(values) => values
                .iter()
                .any(|(key, v)| self.is_secret_key(key) || self.contains_secret(v)),
            _ => false,
        }
    }

    fn scrub(&self, value: &Value, stats: &mut RedactionStats) -> Value {
        match value {
            Value::String(s) => {
                let (text, count) = self.scrub_text(s);
                stats.scrubbed += count;
                Value::String(text)
            }
            Value::Array(values) => {
                Value::Array(values.iter().map(|v| self.scrub(v, stats)).collect())
            }
            Value::Object(values) => Value::Object(
                values
                    .iter()
                    .filter_map(|(key, v)| {
                        if self.is_secret_key(key) {
                            stats.dropped += 1;
                            None
                        } else {
                            Some((key.clone(), self.scrub(v, stats)))
                        }
                    })
                    .collect(),
            ),
            _ => value.clone(),
        }
    }

    fn hash(&self, value: &Value) -> String {
        use std::fmt::Write;
        let mut mac = self.mac.clone();
        mac.update(&serde_json::to_vec(&canonical(value)).expect("JSON value is serializable"));
        let bytes = mac.finalize().into_bytes();
        let mut output = String::from("hmac:");
        for byte in &bytes[..HASH_BYTES] {
            write!(output, "{byte:02x}").unwrap();
        }
        output
    }
}

fn matches_catalog_type(key: &str, value: &Value) -> bool {
    ATTRIBUTES
        .iter()
        .find(|a| a.name == key)
        .is_some_and(|a| matches_type(a.value_type, value))
}

fn matches_type(kind: &str, value: &Value) -> bool {
    if let Some(element) = kind.strip_suffix("[]") {
        return value
            .as_array()
            .is_some_and(|values| values.iter().all(|v| matches_type(element, v)));
    }
    match kind {
        "string" => value.is_string(),
        "int" => value.as_i64().is_some(),
        "double" => value.is_number(),
        "boolean" => value.is_boolean(),
        "any" => true,
        _ => false,
    }
}

fn canonical(value: &Value) -> Value {
    match value {
        Value::Array(values) => Value::Array(values.iter().map(canonical).collect()),
        Value::Object(values) => {
            let sorted: std::collections::BTreeMap<_, _> = values.iter().collect();
            Value::Object(
                sorted
                    .into_iter()
                    .map(|(k, v)| (k.clone(), canonical(v)))
                    .collect(),
            )
        }
        _ => value.clone(),
    }
}

fn dropped() -> RedactionResult {
    RedactionResult {
        value: None,
        stats: RedactionStats {
            dropped: 1,
            ..RedactionStats::default()
        },
    }
}

#[cfg(test)]
mod tests;

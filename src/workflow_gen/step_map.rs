//! Pure helpers for mapping a `StepSchema` onto a Claude workflow primitive.

use crate::templates::schema::{ClassifierConfig, ClassifierOutputType};

/// Sanitize a step name into a valid JS identifier suffix (`r_<name>`).
pub fn result_var(step_name: &str) -> String {
    let cleaned: String = step_name
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect();
    format!("r_{cleaned}")
}

/// Synthesize a JSON-schema `serde_json::Value` for a classifier step that has
/// no explicit `json_schema`. The result is an object with a single `value`
/// property typed according to the classifier's output type.
pub fn classifier_schema(cfg: &ClassifierConfig) -> serde_json::Value {
    let mut value_schema = serde_json::Map::new();
    value_schema.insert(
        "type".to_string(),
        serde_json::json!(classifier_value_type(&cfg.output_type)),
    );
    if cfg.output_type == ClassifierOutputType::Enum {
        if let Some(opts) = &cfg.options {
            value_schema.insert("enum".to_string(), serde_json::json!(opts));
        }
    }
    if cfg.output_type == ClassifierOutputType::ShortString {
        if let Some(max) = cfg.max_length {
            value_schema.insert("maxLength".to_string(), serde_json::json!(max));
        }
    }
    serde_json::json!({
        "type": "object",
        "properties": { "value": serde_json::Value::Object(value_schema) },
        "required": ["value"]
    })
}

/// Map a `ClassifierOutputType` to its primitive JSON type name.
pub fn classifier_value_type(output: &ClassifierOutputType) -> &'static str {
    match output {
        ClassifierOutputType::Boolean => "boolean",
        ClassifierOutputType::Number => "number",
        ClassifierOutputType::ShortString | ClassifierOutputType::BigText => "string",
        ClassifierOutputType::Enum => "string",
    }
}

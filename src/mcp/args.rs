//! Argument parsing and reply shaping shared by every MCP tool.

use axum::Json;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;

use crate::rest::error::ApiError;

/// Render an `ApiError` as a stable `code: message` string.
pub fn mcp_error(error: ApiError) -> String {
    let (_, code, message) = error.parts();
    format!("{code}: {message}")
}

pub fn parse<T: DeserializeOwned>(args: &Value) -> Result<T, String> {
    serde_json::from_value(args.clone()).map_err(|e| format!("invalid arguments: {e}"))
}

pub fn required_str<'a>(args: &'a Value, key: &str) -> Result<&'a str, String> {
    args.get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("Missing required parameter: {key}"))
}

pub fn optional_string(args: &Value, key: &str) -> Option<String> {
    args.get(key).and_then(Value::as_str).map(String::from)
}

pub fn json_value<T: Serialize>(Json(body): Json<T>) -> Result<Value, String> {
    serde_json::to_value(body).map_err(|e| e.to_string())
}

pub fn reply<T: Serialize>(result: Result<Json<T>, ApiError>) -> Result<Value, String> {
    result.map_err(mcp_error).and_then(json_value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// A drain rejection must be machine-readable and carry the same code the
    /// HTTP surface uses, not a debug-formatted Rust variant.
    #[test]
    fn test_mcp_error_uses_the_shared_stable_code_table() {
        assert_eq!(
            mcp_error(ApiError::Unavailable("draining".to_string())),
            "unavailable: draining"
        );
        assert_eq!(
            mcp_error(ApiError::Conflict("already running".to_string())),
            "conflict: already running"
        );
        assert!(
            !mcp_error(ApiError::NotFound("x".to_string())).contains("NotFound("),
            "the MCP error must not leak Rust debug syntax"
        );
    }

    #[test]
    fn test_required_str_names_the_missing_key() {
        let args = json!({ "id": "FEAT-1" });
        assert_eq!(required_str(&args, "id"), Ok("FEAT-1"));
        assert_eq!(
            required_str(&args, "reason"),
            Err("Missing required parameter: reason".to_string())
        );
    }
}

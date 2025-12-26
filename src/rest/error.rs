//! API error types and responses.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// API error types
#[derive(Debug)]
pub enum ApiError {
    /// Resource not found
    NotFound(String),
    /// Validation error
    ValidationError(String),
    /// Resource already exists
    Conflict(String),
    /// Internal server error
    InternalError(String),
    /// Bad request
    BadRequest(String),
    /// Cannot modify builtin resource
    BuiltinReadOnly(String),
}

/// Error response body
#[derive(Serialize, Deserialize, ToSchema)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error, message) = match self {
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, "not_found", msg),
            ApiError::ValidationError(msg) => (StatusCode::BAD_REQUEST, "validation_error", msg),
            ApiError::Conflict(msg) => (StatusCode::CONFLICT, "conflict", msg),
            ApiError::InternalError(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "internal_error", msg)
            }
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, "bad_request", msg),
            ApiError::BuiltinReadOnly(msg) => (StatusCode::FORBIDDEN, "builtin_readonly", msg),
        };

        (
            status,
            Json(ErrorResponse {
                error: error.to_string(),
                message,
            }),
        )
            .into_response()
    }
}

impl From<std::io::Error> for ApiError {
    fn from(err: std::io::Error) -> Self {
        ApiError::InternalError(err.to_string())
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(err: serde_json::Error) -> Self {
        ApiError::BadRequest(format!("JSON error: {}", err))
    }
}

impl From<anyhow::Error> for ApiError {
    fn from(err: anyhow::Error) -> Self {
        ApiError::InternalError(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use http_body_util::BodyExt;

    #[tokio::test]
    async fn test_not_found_response() {
        let error = ApiError::NotFound("Type 'FOO' not found".to_string());
        let response = error.into_response();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let json: ErrorResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(json.error, "not_found");
    }

    #[tokio::test]
    async fn test_validation_error_response() {
        let error = ApiError::ValidationError("Invalid key format".to_string());
        let response = error.into_response();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_builtin_readonly_response() {
        let error = ApiError::BuiltinReadOnly("Cannot modify builtin type 'FEAT'".to_string());
        let response = error.into_response();

        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }
}

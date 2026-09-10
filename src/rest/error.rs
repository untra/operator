//! API error types and responses.

use axum::{
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};

/// Challenge returned with every `401`. Names both accepted schemes so a client that holds neither knows which to obtain.
const WWW_AUTHENTICATE_CHALLENGE: &str = r#"Bearer realm="operator", Cookie realm="operator""#;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// API error types
#[derive(Debug)]
#[allow(dead_code)] // Auth variants are constructed by Phase 3 middleware/handlers.
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
    // The three auth variants below are constructed by the authorization
    // middleware and auth handlers, which land in Phase 3. The error contract
    // ships first so clients can be generated against a settled shape.
    /// No usable credential was presented. Carries a `WWW-Authenticate`
    /// challenge so a client knows *how* to authenticate, not just that it must.
    Unauthorized(String),
    /// A valid credential that lacks the scope this route requires. Distinct
    /// from `Unauthorized`: re-authenticating will not help, so a client must
    /// not retry with the same credential.
    Forbidden(String),
    /// A cookie-authenticated mutation arrived without a valid CSRF token or
    /// with a mismatched `Origin`.
    CsrfFailed(String),
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
            ApiError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, "unauthorized", msg),
            ApiError::Forbidden(msg) => (StatusCode::FORBIDDEN, "forbidden", msg),
            ApiError::CsrfFailed(msg) => (StatusCode::FORBIDDEN, "csrf_failed", msg),
        };

        let body = Json(ErrorResponse {
            error: error.to_string(),
            message,
        });

        // Only a 401 carries a challenge. A 403 means the credential was
        // understood and refused, so advertising a scheme would invite a
        // pointless retry.
        if status == StatusCode::UNAUTHORIZED {
            (
                status,
                [(header::WWW_AUTHENTICATE, WWW_AUTHENTICATE_CHALLENGE)],
                body,
            )
                .into_response()
        } else {
            (status, body).into_response()
        }
    }
}

impl From<std::io::Error> for ApiError {
    fn from(err: std::io::Error) -> Self {
        ApiError::InternalError(err.to_string())
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(err: serde_json::Error) -> Self {
        ApiError::BadRequest(format!("JSON error: {err}"))
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

    #[tokio::test]
    async fn test_unauthorized_carries_a_challenge() {
        let response =
            ApiError::Unauthorized("no credential presented".to_string()).into_response();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        let challenge = response
            .headers()
            .get(header::WWW_AUTHENTICATE)
            .expect("401 must advertise how to authenticate")
            .to_str()
            .unwrap();
        assert!(challenge.contains("Bearer"));
        assert!(challenge.contains("Cookie"));
    }

    #[tokio::test]
    async fn test_forbidden_does_not_invite_a_retry() {
        // The credential was understood and refused; a challenge would suggest
        // re-authenticating fixes it, which it does not.
        let response =
            ApiError::Forbidden("requires the `admin` scope".to_string()).into_response();

        assert_eq!(response.status(), StatusCode::FORBIDDEN);
        assert!(response.headers().get(header::WWW_AUTHENTICATE).is_none());

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let json: ErrorResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(json.error, "forbidden");
    }

    #[tokio::test]
    async fn test_csrf_failure_is_distinguishable_from_a_scope_denial() {
        // A client retries these differently: refetch a CSRF token, versus
        // obtain a credential with more scope.
        let response = ApiError::CsrfFailed("missing CSRF token".to_string()).into_response();

        assert_eq!(response.status(), StatusCode::FORBIDDEN);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let json: ErrorResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(json.error, "csrf_failed");
    }
}

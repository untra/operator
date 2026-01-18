use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Default API port for Operator
const DEFAULT_API_PORT: u16 = 7008;

/// API session file path relative to working directory
const API_SESSION_FILE: &str = ".tickets/operator/api-session.json";

/// Retry configuration
const MAX_RETRIES: u32 = 3;
const INITIAL_BACKOFF_MS: u64 = 1000;

/// API session info from api-session.json
#[derive(Debug, Deserialize)]
pub struct ApiSession {
    pub port: u16,
    #[allow(dead_code)]
    pub pid: u32,
    #[allow(dead_code)]
    pub started_at: String,
    #[allow(dead_code)]
    pub version: String,
}

/// Request body for step completion
#[derive(Debug, Serialize)]
pub struct StepCompleteRequest {
    pub exit_code: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    pub duration_secs: u64,
}

/// Response from step completion endpoint
#[derive(Debug, Deserialize)]
pub struct StepCompleteResponse {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_step: Option<NextStepInfo>,
    pub auto_proceed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_command: Option<String>,
}

/// Information about the next step
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct NextStepInfo {
    pub name: String,
    pub display_name: String,
    pub review_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub json_schema: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub json_schema_file: Option<String>,
}

/// Current step info response
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct CurrentStepInfo {
    pub step_name: String,
    pub step_index: u32,
    pub display_name: String,
    pub review_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub json_schema: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub json_schema_file: Option<String>,
    pub is_final: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_step_name: Option<String>,
}

/// API client for communicating with Operator
pub struct ApiClient {
    client: Client,
    base_url: String,
}

#[derive(Debug)]
pub enum ApiError {
    /// Could not connect to API after retries
    Unreachable(String),
    /// API returned an error response
    ResponseError(u16, String),
    /// Failed to parse response
    ParseError(String),
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiError::Unreachable(msg) => write!(f, "API unreachable: {}", msg),
            ApiError::ResponseError(code, msg) => write!(f, "API error ({}): {}", code, msg),
            ApiError::ParseError(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}

impl std::error::Error for ApiError {}

impl ApiClient {
    /// Create a new API client with the given base URL
    pub fn new(base_url: &str) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
        }
    }

    /// Discover API endpoint from api-session.json or use default
    pub async fn discover(api_url: Option<&str>) -> Result<Self, ApiError> {
        if let Some(url) = api_url {
            return Ok(Self::new(url));
        }

        // Try to read api-session.json (sync is fine for a tiny JSON file)
        if let Ok(content) = std::fs::read_to_string(API_SESSION_FILE) {
            if let Ok(session) = serde_json::from_str::<ApiSession>(&content) {
                let url = format!("http://localhost:{}", session.port);
                return Ok(Self::new(&url));
            }
        }

        // Fall back to default
        let url = format!("http://localhost:{}", DEFAULT_API_PORT);
        Ok(Self::new(&url))
    }

    /// Report step completion to the API with retry logic
    pub async fn complete_step(
        &self,
        ticket_id: &str,
        step: &str,
        request: StepCompleteRequest,
    ) -> Result<StepCompleteResponse, ApiError> {
        let url = format!(
            "{}/api/v1/tickets/{}/steps/{}/complete",
            self.base_url, ticket_id, step
        );

        self.post_with_retry(&url, &request).await
    }

    /// POST request with retry logic
    async fn post_with_retry<T: Serialize, R: for<'de> Deserialize<'de>>(
        &self,
        url: &str,
        body: &T,
    ) -> Result<R, ApiError> {
        let mut last_error = None;
        let mut backoff_ms = INITIAL_BACKOFF_MS;

        for attempt in 0..MAX_RETRIES {
            if attempt > 0 {
                tokio::time::sleep(Duration::from_millis(backoff_ms)).await;
                backoff_ms *= 2; // Exponential backoff
            }

            match self.client.post(url).json(body).send().await {
                Ok(response) => {
                    let status = response.status();
                    if status.is_success() {
                        return response.json::<R>().await.map_err(|e| {
                            ApiError::ParseError(format!("Failed to parse response: {}", e))
                        });
                    } else {
                        let error_text = response
                            .text()
                            .await
                            .unwrap_or_else(|_| "Unknown error".to_string());
                        last_error = Some(ApiError::ResponseError(status.as_u16(), error_text));
                    }
                }
                Err(e) => {
                    last_error = Some(ApiError::Unreachable(e.to_string()));
                }
            }
        }

        Err(last_error.unwrap_or_else(|| ApiError::Unreachable("Unknown error".to_string())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_step_complete_request_serialization() {
        let request = StepCompleteRequest {
            exit_code: 0,
            session_id: Some("abc-123".to_string()),
            duration_secs: 342,
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"exit_code\":0"));
        assert!(json.contains("\"session_id\":\"abc-123\""));
        assert!(json.contains("\"duration_secs\":342"));
    }

    #[test]
    fn test_api_client_new() {
        let client = ApiClient::new("http://localhost:7008/");
        assert_eq!(client.base_url, "http://localhost:7008");

        let client = ApiClient::new("http://localhost:7008");
        assert_eq!(client.base_url, "http://localhost:7008");
    }

    #[test]
    fn test_api_error_display() {
        let err = ApiError::Unreachable("connection refused".to_string());
        assert!(err.to_string().contains("unreachable"));
        assert!(err.to_string().contains("connection refused"));

        let err = ApiError::ResponseError(404, "not found".to_string());
        assert!(err.to_string().contains("404"));
        assert!(err.to_string().contains("not found"));

        let err = ApiError::ParseError("invalid json".to_string());
        assert!(err.to_string().contains("Parse error"));
    }

    #[test]
    fn test_step_complete_request_no_session() {
        let request = StepCompleteRequest {
            exit_code: 1,
            session_id: None,
            duration_secs: 60,
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"exit_code\":1"));
        // session_id should be omitted when None
        assert!(!json.contains("session_id"));
    }

    #[test]
    fn test_step_complete_response_deserialization() {
        let json = r#"{
            "status": "completed",
            "auto_proceed": true,
            "next_command": "opr8r --ticket-id=FEAT-1 --step=build -- claude"
        }"#;

        let response: StepCompleteResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.status, "completed");
        assert!(response.auto_proceed);
        assert!(response.next_command.is_some());
        assert!(response.next_step.is_none());
    }

    #[test]
    fn test_step_complete_response_with_next_step() {
        let json = r#"{
            "status": "awaiting_review",
            "auto_proceed": false,
            "next_step": {
                "name": "review",
                "display_name": "Code Review",
                "review_type": "plan"
            }
        }"#;

        let response: StepCompleteResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.status, "awaiting_review");
        assert!(!response.auto_proceed);
        assert!(response.next_step.is_some());
        let next = response.next_step.unwrap();
        assert_eq!(next.name, "review");
        assert_eq!(next.review_type, "plan");
    }

    #[test]
    fn test_api_session_deserialization() {
        let json = r#"{
            "port": 7008,
            "pid": 12345,
            "started_at": "2024-01-15T10:30:00Z",
            "version": "0.1.14"
        }"#;

        let session: ApiSession = serde_json::from_str(json).unwrap();
        assert_eq!(session.port, 7008);
        assert_eq!(session.pid, 12345);
    }
}

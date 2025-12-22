//! API error types with authentication failure tracking

use std::fmt;

/// Errors that can occur when interacting with external APIs
#[derive(Debug, Clone)]
pub enum ApiError {
    /// 401 Unauthorized - token invalid or expired
    Unauthorized {
        provider: String,
        consecutive_count: u32,
    },
    /// 403 Forbidden - token lacks required permissions
    Forbidden { provider: String },
    /// 429 Rate Limited
    RateLimited {
        provider: String,
        retry_after_secs: Option<u64>,
    },
    /// Network or timeout error
    NetworkError { provider: String, message: String },
    /// Other HTTP errors
    HttpError {
        provider: String,
        status: u16,
        message: String,
    },
    /// Provider not configured (no token in environment)
    NotConfigured { provider: String },
}

impl ApiError {
    /// Check if this is an authentication error (401 or 403)
    pub fn is_auth_error(&self) -> bool {
        matches!(
            self,
            ApiError::Unauthorized { .. } | ApiError::Forbidden { .. }
        )
    }

    /// Check if this error indicates the token needs to be refreshed
    /// Returns true if there have been 3+ consecutive 401 errors
    pub fn needs_token_refresh(&self) -> bool {
        matches!(
            self,
            ApiError::Unauthorized {
                consecutive_count,
                ..
            } if *consecutive_count >= 3
        )
    }

    /// Get the provider name for this error
    pub fn provider_name(&self) -> &str {
        match self {
            ApiError::Unauthorized { provider, .. } => provider,
            ApiError::Forbidden { provider } => provider,
            ApiError::RateLimited { provider, .. } => provider,
            ApiError::NetworkError { provider, .. } => provider,
            ApiError::HttpError { provider, .. } => provider,
            ApiError::NotConfigured { provider } => provider,
        }
    }

    /// Check if this is a rate limiting error
    pub fn is_rate_limited(&self) -> bool {
        matches!(self, ApiError::RateLimited { .. })
    }

    /// Get retry-after seconds if rate limited
    pub fn retry_after(&self) -> Option<u64> {
        match self {
            ApiError::RateLimited {
                retry_after_secs, ..
            } => *retry_after_secs,
            _ => None,
        }
    }

    /// Create an unauthorized error for a provider
    pub fn unauthorized(provider: impl Into<String>) -> Self {
        ApiError::Unauthorized {
            provider: provider.into(),
            consecutive_count: 1,
        }
    }

    /// Create a forbidden error for a provider
    pub fn forbidden(provider: impl Into<String>) -> Self {
        ApiError::Forbidden {
            provider: provider.into(),
        }
    }

    /// Create a rate limited error for a provider
    pub fn rate_limited(provider: impl Into<String>, retry_after: Option<u64>) -> Self {
        ApiError::RateLimited {
            provider: provider.into(),
            retry_after_secs: retry_after,
        }
    }

    /// Create a network error for a provider
    pub fn network(provider: impl Into<String>, message: impl Into<String>) -> Self {
        ApiError::NetworkError {
            provider: provider.into(),
            message: message.into(),
        }
    }

    /// Create an HTTP error for a provider
    pub fn http(provider: impl Into<String>, status: u16, message: impl Into<String>) -> Self {
        ApiError::HttpError {
            provider: provider.into(),
            status,
            message: message.into(),
        }
    }

    /// Create a not configured error for a provider
    pub fn not_configured(provider: impl Into<String>) -> Self {
        ApiError::NotConfigured {
            provider: provider.into(),
        }
    }

    /// Increment the consecutive count for unauthorized errors
    pub fn with_consecutive_count(self, count: u32) -> Self {
        match self {
            ApiError::Unauthorized { provider, .. } => ApiError::Unauthorized {
                provider,
                consecutive_count: count,
            },
            other => other,
        }
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::Unauthorized {
                provider,
                consecutive_count,
            } => {
                write!(
                    f,
                    "{}: Unauthorized (401) - {} consecutive failures",
                    provider, consecutive_count
                )
            }
            ApiError::Forbidden { provider } => {
                write!(
                    f,
                    "{}: Forbidden (403) - insufficient permissions",
                    provider
                )
            }
            ApiError::RateLimited {
                provider,
                retry_after_secs,
            } => {
                if let Some(secs) = retry_after_secs {
                    write!(f, "{}: Rate limited - retry after {}s", provider, secs)
                } else {
                    write!(f, "{}: Rate limited", provider)
                }
            }
            ApiError::NetworkError { provider, message } => {
                write!(f, "{}: Network error - {}", provider, message)
            }
            ApiError::HttpError {
                provider,
                status,
                message,
            } => {
                write!(f, "{}: HTTP {} - {}", provider, status, message)
            }
            ApiError::NotConfigured { provider } => {
                write!(f, "{}: Not configured (no API token)", provider)
            }
        }
    }
}

impl std::error::Error for ApiError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_auth_error() {
        assert!(ApiError::unauthorized("test").is_auth_error());
        assert!(ApiError::forbidden("test").is_auth_error());
        assert!(!ApiError::rate_limited("test", None).is_auth_error());
        assert!(!ApiError::network("test", "timeout").is_auth_error());
    }

    #[test]
    fn test_needs_token_refresh() {
        let err1 = ApiError::Unauthorized {
            provider: "test".to_string(),
            consecutive_count: 2,
        };
        assert!(!err1.needs_token_refresh());

        let err2 = ApiError::Unauthorized {
            provider: "test".to_string(),
            consecutive_count: 3,
        };
        assert!(err2.needs_token_refresh());

        let err3 = ApiError::Unauthorized {
            provider: "test".to_string(),
            consecutive_count: 5,
        };
        assert!(err3.needs_token_refresh());
    }

    #[test]
    fn test_provider_name() {
        assert_eq!(
            ApiError::unauthorized("anthropic").provider_name(),
            "anthropic"
        );
        assert_eq!(ApiError::forbidden("github").provider_name(), "github");
        assert_eq!(
            ApiError::rate_limited("openai", Some(60)).provider_name(),
            "openai"
        );
    }

    #[test]
    fn test_with_consecutive_count() {
        let err = ApiError::unauthorized("test").with_consecutive_count(5);
        match err {
            ApiError::Unauthorized {
                consecutive_count, ..
            } => {
                assert_eq!(consecutive_count, 5);
            }
            _ => panic!("Expected Unauthorized variant"),
        }
    }

    #[test]
    fn test_display() {
        let err = ApiError::rate_limited("anthropic", Some(30));
        assert_eq!(err.to_string(), "anthropic: Rate limited - retry after 30s");

        let err = ApiError::not_configured("github");
        assert_eq!(err.to_string(), "github: Not configured (no API token)");
    }
}

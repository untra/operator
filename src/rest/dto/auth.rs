//! DTOs for authentication: bootstrap, sessions, OAuth device flow, and
//! service access keys.
//!
//! Two rules hold across every type in this module, and the tests at the
//! bottom enforce both:
//!
//! 1. **A secret crosses the wire only where authentication requires it.**
//!    Bootstrap, login, and password reset accept passwords inbound;
//!    access-key and token creation return a secret exactly once at creation.
//! 2. **No summary type ever carries a hash.** Metadata DTOs describe a
//!    credential (created, expires, last used, revoked) so it can be managed
//!    without ever exposing the material used to authenticate with it.

use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;

pub const MAX_ACCESS_KEY_EXPIRY_DAYS: u64 = 365;
pub const MAX_IDENTIFIER_LENGTH: usize = 128;

// =============================================================================
// Scopes
// =============================================================================

/// A typed authorization scope.
///
/// Scopes are **not hierarchical**: `Write` does not imply `Read`. A credential
/// is granted each scope it needs explicitly, so an integration's authority is
/// legible from its scope list alone rather than requiring the reader to reason
/// about implication.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
    ToSchema,
    JsonSchema,
    TS,
)]
#[ts(export)]
#[serde(rename_all = "lowercase")]
pub enum Scope {
    /// Observe the queue, agents, tickets, projects, and issue types.
    Read,
    /// Mutate tickets, issue types, steps, and collections.
    Write,
    /// Launch agents, complete steps, probe providers, call MCP tools.
    Execute,
    /// Configuration, delegators, model servers, and auth administration.
    Admin,
}

impl Scope {
    /// Every scope, in privilege-suggesting order. The single source of truth
    /// for "all scopes" across the API, the CLI, and the clients.
    ///
    /// Unused in the binary until Phase 3 wires route authorization; the DTO
    /// contract ships a phase ahead of its consumers so generated clients and
    /// the OpenAPI spec are settled before any handler depends on them.
    #[allow(dead_code)]
    pub const ALL: [Scope; 4] = [Scope::Read, Scope::Write, Scope::Execute, Scope::Admin];

    /// Wire representation, matching the `serde` rename.
    pub fn as_str(self) -> &'static str {
        match self {
            Scope::Read => "read",
            Scope::Write => "write",
            Scope::Execute => "execute",
            Scope::Admin => "admin",
        }
    }
}

impl std::fmt::Display for Scope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for Scope {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "read" => Ok(Scope::Read),
            "write" => Ok(Scope::Write),
            "execute" => Ok(Scope::Execute),
            "admin" => Ok(Scope::Admin),
            _ => Err(()),
        }
    }
}

// =============================================================================
// Bootstrap
// =============================================================================

/// Where the deployment sits in the one-time admin-creation sequence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
#[serde(rename_all = "snake_case")]
pub enum BootstrapState {
    /// No admin account exists. The bootstrap endpoint accepts a submission.
    Uninitialized,
    /// A temporary password was supplied out of band; a new one must be set
    /// before the account is usable.
    AwaitingPassword,
    /// The admin account is usable. Bootstrap is closed permanently.
    Complete,
}

/// Current bootstrap state, readable without authentication so a client can
/// route a first-time visitor to setup rather than to login.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct BootstrapStatusResponse {
    /// The state this deployment is in.
    pub state: BootstrapState,
    /// Whether a temporary password was supplied out of band (a mounted
    /// bootstrap secret). When true, submission must present it.
    pub requires_temporary_password: bool,
}

/// Claim the admin account and set its password.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct BootstrapSubmitRequest {
    /// The out-of-band temporary password, when
    /// `requires_temporary_password` is set. Never persisted or logged.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schema(write_only, format = Password, min_length = 12, max_length = 1024)]
    pub temporary_password: Option<String>,
    /// The admin password to set. Never persisted in plaintext or logged.
    #[schema(write_only, format = Password, min_length = 12, max_length = 1024)]
    pub new_password: String,
}

/// Result of a successful bootstrap.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct BootstrapSubmitResponse {
    /// The state after submission — `Complete` on success.
    pub state: BootstrapState,
    /// The account name created by bootstrap.
    pub username: String,
}

// =============================================================================
// Browser sessions
// =============================================================================

/// Password login, exchanged for an opaque server-side session cookie.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct LoginRequest {
    /// The account name. Required even while Operator supports one human account.
    #[schema(min_length = 1, max_length = 128)]
    pub username: String,
    /// The admin password. Never persisted in plaintext or logged.
    #[schema(write_only, format = Password, min_length = 12, max_length = 1024)]
    pub password: String,
}

/// Request recovery instructions without revealing whether an account exists.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct ForgotPasswordRequest {
    #[schema(min_length = 1, max_length = 128)]
    pub username: String,
}

/// Generic recovery guidance for a self-hosted Operator deployment.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct ForgotPasswordResponse {
    pub message: String,
}

/// Change the account password after proving knowledge of the current one.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct ResetPasswordRequest {
    #[schema(min_length = 1, max_length = 128)]
    pub username: String,
    #[schema(write_only, format = Password, min_length = 12, max_length = 1024)]
    pub current_password: String,
    #[schema(write_only, format = Password, min_length = 12, max_length = 1024)]
    pub new_password: String,
}

/// Result of changing the account password.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct ResetPasswordResponse {
    pub changed: bool,
}

/// Successful login. The session itself rides in a `Set-Cookie` header, not in
/// this body — a body-borne session identifier would be readable by script.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct LoginResponse {
    /// Scopes the session holds.
    pub scopes: Vec<Scope>,
    /// When the session expires.
    pub expires_at: DateTime<Utc>,
    /// CSRF token to send on subsequent cookie-authenticated mutations.
    #[schema(read_only, format = Password)]
    pub csrf_token: String,
}

/// Result of destroying the current session server-side.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct LogoutResponse {
    /// Always true; present so the response has a stable, non-empty shape.
    pub ended: bool,
}

/// The caller's current authenticated identity.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct CurrentSessionResponse {
    /// Account name — always `admin`, the single human account.
    pub subject: String,
    /// Scopes this credential holds.
    pub scopes: Vec<Scope>,
    /// How the caller authenticated.
    pub principal_kind: PrincipalKind,
    /// When this credential expires, if it does.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<Utc>>,
}

/// What kind of credential authenticated a request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
#[serde(rename_all = "snake_case")]
pub enum PrincipalKind {
    /// A browser session cookie.
    Session,
    /// A bearer access token from the device flow or a key exchange.
    AccessToken,
    /// The automatically issued loopback credential for a local process.
    LocalProcess,
    /// A single-purpose agent step-completion callback token.
    AgentCallback,
}

/// A freshly minted CSRF token for the current session.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct CsrfTokenResponse {
    /// Send as the CSRF header on cookie-authenticated mutations.
    #[schema(read_only, format = Password)]
    pub csrf_token: String,
}

// =============================================================================
// OAuth device authorization
// =============================================================================

/// Begin device authorization for a public client that cannot hold a secret.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct DeviceAuthorizationRequest {
    /// Identifier for the requesting client (e.g. `vscode`).
    #[schema(min_length = 1, max_length = 128, pattern = "^[A-Za-z0-9._:-]+$")]
    pub client_id: String,
    /// Scopes requested. IDE clients request all four, because such a client
    /// acts as the human admin.
    #[serde(default)]
    pub scopes: Vec<Scope>,
}

/// RFC 8628 device authorization response.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct DeviceAuthorizationResponse {
    /// Opaque code the client polls the token endpoint with. Never logged.
    #[schema(read_only, format = Password, min_length = 43, max_length = 43)]
    pub device_code: String,
    /// Short code the human types into the approval screen.
    #[schema(
        read_only,
        pattern = "^[A-HJ-KM-NP-TV-Z2-9]{4}-[A-HJ-KM-NP-TV-Z2-9]{4}$"
    )]
    pub user_code: String,
    /// Where the human goes to approve.
    pub verification_uri: String,
    /// `verification_uri` with the user code pre-filled.
    pub verification_uri_complete: String,
    /// Seconds until the device code expires.
    pub expires_in: u64,
    /// Minimum seconds the client must wait between polls.
    pub interval: u64,
}

/// Approve a pending device authorization from an authenticated session.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct DeviceApprovalRequest {
    /// The user code shown on the requesting device.
    #[schema(pattern = "^[A-HJ-KM-NP-TV-Z2-9]{4}-[A-HJ-KM-NP-TV-Z2-9]{4}$")]
    pub user_code: String,
}

/// Result of approving a device.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct DeviceApprovalResponse {
    /// Client that requested authorization, echoed so the approver can confirm.
    pub client_id: String,
    /// Scopes granted.
    pub scopes: Vec<Scope>,
    /// Whether approval completed.
    pub approved: bool,
}

// =============================================================================
// Token endpoint
// =============================================================================

/// Token endpoint request. The discriminator makes unrelated credential
/// combinations unrepresentable.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
#[serde(tag = "grant_type")]
pub enum TokenRequest {
    /// Poll for a previously approved device authorization.
    #[serde(rename = "urn:ietf:params:oauth:grant-type:device_code")]
    DeviceCode {
        #[schema(write_only, format = Password, min_length = 43, max_length = 43)]
        device_code: String,
        #[schema(min_length = 1, max_length = 128, pattern = "^[A-Za-z0-9._:-]+$")]
        client_id: String,
    },
    /// Redeem a rotating refresh token.
    #[serde(rename = "refresh_token")]
    RefreshToken {
        #[schema(write_only, format = Password, min_length = 43, max_length = 43)]
        refresh_token: String,
        #[schema(min_length = 1, max_length = 128, pattern = "^[A-Za-z0-9._:-]+$")]
        client_id: String,
    },
    /// Exchange a service access key.
    #[serde(rename = "operator:access-key")]
    AccessKey {
        #[schema(write_only, format = Password, min_length = 47, max_length = 47)]
        access_key: String,
    },
}

/// A newly issued access token, and a refresh token when the grant produces one.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct TokenResponse {
    /// Signed, short-lived bearer token.
    #[schema(read_only, format = Password)]
    pub access_token: String,
    /// Always `Bearer`.
    pub token_type: String,
    /// Seconds until `access_token` expires.
    pub expires_in: u64,
    /// Opaque rotating refresh token. Absent for access-key exchange, which is
    /// re-exercised with the key itself rather than refreshed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schema(read_only, format = Password)]
    pub refresh_token: Option<String>,
    /// Scopes the access token carries.
    pub scopes: Vec<Scope>,
}

/// Standardized OAuth error, shaped per RFC 6749 §5.2 so stock clients can
/// interpret it — notably `authorization_pending` and `slow_down`, which a
/// device-flow client polls against.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct OAuthErrorResponse {
    /// Machine-readable error code.
    pub error: OAuthErrorCode,
    /// Human-readable explanation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_description: Option<String>,
    /// Documentation link.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_uri: Option<String>,
}

/// OAuth error codes Operator emits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
#[serde(rename_all = "snake_case")]
pub enum OAuthErrorCode {
    /// The device code is valid but the human has not approved yet — keep polling.
    AuthorizationPending,
    /// Polling faster than `interval`; back off.
    SlowDown,
    /// The device code expired before approval.
    ExpiredToken,
    /// The human declined.
    AccessDenied,
    /// The credential presented is invalid, expired, revoked, or already used.
    InvalidGrant,
    /// The request is missing a required field or is internally inconsistent.
    InvalidRequest,
    /// The client identifier is not recognized.
    InvalidClient,
    /// The requested scopes exceed what this credential may be granted.
    InvalidScope,
    /// The grant type is not supported.
    UnsupportedGrantType,
}

// =============================================================================
// Service access keys
// =============================================================================

/// Create a service access key for an integration.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct CreateAccessKeyRequest {
    /// Human-readable label identifying what holds this key.
    #[schema(min_length = 1, max_length = 128)]
    pub name: String,
    /// Scopes to grant. Only what the integration needs.
    #[schema(min_items = 1, max_items = 4)]
    pub scopes: Vec<Scope>,
    /// Days until the key expires. Expiry is mandatory — there is no
    /// non-expiring key.
    #[schema(minimum = 1, maximum = 365)]
    pub expires_in_days: u64,
}

/// A newly created access key. **The secret appears here and nowhere else,
/// ever** — only its hash is stored, so it cannot be shown again.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct CreateAccessKeyResponse {
    /// Metadata for the created key.
    pub key: AccessKeySummary,
    /// The key secret, returned exactly once. Store it now; it is unrecoverable.
    #[schema(read_only, format = Password, min_length = 47, max_length = 47)]
    pub secret: String,
}

/// Access key metadata. Carries no secret and no hash.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct AccessKeySummary {
    /// Stable identifier, safe to display and to reference for revocation.
    #[schema(format = Uuid)]
    pub id: String,
    /// Human-readable label.
    pub name: String,
    /// Scopes granted.
    pub scopes: Vec<Scope>,
    /// When the key was created.
    pub created_at: DateTime<Utc>,
    /// When the key expires.
    pub expires_at: DateTime<Utc>,
    /// When the key was last exchanged for a token; `None` if never used.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_used_at: Option<DateTime<Utc>>,
    /// When the key was revoked; `None` while active.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub revoked_at: Option<DateTime<Utc>>,
}

/// All access keys, active and revoked.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct AccessKeyListResponse {
    /// The keys.
    pub keys: Vec<AccessKeySummary>,
}

/// Result of revoking an access key.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct RevokeAccessKeyResponse {
    /// The revoked key's identifier.
    #[schema(format = Uuid)]
    pub id: String,
    /// When revocation took effect.
    pub revoked_at: DateTime<Utc>,
}

// =============================================================================
// Session and device metadata
// =============================================================================

/// An active or expired browser session. Carries no session identifier — the
/// cookie value is never readable back out, only the session's `id` for
/// revocation.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct SessionSummary {
    /// Stable identifier, safe to display and to reference for revocation.
    #[schema(format = Uuid)]
    pub id: String,
    /// When the session began.
    pub created_at: DateTime<Utc>,
    /// When the session expires.
    pub expires_at: DateTime<Utc>,
    /// When the session was last used.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_used_at: Option<DateTime<Utc>>,
    /// When the session was revoked; `None` while active.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub revoked_at: Option<DateTime<Utc>>,
    /// Whether this is the session making the request.
    pub current: bool,
}

/// A client authorized through the device flow.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct DeviceSummary {
    /// Stable identifier, safe to display and to reference for revocation.
    #[schema(format = Uuid)]
    pub id: String,
    /// Client identifier supplied at authorization (e.g. `vscode`).
    #[schema(min_length = 1, max_length = 128, pattern = "^[A-Za-z0-9._:-]+$")]
    pub client_id: String,
    /// Scopes granted to this device.
    pub scopes: Vec<Scope>,
    /// When the device was approved.
    pub created_at: DateTime<Utc>,
    /// When the device's refresh credential reaches its absolute deadline.
    pub expires_at: DateTime<Utc>,
    /// When the device last refreshed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_used_at: Option<DateTime<Utc>>,
    /// When the device was revoked; `None` while active.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub revoked_at: Option<DateTime<Utc>>,
}

/// All sessions and devices for the admin account.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct SessionListResponse {
    /// Browser sessions.
    pub sessions: Vec<SessionSummary>,
    /// Device-flow clients.
    pub devices: Vec<DeviceSummary>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ts(s: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(s).unwrap().with_timezone(&Utc)
    }

    #[test]
    fn test_scope_wire_format_is_lowercase() {
        let json = serde_json::to_string(&Scope::ALL.to_vec()).unwrap();
        assert_eq!(json, r#"["read","write","execute","admin"]"#);
    }

    #[test]
    fn test_scope_roundtrips_through_str() {
        for scope in Scope::ALL {
            assert_eq!(scope.as_str().parse::<Scope>(), Ok(scope));
        }
        assert_eq!("".parse::<Scope>(), Err(()));
        assert_eq!("superuser".parse::<Scope>(), Err(()));
    }

    #[test]
    fn test_scopes_are_not_hierarchical() {
        // Ord exists for stable sorting and set membership, not for privilege
        // implication: holding Write must never imply holding Read. Anything
        // that grants access does so by explicit membership.
        let granted = [Scope::Write];
        assert!(!granted.contains(&Scope::Read));
        assert!(!granted.contains(&Scope::Admin));
    }

    #[test]
    fn test_access_key_summary_carries_no_secret_or_hash() {
        // The management view describes a key without exposing anything usable
        // to authenticate with it.
        let summary = AccessKeySummary {
            id: "ak_7f3a".to_string(),
            name: "ci-pipeline".to_string(),
            scopes: vec![Scope::Read, Scope::Execute],
            created_at: ts("2026-01-01T00:00:00Z"),
            expires_at: ts("2026-04-01T00:00:00Z"),
            last_used_at: Some(ts("2026-01-05T12:00:00Z")),
            revoked_at: None,
        };
        let json = serde_json::to_string(&summary).unwrap();
        assert!(json.contains("\"id\":\"ak_7f3a\""));
        assert!(json.contains("\"scopes\":[\"read\",\"execute\"]"));
        for forbidden in ["secret", "hash", "password", "token"] {
            assert!(
                !json.contains(forbidden),
                "AccessKeySummary must not carry a `{forbidden}` field: {json}"
            );
        }
    }

    #[test]
    fn test_create_access_key_response_returns_secret_exactly_once() {
        // Creation is the one place a key secret is ever on the wire; the
        // nested summary still carries none.
        let resp = CreateAccessKeyResponse {
            key: AccessKeySummary {
                id: "ak_7f3a".to_string(),
                name: "ci".to_string(),
                scopes: vec![Scope::Read],
                created_at: ts("2026-01-01T00:00:00Z"),
                expires_at: ts("2026-04-01T00:00:00Z"),
                last_used_at: None,
                revoked_at: None,
            },
            secret: "opk_live_abc123".to_string(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert_eq!(json.matches("opk_live_abc123").count(), 1);
    }

    #[test]
    fn test_session_and_device_summaries_carry_no_credential() {
        let session = SessionSummary {
            id: "sess_1".to_string(),
            created_at: ts("2026-01-01T00:00:00Z"),
            expires_at: ts("2026-01-02T00:00:00Z"),
            last_used_at: None,
            revoked_at: None,
            current: true,
        };
        let device = DeviceSummary {
            id: "dev_1".to_string(),
            client_id: "vscode".to_string(),
            scopes: Scope::ALL.to_vec(),
            created_at: ts("2026-01-01T00:00:00Z"),
            expires_at: ts("2026-04-01T00:00:00Z"),
            last_used_at: None,
            revoked_at: None,
        };
        for json in [
            serde_json::to_string(&session).unwrap(),
            serde_json::to_string(&device).unwrap(),
        ] {
            for forbidden in ["secret", "hash", "cookie", "refresh_token"] {
                assert!(
                    !json.contains(forbidden),
                    "metadata DTO must not carry `{forbidden}`: {json}"
                );
            }
        }
    }

    #[test]
    fn test_login_response_omits_session_identifier() {
        // The session rides in Set-Cookie (HttpOnly); a body-borne identifier
        // would be readable by script and defeat the cookie flags.
        let resp = LoginResponse {
            scopes: Scope::ALL.to_vec(),
            expires_at: ts("2026-01-02T00:00:00Z"),
            csrf_token: "csrf_abc".to_string(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(!json.contains("session_id"));
        assert!(json.contains("csrf_token"));
    }

    #[test]
    fn test_oauth_error_codes_match_rfc_spelling() {
        // A device-flow client branches on these exact strings while polling.
        for (code, expected) in [
            (
                OAuthErrorCode::AuthorizationPending,
                "authorization_pending",
            ),
            (OAuthErrorCode::SlowDown, "slow_down"),
            (OAuthErrorCode::ExpiredToken, "expired_token"),
            (OAuthErrorCode::AccessDenied, "access_denied"),
            (OAuthErrorCode::InvalidGrant, "invalid_grant"),
        ] {
            assert_eq!(
                serde_json::to_string(&code).unwrap(),
                format!("\"{expected}\"")
            );
        }
    }

    #[test]
    fn test_token_request_is_discriminated_by_grant() {
        let req = TokenRequest::RefreshToken {
            refresh_token: "rt_abc".to_string(),
            client_id: "vscode".to_string(),
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("refresh_token"));
        assert!(!json.contains("device_code"));
        assert!(!json.contains("access_key"));
    }

    #[test]
    fn test_bootstrap_states_are_snake_case() {
        for (state, expected) in [
            (BootstrapState::Uninitialized, "uninitialized"),
            (BootstrapState::AwaitingPassword, "awaiting_password"),
            (BootstrapState::Complete, "complete"),
        ] {
            assert_eq!(
                serde_json::to_string(&state).unwrap(),
                format!("\"{expected}\"")
            );
        }
    }
}

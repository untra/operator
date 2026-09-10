//! The auth database and the service wrapping it.
//!
//! `rusqlite` is synchronous, so every public method here is `async` and does
//! its work inside `spawn_blocking`. The connection sits behind a `Mutex`
//! rather than a pool: this is a single-writer application over one small
//! database, and a pool would add contention management for no benefit.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Duration, Utc};
use rusqlite::{Connection, OptionalExtension};
use uuid::Uuid;

use crate::auth::password::{hash_password, validate_password, verify_password};
use crate::auth::schema;
use crate::auth::scope::Principal;
use crate::auth::secret::{
    generate_prefixed_secret, generate_secret, generate_user_code, hash_secret,
};
use crate::auth::tokens::SigningKey;
use crate::rest::dto::auth::{
    AccessKeySummary, BootstrapState, DeviceSummary, PrincipalKind, Scope, SessionSummary,
    MAX_ACCESS_KEY_EXPIRY_DAYS, MAX_IDENTIFIER_LENGTH,
};

/// Filename of the auth database inside the state directory.
pub const AUTH_DB_FILENAME: &str = "auth.sqlite3";

/// The one human account.
pub const ADMIN_SUBJECT: &str = "admin";

/// Browser session lifetime.
const SESSION_TTL: Duration = Duration::hours(12);
/// Refresh token idle lifetime — using a token resets this.
const REFRESH_IDLE_TTL: Duration = Duration::days(30);
/// Refresh token absolute lifetime — fixed at issuance, never extended.
const REFRESH_ABSOLUTE_TTL: Duration = Duration::days(90);
/// Device code lifetime in seconds.
pub const DEVICE_CODE_TTL_SECS: u64 = 15 * 60;
/// Minimum seconds between device-code polls.
pub const DEVICE_POLL_INTERVAL_SECS: u64 = 5;

/// Prefix marking a service access key, so a leaked one is recognizable to a
/// secret scanner and in logs.
const ACCESS_KEY_PREFIX: &str = "opk";

/// Outcome of redeeming a refresh token.
#[derive(Debug)]
pub enum RefreshOutcome {
    /// Rotated successfully; the caller receives a replacement token.
    Rotated {
        refresh_token: String,
        scopes: Vec<Scope>,
    },
    /// The token was valid once but has already been redeemed. The family is
    /// now revoked — see [`AuthStore::redeem_refresh_token`].
    Reused,
    /// No such token, or it is expired or revoked.
    Invalid,
}

/// A `device_authorization` row as read during polling:
/// `(id, client_id, scopes, expires_at, approved_at, denied_at, consumed_at,
/// last_polled_at)`.
type DeviceAuthorizationRow = (
    String,
    String,
    String,
    String,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
);

/// Outcome of polling a device authorization.
#[derive(Debug)]
pub enum DevicePollOutcome {
    /// Approved; the caller may mint tokens with these scopes.
    Approved {
        client_id: String,
        scopes: Vec<Scope>,
    },
    /// Not approved yet — keep polling.
    Pending,
    /// Polled faster than the advertised interval.
    SlowDown,
    /// The human declined.
    Denied,
    /// Expired, already consumed, or unknown.
    Expired,
}

/// A persistent rate-limit decision.
#[derive(Debug, PartialEq, Eq)]
pub enum RateLimitDecision {
    /// Proceed.
    Allow,
    /// Backoff is in effect; retry after this many seconds.
    Backoff { retry_after_secs: u64 },
}

/// The auth database.
#[derive(Clone)]
pub struct AuthStore {
    conn: Arc<Mutex<Connection>>,
    path: PathBuf,
}

impl std::fmt::Debug for AuthStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Hand-written so the connection is never formatted: it is a live
        // handle to a database full of credential hashes.
        f.debug_struct("AuthStore")
            .field("path", &self.path)
            .field("conn", &"<sqlite connection>")
            .finish()
    }
}

/// Restrict a file to its owner. Credentials live in this database, and the
/// local-unlock token is authenticated *by* file ownership, so the mode is a
/// security control rather than tidiness.
#[cfg(unix)]
pub fn restrict_to_owner(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let mut perms = std::fs::metadata(path)
        .with_context(|| format!("reading permissions of {}", path.display()))?
        .permissions();
    perms.set_mode(0o600);
    std::fs::set_permissions(path, perms)
        .with_context(|| format!("restricting {} to its owner", path.display()))
}

/// On Windows the file inherits the user-profile ACL, which already excludes
/// other users; there is no portable mode bit to set.
#[cfg(not(unix))]
pub fn restrict_to_owner(_path: &Path) -> Result<()> {
    Ok(())
}

impl AuthStore {
    /// Open (creating if needed) the auth database at `state_path`.
    pub fn open(state_path: &Path) -> Result<Self> {
        std::fs::create_dir_all(state_path)
            .with_context(|| format!("creating state directory {}", state_path.display()))?;
        let path = state_path.join(AUTH_DB_FILENAME);

        let mut conn = Connection::open(&path)
            .with_context(|| format!("opening auth database {}", path.display()))?;
        // Restrict before writing anything into it.
        restrict_to_owner(&path)?;
        schema::apply_pragmas(&conn)?;
        schema::migrate(&mut conn)?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            path,
        })
    }

    /// An in-memory store, for tests.
    #[cfg(test)]
    pub fn in_memory() -> Result<Self> {
        let mut conn = Connection::open_in_memory()?;
        schema::apply_pragmas(&conn)?;
        schema::migrate(&mut conn)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            path: PathBuf::from(":memory:"),
        })
    }

    fn with_conn<T>(&self, f: impl FnOnce(&mut Connection) -> Result<T>) -> Result<T> {
        let mut guard = self
            .conn
            .lock()
            .map_err(|_| anyhow!("auth database lock poisoned"))?;
        f(&mut guard)
    }

    // =========================================================================
    // Bootstrap and the admin account
    // =========================================================================

    /// Current bootstrap state.
    pub fn bootstrap_state(&self) -> Result<BootstrapState> {
        self.with_conn(|conn| {
            let row: Option<i64> = conn
                .query_row(
                    "SELECT awaiting_reset FROM admin_account WHERE id = 1",
                    [],
                    |r| r.get(0),
                )
                .optional()?;
            Ok(match row {
                None => BootstrapState::Uninitialized,
                Some(0) => BootstrapState::Complete,
                Some(_) => BootstrapState::AwaitingPassword,
            })
        })
    }

    /// Create the admin account. **Atomic**: the `id = 1` primary key means a
    /// concurrent second attempt fails rather than creating a second admin, so
    /// exactly one racer wins with no application-level locking.
    ///
    /// `awaiting_reset` marks a password that came from a mounted bootstrap
    /// secret and must be replaced before the account is usable.
    pub fn create_admin(&self, password: &str, awaiting_reset: bool) -> Result<bool> {
        validate_password(password)?;
        let phc = hash_password(password)?;
        let now = Utc::now().to_rfc3339();

        self.with_conn(|conn| {
            let inserted = conn.execute(
                "INSERT OR ABORT INTO admin_account \
                 (id, subject, password_hash, awaiting_reset, created_at, updated_at) \
                 VALUES (1, ?1, ?2, ?3, ?4, ?4)",
                rusqlite::params![ADMIN_SUBJECT, phc, i64::from(awaiting_reset), now],
            );
            match inserted {
                Ok(_) => Ok(true),
                // A losing racer, not a fault.
                Err(rusqlite::Error::SqliteFailure(e, _))
                    if e.code == rusqlite::ErrorCode::ConstraintViolation =>
                {
                    Ok(false)
                }
                Err(e) => Err(e.into()),
            }
        })
    }

    /// Replace the admin password and clear `awaiting_reset`.
    pub fn set_admin_password(&self, password: &str) -> Result<()> {
        validate_password(password)?;
        let phc = hash_password(password)?;
        let now = Utc::now().to_rfc3339();

        self.with_conn(|conn| {
            let n = conn.execute(
                "UPDATE admin_account SET password_hash = ?1, awaiting_reset = 0, updated_at = ?2 \
                 WHERE id = 1",
                rusqlite::params![phc, now],
            )?;
            if n == 0 {
                return Err(anyhow!("no admin account exists"));
            }
            Ok(())
        })
    }

    /// Verify a password against the stored admin hash.
    pub fn verify_admin_password(&self, password: &str) -> Result<bool> {
        let phc: Option<String> = self.with_conn(|conn| {
            Ok(conn
                .query_row(
                    "SELECT password_hash FROM admin_account WHERE id = 1",
                    [],
                    |r| r.get(0),
                )
                .optional()?)
        })?;

        match phc {
            Some(phc) => verify_password(password, &phc),
            None => Ok(false),
        }
    }

    /// Revoke every credential: sessions, refresh families, access keys, and
    /// pending device authorizations. Used by password reset and re-bootstrap.
    pub fn revoke_all_credentials(&self, reason: &str) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        self.with_conn(|conn| {
            let tx = conn.transaction()?;
            tx.execute(
                "UPDATE session SET revoked_at = ?1 WHERE revoked_at IS NULL",
                [&now],
            )?;
            tx.execute(
                "UPDATE refresh_family SET revoked_at = ?1, revoked_reason = ?2 \
                 WHERE revoked_at IS NULL",
                rusqlite::params![now, reason],
            )?;
            tx.execute(
                "UPDATE access_key SET revoked_at = ?1 WHERE revoked_at IS NULL",
                [&now],
            )?;
            tx.execute(
                "UPDATE device_authorization SET denied_at = ?1 \
                 WHERE approved_at IS NULL AND denied_at IS NULL",
                [&now],
            )?;
            tx.commit()?;
            Ok(())
        })
    }

    // =========================================================================
    // Signing key
    // =========================================================================

    /// Load the active signing key, generating and persisting one on first use.
    pub fn load_or_create_signing_key(&self) -> Result<SigningKey> {
        let existing: Option<(String, Vec<u8>, Vec<u8>)> = self.with_conn(|conn| {
            Ok(conn
                .query_row(
                    "SELECT kid, private_pem, public_pem FROM signing_key \
                     WHERE retired_at IS NULL ORDER BY created_at DESC LIMIT 1",
                    [],
                    |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
                )
                .optional()?)
        })?;

        if let Some((kid, private_der, public_raw)) = existing {
            return SigningKey::from_der(kid, private_der, public_raw);
        }

        let kid = Uuid::new_v4().to_string();
        let key = SigningKey::generate(kid.clone())?;
        let now = Utc::now().to_rfc3339();
        self.with_conn(|conn| {
            conn.execute(
                "INSERT INTO signing_key (kid, private_pem, public_pem, created_at) \
                 VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params![kid, key.private_der(), key.public_raw(), now],
            )?;
            Ok(())
        })?;
        Ok(key)
    }

    // =========================================================================
    // Browser sessions
    // =========================================================================

    /// Create a session, returning `(session_token, csrf_token, expires_at)`.
    /// Only hashes are stored, so neither value can be read back out.
    pub fn create_session(&self) -> Result<(String, String, DateTime<Utc>)> {
        let token = generate_secret()?;
        let csrf = generate_secret()?;
        let now = Utc::now();
        let expires = now + SESSION_TTL;

        self.with_conn(|conn| {
            conn.execute(
                "INSERT INTO session (id, token_hash, csrf_hash, created_at, expires_at) \
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![
                    Uuid::new_v4().to_string(),
                    hash_secret(&token),
                    hash_secret(&csrf),
                    now.to_rfc3339(),
                    expires.to_rfc3339(),
                ],
            )?;
            Ok(())
        })?;

        Ok((token, csrf, expires))
    }

    /// Resolve a session cookie to a principal, refreshing `last_used_at`.
    pub fn authenticate_session(&self, token: &str) -> Result<Option<Principal>> {
        let hash = hash_secret(token);
        let now = Utc::now();

        self.with_conn(|conn| {
            let row: Option<(String, String)> = conn
                .query_row(
                    "SELECT id, expires_at FROM session \
                     WHERE token_hash = ?1 AND revoked_at IS NULL",
                    [&hash],
                    |r| Ok((r.get(0)?, r.get(1)?)),
                )
                .optional()?;

            let Some((id, expires_at)) = row else {
                return Ok(None);
            };
            let expires_at = parse_time(&expires_at)?;
            if expires_at <= now {
                return Ok(None);
            }

            conn.execute(
                "UPDATE session SET last_used_at = ?1 WHERE id = ?2",
                rusqlite::params![now.to_rfc3339(), id],
            )?;

            Ok(Some(Principal {
                subject: ADMIN_SUBJECT.to_string(),
                scopes: Scope::ALL.to_vec(),
                kind: PrincipalKind::Session,
                expires_at: Some(expires_at),
                session_id: Some(id),
                ticket_id: None,
                step: None,
            }))
        })
    }

    /// Whether `csrf` matches the session's stored CSRF hash.
    pub fn verify_csrf(&self, session_id: &str, csrf: &str) -> Result<bool> {
        let provided = hash_secret(csrf);
        self.with_conn(|conn| {
            let stored: Option<String> = conn
                .query_row(
                    "SELECT csrf_hash FROM session WHERE id = ?1 AND revoked_at IS NULL",
                    [session_id],
                    |r| r.get(0),
                )
                .optional()?;
            Ok(stored.is_some_and(|s| crate::auth::secret::hashes_equal(&s, &provided)))
        })
    }

    /// Issue a fresh CSRF token for an existing session, replacing the old one.
    ///
    /// The SPA needs this after a page reload: the session cookie survives, but
    /// the CSRF token was only ever held in memory. Rotating rather than
    /// returning the existing one means the stored value stays hash-only.
    pub fn rotate_csrf(&self, session_id: &str) -> Result<Option<String>> {
        let csrf = generate_secret()?;
        let hash = hash_secret(&csrf);
        self.with_conn(|conn| {
            let n = conn.execute(
                "UPDATE session SET csrf_hash = ?1 WHERE id = ?2 AND revoked_at IS NULL",
                rusqlite::params![hash, session_id],
            )?;
            Ok((n > 0).then_some(csrf))
        })
    }

    /// Revoke one session.
    pub fn revoke_session(&self, session_id: &str) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        self.with_conn(|conn| {
            conn.execute(
                "UPDATE session SET revoked_at = ?1 WHERE id = ?2 AND revoked_at IS NULL",
                rusqlite::params![now, session_id],
            )?;
            Ok(())
        })
    }

    /// All sessions, newest first.
    pub fn list_sessions(&self, current_session_id: Option<&str>) -> Result<Vec<SessionSummary>> {
        self.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, created_at, expires_at, last_used_at, revoked_at \
                 FROM session ORDER BY created_at DESC",
            )?;
            let rows = stmt
                .query_map([], |r| {
                    Ok((
                        r.get::<_, String>(0)?,
                        r.get::<_, String>(1)?,
                        r.get::<_, String>(2)?,
                        r.get::<_, Option<String>>(3)?,
                        r.get::<_, Option<String>>(4)?,
                    ))
                })?
                .collect::<std::result::Result<Vec<_>, _>>()?;

            rows.into_iter()
                .map(|(id, created, expires, last_used, revoked)| {
                    Ok(SessionSummary {
                        current: current_session_id == Some(id.as_str()),
                        id,
                        created_at: parse_time(&created)?,
                        expires_at: parse_time(&expires)?,
                        last_used_at: parse_opt_time(last_used.as_deref())?,
                        revoked_at: parse_opt_time(revoked.as_deref())?,
                    })
                })
                .collect()
        })
    }

    // =========================================================================
    // Refresh tokens
    // =========================================================================

    /// Start a refresh-token family and issue its first token.
    pub fn create_refresh_family(&self, client_id: &str, scopes: &[Scope]) -> Result<String> {
        let now = Utc::now();
        let token = generate_secret()?;
        let family_id = Uuid::new_v4().to_string();

        self.with_conn(|conn| {
            let tx = conn.transaction()?;
            tx.execute(
                "INSERT INTO refresh_family \
                 (id, client_id, scopes, created_at, absolute_expires_at) \
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![
                    family_id,
                    client_id,
                    encode_scopes(scopes),
                    now.to_rfc3339(),
                    (now + REFRESH_ABSOLUTE_TTL).to_rfc3339(),
                ],
            )?;
            tx.execute(
                "INSERT INTO refresh_token (id, family_id, token_hash, created_at, expires_at) \
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![
                    Uuid::new_v4().to_string(),
                    family_id,
                    hash_secret(&token),
                    now.to_rfc3339(),
                    (now + REFRESH_IDLE_TTL).to_rfc3339(),
                ],
            )?;
            tx.commit()?;
            Ok(())
        })?;

        Ok(token)
    }

    /// Redeem a refresh token, rotating it.
    ///
    /// Presenting an **already-consumed** token means two parties hold the same
    /// credential — the legitimate client and a thief — and there is no way to
    /// tell which is calling. The whole family is revoked rather than guessing:
    /// a forced re-authentication is a far better outcome than silently serving
    /// an attacker.
    pub fn redeem_refresh_token(&self, token: &str, client_id: &str) -> Result<RefreshOutcome> {
        let hash = hash_secret(token);
        let now = Utc::now();
        let replacement = generate_secret()?;

        self.with_conn(|conn| {
            let tx = conn.transaction()?;

            let row: Option<(String, String, Option<String>, String)> = tx
                .query_row(
                    "SELECT id, family_id, consumed_at, expires_at FROM refresh_token \
                     WHERE token_hash = ?1",
                    [&hash],
                    |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
                )
                .optional()?;

            let Some((token_id, family_id, consumed_at, expires_at)) = row else {
                return Ok(RefreshOutcome::Invalid);
            };

            if consumed_at.is_some() {
                tx.execute(
                    "UPDATE refresh_family SET revoked_at = ?1, revoked_reason = 'token reuse' \
                     WHERE id = ?2 AND revoked_at IS NULL",
                    rusqlite::params![now.to_rfc3339(), family_id],
                )?;
                tx.commit()?;
                return Ok(RefreshOutcome::Reused);
            }

            let family: Option<(String, String, String, Option<String>)> = tx
                .query_row(
                    "SELECT client_id, scopes, absolute_expires_at, revoked_at FROM refresh_family \
                     WHERE id = ?1",
                    [&family_id],
                    |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
                )
                .optional()?;

            let Some((registered_client_id, scopes, absolute_expires_at, revoked_at)) = family
            else {
                return Ok(RefreshOutcome::Invalid);
            };
            if registered_client_id != client_id {
                return Ok(RefreshOutcome::Invalid);
            }

            // Idle expiry, absolute expiry, and revocation each end the family.
            if revoked_at.is_some()
                || parse_time(&expires_at)? <= now
                || parse_time(&absolute_expires_at)? <= now
            {
                return Ok(RefreshOutcome::Invalid);
            }

            tx.execute(
                "UPDATE refresh_token SET consumed_at = ?1 WHERE id = ?2",
                rusqlite::params![now.to_rfc3339(), token_id],
            )?;
            tx.execute(
                "INSERT INTO refresh_token (id, family_id, token_hash, created_at, expires_at) \
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![
                    Uuid::new_v4().to_string(),
                    family_id,
                    hash_secret(&replacement),
                    now.to_rfc3339(),
                    // Rotation resets the idle clock but never the absolute one.
                    (now + REFRESH_IDLE_TTL).to_rfc3339(),
                ],
            )?;
            tx.execute(
                "UPDATE refresh_family SET last_used_at = ?1 WHERE id = ?2",
                rusqlite::params![now.to_rfc3339(), family_id],
            )?;
            tx.commit()?;

            Ok(RefreshOutcome::Rotated {
                refresh_token: replacement,
                scopes: decode_scopes(&scopes),
            })
        })
    }

    /// Device-flow clients, for the security settings screen.
    pub fn list_devices(&self) -> Result<Vec<DeviceSummary>> {
        self.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, client_id, scopes, created_at, absolute_expires_at, \
                        last_used_at, revoked_at \
                 FROM refresh_family ORDER BY created_at DESC",
            )?;
            let rows = stmt
                .query_map([], |r| {
                    Ok((
                        r.get::<_, String>(0)?,
                        r.get::<_, String>(1)?,
                        r.get::<_, String>(2)?,
                        r.get::<_, String>(3)?,
                        r.get::<_, String>(4)?,
                        r.get::<_, Option<String>>(5)?,
                        r.get::<_, Option<String>>(6)?,
                    ))
                })?
                .collect::<std::result::Result<Vec<_>, _>>()?;

            rows.into_iter()
                .map(
                    |(id, client_id, scopes, created, expires, last_used, revoked)| {
                        Ok(DeviceSummary {
                            id,
                            client_id,
                            scopes: decode_scopes(&scopes),
                            created_at: parse_time(&created)?,
                            expires_at: parse_time(&expires)?,
                            last_used_at: parse_opt_time(last_used.as_deref())?,
                            revoked_at: parse_opt_time(revoked.as_deref())?,
                        })
                    },
                )
                .collect()
        })
    }

    // =========================================================================
    // Device authorization
    // =========================================================================

    /// Create a device authorization, returning `(device_code, user_code)`.
    pub fn create_device_authorization(
        &self,
        client_id: &str,
        scopes: &[Scope],
    ) -> Result<(String, String)> {
        let device_code = generate_secret()?;
        let user_code = generate_user_code()?;
        let now = Utc::now();

        self.with_conn(|conn| {
            conn.execute(
                "INSERT INTO device_authorization \
                 (id, device_code_hash, user_code, client_id, scopes, created_at, expires_at) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                rusqlite::params![
                    Uuid::new_v4().to_string(),
                    hash_secret(&device_code),
                    user_code,
                    client_id,
                    encode_scopes(scopes),
                    now.to_rfc3339(),
                    (now + Duration::seconds(DEVICE_CODE_TTL_SECS as i64)).to_rfc3339(),
                ],
            )?;
            Ok(())
        })?;

        Ok((device_code, user_code))
    }

    /// Approve a pending device authorization by its user code.
    pub fn approve_device(&self, user_code: &str) -> Result<Option<(String, Vec<Scope>)>> {
        let now = Utc::now();
        self.with_conn(|conn| {
            let row: Option<(String, String, String, String)> = conn
                .query_row(
                    "SELECT id, client_id, scopes, expires_at FROM device_authorization \
                     WHERE user_code = ?1 AND approved_at IS NULL AND denied_at IS NULL",
                    [user_code],
                    |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
                )
                .optional()?;

            let Some((id, client_id, scopes, expires_at)) = row else {
                return Ok(None);
            };
            if parse_time(&expires_at)? <= now {
                return Ok(None);
            }

            conn.execute(
                "UPDATE device_authorization SET approved_at = ?1 WHERE id = ?2",
                rusqlite::params![now.to_rfc3339(), id],
            )?;
            Ok(Some((client_id, decode_scopes(&scopes))))
        })
    }

    /// Poll a device authorization, consuming it once approved.
    ///
    /// The poll interval is enforced here rather than trusted to the client,
    /// which is the only way it actually bounds anything.
    pub fn poll_device(
        &self,
        device_code: &str,
        requesting_client_id: &str,
    ) -> Result<DevicePollOutcome> {
        let hash = hash_secret(device_code);
        let now = Utc::now();

        self.with_conn(|conn| {
            let tx = conn.transaction()?;
            let row: Option<DeviceAuthorizationRow> = tx
                .query_row(
                    "SELECT id, client_id, scopes, expires_at, approved_at, denied_at, \
                            consumed_at, last_polled_at \
                     FROM device_authorization WHERE device_code_hash = ?1",
                    [&hash],
                    |r| {
                        Ok((
                            r.get(0)?,
                            r.get(1)?,
                            r.get(2)?,
                            r.get(3)?,
                            r.get(4)?,
                            r.get(5)?,
                            r.get(6)?,
                            r.get(7)?,
                        ))
                    },
                )
                .optional()?;

            let Some((
                id,
                client_id,
                scopes,
                expires_at,
                approved_at,
                denied_at,
                consumed_at,
                last_polled_at,
            )) = row
            else {
                return Ok(DevicePollOutcome::Expired);
            };

            if client_id != requesting_client_id {
                return Ok(DevicePollOutcome::Expired);
            }

            if consumed_at.is_some() || parse_time(&expires_at)? <= now {
                return Ok(DevicePollOutcome::Expired);
            }
            if denied_at.is_some() {
                return Ok(DevicePollOutcome::Denied);
            }

            if let Some(last) = parse_opt_time(last_polled_at.as_deref())? {
                if (now - last).num_seconds() < DEVICE_POLL_INTERVAL_SECS as i64 {
                    return Ok(DevicePollOutcome::SlowDown);
                }
            }
            tx.execute(
                "UPDATE device_authorization SET last_polled_at = ?1 WHERE id = ?2",
                rusqlite::params![now.to_rfc3339(), id],
            )?;

            if approved_at.is_none() {
                tx.commit()?;
                return Ok(DevicePollOutcome::Pending);
            }

            // Consume on success so a device code is single-use.
            tx.execute(
                "UPDATE device_authorization SET consumed_at = ?1 WHERE id = ?2",
                rusqlite::params![now.to_rfc3339(), id],
            )?;
            tx.commit()?;

            Ok(DevicePollOutcome::Approved {
                client_id,
                scopes: decode_scopes(&scopes),
            })
        })
    }

    // =========================================================================
    // Service access keys
    // =========================================================================

    /// Create an access key, returning `(summary, secret)`. The secret is
    /// returned once and is unrecoverable afterward.
    pub fn create_access_key(
        &self,
        name: &str,
        scopes: &[Scope],
        expires_in_days: u64,
    ) -> Result<(AccessKeySummary, String)> {
        let name = name.trim();
        if name.is_empty() || name.len() > MAX_IDENTIFIER_LENGTH {
            return Err(anyhow!("access key name must contain 1 to 128 characters"));
        }
        if scopes.is_empty() {
            return Err(anyhow!("an access key must grant at least one scope"));
        }
        let unique_scopes: std::collections::HashSet<_> = scopes.iter().collect();
        if unique_scopes.len() != scopes.len() {
            return Err(anyhow!("access key scopes must be unique"));
        }
        if expires_in_days == 0 || expires_in_days > MAX_ACCESS_KEY_EXPIRY_DAYS {
            return Err(anyhow!("access key expiry must be between 1 and 365 days"));
        }

        let secret = generate_prefixed_secret(ACCESS_KEY_PREFIX)?;
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();
        let expires = now
            + Duration::try_days(i64::try_from(expires_in_days).unwrap_or(i64::MAX))
                .ok_or_else(|| anyhow!("expiry is too far in the future"))?;

        self.with_conn(|conn| {
            conn.execute(
                "INSERT INTO access_key (id, name, key_hash, scopes, created_at, expires_at) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                rusqlite::params![
                    id,
                    name,
                    hash_secret(&secret),
                    encode_scopes(scopes),
                    now.to_rfc3339(),
                    expires.to_rfc3339(),
                ],
            )?;
            Ok(())
        })?;

        Ok((
            AccessKeySummary {
                id,
                name: name.to_string(),
                scopes: scopes.to_vec(),
                created_at: now,
                expires_at: expires,
                last_used_at: None,
                revoked_at: None,
            },
            secret,
        ))
    }

    /// Exchange an access key for its scopes, recording the use.
    pub fn redeem_access_key(&self, secret: &str) -> Result<Option<Vec<Scope>>> {
        let hash = hash_secret(secret);
        let now = Utc::now();

        self.with_conn(|conn| {
            let row: Option<(String, String, String, Option<String>)> = conn
                .query_row(
                    "SELECT id, scopes, expires_at, revoked_at FROM access_key \
                     WHERE key_hash = ?1",
                    [&hash],
                    |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
                )
                .optional()?;

            let Some((id, scopes, expires_at, revoked_at)) = row else {
                return Ok(None);
            };
            if revoked_at.is_some() || parse_time(&expires_at)? <= now {
                return Ok(None);
            }

            conn.execute(
                "UPDATE access_key SET last_used_at = ?1 WHERE id = ?2",
                rusqlite::params![now.to_rfc3339(), id],
            )?;
            Ok(Some(decode_scopes(&scopes)))
        })
    }

    /// Revoke an access key.
    pub fn revoke_access_key(&self, id: &str) -> Result<Option<DateTime<Utc>>> {
        let now = Utc::now();
        self.with_conn(|conn| {
            let n = conn.execute(
                "UPDATE access_key SET revoked_at = ?1 WHERE id = ?2 AND revoked_at IS NULL",
                rusqlite::params![now.to_rfc3339(), id],
            )?;
            Ok((n > 0).then_some(now))
        })
    }

    /// All access keys, newest first.
    pub fn list_access_keys(&self) -> Result<Vec<AccessKeySummary>> {
        self.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, name, scopes, created_at, expires_at, last_used_at, revoked_at \
                 FROM access_key ORDER BY created_at DESC",
            )?;
            let rows = stmt
                .query_map([], |r| {
                    Ok((
                        r.get::<_, String>(0)?,
                        r.get::<_, String>(1)?,
                        r.get::<_, String>(2)?,
                        r.get::<_, String>(3)?,
                        r.get::<_, String>(4)?,
                        r.get::<_, Option<String>>(5)?,
                        r.get::<_, Option<String>>(6)?,
                    ))
                })?
                .collect::<std::result::Result<Vec<_>, _>>()?;

            rows.into_iter()
                .map(|(id, name, scopes, created, expires, last_used, revoked)| {
                    Ok(AccessKeySummary {
                        id,
                        name,
                        scopes: decode_scopes(&scopes),
                        created_at: parse_time(&created)?,
                        expires_at: parse_time(&expires)?,
                        last_used_at: parse_opt_time(last_used.as_deref())?,
                        revoked_at: parse_opt_time(revoked.as_deref())?,
                    })
                })
                .collect()
        })
    }

    // =========================================================================
    // Rate limiting
    // =========================================================================

    /// Check a rate-limit bucket without recording an attempt.
    pub fn check_rate_limit(&self, bucket: &str) -> Result<RateLimitDecision> {
        let now = Utc::now();
        self.with_conn(|conn| {
            let retry_after: Option<Option<String>> = conn
                .query_row(
                    "SELECT retry_after FROM rate_limit WHERE bucket = ?1",
                    [bucket],
                    |r| r.get(0),
                )
                .optional()?;

            let Some(Some(retry_after)) = retry_after else {
                return Ok(RateLimitDecision::Allow);
            };
            let retry_at = parse_time(&retry_after)?;
            if retry_at <= now {
                return Ok(RateLimitDecision::Allow);
            }
            Ok(RateLimitDecision::Backoff {
                retry_after_secs: (retry_at - now).num_seconds().max(1) as u64,
            })
        })
    }

    /// Record a failed attempt and extend the backoff.
    ///
    /// Delay grows exponentially and is capped. It never becomes a permanent
    /// lockout: on a single-account system that would be a denial-of-service
    /// against the only person who could undo it.
    pub fn record_failure(&self, bucket: &str) -> Result<()> {
        const MAX_BACKOFF_SECS: i64 = 15 * 60;
        let now = Utc::now();

        self.with_conn(|conn| {
            let attempts: Option<i64> = conn
                .query_row(
                    "SELECT attempts FROM rate_limit WHERE bucket = ?1",
                    [bucket],
                    |r| r.get(0),
                )
                .optional()?;

            let attempts = attempts.unwrap_or(0) + 1;
            // No delay for the first few attempts, then 2^n seconds, capped.
            let delay = if attempts <= 3 {
                0
            } else {
                (1i64 << (attempts - 3).min(20)).min(MAX_BACKOFF_SECS)
            };
            let retry_after = (now + Duration::seconds(delay)).to_rfc3339();

            conn.execute(
                "INSERT INTO rate_limit (bucket, attempts, first_at, last_at, retry_after) \
                 VALUES (?1, ?2, ?3, ?3, ?4) \
                 ON CONFLICT(bucket) DO UPDATE SET \
                   attempts = ?2, last_at = ?3, retry_after = ?4",
                rusqlite::params![bucket, attempts, now.to_rfc3339(), retry_after],
            )?;
            Ok(())
        })
    }

    /// Clear a bucket after a success.
    pub fn clear_rate_limit(&self, bucket: &str) -> Result<()> {
        self.with_conn(|conn| {
            conn.execute("DELETE FROM rate_limit WHERE bucket = ?1", [bucket])?;
            Ok(())
        })
    }

    // =========================================================================
    // Audit
    // =========================================================================

    /// Append an audit record. Callers must never pass secret material.
    pub fn audit(&self, event: &str, detail: Option<&str>, succeeded: bool) -> Result<()> {
        self.with_conn(|conn| {
            conn.execute(
                "INSERT INTO audit_log (at, event, subject, detail, succeeded) \
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![
                    Utc::now().to_rfc3339(),
                    event,
                    ADMIN_SUBJECT,
                    detail,
                    i64::from(succeeded),
                ],
            )?;
            Ok(())
        })
    }

    /// Recent audit records, newest first, as `(at, event, detail, succeeded)`.
    pub fn recent_audit(&self, limit: u32) -> Result<Vec<(String, String, Option<String>, bool)>> {
        self.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT at, event, detail, succeeded FROM audit_log \
                 ORDER BY id DESC LIMIT ?1",
            )?;
            let rows = stmt
                .query_map([limit], |r| {
                    Ok((
                        r.get::<_, String>(0)?,
                        r.get::<_, String>(1)?,
                        r.get::<_, Option<String>>(2)?,
                        r.get::<_, i64>(3)? != 0,
                    ))
                })?
                .collect::<std::result::Result<Vec<_>, _>>()?;
            Ok(rows)
        })
    }
}

fn encode_scopes(scopes: &[Scope]) -> String {
    scopes
        .iter()
        .map(|s| s.as_str())
        .collect::<Vec<_>>()
        .join(" ")
}

fn decode_scopes(raw: &str) -> Vec<Scope> {
    raw.split_whitespace()
        .filter_map(|s| s.parse::<Scope>().ok())
        .collect()
}

fn parse_time(raw: &str) -> Result<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(raw)
        .map(|t| t.with_timezone(&Utc))
        .with_context(|| format!("parsing timestamp {raw:?}"))
}

fn parse_opt_time(raw: Option<&str>) -> Result<Option<DateTime<Utc>>> {
    raw.map(parse_time).transpose()
}

#[cfg(test)]
mod tests {
    use super::*;

    const GOOD_PASSWORD: &str = "correct horse battery staple";

    fn store() -> AuthStore {
        AuthStore::in_memory().unwrap()
    }

    // --- bootstrap ----------------------------------------------------------

    #[test]
    fn test_bootstrap_state_progresses() {
        let s = store();
        assert_eq!(s.bootstrap_state().unwrap(), BootstrapState::Uninitialized);

        s.create_admin(GOOD_PASSWORD, true).unwrap();
        assert_eq!(
            s.bootstrap_state().unwrap(),
            BootstrapState::AwaitingPassword
        );

        s.set_admin_password("a different long password").unwrap();
        assert_eq!(s.bootstrap_state().unwrap(), BootstrapState::Complete);
    }

    #[test]
    fn test_only_one_admin_can_ever_be_created() {
        // The security property behind the bootstrap race: whoever gets there
        // first wins, and every other attempt is refused rather than creating
        // a second account.
        let s = store();
        assert!(s.create_admin(GOOD_PASSWORD, false).unwrap());
        assert!(!s.create_admin("some other long password", false).unwrap());

        // The first password still works; the loser did not overwrite it.
        assert!(s.verify_admin_password(GOOD_PASSWORD).unwrap());
        assert!(!s.verify_admin_password("some other long password").unwrap());
    }

    #[test]
    fn test_concurrent_bootstrap_yields_exactly_one_winner() {
        let s = store();
        let wins: usize = std::thread::scope(|scope| {
            let handles: Vec<_> = (0..8)
                .map(|i| {
                    let s = s.clone();
                    scope.spawn(move || {
                        s.create_admin(&format!("password number {i:02} long"), false)
                            .unwrap_or(false)
                    })
                })
                .collect();
            handles
                .into_iter()
                .filter_map(|h| h.join().ok())
                .filter(|won| *won)
                .count()
        });
        assert_eq!(wins, 1, "exactly one concurrent bootstrap must succeed");
    }

    #[test]
    fn test_verify_password_on_an_uninitialized_store_is_false_not_an_error() {
        let s = store();
        assert!(!s.verify_admin_password(GOOD_PASSWORD).unwrap());
    }

    #[test]
    fn test_short_password_is_refused() {
        let s = store();
        assert!(s.create_admin("short", false).is_err());
        assert_eq!(s.bootstrap_state().unwrap(), BootstrapState::Uninitialized);
    }

    // --- signing key --------------------------------------------------------

    #[test]
    fn test_signing_key_is_generated_once_and_then_reused() {
        // Regenerating on every open would invalidate every issued token on
        // each restart.
        let s = store();
        let first = s.load_or_create_signing_key().unwrap();
        let second = s.load_or_create_signing_key().unwrap();
        assert_eq!(first.kid, second.kid);
        assert_eq!(first.public_raw(), second.public_raw());
    }

    // --- sessions -----------------------------------------------------------

    #[test]
    fn test_session_round_trip() {
        let s = store();
        let (token, _csrf, _expires) = s.create_session().unwrap();
        let principal = s
            .authenticate_session(&token)
            .unwrap()
            .expect("valid session");
        assert_eq!(principal.subject, ADMIN_SUBJECT);
        assert_eq!(principal.kind, PrincipalKind::Session);
        assert!(principal.session_id.is_some());
    }

    #[test]
    fn test_unknown_or_revoked_session_does_not_authenticate() {
        let s = store();
        assert!(s.authenticate_session("nonsense").unwrap().is_none());

        let (token, _, _) = s.create_session().unwrap();
        let id = s
            .authenticate_session(&token)
            .unwrap()
            .unwrap()
            .session_id
            .unwrap();
        s.revoke_session(&id).unwrap();
        assert!(
            s.authenticate_session(&token).unwrap().is_none(),
            "a revoked session must stop authenticating immediately"
        );
    }

    #[test]
    fn test_session_token_is_not_recoverable_from_the_database() {
        let s = store();
        let (token, csrf, _) = s.create_session().unwrap();
        let stored: Vec<(String, String)> = s
            .with_conn(|conn| {
                let mut stmt = conn.prepare("SELECT token_hash, csrf_hash FROM session")?;
                let rows = stmt
                    .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
                    .collect::<std::result::Result<Vec<_>, _>>()?;
                Ok(rows)
            })
            .unwrap();
        for (token_hash, csrf_hash) in stored {
            assert_ne!(token_hash, token);
            assert_ne!(csrf_hash, csrf);
        }
    }

    #[test]
    fn test_csrf_token_is_bound_to_its_own_session() {
        let s = store();
        let (token_a, csrf_a, _) = s.create_session().unwrap();
        let (_token_b, csrf_b, _) = s.create_session().unwrap();
        let id_a = s
            .authenticate_session(&token_a)
            .unwrap()
            .unwrap()
            .session_id
            .unwrap();

        assert!(s.verify_csrf(&id_a, &csrf_a).unwrap());
        assert!(
            !s.verify_csrf(&id_a, &csrf_b).unwrap(),
            "another session's CSRF token must not satisfy this one"
        );
    }

    #[test]
    fn test_rotating_csrf_invalidates_the_previous_token() {
        let s = store();
        let (token, first_csrf, _) = s.create_session().unwrap();
        let id = s
            .authenticate_session(&token)
            .unwrap()
            .unwrap()
            .session_id
            .unwrap();

        let second_csrf = s.rotate_csrf(&id).unwrap().expect("session exists");
        assert_ne!(first_csrf, second_csrf);
        assert!(s.verify_csrf(&id, &second_csrf).unwrap());
        assert!(
            !s.verify_csrf(&id, &first_csrf).unwrap(),
            "the superseded CSRF token must stop working"
        );
    }

    #[test]
    fn test_rotating_csrf_on_an_unknown_session_returns_none() {
        let s = store();
        assert!(s.rotate_csrf("no-such-session").unwrap().is_none());
    }

    #[test]
    fn test_list_sessions_marks_the_current_one() {
        let s = store();
        let (token, _, _) = s.create_session().unwrap();
        let id = s
            .authenticate_session(&token)
            .unwrap()
            .unwrap()
            .session_id
            .unwrap();
        s.create_session().unwrap();

        let listed = s.list_sessions(Some(&id)).unwrap();
        assert_eq!(listed.len(), 2);
        assert_eq!(listed.iter().filter(|x| x.current).count(), 1);
    }

    // --- refresh tokens -----------------------------------------------------

    #[test]
    fn test_refresh_token_rotates_and_the_old_one_dies() {
        let s = store();
        let first = s
            .create_refresh_family("vscode", &[Scope::Read, Scope::Write])
            .unwrap();

        let RefreshOutcome::Rotated {
            refresh_token: second,
            scopes,
        } = s.redeem_refresh_token(&first, "vscode").unwrap()
        else {
            panic!("first redemption should rotate");
        };
        assert_ne!(first, second);
        assert_eq!(scopes, vec![Scope::Read, Scope::Write]);

        // The replacement works.
        assert!(matches!(
            s.redeem_refresh_token(&second, "vscode").unwrap(),
            RefreshOutcome::Rotated { .. }
        ));
    }

    #[test]
    fn test_reusing_a_consumed_refresh_token_revokes_the_whole_family() {
        // Two parties hold the same token and we cannot tell which is calling,
        // so both are cut off rather than serving a possible thief.
        let s = store();
        let first = s.create_refresh_family("vscode", &[Scope::Read]).unwrap();
        let RefreshOutcome::Rotated {
            refresh_token: second,
            ..
        } = s.redeem_refresh_token(&first, "vscode").unwrap()
        else {
            panic!("expected rotation");
        };

        assert!(matches!(
            s.redeem_refresh_token(&first, "vscode").unwrap(),
            RefreshOutcome::Reused
        ));

        // The legitimate holder's current token is now dead too.
        assert!(
            matches!(
                s.redeem_refresh_token(&second, "vscode").unwrap(),
                RefreshOutcome::Invalid
            ),
            "reuse must revoke the family, not just the replayed token"
        );
    }

    #[test]
    fn test_unknown_refresh_token_is_invalid() {
        let s = store();
        assert!(matches!(
            s.redeem_refresh_token("nope", "vscode").unwrap(),
            RefreshOutcome::Invalid
        ));
    }

    #[test]
    fn test_refresh_token_is_bound_to_its_client() {
        let s = store();
        let token = s.create_refresh_family("vscode", &[Scope::Read]).unwrap();

        assert!(matches!(
            s.redeem_refresh_token(&token, "other-client").unwrap(),
            RefreshOutcome::Invalid
        ));
        assert!(matches!(
            s.redeem_refresh_token(&token, "vscode").unwrap(),
            RefreshOutcome::Rotated { .. }
        ));
    }

    #[test]
    fn test_rotation_does_not_extend_the_absolute_deadline() {
        // The idle clock resets on use; the absolute one is fixed at issuance,
        // so a continuously refreshed client still re-authenticates eventually.
        let s = store();
        let token = s.create_refresh_family("vscode", &[Scope::Read]).unwrap();
        let before: String = s
            .with_conn(|conn| {
                Ok(
                    conn.query_row("SELECT absolute_expires_at FROM refresh_family", [], |r| {
                        r.get(0)
                    })?,
                )
            })
            .unwrap();

        s.redeem_refresh_token(&token, "vscode").unwrap();

        let after: String = s
            .with_conn(|conn| {
                Ok(
                    conn.query_row("SELECT absolute_expires_at FROM refresh_family", [], |r| {
                        r.get(0)
                    })?,
                )
            })
            .unwrap();
        assert_eq!(before, after);
    }

    // --- device flow --------------------------------------------------------

    #[test]
    fn test_device_flow_pending_then_approved_then_consumed() {
        let s = store();
        let (device_code, user_code) = s
            .create_device_authorization("vscode", &Scope::ALL)
            .unwrap();

        assert!(matches!(
            s.poll_device(&device_code, "vscode").unwrap(),
            DevicePollOutcome::Pending
        ));

        let (client_id, scopes) = s.approve_device(&user_code).unwrap().expect("approved");
        assert_eq!(client_id, "vscode");
        assert_eq!(scopes, Scope::ALL.to_vec());

        // Poll again immediately: the interval is enforced server-side.
        assert!(matches!(
            s.poll_device(&device_code, "vscode").unwrap(),
            DevicePollOutcome::SlowDown
        ));
    }

    #[test]
    fn test_a_device_code_cannot_be_redeemed_twice() {
        let s = store();
        let (device_code, user_code) = s
            .create_device_authorization("vscode", &[Scope::Read])
            .unwrap();
        s.approve_device(&user_code).unwrap().unwrap();

        // Backdate the poll clock so the interval check does not mask this.
        s.with_conn(|conn| {
            conn.execute("UPDATE device_authorization SET last_polled_at = NULL", [])?;
            Ok(())
        })
        .unwrap();

        assert!(matches!(
            s.poll_device(&device_code, "vscode").unwrap(),
            DevicePollOutcome::Approved { .. }
        ));
        assert!(
            matches!(
                s.poll_device(&device_code, "vscode").unwrap(),
                DevicePollOutcome::Expired
            ),
            "a device code must be single-use"
        );
    }

    #[test]
    fn test_approving_an_unknown_user_code_returns_none() {
        let s = store();
        assert!(s.approve_device("ZZZZ-ZZZZ").unwrap().is_none());
    }

    #[test]
    fn test_device_code_is_bound_to_its_client() {
        let s = store();
        let (device_code, _) = s
            .create_device_authorization("vscode", &[Scope::Read])
            .unwrap();

        assert!(matches!(
            s.poll_device(&device_code, "other-client").unwrap(),
            DevicePollOutcome::Expired
        ));
        assert!(matches!(
            s.poll_device(&device_code, "vscode").unwrap(),
            DevicePollOutcome::Pending
        ));
    }

    #[test]
    fn test_a_device_cannot_be_approved_twice() {
        let s = store();
        let (_dc, user_code) = s
            .create_device_authorization("vscode", &[Scope::Read])
            .unwrap();
        assert!(s.approve_device(&user_code).unwrap().is_some());
        assert!(
            s.approve_device(&user_code).unwrap().is_none(),
            "re-approving must not silently succeed"
        );
    }

    // --- access keys --------------------------------------------------------

    #[test]
    fn test_access_key_round_trip() {
        let s = store();
        let (summary, secret) = s
            .create_access_key("ci", &[Scope::Read, Scope::Execute], 30)
            .unwrap();
        assert!(secret.starts_with("opk_"));
        assert_eq!(summary.scopes, vec![Scope::Read, Scope::Execute]);

        let scopes = s.redeem_access_key(&secret).unwrap().expect("valid key");
        assert_eq!(scopes, vec![Scope::Read, Scope::Execute]);
    }

    #[test]
    fn test_access_key_secret_is_not_recoverable() {
        let s = store();
        let (_summary, secret) = s.create_access_key("ci", &[Scope::Read], 30).unwrap();
        let listed = s.list_access_keys().unwrap();
        let rendered = serde_json::to_string(&listed).unwrap();
        assert!(
            !rendered.contains(&secret),
            "listing keys must never expose the secret"
        );
    }

    #[test]
    fn test_revoked_access_key_stops_working() {
        let s = store();
        let (summary, secret) = s.create_access_key("ci", &[Scope::Read], 30).unwrap();
        assert!(s.redeem_access_key(&secret).unwrap().is_some());

        assert!(s.revoke_access_key(&summary.id).unwrap().is_some());
        assert!(s.redeem_access_key(&secret).unwrap().is_none());
        // Revoking twice is not an error, but reports no second revocation.
        assert!(s.revoke_access_key(&summary.id).unwrap().is_none());
    }

    #[test]
    fn test_access_key_records_last_use() {
        let s = store();
        let (summary, secret) = s.create_access_key("ci", &[Scope::Read], 30).unwrap();
        assert!(s.list_access_keys().unwrap()[0].last_used_at.is_none());

        s.redeem_access_key(&secret).unwrap();
        let listed = s.list_access_keys().unwrap();
        assert_eq!(listed[0].id, summary.id);
        assert!(
            listed[0].last_used_at.is_some(),
            "last use must be tracked so an unused key is visible"
        );
    }

    #[test]
    fn test_access_key_expiry_is_mandatory_and_scopes_required() {
        let s = store();
        assert!(s.create_access_key("ci", &[Scope::Read], 0).is_err());
        assert!(s.create_access_key("ci", &[], 30).is_err());
    }

    // --- revoke everything --------------------------------------------------

    #[test]
    fn test_revoke_all_credentials_cuts_off_every_kind_at_once() {
        let s = store();
        let (session, _, _) = s.create_session().unwrap();
        let refresh = s.create_refresh_family("vscode", &[Scope::Read]).unwrap();
        let (_summary, key) = s.create_access_key("ci", &[Scope::Read], 30).unwrap();

        s.revoke_all_credentials("password reset").unwrap();

        assert!(s.authenticate_session(&session).unwrap().is_none());
        assert!(matches!(
            s.redeem_refresh_token(&refresh, "vscode").unwrap(),
            RefreshOutcome::Invalid
        ));
        assert!(s.redeem_access_key(&key).unwrap().is_none());
    }

    // --- rate limiting ------------------------------------------------------

    #[test]
    fn test_backoff_engages_only_after_a_few_failures() {
        let s = store();
        for _ in 0..3 {
            assert_eq!(
                s.check_rate_limit("login").unwrap(),
                RateLimitDecision::Allow
            );
            s.record_failure("login").unwrap();
        }
        // A typo or two should not cost the operator anything.
        assert_eq!(
            s.check_rate_limit("login").unwrap(),
            RateLimitDecision::Allow
        );

        s.record_failure("login").unwrap();
        assert!(matches!(
            s.check_rate_limit("login").unwrap(),
            RateLimitDecision::Backoff { .. }
        ));
    }

    #[test]
    fn test_backoff_is_capped_and_never_becomes_a_lockout() {
        // On a single-account system a permanent lockout would be a
        // denial-of-service against the only person who could undo it.
        let s = store();
        for _ in 0..64 {
            s.record_failure("login").unwrap();
        }
        match s.check_rate_limit("login").unwrap() {
            RateLimitDecision::Backoff { retry_after_secs } => {
                assert!(
                    retry_after_secs <= 15 * 60,
                    "backoff must stay bounded, got {retry_after_secs}s"
                );
            }
            RateLimitDecision::Allow => panic!("expected backoff after many failures"),
        }
    }

    #[test]
    fn test_success_clears_the_backoff() {
        let s = store();
        for _ in 0..8 {
            s.record_failure("login").unwrap();
        }
        s.clear_rate_limit("login").unwrap();
        assert_eq!(
            s.check_rate_limit("login").unwrap(),
            RateLimitDecision::Allow
        );
    }

    #[test]
    fn test_rate_limit_buckets_are_independent() {
        let s = store();
        for _ in 0..8 {
            s.record_failure("login").unwrap();
        }
        assert_eq!(
            s.check_rate_limit("bootstrap").unwrap(),
            RateLimitDecision::Allow,
            "failing to log in must not throttle bootstrap"
        );
    }

    // --- audit --------------------------------------------------------------

    #[test]
    fn test_audit_records_are_appended_newest_first() {
        let s = store();
        s.audit("login", Some("failure"), false).unwrap();
        s.audit("login", None, true).unwrap();

        let recent = s.recent_audit(10).unwrap();
        assert_eq!(recent.len(), 2);
        assert!(recent[0].3, "newest record should be the successful login");
        assert_eq!(recent[1].2.as_deref(), Some("failure"));
    }
}

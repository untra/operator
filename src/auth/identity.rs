use std::collections::BTreeSet;

use anyhow::{ensure, Context, Result};
use chrono::{DateTime, Utc};
use rusqlite::{params, OptionalExtension};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;
use uuid::Uuid;

use super::audit::AuditAction;
use super::scope::Principal;
use super::secret::{generate_secret, hash_secret};
use super::store::AuthStore;
use crate::rest::dto::auth::{AccessKeySummary, PrincipalKind};

pub const KEY_PREFIX: &str = "opk_v1";
pub const KEY_LENGTH: usize = 87;
pub const KEY_PAGE_SIZE: usize = 50;
pub const RECENT_LOGIN_SECONDS: i64 = 300;
pub const ISSUANCE_WINDOW_SECONDS: u32 = 3600;
const MAX_ISSUED_PER_WINDOW: u32 = 20;

#[derive(Debug, thiserror::Error)]
#[error("key issuance limit reached")]
pub struct IssuanceLimited;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
#[serde(rename_all = "snake_case")]
pub enum OidcProvider {
    Keycloak,
    Authentik,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct OidcConnection {
    pub id: String,
    pub provider: OidcProvider,
    pub issuer: String,
    pub client_id: String,
    pub redirect_uri: String,
    pub disabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct CreateOidcConnection {
    pub provider: OidcProvider,
    pub issuer: String,
    pub client_id: String,
    pub redirect_uri: String,
    pub client_secret_file: Option<String>,
}

#[derive(
    Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, ToSchema, JsonSchema, TS,
)]
#[ts(export)]
pub struct PermissionGrant {
    pub profile_id: Uuid,
    #[serde(default)]
    pub instance_wide: bool,
    pub capabilities: Vec<String>,
    pub projects: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct OperatorUser {
    pub id: String,
    pub connection_id: String,
    pub issuer: String,
    pub subject: String,
    pub enabled: bool,
    pub administrator: bool,
    pub grants: Vec<PermissionGrant>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct UpdateOperatorUser {
    pub enabled: bool,
    pub administrator: bool,
    pub grants: PermissionGrant,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct CreateUserKey {
    pub name: String,
    pub expires_in_days: u32,
    pub grants: PermissionGrant,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct UserKey {
    #[serde(flatten)]
    pub summary: AccessKeySummary,
    pub owner_id: String,
    pub profile_id: Uuid,
    pub display_prefix: String,
    pub grants: PermissionGrant,
    pub replaces_id: Option<String>,
}

#[derive(Serialize, ToSchema, TS)]
#[ts(export)]
pub struct IssuedUserKey {
    pub key: UserKey,
    pub secret: String,
}

#[derive(Debug, Clone)]
pub struct UserAuthority {
    pub user: OperatorUser,
    pub key_id: Option<String>,
    pub grants: PermissionGrant,
    pub authenticated_at: Option<i64>,
}

pub fn capabilities() -> Vec<String> {
    super::policy::capabilities()
}

pub fn validate_grants(grants: &PermissionGrant) -> Result<()> {
    let known = capabilities();
    ensure!(
        grants.capabilities.iter().all(|c| known.contains(c)),
        "unknown capability"
    );
    ensure!(
        grants.capabilities.iter().collect::<BTreeSet<_>>().len() == grants.capabilities.len(),
        "duplicate capability"
    );
    ensure!(
        grants.projects.iter().all(|p| !p.is_empty()
            && p.len() <= 128
            && p != "*"
            && !p.contains('/')
            && !p.contains('\\')),
        "invalid project"
    );
    ensure!(
        grants.projects.iter().collect::<BTreeSet<_>>().len() == grants.projects.len(),
        "duplicate project"
    );
    Ok(())
}

fn strings(
    conn: &rusqlite::Connection,
    sql: &str,
    parameters: impl rusqlite::Params,
) -> Result<Vec<String>> {
    Ok(conn
        .prepare(sql)?
        .query_map(parameters, |r| r.get(0))?
        .collect::<rusqlite::Result<_>>()?)
}

fn read_grants(
    conn: &rusqlite::Connection,
    id: &str,
    profile: Uuid,
    key: bool,
) -> Result<PermissionGrant> {
    let profile_id = profile.to_string();
    if key {
        Ok(PermissionGrant {
            profile_id: profile,
            instance_wide: conn.query_row(
                "SELECT instance_operations FROM access_key WHERE id = ?1 AND profile_id = ?2",
                params![id, profile_id],
                |r| r.get(0),
            )?,
            capabilities: strings(
                conn,
                "SELECT capability FROM key_capability WHERE key_id = ?1 ORDER BY capability",
                [id],
            )?,
            projects: strings(
                conn,
                "SELECT project FROM key_project WHERE key_id = ?1 ORDER BY project",
                [id],
            )?,
        })
    } else {
        Ok(PermissionGrant {
            profile_id: profile,
            instance_wide: conn.query_row("SELECT instance_operations FROM user_profile WHERE user_id = ?1 AND profile_id = ?2", params![id, profile_id], |r| r.get(0)).optional()?.unwrap_or(false),
            capabilities: strings(conn, "SELECT capability FROM user_grant WHERE user_id = ?1 AND profile_id = ?2 ORDER BY capability", params![id, profile_id])?,
            projects: strings(conn, "SELECT project FROM user_project WHERE user_id = ?1 AND profile_id = ?2 ORDER BY project", params![id, profile_id])?,
        })
    }
}

fn write_grants(
    conn: &rusqlite::Connection,
    id: &str,
    grants: &PermissionGrant,
    key: bool,
) -> Result<()> {
    if key {
        for capability in &grants.capabilities {
            conn.execute(
                "INSERT INTO key_capability VALUES (?1, ?2)",
                params![id, capability],
            )?;
        }
        for project in &grants.projects {
            conn.execute(
                "INSERT INTO key_project VALUES (?1, ?2)",
                params![id, project],
            )?;
        }
    } else {
        let profile = grants.profile_id.to_string();
        conn.execute("INSERT INTO user_profile VALUES (?1, ?2, ?3) ON CONFLICT(user_id, profile_id) DO UPDATE SET instance_operations = excluded.instance_operations", params![id, profile, grants.instance_wide])?;
        conn.execute(
            "DELETE FROM user_grant WHERE user_id = ?1 AND profile_id = ?2",
            params![id, profile],
        )?;
        conn.execute(
            "DELETE FROM user_project WHERE user_id = ?1 AND profile_id = ?2",
            params![id, profile],
        )?;
        for capability in &grants.capabilities {
            conn.execute(
                "INSERT INTO user_grant VALUES (?1, ?2, ?3)",
                params![id, profile, capability],
            )?;
        }
        for project in &grants.projects {
            conn.execute(
                "INSERT INTO user_project VALUES (?1, ?2, ?3)",
                params![id, profile, project],
            )?;
        }
    }
    Ok(())
}

fn read_user(conn: &rusqlite::Connection, id: &str) -> Result<Option<OperatorUser>> {
    let user = conn.query_row("SELECT id, connection_id, issuer, subject, enabled, administrator FROM operator_user WHERE id = ?1", [id], |r| Ok(OperatorUser {
        id: r.get(0)?, connection_id: r.get(1)?, issuer: r.get(2)?, subject: r.get(3)?, enabled: r.get(4)?, administrator: r.get(5)?, grants: Vec::new(),
    })).optional()?;
    user.map(|mut user| {
        for profile in strings(
            conn,
            "SELECT profile_id FROM user_profile WHERE user_id = ?1 ORDER BY profile_id",
            [id],
        )? {
            user.grants
                .push(read_grants(conn, id, Uuid::parse_str(&profile)?, false)?);
        }
        Ok(user)
    })
    .transpose()
}

fn authority(
    conn: &rusqlite::Connection,
    user_id: &str,
    profile: Uuid,
) -> Result<Option<UserAuthority>> {
    let Some(user) = read_user(conn, user_id)? else {
        return Ok(None);
    };
    let disabled: bool = conn.query_row(
        "SELECT disabled FROM oidc_connection WHERE id = ?1",
        [&user.connection_id],
        |r| r.get(0),
    )?;
    if disabled || !user.enabled {
        return Ok(None);
    }
    let grants = read_grants(conn, user_id, profile, false)?;
    Ok(Some(UserAuthority {
        user,
        key_id: None,
        grants,
        authenticated_at: None,
    }))
}

fn record_audit(
    conn: &rusqlite::Connection,
    actor: &str,
    action: AuditAction,
    target: &str,
) -> Result<()> {
    conn.execute(
        "INSERT INTO audit_log (at, event, subject, detail, succeeded) VALUES (?1, ?2, ?3, ?4, 1)",
        params![Utc::now().to_rfc3339(), action.as_str(), actor, target],
    )?;
    Ok(())
}

fn export_audit(actor: &str, action: AuditAction, target: &str) {
    let detail = serde_json::json!({"actor_id": actor, "target_id": target}).to_string();
    crate::telemetry::events::audit_event(action, true, Some(&detail));
}

impl AuthStore {
    pub fn oidc_connections(&self) -> Result<Vec<OidcConnection>> {
        self.with_conn(|conn| Ok(conn.prepare("SELECT id, provider, issuer, client_id, redirect_uri, disabled FROM oidc_connection ORDER BY created_at")?.query_map([], |r| {
            let provider: String = r.get(1)?;
            Ok(OidcConnection { id: r.get(0)?, provider: if provider == "keycloak" { OidcProvider::Keycloak } else { OidcProvider::Authentik }, issuer: r.get(2)?, client_id: r.get(3)?, redirect_uri: r.get(4)?, disabled: r.get(5)? })
        })?.collect::<rusqlite::Result<_>>()?))
    }

    pub fn add_oidc_connection(
        &self,
        actor: &str,
        request: &CreateOidcConnection,
    ) -> Result<String> {
        let id = Uuid::new_v4().to_string();
        self.with_conn(|conn| {
            let tx = conn.transaction()?;
            let provider = match request.provider { OidcProvider::Keycloak => "keycloak", OidcProvider::Authentik => "authentik" };
            tx.execute("INSERT INTO oidc_connection (id, provider, issuer, client_id, client_secret_file, redirect_uri, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)", params![id, provider, request.issuer, request.client_id, request.client_secret_file, request.redirect_uri, Utc::now().timestamp()])?;
            record_audit(&tx, actor, AuditAction::OidcConnectionCreated, &id)?;
            tx.commit()?;
            Ok(())
        })?;
        export_audit(actor, AuditAction::OidcConnectionCreated, &id);
        Ok(id)
    }

    pub fn oidc_secret_file(&self, id: &str) -> Result<Option<String>> {
        self.with_conn(|conn| {
            Ok(conn.query_row(
                "SELECT client_secret_file FROM oidc_connection WHERE id = ?1 AND disabled = 0",
                [id],
                |r| r.get(0),
            )?)
        })
    }

    pub fn disable_oidc_connection(&self, actor: &str, id: &str) -> Result<()> {
        self.with_conn(|conn| {
            let tx = conn.transaction()?;
            ensure!(
                tx.execute(
                    "UPDATE oidc_connection SET disabled = 1 WHERE id = ?1",
                    [id]
                )? == 1,
                "unknown connection"
            );
            record_audit(&tx, actor, AuditAction::OidcConnectionDisabled, id)?;
            tx.commit()?;
            Ok(())
        })?;
        export_audit(actor, AuditAction::OidcConnectionDisabled, id);
        Ok(())
    }

    pub fn provision_oidc_user(
        &self,
        connection: &OidcConnection,
        subject: &str,
    ) -> Result<String> {
        self.with_conn(|conn| {
            let tx = conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
            tx.execute("INSERT INTO operator_user (id, connection_id, issuer, subject, created_at) VALUES (?1, ?2, ?3, ?4, ?5) ON CONFLICT(issuer, subject) DO NOTHING", params![Uuid::new_v4().to_string(), connection.id, connection.issuer, subject, Utc::now().timestamp()])?;
            let id = tx.query_row("SELECT id FROM operator_user WHERE issuer = ?1 AND subject = ?2 AND connection_id = ?3", params![connection.issuer, subject, connection.id], |r| r.get(0))?;
            tx.commit()?;
            Ok(id)
        })
    }

    pub fn operator_users(&self, offset: u32) -> Result<Vec<OperatorUser>> {
        self.with_conn(|conn| {
            strings(
                conn,
                "SELECT id FROM operator_user ORDER BY created_at, id LIMIT ?1 OFFSET ?2",
                params![KEY_PAGE_SIZE, offset],
            )?
            .iter()
            .map(|id| read_user(conn, id)?.ok_or_else(|| anyhow::anyhow!("unknown user")))
            .collect()
        })
    }

    pub fn update_operator_user(
        &self,
        actor: &str,
        id: &str,
        update: &UpdateOperatorUser,
        premium: bool,
    ) -> Result<()> {
        validate_grants(&update.grants)?;
        self.with_conn(|conn| {
            let tx = conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
            let current = read_user(&tx, id)?.ok_or_else(|| anyhow::anyhow!("unknown user"))?;
            let old = read_grants(&tx, id, update.grants.profile_id, false)?;
            let expands = (!current.enabled && update.enabled)
                || (!current.administrator && update.administrator)
                || (!old.instance_wide && update.grants.instance_wide)
                || update
                    .grants
                    .capabilities
                    .iter()
                    .any(|c| !old.capabilities.contains(c))
                || update
                    .grants
                    .projects
                    .iter()
                    .any(|p| !old.projects.contains(p));
            ensure!(!expands || premium, "permission expansion requires premium");
            tx.execute(
                "UPDATE operator_user SET enabled = ?1, administrator = ?2 WHERE id = ?3",
                params![update.enabled, update.administrator, id],
            )?;
            write_grants(&tx, id, &update.grants, false)?;
            record_audit(&tx, actor, AuditAction::UserPermissionsChanged, id)?;
            tx.commit()?;
            Ok(())
        })?;
        export_audit(actor, AuditAction::UserPermissionsChanged, id);
        Ok(())
    }

    /// Authority of an already provisioned user, never provisioning one.
    pub fn subject_authority(
        &self,
        connection: &OidcConnection,
        subject: &str,
        profile: Uuid,
    ) -> Result<Option<UserAuthority>> {
        self.with_conn(|conn| {
            let id: Option<String> = conn.query_row("SELECT id FROM operator_user WHERE issuer = ?1 AND subject = ?2 AND connection_id = ?3", params![connection.issuer, subject, connection.id], |r| r.get(0)).optional()?;
            id.map_or(Ok(None), |id| authority(conn, &id, profile))
        })
    }

    pub fn user_authority(&self, id: &str, profile: Uuid) -> Result<Option<UserAuthority>> {
        self.with_conn(|conn| authority(conn, id, profile))
    }

    pub fn oidc_session(
        &self,
        user_id: &str,
        authenticated_at: i64,
    ) -> Result<(String, String, DateTime<Utc>)> {
        let token = generate_secret()?;
        let csrf = generate_secret()?;
        let now = Utc::now();
        let expires = now + chrono::Duration::hours(12);
        self.with_conn(|conn| {
            let tx = conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
            ensure!(authority(&tx, user_id, Uuid::nil())?.is_some(), "account requires administrator approval");
            tx.execute("INSERT INTO session (id, token_hash, csrf_hash, created_at, expires_at, user_id, authenticated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)", params![Uuid::new_v4().to_string(), hash_secret(&token), hash_secret(&csrf), now.to_rfc3339(), expires.to_rfc3339(), user_id, authenticated_at])?;
            record_audit(&tx, user_id, AuditAction::OidcLogin, user_id)?;
            tx.commit()?;
            Ok(())
        })?;
        export_audit(user_id, AuditAction::OidcLogin, user_id);
        Ok((token, csrf, expires))
    }

    pub fn resolve_user_session(
        &self,
        principal: Principal,
        profile: Uuid,
    ) -> Result<Option<Principal>> {
        let Some(id) = principal.session_id.clone() else {
            return Ok(Some(principal));
        };
        self.with_conn(|conn| {
            let tx = conn.transaction()?;
            let (user_id, authenticated_at): (Option<String>, Option<i64>) = tx.query_row("SELECT user_id, authenticated_at FROM session WHERE id = ?1 AND revoked_at IS NULL", [id], |r| Ok((r.get(0)?, r.get(1)?)))?;
            let Some(user_id) = user_id else { return Ok(Some(principal)) };
            let Some(mut auth) = authority(&tx, &user_id, profile)? else { return Ok(None) };
            auth.authenticated_at = authenticated_at;
            Ok(Some(auth.principal(principal.expires_at.context("missing session expiry")?, principal.session_id)))
        })
    }

    pub fn create_user_key(
        &self,
        user: &UserAuthority,
        request: &CreateUserKey,
        replaces: Option<&str>,
    ) -> Result<IssuedUserKey> {
        validate_grants(&request.grants)?;
        ensure!(
            !request.name.trim().is_empty()
                && request.name.len() <= crate::rest::dto::auth::MAX_IDENTIFIER_LENGTH,
            "invalid key name"
        );
        ensure!(
            (1..=crate::rest::dto::auth::MAX_ACCESS_KEY_EXPIRY_DAYS)
                .contains(&u64::from(request.expires_in_days)),
            "expiry must be 1 to 365 days"
        );
        ensure!(
            !request.grants.capabilities.is_empty(),
            "select at least one capability"
        );
        let id = Uuid::new_v4().to_string();
        let secret = format!("{KEY_PREFIX}_{id}_{}", generate_secret()?);
        let now = Utc::now();
        let expires = now + chrono::Duration::days(i64::from(request.expires_in_days));
        let action = if replaces.is_some() {
            AuditAction::AccessKeyRotated
        } else {
            AuditAction::AccessKeyCreated
        };
        let audit_detail = serde_json::json!({
            "actor_id": user.user.id, "owner_id": user.user.id, "key_id": id,
            "profile_id": request.grants.profile_id, "grants": request.grants,
            "replaces_id": replaces,
        })
        .to_string();
        self.with_conn(|conn| {
            let tx = conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
            let owner = authority(&tx, &user.user.id, request.grants.profile_id)?.context("account or connection disabled")?;
            tx.execute("INSERT INTO key_issuance (user_id, window_start, issued) VALUES (?1, ?2, 0) ON CONFLICT(user_id) DO UPDATE SET window_start = excluded.window_start, issued = 0 WHERE key_issuance.window_start <= ?2 - ?3", params![owner.user.id, now.timestamp(), ISSUANCE_WINDOW_SECONDS])?;
            if tx.execute("UPDATE key_issuance SET issued = issued + 1 WHERE user_id = ?1 AND issued < ?2", params![owner.user.id, MAX_ISSUED_PER_WINDOW])? != 1 { return Err(IssuanceLimited.into()); }
            ensure!(request.grants.is_subset_of(&owner.grants), "grant exceeds owner permissions");
            if let Some(old) = replaces {
                let grants = read_grants(&tx, old, request.grants.profile_id, true)?;
                ensure!(request.grants.is_subset_of(&grants), "rotation cannot expand permissions");
                ensure!(tx.execute("UPDATE access_key SET revoked_at = ?1 WHERE id = ?2 AND user_id = ?3 AND profile_id = ?4 AND revoked_at IS NULL AND julianday(expires_at) > julianday(?1)", params![now.to_rfc3339(), old, owner.user.id, request.grants.profile_id.to_string()])? == 1, "key is not active");
            }
            tx.execute("INSERT INTO access_key (id, name, key_hash, scopes, created_at, expires_at, user_id, profile_id, instance_operations, replaces_id) VALUES (?1, ?2, ?3, '', ?4, ?5, ?6, ?7, ?8, ?9)", params![id, request.name.trim(), hash_secret(&secret), now.to_rfc3339(), expires.to_rfc3339(), owner.user.id, request.grants.profile_id.to_string(), request.grants.instance_wide, replaces])?;
            write_grants(&tx, &id, &request.grants, true)?;
            record_audit(&tx, &owner.user.id, action, &audit_detail)?;
            tx.commit()?;
            Ok(())
        })?;
        crate::telemetry::events::audit_event(action, true, Some(&audit_detail));
        Ok(IssuedUserKey {
            key: UserKey {
                summary: AccessKeySummary {
                    id: id.clone(),
                    name: request.name.trim().to_string(),
                    scopes: Vec::new(),
                    created_at: now,
                    expires_at: expires,
                    last_used_at: None,
                    revoked_at: None,
                },
                owner_id: user.user.id.clone(),
                profile_id: request.grants.profile_id,
                display_prefix: format!("{KEY_PREFIX}_{id}_…"),
                grants: request.grants.clone(),
                replaces_id: replaces.map(str::to_owned),
            },
            secret,
        })
    }

    pub fn key_authority(&self, id: &str) -> Result<Option<(UserAuthority, DateTime<Utc>)>> {
        let result = self.with_conn(|conn| {
            let tx = conn.transaction()?;
            let row: Option<(String, String, String)> = tx.query_row("SELECT user_id, profile_id, expires_at FROM access_key WHERE id = ?1 AND user_id IS NOT NULL AND profile_id IS NOT NULL AND revoked_at IS NULL", [id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?))).optional()?;
            let Some((owner, profile, expiry)) = row else { return Ok(None) };
            let expires = DateTime::parse_from_rfc3339(&expiry)?.with_timezone(&Utc);
            if expires <= Utc::now() { return Ok(None); }
            let Some(mut auth) = authority(&tx, &owner, Uuid::parse_str(&profile)?)? else { return Ok(None) };
            let grants = read_grants(&tx, id, auth.grants.profile_id, true)?;
            auth.grants.instance_wide &= grants.instance_wide;
            auth.grants.capabilities.retain(|c| grants.capabilities.contains(c));
            auth.grants.projects.retain(|p| grants.projects.contains(p));
            auth.key_id = Some(id.to_string());
            Ok(Some((auth, expires)))
        })?;
        Ok(result)
    }

    pub fn record_key_use(&self, id: &str) {
        self.usage
            .record(super::usage::CredentialTable::Key, id, Utc::now());
    }

    pub fn exchange_user_key(
        &self,
        secret: &str,
    ) -> Result<Option<(UserAuthority, DateTime<Utc>)>> {
        if !secret.starts_with(&format!("{KEY_PREFIX}_")) || secret.len() != KEY_LENGTH {
            return Ok(None);
        }
        let id: Option<String> = self.with_conn(|conn| {
            Ok(conn
                .query_row(
                    "SELECT id FROM access_key WHERE key_hash = ?1",
                    [hash_secret(secret)],
                    |r| r.get(0),
                )
                .optional()?)
        })?;
        let result = id
            .map(|id| self.key_authority(&id))
            .transpose()
            .map(Option::flatten)?;
        if let Some((authority, _)) = result.as_ref() {
            let detail = serde_json::json!({
                "actor_id": authority.user.id,
                "owner_id": authority.user.id,
                "key_id": authority.key_id,
                "profile_id": authority.grants.profile_id,
                "reason": "credential_authenticated",
            })
            .to_string();
            self.with_conn(|conn| {
                record_audit(
                    conn,
                    &authority.user.id,
                    AuditAction::AccessKeyExchange,
                    &detail,
                )
            })?;
            crate::telemetry::events::audit_event(
                AuditAction::AccessKeyExchange,
                true,
                Some(&detail),
            );
            if let Some(id) = &authority.key_id {
                self.record_key_use(id);
            }
        }
        Ok(result)
    }

    pub fn revoke_user_key(&self, actor: &str, id: &str, administrator: bool) -> Result<bool> {
        let result = self.with_conn(|conn| {
            let tx = conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
            let owner: Option<(Option<String>, Option<String>)> = tx.query_row(
                "SELECT user_id, profile_id FROM access_key WHERE id = ?1 AND (user_id = ?2 OR ?3)",
                params![id, actor, administrator], |row| Ok((row.get(0)?, row.get(1)?)),
            ).optional()?;
            let Some((owner, profile)) = owner else {
                return Ok(None);
            };
            let changed = tx.execute(
                "UPDATE access_key SET revoked_at = ?1 WHERE id = ?2 AND revoked_at IS NULL",
                params![Utc::now().to_rfc3339(), id],
            )? > 0;
            let detail = serde_json::json!({
                "actor_id": actor, "owner_id": owner, "key_id": id, "profile_id": profile,
            })
            .to_string();
            if changed {
                record_audit(&tx, actor, AuditAction::AccessKeyRevoked, &detail)?;
            }
            tx.commit()?;
            Ok(Some((changed, detail)))
        })?;
        if let Some((true, detail)) = result.as_ref() {
            crate::telemetry::events::audit_event(
                AuditAction::AccessKeyRevoked,
                true,
                Some(detail),
            );
        }
        Ok(result.is_some())
    }

    pub fn user_keys(&self, owner: &str, profile: Uuid, offset: u32) -> Result<Vec<UserKey>> {
        self.with_conn(|conn| {
            let mut stmt = conn.prepare("SELECT id, name, created_at, expires_at, last_used_at, revoked_at, replaces_id FROM access_key WHERE user_id = ?1 AND profile_id = ?2 ORDER BY created_at DESC, id LIMIT ?3 OFFSET ?4")?;
            let rows = stmt.query_map(params![owner, profile.to_string(), KEY_PAGE_SIZE, offset], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?, r.get::<_, String>(3)?, r.get::<_, Option<String>>(4)?, r.get::<_, Option<String>>(5)?, r.get::<_, Option<String>>(6)?)))?.collect::<rusqlite::Result<Vec<_>>>()?;
            rows.into_iter().map(|(id, name, created, expires, used, revoked, replaces_id)| {
                let parse = |s: &str| -> Result<DateTime<Utc>> { Ok(DateTime::parse_from_rfc3339(s)?.with_timezone(&Utc)) };
                Ok(UserKey { display_prefix: format!("{KEY_PREFIX}_{id}_…"), owner_id: owner.to_string(), profile_id: profile, grants: read_grants(conn, &id, profile, true)?, replaces_id, summary: AccessKeySummary { id, name, scopes: Vec::new(), created_at: parse(&created)?, expires_at: parse(&expires)?, last_used_at: used.as_deref().map(parse).transpose()?, revoked_at: revoked.as_deref().map(parse).transpose()? } })
            }).collect()
        })
    }
}

impl PermissionGrant {
    pub fn is_subset_of(&self, other: &Self) -> bool {
        self.profile_id == other.profile_id
            && (!self.instance_wide || other.instance_wide)
            && self
                .capabilities
                .iter()
                .all(|c| other.capabilities.contains(c))
            && self.projects.iter().all(|p| other.projects.contains(p))
    }
}

impl UserAuthority {
    pub fn principal(&self, expires_at: DateTime<Utc>, session_id: Option<String>) -> Principal {
        Principal {
            subject: self.user.id.clone(),
            scopes: Vec::new(),
            kind: if session_id.is_some() {
                PrincipalKind::Session
            } else {
                PrincipalKind::ApiKey
            },
            expires_at: Some(expires_at),
            session_id,
            ticket_id: None,
            step: None,
            agent_session_id: None,
            authority: Some(self.clone()),
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    fn fixture() -> (AuthStore, UserAuthority, CreateUserKey) {
        fixture_in(AuthStore::in_memory().unwrap())
    }

    pub(crate) fn fixture_in(store: AuthStore) -> (AuthStore, UserAuthority, CreateUserKey) {
        store
            .add_oidc_connection(
                "admin",
                &CreateOidcConnection {
                    provider: OidcProvider::Keycloak,
                    issuer: "https://identity.example/realm".into(),
                    client_id: "operator".into(),
                    redirect_uri: "https://operator.example/api/v1/auth/oidc/callback".into(),
                    client_secret_file: None,
                },
            )
            .unwrap();
        let connection = store.oidc_connections().unwrap().remove(0);
        let id = store.provision_oidc_user(&connection, "subject").unwrap();
        let grants = PermissionGrant {
            profile_id: Uuid::new_v4(),
            instance_wide: false,
            capabilities: vec!["tickets:read".into()],
            projects: vec!["operator".into()],
        };
        assert!(store
            .user_authority(&id, grants.profile_id)
            .unwrap()
            .is_none());
        store
            .update_operator_user(
                "admin",
                &id,
                &UpdateOperatorUser {
                    enabled: true,
                    administrator: false,
                    grants: grants.clone(),
                },
                true,
            )
            .unwrap();
        let authority = store
            .user_authority(&id, grants.profile_id)
            .unwrap()
            .unwrap();
        (
            store,
            authority,
            CreateUserKey {
                name: "test".into(),
                expires_in_days: 90,
                grants,
            },
        )
    }

    #[test]
    fn test_subject_authority_resolves_only_provisioned_enabled_users() {
        let (store, owner, request) = fixture();
        let profile = request.grants.profile_id;
        let connection = store.oidc_connections().unwrap().remove(0);
        let found = store
            .subject_authority(&connection, "subject", profile)
            .unwrap()
            .unwrap();
        assert_eq!(found.user.id, owner.user.id);
        assert!(store
            .subject_authority(&connection, "stranger", profile)
            .unwrap()
            .is_none());
        store
            .disable_oidc_connection("admin", &connection.id)
            .unwrap();
        assert!(store
            .subject_authority(&connection, "subject", profile)
            .unwrap()
            .is_none());
    }

    #[test]
    fn test_authority_lookup_does_not_record_usage() {
        let (store, owner, request) = fixture();
        let issued = store.create_user_key(&owner, &request, None).unwrap();
        assert!(store
            .key_authority(&issued.key.summary.id)
            .unwrap()
            .is_some());
        store.flush_usage().unwrap();
        assert!(store
            .user_keys(&owner.user.id, request.grants.profile_id, 0)
            .unwrap()[0]
            .summary
            .last_used_at
            .is_none());
        assert!(store.exchange_user_key(&issued.secret).unwrap().is_some());
        store.flush_usage().unwrap();
        assert!(store
            .user_keys(&owner.user.id, request.grants.profile_id, 0)
            .unwrap()[0]
            .summary
            .last_used_at
            .is_some());
        let audit = store.recent_audit(1).unwrap();
        let detail: serde_json::Value = serde_json::from_str(audit[0].2.as_ref().unwrap()).unwrap();
        assert_eq!(detail["key_id"], issued.key.summary.id);
        assert_eq!(detail["owner_id"], owner.user.id);
        assert!(!audit[0].2.as_ref().unwrap().contains(&issued.secret));
    }

    #[tokio::test]
    async fn test_exchange_storage_failure_returns_unavailable_without_secret() {
        use axum::{
            body::Body,
            http::{Request, StatusCode},
        };
        use http_body_util::BodyExt;
        use tower::ServiceExt;
        let state = crate::rest::state::ApiState::in_memory(crate::config::Config::default());
        let (store, owner, request) = fixture_in(state.auth.store.clone());
        let issued = store.create_user_key(&owner, &request, None).unwrap();
        store
            .with_conn(|conn| {
                conn.execute_batch("ALTER TABLE access_key RENAME TO unavailable_key")?;
                Ok(())
            })
            .unwrap();
        let response = crate::rest::build_profile_router(state).oneshot(Request::builder()
            .method("POST").uri("/api/v1/auth/token")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::json!({"grant_type": "operator:access-key", "access_key": issued.secret}).to_string())).unwrap()).await.unwrap();
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(response.headers()["cache-control"], "no-store");
        let body = response.into_body().collect().await.unwrap().to_bytes();
        assert!(!String::from_utf8_lossy(&body).contains(&issued.secret));
    }

    #[test]
    fn test_profile_reads_require_explicit_capability() {
        let (store, owner, request) = fixture();
        let issued = store.create_user_key(&owner, &request, None).unwrap();
        let (mut authority, _) = store
            .key_authority(&issued.key.summary.id)
            .unwrap()
            .unwrap();
        for path in ["/api/v1/profiles", "/api/v1/profiles/{profile_id}"] {
            assert!(super::super::policy::authorize(&authority, "GET", path).is_err());
        }
        authority.grants.capabilities.push("profiles:read".into());
        authority.grants.instance_wide = true;
        assert!(super::super::policy::authorize(&authority, "GET", "/api/v1/profiles").is_ok());
        assert!(super::super::policy::authorize(&authority, "POST", "/api/v1/profiles").is_err());
    }

    #[test]
    fn test_concurrent_rotation_commits_one_replacement() {
        let (store, owner, request) = fixture();
        let original = store.create_user_key(&owner, &request, None).unwrap();
        let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
        let workers: Vec<_> = (0..2)
            .map(|_| {
                let store = store.clone();
                let owner = owner.clone();
                let request = request.clone();
                let id = original.key.summary.id.clone();
                let barrier = std::sync::Arc::clone(&barrier);
                std::thread::spawn(move || {
                    barrier.wait();
                    store.create_user_key(&owner, &request, Some(&id)).is_ok()
                })
            })
            .collect();
        assert_eq!(
            workers
                .into_iter()
                .map(|worker| usize::from(worker.join().unwrap()))
                .sum::<usize>(),
            1
        );
        assert_eq!(
            store
                .user_keys(&owner.user.id, request.grants.profile_id, 0)
                .unwrap()
                .len(),
            2
        );
    }

    #[test]
    fn test_profile_expiry_and_issuance_limits_fail_closed() {
        let (store, owner, mut request) = fixture();
        let profile = request.grants.profile_id;
        request.grants.profile_id = Uuid::new_v4();
        assert!(store.create_user_key(&owner, &request, None).is_err());
        request.grants.profile_id = profile;
        let key = store.create_user_key(&owner, &request, None).unwrap();
        store
            .with_conn(|conn| {
                conn.execute(
                    "UPDATE access_key SET expires_at = '2000-01-01T00:00:00Z' WHERE id = ?1",
                    [&key.key.summary.id],
                )?;
                Ok(())
            })
            .unwrap();
        assert!(store.exchange_user_key(&key.secret).unwrap().is_none());
        for _ in 1..MAX_ISSUED_PER_WINDOW {
            store.create_user_key(&owner, &request, None).unwrap();
        }
        assert!(store
            .create_user_key(&owner, &request, None)
            .err()
            .unwrap()
            .is::<IssuanceLimited>());
    }

    #[test]
    fn test_key_lifecycle_hashes_and_revalidates_owner() {
        let (store, owner, request) = fixture();
        let issued = store.create_user_key(&owner, &request, None).unwrap();
        assert_eq!(issued.secret.len(), KEY_LENGTH);
        let hash: String = store
            .with_conn(|conn| {
                Ok(conn.query_row(
                    "SELECT key_hash FROM access_key WHERE id = ?1",
                    [&issued.key.summary.id],
                    |row| row.get(0),
                )?)
            })
            .unwrap();
        assert_eq!(hash, hash_secret(&issued.secret));
        assert_ne!(hash, issued.secret);
        assert!(store.exchange_user_key(&issued.secret).unwrap().is_some());
        assert!(store
            .user_keys(&owner.user.id, Uuid::new_v4(), 0)
            .unwrap()
            .is_empty());
        let mut reduced = request.grants;
        reduced.projects.clear();
        store
            .update_operator_user(
                "admin",
                &owner.user.id,
                &UpdateOperatorUser {
                    enabled: true,
                    administrator: false,
                    grants: reduced,
                },
                false,
            )
            .unwrap();
        let (authority, _) = store.exchange_user_key(&issued.secret).unwrap().unwrap();
        assert!(authority.grants.projects.is_empty());
        store
            .disable_oidc_connection("admin", &owner.user.connection_id)
            .unwrap();
        assert!(store.exchange_user_key(&issued.secret).unwrap().is_none());
    }

    #[test]
    fn test_rotation_has_one_winner_and_preserves_old_on_failure() {
        let (store, owner, request) = fixture();
        let old = store.create_user_key(&owner, &request, None).unwrap();
        let mut expanded = request.clone();
        expanded.grants.instance_wide = true;
        assert!(store
            .create_user_key(&owner, &expanded, Some(&old.key.summary.id))
            .is_err());
        assert!(store.exchange_user_key(&old.secret).unwrap().is_some());
        let replacement = store
            .create_user_key(&owner, &request, Some(&old.key.summary.id))
            .unwrap();
        assert!(store
            .create_user_key(&owner, &request, Some(&old.key.summary.id))
            .is_err());
        assert!(store.exchange_user_key(&old.secret).unwrap().is_none());
        assert!(store
            .exchange_user_key(&replacement.secret)
            .unwrap()
            .is_some());
        assert!(store
            .revoke_user_key(&owner.user.id, &replacement.key.summary.id, false)
            .unwrap());
        assert!(store
            .exchange_user_key(&replacement.secret)
            .unwrap()
            .is_none());
        assert_eq!(
            store
                .user_keys(&owner.user.id, request.grants.profile_id, 0)
                .unwrap()
                .len(),
            2
        );
    }
}

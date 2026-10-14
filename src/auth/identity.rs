use std::collections::BTreeSet;

use anyhow::{ensure, Result};
use chrono::{DateTime, Utc};
use rusqlite::{params, OptionalExtension};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;
use uuid::Uuid;

use super::scope::{Access, Principal, RouteRule, ROUTE_RULES};
use super::secret::{generate_prefixed_secret, hash_secret};
use super::store::AuthStore;
use crate::rest::dto::auth::{AccessKeySummary, PrincipalKind, Scope};

pub const KEY_PREFIX: &str = "opk_v1";
pub const KEY_PAGE_SIZE: usize = 50;
pub const RECENT_LOGIN_SECONDS: i64 = 300;

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

#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema, JsonSchema, TS)]
#[ts(export)]
pub struct PermissionGrant {
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
    pub grants: PermissionGrant,
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
    pub display_prefix: String,
    pub grants: PermissionGrant,
    pub replaces_id: Option<String>,
}

#[derive(Serialize, ToSchema)]
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

pub fn route_capability(rule: &RouteRule) -> Option<String> {
    let Access::Scoped(scope) = rule.access else {
        return None;
    };
    let resource = rule.path.strip_prefix("/api/v1/")?.split('/').next()?;
    if matches!(resource, "auth" | "oidc" | "users") {
        return None;
    }
    let action = match (scope, rule.method) {
        (Scope::Execute, _) => "execute",
        (_, "GET" | "HEAD") => "read",
        (_, "POST") => "create",
        (_, "DELETE") => "delete",
        (_, "PUT" | "PATCH") => "update",
        _ => return None,
    };
    Some(format!("{resource}:{action}"))
}

pub fn capabilities() -> Vec<String> {
    ROUTE_RULES
        .iter()
        .filter_map(route_capability)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
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

fn read_grants(conn: &rusqlite::Connection, id: &str, key: bool) -> Result<PermissionGrant> {
    let (capability_sql, project_sql) = if key {
        (
            "SELECT capability FROM key_capability WHERE key_id = ?1 ORDER BY capability",
            "SELECT project FROM key_project WHERE key_id = ?1 ORDER BY project",
        )
    } else {
        (
            "SELECT capability FROM user_capability WHERE user_id = ?1 ORDER BY capability",
            "SELECT project FROM user_project WHERE user_id = ?1 ORDER BY project",
        )
    };
    let read = |sql| -> Result<Vec<String>> {
        Ok(conn
            .prepare(sql)?
            .query_map([id], |r| r.get(0))?
            .collect::<rusqlite::Result<_>>()?)
    };
    let sql = if key {
        "SELECT instance_wide FROM access_key WHERE id = ?1"
    } else {
        "SELECT instance_wide FROM operator_user WHERE id = ?1"
    };
    Ok(PermissionGrant {
        instance_wide: conn.query_row(sql, [id], |r| r.get(0))?,
        capabilities: read(capability_sql)?,
        projects: read(project_sql)?,
    })
}

fn write_grants(
    conn: &rusqlite::Connection,
    id: &str,
    grants: &PermissionGrant,
    key: bool,
) -> Result<()> {
    let (capability_sql, project_sql) = if key {
        (
            "INSERT INTO key_capability VALUES (?1, ?2)",
            "INSERT INTO key_project VALUES (?1, ?2)",
        )
    } else {
        (
            "INSERT INTO user_capability VALUES (?1, ?2)",
            "INSERT INTO user_project VALUES (?1, ?2)",
        )
    };
    let sql = if key {
        "UPDATE access_key SET instance_wide = ?1 WHERE id = ?2"
    } else {
        "UPDATE operator_user SET instance_wide = ?1 WHERE id = ?2"
    };
    conn.execute(sql, params![grants.instance_wide, id])?;
    for value in &grants.capabilities {
        conn.execute(capability_sql, params![id, value])?;
    }
    for value in &grants.projects {
        conn.execute(project_sql, params![id, value])?;
    }
    Ok(())
}

fn read_user(conn: &rusqlite::Connection, id: &str) -> Result<Option<OperatorUser>> {
    let user = conn.query_row("SELECT id, connection_id, issuer, subject, enabled, administrator FROM operator_user WHERE id = ?1", [id], |r| Ok(OperatorUser {
        id: r.get(0)?, connection_id: r.get(1)?, issuer: r.get(2)?, subject: r.get(3)?, enabled: r.get(4)?, administrator: r.get(5)?, grants: PermissionGrant::default(),
    })).optional()?;
    user.map(|mut u| {
        u.grants = read_grants(conn, id, false)?;
        Ok(u)
    })
    .transpose()
}

fn audit(conn: &rusqlite::Connection, actor: &str, event: &str, id: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO audit_log (at, event, subject, detail, succeeded) VALUES (?1, ?2, ?3, ?4, 1)",
        params![Utc::now().to_rfc3339(), event, actor, id],
    )?;
    Ok(())
}

impl AuthStore {
    pub fn oidc_connections(&self) -> Result<Vec<OidcConnection>> {
        self.with_conn(|conn| {
            let rows = conn.prepare("SELECT id, provider, issuer, client_id, redirect_uri, disabled FROM oidc_connection ORDER BY created_at")?.query_map([], |r| {
                let provider: String = r.get(1)?;
                Ok(OidcConnection { id: r.get(0)?, provider: if provider == "keycloak" { OidcProvider::Keycloak } else { OidcProvider::Authentik }, issuer: r.get(2)?, client_id: r.get(3)?, redirect_uri: r.get(4)?, disabled: r.get(5)? })
            })?.collect::<rusqlite::Result<_>>()?;
            Ok(rows)
        })
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
            audit(&tx, actor, "oidc connection created", &id)?;
            tx.commit()?;
            Ok(id)
        })
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
            audit(&tx, actor, "oidc connection disabled", id)?;
            tx.commit()?;
            Ok(())
        })
    }

    pub fn provision_oidc_user(
        &self,
        connection: &OidcConnection,
        subject: &str,
    ) -> Result<String> {
        self.with_conn(|conn| {
            let tx = conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
            let id = Uuid::new_v4().to_string();
            tx.execute("INSERT OR IGNORE INTO operator_user (id, connection_id, issuer, subject, created_at) VALUES (?1, ?2, ?3, ?4, ?5)", params![id, connection.id, connection.issuer, subject, Utc::now().timestamp()])?;
            let id = tx.query_row("SELECT id FROM operator_user WHERE issuer = ?1 AND subject = ?2 AND connection_id = ?3", params![connection.issuer, subject, connection.id], |r| r.get(0))?;
            tx.commit()?;
            Ok(id)
        })
    }

    pub fn operator_users(&self) -> Result<Vec<OperatorUser>> {
        self.with_conn(|conn| {
            let ids = conn
                .prepare("SELECT id FROM operator_user ORDER BY created_at")?
                .query_map([], |r| r.get::<_, String>(0))?
                .collect::<rusqlite::Result<Vec<_>>>()?;
            ids.iter()
                .map(|id| read_user(conn, id)?.ok_or_else(|| anyhow::anyhow!("user disappeared")))
                .collect()
        })
    }

    pub fn update_operator_user(
        &self,
        actor: &str,
        id: &str,
        update: &UpdateOperatorUser,
    ) -> Result<()> {
        validate_grants(&update.grants)?;
        self.with_conn(|conn| {
            let tx = conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
            ensure!(
                tx.execute(
                    "UPDATE operator_user SET enabled = ?1, administrator = ?2 WHERE id = ?3",
                    params![update.enabled, update.administrator, id]
                )? == 1,
                "unknown user"
            );
            tx.execute("DELETE FROM user_capability WHERE user_id = ?1", [id])?;
            tx.execute("DELETE FROM user_project WHERE user_id = ?1", [id])?;
            write_grants(&tx, id, &update.grants, false)?;
            audit(&tx, actor, "user permissions updated", id)?;
            tx.commit()?;
            Ok(())
        })
    }

    pub fn user_authority(&self, id: &str) -> Result<Option<UserAuthority>> {
        self.with_conn(|conn| {
            let Some(user) = read_user(conn, id)? else {
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
            Ok(Some(UserAuthority {
                grants: user.grants.clone(),
                user,
                key_id: None,
                authenticated_at: None,
            }))
        })
    }

    pub fn oidc_session(
        &self,
        user_id: &str,
        authenticated_at: i64,
    ) -> Result<(String, String, DateTime<Utc>)> {
        ensure!(
            self.user_authority(user_id)?.is_some(),
            "account requires administrator approval"
        );
        let session = self.create_session()?;
        self.with_conn(|conn| {
            conn.execute(
                "UPDATE session SET user_id = ?1, authenticated_at = ?2 WHERE token_hash = ?3",
                params![user_id, authenticated_at, hash_secret(&session.0)],
            )?;
            Ok(())
        })?;
        Ok(session)
    }

    pub fn session_authority(&self, id: &str) -> Result<Option<UserAuthority>> {
        let row: (Option<String>, Option<i64>) = self.with_conn(|conn| {
            Ok(conn.query_row(
                "SELECT user_id, authenticated_at FROM session WHERE id = ?1",
                [id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )?)
        })?;
        let Some(user_id) = row.0 else {
            return Ok(None);
        };
        Ok(self.user_authority(&user_id)?.map(|mut a| {
            a.authenticated_at = row.1;
            a
        }))
    }

    pub fn session_user_id(&self, id: &str) -> Result<Option<String>> {
        self.with_conn(|conn| {
            Ok(
                conn.query_row("SELECT user_id FROM session WHERE id = ?1", [id], |r| {
                    r.get(0)
                })?,
            )
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
            !request.name.trim().is_empty() && request.name.len() <= 128,
            "invalid key name"
        );
        ensure!(
            (1..=365).contains(&request.expires_in_days),
            "expiry must be 1 to 365 days"
        );
        ensure!(
            !request.grants.capabilities.is_empty(),
            "select at least one capability"
        );
        let id = Uuid::new_v4().to_string();
        let secret = generate_prefixed_secret(KEY_PREFIX)?;
        let now = Utc::now();
        let expires = now + chrono::Duration::days(i64::from(request.expires_in_days));
        self.with_conn(|conn| {
            let tx = conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
            let owner = read_user(&tx, &user.user.id)?.ok_or_else(|| anyhow::anyhow!("unknown user"))?;
            ensure!(owner.enabled, "account disabled");
            let disabled: bool = tx.query_row("SELECT disabled FROM oidc_connection WHERE id = ?1", [&owner.connection_id], |r| r.get(0))?;
            ensure!(!disabled, "connection disabled");
            ensure!(!request.grants.instance_wide || owner.grants.instance_wide, "instance permission exceeds owner");
            ensure!(request.grants.capabilities.iter().all(|c| owner.grants.capabilities.contains(c)), "capability exceeds owner permissions");
            ensure!(request.grants.projects.iter().all(|p| owner.grants.projects.contains(p)), "project exceeds owner permissions");
            if let Some(old) = replaces {
                let grants = read_grants(&tx, old, true)?;
                ensure!(!request.grants.instance_wide || grants.instance_wide, "rotation cannot expand instance permissions");
                ensure!(request.grants.capabilities.iter().all(|c| grants.capabilities.contains(c)) && request.grants.projects.iter().all(|p| grants.projects.contains(p)), "rotation cannot expand permissions");
                ensure!(tx.execute("UPDATE access_key SET revoked_at = ?1 WHERE id = ?2 AND user_id = ?3 AND revoked_at IS NULL AND julianday(expires_at) > julianday(?1)", params![now.to_rfc3339(), old, owner.id])? == 1, "key is not active");
            }
            tx.execute("INSERT INTO access_key (id, name, key_hash, scopes, created_at, expires_at, user_id, replaces_id) VALUES (?1, ?2, ?3, '', ?4, ?5, ?6, ?7)", params![id, request.name.trim(), hash_secret(&secret), now.to_rfc3339(), expires.to_rfc3339(), owner.id, replaces])?;
            write_grants(&tx, &id, &request.grants, true)?;
            audit(&tx, &owner.id, if replaces.is_some() { "key rotated" } else { "key created" }, &id)?;
            tx.commit()?;
            Ok(())
        })?;
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
                display_prefix: format!("{KEY_PREFIX}_{}…", &id[..8]),
                grants: request.grants.clone(),
                replaces_id: replaces.map(str::to_owned),
            },
            secret,
        })
    }

    pub fn key_authority(&self, id: &str) -> Result<Option<(UserAuthority, DateTime<Utc>)>> {
        let row: Option<(String, String)> = self.with_conn(|conn| Ok(conn.query_row("SELECT user_id, expires_at FROM access_key WHERE id = ?1 AND user_id IS NOT NULL AND revoked_at IS NULL", [id], |r| Ok((r.get(0)?, r.get(1)?))).optional()?))?;
        let Some((owner, expires)) = row else {
            return Ok(None);
        };
        let expires = DateTime::parse_from_rfc3339(&expires)?.with_timezone(&Utc);
        if expires <= Utc::now() {
            return Ok(None);
        }
        let Some(mut authority) = self.user_authority(&owner)? else {
            return Ok(None);
        };
        let grants = self.with_conn(|conn| read_grants(conn, id, true))?;
        authority.grants.instance_wide &= grants.instance_wide;
        authority
            .grants
            .capabilities
            .retain(|c| grants.capabilities.contains(c));
        authority
            .grants
            .projects
            .retain(|p| grants.projects.contains(p));
        authority.key_id = Some(id.to_string());
        self.usage
            .record(super::usage::CredentialTable::Key, id, Utc::now());
        Ok(Some((authority, expires)))
    }

    pub fn exchange_user_key(
        &self,
        secret: &str,
    ) -> Result<Option<(UserAuthority, DateTime<Utc>)>> {
        if !secret.starts_with(&format!("{KEY_PREFIX}_")) || secret.len() > 128 {
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
        id.map(|id| self.key_authority(&id))
            .transpose()
            .map(Option::flatten)
    }

    pub fn revoke_user_key(&self, actor: &str, id: &str, administrator: bool) -> Result<bool> {
        self.with_conn(|conn| {
            let tx = conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
            let exists: bool = tx.query_row(
                "SELECT EXISTS(SELECT 1 FROM access_key WHERE id = ?1 AND (user_id = ?2 OR ?3))",
                params![id, actor, administrator],
                |r| r.get(0),
            )?;
            if !exists {
                return Ok(false);
            }
            let count = tx.execute(
                "UPDATE access_key SET revoked_at = ?1 WHERE id = ?2 AND revoked_at IS NULL",
                params![Utc::now().to_rfc3339(), id],
            )?;
            if count > 0 {
                audit(&tx, actor, "key revoked", id)?;
            }
            tx.commit()?;
            Ok(true)
        })
    }

    pub fn user_keys(&self, owner: &str, offset: u32) -> Result<Vec<UserKey>> {
        self.with_conn(|conn| {
            let mut stmt = conn.prepare("SELECT id, name, created_at, expires_at, last_used_at, revoked_at, replaces_id FROM access_key WHERE user_id = ?1 ORDER BY created_at DESC, id LIMIT ?2 OFFSET ?3")?;
            let rows = stmt.query_map(params![owner, KEY_PAGE_SIZE, offset], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?, r.get::<_, String>(3)?, r.get::<_, Option<String>>(4)?, r.get::<_, Option<String>>(5)?, r.get::<_, Option<String>>(6)?)))?.collect::<rusqlite::Result<Vec<_>>>()?;
            rows.into_iter().map(|(id, name, created, expires, used, revoked, replaces_id)| {
                let parse = |s: &str| -> Result<DateTime<Utc>> { Ok(DateTime::parse_from_rfc3339(s)?.with_timezone(&Utc)) };
                Ok(UserKey { display_prefix: format!("{KEY_PREFIX}_{}…", &id[..8]), owner_id: owner.to_string(), grants: read_grants(conn, &id, true)?, replaces_id, summary: AccessKeySummary { id, name, scopes: Vec::new(), created_at: parse(&created)?, expires_at: parse(&expires)?, last_used_at: used.as_deref().map(parse).transpose()?, revoked_at: revoked.as_deref().map(parse).transpose()? } })
            }).collect()
        })
    }
}

impl UserAuthority {
    pub fn principal(&self, expires_at: DateTime<Utc>, session_id: Option<String>) -> Principal {
        Principal {
            subject: self.user.id.clone(),
            scopes: Scope::ALL.to_vec(),
            kind: if session_id.is_some() {
                PrincipalKind::Session
            } else {
                PrincipalKind::AccessToken
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

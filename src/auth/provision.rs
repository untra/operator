use anyhow::{ensure, Result};
use chrono::Utc;
use rusqlite::{params, OptionalExtension, TransactionBehavior};
use uuid::Uuid;

use super::audit::AuditAction;
use super::identity::{insert_connection, record_audit, CreateOidcConnection};
use super::store::AuthStore;

const HOST_ADMIN: &str = "host administrator";

impl AuthStore {
    pub fn has_hosted_administrator(&self) -> Result<bool> {
        self.with_conn(|conn| Ok(conn.query_row(
            "SELECT EXISTS (SELECT 1 FROM operator_user u JOIN oidc_connection c ON u.connection_id = c.id
             WHERE u.enabled AND u.administrator AND NOT c.disabled)", [], |r| r.get(0),
        )?))
    }

    pub fn provision_hosted_admin(
        &self,
        request: &CreateOidcConnection,
        subject: &str,
    ) -> Result<String> {
        ensure!(!subject.trim().is_empty(), "OIDC subject is required");
        ensure!(
            !request.client_id.trim().is_empty() && request.client_id.len() <= 128,
            "Invalid client ID"
        );
        let issuer = request.issuer.trim();
        self.with_conn(|conn| {
            let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
            let connection: Option<(String, bool)> = tx.query_row(
                "SELECT id, client_id = ?2 AND redirect_uri = ?3 AND client_secret_file IS ?4
                 AND provider = ?5 AND NOT disabled FROM oidc_connection WHERE issuer = ?1",
                params![issuer, request.client_id, request.redirect_uri, request.client_secret_file,
                    request.provider.as_str()], |row| Ok((row.get(0)?, row.get(1)?)),
            ).optional()?;
            let connection_id = match connection {
                Some((id, true)) => id,
                Some((_, false)) => anyhow::bail!("Existing OIDC connection is disabled or has different settings"),
                None => {
                    let id = Uuid::new_v4().to_string();
                    insert_connection(&tx, HOST_ADMIN, &id, request)?;
                    id
                }
            };
            let id: String = tx.query_row(
                "INSERT INTO operator_user (id, connection_id, issuer, subject, enabled, administrator, created_at)
                 VALUES (?1, ?2, ?3, ?4, 1, 1, ?5) ON CONFLICT (issuer, subject) DO UPDATE
                 SET enabled = 1, administrator = 1 WHERE connection_id = excluded.connection_id RETURNING id",
                params![Uuid::new_v4().to_string(), connection_id, issuer, subject, Utc::now().timestamp()],
                |row| row.get(0),
            )?;
            record_audit(&tx, HOST_ADMIN, AuditAction::UserPermissionsChanged, &id)?;
            tx.commit()?;
            Ok(id)
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::auth::identity::{CreateOidcConnection, OidcProvider};
    use crate::auth::store::AuthStore;

    fn connection() -> CreateOidcConnection {
        CreateOidcConnection {
            provider: OidcProvider::Keycloak,
            issuer: "https://id.example/realm".into(),
            client_id: "operator".into(),
            redirect_uri: "https://operator.example/api/v1/auth/oidc/callback".into(),
            client_secret_file: None,
        }
    }

    #[test]
    fn test_provision_hosted_admin_retry_preserves_identity() {
        let store = AuthStore::in_memory().unwrap();
        assert!(!store.has_hosted_administrator().unwrap());
        let first = store
            .provision_hosted_admin(&connection(), "sub-1")
            .unwrap();
        assert_eq!(
            first,
            store
                .provision_hosted_admin(&connection(), "sub-1")
                .unwrap()
        );
        assert!(store.has_hosted_administrator().unwrap());
        let users = store.operator_users(0).unwrap();
        assert_eq!(users.len(), 1);
        assert!(users[0].enabled && users[0].administrator);
    }

    #[test]
    fn test_provision_hosted_admin_conflicting_client_preserves_provider() {
        let store = AuthStore::in_memory().unwrap();
        store
            .provision_hosted_admin(&connection(), "sub-1")
            .unwrap();
        let mut other = connection();
        other.client_id = "another-client".into();
        assert!(store.provision_hosted_admin(&other, "sub-2").is_err());
        assert_eq!(store.operator_users(0).unwrap().len(), 1);
    }
}

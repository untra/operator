//! Auth database schema and migrations.
//!
//! There is no migration framework in this repo, so this is the smallest thing
//! that works: an ordered list of migrations applied inside one transaction and
//! tracked by `SQLite`'s own `user_version` pragma. Appending is the only legal
//! edit — editing a shipped migration would leave already-migrated databases
//! silently inconsistent with new ones.

use anyhow::{Context, Result};
use rusqlite::{Connection, TransactionBehavior};

/// Ordered schema migrations. **Append only.**
const MIGRATIONS: &[&str] = &[
    // v1 — initial schema.
    r#"
    -- The single admin account. `id` is pinned to 1 by CHECK, so a second
    -- INSERT fails on the primary key rather than creating a second admin.
    -- This is what makes concurrent bootstrap resolve to exactly one winner
    -- without any application-level locking.
    CREATE TABLE admin_account (
        id              INTEGER PRIMARY KEY CHECK (id = 1),
        subject         TEXT    NOT NULL,
        password_hash   TEXT    NOT NULL,
        -- Set while a mounted bootstrap secret has been accepted but the
        -- operator has not yet chosen their own password.
        awaiting_reset  INTEGER NOT NULL DEFAULT 0,
        created_at      TEXT    NOT NULL,
        updated_at      TEXT    NOT NULL
    );

    -- Ed25519 signing keys. Keyed by `kid` so a rotation can verify tokens
    -- issued under the previous key until they expire.
    CREATE TABLE signing_key (
        kid         TEXT PRIMARY KEY,
        private_pem TEXT NOT NULL,
        public_pem  TEXT NOT NULL,
        created_at  TEXT NOT NULL,
        retired_at  TEXT
    );

    -- Opaque browser sessions. Only the hash of the cookie value is stored, so
    -- a database read does not yield a usable cookie.
    CREATE TABLE session (
        id           TEXT PRIMARY KEY,
        token_hash   TEXT NOT NULL UNIQUE,
        csrf_hash    TEXT NOT NULL,
        created_at   TEXT NOT NULL,
        expires_at   TEXT NOT NULL,
        last_used_at TEXT,
        revoked_at   TEXT
    );
    CREATE INDEX idx_session_token_hash ON session(token_hash);

    -- A refresh-token family is one continuous client authorization. Rotation
    -- replaces the token within the family; presenting a retired token revokes
    -- the whole family, which is why the family is a first-class row.
    CREATE TABLE refresh_family (
        id            TEXT PRIMARY KEY,
        client_id     TEXT NOT NULL,
        scopes        TEXT NOT NULL,
        created_at    TEXT NOT NULL,
        -- Absolute deadline, fixed at issuance and never extended by rotation.
        absolute_expires_at TEXT NOT NULL,
        last_used_at  TEXT,
        revoked_at    TEXT,
        revoked_reason TEXT
    );

    CREATE TABLE refresh_token (
        id          TEXT PRIMARY KEY,
        family_id   TEXT NOT NULL REFERENCES refresh_family(id) ON DELETE CASCADE,
        token_hash  TEXT NOT NULL UNIQUE,
        created_at  TEXT NOT NULL,
        -- Idle deadline for this particular token in the chain.
        expires_at  TEXT NOT NULL,
        -- Set when redeemed. A redeemed token presented again is reuse.
        consumed_at TEXT
    );
    CREATE INDEX idx_refresh_token_hash ON refresh_token(token_hash);
    CREATE INDEX idx_refresh_token_family ON refresh_token(family_id);

    -- In-flight device authorizations (RFC 8628).
    CREATE TABLE device_authorization (
        id               TEXT PRIMARY KEY,
        device_code_hash TEXT NOT NULL UNIQUE,
        user_code        TEXT NOT NULL UNIQUE,
        client_id        TEXT NOT NULL,
        scopes           TEXT NOT NULL,
        created_at       TEXT NOT NULL,
        expires_at       TEXT NOT NULL,
        approved_at      TEXT,
        denied_at        TEXT,
        -- Set once exchanged, so a device code cannot be redeemed twice.
        consumed_at      TEXT,
        -- Enforces the poll interval without trusting the client.
        last_polled_at   TEXT
    );
    CREATE INDEX idx_device_user_code ON device_authorization(user_code);

    -- Service access keys. Hash-only: the secret is shown once at creation.
    CREATE TABLE access_key (
        id           TEXT PRIMARY KEY,
        name         TEXT NOT NULL,
        key_hash     TEXT NOT NULL UNIQUE,
        scopes       TEXT NOT NULL,
        created_at   TEXT NOT NULL,
        expires_at   TEXT NOT NULL,
        last_used_at TEXT,
        revoked_at   TEXT
    );
    CREATE INDEX idx_access_key_hash ON access_key(key_hash);

    -- Append-only audit trail. Never contains secrets.
    CREATE TABLE audit_log (
        id         INTEGER PRIMARY KEY AUTOINCREMENT,
        at         TEXT NOT NULL,
        event      TEXT NOT NULL,
        subject    TEXT,
        detail     TEXT,
        succeeded  INTEGER NOT NULL
    );
    CREATE INDEX idx_audit_at ON audit_log(at);

    -- Persisted rate limiting, so restarting the process does not reset an
    -- attacker's budget. Keyed by bucket name (e.g. "login").
    CREATE TABLE rate_limit (
        bucket        TEXT PRIMARY KEY,
        attempts      INTEGER NOT NULL DEFAULT 0,
        first_at      TEXT NOT NULL,
        last_at       TEXT NOT NULL,
        -- Backoff only; never a permanent lockout. A permanent lockout on a
        -- single-account system is a denial-of-service against the only human
        -- who could undo it.
        retry_after   TEXT
    );
    "#,
];

/// Apply any migrations the database has not seen.
///
/// Two Operator processes can open the same workspace at once — the TUI runs an
/// embedded API server while `operator api` may already be running, and the
/// test suite opens many at once. So the version check and the migration must
/// be one atomic step.
///
/// `BEGIN IMMEDIATE` takes the write lock up front, before `user_version` is
/// read. A second starter blocks there, and by the time it acquires the lock
/// the first has committed, so it re-reads the *new* version and finds nothing
/// to do. Reading the version outside the transaction instead lets both see 0
/// and both try to create the same tables.
pub fn migrate(conn: &mut Connection) -> Result<()> {
    let tx = conn
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .context("beginning auth migration")?;

    let current: i64 = tx
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .context("reading user_version")?;

    let current = usize::try_from(current).unwrap_or(0);
    if current > MIGRATIONS.len() {
        anyhow::bail!(
            "auth database is at schema version {current}, but this build only knows {}. \
             Downgrading is not supported.",
            MIGRATIONS.len()
        );
    }
    if current == MIGRATIONS.len() {
        return Ok(());
    }

    for (index, migration) in MIGRATIONS.iter().enumerate().skip(current) {
        let version = index + 1;
        tx.execute_batch(migration)
            .with_context(|| format!("applying auth migration v{version}"))?;
        // `PRAGMA user_version` does not accept a bound parameter.
        tx.execute_batch(&format!("PRAGMA user_version = {version}"))
            .with_context(|| format!("stamping auth schema version {version}"))?;
    }

    tx.commit().context("committing auth migrations")?;
    Ok(())
}

/// Connection pragmas applied on open.
pub fn apply_pragmas(conn: &Connection) -> Result<()> {
    // WAL keeps a reader from blocking the writer, which matters because the
    // TUI reads auth state on the same database the API server writes.
    conn.pragma_update(None, "journal_mode", "WAL")
        .context("enabling WAL")?;
    conn.pragma_update(None, "foreign_keys", "ON")
        .context("enabling foreign keys")?;
    conn.pragma_update(None, "synchronous", "NORMAL")
        .context("setting synchronous")?;
    // Wait for a concurrent writer rather than failing instantly. Two Operator
    // processes on one workspace is normal, not exceptional.
    conn.busy_timeout(std::time::Duration::from_secs(5))
        .context("setting busy timeout")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn migrated() -> Connection {
        let mut conn = Connection::open_in_memory().unwrap();
        apply_pragmas(&conn).unwrap();
        migrate(&mut conn).unwrap();
        conn
    }

    #[test]
    fn test_migrate_creates_every_table() {
        let conn = migrated();
        for table in [
            "admin_account",
            "signing_key",
            "session",
            "refresh_family",
            "refresh_token",
            "device_authorization",
            "access_key",
            "audit_log",
            "rate_limit",
        ] {
            let count: i64 = conn
                .query_row(
                    "SELECT count(*) FROM sqlite_master WHERE type='table' AND name=?1",
                    [table],
                    |r| r.get(0),
                )
                .unwrap();
            assert_eq!(count, 1, "migration should create {table}");
        }
    }

    #[test]
    fn test_migrate_is_idempotent() {
        let mut conn = Connection::open_in_memory().unwrap();
        apply_pragmas(&conn).unwrap();
        migrate(&mut conn).unwrap();
        let version: i64 = conn
            .query_row("PRAGMA user_version", [], |r| r.get(0))
            .unwrap();

        // Running again must be a no-op, not an error and not a re-apply.
        migrate(&mut conn).unwrap();
        let after: i64 = conn
            .query_row("PRAGMA user_version", [], |r| r.get(0))
            .unwrap();
        assert_eq!(version, after);
        assert_eq!(after as usize, MIGRATIONS.len());
    }

    #[test]
    fn test_admin_account_is_a_singleton() {
        // This constraint is what makes concurrent bootstrap safe: the second
        // insert fails rather than creating a second admin.
        let conn = migrated();
        let insert =
            "INSERT INTO admin_account (id, subject, password_hash, created_at, updated_at) \
                      VALUES (?1, 'admin', 'hash', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')";
        conn.execute(insert, [1]).unwrap();

        assert!(
            conn.execute(insert, [1]).is_err(),
            "a second admin row with id=1 must be rejected"
        );
        assert!(
            conn.execute(insert, [2]).is_err(),
            "the CHECK must reject any id other than 1"
        );
    }

    #[test]
    fn test_refresh_tokens_cascade_with_their_family() {
        // Revoking a family must not leave orphaned tokens behind that could
        // still be looked up by hash.
        let conn = migrated();
        conn.execute(
            "INSERT INTO refresh_family (id, client_id, scopes, created_at, absolute_expires_at) \
             VALUES ('fam1', 'vscode', 'read', '2026-01-01T00:00:00Z', '2026-04-01T00:00:00Z')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO refresh_token (id, family_id, token_hash, created_at, expires_at) \
             VALUES ('rt1', 'fam1', 'hash1', '2026-01-01T00:00:00Z', '2026-02-01T00:00:00Z')",
            [],
        )
        .unwrap();

        conn.execute("DELETE FROM refresh_family WHERE id = 'fam1'", [])
            .unwrap();
        let remaining: i64 = conn
            .query_row("SELECT count(*) FROM refresh_token", [], |r| r.get(0))
            .unwrap();
        assert_eq!(remaining, 0, "tokens must cascade with their family");
    }

    #[test]
    fn test_credential_hashes_are_unique() {
        let conn = migrated();
        conn.execute(
            "INSERT INTO access_key (id, name, key_hash, scopes, created_at, expires_at) \
             VALUES ('k1', 'ci', 'samehash', 'read', '2026-01-01T00:00:00Z', '2026-04-01T00:00:00Z')",
            [],
        )
        .unwrap();
        assert!(
            conn.execute(
                "INSERT INTO access_key (id, name, key_hash, scopes, created_at, expires_at) \
                 VALUES ('k2', 'other', 'samehash', 'read', '2026-01-01T00:00:00Z', '2026-04-01T00:00:00Z')",
                [],
            )
            .is_err(),
            "two keys must never share a hash"
        );
    }

    #[test]
    fn test_concurrent_migration_of_one_database_is_safe() {
        // Regression: reading `user_version` outside the transaction let two
        // starters both see 0 and both run migration v1, and the loser died
        // with "table admin_account already exists". The TUI's embedded server
        // and a separate `operator api` really do race here.
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("auth.sqlite3");

        let failures: Vec<String> = std::thread::scope(|scope| {
            let handles: Vec<_> = (0..8)
                .map(|_| {
                    let path = path.clone();
                    scope.spawn(move || {
                        let mut conn = Connection::open(&path)?;
                        apply_pragmas(&conn)?;
                        migrate(&mut conn)
                    })
                })
                .collect();
            handles
                .into_iter()
                .filter_map(|h| h.join().expect("thread should not panic").err())
                .map(|e| format!("{e:#}"))
                .collect()
        });

        assert!(
            failures.is_empty(),
            "concurrent migration must be safe, got: {failures:#?}"
        );

        let conn = Connection::open(&path).unwrap();
        let version: i64 = conn
            .query_row("PRAGMA user_version", [], |r| r.get(0))
            .unwrap();
        assert_eq!(version as usize, MIGRATIONS.len());
    }

    #[test]
    fn test_downgrade_is_refused_rather_than_silently_accepted() {
        let mut conn = Connection::open_in_memory().unwrap();
        apply_pragmas(&conn).unwrap();
        conn.execute_batch("PRAGMA user_version = 999").unwrap();

        let err = migrate(&mut conn).expect_err("a future schema must not be opened");
        assert!(err.to_string().contains("Downgrading is not supported"));
    }
}

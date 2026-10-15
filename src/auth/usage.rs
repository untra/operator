use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};

pub const FLUSH_INTERVAL: Duration = Duration::from_mins(1);
const MAX_PENDING: usize = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CredentialTable {
    Key,
    Session,
}

#[derive(Default)]
pub struct Usage {
    pending: Mutex<HashMap<(CredentialTable, String), DateTime<Utc>>>,
    started: AtomicBool,
    dropped: AtomicU64,
    flush_lock: Mutex<()>,
    inflight: AtomicUsize,
}

impl Usage {
    pub fn record(&self, table: CredentialTable, id: &str, observed: DateTime<Utc>) {
        let Ok(mut pending) = self.pending.lock() else {
            return;
        };
        let key = (table, id.to_string());
        if pending.len() + self.inflight.load(Ordering::Acquire) >= MAX_PENDING
            && !pending.contains_key(&key)
        {
            self.dropped.fetch_add(1, Ordering::Relaxed);
            return;
        }
        pending
            .entry(key)
            .and_modify(|at| *at = (*at).max(observed))
            .or_insert(observed);
    }

    pub fn flush(&self, conn: &Mutex<Connection>) -> anyhow::Result<()> {
        let _flushing = self
            .flush_lock
            .lock()
            .map_err(|_| anyhow::anyhow!("usage flush lock poisoned"))?;
        let batch = {
            let mut pending = self
                .pending
                .lock()
                .map_err(|_| anyhow::anyhow!("usage lock poisoned"))?;
            let batch = std::mem::take(&mut *pending);
            self.inflight.store(batch.len(), Ordering::Release);
            batch
        };
        let dropped = self.dropped.swap(0, Ordering::Relaxed);
        if dropped > 0 {
            tracing::warn!(dropped, "credential usage observations dropped at capacity");
        }
        if batch.is_empty() {
            return Ok(());
        }
        let result = (|| -> anyhow::Result<()> {
            let mut conn = conn
                .lock()
                .map_err(|_| anyhow::anyhow!("auth database unavailable"))?;
            let tx = conn.transaction()?;
            for ((table, id), at) in &batch {
                let sql = match table {
                    CredentialTable::Key => "UPDATE access_key SET last_used_at = ?1 WHERE id = ?2 AND (last_used_at IS NULL OR julianday(last_used_at) < julianday(?1))",
                    CredentialTable::Session => "UPDATE session SET last_used_at = ?1 WHERE id = ?2 AND (last_used_at IS NULL OR julianday(last_used_at) < julianday(?1))",
                };
                tx.execute(sql, params![at.to_rfc3339(), id])?;
            }
            tx.commit()?;
            Ok(())
        })();
        self.inflight.store(0, Ordering::Release);
        if result.is_err() {
            for ((table, id), at) in batch {
                self.record(table, &id, at);
            }
        }
        result
    }

    pub fn start(usage: &Arc<Self>, conn: &Arc<Mutex<Connection>>) {
        let Ok(runtime) = tokio::runtime::Handle::try_current() else {
            return;
        };
        if usage.started.swap(true, Ordering::AcqRel) {
            return;
        }
        let weak_usage = Arc::downgrade(usage);
        let weak_conn = Arc::downgrade(conn);
        runtime.spawn(async move {
            loop {
                tokio::time::sleep(FLUSH_INTERVAL).await;
                let (Some(usage), Some(conn)) = (weak_usage.upgrade(), weak_conn.upgrade()) else {
                    break;
                };
                let result = tokio::task::spawn_blocking(move || usage.flush(&conn)).await;
                if !matches!(result, Ok(Ok(()))) {
                    tracing::warn!("credential usage flush failed");
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_usage_coalesces_and_never_moves_backwards() {
        let conn = Mutex::new(Connection::open_in_memory().unwrap());
        conn.lock().unwrap().execute_batch("CREATE TABLE access_key (id TEXT, last_used_at TEXT); INSERT INTO access_key VALUES ('k', NULL);").unwrap();
        let usage = Usage::default();
        let now = Utc::now();
        for _ in 0..1000 {
            usage.record(CredentialTable::Key, "k", now);
        }
        assert_eq!(usage.pending.lock().unwrap().len(), 1);
        usage.flush(&conn).unwrap();
        usage.record(
            CredentialTable::Key,
            "k",
            now - chrono::Duration::seconds(20),
        );
        usage.flush(&conn).unwrap();
        let at: String = conn
            .lock()
            .unwrap()
            .query_row("SELECT last_used_at FROM access_key", [], |r| r.get(0))
            .unwrap();
        assert_eq!(at, now.to_rfc3339());
    }

    #[test]
    fn test_usage_capacity_is_bounded_and_existing_observations_advance() {
        let usage = Usage::default();
        let now = Utc::now();
        for index in 0..MAX_PENDING {
            usage.record(CredentialTable::Key, &index.to_string(), now);
        }
        usage.record(CredentialTable::Key, "overflow", now);
        usage.record(CredentialTable::Key, "0", now + chrono::Duration::seconds(1));
        let pending = usage.pending.lock().unwrap();
        assert_eq!(pending.len(), MAX_PENDING);
        assert_eq!(pending[&(CredentialTable::Key, "0".into())], now + chrono::Duration::seconds(1));
        assert_eq!(usage.dropped.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_usage_failed_batch_merges_with_concurrent_observation() {
        let conn = Arc::new(Mutex::new(Connection::open_in_memory().unwrap()));
        let usage = Arc::new(Usage::default());
        let now = Utc::now();
        usage.record(CredentialTable::Key, "k", now);
        let locked = conn.lock().unwrap();
        let worker = {
            let conn = Arc::clone(&conn);
            let usage = Arc::clone(&usage);
            std::thread::spawn(move || usage.flush(&conn))
        };
        while usage.inflight.load(Ordering::Acquire) == 0 {
            std::thread::yield_now();
        }
        let newer = now + chrono::Duration::seconds(1);
        usage.record(CredentialTable::Key, "k", newer);
        drop(locked);
        assert!(worker.join().unwrap().is_err());
        assert_eq!(usage.pending.lock().unwrap()[&(CredentialTable::Key, "k".into())], newer);
    }

    #[test]
    fn test_usage_failed_flush_retains_observations() {
        let conn = Mutex::new(Connection::open_in_memory().unwrap());
        let usage = Usage::default();
        usage.record(CredentialTable::Key, "k", Utc::now());
        assert!(usage.flush(&conn).is_err());
        assert_eq!(usage.pending.lock().unwrap().len(), 1);
    }
}

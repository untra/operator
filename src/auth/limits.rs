use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

pub const EXCHANGES_PER_MINUTE: u32 = 60;
pub const REQUESTS_PER_KEY: u32 = 600;
pub const REQUESTS_PER_OWNER: u32 = 1200;
const WINDOW: Duration = Duration::from_mins(1);
const MAX_BUCKETS: usize = 10_000;

#[derive(Default)]
pub struct Limits(Mutex<LimitState>);

#[derive(Default)]
struct LimitState {
    entries: HashMap<String, (Instant, u32)>,
    last_cleanup: Option<Instant>,
}

impl Limits {
    pub fn check(&self, buckets: &[(String, u32)]) -> Result<(), u64> {
        self.check_at(buckets, Instant::now())
    }

    fn check_at(&self, buckets: &[(String, u32)], now: Instant) -> Result<(), u64> {
        let mut state = self.0.lock().map_err(|_| WINDOW.as_secs())?;
        if state
            .last_cleanup
            .is_none_or(|last| now.duration_since(last) >= WINDOW)
        {
            state
                .entries
                .retain(|_, (start, _)| now.duration_since(*start) < WINDOW);
            state.last_cleanup = Some(now);
        }
        let entries = &mut state.entries;
        for (id, _) in buckets {
            if entries
                .get(id)
                .is_some_and(|(start, _)| now.duration_since(*start) >= WINDOW)
            {
                entries.remove(id);
            }
        }
        let additional = buckets
            .iter()
            .filter(|(id, _)| !entries.contains_key(id))
            .count();
        if entries.len() + additional > MAX_BUCKETS {
            return Err(WINDOW.as_secs());
        }
        for (id, limit) in buckets {
            if let Some((start, used)) = entries.get(id) {
                if used >= limit {
                    return Err(WINDOW
                        .saturating_sub(now.duration_since(*start))
                        .as_secs()
                        .max(1));
                }
            }
        }
        for (id, _) in buckets {
            entries.entry(id.clone()).or_insert((now, 0)).1 += 1;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_limits_reserve_owner_and_key_together_and_reset() {
        let limits = Limits::default();
        let now = Instant::now();
        let first = [("key:a".into(), 1), ("owner:o".into(), 2)];
        let second = [("key:b".into(), 1), ("owner:o".into(), 2)];
        assert!(limits.check_at(&first, now).is_ok());
        assert!(limits.check_at(&first, now).is_err());
        assert!(limits.check_at(&second, now).is_ok());
        assert!(limits.check_at(&second, now).is_err());
        assert!(limits.check_at(&first, now + WINDOW).is_ok());
    }

    #[test]
    fn test_limits_capacity_does_not_evict_live_buckets() {
        let limits = Limits::default();
        let now = Instant::now();
        for index in 0..MAX_BUCKETS {
            assert!(limits.check_at(&[(index.to_string(), 1)], now).is_ok());
        }
        assert!(limits.check_at(&[("new".into(), 1)], now).is_err());
        assert!(limits.check_at(&[("0".into(), 1)], now).is_err());
        assert!(limits.check_at(&[("new".into(), 1)], now + WINDOW).is_ok());
    }
}

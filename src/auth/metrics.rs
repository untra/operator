use std::sync::atomic::{AtomicU64, Ordering};

use operator_telemetry::MetricName;

static COUNTERS: [AtomicU64; Counter::ALL.len()] =
    [const { AtomicU64::new(0) }; Counter::ALL.len()];

#[derive(Clone, Copy)]
pub enum Counter {
    AuthenticationDenied,
    AuthorizationDenied,
    RateLimited,
    UsageDropped,
    UsageFlushFailed,
}

impl Counter {
    pub const ALL: [Self; 5] = [
        Self::AuthenticationDenied,
        Self::AuthorizationDenied,
        Self::RateLimited,
        Self::UsageDropped,
        Self::UsageFlushFailed,
    ];

    pub fn increment(self) {
        COUNTERS[self as usize].fetch_add(1, Ordering::Relaxed);
    }

    pub fn value(self) -> u64 {
        COUNTERS[self as usize].load(Ordering::Relaxed)
    }

    pub fn name(self) -> MetricName {
        match self {
            Self::AuthenticationDenied => MetricName::OperatorAuthAuthenticationDenied,
            Self::AuthorizationDenied => MetricName::OperatorAuthAuthorizationDenied,
            Self::RateLimited => MetricName::OperatorAuthRateLimited,
            Self::UsageDropped => MetricName::OperatorAuthUsageDropped,
            Self::UsageFlushFailed => MetricName::OperatorAuthUsageFlushFailed,
        }
    }
}

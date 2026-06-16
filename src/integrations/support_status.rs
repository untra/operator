//! Official support / maturity level for advertised integrations.
//!
//! [`SupportStatus`] is the single, low-level designation attached to every
//! entry in the vertical catalog ([`crate::integrations::catalog`]). It is the
//! canonical DTO for "how supported is X" — every surface (the REST
//! `/api/v1/integrations` endpoint, the generated TypeScript bindings, the
//! JSON-Schema, and the generated `docs/maturity/` page) derives its notion of
//! maturity from here, so the four surfaces can't drift.
//!
//! Variants are ordered `Proto < Alpha < Beta < Ga` so callers can gate on
//! maturity: today that drives docs surfacing and the README-badge parity test;
//! later it is the hook for entitlement control.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;

/// Official support / maturity level of an advertised integration.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
    JsonSchema,
    ToSchema,
    TS,
)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum SupportStatus {
    /// Experimental — wired in code with no guarantees. Not publicly advertised
    /// (no README badge); docs optional.
    Proto,
    /// Usable, but expect breaking change. Advertised with caveats.
    Alpha,
    /// Stable-ish and hardening toward general availability.
    Beta,
    /// Generally available.
    Ga,
}

impl SupportStatus {
    /// All levels, ascending by maturity.
    pub const ALL: [SupportStatus; 4] = [
        SupportStatus::Proto,
        SupportStatus::Alpha,
        SupportStatus::Beta,
        SupportStatus::Ga,
    ];

    /// Display label used in docs and badges (`GA`, not `Ga`).
    pub fn label(&self) -> &'static str {
        match self {
            SupportStatus::Proto => "Proto",
            SupportStatus::Alpha => "Alpha",
            SupportStatus::Beta => "Beta",
            SupportStatus::Ga => "GA",
        }
    }

    /// Lowercase wire slug (matches the serde representation).
    pub fn slug(&self) -> &'static str {
        match self {
            SupportStatus::Proto => "proto",
            SupportStatus::Alpha => "alpha",
            SupportStatus::Beta => "beta",
            SupportStatus::Ga => "ga",
        }
    }

    /// Hex color (no `#`) for the shields.io status badge on the maturity page.
    /// Chosen to read as a maturity ramp: neutral gray → cornflower → amber →
    /// green, consistent with the brand tokens in `docs/assets/css/tokens.css`.
    pub fn badge_color(&self) -> &'static str {
        match self {
            SupportStatus::Proto => "6B7280", // neutral gray
            SupportStatus::Alpha => "6495ED", // cornflower
            SupportStatus::Beta => "E8A33D",  // amber
            SupportStatus::Ga => "1BB91F",    // green
        }
    }

    /// One-line explanation shown in the maturity-page legend.
    pub fn blurb(&self) -> &'static str {
        match self {
            SupportStatus::Proto => {
                "Experimental — present in code with no guarantees. Not advertised yet."
            }
            SupportStatus::Alpha => "Usable, but expect breaking changes. Advertised with caveats.",
            SupportStatus::Beta => "Stable-ish and hardening toward general availability.",
            SupportStatus::Ga => "Generally available and supported.",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_support_status_ordering() {
        assert!(SupportStatus::Proto < SupportStatus::Alpha);
        assert!(SupportStatus::Alpha < SupportStatus::Beta);
        assert!(SupportStatus::Beta < SupportStatus::Ga);
    }

    #[test]
    fn test_support_status_serde_lowercase() {
        let json = serde_json::to_string(&SupportStatus::Ga).unwrap();
        assert_eq!(json, "\"ga\"");
        let parsed: SupportStatus = serde_json::from_str("\"beta\"").unwrap();
        assert_eq!(parsed, SupportStatus::Beta);
    }

    #[test]
    fn test_support_status_all_covers_four_levels() {
        assert_eq!(SupportStatus::ALL.len(), 4);
    }

    #[test]
    fn test_support_status_label_ga_uppercase() {
        assert_eq!(SupportStatus::Ga.label(), "GA");
    }
}

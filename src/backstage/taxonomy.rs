//! Project taxonomy types and loader.
//!
//! The taxonomy is defined in `taxonomy.toml` (single source of truth) and
//! loaded at compile time via `include_str!`. This ensures:
//! - Documentation stays synchronized with code
//! - Compile-time validation of taxonomy structure
//! - Zero runtime I/O for taxonomy access
//!
//! Note: Types are marked #[allow(dead_code)] during Milestone 1 as they will
//! be used in subsequent milestones (ASSESS issue type, project analysis, etc.)

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The complete project taxonomy, loaded from taxonomy.toml
#[allow(dead_code)]
static TAXONOMY: Lazy<Taxonomy> = Lazy::new(|| {
    toml::from_str(include_str!("taxonomy.toml")).expect("taxonomy.toml must be valid TOML")
});

/// Kind tier classification
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum KindTier {
    Foundation,
    Standards,
    Engines,
    Ecosystem,
}

#[allow(dead_code)]
impl KindTier {
    /// Returns all tiers in order
    pub fn all() -> &'static [KindTier] {
        &[
            KindTier::Foundation,
            KindTier::Standards,
            KindTier::Engines,
            KindTier::Ecosystem,
        ]
    }

    /// Returns the tier key string
    pub fn as_str(&self) -> &'static str {
        match self {
            KindTier::Foundation => "foundation",
            KindTier::Standards => "standards",
            KindTier::Engines => "engines",
            KindTier::Ecosystem => "ecosystem",
        }
    }

    /// Parse tier from string key
    pub fn from_key(key: &str) -> Option<KindTier> {
        match key.to_lowercase().as_str() {
            "foundation" => Some(KindTier::Foundation),
            "standards" => Some(KindTier::Standards),
            "engines" => Some(KindTier::Engines),
            "ecosystem" => Some(KindTier::Ecosystem),
            _ => None,
        }
    }
}

impl std::fmt::Display for KindTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            KindTier::Foundation => "Foundation",
            KindTier::Standards => "Standards",
            KindTier::Engines => "Engines",
            KindTier::Ecosystem => "Ecosystem",
        };
        write!(f, "{}", name)
    }
}

/// Metadata about the taxonomy
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxonomyMeta {
    pub version: String,
    pub description: String,
}

/// A tier grouping for Kinds
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tier {
    pub id: u8,
    pub key: String,
    pub name: String,
    pub range: (u8, u8),
    pub description: String,
    /// Icon identifier for sidebar navigation
    #[serde(default)]
    pub icon: Option<String>,
    /// Display order in sidebar (falls back to id if not set)
    #[serde(default)]
    pub display_order: Option<u8>,
    /// Optional override label for sidebar display
    #[serde(default)]
    pub sidebar_label: Option<String>,
}

#[allow(dead_code)]
impl Tier {
    /// Returns the tier enum variant
    pub fn tier(&self) -> Option<KindTier> {
        KindTier::from_key(&self.key)
    }

    /// Check if a Kind ID falls within this tier's range
    pub fn contains_id(&self, id: u8) -> bool {
        id >= self.range.0 && id <= self.range.1
    }

    /// Get sidebar label (falls back to name)
    pub fn sidebar_label(&self) -> &str {
        self.sidebar_label.as_deref().unwrap_or(&self.name)
    }

    /// Get display order (falls back to id)
    pub fn display_order(&self) -> u8 {
        self.display_order.unwrap_or(self.id)
    }

    /// Get icon (returns None if not set)
    pub fn icon(&self) -> Option<&str> {
        self.icon.as_deref()
    }
}

/// A project Kind definition
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Kind {
    pub id: u8,
    pub key: String,
    pub name: String,
    pub tier: String,
    pub description: String,
    pub stakeholder: String,
    pub output: String,
    pub file_patterns: Vec<String>,
    pub backstage_type: String,
    /// Icon identifier (optional, falls back to tier icon)
    #[serde(default)]
    pub icon: Option<String>,
    /// Display order within tier (falls back to id if not set)
    #[serde(default)]
    pub display_order: Option<u8>,
}

#[allow(dead_code)]
impl Kind {
    /// Returns the tier enum variant for this Kind
    pub fn tier_enum(&self) -> Option<KindTier> {
        KindTier::from_key(&self.tier)
    }

    /// Check if a file path matches any of this Kind's patterns
    pub fn matches_pattern(&self, path: &str) -> bool {
        use glob::Pattern;
        self.file_patterns.iter().any(|pattern| {
            Pattern::new(pattern)
                .map(|p| p.matches(path))
                .unwrap_or(false)
        })
    }

    /// Get display order within tier (falls back to id)
    pub fn display_order(&self) -> u8 {
        self.display_order.unwrap_or(self.id)
    }

    /// Get icon (returns None if not set)
    pub fn icon(&self) -> Option<&str> {
        self.icon.as_deref()
    }
}

/// The complete taxonomy structure
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Taxonomy {
    pub meta: TaxonomyMeta,
    pub tiers: Vec<Tier>,
    pub kinds: Vec<Kind>,
}

#[allow(dead_code)]
impl Taxonomy {
    /// Get the global taxonomy instance (loaded at first access)
    pub fn load() -> &'static Taxonomy {
        &TAXONOMY
    }

    /// Get all Kinds in a specific tier
    pub fn kinds_by_tier(&self, tier: KindTier) -> Vec<&Kind> {
        let tier_key = tier.as_str();
        self.kinds.iter().filter(|k| k.tier == tier_key).collect()
    }

    /// Get a Kind by its key
    pub fn kind_by_key(&self, key: &str) -> Option<&Kind> {
        self.kinds.iter().find(|k| k.key == key)
    }

    /// Get a Kind by its ID
    pub fn kind_by_id(&self, id: u8) -> Option<&Kind> {
        self.kinds.iter().find(|k| k.id == id)
    }

    /// Get the tier definition for a tier enum
    pub fn tier_def(&self, tier: KindTier) -> Option<&Tier> {
        let tier_key = tier.as_str();
        self.tiers.iter().find(|t| t.key == tier_key)
    }

    /// Get all Kind keys
    pub fn all_kind_keys(&self) -> Vec<&str> {
        self.kinds.iter().map(|k| k.key.as_str()).collect()
    }

    /// Detect the most likely Kind for a set of file paths
    ///
    /// Returns the Kind with the highest match count, along with the count.
    pub fn detect_kind(&self, file_paths: &[&str]) -> Option<(&Kind, usize)> {
        let mut scores: HashMap<&str, usize> = HashMap::new();

        for path in file_paths {
            for kind in &self.kinds {
                if kind.matches_pattern(path) {
                    *scores.entry(&kind.key).or_insert(0) += 1;
                }
            }
        }

        scores
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .and_then(|(key, count)| self.kind_by_key(key).map(|k| (k, count)))
    }

    /// Get all Kinds that match at least one file path
    pub fn matching_kinds(&self, file_paths: &[&str]) -> Vec<(&Kind, usize)> {
        let mut scores: HashMap<&str, usize> = HashMap::new();

        for path in file_paths {
            for kind in &self.kinds {
                if kind.matches_pattern(path) {
                    *scores.entry(&kind.key).or_insert(0) += 1;
                }
            }
        }

        let mut results: Vec<_> = scores
            .into_iter()
            .filter_map(|(key, count)| self.kind_by_key(key).map(|k| (k, count)))
            .collect();

        results.sort_by(|a, b| b.1.cmp(&a.1)); // Sort by count descending
        results
    }

    /// Get tiers sorted by display order
    pub fn tiers_by_display_order(&self) -> Vec<&Tier> {
        let mut tiers: Vec<_> = self.tiers.iter().collect();
        tiers.sort_by_key(|t| t.display_order());
        tiers
    }

    /// Get kinds in a tier sorted by display order
    pub fn kinds_by_tier_ordered(&self, tier: KindTier) -> Vec<&Kind> {
        let mut kinds = self.kinds_by_tier(tier);
        kinds.sort_by_key(|k| k.display_order());
        kinds
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_taxonomy_loads_successfully() {
        let t = Taxonomy::load();
        assert!(!t.kinds.is_empty(), "Taxonomy should have kinds");
        assert!(!t.tiers.is_empty(), "Taxonomy should have tiers");
    }

    #[test]
    fn test_exactly_24_kinds() {
        let t = Taxonomy::load();
        assert_eq!(
            t.kinds.len(),
            24,
            "Taxonomy must have exactly 24 kinds, found {}",
            t.kinds.len()
        );
    }

    #[test]
    fn test_exactly_4_tiers() {
        let t = Taxonomy::load();
        assert_eq!(
            t.tiers.len(),
            4,
            "Taxonomy must have exactly 4 tiers, found {}",
            t.tiers.len()
        );
    }

    #[test]
    fn test_kind_ids_unique_and_sequential() {
        let t = Taxonomy::load();
        let ids: Vec<u8> = t.kinds.iter().map(|k| k.id).collect();
        let expected: Vec<u8> = (1..=24).collect();
        assert_eq!(
            ids, expected,
            "Kind IDs must be sequential from 1 to 24, got {:?}",
            ids
        );
    }

    #[test]
    fn test_kind_keys_unique() {
        let t = Taxonomy::load();
        let keys: Vec<&str> = t.kinds.iter().map(|k| k.key.as_str()).collect();
        let unique: HashSet<_> = keys.iter().collect();
        assert_eq!(
            keys.len(),
            unique.len(),
            "Kind keys must be unique, found duplicates"
        );
    }

    #[test]
    fn test_all_kinds_have_valid_tier() {
        let t = Taxonomy::load();
        let tier_keys: HashSet<_> = t.tiers.iter().map(|t| t.key.as_str()).collect();
        for kind in &t.kinds {
            assert!(
                tier_keys.contains(kind.tier.as_str()),
                "Kind {} has invalid tier: {}",
                kind.key,
                kind.tier
            );
        }
    }

    #[test]
    fn test_kind_ids_within_tier_ranges() {
        let t = Taxonomy::load();
        for kind in &t.kinds {
            let tier = t.tiers.iter().find(|t| t.key == kind.tier);
            assert!(
                tier.is_some(),
                "Kind {} references unknown tier: {}",
                kind.key,
                kind.tier
            );
            let tier = tier.unwrap();
            assert!(
                tier.contains_id(kind.id),
                "Kind {} (id={}) is outside tier {} range {:?}",
                kind.key,
                kind.id,
                tier.key,
                tier.range
            );
        }
    }

    #[test]
    fn test_all_kinds_have_file_patterns() {
        let t = Taxonomy::load();
        for kind in &t.kinds {
            assert!(
                !kind.file_patterns.is_empty(),
                "Kind {} must have at least one file pattern",
                kind.key
            );
        }
    }

    #[test]
    fn test_all_kinds_have_backstage_type() {
        let t = Taxonomy::load();
        let valid_types = [
            "service",
            "website",
            "library",
            "api",
            "resource",
            "system",
            "template",
            "documentation",
            "tool",
        ];
        for kind in &t.kinds {
            assert!(
                valid_types.contains(&kind.backstage_type.as_str()),
                "Kind {} has invalid backstage_type: {}. Valid types: {:?}",
                kind.key,
                kind.backstage_type,
                valid_types
            );
        }
    }

    #[test]
    fn test_tier_ranges_cover_all_ids() {
        let t = Taxonomy::load();
        for id in 1..=24u8 {
            let tier = t.tiers.iter().find(|t| t.contains_id(id));
            assert!(tier.is_some(), "ID {} is not covered by any tier range", id);
        }
    }

    #[test]
    fn test_tier_ranges_no_overlap() {
        let t = Taxonomy::load();
        for (i, tier_a) in t.tiers.iter().enumerate() {
            for tier_b in t.tiers.iter().skip(i + 1) {
                let overlap = (tier_a.range.0..=tier_a.range.1)
                    .any(|id| tier_b.range.0 <= id && id <= tier_b.range.1);
                assert!(
                    !overlap,
                    "Tier {} ({:?}) overlaps with tier {} ({:?})",
                    tier_a.key, tier_a.range, tier_b.key, tier_b.range
                );
            }
        }
    }

    #[test]
    fn test_kind_by_key() {
        let t = Taxonomy::load();
        let kind = t.kind_by_key("microservice");
        assert!(kind.is_some(), "Should find 'microservice' kind");
        assert_eq!(kind.unwrap().id, 13);
    }

    #[test]
    fn test_kind_by_id() {
        let t = Taxonomy::load();
        let kind = t.kind_by_id(15);
        assert!(kind.is_some(), "Should find kind with id 15");
        assert_eq!(kind.unwrap().key, "ui-frontend");
    }

    #[test]
    fn test_kinds_by_tier() {
        let t = Taxonomy::load();
        let foundation_kinds = t.kinds_by_tier(KindTier::Foundation);
        assert_eq!(
            foundation_kinds.len(),
            4,
            "Foundation tier should have 4 kinds"
        );

        let standards_kinds = t.kinds_by_tier(KindTier::Standards);
        assert_eq!(
            standards_kinds.len(),
            6,
            "Standards tier should have 6 kinds"
        );

        let engines_kinds = t.kinds_by_tier(KindTier::Engines);
        assert_eq!(engines_kinds.len(), 6, "Engines tier should have 6 kinds");

        let ecosystem_kinds = t.kinds_by_tier(KindTier::Ecosystem);
        assert_eq!(
            ecosystem_kinds.len(),
            8,
            "Ecosystem tier should have 8 kinds"
        );
    }

    #[test]
    fn test_kind_matches_pattern() {
        let t = Taxonomy::load();
        let infrastructure = t.kind_by_key("infrastructure").unwrap();
        assert!(infrastructure.matches_pattern("main.tf"));
        assert!(infrastructure.matches_pattern("terraform.tfvars"));
        assert!(!infrastructure.matches_pattern("src/main.rs"));
    }

    #[test]
    fn test_detect_kind() {
        let t = Taxonomy::load();
        let rust_service_files = &["Cargo.toml", "src/main.rs", "Dockerfile"];
        let detected = t.detect_kind(rust_service_files);
        assert!(
            detected.is_some(),
            "Should detect a kind for Rust service files"
        );
        // microservice should match src/main.rs and Dockerfile
        let (kind, _) = detected.unwrap();
        assert_eq!(kind.key, "microservice");
    }

    #[test]
    fn test_detect_kind_infrastructure() {
        let t = Taxonomy::load();
        let terraform_files = &["main.tf", "variables.tf", "outputs.tf", "terraform.tfvars"];
        let detected = t.detect_kind(terraform_files);
        assert!(detected.is_some(), "Should detect infrastructure kind");
        let (kind, count) = detected.unwrap();
        assert_eq!(kind.key, "infrastructure");
        assert!(count >= 3, "Should match multiple terraform files");
    }

    #[test]
    fn test_kind_tier_enum() {
        let t = Taxonomy::load();
        let infra = t.kind_by_key("infrastructure").unwrap();
        assert_eq!(infra.tier_enum(), Some(KindTier::Foundation));

        let ml = t.kind_by_key("ml-model").unwrap();
        assert_eq!(ml.tier_enum(), Some(KindTier::Engines));
    }

    #[test]
    fn test_tier_display_order_fallback() {
        let t = Taxonomy::load();
        for tier in &t.tiers {
            // display_order should fall back to id if not set
            let order = tier.display_order();
            assert!(
                order >= 1 && order <= 4,
                "Tier {} display_order {} should be 1-4",
                tier.key,
                order
            );
        }
    }

    #[test]
    fn test_tier_sidebar_label_fallback() {
        let t = Taxonomy::load();
        for tier in &t.tiers {
            // sidebar_label should fall back to name if not set
            let label = tier.sidebar_label();
            assert!(
                !label.is_empty(),
                "Tier {} should have a sidebar label",
                tier.key
            );
        }
    }

    #[test]
    fn test_tiers_by_display_order() {
        let t = Taxonomy::load();
        let ordered = t.tiers_by_display_order();
        assert_eq!(ordered.len(), 4);
        // First tier should be Foundation (display_order 1 or id 1)
        assert_eq!(ordered[0].key, "foundation");
    }

    #[test]
    fn test_kinds_by_tier_ordered() {
        let t = Taxonomy::load();
        let foundation_kinds = t.kinds_by_tier_ordered(KindTier::Foundation);
        assert_eq!(foundation_kinds.len(), 4, "Foundation should have 4 kinds");
        // Kinds should be sorted by display_order (falls back to id)
        for i in 1..foundation_kinds.len() {
            assert!(
                foundation_kinds[i - 1].display_order() <= foundation_kinds[i].display_order(),
                "Kinds should be sorted by display_order"
            );
        }
    }

    #[test]
    fn test_kind_display_order_fallback() {
        let t = Taxonomy::load();
        for kind in &t.kinds {
            // display_order should fall back to id if not set
            let order = kind.display_order();
            assert!(
                order >= 1 && order <= 24,
                "Kind {} display_order {} should be 1-24",
                kind.key,
                order
            );
        }
    }
}

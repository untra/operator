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
    Noncurrent,
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
            KindTier::Noncurrent,
        ]
    }

    /// Returns the tier key string
    pub fn as_str(&self) -> &'static str {
        match self {
            KindTier::Foundation => "foundation",
            KindTier::Standards => "standards",
            KindTier::Engines => "engines",
            KindTier::Ecosystem => "ecosystem",
            KindTier::Noncurrent => "noncurrent",
        }
    }

    /// Parse tier from string key
    pub fn from_key(key: &str) -> Option<KindTier> {
        match key.to_lowercase().as_str() {
            "foundation" => Some(KindTier::Foundation),
            "standards" => Some(KindTier::Standards),
            "engines" => Some(KindTier::Engines),
            "ecosystem" => Some(KindTier::Ecosystem),
            "noncurrent" => Some(KindTier::Noncurrent),
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
            KindTier::Noncurrent => "Noncurrent",
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
    /// Optional ID range hint for documentation (not enforced - use Kind.tier field)
    #[serde(default)]
    pub range: Option<(u8, u8)>,
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
    /// Whether to assess for frameworks in this tier
    #[serde(default)]
    pub assess_frameworks: bool,
    /// Whether to assess for databases in this tier
    #[serde(default)]
    pub assess_databases: bool,
    /// Whether to assess for testing frameworks in this tier
    #[serde(default)]
    pub assess_testing: bool,
}

#[allow(dead_code)]
impl Tier {
    /// Returns the tier enum variant
    pub fn tier(&self) -> Option<KindTier> {
        KindTier::from_key(&self.key)
    }

    /// Check if a Kind ID falls within this tier's range hint.
    /// Note: This is for documentation only. The Kind.tier field is the source of truth.
    #[deprecated(note = "Use Kind.tier field instead - range is only a documentation hint")]
    pub fn contains_id(&self, id: u8) -> bool {
        self.range
            .map(|(start, end)| id >= start && id <= end)
            .unwrap_or(false)
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

    /// Whether framework detection should run for this tier
    pub fn should_assess_frameworks(&self) -> bool {
        self.assess_frameworks
    }

    /// Whether database detection should run for this tier
    pub fn should_assess_databases(&self) -> bool {
        self.assess_databases
    }

    /// Whether testing framework detection should run for this tier
    pub fn should_assess_testing(&self) -> bool {
        self.assess_testing
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
    fn test_kind_ids_unique() {
        let t = Taxonomy::load();
        let ids: HashSet<u8> = t.kinds.iter().map(|k| k.id).collect();
        assert_eq!(ids.len(), t.kinds.len(), "Kind IDs must be unique");
    }

    #[test]
    fn test_tier_ids_unique() {
        let t = Taxonomy::load();
        let ids: HashSet<u8> = t.tiers.iter().map(|t| t.id).collect();
        assert_eq!(ids.len(), t.tiers.len(), "Tier IDs must be unique");
    }

    #[test]
    fn test_each_tier_has_at_least_one_kind() {
        let t = Taxonomy::load();
        for tier_enum in KindTier::all() {
            let kinds = t.kinds_by_tier(*tier_enum);
            assert!(
                !kinds.is_empty(),
                "{} tier should have at least one kind",
                tier_enum
            );
        }
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
        // Verify each tier returns kinds and they all reference the correct tier
        for tier_enum in KindTier::all() {
            let kinds = t.kinds_by_tier(*tier_enum);
            assert!(!kinds.is_empty(), "{} tier should have kinds", tier_enum);
            for kind in &kinds {
                assert_eq!(
                    kind.tier,
                    tier_enum.as_str(),
                    "Kind {} should be in {} tier",
                    kind.key,
                    tier_enum
                );
            }
        }
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
                order >= 1,
                "Tier {} display_order {} should be positive",
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
        assert!(!ordered.is_empty(), "Should have tiers");
        // Verify tiers are sorted by display_order
        for i in 1..ordered.len() {
            assert!(
                ordered[i - 1].display_order() <= ordered[i].display_order(),
                "Tiers should be sorted by display_order"
            );
        }
    }

    #[test]
    fn test_kinds_by_tier_ordered() {
        let t = Taxonomy::load();
        // Test ordering for each tier
        for tier_enum in KindTier::all() {
            let kinds = t.kinds_by_tier_ordered(*tier_enum);
            // Kinds should be sorted by display_order (falls back to id)
            for i in 1..kinds.len() {
                assert!(
                    kinds[i - 1].display_order() <= kinds[i].display_order(),
                    "Kinds in {} should be sorted by display_order",
                    tier_enum
                );
            }
        }
    }

    #[test]
    fn test_kind_display_order_fallback() {
        let t = Taxonomy::load();
        for kind in &t.kinds {
            // display_order should fall back to id if not set
            let order = kind.display_order();
            assert!(
                order >= 1,
                "Kind {} display_order {} should be positive",
                kind.key,
                order
            );
        }
    }

    #[test]
    fn test_tier_assessment_scope_flags() {
        let t = Taxonomy::load();

        // Foundation: no assessments
        let foundation = t.tier_def(KindTier::Foundation).unwrap();
        assert!(!foundation.should_assess_frameworks());
        assert!(!foundation.should_assess_databases());
        assert!(!foundation.should_assess_testing());

        // Standards: frameworks and testing only
        let standards = t.tier_def(KindTier::Standards).unwrap();
        assert!(standards.should_assess_frameworks());
        assert!(!standards.should_assess_databases());
        assert!(standards.should_assess_testing());

        // Engines: all assessments
        let engines = t.tier_def(KindTier::Engines).unwrap();
        assert!(engines.should_assess_frameworks());
        assert!(engines.should_assess_databases());
        assert!(engines.should_assess_testing());

        // Ecosystem: all assessments
        let ecosystem = t.tier_def(KindTier::Ecosystem).unwrap();
        assert!(ecosystem.should_assess_frameworks());
        assert!(ecosystem.should_assess_databases());
        assert!(ecosystem.should_assess_testing());

        // Noncurrent: no assessments
        let noncurrent = t.tier_def(KindTier::Noncurrent).unwrap();
        assert!(!noncurrent.should_assess_frameworks());
        assert!(!noncurrent.should_assess_databases());
        assert!(!noncurrent.should_assess_testing());
    }

    #[test]
    fn test_noncurrent_tier_kinds() {
        let t = Taxonomy::load();
        let noncurrent_kinds = t.kinds_by_tier(KindTier::Noncurrent);

        let keys: Vec<&str> = noncurrent_kinds.iter().map(|k| k.key.as_str()).collect();
        assert!(keys.contains(&"reference-example"));
        assert!(keys.contains(&"experiment-sandbox"));
        assert!(keys.contains(&"archival-fork"));
        assert!(keys.contains(&"test-data-fixtures"));
    }

    #[test]
    fn test_test_data_fixtures_kind() {
        let t = Taxonomy::load();
        let kind = t.kind_by_key("test-data-fixtures");
        assert!(kind.is_some(), "Should find 'test-data-fixtures' kind");

        let kind = kind.unwrap();
        assert_eq!(kind.tier, "noncurrent");
        assert_eq!(kind.backstage_type, "resource");
        assert!(kind.matches_pattern("fixtures/users.json"));
        assert!(kind.matches_pattern("testdata/sample.csv"));
        assert!(kind.matches_pattern("db/seeds/users.sql"));
    }
}

//! Artifact detection for positive step-completion signals.
//!
//! When a step declares `artifact_patterns`, this module checks whether
//! expected output files exist in the agent's worktree. This supplements
//! the existing negative-signal detection (idle/silence) with a positive
//! confirmation that the agent actually produced output.

use std::collections::HashMap;
use std::path::Path;
use std::time::{Duration, Instant};

use glob::glob;

/// Whether all expected artifacts for a step have been found.
#[derive(Debug, Clone, PartialEq)]
pub enum ArtifactStatus {
    /// Not all artifact patterns matched yet
    Pending,
    /// All artifact patterns matched at least one file
    Ready,
}

/// Checks for expected output files in an agent's worktree.
pub struct ArtifactDetector {
    cache: HashMap<String, (ArtifactStatus, Instant)>,
    cache_ttl: Duration,
}

impl Default for ArtifactDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl ArtifactDetector {
    /// Create a new detector with a 2-second cache TTL (matching agtx).
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            cache_ttl: Duration::from_secs(2),
        }
    }

    /// Check if all artifact patterns match at least one file in the worktree.
    ///
    /// Empty patterns list returns `Ready` (nothing to check).
    /// Results are cached for `cache_ttl` keyed on worktree + patterns.
    pub fn check_artifacts(&mut self, worktree_path: &Path, patterns: &[String]) -> ArtifactStatus {
        if patterns.is_empty() {
            return ArtifactStatus::Ready;
        }

        let cache_key = Self::cache_key(worktree_path, patterns);

        // Check cache
        if let Some((status, cached_at)) = self.cache.get(&cache_key) {
            if cached_at.elapsed() < self.cache_ttl {
                return status.clone();
            }
        }

        // Evaluate each pattern
        let status = if patterns
            .iter()
            .all(|p| Self::resolve_pattern(worktree_path, p))
        {
            ArtifactStatus::Ready
        } else {
            ArtifactStatus::Pending
        };

        self.cache
            .insert(cache_key, (status.clone(), Instant::now()));
        status
    }

    /// Convenience wrapper: check artifacts given a worktree path string.
    pub fn poll_agent_artifacts(
        &mut self,
        worktree_path: &str,
        patterns: &[String],
    ) -> ArtifactStatus {
        self.check_artifacts(Path::new(worktree_path), patterns)
    }

    /// Check if a single glob pattern matches at least one file relative to `base`.
    fn resolve_pattern(base: &Path, pattern: &str) -> bool {
        let full_pattern = base.join(pattern).to_string_lossy().to_string();
        match glob(&full_pattern) {
            Ok(mut paths) => paths.find_map(Result::ok).is_some(),
            Err(_) => false,
        }
    }

    /// Build a cache key from worktree path and patterns.
    fn cache_key(worktree_path: &Path, patterns: &[String]) -> String {
        format!("{}:{}", worktree_path.display(), patterns.join(","))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_empty_patterns_returns_ready() {
        let mut detector = ArtifactDetector::new();
        let tmp = TempDir::new().unwrap();
        let status = detector.check_artifacts(tmp.path(), &[]);
        assert_eq!(status, ArtifactStatus::Ready);
    }

    #[test]
    fn test_single_file_exists_returns_ready() {
        let mut detector = ArtifactDetector::new();
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("plan.md"), "# Plan").unwrap();

        let status = detector.check_artifacts(tmp.path(), &["plan.md".to_string()]);
        assert_eq!(status, ArtifactStatus::Ready);
    }

    #[test]
    fn test_single_file_missing_returns_pending() {
        let mut detector = ArtifactDetector::new();
        let tmp = TempDir::new().unwrap();

        let status = detector.check_artifacts(tmp.path(), &["plan.md".to_string()]);
        assert_eq!(status, ArtifactStatus::Pending);
    }

    #[test]
    fn test_multiple_patterns_all_exist_returns_ready() {
        let mut detector = ArtifactDetector::new();
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("plan.md"), "# Plan").unwrap();
        fs::write(tmp.path().join("notes.txt"), "Notes").unwrap();

        let patterns = vec!["plan.md".to_string(), "notes.txt".to_string()];
        let status = detector.check_artifacts(tmp.path(), &patterns);
        assert_eq!(status, ArtifactStatus::Ready);
    }

    #[test]
    fn test_multiple_patterns_partial_returns_pending() {
        let mut detector = ArtifactDetector::new();
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("plan.md"), "# Plan").unwrap();
        // notes.txt missing

        let patterns = vec!["plan.md".to_string(), "notes.txt".to_string()];
        let status = detector.check_artifacts(tmp.path(), &patterns);
        assert_eq!(status, ArtifactStatus::Pending);
    }

    #[test]
    fn test_wildcard_pattern_matches() {
        let mut detector = ArtifactDetector::new();
        let tmp = TempDir::new().unwrap();
        let plans_dir = tmp.path().join("plans");
        fs::create_dir_all(&plans_dir).unwrap();
        fs::write(plans_dir.join("FEAT-123.md"), "# Plan").unwrap();

        let status = detector.check_artifacts(tmp.path(), &["plans/*.md".to_string()]);
        assert_eq!(status, ArtifactStatus::Ready);
    }

    #[test]
    fn test_wildcard_pattern_no_match_returns_pending() {
        let mut detector = ArtifactDetector::new();
        let tmp = TempDir::new().unwrap();
        let plans_dir = tmp.path().join("plans");
        fs::create_dir_all(&plans_dir).unwrap();
        // No .md files in plans/

        let status = detector.check_artifacts(tmp.path(), &["plans/*.md".to_string()]);
        assert_eq!(status, ArtifactStatus::Pending);
    }

    #[test]
    fn test_cache_returns_same_result_within_ttl() {
        let mut detector = ArtifactDetector::new();
        let tmp = TempDir::new().unwrap();

        // First call: file missing → Pending
        let patterns = vec!["plan.md".to_string()];
        let status1 = detector.check_artifacts(tmp.path(), &patterns);
        assert_eq!(status1, ArtifactStatus::Pending);

        // Create the file
        fs::write(tmp.path().join("plan.md"), "# Plan").unwrap();

        // Second call within TTL: should still be Pending (cached)
        let status2 = detector.check_artifacts(tmp.path(), &patterns);
        assert_eq!(status2, ArtifactStatus::Pending);
    }

    #[test]
    fn test_cache_expires_after_ttl() {
        let mut detector = ArtifactDetector::new();
        // Use a very short TTL for testing
        detector.cache_ttl = Duration::from_millis(10);

        let tmp = TempDir::new().unwrap();

        // First call: file missing → Pending
        let patterns = vec!["plan.md".to_string()];
        let status1 = detector.check_artifacts(tmp.path(), &patterns);
        assert_eq!(status1, ArtifactStatus::Pending);

        // Create the file
        fs::write(tmp.path().join("plan.md"), "# Plan").unwrap();

        // Wait for cache to expire
        std::thread::sleep(Duration::from_millis(20));

        // Third call: cache expired, should re-check → Ready
        let status3 = detector.check_artifacts(tmp.path(), &patterns);
        assert_eq!(status3, ArtifactStatus::Ready);
    }

    #[test]
    fn test_poll_agent_artifacts_convenience() {
        let mut detector = ArtifactDetector::new();
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("output.json"), "{}").unwrap();

        let status = detector
            .poll_agent_artifacts(tmp.path().to_str().unwrap(), &["output.json".to_string()]);
        assert_eq!(status, ArtifactStatus::Ready);
    }

    #[test]
    fn test_nested_directory_pattern() {
        let mut detector = ArtifactDetector::new();
        let tmp = TempDir::new().unwrap();
        let nested = tmp.path().join(".tickets").join("plans");
        fs::create_dir_all(&nested).unwrap();
        fs::write(nested.join("FEAT-001.md"), "# Plan").unwrap();

        let status = detector.check_artifacts(tmp.path(), &[".tickets/plans/*.md".to_string()]);
        assert_eq!(status, ArtifactStatus::Ready);
    }
}

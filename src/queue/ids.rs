//! Ticket id allocation and filename slugs.

use std::path::{Path, PathBuf};

/// Next sequential id for `type_key`: one past the highest `TYPE-NNNN` in `existing`.
pub fn next_ticket_id<'a>(type_key: &str, existing: impl IntoIterator<Item = &'a str>) -> String {
    let prefix = format!("{type_key}-");
    let max = existing
        .into_iter()
        .filter_map(|id| id.strip_prefix(&prefix)?.parse::<u32>().ok())
        .max()
        .unwrap_or(0);
    format!("{type_key}-{:04}", max + 1)
}

/// Lowercase ASCII slug of `s`, dash-separated, cut at a word boundary within `max_len`.
pub fn slugify(s: &str, max_len: usize) -> String {
    let slug = s
        .to_ascii_lowercase()
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|w| !w.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    if slug.len() <= max_len {
        return slug;
    }
    let truncated = &slug[..max_len];
    truncated
        .rfind('-')
        .map_or(truncated, |dash| &truncated[..dash])
        .to_string()
}

/// First path `{stem}.md`, `{stem}-2.md`, ... that does not exist in `dir`.
pub fn unique_path(dir: &Path, stem: &str) -> PathBuf {
    (1..=u32::MAX)
        .map(|n| match n {
            1 => dir.join(format!("{stem}.md")),
            n => dir.join(format!("{stem}-{n}.md")),
        })
        .find(|p| !p.exists())
        .unwrap_or_else(|| dir.join(format!("{stem}.md")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_next_ticket_id_starts_at_one() {
        assert_eq!(next_ticket_id("FEAT", []), "FEAT-0001");
    }

    #[test]
    fn test_next_ticket_id_continues_past_max_and_ignores_other_types() {
        let existing = [
            "FEAT-0007",
            "FEAT-0002",
            "FIX-0099",
            "FEAT-PROJ12",
            "FEATURE-0050",
        ];
        assert_eq!(next_ticket_id("FEAT", existing), "FEAT-0008");
    }

    #[test]
    fn test_next_ticket_id_grows_past_four_digits() {
        assert_eq!(next_ticket_id("TASK", ["TASK-9999"]), "TASK-10000");
    }

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("Hello World", 50), "hello-world");
        assert_eq!(slugify("Fix: Login Bug!", 50), "fix-login-bug");
        assert_eq!(slugify("  Multiple   Spaces  ", 50), "multiple-spaces");
    }

    #[test]
    fn test_slugify_truncates_at_word_boundary() {
        let slug = slugify("This is a very long string that should be truncated", 30);
        assert!(slug.len() <= 30);
        assert!(!slug.ends_with('-'));
    }

    #[test]
    fn test_slugify_non_ascii_never_panics() {
        assert_eq!(slugify("Ünïcödé façade 日本語 fix", 12), "n-c-d-fa");
    }

    #[test]
    fn test_unique_path_suffixes_existing_names() {
        let dir = tempfile::tempdir().unwrap();
        let first = unique_path(dir.path(), "a");
        std::fs::write(&first, "").unwrap();
        let second = unique_path(dir.path(), "a");
        assert!(second.ends_with("a-2.md"));
    }
}

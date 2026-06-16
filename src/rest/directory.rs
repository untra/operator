//! Directory identity for the operator working root.
//!
//! Lets a client decide whether an API already listening on the expected port
//! belongs to the *same* project before adopting it as "connected". We expose:
//!
//! - `directory_name`: the top-level directory name (basename of the working
//!   root). This is intentionally human-readable — operator's purpose is to
//!   report on the projects/repos under that directory — and is the value the
//!   code-projects API also surfaces.
//! - `directory_id`: a non-reversible fingerprint (first 12 hex chars of
//!   `SHA-256(canonical absolute path)`). Used *only* for exact same-directory
//!   matching, so adoption never cross-wires two repos that share a basename.
//!   The full path is never exposed over the wire.

use std::fmt::Write as _;
use std::path::Path;

use sha2::{Digest, Sha256};

/// Identity of the operator working root: `(directory_name, directory_id)`.
///
/// The working root is the directory that *contains* the `.tickets` directory
/// (i.e. the directory operator was launched in). `tickets_path` is the
/// `.tickets` directory itself.
pub fn directory_identity(tickets_path: &Path) -> (String, String) {
    // Working root = parent of the tickets dir; fall back to the tickets dir
    // itself if it has no parent (e.g. a bare relative path).
    let root = tickets_path.parent().unwrap_or(tickets_path);

    // Canonicalize for a stable id; fall back to the path as given when it does
    // not exist on disk yet (e.g. in tests).
    let canonical = std::fs::canonicalize(root).unwrap_or_else(|_| root.to_path_buf());

    let directory_name = canonical
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "operator".to_string());

    let mut hasher = Sha256::new();
    hasher.update(canonical.to_string_lossy().as_bytes());
    let digest = hasher.finalize();
    // First 6 bytes -> 12 hex chars. Enough to make accidental collisions
    // between distinct absolute paths negligible without pulling in `hex`.
    let mut directory_id = String::with_capacity(12);
    for byte in digest.iter().take(6) {
        let _ = write!(directory_id, "{byte:02x}");
    }

    (directory_name, directory_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_directory_name_is_parent_basename() {
        // /tmp/test does not exist; the helper falls back to the path as given.
        let (name, _id) = directory_identity(&PathBuf::from("/home/acme/.tickets"));
        assert_eq!(name, "acme");
    }

    #[test]
    fn test_directory_id_is_12_hex_chars() {
        let (_name, id) = directory_identity(&PathBuf::from("/home/acme/.tickets"));
        assert_eq!(id.len(), 12);
        assert!(id.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_directory_id_is_stable_for_same_path() {
        let p = PathBuf::from("/home/acme/.tickets");
        let (_n1, id1) = directory_identity(&p);
        let (_n2, id2) = directory_identity(&p);
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_directory_id_differs_for_different_paths() {
        // Same basename, different absolute path -> different id (no cross-wiring).
        let (n1, id1) = directory_identity(&PathBuf::from("/work/acme/.tickets"));
        let (n2, id2) = directory_identity(&PathBuf::from("/clients/acme/.tickets"));
        assert_eq!(n1, n2, "basenames intentionally match");
        assert_ne!(id1, id2, "ids must distinguish the two projects");
    }

    #[test]
    fn test_directory_id_does_not_contain_path() {
        let (_name, id) = directory_identity(&PathBuf::from("/home/secretuser/acme/.tickets"));
        assert!(!id.contains("secretuser"));
        assert!(!id.contains('/'));
    }
}

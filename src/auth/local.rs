//! Local auto-unlock for loopback processes.
//!
//! A local `operator` run — the TUI, the CLI, and the `opr8r` client talking to
//! `127.0.0.1` — needs no login. This is **not** an authentication bypass: the
//! credential is real and is checked like any other. It is issued
//! automatically to a caller who has already proven, by reading a file only its
//! owner can read, that they are the user who started the process.
//!
//! The proof is file ownership rather than peer-credential inspection
//! (`SO_PEERCRED` / `LOCAL_PEERCRED`). Those are Unix-socket mechanisms and
//! Operator listens on TCP, where they do not apply; mode `0600` establishes
//! the same boundary — only the owning uid (and root, which can bypass any
//! check anyway) can read the token — and works identically on Windows, where
//! the file inherits the user profile's ACL.
//!
//! Two conditions must both hold before a token is written:
//!
//! 1. The server is bound to a loopback address.
//! 2. The file is created with owner-only permissions.
//!
//! A non-loopback bind writes nothing, so a container or a `0.0.0.0` bind has
//! no local credential to find and must bootstrap.

use std::net::IpAddr;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::auth::secret::{generate_secret, hash_secret, hashes_equal};

/// Filename of the local-unlock token inside the state directory.
pub const LOCAL_TOKEN_FILENAME: &str = "local-token";

/// Create (or replace) a file containing `contents`, readable only by its owner.
///
/// The mode is set **as the file is created**, not afterwards. A
/// write-then-chmod sequence leaves a window in which the token is
/// world-readable, and — as the test suite found — it also fails outright if
/// anything removes the file in between.
#[cfg(unix)]
fn write_owner_only(path: &Path, contents: &str) -> std::io::Result<()> {
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;

    // Write to a uniquely named sibling with the mode set at creation, then
    // rename over the target.
    //
    // Two subtleties this avoids. `mode()` applies only when a file is
    // *created*, so writing straight to an existing token file would keep its
    // old permissions. And `create_new` on a fixed path fails when two
    // processes start at once, which is normal here — the TUI's embedded
    // server and a separate `operator api` share a state directory. Rename is
    // atomic and indifferent to an existing target, so both succeed and the
    // last writer wins.
    // Unique per call, not per process: several threads in one process issue
    // concurrently, and a shared temp name means one thread renames the file
    // out from under another.
    let temp = path.with_extension(format!("tmp.{}", uuid::Uuid::new_v4()));

    let result = (|| {
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(&temp)?;
        file.write_all(contents.as_bytes())?;
        file.sync_all()?;
        std::fs::rename(&temp, path)
    })();

    if result.is_err() {
        let _ = std::fs::remove_file(&temp);
    }
    result
}

/// On Windows the file inherits the user profile's ACL, which already excludes
/// other users; there is no portable mode bit to set at creation.
#[cfg(not(unix))]
fn write_owner_only(path: &Path, contents: &str) -> std::io::Result<()> {
    std::fs::write(path, contents)
}

/// A loopback bind is the precondition for issuing a local credential.
pub fn is_loopback(addr: IpAddr) -> bool {
    addr.is_loopback()
}

/// Path of the local token file.
pub fn local_token_path(state_path: &Path) -> PathBuf {
    state_path.join(LOCAL_TOKEN_FILENAME)
}

/// Issue (or re-issue) the local token, returning its value.
///
/// Called on every loopback start, generating a fresh secret each time: a token
/// left behind by a previous run should not authenticate against this one.
pub fn issue(state_path: &Path) -> Result<String> {
    std::fs::create_dir_all(state_path)
        .with_context(|| format!("creating state directory {}", state_path.display()))?;
    let path = local_token_path(state_path);
    let token = generate_secret()?;

    write_owner_only(&path, &token)
        .with_context(|| format!("writing local token {}", path.display()))?;

    Ok(token)
}

/// Remove the local token, on shutdown or when the bind is not loopback.
pub fn revoke(state_path: &Path) {
    let path = local_token_path(state_path);
    if path.exists() {
        if let Err(e) = std::fs::remove_file(&path) {
            tracing::warn!(error = %e, "failed to remove local auth token");
        }
    }
}

/// Read the local token, for a client that needs to call the API as itself
/// (the `ExternalApiProbe`, the CLI, `opr8r` on the same host).
pub fn read(state_path: &Path) -> Option<String> {
    std::fs::read_to_string(local_token_path(state_path))
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

/// Whether `presented` matches the issued local token, in constant time.
pub fn matches(issued: &str, presented: &str) -> bool {
    hashes_equal(&hash_secret(issued), &hash_secret(presented))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{Ipv4Addr, Ipv6Addr};

    #[test]
    fn test_loopback_detection() {
        assert!(is_loopback(IpAddr::V4(Ipv4Addr::LOCALHOST)));
        assert!(is_loopback(IpAddr::V6(Ipv6Addr::LOCALHOST)));
        assert!(is_loopback(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 5))));

        // The binds that must NOT get a free credential.
        assert!(!is_loopback(IpAddr::V4(Ipv4Addr::UNSPECIFIED)));
        assert!(!is_loopback(IpAddr::V4(Ipv4Addr::new(10, 1, 2, 3))));
        assert!(!is_loopback(IpAddr::V6(Ipv6Addr::UNSPECIFIED)));
    }

    #[test]
    fn test_issue_then_read_round_trips() {
        let dir = tempfile::tempdir().unwrap();
        let token = issue(dir.path()).unwrap();
        assert_eq!(read(dir.path()).as_deref(), Some(token.as_str()));
        assert!(matches(&token, &token));
    }

    #[test]
    fn test_each_issue_replaces_the_previous_token() {
        // A token from a previous run must not authenticate against this one.
        let dir = tempfile::tempdir().unwrap();
        let first = issue(dir.path()).unwrap();
        let second = issue(dir.path()).unwrap();
        assert_ne!(first, second);
        assert!(!matches(&second, &first));
    }

    #[cfg(unix)]
    #[test]
    fn test_token_file_is_readable_only_by_its_owner() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        issue(dir.path()).unwrap();

        let mode = std::fs::metadata(local_token_path(dir.path()))
            .unwrap()
            .permissions()
            .mode();
        assert_eq!(
            mode & 0o777,
            0o600,
            "file ownership is the authentication check here, so the mode is a \
             security control, not tidiness"
        );
    }

    #[cfg(unix)]
    #[test]
    fn test_token_is_never_briefly_world_readable() {
        // Regression: the mode was applied after the write, which both left a
        // window where the token was world-readable and made concurrent
        // issue/revoke on a shared state directory fail outright.
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();

        // Pre-create a permissive file so a failure to set the mode on
        // *replacement* would be visible rather than masked by a fresh create.
        let path = local_token_path(dir.path());
        std::fs::write(&path, "stale").unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();

        issue(dir.path()).unwrap();
        let mode = std::fs::metadata(&path).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o600);
    }

    #[test]
    fn test_concurrent_issue_on_one_directory_does_not_fail() {
        // The TUI's embedded server and a separate `operator api` share a state
        // directory; neither start may error because the other is also starting.
        let dir = tempfile::tempdir().unwrap();
        let errors: Vec<String> = std::thread::scope(|scope| {
            let handles: Vec<_> = (0..8)
                .map(|_| {
                    let path = dir.path().to_path_buf();
                    scope.spawn(move || issue(&path).map(|_| ()))
                })
                .collect();
            handles
                .into_iter()
                .filter_map(|h| h.join().expect("no panic").err())
                .map(|e| format!("{e:#}"))
                .collect()
        });
        assert!(errors.is_empty(), "concurrent issue failed: {errors:#?}");
        assert!(read(dir.path()).is_some());
    }

    #[test]
    fn test_read_returns_none_when_no_token_was_issued() {
        // A non-loopback bind issues nothing, so there is nothing to find.
        let dir = tempfile::tempdir().unwrap();
        assert!(read(dir.path()).is_none());
    }

    #[test]
    fn test_revoke_removes_the_token() {
        let dir = tempfile::tempdir().unwrap();
        issue(dir.path()).unwrap();
        revoke(dir.path());
        assert!(read(dir.path()).is_none());
        // Revoking again is a no-op, not a panic.
        revoke(dir.path());
    }

    #[test]
    fn test_empty_or_whitespace_token_file_reads_as_absent() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(local_token_path(dir.path()), "   \n").unwrap();
        assert!(read(dir.path()).is_none());
    }

    #[test]
    fn test_mismatched_token_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let token = issue(dir.path()).unwrap();
        assert!(!matches(&token, "some other value"));
        assert!(!matches(&token, ""));
    }
}

//! No hand-rolled encodings, hashes, or bit-level parsing.
//!
//! Bit twiddling in application code is almost always a reimplementation of
//! something a vetted crate or std already does correctly. Use these instead:
//!
//! | Need | Use |
//! |------|-----|
//! | Base64 | `base64` crate, or reqwest `RequestBuilder::basic_auth` for Basic auth |
//! | Hex | `hex::encode` |
//! | UUIDs | `uuid::Uuid::new_v4` |
//! | URL / query encoding | `url::form_urlencoded`, reqwest `RequestBuilder::query` |
//! | Digests | `sha2` |
//! | Small non-crypto hashes | `fnv::FnvHasher` |
//! | IP range checks | `std::net::Ipv{4,6}Addr::is_*` |
//!
//! Run `cargo test --test no_handrolled_encoding` before committing.

use std::path::{Path, PathBuf};

use regex::Regex;

/// Source roots scanned, one per Rust crate.
const SOURCE_ROOTS: &[&str] = &["src", "crates/relay/src", "opr8r/src", "zed-extension/src"];

/// Files allowed to use bitwise operators, with the reason.
const EXEMPT: &[(&str, &str)] = &[(
    "src/auth/store.rs",
    "exponential login backoff `1 << n` is arithmetic, not an encoding",
)];

/// Forbidden operator patterns. rustfmt spaces binary operators, which keeps
/// generics like `Vec<Vec<u8>>` from matching.
const FORBIDDEN: &[(&str, &str)] = &[
    (r"\s(<<|>>)=?\s", "bit shift"),
    (r"\s\^=?\s", "xor"),
    (r"[&|]=?\s*0x[0-9a-fA-F_]+", "hex-literal mask"),
];

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn rust_files() -> Vec<PathBuf> {
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, out);
            } else if path.extension().is_some_and(|e| e == "rs") {
                out.push(path);
            }
        }
    }

    let root = repo_root();
    let mut files = Vec::new();
    for dir in SOURCE_ROOTS {
        walk(&root.join(dir), &mut files);
    }
    files.sort();
    assert!(
        !files.is_empty(),
        "found no sources under {SOURCE_ROOTS:?} - has the layout moved?"
    );
    files
}

fn rel(path: &Path) -> String {
    path.strip_prefix(repo_root())
        .unwrap_or(path)
        .display()
        .to_string()
}

fn code_of(line: &str) -> &str {
    line.split("//").next().unwrap_or_default()
}

#[test]
fn test_no_handrolled_bitwise_encoding() {
    let patterns: Vec<(Regex, &str)> = FORBIDDEN
        .iter()
        .map(|(re, name)| (Regex::new(re).unwrap(), *name))
        .collect();

    let mut hits = Vec::new();
    for file in rust_files() {
        let name = rel(&file);
        if EXEMPT.iter().any(|(path, _)| *path == name) {
            continue;
        }
        let source =
            std::fs::read_to_string(&file).unwrap_or_else(|e| panic!("{name}: cannot read: {e}"));
        for (idx, line) in source.lines().enumerate() {
            let code = code_of(line);
            for (re, what) in &patterns {
                if re.is_match(code) {
                    hits.push(format!("{name}:{}: {what}: {}", idx + 1, line.trim()));
                }
            }
        }
    }

    assert!(
        hits.is_empty(),
        "hand-rolled bitwise code found; use a crate or std API instead \
         (see tests/no_handrolled_encoding.rs):\n{}",
        hits.join("\n")
    );
}

#[test]
fn test_exemptions_still_exist() {
    for (path, reason) in EXEMPT {
        assert!(
            repo_root().join(path).is_file(),
            "stale exemption {path} ({reason}): file no longer exists"
        );
    }
}

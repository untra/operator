//! One write path for ticket column moves, enforced.
//!
//! A ticket's board column is which of `.tickets/{queue,in-progress,completed}`
//! it lives in, so *moving the file is the state change*. Every surface must do
//! it through `services::ticket_transitions::move_ticket`, which also mirrors
//! the move to the board the ticket was synced from.
//!
//! Renaming a ticket file directly still moves it locally, and still looks
//! correct on operator's own board - it just silently strands the ticket in its
//! old column on Jira/Linear/GitHub. That is exactly the bug this test prevents
//! from coming back, and it is invisible without a configured provider.

use std::path::{Path, PathBuf};

/// Modules that own the move and are allowed to rename a ticket file.
const MOVE_OWNERS: &[&str] = &["src/queue/mod.rs"];

/// Surfaces that act on tickets and must delegate the move.
const TICKET_SURFACES: &[&str] = &["src/rest", "src/mcp", "src/acp", "src/app", "src/agents"];

/// Renames that move something other than a ticket file, with the reason.
/// Keep this list short and justified.
const TEST_EXEMPT: &[(&str, &str)] = &[(
    "src/agents/launcher/coder.rs",
    "atomic install of the downloaded Coder CLI binary, not a ticket file",
)];

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            rust_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

/// 1-based lines calling `fs::rename`. Doc comments may name the call while
/// explaining why it is avoided, so comment lines do not count.
fn rename_lines(source: &str) -> Vec<usize> {
    source
        .lines()
        .enumerate()
        .filter(|(_, line)| {
            let code = line.trim_start();
            !code.starts_with("//") && code.contains("fs::rename")
        })
        .map(|(n, _)| n + 1)
        .collect()
}

fn exempt_paths(root: &Path) -> Vec<PathBuf> {
    TEST_EXEMPT.iter().map(|(p, _)| root.join(p)).collect()
}

#[test]
fn test_ticket_surfaces_do_not_rename_ticket_files_directly() {
    let root = repo_root();
    let owners: Vec<PathBuf> = MOVE_OWNERS.iter().map(|p| root.join(p)).collect();
    let exempt = exempt_paths(&root);

    let mut files = Vec::new();
    for surface in TICKET_SURFACES {
        rust_files(&root.join(surface), &mut files);
    }
    assert!(!files.is_empty(), "no source files found to check");

    let mut offenders = Vec::new();
    for file in files {
        if owners.contains(&file) || exempt.contains(&file) {
            continue;
        }
        let Ok(source) = std::fs::read_to_string(&file) else {
            continue;
        };
        for line in rename_lines(&source) {
            offenders.push(format!(
                "{}:{line}",
                file.strip_prefix(&root).unwrap_or(&file).display()
            ));
        }
    }

    assert!(
        offenders.is_empty(),
        "these ticket surfaces rename files directly instead of calling \
         services::ticket_transitions::move_ticket, so a synced ticket will not \
         move on its external board:\n  {}",
        offenders.join("\n  ")
    );
}

#[test]
fn test_exemptions_are_real_and_still_needed() {
    let root = repo_root();
    for (path, reason) in TEST_EXEMPT {
        assert!(
            TICKET_SURFACES.iter().any(|s| path.starts_with(s)),
            "exempt path '{path}' is not under a ticket surface, so the exemption does nothing"
        );
        assert!(
            !MOVE_OWNERS.contains(path),
            "exempt path '{path}' already owns the move - drop one of the two lists"
        );
        let source = std::fs::read_to_string(root.join(path))
            .unwrap_or_else(|e| panic!("exempt path '{path}' is unreadable ({e}) - remove it"));
        assert!(
            !rename_lines(&source).is_empty(),
            "exempt path '{path}' no longer renames anything ({reason}) - remove the exemption"
        );
    }
}

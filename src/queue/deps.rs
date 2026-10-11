//! Resolve `depends_on` references to local tickets.

use super::Ticket;

/// Providers whose issues can be referenced as `provider:key`.
pub const PROVIDERS: [&str; 4] = ["jira", "linear", "github", "openspec"];

/// Where a dependency reference points.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Resolved {
    /// A local ticket, by id.
    Local(String),
    /// Nothing local matches yet; the ticket stays blocked on it.
    Unsynced(String),
}

/// Split `provider:key`; local ids return `None`.
pub fn provider_ref(r: &str) -> Option<(&str, &str)> {
    let (provider, key) = r.split_once(':')?;
    PROVIDERS.contains(&provider).then_some((provider, key))
}

/// Resolve one reference against known tickets (queue, in-progress and completed).
pub fn resolve(r: &str, tickets: &[Ticket]) -> Resolved {
    let found = match provider_ref(r) {
        Some((provider, key)) => tickets.iter().find(|t| {
            t.external_id.as_deref() == Some(key)
                && t.external_provider.as_deref().is_none_or(|p| p == provider)
        }),
        None => tickets.iter().find(|t| t.id == r),
    };
    found.map_or_else(
        || Resolved::Unsynced(r.to_string()),
        |t| Resolved::Local(t.id.clone()),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn ticket(dir: &Path, name: &str, frontmatter: &str) -> Ticket {
        let path = dir.join(name);
        std::fs::write(&path, format!("---\n{frontmatter}\n---\n\n# Task: t\n")).unwrap();
        Ticket::from_file(&path).unwrap()
    }

    fn tickets(dir: &Path) -> Vec<Ticket> {
        vec![
            ticket(
                dir,
                "20260101-1200-FEAT-demo-a.md",
                "id: FEAT-0003\nstatus: queued",
            ),
            ticket(
                dir,
                "20260101-1200-FIX-demo-b.md",
                "id: FIX-PROJ12\nstatus: completed\nexternal_id: PROJ-12\nexternal_provider: jira",
            ),
        ]
    }

    #[test]
    fn test_provider_ref_only_matches_known_providers() {
        assert_eq!(provider_ref("jira:PROJ-12"), Some(("jira", "PROJ-12")));
        assert_eq!(
            provider_ref("github:octo/hello#9"),
            Some(("github", "octo/hello#9"))
        );
        assert_eq!(provider_ref("FEAT-0003"), None);
        assert_eq!(provider_ref("http://example.com"), None);
    }

    #[test]
    fn test_resolve_local_id() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(
            resolve("FEAT-0003", &tickets(dir.path())),
            Resolved::Local("FEAT-0003".into())
        );
    }

    #[test]
    fn test_resolve_provider_ref_ignores_local_type_prefix() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(
            resolve("jira:PROJ-12", &tickets(dir.path())),
            Resolved::Local("FIX-PROJ12".into())
        );
    }

    #[test]
    fn test_resolve_wrong_provider_or_missing_is_unsynced() {
        let dir = tempfile::tempdir().unwrap();
        let all = tickets(dir.path());
        assert_eq!(
            resolve("linear:PROJ-12", &all),
            Resolved::Unsynced("linear:PROJ-12".into())
        );
        assert_eq!(
            resolve("FEAT-9999", &all),
            Resolved::Unsynced("FEAT-9999".into())
        );
    }
}

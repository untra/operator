//! The vertical integration catalog — single source of truth for every
//! advertised integration and its [`SupportStatus`].
//!
//! Operator advertises integrations across several **verticals** (kanban
//! providers, model providers, git providers, session wrappers, editors, LLM
//! tools, platforms, integrations). Each [`CatalogEntry`] names one entry, where
//! its docs live, whether it carries a README badge, and its official support
//! status. Every downstream surface derives from this one list:
//!
//! - REST `/api/v1/integrations` ([`crate::rest::dto::integration_catalog`])
//! - the generated `docs/maturity/` page ([`crate::docs_gen::integrations`])
//! - the `tests/vertical_parity.rs` soup-to-nuts alignment test, which also
//!   cross-checks that every provider-enum variant (`KanbanProviderType::ALL`,
//!   `ModelServerKind::ALL`, `GitProvider::ALL`, `SessionWrapperType::ALL`) has a
//!   catalog entry — so a new variant can't ship without docs/badges/UI.
//!
//! Adding a new vertical entry here, plus its docs page (and README badge for
//! `Alpha`+), is all that is required to keep the surfaces aligned.

use crate::integrations::SupportStatus;

/// A top-level advertised vertical. The [`label`](Self::label) matches the
/// bolded category in `README.md`'s badge list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Vertical {
    Kanban,
    Model,
    Git,
    Session,
    Editor,
    LlmTool,
    Platform,
    Integration,
    Workflows,
}

impl Vertical {
    /// All verticals, in README display order.
    pub const ALL: [Vertical; 9] = [
        Vertical::Kanban,
        Vertical::Model,
        Vertical::Git,
        Vertical::Session,
        Vertical::Editor,
        Vertical::LlmTool,
        Vertical::Platform,
        Vertical::Integration,
        Vertical::Workflows,
    ];

    /// Stable lowercase slug (wire id for the REST DTO).
    pub fn slug(&self) -> &'static str {
        match self {
            Vertical::Kanban => "kanban",
            Vertical::Model => "model",
            Vertical::Git => "git",
            Vertical::Session => "session",
            Vertical::Editor => "editor",
            Vertical::LlmTool => "llm-tool",
            Vertical::Platform => "platform",
            Vertical::Integration => "integration",
            Vertical::Workflows => "workflows",
        }
    }

    /// Human label — matches the bold category in the README badge list.
    pub fn label(&self) -> &'static str {
        match self {
            Vertical::Kanban => "Kanban Provider",
            Vertical::Model => "Model Provider",
            Vertical::Git => "Git Version Control",
            Vertical::Session => "Session",
            Vertical::Editor => "Editor",
            Vertical::LlmTool => "LLM Tool",
            Vertical::Platform => "Platform",
            Vertical::Integration => "Integration",
            Vertical::Workflows => "Workflow Format",
        }
    }
}

/// One advertised integration within a [`Vertical`].
#[derive(Debug, Clone)]
pub struct CatalogEntry {
    /// Which vertical this entry belongs to.
    pub vertical: Vertical,
    /// Stable slug. For verticals with a provider enum this equals that enum's
    /// `slug()` (so the parity test can cross-check coverage).
    pub slug: &'static str,
    /// Display / README-badge label.
    pub label: &'static str,
    /// Docs path relative to the site root (e.g.
    /// `getting-started/kanban/jira`), or `None` if undocumented. Drives both
    /// the docs link and the expected README badge URL.
    pub docs_path: Option<&'static str>,
    /// Whether this entry carries a curated README badge.
    pub readme_badge: bool,
    /// Official support / maturity status.
    pub status: SupportStatus,
}

impl CatalogEntry {
    /// The absolute docs URL this entry resolves to, if documented.
    pub fn docs_url(&self) -> Option<String> {
        self.docs_path
            .map(|p| format!("https://operator.untra.io/{p}/"))
    }
}

/// The canonical list of advertised integrations. **Single source of truth.**
///
/// Support statuses reflect the current maturity of each integration. `Proto`
/// entries are intentionally not advertised (no README badge); `Alpha`+ entries
/// require a docs page (enforced by `tests/vertical_parity.rs`).
pub fn all_integrations() -> Vec<CatalogEntry> {
    use SupportStatus::{Alpha, Beta, Ga, Proto};
    use Vertical::{
        Editor, Git, Integration, Kanban, LlmTool, Model, Platform, Session, Workflows,
    };
    vec![
        // --- Kanban providers (mirror KanbanProviderType::ALL) ---
        entry(
            Kanban,
            "jira",
            "Jira",
            Some("getting-started/kanban/jira"),
            true,
            Beta,
        ),
        entry(
            Kanban,
            "linear",
            "Linear",
            Some("getting-started/kanban/linear"),
            true,
            Beta,
        ),
        entry(
            Kanban,
            "github",
            "GitHub Projects",
            Some("getting-started/kanban/github"),
            true,
            Beta,
        ),
        // --- Model providers (mirror ModelServerKind::ALL; slug == kind slug) ---
        entry(
            Model,
            "anthropic-api",
            "Anthropic",
            Some("getting-started/model-servers/anthropic"),
            true,
            Beta,
        ),
        entry(
            Model,
            "openai-api",
            "OpenAI",
            Some("getting-started/model-servers/openai"),
            true,
            Beta,
        ),
        entry(
            Model,
            "google-api",
            "Google",
            Some("getting-started/model-servers/google"),
            true,
            Alpha,
        ),
        entry(
            Model,
            "ollama",
            "Ollama",
            Some("getting-started/model-servers/ollama"),
            true,
            Beta,
        ),
        entry(
            Model,
            "openrouter",
            "OpenRouter",
            Some("getting-started/model-servers/openrouter"),
            true,
            Beta,
        ),
        entry(
            Model,
            "openai-compat",
            "OpenAI-compatible",
            None,
            false,
            Proto,
        ),
        entry(Model, "lmstudio", "LM Studio", None, false, Proto),
        // --- Git providers (mirror GitProvider::ALL) ---
        entry(
            Git,
            "github",
            "GitHub",
            Some("getting-started/git/github"),
            true,
            Beta,
        ),
        entry(
            Git,
            "gitlab",
            "GitLab",
            Some("getting-started/git/gitlab"),
            true,
            Alpha,
        ),
        entry(Git, "bitbucket", "Bitbucket", None, false, Proto),
        entry(Git, "azure", "Azure DevOps", None, false, Proto),
        // --- Session wrappers (mirror SessionWrapperType::ALL; vscode lives under Editor) ---
        entry(
            Session,
            "tmux",
            "tmux",
            Some("getting-started/sessions/tmux"),
            true,
            Beta,
        ),
        entry(
            Session,
            "cmux",
            "cmux",
            Some("getting-started/sessions/cmux"),
            true,
            Beta,
        ),
        entry(
            Session,
            "zellij",
            "Zellij",
            Some("getting-started/sessions/zellij"),
            true,
            Beta,
        ),
        // --- Editors ---
        entry(
            Editor,
            "vscode",
            "VS Code",
            Some("getting-started/sessions/vscode"),
            true,
            Beta,
        ),
        entry(
            Editor,
            "zed",
            "Zed",
            Some("getting-started/sessions/zed"),
            true,
            Alpha,
        ),
        entry(
            Editor,
            "cursor",
            "Cursor",
            Some("getting-started/sessions/cursor"),
            false,
            Proto,
        ),
        // --- LLM tools ---
        entry(
            LlmTool,
            "claude",
            "Claude",
            Some("getting-started/agents/claude"),
            true,
            Ga,
        ),
        entry(
            LlmTool,
            "codex",
            "Codex",
            Some("getting-started/agents/codex"),
            true,
            Beta,
        ),
        entry(
            LlmTool,
            "gemini-cli",
            "Gemini CLI",
            Some("getting-started/agents/gemini-cli"),
            true,
            Alpha,
        ),
        // --- Platforms ---
        entry(
            Platform,
            "docker",
            "Docker",
            Some("getting-started/platforms/docker"),
            true,
            Beta,
        ),
        entry(
            Platform,
            "coder",
            "Coder",
            Some("getting-started/platforms/coder"),
            true,
            Alpha,
        ),
        // --- Integrations (documented, no README badge row) ---
        entry(
            Integration,
            "agnt",
            "AGNT",
            Some("getting-started/integrations/agnt"),
            false,
            Alpha,
        ),
        // --- Workflow formats (mirror WorkflowFormat::ALL) ---
        entry(
            Workflows,
            "claude",
            "Claude Workflow",
            Some("getting-started/workflows/claude"),
            true,
            Ga,
        ),
        entry(
            Workflows,
            "agnt",
            "AGNT Workflow",
            Some("getting-started/workflows/agnt"),
            true,
            Alpha,
        ),
    ]
}

/// Find the catalog entry for a `(vertical, slug)` pair, if present.
pub fn entry_for(vertical: Vertical, slug: &str) -> Option<CatalogEntry> {
    all_integrations()
        .into_iter()
        .find(|e| e.vertical == vertical && e.slug == slug)
}

/// Terse constructor keeping [`all_integrations`] readable.
fn entry(
    vertical: Vertical,
    slug: &'static str,
    label: &'static str,
    docs_path: Option<&'static str>,
    readme_badge: bool,
    status: SupportStatus,
) -> CatalogEntry {
    CatalogEntry {
        vertical,
        slug,
        label,
        docs_path,
        readme_badge,
        status,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_catalog_non_empty() {
        assert!(!all_integrations().is_empty());
    }

    #[test]
    fn test_proto_entries_are_not_badged() {
        for e in all_integrations() {
            if e.status == SupportStatus::Proto {
                assert!(
                    !e.readme_badge,
                    "Proto entry '{}/{}' must not be advertised with a README badge",
                    e.vertical.slug(),
                    e.slug
                );
            }
        }
    }

    #[test]
    fn test_alpha_plus_entries_are_documented() {
        for e in all_integrations() {
            if e.status >= SupportStatus::Alpha {
                assert!(
                    e.docs_path.is_some(),
                    "Alpha+ entry '{}/{}' must have a docs page",
                    e.vertical.slug(),
                    e.slug
                );
            }
        }
    }

    #[test]
    fn test_badged_entries_have_docs() {
        for e in all_integrations() {
            if e.readme_badge {
                assert!(
                    e.docs_path.is_some(),
                    "Badged entry '{}/{}' must link to a docs page",
                    e.vertical.slug(),
                    e.slug
                );
            }
        }
    }

    #[test]
    fn test_vertical_slug_per_entry_is_unique() {
        let mut seen = std::collections::HashSet::new();
        for e in all_integrations() {
            let key = (e.vertical, e.slug);
            assert!(
                seen.insert(key),
                "Duplicate catalog entry for {}/{}",
                e.vertical.slug(),
                e.slug
            );
        }
    }

    #[test]
    fn test_entry_for_resolves_known_pair() {
        let jira = entry_for(Vertical::Kanban, "jira").expect("jira entry");
        assert_eq!(jira.status, SupportStatus::Beta);
        assert!(entry_for(Vertical::Kanban, "nope").is_none());
    }
}

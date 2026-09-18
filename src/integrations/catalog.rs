//! The vertical integration catalog - single source of truth for every
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
//!   catalog entry - so a new variant can't ship without docs/badges/UI.
//!
//! Adding a new vertical entry here, plus its docs page (and README badge for
//! `Alpha`+), is all that is required to keep the surfaces aligned.

use crate::config::SessionWrapperType;
use crate::integrations::SupportStatus;

pub fn ide_session_wrappers(ide: &str) -> Option<&'static [SessionWrapperType]> {
    match ide {
        "vscode" => Some(&[SessionWrapperType::Vscode]),
        "zed" | "cursor" => Some(&[]),
        _ => None,
    }
}

pub fn validate_ide_session(ide: &str, wrapper: SessionWrapperType) -> Result<(), String> {
    let wrappers = ide_session_wrappers(ide).ok_or_else(|| format!("Unknown IDE: {ide}"))?;
    if wrappers.contains(&wrapper) {
        Ok(())
    } else {
        Err(format!(
            "{} cannot manage sessions in {ide}",
            wrapper.display_name()
        ))
    }
}

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
    Notification,
    Transport,
    AgentRelay,
    RemoteTargets,
}

impl Vertical {
    /// All verticals, in README display order.
    pub const ALL: [Vertical; 13] = [
        Vertical::Kanban,
        Vertical::Model,
        Vertical::Git,
        Vertical::Session,
        Vertical::Editor,
        Vertical::LlmTool,
        Vertical::Platform,
        Vertical::Integration,
        Vertical::Workflows,
        Vertical::Notification,
        Vertical::Transport,
        Vertical::AgentRelay,
        Vertical::RemoteTargets,
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
            Vertical::Notification => "notification",
            Vertical::Transport => "transport",
            Vertical::AgentRelay => "agent-relay",
            Vertical::RemoteTargets => "remote-targets",
        }
    }

    /// Human label - matches the bold category in the README badge list.
    pub fn label(&self) -> &'static str {
        match self {
            Vertical::Kanban => "Kanban Provider",
            Vertical::Model => "Model Provider",
            Vertical::Git => "Git Version Control",
            Vertical::Session => "Session Management",
            Vertical::Editor => "IDE",
            Vertical::LlmTool => "LLM Tool",
            Vertical::Platform => "Platform",
            Vertical::Integration => "Integration",
            Vertical::Workflows => "Workflow Export Format",
            Vertical::Notification => "Notification Channel",
            Vertical::Transport => "Execution Transport",
            Vertical::AgentRelay => "Agent Relay",
            Vertical::RemoteTargets => "Remote Targets",
        }
    }

    /// Docs section directory (site-root-relative) that hosts this vertical's
    /// entry pages - the sidebar nav item URL and the section `index.md`.
    pub fn docs_section(&self) -> &'static str {
        match self {
            Vertical::Kanban => "getting-started/kanban",
            Vertical::Model => "getting-started/model-servers",
            Vertical::Git => "getting-started/git",
            Vertical::Session => "getting-started/sessions",
            Vertical::Editor => "getting-started/ides",
            Vertical::LlmTool => "getting-started/agents",
            Vertical::Platform => "getting-started/platforms",
            Vertical::Integration => "getting-started/integrations",
            Vertical::Workflows => "getting-started/workflows",
            Vertical::Notification => "getting-started/notifications",
            Vertical::Transport => "getting-started/transports",
            Vertical::AgentRelay => "getting-started/agent-relays",
            Vertical::RemoteTargets => "getting-started/remote-targets",
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
    /// Brand icon filename stem in `docs/assets/icons/{icon}.svg`, or `None`
    /// when no brand icon ships. Drives the sidebar nav `icon:` values
    /// (enforced by `tests/docs_structure.rs`).
    pub icon: Option<&'static str>,
    /// Whether this entry carries a curated README badge.
    pub readme_badge: bool,
    /// Official support / maturity status.
    pub status: SupportStatus,
    pub premium: bool,
}

impl CatalogEntry {
    pub fn is_public(&self) -> bool {
        self.status >= SupportStatus::Alpha
    }

    fn premium(mut self) -> Self {
        self.premium = true;
        self
    }

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
        AgentRelay, Editor, Git, Integration, Kanban, LlmTool, Model, Notification, Platform,
        RemoteTargets, Session, Transport, Workflows,
    };
    vec![
        // --- Kanban providers (mirror KanbanProviderType::ALL) ---
        // The built-in board leads the vertical: every other entry syncs into it.
        entry(
            Kanban,
            "operator",
            "Operator",
            Some("getting-started/kanban/operator"),
            Some("operator"),
            true,
            Ga,
        ),
        entry(
            Kanban,
            "jira",
            "Jira",
            Some("getting-started/kanban/jira"),
            Some("jira"),
            true,
            Beta,
        ),
        entry(
            Kanban,
            "linear",
            "Linear",
            Some("getting-started/kanban/linear"),
            Some("linear"),
            true,
            Beta,
        ),
        entry(
            Kanban,
            "github",
            "GitHub Projects",
            Some("getting-started/kanban/github"),
            Some("github"),
            true,
            Beta,
        ),
        entry(
            Kanban,
            "openspec",
            "OpenSpec",
            Some("getting-started/kanban/openspec"),
            Some("openspec"),
            false,
            Alpha,
        ),
        // --- Model providers (mirror ModelServerKind::ALL; slug == kind slug) ---
        entry(
            Model,
            "anthropic-api",
            "Anthropic",
            Some("getting-started/model-servers/anthropic"),
            Some("anthropic"),
            true,
            Beta,
        ),
        entry(
            Model,
            "openai-api",
            "OpenAI",
            Some("getting-started/model-servers/openai"),
            Some("openai"),
            true,
            Beta,
        ),
        entry(
            Model,
            "google-api",
            "Google",
            Some("getting-started/model-servers/google"),
            Some("google"),
            true,
            Alpha,
        ),
        entry(
            Model,
            "ollama",
            "Ollama",
            Some("getting-started/model-servers/ollama"),
            Some("ollama"),
            true,
            Beta,
        ),
        entry(
            Model,
            "openrouter",
            "OpenRouter",
            Some("getting-started/model-servers/openrouter"),
            Some("openrouter"),
            true,
            Beta,
        ),
        entry(
            Model,
            "openai-compat",
            "OpenAI-compatible",
            None,
            None,
            false,
            Proto,
        ),
        entry(Model, "lmstudio", "LM Studio", None, None, false, Proto),
        // --- Git providers (mirror GitProvider::ALL) ---
        entry(
            Git,
            "github",
            "GitHub",
            Some("getting-started/git/github"),
            Some("github"),
            true,
            Beta,
        ),
        entry(
            Git,
            "gitlab",
            "GitLab",
            Some("getting-started/git/gitlab"),
            Some("gitlab"),
            true,
            Alpha,
        ),
        entry(Git, "bitbucket", "Bitbucket", None, None, false, Proto),
        entry(Git, "azure", "Azure DevOps", None, None, false, Proto),
        entry(Git, "forgejo", "Forgejo", None, None, false, Proto),
        entry(
            Git,
            "gitea",
            "Gitea",
            Some("getting-started/git/gitea"),
            Some("gitea"),
            false,
            Alpha,
        ),
        // --- Session wrappers (mirror SessionWrapperType::ALL) ---
        entry(
            Session,
            "tmux",
            "tmux",
            Some("getting-started/sessions/tmux"),
            Some("tmux"),
            true,
            Beta,
        ),
        entry(
            Session,
            "cmux",
            "cmux",
            Some("getting-started/sessions/cmux"),
            Some("cmux"),
            true,
            Beta,
        ),
        entry(
            Session,
            "zellij",
            "Zellij",
            Some("getting-started/sessions/zellij"),
            Some("zellij"),
            true,
            Beta,
        ),
        entry(
            Session,
            "vscode",
            "VS Code Terminals",
            Some("getting-started/sessions/vscode-terminals"),
            Some("vscode"),
            true,
            Beta,
        ),
        // --- IDEs ---
        entry(
            Editor,
            "vscode",
            "VS Code",
            Some("getting-started/ides/vscode"),
            Some("vscode"),
            true,
            Beta,
        ),
        entry(
            Editor,
            "zed",
            "Zed",
            Some("getting-started/ides/zed"),
            Some("zed"),
            true,
            Alpha,
        ),
        entry(
            Editor,
            "cursor",
            "Cursor",
            Some("getting-started/ides/cursor"),
            Some("cursor"),
            false,
            Proto,
        ),
        // --- LLM tools ---
        entry(
            LlmTool,
            "claude",
            "Claude",
            Some("getting-started/agents/claude"),
            Some("claude"),
            true,
            Ga,
        ),
        entry(
            LlmTool,
            "codex",
            "Codex",
            Some("getting-started/agents/codex"),
            Some("codex"),
            true,
            Beta,
        ),
        entry(
            LlmTool,
            "gemini-cli",
            "Gemini CLI",
            Some("getting-started/agents/gemini-cli"),
            Some("gemini"),
            true,
            Alpha,
        ),
        // --- Platforms ---
        entry(
            Platform,
            "docker",
            "Docker",
            Some("getting-started/platforms/docker"),
            Some("docker"),
            true,
            Beta,
        ),
        entry(
            RemoteTargets,
            "coder",
            "Coder",
            Some("getting-started/remote-targets/coder"),
            Some("coder"),
            true,
            Alpha,
        )
        .premium(),
        entry(
            Platform,
            "kubernetes",
            "Kubernetes",
            Some("getting-started/platforms/kubernetes"),
            Some("kubernetes"),
            true,
            Alpha,
        ),
        entry(
            Transport,
            "local",
            "Local",
            Some("getting-started/transports/local"),
            None,
            false,
            Alpha,
        ),
        entry(
            Transport,
            "ssh",
            "SSH",
            Some("getting-started/transports/ssh"),
            None,
            false,
            Alpha,
        )
        .premium(),
        entry(
            AgentRelay,
            "claude-relay",
            "Claude Relay",
            Some("getting-started/agent-relays/claude-relay"),
            Some("claude"),
            false,
            Alpha,
        ),
        entry(
            RemoteTargets,
            "ssh",
            "SSH Hosts",
            Some("getting-started/remote-targets/ssh"),
            None,
            false,
            Alpha,
        )
        .premium(),
        // --- Integrations (documented, no README badge row) ---
        entry(
            Integration,
            "agnt",
            "AGNT",
            Some("getting-started/integrations/agnt"),
            Some("agnt"),
            false,
            Alpha,
        ),
        // --- Workflow formats (mirror WorkflowFormat::ALL) ---
        entry(
            Workflows,
            "claude",
            "Claude Workflow",
            Some("getting-started/workflows/claude"),
            Some("claude"),
            true,
            Ga,
        ),
        entry(
            Workflows,
            "agnt",
            "AGNT Workflow",
            Some("getting-started/workflows/agnt"),
            Some("agnt"),
            true,
            Alpha,
        ),
        // --- Notification channels ---
        entry(
            Notification,
            "os",
            "Operating System",
            Some("getting-started/notifications/os"),
            Some("notification"),
            true,
            Beta,
        ),
        entry(
            Notification,
            "webhooks",
            "Webhooks",
            Some("getting-started/notifications/webhooks"),
            Some("webhook"),
            true,
            Beta,
        ),
    ]
}

/// Find the catalog entry for a `(vertical, slug)` pair, if present.
/// Entries a first-run wizard may offer for this vertical.
///
/// `Alpha`+ and documented: onboarding links out to each provider's page, and
/// `Proto` entries are by definition not advertised. This is what keeps the TUI
/// wizard, the web wizard and the docs offering the same providers - promoting
/// one is a status bump here, not an edit in each surface.
pub fn onboardable(vertical: Vertical) -> Vec<CatalogEntry> {
    all_integrations()
        .into_iter()
        .filter(|e| e.vertical == vertical && e.is_public() && e.docs_path.is_some())
        .collect()
}

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
    icon: Option<&'static str>,
    readme_badge: bool,
    status: SupportStatus,
) -> CatalogEntry {
    CatalogEntry {
        vertical,
        slug,
        label,
        docs_path,
        icon,
        readme_badge,
        status,
        premium: false,
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
    fn remote_execution_is_premium_independently_of_maturity() {
        for entry in all_integrations()
            .into_iter()
            .filter(|e| e.vertical == Vertical::RemoteTargets)
        {
            assert!(entry.premium);
            assert!(entry.is_public());
        }
        assert!(!entry_for(Vertical::Transport, "local").unwrap().premium);
        assert!(entry_for(Vertical::Transport, "ssh").unwrap().premium);
        assert!(!entry_for(Vertical::Editor, "cursor").unwrap().is_public());
    }

    #[test]
    fn ide_controllers_are_not_interchangeable() {
        assert!(validate_ide_session("vscode", SessionWrapperType::Vscode).is_ok());
        assert!(validate_ide_session("vscode", SessionWrapperType::Cmux).is_err());
        assert!(validate_ide_session("cursor", SessionWrapperType::Vscode).is_err());
        assert!(validate_ide_session("zed", SessionWrapperType::Tmux).is_err());
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

    // --- Onboarding surface ---

    fn slugs(vertical: Vertical) -> Vec<&'static str> {
        onboardable(vertical).iter().map(|e| e.slug).collect()
    }

    #[test]
    fn test_onboardable_git_is_the_alpha_or_better_set() {
        assert_eq!(slugs(Vertical::Git), vec!["github", "gitlab", "gitea"]);
    }

    #[test]
    fn test_onboardable_model_excludes_proto_entries() {
        let model = slugs(Vertical::Model);
        assert!(model.contains(&"anthropic-api"));
        assert!(model.contains(&"ollama"));
        assert!(!model.contains(&"openai-compat"), "openai-compat is Proto");
        assert!(!model.contains(&"lmstudio"), "lmstudio is Proto");
    }

    /// Onboarding links out to each provider's page, so an entry without docs
    /// must never reach a wizard.
    #[test]
    fn test_onboardable_entries_are_all_documented() {
        for vertical in Vertical::ALL {
            for entry in onboardable(vertical) {
                assert!(
                    entry.docs_path.is_some(),
                    "{}/{} is offered for onboarding but has no docs page",
                    vertical.slug(),
                    entry.slug
                );
            }
        }
    }

    #[test]
    fn test_onboardable_never_includes_proto() {
        for vertical in Vertical::ALL {
            for entry in onboardable(vertical) {
                assert!(entry.status >= SupportStatus::Alpha, "{}", entry.slug);
            }
        }
    }

    #[test]
    fn test_onboardable_preserves_catalog_order() {
        let all: Vec<&str> = all_integrations()
            .iter()
            .filter(|e| e.vertical == Vertical::Model && e.status >= SupportStatus::Alpha)
            .map(|e| e.slug)
            .collect();
        assert_eq!(slugs(Vertical::Model), all);
    }
}

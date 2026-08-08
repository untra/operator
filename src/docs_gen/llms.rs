//! Documentation generator for `llms.txt` (the llmstxt.org standard).
//!
//! Produces a curated, token-efficient site map for LLMs, written to
//! `docs/llms.txt` and served verbatim at <https://operator.untra.io/llms.txt>.
//! Unlike the other generators this output has **no Jekyll front matter**, so
//! Jekyll copies it byte-for-byte instead of wrapping it in a layout.
//!
//! Section titles and descriptions are read from each section's `index.md`
//! front matter so the file stays in sync with the docs; the grouping/order is
//! editorial and lives here.

use super::DocGenerator;
use anyhow::Result;
use std::path::Path;

/// Canonical site URL used to build absolute links (required by llms.txt).
const SITE_URL: &str = "https://operator.untra.io";

/// Project name (the required H1).
const TITLE: &str = "Operator!";

/// One-line summary rendered as the required blockquote.
const SUMMARY: &str = "Operator! is a Rust TUI for orchestrating Claude Code and other LLM coding agents across multi-repository codebases, driven by kanban-style markdown tickets. It connects to Jira, Linear, and GitHub Projects; launches agents in tmux, cmux, Zellij, VS Code, or Zed sessions; reaches models via Anthropic, OpenAI, Google, OpenRouter, or Ollama; and enforces team workflows.";

/// Free-form context paragraph after the summary.
const INTRO: &str = "Operator runs from the root of your work directory, discovers projects by LLM marker files (CLAUDE.md / CODEX.md / GEMINI.md) and git repositories, and manages a `.tickets/` queue. It exposes a REST API and an embedded web developer portal.";

/// A doc section sourced from `docs/{slug}/index.md`.
struct Link {
    /// Directory slug; also the URL path segment.
    slug: &'static str,
    /// Used only when the page's front matter has no `description`.
    fallback_desc: &'static str,
}

/// An editorial grouping of links in the output.
struct Section {
    heading: &'static str,
    links: &'static [Link],
    /// Literal external links appended after the sourced links.
    extra: &'static [(&'static str, &'static str, &'static str)],
}

const SECTIONS: &[Section] = &[
    Section {
        heading: "Getting Started",
        links: &[
            Link {
                slug: "getting-started",
                fallback_desc: "",
            },
            Link {
                slug: "getting-started/tickets",
                fallback_desc: "Ticket format, naming, and lifecycle.",
            },
            Link {
                slug: "getting-started/kanban",
                fallback_desc: "Sync work from Jira, Linear, or GitHub Projects.",
            },
            Link {
                slug: "getting-started/agents",
                fallback_desc: "Supported coding agents, lifecycle, and modes.",
            },
            Link {
                slug: "downloads",
                fallback_desc: "",
            },
        ],
        extra: &[],
    },
    Section {
        heading: "Workflows",
        links: &[
            Link {
                slug: "workflows",
                fallback_desc: "Shareable collections of Operator workflows.",
            },
            Link {
                slug: "getting-started/workflows",
                fallback_desc: "Export a workflow to another runner's format.",
            },
            Link {
                slug: "delegators",
                fallback_desc: "",
            },
            Link {
                slug: "getting-started/sessions/remote-hosts",
                fallback_desc:
                    "Run launched agents on remote machines over SSH (execution targets).",
            },
        ],
        extra: &[],
    },
    Section {
        heading: "Integrations",
        links: &[
            Link {
                slug: "llm-tools",
                fallback_desc: "",
            },
            Link {
                slug: "relay",
                fallback_desc: "",
            },
        ],
        extra: &[],
    },
    Section {
        heading: "Reference",
        links: &[
            Link {
                slug: "cli",
                fallback_desc: "Commands and environment variables.",
            },
            Link {
                slug: "configuration",
                fallback_desc: "TOML configuration structure and options.",
            },
            Link {
                slug: "schemas",
                fallback_desc: "Ticket metadata, issue type, and OpenAPI schema reference.",
            },
            Link {
                slug: "shortcuts",
                fallback_desc: "TUI keyboard shortcuts by context.",
            },
            Link {
                slug: "taxonomy",
                fallback_desc: "Project Kinds across five tiers.",
            },
            Link {
                slug: "artifact-detection",
                fallback_desc: "Using produced files as step-completion signals.",
            },
        ],
        extra: &[],
    },
    Section {
        heading: "Optional",
        // `docs/architecture/index.md` is `published: false`, so Jekyll never
        // builds it — listing it here produced a live link to a 404. Re-add it
        // when the page is published.
        links: &[],
        extra: &[(
            "GitHub Repository",
            "https://github.com/untra/operator",
            "Source code (Rust, MIT).",
        )],
    },
];

/// Generates `llms.txt` from the docs section front matter.
pub struct LlmsTxtDocGenerator;

impl DocGenerator for LlmsTxtDocGenerator {
    fn name(&self) -> &'static str {
        "llms"
    }

    fn source(&self) -> &'static str {
        "docs/*/index.md (section front matter)"
    }

    fn output_path(&self) -> &'static str {
        "llms.txt"
    }

    fn generate(&self) -> Result<String> {
        let docs_root = Path::new("docs");
        let mut out = String::new();

        // Auto-gen marker (HTML comment — valid markdown, ignored by llms.txt
        // parsers, and not YAML front matter so Jekyll copies the file as-is).
        out.push_str("<!-- AUTO-GENERATED FROM docs/*/index.md - DO NOT EDIT MANUALLY -->\n");
        out.push_str("<!-- Regenerate with: cargo run -- docs --only llms -->\n\n");

        out.push_str(&format!("# {TITLE}\n\n"));
        out.push_str(&format!("> {SUMMARY}\n\n"));
        out.push_str(&format!("{INTRO}\n"));

        for section in SECTIONS {
            out.push_str(&format!("\n## {}\n", section.heading));
            for link in section.links {
                let (title, desc) = read_front_matter(docs_root, link.slug);
                let desc = desc.unwrap_or_else(|| link.fallback_desc.to_string());
                let url = format!("{SITE_URL}/{}/", link.slug);
                out.push_str(&render_item(&title, &url, &desc));
            }
            for (label, url, desc) in section.extra {
                out.push_str(&render_item(label, url, desc));
            }
        }

        Ok(out)
    }
}

/// Render one list item: `- [Title](url): description` (description optional).
fn render_item(title: &str, url: &str, desc: &str) -> String {
    if desc.is_empty() {
        format!("- [{title}]({url})\n")
    } else {
        format!("- [{title}]({url}): {desc}\n")
    }
}

/// Read `title` and `description` from a section's `index.md` front matter.
///
/// Falls back to a title-cased slug when the page or its `title` is missing.
fn read_front_matter(docs_root: &Path, slug: &str) -> (String, Option<String>) {
    let path = docs_root.join(slug).join("index.md");
    let content = std::fs::read_to_string(&path).unwrap_or_default();

    let mut title = None;
    let mut description = None;
    let mut in_front_matter = false;
    for line in content.lines() {
        if line.trim() == "---" {
            if in_front_matter {
                break; // end of front matter
            }
            in_front_matter = true;
            continue;
        }
        if !in_front_matter {
            continue;
        }
        if let Some(v) = line.strip_prefix("title:") {
            title = Some(unquote(v.trim()));
        } else if let Some(v) = line.strip_prefix("description:") {
            let v = unquote(v.trim());
            if !v.is_empty() {
                description = Some(v);
            }
        }
    }

    (title.unwrap_or_else(|| title_case(slug)), description)
}

/// Strip a single pair of surrounding single or double quotes.
fn unquote(s: &str) -> String {
    let s = s.trim();
    if s.len() >= 2
        && ((s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')))
    {
        s[1..s.len() - 1].to_string()
    } else {
        s.to_string()
    }
}

/// Turn a slug like `getting-started` into `Getting Started`.
fn title_case(slug: &str) -> String {
    slug.split('-')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    /// `read_front_matter` swallows a missing file, so a slug pointing at a
    /// deleted or renamed page yields a title-cased slug and a live link to a
    /// 404 with no error anywhere. Assert every listed page actually exists.
    #[test]
    fn test_every_listed_slug_resolves_to_a_real_page() {
        let docs_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("docs");
        for section in super::SECTIONS {
            for link in section.links {
                let path = docs_root.join(link.slug).join("index.md");
                assert!(
                    path.is_file(),
                    "llms.txt lists '{}' under '{}', but {} does not exist. The generator \
                     requires docs/<slug>/index.md specifically — a flat <slug>.md yields \
                     empty output and a broken link.",
                    link.slug,
                    section.heading,
                    path.display()
                );
                // Existing on disk is not enough: Jekyll skips `published: false`
                // pages entirely, so the link would still 404.
                let content = std::fs::read_to_string(&path).expect("page reads");
                assert!(
                    !content.contains("published: false"),
                    "llms.txt lists '{}', but {} is `published: false` — Jekyll will not \
                     build it and the link will 404.",
                    link.slug,
                    path.display()
                );
            }
        }
    }

    use super::*;

    #[test]
    fn test_unquote_strips_matching_quotes() {
        assert_eq!(unquote("\"Getting Started\""), "Getting Started");
        assert_eq!(unquote("'Relay'"), "Relay");
        assert_eq!(unquote("Kanban Workflow"), "Kanban Workflow");
        assert_eq!(unquote("\"\""), "");
    }

    #[test]
    fn test_title_case() {
        assert_eq!(title_case("getting-started"), "Getting Started");
        assert_eq!(title_case("cli"), "Cli");
    }

    #[test]
    fn test_render_item_with_and_without_desc() {
        assert_eq!(
            render_item("T", "https://x/", "d"),
            "- [T](https://x/): d\n"
        );
        assert_eq!(render_item("T", "https://x/", ""), "- [T](https://x/)\n");
    }

    #[test]
    fn test_generate_is_spec_shaped() {
        let out = LlmsTxtDocGenerator.generate().unwrap();

        // No YAML front matter — must be served verbatim, not wrapped in a layout.
        assert!(!out.starts_with("---"));
        assert!(!out.contains("layout:"));

        // Required H1 and blockquote summary.
        assert!(out.contains("# Operator!"));
        assert!(out.contains("> Operator! is a Rust TUI"));

        // Section headings and absolute links.
        assert!(out.contains("## Getting Started"));
        assert!(out.contains("(https://operator.untra.io/getting-started/)"));
        assert!(out.contains("## Reference"));
        assert!(out.contains("(https://github.com/untra/operator)"));

        // Every link is absolute (llms.txt requires fully-qualified URLs).
        for line in out.lines().filter(|l| l.starts_with("- [")) {
            assert!(line.contains("](https://"), "non-absolute link: {line}");
        }
    }
}

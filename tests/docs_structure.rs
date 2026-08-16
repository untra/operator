//! Structural alignment between the vertical catalog and the docs site.
//!
//! `docs/_data/navigation.yml` is hand-maintained (editorial ordering, section
//! titles) but its *contents* are enforced against
//! `operator::integrations::catalog` — every documented `Alpha`+ integration
//! must be reachable from the sidebar with its catalog icon, and the nav may
//! not advertise integrations the catalog doesn't know. The suite also guards
//! general docs hygiene: every nav URL resolves, every published page is
//! reachable, internal links resolve, and no page duplicates the layout's
//! front-matter title with a body H1.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use serde::Deserialize;

use operator::api::providers::model_server::ModelServerKind;
use operator::integrations::{all_integrations, SupportStatus, Vertical};

/// Catalog entries deliberately absent from the sidebar for now.
/// Deleting a pair from this list forces the corresponding nav addition.
const NAV_DEFERRED: &[(&str, &str)] = &[("workflows", "claude"), ("workflows", "agnt")];

/// Leaf URLs under a vertical section that are supporting pages rather than catalog integrations.
const NAV_EXTRA_PAGES: &[&str] = &[
    "/getting-started/git/provider-support/",
    "/getting-started/sessions/remote-hosts/", // execution-target concept page, not a session wrapper vertical
];

/// Published pages intentionally not linked from the sidebar.
const NAV_ORPHAN_ALLOWLIST: &[&str] = &[
    "index.md",                            // site home, hardcoded in the header
    "VERSION.md",                          // raw version endpoint
    "privacy-policy.md",                   // footer link
    "terms-of-service.md",                 // footer link
    "downloads/index.md",                  // hardcoded sidebar link
    "getting-started/kanban/jira-api.md",  // generated API appendix, linked from jira.md
    "getting-started/workflows/index.md",  // deferred nav section
    "getting-started/workflows/claude.md", // deferred nav section
    "getting-started/workflows/agnt.md",   // deferred nav section
];

/// Nav titles allowed to differ from the target page's front-matter title
/// (intentional short sidebar labels), keyed by URL.
const NAV_TITLE_EXCEPTIONS: &[(&str, &str)] = &[
    ("/getting-started/", "Overview"),
    ("/getting-started/platform-support/", "Platform Support"),
    ("/getting-started/sessions/tmux/", "tmux"),
    ("/getting-started/sessions/cmux/", "cmux"),
    ("/getting-started/sessions/zellij/", "Zellij"),
    ("/getting-started/sessions/vscode/", "VS Code Extension"),
    ("/cli/", "CLI"),
    ("/shortcuts/", "Shortcuts"),
    ("/schemas/", "Overview"),
    ("/schemas/config/", "Configuration"),
    ("/schemas/state/", "State"),
    ("/schemas/metadata/", "Ticket Metadata"),
];

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Nav {
    docs: Vec<Section>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Section {
    // Deserialized for schema strictness; sections are grouping-only in the sidebar.
    #[allow(dead_code)]
    title: String,
    children: Vec<Item>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Item {
    title: String,
    url: Option<String>,
    // Deserialized for schema strictness; rendered by the sidebar, not asserted on.
    #[allow(dead_code)]
    codicon: Option<String>,
    children: Option<Vec<Leaf>>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Leaf {
    title: String,
    url: String,
    icon: Option<String>,
}

fn repo_path(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

/// Deserializing through the strict structs is itself the shape test: it
/// rejects a 4th nesting level, `icon` on items, `codicon` on leaves, and any
/// unknown key — exactly what `docs/_includes/sidebar.html` would silently drop.
fn load_nav() -> Nav {
    let raw = std::fs::read_to_string(repo_path("docs/_data/navigation.yml"))
        .expect("docs/_data/navigation.yml should be readable");
    serde_yaml::from_str(&raw).expect("navigation.yml should match the sidebar's 3-level schema")
}

/// A docs URL resolves if it's `docs/<path>.md` or `docs/<path>/index.md`.
fn docs_exists(docs_path: &str) -> bool {
    let docs = repo_path("docs");
    docs.join(format!("{docs_path}.md")).exists() || docs.join(docs_path).join("index.md").exists()
}

fn url_resolves(url: &str) -> bool {
    let trimmed = url.trim_matches('/');
    if trimmed.is_empty() {
        return repo_path("docs/index.md").exists();
    }
    docs_exists(trimmed)
}

fn nav_urls(nav: &Nav) -> Vec<String> {
    let mut urls = Vec::new();
    for section in &nav.docs {
        for item in &section.children {
            urls.extend(item.url.clone());
            for leaf in item.children.iter().flatten() {
                urls.push(leaf.url.clone());
            }
        }
    }
    urls
}

/// All leaves grouped by their parent item URL.
fn leaves_by_item(nav: &Nav) -> BTreeMap<String, Vec<&Leaf>> {
    let mut map: BTreeMap<String, Vec<&Leaf>> = BTreeMap::new();
    for section in &nav.docs {
        for item in &section.children {
            if let (Some(url), Some(children)) = (&item.url, &item.children) {
                map.entry(url.clone()).or_default().extend(children.iter());
            }
        }
    }
    map
}

fn front_matter(content: &str) -> Option<&str> {
    let rest = content.strip_prefix("---\n")?;
    rest.split("\n---").next()
}

fn front_matter_title(content: &str) -> Option<String> {
    front_matter(content)?
        .lines()
        .find_map(|l| l.strip_prefix("title:"))
        .map(|t| t.trim().trim_matches('"').to_string())
}

fn body_after_front_matter(content: &str) -> &str {
    match content
        .strip_prefix("---\n")
        .and_then(|r| r.split_once("\n---"))
    {
        Some((_, body)) => body.trim_start_matches('-').trim_start(),
        None => content,
    }
}

/// Every published markdown page under `docs/`, as paths relative to `docs/`.
/// Skips Jekyll internals, excluded trees, and `published: false` pages.
fn published_pages() -> Vec<String> {
    const SKIP_DIRS: &[&str] = &[
        "_site",
        "_includes",
        "_layouts",
        "_data",
        "assets",
        "collections",
        "superpowers",
        "architecture",
        "vendor",
    ];
    let docs = repo_path("docs");
    let mut pages = Vec::new();
    let mut stack = vec![docs.clone()];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).expect("docs dir should be readable") {
            let path = entry.expect("dir entry").path();
            let name = path.file_name().unwrap().to_string_lossy().to_string();
            if path.is_dir() {
                if !(dir == docs && SKIP_DIRS.contains(&name.as_str())) {
                    stack.push(path);
                }
            } else if path.extension().is_some_and(|e| e == "md") {
                let content = std::fs::read_to_string(&path).expect("page should be readable");
                let unpublished = front_matter(&content)
                    .is_some_and(|fm| fm.lines().any(|l| l.trim() == "published: false"));
                if !unpublished {
                    let rel = path.strip_prefix(&docs).unwrap();
                    pages.push(rel.to_string_lossy().to_string());
                }
            }
        }
    }
    pages.sort();
    pages
}

/// URL a docs-relative page path serves under `permalink: pretty`.
fn page_url(rel: &str) -> String {
    let stem = rel.strip_suffix(".md").unwrap_or(rel);
    match stem
        .strip_suffix("/index")
        .or(if stem == "index" { Some("") } else { None })
    {
        Some("") => "/".to_string(),
        Some(dir) => format!("/{dir}/"),
        None => format!("/{stem}/"),
    }
}

#[test]
fn test_nav_urls_resolve() {
    let nav = load_nav();
    for url in nav_urls(&nav) {
        assert!(
            url_resolves(&url),
            "navigation.yml links '{url}' but no docs page exists for it"
        );
    }
}

#[test]
fn test_catalog_entries_in_nav() {
    let nav = load_nav();
    let by_item = leaves_by_item(&nav);
    for e in all_integrations() {
        let Some(docs_path) = e.docs_path else {
            continue;
        };
        if e.status < SupportStatus::Alpha || NAV_DEFERRED.contains(&(e.vertical.slug(), e.slug)) {
            continue;
        }
        let section_url = format!("/{}/", e.vertical.docs_section());
        let expected = format!("/{docs_path}/");
        let leaves = by_item.get(&section_url).unwrap_or_else(|| {
            panic!(
                "navigation.yml has no section item at '{section_url}' for the {} vertical",
                e.vertical.label()
            )
        });
        assert!(
            leaves.iter().any(|l| l.url == expected),
            "catalog entry '{}/{}' ({}) is missing from navigation.yml under '{section_url}' \
             (expected a leaf with url '{expected}')",
            e.vertical.slug(),
            e.slug,
            e.status.label()
        );
    }
}

#[test]
fn test_nav_vertical_leaves_map_to_catalog() {
    let nav = load_nav();
    let by_item = leaves_by_item(&nav);
    let section_urls: BTreeSet<String> = Vertical::ALL
        .iter()
        .map(|v| format!("/{}/", v.docs_section()))
        .collect();
    let catalog_urls: BTreeSet<String> = all_integrations()
        .iter()
        .filter_map(|e| e.docs_path.map(|p| format!("/{p}/")))
        .collect();
    for (item_url, leaves) in &by_item {
        if !section_urls.contains(item_url) {
            continue;
        }
        for leaf in leaves {
            assert!(
                catalog_urls.contains(&leaf.url) || NAV_EXTRA_PAGES.contains(&leaf.url.as_str()),
                "nav leaf '{}' ({}) under '{item_url}' advertises a page with no catalog entry — \
                 add it to src/integrations/catalog.rs or NAV_EXTRA_PAGES",
                leaf.title,
                leaf.url
            );
        }
    }
}

#[test]
fn test_catalog_icons_exist() {
    for e in all_integrations() {
        if let Some(icon) = e.icon {
            assert!(
                repo_path(&format!("docs/assets/icons/{icon}.svg")).exists(),
                "catalog entry '{}/{}' names icon '{icon}' but docs/assets/icons/{icon}.svg is missing",
                e.vertical.slug(),
                e.slug
            );
        }
        if e.status >= SupportStatus::Alpha && e.docs_path.is_some() {
            assert!(
                e.icon.is_some(),
                "documented Alpha+ entry '{}/{}' must declare a brand icon",
                e.vertical.slug(),
                e.slug
            );
        }
    }
}

#[test]
fn test_nav_icons_match_catalog() {
    let nav = load_nav();
    let by_item = leaves_by_item(&nav);
    let catalog: BTreeMap<String, Option<&'static str>> = all_integrations()
        .iter()
        .filter_map(|e| e.docs_path.map(|p| (format!("/{p}/"), e.icon)))
        .collect();

    let mut referenced: BTreeSet<String> = BTreeSet::new();
    for leaves in by_item.values() {
        for leaf in leaves {
            if let Some(icon) = &leaf.icon {
                referenced.insert(icon.clone());
                assert!(
                    repo_path(&format!("docs/assets/icons/{icon}.svg")).exists(),
                    "nav leaf '{}' references icon '{icon}' with no docs/assets/icons/{icon}.svg",
                    leaf.title
                );
            }
            if let Some(expected) = catalog.get(&leaf.url) {
                assert_eq!(
                    leaf.icon.as_deref(),
                    *expected,
                    "nav leaf '{}' ({}) icon must match the catalog entry's icon",
                    leaf.title,
                    leaf.url
                );
            }
        }
    }

    let icons_dir = repo_path("docs/assets/icons");
    for entry in std::fs::read_dir(icons_dir).expect("icons dir should be readable") {
        let name = entry
            .expect("dir entry")
            .file_name()
            .to_string_lossy()
            .to_string();
        if let Some(stem) = name.strip_suffix(".svg") {
            assert!(
                referenced.contains(stem),
                "docs/assets/icons/{name} is referenced by no navigation.yml entry — remove it or wire it up"
            );
        }
    }
}

#[test]
fn test_model_brand_icons_consistent() {
    for kind in ModelServerKind::ALL {
        if let Some(brand) = kind.brand_icon() {
            let entry = all_integrations()
                .into_iter()
                .find(|e| e.vertical == Vertical::Model && e.slug == kind.slug())
                .unwrap_or_else(|| panic!("model kind '{}' missing from catalog", kind.slug()));
            assert_eq!(
                entry.icon,
                Some(brand),
                "catalog icon for model '{}' must match ModelServerKind::brand_icon()",
                kind.slug()
            );
        }
    }
}

#[test]
fn test_nav_titles_match_pages() {
    let nav = load_nav();
    let exceptions: BTreeMap<&str, &str> = NAV_TITLE_EXCEPTIONS.iter().copied().collect();
    let check = |title: &str, url: &str| {
        if exceptions.get(url) == Some(&title) {
            return;
        }
        let trimmed = url.trim_matches('/');
        let docs = repo_path("docs");
        let file = if docs.join(format!("{trimmed}.md")).exists() {
            docs.join(format!("{trimmed}.md"))
        } else {
            docs.join(trimmed).join("index.md")
        };
        let content = std::fs::read_to_string(&file)
            .unwrap_or_else(|_| panic!("page for nav url '{url}' should be readable"));
        let page_title = front_matter_title(&content)
            .unwrap_or_else(|| panic!("page for nav url '{url}' has no front-matter title"));
        assert_eq!(
            title, page_title,
            "nav title for '{url}' differs from the page's front-matter title — \
             align them or add a NAV_TITLE_EXCEPTIONS entry"
        );
    };
    for section in &nav.docs {
        for item in &section.children {
            if let Some(url) = &item.url {
                check(&item.title, url);
            }
            for leaf in item.children.iter().flatten() {
                check(&leaf.title, &leaf.url);
            }
        }
    }
}

#[test]
fn test_docs_pages_reachable() {
    let nav = load_nav();
    let reachable: BTreeSet<String> = nav_urls(&nav).into_iter().collect();
    for page in published_pages() {
        if NAV_ORPHAN_ALLOWLIST.contains(&page.as_str()) {
            continue;
        }
        // The generated collections hub under docs/workflows/ is linked from
        // the hardcoded sidebar entry, not navigation.yml.
        if page.starts_with("workflows/") {
            continue;
        }
        let url = page_url(&page);
        assert!(
            reachable.contains(&url),
            "docs/{page} ({url}) is published but unreachable from navigation.yml — \
             add a nav entry or extend NAV_ORPHAN_ALLOWLIST"
        );
    }
}

#[test]
fn test_internal_links_resolve() {
    for page in published_pages() {
        let content = std::fs::read_to_string(repo_path(&format!("docs/{page}")))
            .expect("page should be readable");
        let mut in_fence = false;
        for (lineno, line) in content.lines().enumerate() {
            if line.trim_start().starts_with("```") {
                in_fence = !in_fence;
                continue;
            }
            if in_fence {
                continue;
            }
            let mut rest = line;
            while let Some(i) = rest.find("](/") {
                let after = &rest[i + 2..];
                let Some(end) = after.find(')') else { break };
                let target = after[..end].split(['#', '?']).next().unwrap_or("");
                rest = &after[end..];
                if target.is_empty() || target == "/" {
                    continue;
                }
                let as_file = repo_path(&format!("docs/{}", target.trim_start_matches('/')));
                assert!(
                    url_resolves(target) || as_file.is_file(),
                    "docs/{page}:{} links to '{target}' which resolves to no page or asset",
                    lineno + 1
                );
            }
        }
    }
}

#[test]
fn test_section_indexes_list_catalog_entries() {
    for e in all_integrations() {
        let Some(docs_path) = e.docs_path else {
            continue;
        };
        if e.status < SupportStatus::Alpha {
            continue;
        }
        let index = repo_path(&format!("docs/{}/index.md", e.vertical.docs_section()));
        let content = std::fs::read_to_string(&index).unwrap_or_else(|_| {
            panic!(
                "section index for {} vertical should exist at {}",
                e.vertical.label(),
                index.display()
            )
        });
        assert!(
            content.contains(&format!("/{docs_path}/")),
            "section index docs/{}/index.md does not link catalog entry '{}/{}' (/{docs_path}/)",
            e.vertical.docs_section(),
            e.vertical.slug(),
            e.slug
        );
    }
}

#[test]
fn test_no_duplicate_body_h1() {
    for page in published_pages() {
        let content = std::fs::read_to_string(repo_path(&format!("docs/{page}")))
            .expect("page should be readable");
        if front_matter_title(&content).is_none() {
            continue;
        }
        let body = body_after_front_matter(&content);
        let first_line = body.lines().find(|l| !l.trim().is_empty()).unwrap_or("");
        assert!(
            !first_line.starts_with("# "),
            "docs/{page} opens with a body H1 ('{first_line}') — the doc layout already \
             renders the front-matter title; remove the duplicate heading"
        );
    }
}

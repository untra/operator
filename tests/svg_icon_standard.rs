//! The Operator icon standard, enforced.
//!
//! Every icon Operator ships is a **Simple Icons-shaped glyph**: a single monochrome `<path>` on a 24×24 canvas that carries no color or size of its
//! own. That shape is what makes one file reusable across all four rendering surfaces at any size and in either theme.
//!
//! The rules, and why each one exists:
//!
//! | Rule | Why |
//! |------|-----|
//! | `viewBox="0 0 24 24"` | One coordinate space, so icons are interchangeable and align optically when mixed. |
//! | `role="img"` + one `<title>` | The glyph is content, not decoration; assistive tech needs a name. |
//! | Exactly one `<path>` | A single shape can be recolored, masked, or inlined as a unit. Multiple paths imply per-shape color the container cannot control. |
//! | No other elements | `<text>` depends on fonts that differ per surface; `<image>`/`<use>` pull in external documents; `<style>`/`<script>` do not survive inlining. |
//! | No `fill`/`stroke`/`style` | Color must come from `currentColor` so the icon inherits the theme. A hardcoded fill is invisible in one of light/dark. |
//! | No `width`/`height` | Size is the container's decision; a pinned size breaks every other usage. |
//! | No `on*` handlers or external refs | These files get inlined verbatim into generated pages. |
//!
//! Adding an icon: copy the shape from <https://simpleicons.org> where one exists, or draw a single path on a 24×24 grid.
//! Run `cargo test --test svg_icon_standard` before committing.

use std::path::{Path, PathBuf};

use regex::Regex;

/// Every directory whose SVGs are icons subject to the standard.
///
/// `icons/` is the canonical set; the others are the per-surface copies and
/// the collection glyphs.
const ICON_GLOBS: &[&str] = &[
    "icons",
    "docs/assets/icons",
    "ui/public/icons",
    "src/collections",
    "collections/community",
];

/// SVGs that are deliberately **not** icons, with the reason.
///
/// Keep this list short and justified: an exemption is a file no surface can
/// recolor or resize safely.
const EXEMPT: &[(&str, &str)] = &[(
    "docs/assets/img/operator_logo.svg",
    "full-color brand wordmark, not a monochrome glyph — it is never tinted, \
     inlined, or rendered at icon sizes",
)];

/// Elements that must never appear: anything that is not the single `<path>`
/// carrying the whole glyph.
const FORBIDDEN_ELEMENTS: &[&str] = &[
    "g",
    "rect",
    "circle",
    "ellipse",
    "line",
    "polyline",
    "polygon",
    "text",
    "tspan",
    "defs",
    "style",
    "script",
    "image",
    "use",
    "symbol",
    "mask",
    "clipPath",
    "linearGradient",
    "radialGradient",
];

/// Attributes that pin color or size and defeat reuse.
const FORBIDDEN_ATTRS: &[&str] = &["fill", "stroke", "style", "width", "height"];

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Every icon SVG under the governed directories, recursively (collection
/// icons live one level down, in `<id>/icon.svg`).
fn icon_files() -> Vec<PathBuf> {
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, out);
            } else if path.extension().is_some_and(|e| e == "svg") {
                out.push(path);
            }
        }
    }

    let root = repo_root();
    let mut files = Vec::new();
    for dir in ICON_GLOBS {
        walk(&root.join(dir), &mut files);
    }
    files.sort();
    assert!(
        !files.is_empty(),
        "found no icons under {ICON_GLOBS:?} — has the layout moved?"
    );
    files
}

/// Path relative to the repo root, for readable failure messages.
fn rel(path: &Path) -> String {
    path.strip_prefix(repo_root())
        .unwrap_or(path)
        .display()
        .to_string()
}

/// The `<title>` a collection icon must carry: its manifest's display name.
/// Non-collection icons only need *a* title, since their name is contextual.
fn expected_collection_title(path: &Path) -> Option<String> {
    let manifest = path.parent()?.join("collection.json");
    let json = std::fs::read_to_string(manifest).ok()?;
    let value: serde_json::Value = serde_json::from_str(&json).ok()?;
    Some(value["name"].as_str()?.to_string())
}

#[test]
fn test_every_icon_matches_the_operator_icon_standard() {
    let svg_re = Regex::new(r"(?s)^<svg\s([^>]*)>(.*)</svg>\s*$").unwrap();
    let title_re = Regex::new(r"(?s)<title>(.*?)</title>").unwrap();
    let path_re = Regex::new(r"<path\s([^>]*?)/>").unwrap();
    let attr_re = Regex::new(r#"([a-zA-Z-]+)\s*=\s*"[^"]*""#).unwrap();
    let handler_re = Regex::new(r"\bon[a-z]+\s*=").unwrap();

    for icon in icon_files() {
        let name = rel(&icon);
        let svg =
            std::fs::read_to_string(&icon).unwrap_or_else(|e| panic!("{name}: cannot read: {e}"));

        let caps = svg_re
            .captures(svg.trim())
            .unwrap_or_else(|| panic!("{name}: not a single <svg>…</svg> document"));
        let (root_attrs, body) = (&caps[1], &caps[2]);

        // 1. Root attributes: one coordinate space, and named for assistive tech.
        for required in [
            r#"role="img""#,
            r#"viewBox="0 0 24 24""#,
            r#"xmlns="http://www.w3.org/2000/svg""#,
        ] {
            assert!(
                root_attrs.contains(required),
                "{name}: <svg> is missing {required}"
            );
        }

        // 2. Exactly one <title>, non-empty.
        let titles: Vec<_> = title_re.captures_iter(&svg).collect();
        assert_eq!(titles.len(), 1, "{name}: expected exactly one <title>");
        assert!(
            !titles[0][1].trim().is_empty(),
            "{name}: <title> must not be empty"
        );

        // 3. Exactly one <path>, carrying a `d`.
        let paths: Vec<_> = path_re.captures_iter(body).collect();
        assert_eq!(
            paths.len(),
            1,
            "{name}: expected exactly one <path>; merge subpaths into a single d attribute"
        );
        assert!(
            paths[0][1].contains("d=\""),
            "{name}: <path> has no d attribute"
        );

        // 4. Nothing else in the document.
        for element in FORBIDDEN_ELEMENTS {
            assert!(
                !svg.contains(&format!("<{element} ")) && !svg.contains(&format!("<{element}>")),
                "{name}: <{element}> is not allowed; the glyph must be a single <path>"
            );
        }

        // 5. No pinned color or size — the container decides both.
        for caps in attr_re.captures_iter(&svg) {
            let attr = &caps[1];
            assert!(
                !FORBIDDEN_ATTRS.contains(&attr),
                "{name}: attribute '{attr}' is not allowed; icons inherit currentColor \
                 and are sized by their container"
            );
        }

        // 6. Nothing that breaks when the file is inlined into a page.
        assert!(
            !svg.contains("javascript:") && !handler_re.is_match(&svg),
            "{name}: event handlers and javascript: URLs are not allowed — these files \
             are inlined verbatim into generated pages"
        );
        assert!(
            !svg.contains("href="),
            "{name}: external references are not allowed; the glyph must be self-contained"
        );
    }
}

/// A collection's icon is shown beside its name everywhere it appears, so the
/// two must agree.
#[test]
fn test_collection_icons_are_titled_with_their_collection_name() {
    let title_re = Regex::new(r"(?s)<title>(.*?)</title>").unwrap();
    let mut checked = 0;

    for icon in icon_files() {
        let Some(expected) = expected_collection_title(&icon) else {
            continue; // Not a collection icon.
        };
        let svg = std::fs::read_to_string(&icon).unwrap();
        let actual = &title_re.captures(&svg).expect("has a title")[1];
        assert_eq!(
            actual,
            expected,
            "{}: <title> must equal the collection's manifest name",
            rel(&icon)
        );
        checked += 1;
    }

    assert!(
        checked >= 8,
        "expected to check every collection icon, saw {checked}"
    );
}

#[test]
fn test_every_collection_declares_and_ships_an_icon() {
    for base in ["src/collections", "collections/community"] {
        let Ok(entries) = std::fs::read_dir(repo_root().join(base)) else {
            continue;
        };
        for entry in entries.flatten() {
            let dir = entry.path();
            if !dir.join("collection.json").is_file() {
                continue;
            }
            let id = dir.file_name().unwrap().to_string_lossy().to_string();
            let manifest: serde_json::Value = serde_json::from_str(
                &std::fs::read_to_string(dir.join("collection.json")).unwrap(),
            )
            .unwrap();
            let declared = manifest["icon_path"]
                .as_str()
                .unwrap_or_else(|| panic!("collection '{id}' does not declare icon_path"));
            assert_eq!(
                declared, "icon.svg",
                "collection '{id}' declares icon_path '{declared}'; the flat collection \
                 layout expects a bare 'icon.svg' next to collection.json"
            );
            assert!(
                dir.join(declared).is_file(),
                "collection '{id}' declares '{declared}' but the file does not exist"
            );
        }
    }
}

/// Exemptions are a standing claim that a file cannot meet the standard. If one
/// starts complying, the exemption is stale and should go.
#[test]
fn test_exemptions_are_real_and_still_needed() {
    let svg_re = Regex::new(r"(?s)^<svg\s([^>]*)>").unwrap();

    for (path, reason) in EXEMPT {
        let full = repo_root().join(path);
        assert!(
            full.is_file(),
            "exempt file {path} no longer exists — remove it from EXEMPT ({reason})"
        );
        let svg = std::fs::read_to_string(&full).unwrap();
        let compliant = svg_re
            .captures(svg.trim())
            .is_some_and(|c| c[1].contains(r#"viewBox="0 0 24 24""#))
            && svg.matches("<path").count() == 1;
        assert!(
            !compliant,
            "{path} now meets the icon standard — drop its exemption and move it under \
             a governed directory"
        );
    }
}

/// The exempt list must not quietly cover a governed directory, which would
/// let an icon opt out of the standard by being listed here.
#[test]
fn test_exemptions_do_not_shadow_governed_directories() {
    for (path, _) in EXEMPT {
        for dir in ICON_GLOBS {
            assert!(
                !path.starts_with(dir),
                "{path} is exempt but sits under the governed directory {dir}; \
                 icons there must meet the standard"
            );
        }
    }
}

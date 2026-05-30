---
title: "Design System"
description: "Operator's brand palette, design tokens, and the consistency rules each rendering surface follows."
layout: doc
---

<span class="operator-brand">Operator!</span> presents one brand across four
rendering surfaces. This page is the human-readable companion to the brand
tokens — it explains *intent* the raw `:root` block can't, and records the rules
each surface follows so the look stays consistent as the codebase grows.

## Source of truth

All brand colors live in one file: **`docs/assets/css/tokens.css`**. It declares
the palette and the dark-mode overrides as CSS custom properties, and nothing
else re-declares them. Change a color there and both web surfaces follow.

```
docs/assets/css/tokens.css   ← single source of truth (:root + [data-theme="dark"])
   ├─ docs site:  linked in _includes/head.html, before main.css
   └─ embedded SPA: @import "../../docs/assets/css/tokens.css" in ui/src/index.css
```

## Palette

| Token | Light | Role |
|-------|-------|------|
| `--color-salmon` | `#e05d44` | Terracotta — primary brand, headings accents, CTAs |
| `--color-cornflower` | `#6688aa` | Muted blue — secondary/muted text, separators |
| `--color-cream` | `#f2eac9` | Warm accent / highlight surfaces |
| `--color-coral` | `#e05d44` | Links / accents (alias of salmon in light mode) |
| `--color-bg` | `#faf8f5` | Page background |
| `--color-white` | `#ffffff` | Base surface |
| `--color-green-l1` | `#66aa99` | Sage — nav buttons |
| `--color-green-l2` | `#448880` | Teal — hover / success |
| `--color-green-l3` | `#115566` | Deep pine — selected / primary text |
| `--color-green-l4` | `#082226` | Midnight — darkest |
| `--color-teal` | `#115566` | Body text (equals green-l3) |

Dark mode (`[data-theme="dark"]`) keeps salmon constant, brightens coral, and
inverts the green scale. See `tokens.css` for the exact dark values.

> **Naming note:** `--color-cornflower` was previously named
> `--color-salmon-dark`, which lied about its value (`#6688aa` is blue, not a
> dark salmon). It was renamed in lockstep across both web surfaces.

## Semantic tokens (SPA only)

The embedded SPA layers app-specific semantic tokens on top of the brand
palette, in `ui/src/index.css`. The docs site doesn't need these. Components
reference the semantic token, never the raw brand color:

`--surface`, `--surface-alt`, `--border`, `--text`, `--text-muted`,
`--danger` / `--danger-bg`, `--warning` / `--warning-bg`,
`--success` / `--success-bg`, plus layout tokens `--radius-sm|--radius|--radius-lg`
and `--font-sans|--font-mono`.

## The four surfaces

Each surface gets the rule that fits it — they are deliberately not styled
identically.

| Surface | Where | Rule |
|---------|-------|------|
| **Docs site** (Jekyll) | `docs/assets/css/main.css` | Links `tokens.css`; style with `var(--...)`, never raw hex. |
| **Embedded SPA** (Vite/React) | `ui/src/index.css` + `*.module.css` | Imports `tokens.css`; uses semantic tokens, never raw hex. |
| **Ratatui TUI** | `src/ui/*.rs` | Terminal can't render hex — map a semantic **role to ANSI** (danger→Red, success→Green, warning→Yellow, focus→Cyan). |
| **VS Code webview** (MUI) | `vscode-extension/webview-ui/` | Defers to the VS Code host theme; brand only as accents via `OPERATOR_BRAND`. Never overrides the editor theme. |

## Concept icons (codicons)

Each high-level Operator concept gets **one icon** so the same idea reads the same
across surfaces. The vocabulary is [codicons](https://github.com/microsoft/vscode-codicons)
— the icon set VS Code uses — chosen because the VS Code extension already renders
its tree with codicon `ThemeIcon`s. This table is the **single source of truth**:
consult it (and update it) whenever you give a concept an icon.

Each surface follows it by convention — there is no shared runtime registry:

- **Embedded SPA** reads it via `ui/src/concepts.ts` (`CONCEPTS[key].icon`), rendered
  by `ui/src/components/ConceptIcon.tsx`. The font is imported once in `main.tsx`.
- **Docs site** reads it via the `codicon:` field on items in
  `_data/navigation.yml`, emitted by `_includes/sidebar.html`. The vendored webfont
  is linked from `_includes/head.html` (`assets/css/codicon.css` + `assets/fonts/codicon.ttf`).
- **VS Code extension** already uses codicon `ThemeIcon`s directly.

This is **additive** — distinct from the issue-type `glyph`→icon map in
`vscode-extension/src/issuetype-service.ts` and the `glyph_for_key`/`color_for_key`
helpers in `src/templates/mod.rs` (documented below). It follows the same
"central key → presentation" pattern, keyed by section concept.

| Concept (key) | Icon | codicon | SPA | Docs |
|---------------|:----:|---------|:---:|:----:|
| dashboard | <i class="codicon codicon-dashboard"></i> | `dashboard` | ✓ | |
| queue | <i class="codicon codicon-list-ordered"></i> | `list-ordered` | ✓ | |
| config (Configuration) | <i class="codicon codicon-settings-gear"></i> | `settings-gear` | ✓ | ✓ |
| connections | <i class="codicon codicon-plug"></i> | `plug` | ✓ | |
| kanban | <i class="codicon codicon-layout"></i> | `layout` | ✓ | ✓ |
| llm (LLM Tools) | <i class="codicon codicon-sparkle"></i> | `sparkle` | ✓ | ✓ |
| model-servers | <i class="codicon codicon-server"></i> | `server` | ✓ | |
| git | <i class="codicon codicon-git-branch"></i> | `git-branch` | ✓ | |
| issuetypes (Issue Types) | <i class="codicon codicon-issues"></i> | `issues` | ✓ | ✓ |
| delegators | <i class="codicon codicon-rocket"></i> | `rocket` | ✓ | |
| projects (Managed Projects) | <i class="codicon codicon-project"></i> | `project` | ✓ | |
| agents | <i class="codicon codicon-robot"></i> | `robot` | | ✓ |
| tickets | <i class="codicon codicon-note"></i> | `note` | | ✓ |
| taxonomy | <i class="codicon codicon-type-hierarchy"></i> | `type-hierarchy` | | ✓ |
| schemas | <i class="codicon codicon-bracket"></i> | `bracket` | | ✓ |
| shortcuts | <i class="codicon codicon-keyboard"></i> | `keyboard` | | ✓ |
| cli | <i class="codicon codicon-terminal"></i> | `terminal` | | ✓ |
| design-system | <i class="codicon codicon-symbol-color"></i> | `symbol-color` | | ✓ |

Keys match the `SectionId` serde renames in `src/ui/status_panel.rs` (and the SPA's
section ids). Every codicon name is unique — the Kanban-vs-Managed-Projects collision
was resolved as kanban→`layout`, projects→`project`.

> **Attribution:** codicon **icons** are licensed [CC-BY-4.0](https://github.com/microsoft/vscode-codicons/blob/main/LICENSE);
> the font/CSS **code** is MIT, © Microsoft. The webfont is vendored under
> `docs/assets/` and bundled into the SPA via `@vscode/codicons`.

## Issue type glyphs & colors

Issue type color + glyph are defined once in the collection JSON schemas and
read through `color_for_key` / `glyph_for_key` in `src/templates/mod.rs`. Reuse
those helpers — do not re-hardcode the mapping in new UI.

| Type | Glyph | Color |
|------|-------|-------|
| FEAT | `*` | green |
| FIX | `#` | magenta |
| TASK | `>` | cyan |
| SPIKE | `?` | blue |
| INV | `!` | yellow |
| ASSESS | `~` | magenta |
| SYNC | `@` | blue |
| INIT | `%` | green |

## Priority colors

Priority maps to a single role across surfaces: P0 → danger (red), P1 → warning
(yellow/gold), P2 → sage green, P3 → muted border. In the SPA these resolve to
`--danger` / `--warning` / `--color-green-l1` / `--border`
(`ui/src/components/KanbanBoard.module.css`); in the TUI to the nearest ANSI
(`src/ui/panels.rs`).

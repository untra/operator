# operator/ui

The embedded web UI for Operator — a [Vite](https://vite.dev) + React 19 single-page
app that talks to the operator REST API (`/api/v1/*`). It is one of Operator's **four
rendering surfaces** (alongside the Ratatui TUI, the Jekyll docs site, and the VS Code
webview); see the root `CLAUDE.md` "Design & UI Consistency" section for how they stay
consistent.

At runtime this SPA is compiled and **baked into the Rust binary** — there is no separate
web server to deploy. The TUI opens it in a browser (or the VS Code extension hosts it in a
webview).

## Toolchain (bun)

This package uses [bun](https://bun.sh) (`bun.lock`). The npm scripts wrap Vite/tsc:

```bash
bun install            # install dependencies
bun run dev            # Vite dev server on http://127.0.0.1:5173
                       #   proxies /api and /swagger-ui → http://127.0.0.1:7008
                       #   (run the operator REST server separately: cargo run -- serve)
bun run build          # production build → ui/dist
bun run typecheck      # tsc --noEmit
bun run preview        # preview the production build
```

For local development run the operator REST server on its default port (7008) so the dev
server's `/api` proxy has something to talk to.

## How it embeds into the Rust binary

`src/rest/web_ui.rs` uses [`rust-embed`](https://crates.io/crates/rust-embed) with
`#[folder = "ui/dist"]`, gated behind the **`embed-ui`** cargo feature. So:

1. `bun run build` writes the hashed assets (including the codicon `.ttf`) into `ui/dist`.
2. `cargo build --features embed-ui` embeds that directory into the binary; the SPA is then
   served offline by `spa_handler`, with `index.html` as the client-side-routing fallback.

If `ui/dist` wasn't built, `build.rs` writes a placeholder so the TUI can show an actionable
message instead of a blank page. There are size-budget tests in `web_ui.rs` (10 MB gzipped /
15 MB uncompressed) — keep new assets well under them.

## Routing model

[`HashRouter`](src/main.tsx) (`#/path`), because the app is served from a `file:`-style
embedded context. Two kinds of routes:

- **Status sections** — one route per concept in the `SectionId` model shared with the TUI
  and VS Code extension (`src/ui/status_panel.rs`): `#/config`, `#/connections`, `#/kanban`,
  `#/llm`, `#/model-servers`, `#/git`, `#/issuetypes`, `#/delegators`, `#/projects`. Each
  renders the live status of that section from `GET /api/v1/sections`. `#/status` is the
  "all sections" overview (reachable from the Dashboard).
- **Web-only pages** — `#/` (Dashboard) and `#/queue`, which have no section analog.

The sidebar (`src/Layout.tsx`) reflects each section's health and gates not-yet-available
sections (disabled with a tooltip naming the unmet prerequisites). Section data is polled
once by `SectionsProvider` (`src/sections-context.tsx`) and shared by the sidebar and pages.

## Styling

Brand colors come from the single shared source of truth,
[`docs/assets/css/tokens.css`](../docs/assets/css/tokens.css), imported by
[`src/index.css`](src/index.css). On top of that palette `index.css` layers app-only
**semantic tokens** (`--surface`, `--border`, `--text`, `--danger`, `--warning`, `--success`,
radii, fonts) with light/dark variants. Components use **CSS Modules** (`*.module.css`) and
reference semantic tokens — never raw hex (per `CLAUDE.md`).

## Icons

Sidebar and page icons use [`@vscode/codicons`](https://github.com/microsoft/vscode-codicons)
(MIT-licensed code, **CC-BY-4.0 icons**), the same vocabulary as the VS Code extension. The
font is imported once in `src/main.tsx`; Vite fingerprints `codicon.ttf` into `dist/assets`,
so it stays embedded and offline. The concept→icon mapping lives in
[`src/concepts.ts`](src/concepts.ts) and follows the **canonical table** documented at
[`/design-system/`](../docs/design-system/index.md) — the single place to consult or update
when giving an operator concept an icon.

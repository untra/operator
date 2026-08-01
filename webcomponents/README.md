# @operator/webcomponents

Shared frontend components rendered by **both** operator surfaces:

| Consumer | How it consumes this package |
|---|---|
| `ui/` — the embedded React SPA | Imports the **React** entry (`dist/index.js`) through a path alias, exactly like `@operator/bindings`. React stays external so the SPA keeps one React instance. |
| `docs/` — the static Jekyll site | Loads the **custom-elements** entry (`dist/elements.js` + `.css`), which bundles React in. Generated markdown writes plain tags; the docs site has no build step of its own. |

The point is that there is one implementation. An Operator workflow drawn in
the app and the same workflow drawn on the docs site come from the same source
over the same bytes, so they cannot disagree.

## What belongs here

Anything the docs site and the SPA both need to render. Today that is the
workflow graph and the collection-catalog search; new shared JS should land
here rather than being written twice.

What does *not* belong here: page chrome and layout owned by one surface. The
collection card grid and table are rendered by the Rust docs generator and
styled by `docs/assets/css/main.css`; this package only enhances them.

## Layout

```
src/
├── index.ts                    React entry (ui/)
├── elements.ts                 Custom-elements entry (docs/)
├── elements.css                Styles for the DOM these components create
├── workflow/
│   ├── types.ts                The Operator workflow document shape
│   ├── issuetype-to-ir.ts      steps[] → flat node/edge graph
│   └── WorkflowGraph.tsx       The renderer
├── elements/                   Custom-element wrappers
└── shared/theme.ts             data-theme + brand-token plumbing
```

## The native projection

`issuetype-to-ir.ts` reads an **Operator workflow** — the issue type's
`steps[]`, which is the native JSON the runtime executes — and projects it onto
the flat graph `@untra/naiveworkflow-react` draws. It deliberately does *not*
render a Claude or AGNT export: those are lossy formats derived from this same
source, and drawing one would present an export target as the source of truth.

Step ordering mirrors `ordered_steps` in `src/workflow_gen/export.rs`, so the
graph matches the order the runtime actually executes.

Drift protection lives in `tests/webcomponents_workflow_parity.rs`: adding a
step type or a graph-bearing step field in Rust fails that test until the mapper
handles it.

## Theming

Colors come from the shared brand tokens in `docs/assets/css/tokens.css`, which
both surfaces already load, and components observe `data-theme` on `<html>`.
Never declare a brand hex here — see `docs/design-system/`.

## Commands

```bash
bun install
bun run typecheck
bun test         # mapper tests, run against the real src/collections/ fixtures
bun run build    # dist/index.js (+ .d.ts) and dist/elements.js (+ .css)
```

`dist/` is gitignored. `make webcomponents` runs the full sequence, and both
`make ui` and `make docs` depend on it so the artifact always exists before its
consumers build.

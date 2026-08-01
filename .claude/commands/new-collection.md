---
description: Author a new Operator issuetype collection — a shareable AI workflow
allowed-tools: Bash, Read, Write, Edit, Glob, Grep
---

# Author a New Collection

Create a new workflow **collection**: a named, versioned bundle of issue types that encodes a coherent process of doing software development with AI agents.

The mechanics are easy; the design is the hard part. workflows allow agents to apply themselves in a deterministic manner.

## Vocabulary — get this right first

Three terms that are easy to conflate:

| Term | What it is |
|---|---|
| **Operator workflow** | The step graph itself: an ordered set of typed steps with review gates and reject edges. Lives in an issue type's `steps`. This is the native format. |
| **Issue type** | One kind of work (`FEAT`, `PRD`, `ELVSTAGE`). Carries identity, input `fields`, and exactly one Operator workflow. |
| **Collection** | A bundle of issue types that work together. What you are authoring. |

Kanban issue types describe how a *team labels* work; a collection describes how the *agents do* the work.

### Other non-Operator "workflows"

the term _"Workflow"_ is overloaded in the AI space. Operator workflows are json described collections of issuetypes. they can export to other workflows, such as `claude workflows` among others. As a result Operator workflows are meant to compose as broad and neutral as possible, with some opinions about structure and behavior. Define your workflows in the context of the work you want to get done.

## Step 1 — Decide what loop you are encoding

Answer these before opening an editor. If you cannot answer them crisply, the
collection is not ready to write.

1. **What is the loop?** What repeats, and what ends it? "Implement one
   right-sized story per fresh agent context" (`ralph_loop`) is a loop.
   "Do software development" is not.
2. **What are the work types?** Each issue type must be a *genuinely different
   shape of work*, not a different priority or label. If two types have the
   same steps, they are one type with a field.
3. **Where does state live between steps?** Agents lose context. Name the
   files or surfaces that carry memory forward (`workflow_hints.memory_surfaces`).
4. **What are the gates?** Where must a human or a test approve before
   continuing, and what happens on rejection?
5. **When does it stop?** Both success and give-up conditions.

Study the shipped collections before inventing a shape — they are short and
each encodes a real, published methodology:

```bash
ls src/collections/                       # simple, dev_kanban, devops_kanban, operator,
                                          # ralph_loop, jr_orchestration, elves_overnight
cat src/collections/ralph_loop/collection.json
cat src/collections/dev_kanban/FEAT.json  # the canonical multi-step example
```

## Step 2 — Create the directory

Official/curated collections that ship in the binary live in
`src/collections/<id>/`. Community contributions live in
`collections/community/<id>/` and are hosted-only.

**Pick `src/collections/` only if the collection should be available offline
to every user.** When in doubt, use `collections/community/`.

```
<id>/
├── collection.json    # the manifest
├── icon.svg           # Simple Icons-shaped glyph
├── <KEY>.json         # one per issue type — the Operator workflow
└── <KEY>.md           # optional ticket template per issue type
```

`<id>` must match `^[a-z0-9_]{3,64}$` and equal the directory name.

## Step 3 — Write the issue types

One `<KEY>.json` per issue type. `KEY` matches `^[A-Z][A-Z0-9_]{1,15}$` —
**no hyphens**, because the hyphen separates the key from the ticket number in
`FEAT-123-project-summary.md`.

Start from the JSON Schema and a real example:

```bash
cat src/schemas/issuetype_schema.json     # the contract
cat src/collections/ralph_loop/STORY.json # a 5-step autonomous workflow
```

Required top-level fields: `key`, `name`, `description`, `mode`, `glyph`,
`fields`, `steps`. Also set `"$schema": "../../schemas/issuetype_schema.json"`
so editors validate as you type.

- **`mode`** — `autonomous` (launch and monitor; several run in parallel) or
  `paired` (needs you in the loop; one at a time). This is a real scheduling
  constraint, not a hint. Choose `paired` only when a human genuinely must
  participate throughout.
- **`glyph`** — one character shown in the TUI. Already in use across
  collections: `! # % * > ? @ B E F J L P R S T V ~`. Pick something unused and
  mnemonic.
- **`color`** — one of `cyan`, `green`, `blue`, `magenta`, `yellow`, `red`.
- **`fields`** — the ticket's inputs. Types: `string`, `text`, `enum`, `bool`,
  `date`, `integer`. Use `"auto": "id" | "date" | "branch" | "status"` for
  values Operator fills in, and mark those `"user_editable": false`.

### Designing the steps

Steps are where the methodology actually lives. Each step is one agent session.

```jsonc
{
  "name": "plan",                    // lowercase identifier
  "display_name": "Planning",        // shown in the UI
  "outputs": ["plan"],               // plan|code|test|pr|ticket|review|report|documentation
  "prompt": "...",                   // Handlebars over the ticket's fields: {{ summary }}
  "allowed_tools": ["Read", "Grep"], // least privilege for this step
  "artifact_patterns": [".tickets/plans/{{ id }}.md"],  // files that signal completion
  "review_type": "plan",             // none|plan|visual|pr — a gate
  "on_reject": { "goto_step": "plan", "prompt": "Plan rejected: {{ rejection_reason }}..." },
  "next_step": "build"               // omit on the final step
}
```

Rules that matter:

- **Chain with `next_step`.** Ordering follows the chain from the first step,
  then appends anything unreached. Do not rely on array order alone.
- **`on_reject.goto_step` is the retry edge** — it may point backwards, and
  usually should point at the step that can actually fix the problem (a failed
  PR review goes back to `code`, not to `plan`).
- **One step, one job.** A step that plans *and* implements *and* tests gives
  the agent no checkpoint and no place to fail cleanly.
- **Scope `allowed_tools` per step.** A planning step should not have `Write`
  to source. This is the main safety control you have.
- **Prompts are Handlebars** over the ticket's fields. Reference only fields
  you actually declared.

Beyond plain `task` steps, these types exist — use them when the shape calls
for it, not for novelty: `classifier`, `rag`, `delegator`, `mcp`,
`multi_model` (fan out, then vote), `multi_prompt`, `matrixed`, `pipeline`.

### Ticket templates

`<KEY>.md` is the markdown scaffold for a new ticket, with YAML frontmatter
and Handlebars placeholders. Copy the shape from an existing one:

```bash
cat src/collections/dev_kanban/FEAT.md
```

## Step 4 — Write the manifest

```jsonc
{
  "schema_version": 1,
  "id": "<id>",                     // must equal the directory name
  "name": "Display Name",
  "description": "One line. What loop is this, in plain language.",
  "version": "1.0.0",
  "publisher": "untra",
  "author": "you-or-the-methodology-author",
  "url": "https://github.com/...",  // where the methodology comes from
  "license": "MIT",
  "tags": ["agentic-loop", "..."],
  "tier": "community",              // or "official" for src/collections/
  "icon_path": "icon.svg",
  "created": "YYYY-MM-DD",
  "updated": "YYYY-MM-DD",
  "issue_types": [                  // display order; also priority order
    { "key": "PRD", "schema_path": "PRD.json", "template_path": "PRD.md" }
  ],
  "workflow_hints": {
    "loop_kind": "fresh_context_story_loop",
    "memory_surfaces": ["docs/plan.md"],
    "review_gates": ["plan_review", "test_suite"],
    "external_tools": ["git", "gh"],
    "stop_conditions": ["all stories pass", "budget exhausted"],
    "runner_semantics": "prompt_driven"
  },
  "default_selected": ["PRD"]
}
```

**Do not write `checksum` or `schema_checksum`** — the docs generator computes
them at publish time. `tier: "community"` additionally requires `author`,
`url`, `license`, and `icon_path`.

`workflow_hints` is descriptive metadata (v1 does not execute it) but it is
what the catalog page displays, so it is how a reader decides whether to adopt
your collection. Write it for them, not for the parser.

## Step 5 — Draw the icon

A single-path 24×24 glyph. The full rules and rationale are in
`docs/design-system/` under "Brand & collection icons"; the short version:

```svg
<svg role="img" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><title>Display Name</title><path d="..."/></svg>
```

No `fill`, `stroke`, `width`, or `height` — the icon inherits `currentColor`
and its container's size. The `<title>` must equal the manifest's `name`.

Render it and *look at it* before trusting it — hand-authored path data is easy
to get subtly wrong, and the test checks shape, not whether the glyph reads:

```bash
# whichever is available
rsvg-convert -w 96 -h 96 -b white <id>/icon.svg -o /tmp/icon.png
magick -background white -density 384 <id>/icon.svg /tmp/icon.png
```

Then open `/tmp/icon.png`. If neither tool is installed, open the SVG in a
browser.

## Step 6 — Register it (embedded collections only)

Skip this for `collections/community/`. For `src/collections/<id>/`, add an
entry to `EMBEDDED_COLLECTIONS` in `src/collections/mod.rs`, following the
existing entries exactly — `manifest`, `icon_svg`, and one `EmbeddedIssueType`
per key, in the same order as the manifest.

## Step 7 — Validate

Run these in order and fix anything that fails. Do not skip ahead.

```bash
# Community collections: schema, key grammar, paths, attribution, referenced files
cargo test --test community_collections

# Icon shape
cargo test --test svg_icon_standard

# Everything: manifest parsing, embedded/manifest ordering, checksums, generators
make check

# Publish the bundle and the catalog page
cargo run -- docs
```

Then look at the result:

```bash
make docs
cd docs/_site && python3 -m http.server 4100
```

Open `http://localhost:4100/workflows/` — your collection should appear as a
card — then its page, and step through each issue type's graph. **A workflow
that looks wrong as a graph is wrong.** Disconnected nodes, a reject edge
pointing somewhere useless, or a 12-step chain with no gates are all visible
at a glance and all worth fixing before shipping.

## What good looks like

Before opening a PR, check the collection against its own claims:

- Could someone adopt this without reading the source? The description and
  `workflow_hints` should be enough.
- Does each issue type earn its place, or is one of them a field on another?
- Does every reject edge point at a step that can actually fix the problem?
- Is `allowed_tools` scoped per step, or did every step get `["*"]`?
- Does the loop actually terminate, and is that visible in `stop_conditions`?

Submissions are reviewed for prompt quality and safety, not just schema
validity. A good collection describes a workflow shape worth sharing: what loop
it runs, what memory it keeps, what gates it enforces, and when it stops.

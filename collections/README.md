# Community Collections

This directory hosts **community-contributed issuetype collections** — shareable
AI workflow shapes that operator instances can browse and install from
[operator.untra.io/collections](https://operator.untra.io/collections/).

Community collections are **hosted-only**: they are published to the docs site
by the docs generator but are never compiled into the operator binary. The
curated embedded set lives in `src/collections/`.

## Contributing a collection

1. Create `collections/community/<id>/` where `<id>` matches
   `^[a-z0-9_]{3,64}$` (e.g. `gastown_loop`).
2. Add a `collection.json` manifest conforming to
   [the collection schema](https://operator.untra.io/collections/schema.json):
   - `schema_version: 1`
   - `id` equal to the directory name
   - `tier: "community"` with **`author`, `url`, and `license`** (SPDX id) —
     required for community submissions
   - `issue_types`: 1–32 entries; keys match `^[A-Z][A-Z0-9_]{1,15}$`
     (hyphens are reserved for the `{KEY}-{number}` ticket-id separator);
     paths are bare filenames next to the manifest
   - optional `workflow_hints` (loop shape, memory surfaces, review gates,
     stop conditions) and `kanban_defaults.suggested_type_mappings`
     (descriptive only — they inform users and onboarding, not execution)
3. Add one `<KEY>.json` per issuetype conforming to
   [the issuetype schema](https://operator.untra.io/schemas/issuetype.json),
   plus an optional `<KEY>.md` ticket template.
4. Do **not** set checksums — the docs generator computes them at publish time.
5. Run the CI gate locally before opening a PR:

   ```bash
   cargo test --test community_collections
   ```

See `community/example_chores/` for a minimal working example.

## Review expectations

Submissions are reviewed for prompt quality and safety, not just schema
validity. A good collection describes a *workflow shape worth sharing*:
what loop it runs, what memory it keeps, what gates it enforces, and when
it stops.

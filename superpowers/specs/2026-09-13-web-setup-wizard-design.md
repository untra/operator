# Web Setup Wizard — design spec (D2 / D3)

**Status:** ready to plan. Phase A (TUI alignment) is landed and verified.
**Audience:** the agent that will plan and implement this. Read this whole file
before writing a plan; it encodes decisions already made and traps already hit.

**Related:** `superpowers/plans/2026-09-12-getting-started-parity-phase-a.md` —
the completed phase, including its audit and implementation log. This spec
supersedes that plan's "Deferred (stages 6–10)" section, which was written
before Phase A landed and is stale in two places called out below.

---

## 1. Goal

- **D2** — a multi-step first-run wizard in the web SPA that mirrors the TUI
  wizard and writes the same `config.toml`.
- **D3** — **either** flow, run alone, ends with a `config.toml` the other
  surface can use. This is the acceptance bar, not a nice-to-have.

Out of scope here: documentation (D4). Docs come after this lands, per the
user's sequencing — "document as it exists". Do not write getting-started docs
as part of this work.

## 2. Why this exists

An audit found the two surfaces were not merely inconsistent: **the web flow
was not independently completable.** Its only guided screen is the admin-password
bootstrap; kanban and git are unreachable from the browser entirely, and the
pages that do work were gated behind a prerequisite only the TUI could satisfy.

Phase A fixed the TUI half. The wizard now has the three external-service
connections it was missing, all derived from one catalog. This phase brings the
browser to parity.

## 3. What Phase A already built (do not re-derive)

### 3.1 The step catalog is the source of truth

`src/startup/steps.rs` — `SetupStep`, 17 variants, declared **in wizard order**:

```
welcome, kanban-info, model-server, git-provider, collection-source,
hosted-collections, task-field-config, session-wrapper-choice,
worktree-preference, admin-password, tmux-onboarding, vscode-setup,
cmux-setup, zellij-setup, acceptance-criteria, startup-tickets, confirm
```

- `SetupStep::ALL` is a fixed-size array in walk order. `info()` and `slug()`
  are **exhaustive matches**, so adding a variant is a compile error until copy
  exists and `ALL` is resized.
- `#[derive(TS)] #[ts(export)]` generates `bindings/SetupStep.ts` (a string
  union of the slugs above) via `make bindings`.
- `setup_steps()` feeds `docs/startup/index.md` through `src/docs_gen/startup.rs`.

The slugs are a **frozen public identifier** — they key docs URLs and will key
your component map. `test_slugs_match_frozen_snapshot` guards renames.

### 3.2 One workspace writer

`setup::initialize_workspace(&mut Config, &SetupOptions) -> Result<SetupResult>`
creates directories, writes templates, and **mutates config without saving**.
The caller owns persistence. This was done specifically so a REST handler can
wrap it in `ApiState::mutate_config`. `SetupOptions` today:

```rust
preset, force, task_fields, working_dir, kanban_provider, llm_tool,
use_worktrees, wrapper, acceptance_criteria, custom_collection,
active_collection, hosted_collections, model_servers
```

`App::initialize_tickets` builds one of these from the wizard screen and calls
it. Your REST endpoint must do the same, from a DTO.

### 3.3 Provider lists come from the integration catalog

`integrations::catalog::onboardable(vertical) -> Vec<CatalogEntry>` returns
`Alpha`+ **and** documented entries. Both TUI provider steps derive rows,
labels and docs links from it.

| Vertical | Offered today |
|---|---|
| Git | github, gitlab, gitea |
| Model | anthropic-api, openai-api, google-api, ollama, openrouter |

Proto entries (bitbucket, azure, forgejo, openai-compat, lmstudio) are
deliberately **not** offered — they are unadvertised and have no docs page to
link to. Promoting one into onboarding is a `SupportStatus` bump in
`src/integrations/catalog.rs`, nothing else.

**Your web wizard must use this same list.** Three guards in
`tests/setup_parity.rs` already enforce it for the TUI; extend them, do not
work around them:

- `test_wizard_steps_do_not_hardcode_provider_slugs`
- `test_wizard_derives_provider_lists_from_the_catalog`
- `test_binding_union_matches_catalog_slugs`

### 3.4 Secrets are stored by reference, everywhere

Kanban, git and model servers all store the **name of an env var**, never the
secret. The secret is `set_var`'d into the running process and the user is shown
a shell-export line to make it permanent. Preserve this. It is also the single
thing users trip over, so the web wizard should surface the export line at least
as prominently as the TUI does.

## 4. Load-bearing constraints

Each of these cost real time to discover. Respect them or re-pay that cost.

### 4.1 `setup` and `startup` are bin-only — this blocks the obvious approach

```
src/lib.rs :  mod startup;          (private)   pub mod integrations;
src/main.rs:  mod setup;  mod startup;  mod integrations;
```

`src/rest/` compiles **in the lib**, so it cannot see `crate::setup` or
`crate::startup` as they are declared today. A `POST /setup/initialize` that
calls `initialize_workspace` **will not compile** without first moving
`src/setup.rs` into the lib (`pub mod setup;`) and widening `startup`.

The Phase A plan's deferred section assumed this was free. It is not. Budget it
as the first task of the REST work, and check what else `setup.rs` pulls in
(`projects`, `startup::templates`, `collections::fetch` are all lib-private
today — visibility widening, not relocation).

`integrations` is already `pub mod`, which is why §3.3 routes cleanly to the
browser with no move at all.

### 4.2 Mount the wizard OUTSIDE `Layout`

`ui/src/Layout.tsx:22` disables any nav item whose `section.met` is false. A
wizard rendered inside `Layout` would be surrounded by the very gating it exists
to resolve. `ui/src/main.tsx` already has the precedent and states the reason:

```tsx
{/* Unauthenticated screens render outside Layout: the shell's own
    API calls would 401 for a visitor who cannot yet authenticate. */}
<Route path="login" … /> <Route path="setup" … />
```

Add `onboarding` as a sibling of `setup`, not inside the `<Route element={<Layout />}>` block.

### 4.3 LLM detection runs only in the TUI

`crate::llm::refresh_tool_detection` is called exactly once, at
`src/app/mod.rs:121`. `operator api` never detects tools, so `llm_tools.detected`
stays empty, the `llm` section stays Yellow, and **Model Providers + Delegators
are disabled in the web sidebar** — on real, working routes.

Fix this by hoisting detection into a shared entry point called from `cmd_api`
as well. It is a **user-visible fix worth shipping on its own**, independent of
the wizard, and the wizard's Welcome step needs it to show anything useful.

Cost is bounded: `refresh_cached_tool` reuses cached path/version, and the
config write is gated on `detection_changed`.

### 4.4 Blocking calls in async handlers

`git_onboarding::resolve_onboarding*` shells out via `std::process::Command`,
and `validate_*_token` uses `reqwest::blocking`. Both are fine in the TUI (they
briefly stall the event loop). In an axum handler they block a runtime worker,
and `reqwest::blocking` can panic when constructed inside a tokio context.

**Wrap every such call in `tokio::task::spawn_blocking`.** This is the largest
correctness risk in the REST work. Prefer wrapping over writing async twins, so
one implementation stays shared with the TUI.

### 4.5 The SPA dependency allowlist

`tests/ui_packaging.rs` enforces a **7-entry** allowlist for `ui/package.json`
(react, react-dom, react-router-dom, three @dnd-kit packages, @vscode/codicons)
and bans CSS-in-JS. Build the wizard with plain React and CSS modules, matching
`SetupPage.tsx` / `AuthPage.module.css`. Reaching for a form or wizard library
fails the build.

### 4.6 Config is written relative to `paths.state`

`Config::save()` writes `config.operator_config_path_for()` = `paths.state` +
`config.toml` (default `.tickets/operator/config.toml`, resolved against the
process cwd). `operator api` started from a different directory silently uses a
different config. Mention this in whatever the wizard shows as its destination
path — `GET /setup/status` should return the resolved absolute path.

### 4.7 `ApiState::mutate_config` is the only REST write path

```rust
pub async fn mutate_config<T>(&self, mutate: impl FnOnce(&mut Config) -> Result<T, ApiError>)
    -> Result<T, ApiError>
```

It serialises on a lock, saves atomically, **and** `replace_config`s so later
handlers do not see a stale snapshot. Never call `config.save()` directly from a
handler.

### 4.8 Route scopes fail closed

`tests/route_scope_parity.rs` walks the generated OpenAPI spec and fails on any
route missing from `ROUTE_RULES` in `src/auth/scope.rs` (`read` / `execute` /
`admin` helpers). Add an entry per new route or the test fails — which is the
desired behaviour, since an unknown route denies.

**Auth needs no other work.** Browser session principals carry `Scope::ALL`
(`src/auth/store.rs:462`), so a post-bootstrap admin satisfies every rule the
wizard touches.

## 5. What already exists vs. what is new

### Already built — zero or near-zero Rust work

| Concern | Endpoints | Note |
|---|---|---|
| Admin bootstrap | `GET`/`POST /api/v1/auth/bootstrap`, `POST /auth/login` | already drives `SetupPage.tsx` |
| **Kanban** | `POST /kanban/validate`, `/kanban/projects`, `/kanban/statuses`, `PUT /kanban/config`, `POST /kanban/session-env` | **fully built and proven — persists correctly.** Only `vscode-extension/src/kanban-onboarding.ts` calls it; `ui/src/api-client.ts` has no methods. Port that client. |
| Model servers | `GET /model-servers/kinds`, `GET /kinds/{slug}/models` (live probe), `POST /model-servers` | `ModelProvidersPage.tsx` already drives connect; extract its card grid |
| Provider catalog | `GET /api/v1/integrations` | serves §3.3 to the browser with no new route |
| Collections | `GET /collections`, `POST /collections/{name}/activate` | **activate is in-memory only** — see below |

The kanban quartet being done is the single biggest head start here. Read
`vscode-extension/src/kanban-onboarding.ts` as the working reference.

### New work

1. **Move `setup.rs` into the lib** (§4.1). Prerequisite for everything else.
2. **Detection in `cmd_api`** (§4.3). Independently shippable.
3. **Persist collection activation** — `src/rest/routes/collections.rs:102`
   mutates the in-memory registry only, so the wizard's collection choice
   evaporates on restart. A direct D3 violation. Wrap in `mutate_config`.
4. **Git REST surface** — nothing exists; zero `/api/v1/git*` routes. Mirror
   the kanban quartet exactly, including its convention that `validate` returns
   `{valid, error}` rather than a 4xx on a bad token:

   ```
   GET  /api/v1/git/providers     read     -> catalog entries + CLI/auth state
   POST /api/v1/git/validate      execute  -> {valid, username?, error?}
   PUT  /api/v1/git/config        admin    -> {provider, token_env}   NO secret
   POST /api/v1/git/session-env   admin    -> {provider, token} -> {shell_export_block}
   ```

5. **Setup surface**:

   ```
   GET  /api/v1/setup/status      read   -> {initialized, admin_configured, config_path, tickets_path}
   GET  /api/v1/setup/steps       read   -> the catalog: slug, name, description, help_text, order
   POST /api/v1/setup/initialize  admin  -> SetupOptions DTO, wrapped in mutate_config
   ```

   `initialized` **must** use the existing shared predicate
   `startup::workspace_initialized(&config)` — already extracted in Phase A and
   already used by `App::new`. If the two surfaces compute "is this set up?"
   differently, D3 is unenforceable.

6. **`ui/src/api-client.ts`** — kanban, git, setup, integrations methods.
7. **The wizard itself** — `ui/src/routes/onboarding/`, one component per slug,
   map declared `satisfies Record<SetupStep, StepComponent>` so `tsc` errors
   when a Rust variant lands without a web component. That is the TypeScript-side
   compile-time guard, replacing a drift test.

## 6. Deliberate scope cuts

- **Startup Tickets renders read-only in v1** ("you can create these later from
  the dashboard"), and the parity test allowlists it explicitly. Shipping
  half-working ticket creation and calling it parity is worse than an honest gap.
- **Per-wrapper steps** (tmux/vscode/cmux/zellij onboarding) are terminal-centric
  help screens. Decide deliberately whether the browser shows them, shows one
  combined "session target" screen, or skips them — and record the decision in
  the parity test's allowlist either way.

## 7. Acceptance — how to prove D3

The bar is not "tests pass". It is two clean rooms producing interchangeable
config.

1. `rm -rf /tmp/opr-tui && mkdir -p /tmp/opr-tui && cd /tmp/opr-tui && operator`
   — walk the wizard, choosing a non-default on every screen.
2. `rm -rf /tmp/opr-web && mkdir -p /tmp/opr-web && cd /tmp/opr-web && operator api --open`
   — bootstrap at `/#/setup`, complete `/#/onboarding`, choosing the same options.
3. **Diff the two `config.toml` files.** Differences must be explainable by the
   choices made, not by which surface wrote them. This is the D3 check.
4. Start `operator` in `/tmp/opr-web`: the TUI must read the web-produced config
   without re-running setup. Then the reverse.
5. In the web run, confirm the sidebar has **no disabled entries** afterwards
   (the §4.3 check).
6. `make check`, then `cargo run -- docs && make bindings && git diff --exit-code`.

## 8. Known-bad state you will inherit

Both pre-date this work. Neither is yours to fix, but both will confuse you.

- **`make check` is not reliably green.**
  `auth::schema::tests::test_concurrent_migration_of_one_database_is_safe`
  fails ~8% of the time **in isolation** (measured 2/25) with
  `enabling WAL: database is locked`. A hypothesis that `apply_pragmas` sets
  `busy_timeout` after the `journal_mode=WAL` switch that needs it was **tested
  and disproven** (2/25 before, 2/25 after) and reverted. There is no working
  explanation yet. Treat a red `make check` as "check which test failed".
- **Generated artifacts do not round-trip.** `cargo run -- docs` regenerates
  `docs/schemas/{config,metadata,state}.md` and `docs/schemas/openapi.json` with
  em-dashes where the committed copies have hyphens, while
  `src/schemas/issuetype_schema.json` regenerates with hyphens where the
  committed copy has em-dashes. None of those sources were touched in Phase A.
  Step 6 of §7 will surface this. It wants its own cleanup commit.

## 9. Working agreements

From `CLAUDE.md` and established over Phase A:

- **TDD.** Write the failing test first; confirm it fails for the right reason.
- **Verify guards are not vacuous.** Every parity test added in Phase A was
  confirmed by deliberately breaking it and watching it fail. Do the same.
- **`make check` before declaring done** — and read its *actual* exit code. A
  grep over its output that matches nothing is not evidence of success; that
  mistake hid a real failure during Phase A.
- **Do not commit.** The user handles all git operations manually.
- **Minimal comments**, terse and one line where they earn their place.
- **Ask rather than assume** on anything that changes the shape of the work.
  Phase A's kanban step was redesigned mid-flight because the planned approach
  served nobody on a first run; that was worth one question.

# Getting-Started Parity — Phase A: audit + TUI alignment

## Context

Operator can be driven from two surfaces — the ratatui TUI (`operator`) and the
web SPA (`operator api`) — and both are supposed to land a new user at the same
place: a working `config.toml` with kanban, model and git providers connected.

`docs/getting-started/index.md` is a five-line stub naming an order
(agent → kanban → git) that **neither surface implements**. The audit found the
mismatch is not editorial: the TUI wizard has an unreachable step, silently
discards three of the user's choices, and has two divergent workspace writers;
the web UI has no guided flow at all and locks its own configuration pages
behind a prerequisite only the TUI can satisfy.

**Neither surface currently satisfies "by the end of this flow you have a usable
config.toml", and the web flow is not independently completable.**

Sequencing is the user's: **audit and report first, then align UI and TUI, then
document as it exists.**

### Scope of this plan

The full arc runs ten stages. **This plan authorizes stages 0–5 only** — fix the
defects and land both new TUI steps (decision D1). The web wizard (D2), the REST
surface it needs, re-runnability and the documentation rewrite are deferred to a
follow-up plan, to be written once the TUI half is real and has been used.

Docs are deliberately *not* in this phase: the user's rule is "document as it
exists", and after stage 5 only half the alignment exists. Writing the
getting-started flows now would document a state that stage 8 changes.

Decisions already made by the user, carried forward:

- **D1 (this plan)** — add a Model Server step and a Git Provider step to the TUI wizard.
- **D2 (deferred)** — a web wizard mirroring the TUI, writing the same `config.toml`.
- **D3 (deferred)** — either flow, run alone, ends with a usable `config.toml`.
- **D4 (deferred)** — docs last; per-surface flows link out to the existing
  kanban/model/git pages rather than re-explaining them inline.

> One open recommendation, easy to reverse: stage 2 promotes the step catalog to a
> single Rust source of truth. The user did not rule on this; it is on the critical
> path to D1 regardless, and the rationale is in Phase 2 below.

> Per `CLAUDE.md`, once approved this plan should also be committed to
> `superpowers/plans/`.

---

## Phase 1 — Audit report (complete)

### Blocking defects

| | Defect | Evidence |
|---|---|---|
| **A** | Kanban provider step is unreachable dead code | `valid_kanban_providers` (`src/ui/setup/mod.rs:70`) initialized empty at `:165`, **never pushed to anywhere**; `confirm()` at `:566` always takes the `is_empty()` branch |
| **B** | Three screens collect choices that are never persisted | `src/app/tickets.rs` has **zero** matches for `wrapper`/`worktree`/`acceptance`; `sessions.wrapper` and `git.use_worktrees` keep their defaults regardless of what the user picked |
| **C** | Two divergent workspace writers | `App::initialize_tickets` (`src/app/tickets.rs:19-207`) vs `setup::initialize_workspace` (`src/setup.rs:56-150`) — different dirs, different files, different config keys |
| **D** | Wizard can't be re-run | `setup_screen = Some(..)` only in `App::new`; `operator setup --interactive` is a stub (`src/main.rs:1146-1151`); trigger is the *queue dir*, not config (`src/app/mod.rs:157`) |
| **E** | Generated wizard doc is already wrong | `SETUP_STEPS` (`src/startup/mod.rs`) lists the pre-reorder order and mentions a git step that never existed; only guard is `assert_eq!(len(), 16)` at `:244` — it passed while the data was wrong |
| **F** | Web UI locks its own pages behind a TUI-only prerequisite | `refresh_tool_detection` runs only at `src/app/mod.rs:121`; `operator api` never detects ⇒ `llm` Yellow ⇒ **Model Providers + Delegators disabled in the sidebar** on real working routes |
| **G** | Git unreachable from the browser | Zero `/api/v1/git*` routes; `ConfigureGitProvider` has no `web_url()`, so `#/git` rows render as inert text |
| **H** | Kanban plumbing exists but has no SPA client | `PUT /api/v1/kanban/config` + validate/projects/statuses/session-env all persist correctly; only `vscode-extension/` calls them |

Defects **F, G, H** are web-side and are addressed in the deferred phase.

### Two further defects found during design (both verified)

- **I** — `SetupOptions::kanban_provider` and `::llm_tool` (`src/setup.rs:30,32`) are
  validated and printed by `cmd_setup` but **never read** by `initialize_workspace`.
  `operator setup --kanban-provider jira --llm-tool claude` is a no-op.
- **J** — `Config::operator_config_path()` (`src/config.rs:691`) is a hardcoded
  **relative** `.tickets/operator/config.toml`, and `Config::save()` uses it. The
  tempdir tests at `src/setup.rs:351-499` therefore write into the **repo root** —
  confirmed present on disk (2884 bytes, gitignored), which is why it went
  unnoticed. This blocks writing parallel-safe tests for any new init path.

### Corrections to assumptions

- **Forgejo is not covered by git onboarding.** `onboarding_spec_for_slug`
  (`src/api/cli_detection.rs:138`) returns `Some` only when `pat_url` is non-empty;
  verified only `github`, `gitlab`, `gitea` have one.
- **Auth needs no work.** Browser session principals carry `Scope::ALL`
  (`src/auth/store.rs:462`).

### Lesser findings (for the deferred phase)

- `POST /api/v1/collections/{name}/activate` is in-memory only — lost on restart.
- `PATCH /api/v1/configuration`, `PUT /api/v1/llm-tools/default` and the
  model-server `PUT`/`DELETE` persist but no SPA page calls them.
- `#/settings/security` and `#/status` have no nav entry.
- Wizard footers advertise unbound `[S]`/`[R]`/`[T]`/`[n/p]`; `i` and `c` are live
  on every screen (force-initialize / quit).
- Secrets are consistently stored **by reference** (env var *name*) across kanban,
  git and model servers. Correct — preserve it. It is also what users trip on,
  because a token exported after the process started is invisible to it.

---

## Phase 2 — TUI alignment (stages 0–5)

### Stage 0 — Defect J: make the config path derive from state

Add `Config::operator_config_path_for(&self) -> PathBuf` deriving from
`paths.state`; keep the associated `operator_config_path()` for `Config::load()`'s
bootstrap (it must resolve a path before a `Config` exists). Point `Config::save()`
(`src/config.rs:769`) at the `&self` variant.

For a normally-invoked process (cwd == workspace root) the resolved path is
identical, so behaviour is unchanged in practice — but it is a real semantic change
affecting every `config.save()` call site, so it gets **its own commit and its own
test**, ahead of everything else. It also stops `cargo test` scribbling in the repo
root, which is what makes stages 1–5 testable.

### Stage 1 — Defects C, B, I: one workspace writer

`initialize_workspace` is already the better implementation: it honours `force` via
`write_file_if_allowed`, writes `operator/templates/` plus the three interpolation
docs, and persists `git.use_worktrees`. `App::initialize_tickets` does none of that
and unconditionally `fs::write`s.

Extend `SetupOptions` with the dropped choices, and make it **not save** so the
caller owns persistence:

```rust
pub struct SetupOptions {
    // existing: preset, force, task_fields, working_dir,
    //           kanban_provider, llm_tool, use_worktrees
    pub wrapper: Option<SessionWrapperType>,
    pub acceptance_criteria: Option<String>,
    pub custom_collection: Vec<String>,
    pub active_collection: Option<String>,
    pub hosted_collections: Vec<FetchedCollection>,
}

/// Creates directories, writes templates, mutates `config`. Does NOT persist —
/// the caller owns the save.
pub fn initialize_workspace(config: &mut Config, options: &SetupOptions) -> Result<SetupResult>;
```

Dropping the internal save is what later lets a REST handler wrap the whole thing in
`ApiState::mutate_config` (which saves atomically **and** `replace_config`s, so a
running server isn't left on a stale snapshot — `src/rest/state.rs:156-167`). Doing
it now means the deferred phase needs no second refactor here.

`App::initialize_tickets` shrinks to: build `SetupOptions` from `SetupScreen` → call
`initialize_workspace` → TUI-only tail (admin account, `generate_tmux_config`,
registry reload, startup tickets) → one `config.save()`. `cmd_setup` adds its own
`config.save()?`.

**B and I fall out of this**: once `SetupOptions` carries them, `sessions.wrapper`,
`git.use_worktrees`, the acceptance text, `kanban_provider` and `llm_tool` all land.

### Stage 2 — Defect E: promote the step catalog

Defect E's guard already existed and still passed while the data was wrong — the
count never changed, only the order did. Sharing the **step catalog** (identity,
order, copy) replaces that with a compile-time guard. This is not the renderer
abstraction the user rejected; renderers stay hand-written.

1. New `src/startup/steps.rs`; move `SetupStep` there from
   `src/ui/setup/types.rs:329`. **Mandatory, not stylistic**: `src/rest` compiles in
   the lib and must not reference bin-only `src/ui`, so the deferred REST surface
   cannot see the enum where it lives today. Doing the move now avoids reopening
   these files later.
2. Drop the `KanbanProviderSetup { provider_index }` payload — move the index onto
   `SetupScreen` as a plain cursor field like every other step's. A data-free enum
   makes `const ALL` and the derives trivial.
3. Replace the parallel `SETUP_STEPS` static with
   `impl SetupStep { fn info(&self) -> SetupStepInfo }` — one exhaustive `match`, so
   a new variant is a **compile error** until copy exists. `SETUP_STEPS` survives as
   `ORDER.iter().map(SetupStep::info)`, leaving `src/docs_gen/startup.rs:39` untouched.
4. Add `ALL`, `ORDER`, `slug()`. One test: `ORDER` is a permutation of `ALL`.
5. `#[derive(TS, Serialize, Deserialize, ToSchema)] #[ts(export)]` →
   `bindings/SetupStep.ts` via `make bindings`. Unused by the SPA until the deferred
   phase, but generated from the start so the binding never has to be backfilled.

**Defect E closes here**: regenerating `docs/startup/index.md` emits the real order.

### Stage 3 — Defect A: bring the kanban step to life

Deleting `KanbanProviderSetup` would leave the TUI unable to configure kanban while
a future web wizard trivially can — *creating* the asymmetry this work exists to
remove. So wire it up instead:

- Populate `valid_kanban_providers` in the `Welcome` arm from
  `p.has_required_env_vars()` — the same predicate `CollectionSourceOption::with_providers`
  already uses (`src/ui/setup/types.rs:79`).
- Make the step persist via
  `services::kanban_onboarding::{apply_config_request, set_session_env}` — **the same
  two functions `PUT /api/v1/kanban/config` and `POST /api/v1/kanban/session-env`
  call** (`src/rest/routes/kanban_onboarding.rs:104,136`). No new provider logic;
  both surfaces converge on one service.
- Bind the advertised `[S]` skip key, or remove it from the footer.

Also extract `startup::workspace_initialized(&config)` from `src/app/mod.rs:157` as
the single "is this set up?" predicate — cheap now, and the deferred phase depends
on both surfaces agreeing on it.

~80 lines.

### Target step order

```
Welcome → KanbanInfo → KanbanProviderSetup → ModelServer → GitProvider
        → CollectionSource (→ HostedCollectionFetch) → TaskFieldConfig
        → SessionWrapperChoice → WorktreePreference → AdminPassword
        → {Tmux|VSCode|Cmux|Zellij} → AcceptanceCriteria → StartupTickets → Confirm
```

Kanban, model servers and git are the three external-service connections and are
siblings in the sidebar prereq chain, so they group consecutively. Kanban must stay
before `CollectionSource` (that step offers "import from a configured provider" —
`src/ui/setup/mod.rs:553`). The session wrapper is a local concern and reads better
next to its wrapper-specific follow-up. Changing this later is one edit to `ORDER`.

### Stage 4 — TUI Model Server step (D1a)

New `src/ui/setup/steps/model_server.rs`. Lists `ModelServerKind::ALL` grouped by
`provider_class()`, with a live status column from
`probe_models(&server, &EgressPolicy::from_config(&config))` against a transient
server built from kind defaults — byte-for-byte what
`routes::model_servers::kind_models` does (`src/rest/routes/model_servers.rs:353-390`).

`probe_models` is async and `confirm()` is sync, so follow the existing precedent:
`src/app/keyboard.rs:57-72` awaits `load_hosted_collections` right after `confirm()`
returns `Continue` for the hosted step. Add `SetupScreen::probe_model_servers` and an
identical guard clause.

Three sub-states, all preserving secret-by-reference:

- env var present → Enter appends a `ModelServer` to `config.model_servers`;
- absent → an editable env-var-**name** field prefilled with `default_api_key_env()`,
  plus a copy-paste shell export block (precedent:
  `services::kanban_onboarding::build_shell_export_block_*`), and optionally a masked
  paste that only `set_var`s for the running process;
- `openai-compat` / `lmstudio` (`connectable_from_defaults() == false`) require a
  `base_url` before the row can be selected.

No secret reaches disk. Fully skippable.

### Stage 5 — TUI Git Provider step (D1b) — completes D1

New `src/ui/setup/steps/git.rs`, covering **github, gitlab, gitea** out of the box.
State per row comes from `git_onboarding::resolve_onboarding_with_config`
(`src/app/git_onboarding.rs:196`), reused verbatim: `InstallCli` / `AutoConfigured`
(Enter connects with zero typing) / `CollectToken` (opens the PAT page, shows the
existing `GitTokenDialog`).

**Forgejo needs one extra line** — a `pat_url` on its `CliSpec`
(`src/api/cli_detection.rs:95-105`) plus `"forgejo"` arms in
`complete_git_onboarding` and `validate_token_with_config`. Forgejo speaks Gitea's
API and `tea` drives both, so the gitea path works verbatim with a different base
URL. Recommend including it (~15 lines); it closes the "config structs but no TUI
row" gap. Bitbucket and Azure DevOps have no PAT flow — leave them out.

Two refactors this step requires:

1. **`complete_git_onboarding` saves mid-flow** (`git_onboarding.rs:159`). Inside the
   wizard that writes `config.toml` *before* Confirm, so a later Cancel leaves a
   partial config. Split into `apply_git_provider` (mutate + `set_var`, no save) and
   keep `complete_git_onboarding` as `apply + save` for the existing dashboard
   callers (`src/app/status_actions.rs:166`, `src/app/keyboard.rs:333`). Same
   one-writer discipline as stage 1.
2. **Key routing — verified hazard.** `src/app/keyboard.rs:21` has an unconditional
   early `return Ok(())` for the setup screen at `:102`, and the git-token dialog
   branch sits at `:326` — **below it**. The dialog would never receive keys while
   the wizard is open, and typing a PAT would quit the app on the first `c`. Hoist
   the dialog branch above the setup branch, or add a delegation guard at the top of
   it. Precedent: the `AdminPassword` text-routing guard at `:26-39` exists for
   exactly this reason.

Add `git_onboarding::shell_export_block(provider, token_env)` and render it — kanban
has this block, git does not, and without it the token dies with the process. Make
`token_env` editable for users on `GH_TOKEN` or a per-host var.

> `validate_*_token` uses `reqwest::blocking` inside an async fn. Fine in a TUI (it
> briefly stalls the event loop) — **but not fine in an axum handler**, which is a
> constraint the deferred REST work must respect via `spawn_blocking`.

---

## Staging

`make check` must pass at every boundary. All five stages are independently shippable.

| # | Stage | Depends on | Regenerate |
|---|---|---|---|
| 0 | Defect J — `operator_config_path_for(&self)` | — | — |
| 1 | One writer — C + B + I | 0 | `docs/configuration/index.md` if doc-comments change |
| 2 | Step catalog — E | — | `make bindings`; `cargo run -- docs` → **defect E closed** |
| 3 | Kanban step alive — A; `workspace_initialized` | 2 | — |
| 4 | TUI Model Server step (D1a) | 2 | `cargo run -- docs` |
| 5 | TUI Git Provider step (D1b) | 2 | `cargo run -- docs`; watch `tests/vertical_parity.rs` for Forgejo |

Critical path: 0 → 1 → 2 → 5. Stages 3 and 4 are off it and can land in parallel.

---

## Testing

Per repo TDD convention — write the failing test first.

**In-file `#[cfg(test)] mod tests`:**

- `src/config.rs` — `test_save_writes_under_paths_state_not_cwd` (stage 0).
- `src/setup.rs` (tempdir tests at `:351-499` are the model) — wrapper/worktree
  persistence (B), acceptance criteria, kanban/llm options (I),
  `test_initialize_workspace_does_not_save` (the stage-1 contract change),
  `test_initialize_workspace_is_idempotent_without_force`. **Only safe after stage 0.**
- `src/startup/steps.rs` — replaces the hardcoded `len() == 16`:
  `test_order_is_a_permutation_of_all`, `test_every_step_info_field_is_non_empty`,
  `test_slugs_are_unique`, `test_slugs_match_frozen_snapshot` (slugs will key docs
  URLs and, later, the SPA component map).
- `src/ui/setup/tests.rs` (664 lines;
  `test_admin_password_step_follows_worktree_preference:465` is the model) — step-order
  tests for the two new steps, `test_model_server_selection_appends_config_entry`,
  `test_model_server_non_connectable_kind_requires_base_url`,
  `test_git_selection_sets_provider_without_saving`, and the **defect A regression
  test**: `test_wizard_walk_visits_every_catalog_step`, driving `confirm()` from
  Welcome to Confirm across all four wrapper branches and asserting the union equals
  `SetupStep::ALL` minus a documented allowlist. This is the test that would have
  caught A on day one.
- `src/app/git_onboarding.rs` — Forgejo arm, `apply_git_provider` does not save.
- `src/app/keyboard.rs` — a dialog-over-wizard routing test, so the PAT field can
  never again be eaten by the wizard's `c`/`i` bindings.

**`tests/`:**

- New `tests/setup_parity.rs`, using the `include_str!` source-scanning pattern from
  `tests/surface_parity.rs`: every variant has non-empty info and appears in `ORDER`;
  `bindings/SetupStep.ts` union equals the Rust slugs; **`docs/startup/index.md`
  lists headings in `ORDER`** — the direct defect-E regression test.
- `tests/vertical_parity.rs` is an existing *constraint*, not new work: if the git
  step advertises Forgejo, check its `SupportStatus` first — `Alpha`+ entries owe a
  resolving docs page and a README badge.

## End-to-end verification

1. `make check` at every stage boundary.
2. **Clean room**: `rm -rf /tmp/opr-tui && mkdir -p /tmp/opr-tui && cd /tmp/opr-tui && operator`.
   Walk the wizard end to end, choosing a non-default on every screen.
3. Assert on `/tmp/opr-tui/.tickets/operator/config.toml` that `sessions.wrapper`,
   `git.use_worktrees`, `git.provider`, `git.<p>.enabled`, `[kanban.*]` and
   `[[model_servers]]` all reflect what was chosen — the direct B/A/D1 check.
4. Confirm no config.toml appeared in the repo root after `cargo test` (defect J).
5. Re-run the wizard over the existing workspace (delete `.tickets/queue` for now;
   proper re-entry is deferred) and confirm templates are **not** clobbered.
6. `cargo run -- docs && make bindings`, then `git diff --exit-code` to prove
   generated artifacts are committed fresh — and eyeball that
   `docs/startup/index.md` now lists the real order.

---

## Deferred to the follow-up plan (stages 6–10)

Recorded so the design is not lost. Do not build these now.

- **6 — PREREQUISITE FOUND DURING STAGE 1.** `src/setup.rs` is **bin-only**
  (`mod setup;` in `src/main.rs:39`; it is absent from `src/lib.rs`). `src/rest/`
  compiles in the lib, so `POST /setup/initialize` **cannot** call
  `setup::initialize_workspace` where it lives today. Stage 6c must first move
  `src/setup.rs` into the lib (`pub mod setup;`), which also pulls in its deps —
  `projects` and `startup` are already lib-private modules, so they only need
  visibility widening, not relocation. Same class of constraint as the
  `SetupStep` move in stage 2. Budget this before estimating 6c.
- **6 — Server enablers.** `startup::refresh_and_persist_detection` called from
  `cmd_api` (fixes defect F: `llm` goes Green for web users, unlocking Model
  Providers + Delegators); persist `collections/{name}/activate` through
  `mutate_config`; `GET /setup/{status,steps}` + `POST /setup/initialize`;
  `/api/v1/git/*` mirroring the kanban quartet (fixes defect G) — **every shelling
  or `reqwest::blocking` call wrapped in `spawn_blocking`**; `ROUTE_RULES` entries
  (`tests/route_scope_parity.rs` fails closed on any route missing from the table).
- **7 — `ui/src/api-client.ts`** kanban/git/setup methods; port from
  `vscode-extension/src/kanban-onboarding.ts`, the working reference (fixes defect H).
- **8 — Web wizard (D2/D3).** `/#/onboarding` mounted as a **sibling of `setup`, not
  inside `Layout`** (`ui/src/main.tsx:35-36`) — `Layout.tsx:22` disables nav items
  whose `section.met` is false, so a wizard inside it would be surrounded by the
  gating it exists to resolve. Component map `satisfies Record<SetupStep, …>` so
  `tsc` errors on a missing step. Plain React + CSS modules —
  `tests/ui_packaging.rs` enforces a nine-entry dep allowlist and bans CSS-in-JS.
  Render Startup Tickets read-only in v1 rather than shipping half-working ticket
  creation and calling it parity.
- **9 — Re-runnability (defect D).** `SetupScreen::from_config` prefilling from live
  config (`WorktreeOption::from_use_worktrees` already exists marked
  `#[allow(dead_code)] // Useful for future config-to-UI state conversion`,
  `src/ui/setup/types.rs:316` — this is that future); a TUI key; a real
  `operator setup --interactive`; a web re-run entry. Re-run must be additive and
  idempotent: `force = false`, kanban upserts, model servers PUT-or-POST (`POST`
  409s on duplicates, `src/rest/routes/model_servers.rs:143`), startup tickets
  default to none.
- **10 — Docs (D4).** `docs/getting-started/index.md` stays slim: an **"expected
  setup"** section naming the connections a working install needs, linking out to
  `/getting-started/{kanban,model-servers,git,sessions}/` with **no inline provider
  instructions**, then a chooser into two new **level-2** pages (one per surface),
  both linking to `/startup/` rather than restating generated steps.
  Constraints: nav entry in `docs/_data/navigation.yml` with a byte-matching title;
  front matter exactly `title`/`description`/`layout: doc`; body starts at `##`;
  absolute trailing-slash links; no callout/tab includes exist.
  Verify with `cargo test --test docs_structure`.

## Open items to confirm during implementation

1. Forgejo's current `SupportStatus` in `src/integrations/catalog.rs` — determines
   whether stage 5 also owes a docs page and README badge to satisfy
   `tests/vertical_parity.rs`.
2. Stage 0's blast radius across all `config.save()` call sites. The resolved path
   should be identical for normally-invoked processes, but this deserves its own
   commit and review rather than being bundled.
3. Whether dropping the `KanbanProviderSetup` payload (stage 2) disturbs any
   `go_back` logic in `src/ui/setup/mod.rs:720-825` beyond the mechanical change.

---

## Implementation log (Phase A)

Deviations from the plan as written, with reasons.

- **Stage 3 reshaped.** The planned in-wizard per-provider step was gated on
  `has_required_env_vars()`, i.e. it renders nothing unless the user has already
  exported an API key — useless for the first-run audience it exists for, which
  is very likely why `valid_kanban_providers` was never populated. On the user's
  call the step now **delegates to the existing `K` onboarding dialog** (cold
  onboarding: provider → credentials → live validation → project → config +
  shell export). `SetupStep::KanbanProviderSetup` was removed from the catalog
  (15 steps now), and `KanbanInfo` gained a connect/skip choice.
- **Keyboard routing fixed early.** The modal git-token and kanban dialogs were
  below the setup screen's unconditional `return Ok(())`, so they could never
  receive keys over the wizard. Hoisted above it in stage 3 rather than stage 5,
  since stage 3 needed it first; stage 5 now inherits the fix.
- **Latent clobber fixed.** `services::kanban_onboarding::write_config` persists
  straight to disk but `self.config` was never refreshed, so any later
  `config.save()` wrote the kanban section away again. Pre-existing on the
  dashboard path; certain once the wizard delegates (Confirm always saves).
  `reload_config_after_kanban_write` now picks the result back up.
- **`SetupStep::ORDER` dropped.** `ALL` is declared in wizard order, so a
  separate `ORDER` would have been the same array and its permutation test
  vacuous. One ordered const instead.
- **Dead code removed along the way:** `App::generate_tmux_config`,
  `Config::discover_projects_full`, `SetupScreen::{kanban_projects,
  kanban_issue_types, kanban_member_count, kanban_skipped,
  valid_kanban_providers}`, and `render_kanban_provider_setup_step`.
- **Defect J had a wider blast radius than expected** — three call sites
  *reported* or *opened* the config path (`status_panel.rs`,
  `services/kanban_onboarding.rs`, `rest/routes/kanban_onboarding.rs`) and would
  have named a file `save()` no longer writes. All repointed.

### Stages 4-5

- **Forgejo excluded, narrower than the plan recommended.** The plan proposed
  adding a `pat_url` to bring Forgejo into the git step. Checking
  `src/integrations/catalog.rs` settled the open item: Forgejo is **Proto**,
  undocumented, with no `docs_path` - as are Bitbucket and Azure DevOps. Putting
  a Proto integration in the primary onboarding flow would promote it ahead of
  its support status and owe it docs under `tests/vertical_parity.rs`. The step
  offers `GIT_PROVIDER_SLUGS = ["github", "gitlab", "gitea"]`, which is exactly
  the Alpha-or-better set and exactly the set with a PAT flow.
- **Non-connectable model kinds are shown but not declarable.** Rather than an
  inline `base_url` text field, `openai-compat` and `lmstudio` render with a
  "needs base URL" hint and refuse toggling - the same restriction the web UI
  applies (no Connect button), which keeps the two surfaces honest for D3.
- **The git token dialog defers its save inside the wizard.** `apply_git_provider`
  when `setup_screen.is_some()`, `complete_git_onboarding` otherwise, so Confirm
  stays the single persistence point and cancelling strands nothing.
- **Test churn is inherent to inserting steps.** Each new step invalidated the
  ordering assertions of the step before it; those were updated, not deleted.
  The wizard-walk test takes each new step's skip row so it does not shell out
  to provider CLIs.

### Verification

`make check` passes (fmt + clippy `--locked --all-targets --all-features -D
warnings` + full test suite). Both parity tests were confirmed non-vacuous by
deliberately breaking them and watching them fail.

One unrelated flake observed: `auth::schema::tests::test_concurrent_migration_of_one_database_is_safe`
failed once under full-suite load and passed 3/3 in isolation and on every
re-run. It is an 8-thread SQLite contention test in its own tempdir, touching
nothing in this change.

### Pre-existing generated-artifact drift (not from this work)

`cargo run -- docs` + `make bindings` do **not** reproduce the committed
artifacts on this branch, in both directions:

- `docs/schemas/{config,metadata,state}.md` and `docs/schemas/openapi.json`
  regenerate with em-dashes where the committed copies have hyphens;
- `src/schemas/issuetype_schema.json` regenerates with hyphens where the
  committed copy has em-dashes.

None of these files' sources were touched here. This matters for the deferred
stage 10: a docs task that runs the generators will surface this drift, and any
CI check doing `cargo run -- docs && git diff --exit-code` is not currently
clean. Worth a separate cleanup commit.

### Catalog-derived provider lists (added after stage 5, before stage 8)

The stage 4/5 steps keyed off per-vertical enums, and the git step's list was a
hand-written `GIT_PROVIDER_SLUGS` const - the catalog was consulted to *decide*
it, then the answer was hardcoded. Same drift class as defect E.

Both steps now derive from `integrations::catalog::onboardable(vertical)`
(`Alpha`+ and documented). `GIT_PROVIDER_SLUGS` is deleted; rows, labels and
docs links come from `CatalogEntry`, and the slug→CLI mapping comes from
`cli_detection::onboarding_spec_for_slug`, which already owned it.

Why the catalog rather than the enums: `integrations` is `pub mod` in `lib.rs`,
unlike `startup` and `setup`, so the stage-8 web wizard can read the same list
over the existing `/api/v1/integrations` without a module move. One list, three
surfaces; promoting a provider into onboarding is a `SupportStatus` bump.

Consequences:
- Model Server drops from 7 rows to 5 (`openai-compat`, `lmstudio` are Proto).
- `toggle_model_server`'s `connectable_from_defaults()` guard is now unreachable
  via the offered list. Kept (still correct if a future Alpha provider ships
  without a default base URL), but the test that exercised it was replaced with
  one asserting the real invariant rather than left to pass vacuously.
- Two hand-rules had coincidentally agreed: git used support status, model used
  base-URL availability. Same answer today, divergent the moment a Beta provider
  ships without a default base URL.

Three guards, the third verified by reintroducing a hardcoded `"github"`:
`onboardable` never yields Proto/undocumented; the step files contain no
provider slug literals; `setup/mod.rs` must reference `onboardable(Vertical::*)`.

### Open: pre-existing auth test flake (NOT from this work)

`auth::schema::tests::test_concurrent_migration_of_one_database_is_safe` fails
~8% of the time **in isolation** (measured 2/25), with
`enabling WAL: database is locked`. It intermittently fails `make check`.

`src/auth/schema.rs` is not in this change set. A hypothesis that
`apply_pragmas` sets `busy_timeout` *after* the `journal_mode = WAL` switch that
needs it was tested and **disproven** - reordering measured 2/25 before and
2/25 after, so the change was reverted. There is currently no working
explanation; SQLite's busy handler appears not to engage for this particular
conflict, but that was not established. Needs its own investigation.

Treat a red `make check` on this branch as "check which test failed" until this
is resolved.

### Final verification (stages 0-5 + catalog rewiring)

lib 2206 passed / 0 failed; bin 2806 passed / 0 failed; 29 integration targets
all ok; `cargo fmt --check` clean; `clippy --locked --all-targets
--all-features -D warnings` clean.

Not done: nobody has walked the two new screens in a terminal. They are covered
by unit tests and the reachability walk, which is not the same thing.

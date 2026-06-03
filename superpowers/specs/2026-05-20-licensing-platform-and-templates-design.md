# Operator Licensing Platform — Template Bootstrap Plan

## Context

Operator is a Rust TUI that the author (Sam / `untra`) wants to license and sell. Beyond Operator itself, the author intends to build other billable software products under the `untra` umbrella. The licensing/billing infrastructure must therefore be **reusable across future untra products**, not bespoke to Operator.

Two concerns were brainstormed in this session:

- **X. Operator's licensing & billing system** — sub-projects A (entitlement model) through F (admin tool). Roadmapped here; detailed designs deferred to follow-up sessions.
- **Y. Untra platform template & deployment skeleton** — the reusable cloud + DNS + auth + storefront skeleton that every untra product plugs into. **This is the focus of this session.** Y was promoted from a child of X to a top-level peer once the templating goal was made explicit.

Goal of this session: produce a plan for creating six template repositories at `../templates/` — one per archetype. Each template gets a `README.md` and a `HANDOFF.md` and nothing else. A top-level `../templates/README.md` indexes the six archetypes for anyone landing in the directory. The templates are then fleshed out in independent follow-up Claude sessions, one per template, using the handoff briefs.

Cost-deferral is a hard requirement: **no paid commitments should be necessary to complete Phase 0–1**. The first paid commitment is the root-domain registration (~$12/yr), happening at Phase 2.

---

## Reading of the request

This plan rests on one interpretation of the user's template list (`iac api app auth admin license`). Surfacing it loudly so it can be corrected at review time:

- **`api`** — generic product-side backend. Cloud Run service in the public per-project monorepo. Exposes (1) an **unauthenticated** `/version` endpoint and (2) **authenticated** endpoints (verified against the entitlement JWT signed by `license`) that report user details and serve product business logic. Receives the **LemonSqueezy purchase webhook** and, on a successful sale, calls `license` over a signed internal request to mint a license record.
- **`license`** — privileged entitlement microservice. Lives in the **private** sister repo (`operator-private` for Operator; `<product>-private` for other untra products). Holds the Ed25519 signing key in Secret Manager. Only `auth` and `api` may call it. Verifies a posted license key, returns/issues a signed entitlement JWT (booleans + integer limits, ~14-day TTL). Maintains the revocation list.
- **`auth`** — identity orchestrator at `auth.<domain>`. Wraps Firebase Auth (sign-in UI). After the user signs in, `auth` accepts a license key, calls `license` to verify it, and packages the resulting entitlements into a JWT (signed by `license`) returned to the client. Acts as the trust bridge between Firebase identity and license entitlements.

If this reading is wrong, sections below collapse. Push back at spec review.

---

## Roadmap

### X. Operator licensing sub-projects (decomposition only)

| # | Component | Lives in | Status |
|---|---|---|---|
| A | Entitlement model & license-key format | shared schema crate / proto | **Deferred** — detailed spec in a follow-up session. Eight decisions already locked (see "Entitlement model — locked decisions" below). |
| B | License verification service | `operator-private`, generated from `templates/license` | Built via Y. Detailed implementation in a follow-up session. |
| C | Operator client integration | this repo (`operator/`) | Future ticket. Reads entitlement JWT, exposes flags via OpenFeature provider. |
| D | Storefront + billing | LemonSqueezy hosted; no template. Adapters in `templates/app` + `templates/api`. | Built via Y. |
| E | Customer account site | `templates/app` instance | Built via Y. |
| F | Admin tool | `templates/admin` instance | Built via Y. |

### Y. Untra platform template (this session)

Six archetype template repos at `../templates/`:

```
templates/
├── iac/        # Terraform/OpenTofu modules: Cloudflare, GCP, Firebase, Neon, IAP, Secret Manager
├── api/        # Generic product backend (Rust + Axum + Cloud Run)
├── app/        # Customer-facing web app (TypeScript SPA, sign-in via Firebase, license mgmt UI)
├── auth/       # Identity orchestrator (auth.<domain>): Firebase Auth UI + license-exchange endpoint
├── admin/      # Admin console (IAP-gated): plan editor, license mgmt, revocation
└── license/    # Privileged entitlement service (vendors into *-private repos only)
```

### Entitlement model — locked decisions (referenced by `license`)

These were agreed earlier in the session. They become the "Entitlement Model — frozen" section in `license/README.md`. Detailed token schema/crypto is part of the deferred A-spec.

1. **Vintage model:** Adobe-style year-versioned major releases (`standard-2025`, `enterprise-2026`). Each vintage is a distinct SKU.
2. **Access duration:** Perpetual one-time buy per vintage. No subscription expiry.
3. **Feature types:** Booleans + integer limits (e.g. `acp_enabled=true`, `max_projects=20`).
4. **Verification model:** Hybrid — short opaque license key + server-issued signed entitlement JWT cached for ~14 days.
5. **License scope:** Single user, soft cap of 3 machines per license.
6. **Account model:** One account owns many licenses over time.
7. **OpenFeature shape:** Custom OpenFeature provider in client (Rust SDK) reads flags from cached entitlement JWT.
8. **Revocation:** Revocation list + TTL-driven propagation. Already-cached tokens expire within ~14 days of revocation.

---

## Service trust model

```
                Firebase ID token
   user ──signin──▶ auth.<domain> ──┐
                                    │  (verify license key + Firebase identity)
                                    ▼
                            license.<private-domain>
                            (Ed25519 sign entitlement JWT)
                                    │
   client ◀───── entitlement JWT ───┘
        │
        ▼ Bearer JWT
   api.<domain>            ◀── verifies JWT against license public key (embedded)
        │
        └── LemonSqueezy webhook ──▶ signed internal call ──▶ license  (mint license)
```

**Trust boundary:** `license`'s signing key never leaves the private GCP project. `auth` and `api` only hold `license`'s public key. Public-monorepo CI cannot touch `license`'s secrets.

---

## Architectural decisions (named, not buried)

### D1. Two-repo structure per product (public + private)

Every untra product produces **two** GitHub repos at provisioning time:

- `<product>/` — public monorepo, vendors `iac` + `api` + `app` + `auth` + `admin`.
- `<product>-private/` — private monorepo, vendors `license` and a separate slice of `iac` (the private-side state, signing-key Secret Manager, separate Neon project).

The two repos correspond to two separate GCP projects with no cross-project IAM. The only runtime coupling is the signed internal HTTPS call from `api` (public) to `license` (private).

### D2. LemonSqueezy webhook lands in `api`, not `license`

Webhooks have public, unauthenticated ingress by design. `api` already has a public surface and a Neon DB for user records — it's the right place to receive them. `api` then makes a **signed internal request** (HMAC over webhook payload + nonce) to `license` to mint the actual license. `license` never receives unauthenticated external traffic; the only external endpoints on `license` are token-signing endpoints called by `auth`.

### D3. The first untra product (Operator) eats its own dog food

Operator is both:
- a **consumer** of the platform (it has license keys, calls `auth` to refresh entitlements, gates features via OpenFeature)
- the **bootstrap operator** for future products (per its CLAUDE.md, "self-starting work multiplexor")

Therefore Operator's licensing integration (sub-project C) and the platform templates (Y) must be designed so Operator can later orchestrate Phase 2 deployments for *new* untra products. This session does not implement that orchestration; it only avoids painting it into a corner.

---

## Cost-deferral phases

- **Phase 0** *(this session, $0)*: six templates exist at `../templates/<archetype>/`, each `git init`'d, each containing only `README.md` and `HANDOFF.md`.
- **Phase 1** *(follow-up Claude sessions, $0)*: each template gets fleshed out by a fresh Claude session using its HANDOFF.md. Output: working code, Dockerfiles, IaC modules. Still local; no cloud accounts needed.
- **Phase 2** *(first deployment, ~$12/yr)*: register Operator's root domain. Set up Cloudflare zone (free), GCP project (uses $300 trial credit), Firebase Auth (free tier), Neon Postgres (free tier), Google Secret Manager (free tier). Deploy all services to Cloud Run with `min_instances=0`. Cost ceiling: ~$12/yr for the year, possibly $0 within trial credit.
- **Phase 3** *(first sale)*: LemonSqueezy onboarding (~30 min, no business entity required). First transaction triggers the first revenue and the first 5%+50¢ fee. No upfront commitment.

---

## What this session WILL produce (post-ExitPlanMode)

**Top-level index file:**
1. Write `../templates/README.md` — a one-page index explaining the six archetype repos, the platform stack, and how they fit together. Section outline below.

**Per archetype** (`iac`, `api`, `app`, `auth`, `admin`, `license`):
1. Create directory `../templates/<archetype>/`.
2. Run `git init` inside it.
3. Write `README.md` per the outline below.
4. Write `HANDOFF.md` per the outline below.
5. **Do not commit.** Per user instruction (memory: `feedback_no_commits.md`), the user handles all commits.

**Promotion step:**
1. After the templates exist, copy this plan file to `operator/docs/superpowers/specs/2026-05-20-licensing-platform-and-templates-design.md` so it survives outside `~/.claude/plans/`. (User confirmed promotion at plan approval.)

That is the entirety of the action. **No source files, no Dockerfiles, no Terraform.** Those are Phase 1, in separate sessions.

---

## Top-level `../templates/README.md` outline

A short index file at the root of the templates directory. Anyone who `cd`s into `../templates/` should understand the platform in under a minute. Sections in order:

```
# untra platform templates

## What this directory is
One paragraph: these are archetype templates for untra's billable-SaaS platform.
Each subdirectory is its own git repo; they get vendored into per-product
monorepos at Phase 2.

## The six archetypes
A table: archetype name | one-line role | vendors into (public/private).

## Platform stack (defaults)
The cost-deferral stack table from the master spec, abbreviated:
domain (Cloudflare), compute (Cloud Run min=0), identity (Firebase Auth),
data (Neon Postgres), storefront (LemonSqueezy MoR), CI (GitHub Actions),
IaC (OpenTofu). Cost-at-zero-traffic ceiling: ~$12/yr (domain only).

## Trust model
The auth → license → JWT → api diagram, in ASCII.

## How a new product is provisioned
Cross-reference templates/iac/README.md "Quickstart" section for the manual
checklist.

## Status
"Phase 0: READMEs and handoff briefs only. See each subdirectory's HANDOFF.md
to start implementation in a fresh Claude session."

## Master spec
Pointer to operator/docs/superpowers/specs/2026-05-20-licensing-platform-and-templates-design.md
```

---

## README.md outline (per template)

A common shape, plus per-archetype detail. Each `README.md` should contain these sections in this order:

```
# <archetype name> — untra platform template

## Purpose
One paragraph describing what an instance of this template does in a deployed untra product.

## Role in the platform
A short list of which other archetypes this template talks to and how.

## Tech stack
Language, framework, deployment target. From the locked Y stack.

## Repository layout
The directory shape this template prescribes.

## Quickstart (instantiation)
How a Phase 2 operator clones this template into a new product monorepo.

## Configuration
Parameters that must be set per product (e.g. `{root_domain}`, `{gcp_project_id}`).

## Cost profile
Free-tier story; what triggers paid usage.

## Status
"Phase 0: README and handoff brief only. See HANDOFF.md to start implementation."

## Links
Pointer to master spec, the platform stack table, related archetypes.
```

### Per-archetype README key facts

**`templates/iac/README.md`:**
- Purpose: Terraform/OpenTofu modules that provision a single untra product's cloud (public + private sides).
- Modules to enumerate: `cloudflare_zone`, `gcp_project`, `cloud_run_service`, `firebase_auth`, `neon_project`, `secret_manager`, `iap_admin`, `github_oidc_wif`.
- Two top-level compositions: `iac/public/` and `iac/private/`, run against separate GCP projects.
- Backend state: GCS bucket per product, configured via `terraform init -backend-config`.

**`templates/api/README.md`:**
- Purpose: generic product backend. Rust + Axum + Cloud Run.
- Endpoints (initial): `GET /version` (unauth), `GET /me` (entitlement-JWT-auth, returns user + active license summary), `POST /webhooks/lemonsqueezy` (HMAC-verified).
- Verifies entitlement JWT against `license` public key, embedded at build time.
- Talks to: Neon Postgres (user table, license cache table); calls `license` over signed HMAC internal request.

**`templates/app/README.md`:**
- Purpose: customer-facing web app at `app.<domain>`. TypeScript + React + Vite + Firebase JS SDK.
- Routes (initial): `/` (landing/download), `/signin` (redirects to `auth.<domain>`), `/account` (signed-in: shows licenses, machines), `/licenses/:id` (manage machines for a license).
- Calls `auth.<domain>` for sign-in flow; calls `api.<domain>` for authenticated data.
- Static-friendly: deployable to Cloud Storage + Cloud CDN or Cloudflare Pages.

**`templates/auth/README.md`:**
- Purpose: identity orchestrator at `auth.<domain>`. Cloud Run service + small static UI.
- Flow: user signs in via Firebase (email/password, magic-link, OAuth) → static UI captures Firebase ID token → `auth` calls `license.<private-domain>` to verify entitlement → returns entitlement JWT to client. Also handles license-key claim (first-time activation) and machine registration (machine-fingerprint binding within the 3-machine cap).
- Endpoints: `POST /claim` (Firebase token + license key + machine fingerprint), `POST /refresh` (Firebase token + machine fingerprint).

**`templates/admin/README.md`:**
- Purpose: admin console at `admin.<domain>`, gated by GCP IAP (no Firebase Auth — admin is internal).
- Routes (initial): `/plans` (define plans + feature bundles via OpenFeature schema), `/licenses` (search, view, revoke), `/users` (view accounts), `/billing` (LemonSqueezy passthrough links).
- Calls `api` for read data, calls `license` for write actions (mint license manually, revoke).

**`templates/license/README.md`:**
- Purpose: privileged entitlement service. Rust + Axum + Cloud Run. Lives in private sister repo only.
- **Embed the eight locked entitlement decisions verbatim** (see "Entitlement model — locked decisions" above) as a "Frozen entitlement model" section.
- Endpoints: `POST /internal/verify` (called by `auth`, HMAC-authed, returns entitlement JWT), `POST /internal/mint` (called by `api`, HMAC-authed, creates a license record), `POST /internal/revoke` (called by `admin`, HMAC-authed). All endpoints are internal-only (Cloud Run with IAM-based ingress restriction).
- Crypto: Ed25519 signing key in Secret Manager; public key available at a public, unauth, cacheable `GET /.well-known/license-public-key` endpoint (used by client-side OpenFeature provider and by `api` for JWT verification).
- Detailed token schema and key-rotation policy: **deferred to A-spec follow-up.**

---

## HANDOFF.md outline (per template)

Fixed structure across all six templates so the briefs are interchangeable. Each `HANDOFF.md` should contain these sections in this order:

```
# Handoff brief — <archetype>

> Read this file before starting implementation. You are a fresh Claude session
> with no prior context. The README.md in this same directory has the role and
> stack. This file tells you what "done" looks like for the first milestone.

## Pointer to master spec
Path: ~/.claude/plans/this-project-operator-is-rustling-tome.md
(or wherever the user has moved it after approval — check first).

## Role (one paragraph)
Restate the role from README. Confirms shared interpretation.

## Acceptance criteria (testable)
Concrete, runnable checks. Examples:
- "Produces a Cloud Run service that responds HTTP 200 to /healthz"
- "Returns HTTP 401 to /me when no Authorization header is present"
- "Terraform plan against a clean GCP project produces zero errors"

## Public interface
- HTTP endpoints / UI routes / Terraform variables / library exports.
- Schemas where they exist (link to A-spec for entitlement JWT schema).

## Dependencies
- Which other archetypes this calls (by name).
- Which other archetypes call this (by name).
- External services (Firebase, Neon, Cloudflare, LemonSqueezy).

## Non-goals
Explicit list of things NOT to build in this session. Examples for `api`:
- Do not implement license signing — that's `license`'s job.
- Do not build the admin endpoints — those live in `admin`.
- Do not add OpenTelemetry exporters yet; add `tracing` only.

## First milestone (smallest deployable slice)
A specific, minimal end-to-end slice that proves the template works.
Example for `api`: "GET /version returns {\"version\":\"0.1.0\"} as JSON,
deployed to Cloud Run min=0, served at api.<domain>."

## Out-of-scope flags for later milestones
List of features to leave as `// TODO(milestone-2)` comments so the next
session knows what comes next.
```

### Per-archetype HANDOFF key facts

**`templates/iac/HANDOFF.md`:**
- First milestone: a `terraform plan` for a hypothetical product `example-product` against a fresh GCP project produces a valid plan (no apply required this milestone).
- Non-goals: do not write a GitHub Actions workflow that runs `terraform apply` yet; do not provision DNS records for the private domain (private side is its own composition).

**`templates/api/HANDOFF.md`:**
- First milestone: `GET /version` returns the current version as JSON; service builds into a Cloud Run image via the included Dockerfile; `curl localhost:8080/version` works locally.
- Non-goals: do not implement webhook signature verification yet (stub it); do not implement `/me` yet; do not connect to Neon yet (stub the DB layer).

**`templates/app/HANDOFF.md`:**
- First milestone: a static site that renders a landing page with a "Download Operator" button and a "Sign In" link to `auth.<domain>/signin`; `npm run build` produces a deployable `dist/`.
- Non-goals: no `/account` page yet; no API integration; no Firebase wiring in JS yet (just a link).

**`templates/auth/HANDOFF.md`:**
- First milestone: a Cloud Run service exposing a static `/signin` page (Firebase UI or hand-rolled email-link form) that successfully signs a user in and shows their Firebase UID; `POST /claim` is stubbed and returns `501 Not Implemented`.
- Non-goals: do not call `license` yet (stub the call); do not implement machine-fingerprint logic yet; do not implement `/refresh`.

**`templates/admin/HANDOFF.md`:**
- First milestone: a Cloud Run service with IAP enforcement that renders a "Hello, {user.email}" page sourced from `X-Goog-Authenticated-User-Email`.
- Non-goals: no plan editor yet; no license search; no revocation UI; no API integration.

**`templates/license/HANDOFF.md`:**
- First milestone: a Cloud Run service with a `/healthz` endpoint and a `/.well-known/license-public-key` endpoint that returns a hardcoded Ed25519 public key (real key generation deferred to A-spec).
- Non-goals: do not implement `/internal/verify`, `/internal/mint`, or `/internal/revoke` yet; do not implement the revocation list; do not implement HMAC validation of internal callers yet (return `501` with a `TODO` comment); do not freeze the entitlement JWT schema (waits on A-spec follow-up).

---

## Critical files to be created

```
../templates/README.md                (top-level index, not in any git repo)
../templates/iac/README.md
../templates/iac/HANDOFF.md
../templates/iac/.git/                (git init only)
../templates/api/README.md
../templates/api/HANDOFF.md
../templates/api/.git/
../templates/app/README.md
../templates/app/HANDOFF.md
../templates/app/.git/
../templates/auth/README.md
../templates/auth/HANDOFF.md
../templates/auth/.git/
../templates/admin/README.md
../templates/admin/HANDOFF.md
../templates/admin/.git/
../templates/license/README.md
../templates/license/HANDOFF.md
../templates/license/.git/
```

Plus the promotion copy:

```
operator/docs/superpowers/specs/2026-05-20-licensing-platform-and-templates-design.md
```

Total: 13 files written, 6 directories `git init`'d, 1 spec file promoted. **Zero commits.**

---

## Verification

After implementation:

```bash
# 0. Top-level index exists.
test -f ../templates/README.md && echo "top README OK" || echo "TOP README MISSING"

# 1. All directories exist and are git repos.
for t in iac api app auth admin license; do
  test -d ../templates/$t/.git && echo "$t: git OK" || echo "$t: GIT MISSING"
done

# 2. Both files exist per template.
for t in iac api app auth admin license; do
  test -f ../templates/$t/README.md && test -f ../templates/$t/HANDOFF.md \
    && echo "$t: files OK" || echo "$t: files MISSING"
done

# 3. No stray files inside template repos.
find ../templates -mindepth 2 -maxdepth 2 -type f \
  ! -name README.md ! -name HANDOFF.md ! -path '*/.git/*' \
  | grep -v "^$" && echo "stray files present" || echo "no stray files"

# 4. No commits yet (user commits manually).
for t in iac api app auth admin license; do
  ( cd ../templates/$t && test -z "$(git log 2>/dev/null)" \
    && echo "$t: no commits OK" || echo "$t: HAS COMMITS" )
done

# 5. Promoted spec exists.
test -f operator/docs/superpowers/specs/2026-05-20-licensing-platform-and-templates-design.md \
  && echo "promoted spec OK" || echo "PROMOTED SPEC MISSING"
```

After Phase 1 (separate sessions, out of scope here): each template implements its first-milestone acceptance criteria.

---

## Non-goals of this plan (explicit)

- No code beyond `README.md` and `HANDOFF.md`.
- No commits anywhere.
- No registration of any cloud account, domain, or LemonSqueezy account.
- No detailed entitlement-JWT schema or key rotation policy (that's A-spec follow-up).
- No work in `operator/` itself in this session (Operator's licensing-client integration C is a future ticket).
- No GitHub Actions workflows, Dockerfiles, or Terraform code.
- No bootstrap script (user chose "manual checklist" — checklist content lives in `templates/iac/README.md` Quickstart section, not as separate code).

---

## Follow-ups queued after this plan executes

1. **A-spec brainstorm session** — flesh out the entitlement-JWT schema, key format, key rotation, machine-fingerprint algorithm. Produces the detailed `license` data model.
2. **Per-template implementation sessions (6 of them)** — each starts a fresh Claude in the relevant `../templates/<archetype>/` directory and works from HANDOFF.md.
3. **Operator integration (C)** — separate ticket in `operator/` to add the OpenFeature provider, license-key UI, refresh loop. Depends on A-spec being done.
4. **Promote this plan** — confirmed at approval. Copy to `operator/docs/superpowers/specs/2026-05-20-licensing-platform-and-templates-design.md` as part of this session's deliverable (user commits manually).

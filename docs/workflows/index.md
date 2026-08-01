---
title: "Workflows"
layout: doc
section: workflows
---

<!-- AUTO-GENERATED FROM src/collections/ + collections/community/ - DO NOT EDIT MANUALLY -->
<!-- Regenerate with: cargo run -- docs -->


An **Operator workflow** is a process defined once in JSON: an ordered graph of
typed steps, review gates, and retry edges that an LLM agent can follow. It is
the native format — Operator runs it directly, and every
[export format](/getting-started/workflows/) (Claude, AGNT) is derived from it.

Three terms, three different things:

| Term | What it is |
|------|-----------|
| **Operator workflow** | The step graph itself. Lives in an issue type's `steps`. |
| **Issue type** | One kind of work — `FEAT`, `PRD`, `ELVSTAGE`. Carries identity, input fields, and exactly one Operator workflow. |
| **Collection** | A named, versioned bundle of issue types: a complete, shareable way of working. This page lists them. |

Collections are deliberately separate from your **kanban issue types**. Jira,
Linear, and GitHub Projects types describe how *your* team labels work; a
collection describes how the *agents* do it. Map one onto the other once, and
the workflow travels between projects, teams, and providers unchanged.

Every collection below is installable from Operator directly — they are published from this site as a [machine-readable index](/collections/index.json) that operator instances read on startup.

<operator-collection-search for="collection-catalog"></operator-collection-search>

<div id="collection-catalog" data-view="cards">
<div class="collection-grid" data-view-target="cards">
  <article class="collection-card" data-search="autonomous builtin mit official only operator simple single_pass task untra with workflow">
    <a class="collection-card-link" href="/workflows/simple/">
      <svg class="collection-icon" aria-hidden="true" focusable="false" role="img" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><title>Simple</title><path d="M5 3h14a2 2 0 012 2v14a2 2 0 01-2 2H5a2 2 0 01-2-2V5a2 2 0 012-2z"/></svg>
      <h3 class="collection-card-title">Simple</h3>
    </a>
    <p class="collection-card-description">Simple workflow with TASK only</p>
    <p class="collection-card-types"><code>TASK</code></p>
    <p class="collection-card-meta">
      <span class="badge recommended">official</span>
      <span>1 issue type</span>
      <span>Operator!</span>
      <span>updated 2026-07-01</span>
    </p>
  </article>
  <article class="collection-card" data-search="autonomous builtin dev dev_kanban developer feat feature fix kanban mit official operator single_pass task test_suite untra with">
    <a class="collection-card-link" href="/workflows/dev_kanban/">
      <svg class="collection-icon" aria-hidden="true" focusable="false" role="img" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><title>Dev Kanban</title><path d="M2 4h5v16H2V4zm7.5 0h5v16h-5V4zM17 4h5v16h-5V4z"/></svg>
      <h3 class="collection-card-title">Dev Kanban</h3>
    </a>
    <p class="collection-card-description">Developer kanban with TASK, FEAT, FIX</p>
    <p class="collection-card-types"><code>TASK</code> <code>FEAT</code> <code>FIX</code></p>
    <p class="collection-card-meta">
      <span class="badge recommended">official</span>
      <span>3 issue types</span>
      <span>Operator!</span>
      <span>updated 2026-06-16</span>
    </p>
  </article>
  <article class="collection-card" data-search="autonomous builtin devops devops_kanban feat feature fix human inv investigation kanban mit official operator paired review_loop spike task test_suite untra with">
    <a class="collection-card-link" href="/workflows/devops_kanban/">
      <svg class="collection-icon" aria-hidden="true" focusable="false" role="img" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><title>DevOps Kanban</title><path d="M2 4h5v16H2V4zm7.5 0h5v11h-5V4zM17 4h5v6h-5V4z"/></svg>
      <h3 class="collection-card-title">DevOps Kanban</h3>
    </a>
    <p class="collection-card-description">DevOps kanban with TASK, FEAT, FIX, SPIKE, INV</p>
    <p class="collection-card-types"><code>TASK</code> <code>FEAT</code> <code>FIX</code> <code>SPIKE</code> <code>INV</code></p>
    <p class="collection-card-meta">
      <span class="badge recommended">official</span>
      <span>5 issue types</span>
      <span>Operator!</span>
      <span>updated 2026-06-16</span>
    </p>
  </article>
  <article class="collection-card" data-search="agent agent_setup assess assessment automation autonomous builtin catalog human init initialization mit official operator paired project project_init setup single_pass sync tasks untra workspace">
    <a class="collection-card-link" href="/workflows/operator/">
      <svg class="collection-icon" aria-hidden="true" focusable="false" role="img" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><title>Operator</title><path d="M12 8.5a3.5 3.5 0 110 7 3.5 3.5 0 010-7zM12 1a3 3 0 110 6 3 3 0 010-6zM4 16a3 3 0 110 6 3 3 0 010-6zM20 16a3 3 0 110 6 3 3 0 010-6z"/></svg>
      <h3 class="collection-card-title">Operator</h3>
    </a>
    <p class="collection-card-description">Operator automation tasks: ASSESS, SYNC, INIT</p>
    <p class="collection-card-types"><code>ASSESS</code> <code>SYNC</code> <code>INIT</code> <code>AGENT_SETUP</code> <code>PROJECT_INIT</code></p>
    <p class="collection-card-meta">
      <span class="badge recommended">official</span>
      <span>5 issue types</span>
      <span>Operator!</span>
      <span>updated 2026-07-01</span>
    </p>
  </article>
  <article class="collection-card" data-search="agent agentic-loop autonomous community completing context coordinator document for fresh fresh_context_story_loop loop mit one paired per plan_review prd prd-to-story product ralph ralph_loop requirements right-sized rloop snarktank stories story story_completion_check test_suite untra">
    <a class="collection-card-link" href="/workflows/ralph_loop/">
      <svg class="collection-icon" aria-hidden="true" focusable="false" role="img" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><title>Ralph Loop</title><path d="M12 4V1L8 5l4 4V6a6 6 0 11-6 6H4a8 8 0 108-8z"/></svg>
      <h3 class="collection-card-title">Ralph Loop</h3>
    </a>
    <p class="collection-card-description">PRD-to-story loop for completing one right-sized story per fresh agent context.</p>
    <p class="collection-card-types"><code>PRD</code> <code>STORY</code> <code>RLOOP</code></p>
    <p class="collection-card-meta">
      <span class="badge alpha">community</span>
      <span>3 issue types</span>
      <span>snarktank</span>
      <span>updated 2026-07-01</span>
    </p>
  </article>
  <article class="collection-card" data-search="agentic-loop and architect architect_review autonomous code_review coder community feature feature-graph feature/task feature_task_review_graph human_pr_review jr jr_orchestration jrfeat jrplan jrrebase jrrev jrtask mit orchestration paired plan rebase review reviewer snapwich task units untra with work">
    <a class="collection-card-link" href="/workflows/jr_orchestration/">
      <svg class="collection-icon" aria-hidden="true" focusable="false" role="img" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><title>JR Orchestration</title><path d="M12 1a3 3 0 110 6 3 3 0 010-6zM11 7h2v4h-2V7zM3 11h18v2H3v-2zM3 13h2v4H3v-4zm8 0h2v4h-2v-4zm8 0h2v4h-2v-4zM4 17a3 3 0 110 6 3 3 0 010-6zm8 0a3 3 0 110 6 3 3 0 010-6zm8 0a3 3 0 110 6 3 3 0 010-6z"/></svg>
      <h3 class="collection-card-title">JR Orchestration</h3>
    </a>
    <p class="collection-card-description">Feature/task orchestration with coder, reviewer, architect, and rebase work units.</p>
    <p class="collection-card-types"><code>JRPLAN</code> <code>JRFEAT</code> <code>JRTASK</code> <code>JRREV</code> <code>JRREBASE</code></p>
    <p class="collection-card-meta">
      <span class="badge alpha">community</span>
      <span>5 issue types</span>
      <span>snapwich</span>
      <span>updated 2026-07-01</span>
    </p>
  </article>
  <article class="collection-card" data-search="agentic-loop aigora and autonomous batch batch_validation community durable elvbatch elves elves_overnight elvrpt elvstage fresh_review human_land_gate judge_verdict land landpr long-running memory mit overnight paired pr pull report reporting request review stage stage_review staged staged_long_running_batch_loop untra validation with workflow">
    <a class="collection-card-link" href="/workflows/elves_overnight/">
      <svg class="collection-icon" aria-hidden="true" focusable="false" role="img" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><title>Elves Overnight</title><path d="M12 2a10 10 0 110 20 10 10 0 010-20zM16 1a8 8 0 100 16 8 8 0 000-16z"/></svg>
      <h3 class="collection-card-title">Elves Overnight</h3>
    </a>
    <p class="collection-card-description">Long-running staged batch workflow with durable memory, validation, PR review, and reporting.</p>
    <p class="collection-card-types"><code>ELVSTAGE</code> <code>ELVBATCH</code> <code>LANDPR</code> <code>ELVRPT</code></p>
    <p class="collection-card-meta">
      <span class="badge alpha">community</span>
      <span>4 issue types</span>
      <span>Aigora</span>
      <span>updated 2026-07-01</span>
    </p>
  </article>
  <article class="collection-card" data-search="agents and autonomous bug coder coding community delegated engineering feature flow improvement kanban kanban_synced_single_pass linear linear-synced mit plan_review pr_review test_suite to untra work">
    <a class="collection-card-link" href="/workflows/coder/">
      <svg class="collection-icon" aria-hidden="true" focusable="false" role="img" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><title>Coder</title><path d="M14.862 6.67H24v10.663h-9.138zM6.945 15.304c-1.934 0-3.366-1.264-3.366-3.305s1.432-3.323 3.366-3.365c1.411-.03 2.787.99 2.878 2.543l3.472-.106c-.076-2.802-2.33-4.706-6.35-4.706S0 8.558 0 12c0 3.426 3.046 5.635 6.945 5.635 3.898 0 6.29-1.935 6.38-4.782l-3.472-.077c-.152 1.553-1.497 2.528-2.908 2.528Z"/></svg>
      <h3 class="collection-card-title">Coder</h3>
    </a>
    <p class="collection-card-description">Linear-synced engineering flow: Feature, Improvement, and Bug work delegated to coding agents.</p>
    <p class="collection-card-types"><code>FEATURE</code> <code>IMPROVEMENT</code> <code>BUG</code></p>
    <p class="collection-card-meta">
      <span class="badge alpha">community</span>
      <span>3 issue types</span>
      <span>untra</span>
      <span>updated 2026-08-01</span>
    </p>
  </article>
  <article class="collection-card" data-search="autonomous chore chores collection community demonstrating example example_chores format minimal mit shareable single_pass starter test_suite the untra">
    <a class="collection-card-link" href="/workflows/example_chores/">
      <svg class="collection-icon" aria-hidden="true" focusable="false" role="img" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><title>Example Chores</title><path d="M3 5h2v2H3V5zm4 0h14v2H7V5zM3 11h2v2H3v-2zm4 0h14v2H7v-2zM3 17h2v2H3v-2zm4 0h14v2H7v-2z"/></svg>
      <h3 class="collection-card-title">Example Chores</h3>
    </a>
    <p class="collection-card-description">Minimal example community collection demonstrating the shareable format.</p>
    <p class="collection-card-types"><code>CHORE</code></p>
    <p class="collection-card-meta">
      <span class="badge alpha">community</span>
      <span>1 issue type</span>
      <span>untra</span>
      <span>updated 2026-07-01</span>
    </p>
  </article>
</div>
<table class="collection-table" data-view-target="table">
  <thead>
    <tr><th>Collection</th><th>Description</th><th>Issue types</th><th>Loop</th><th>Author</th><th>Tier</th><th>Created</th><th>Updated</th></tr>
  </thead>
  <tbody>
    <tr data-search="autonomous builtin mit official only operator simple single_pass task untra with workflow">
      <td><svg class="collection-icon" aria-hidden="true" focusable="false" role="img" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><title>Simple</title><path d="M5 3h14a2 2 0 012 2v14a2 2 0 01-2 2H5a2 2 0 01-2-2V5a2 2 0 012-2z"/></svg><a href="/workflows/simple/">Simple</a></td>
      <td>Simple workflow with TASK only</td>
      <td>1</td>
      <td>single_pass</td>
      <td>Operator!</td>
      <td><span class="badge recommended">official</span></td>
      <td>2026-01-08</td>
      <td>2026-07-01</td>
    </tr>
    <tr data-search="autonomous builtin dev dev_kanban developer feat feature fix kanban mit official operator single_pass task test_suite untra with">
      <td><svg class="collection-icon" aria-hidden="true" focusable="false" role="img" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><title>Dev Kanban</title><path d="M2 4h5v16H2V4zm7.5 0h5v16h-5V4zM17 4h5v16h-5V4z"/></svg><a href="/workflows/dev_kanban/">Dev Kanban</a></td>
      <td>Developer kanban with TASK, FEAT, FIX</td>
      <td>3</td>
      <td>single_pass</td>
      <td>Operator!</td>
      <td><span class="badge recommended">official</span></td>
      <td>2026-01-08</td>
      <td>2026-06-16</td>
    </tr>
    <tr data-search="autonomous builtin devops devops_kanban feat feature fix human inv investigation kanban mit official operator paired review_loop spike task test_suite untra with">
      <td><svg class="collection-icon" aria-hidden="true" focusable="false" role="img" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><title>DevOps Kanban</title><path d="M2 4h5v16H2V4zm7.5 0h5v11h-5V4zM17 4h5v6h-5V4z"/></svg><a href="/workflows/devops_kanban/">DevOps Kanban</a></td>
      <td>DevOps kanban with TASK, FEAT, FIX, SPIKE, INV</td>
      <td>5</td>
      <td>review_loop</td>
      <td>Operator!</td>
      <td><span class="badge recommended">official</span></td>
      <td>2026-01-08</td>
      <td>2026-06-16</td>
    </tr>
    <tr data-search="agent agent_setup assess assessment automation autonomous builtin catalog human init initialization mit official operator paired project project_init setup single_pass sync tasks untra workspace">
      <td><svg class="collection-icon" aria-hidden="true" focusable="false" role="img" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><title>Operator</title><path d="M12 8.5a3.5 3.5 0 110 7 3.5 3.5 0 010-7zM12 1a3 3 0 110 6 3 3 0 010-6zM4 16a3 3 0 110 6 3 3 0 010-6zM20 16a3 3 0 110 6 3 3 0 010-6z"/></svg><a href="/workflows/operator/">Operator</a></td>
      <td>Operator automation tasks: ASSESS, SYNC, INIT</td>
      <td>5</td>
      <td>single_pass</td>
      <td>Operator!</td>
      <td><span class="badge recommended">official</span></td>
      <td>2026-01-08</td>
      <td>2026-07-01</td>
    </tr>
    <tr data-search="agent agentic-loop autonomous community completing context coordinator document for fresh fresh_context_story_loop loop mit one paired per plan_review prd prd-to-story product ralph ralph_loop requirements right-sized rloop snarktank stories story story_completion_check test_suite untra">
      <td><svg class="collection-icon" aria-hidden="true" focusable="false" role="img" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><title>Ralph Loop</title><path d="M12 4V1L8 5l4 4V6a6 6 0 11-6 6H4a8 8 0 108-8z"/></svg><a href="/workflows/ralph_loop/">Ralph Loop</a></td>
      <td>PRD-to-story loop for completing one right-sized story per fresh agent context.</td>
      <td>3</td>
      <td>fresh_context_story_loop</td>
      <td>snarktank</td>
      <td><span class="badge alpha">community</span></td>
      <td>2026-06-16</td>
      <td>2026-07-01</td>
    </tr>
    <tr data-search="agentic-loop and architect architect_review autonomous code_review coder community feature feature-graph feature/task feature_task_review_graph human_pr_review jr jr_orchestration jrfeat jrplan jrrebase jrrev jrtask mit orchestration paired plan rebase review reviewer snapwich task units untra with work">
      <td><svg class="collection-icon" aria-hidden="true" focusable="false" role="img" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><title>JR Orchestration</title><path d="M12 1a3 3 0 110 6 3 3 0 010-6zM11 7h2v4h-2V7zM3 11h18v2H3v-2zM3 13h2v4H3v-4zm8 0h2v4h-2v-4zm8 0h2v4h-2v-4zM4 17a3 3 0 110 6 3 3 0 010-6zm8 0a3 3 0 110 6 3 3 0 010-6zm8 0a3 3 0 110 6 3 3 0 010-6z"/></svg><a href="/workflows/jr_orchestration/">JR Orchestration</a></td>
      <td>Feature/task orchestration with coder, reviewer, architect, and rebase work units.</td>
      <td>5</td>
      <td>feature_task_review_graph</td>
      <td>snapwich</td>
      <td><span class="badge alpha">community</span></td>
      <td>2026-06-16</td>
      <td>2026-07-01</td>
    </tr>
    <tr data-search="agentic-loop aigora and autonomous batch batch_validation community durable elvbatch elves elves_overnight elvrpt elvstage fresh_review human_land_gate judge_verdict land landpr long-running memory mit overnight paired pr pull report reporting request review stage stage_review staged staged_long_running_batch_loop untra validation with workflow">
      <td><svg class="collection-icon" aria-hidden="true" focusable="false" role="img" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><title>Elves Overnight</title><path d="M12 2a10 10 0 110 20 10 10 0 010-20zM16 1a8 8 0 100 16 8 8 0 000-16z"/></svg><a href="/workflows/elves_overnight/">Elves Overnight</a></td>
      <td>Long-running staged batch workflow with durable memory, validation, PR review, and reporting.</td>
      <td>4</td>
      <td>staged_long_running_batch_loop</td>
      <td>Aigora</td>
      <td><span class="badge alpha">community</span></td>
      <td>2026-06-16</td>
      <td>2026-07-01</td>
    </tr>
    <tr data-search="agents and autonomous bug coder coding community delegated engineering feature flow improvement kanban kanban_synced_single_pass linear linear-synced mit plan_review pr_review test_suite to untra work">
      <td><svg class="collection-icon" aria-hidden="true" focusable="false" role="img" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><title>Coder</title><path d="M14.862 6.67H24v10.663h-9.138zM6.945 15.304c-1.934 0-3.366-1.264-3.366-3.305s1.432-3.323 3.366-3.365c1.411-.03 2.787.99 2.878 2.543l3.472-.106c-.076-2.802-2.33-4.706-6.35-4.706S0 8.558 0 12c0 3.426 3.046 5.635 6.945 5.635 3.898 0 6.29-1.935 6.38-4.782l-3.472-.077c-.152 1.553-1.497 2.528-2.908 2.528Z"/></svg><a href="/workflows/coder/">Coder</a></td>
      <td>Linear-synced engineering flow: Feature, Improvement, and Bug work delegated to coding agents.</td>
      <td>3</td>
      <td>kanban_synced_single_pass</td>
      <td>untra</td>
      <td><span class="badge alpha">community</span></td>
      <td>2026-08-01</td>
      <td>2026-08-01</td>
    </tr>
    <tr data-search="autonomous chore chores collection community demonstrating example example_chores format minimal mit shareable single_pass starter test_suite the untra">
      <td><svg class="collection-icon" aria-hidden="true" focusable="false" role="img" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><title>Example Chores</title><path d="M3 5h2v2H3V5zm4 0h14v2H7V5zM3 11h2v2H3v-2zm4 0h14v2H7v-2zM3 17h2v2H3v-2zm4 0h14v2H7v-2z"/></svg><a href="/workflows/example_chores/">Example Chores</a></td>
      <td>Minimal example community collection demonstrating the shareable format.</td>
      <td>1</td>
      <td>single_pass</td>
      <td>untra</td>
      <td><span class="badge alpha">community</span></td>
      <td>2026-07-01</td>
      <td>2026-07-01</td>
    </tr>
  </tbody>
</table>
</div>

## Contribute a collection

There is no single best way to run agents — the right loop depends on the work.
That is exactly why these are shareable: a workflow that works for you is worth
publishing, and one that does not fit is worth forking.

Official collections live in the [operator repository](https://github.com/untra/operator/tree/main/collections):

1. Create `collections/community/<id>/`, where `<id>` matches `^[a-z0-9_]{3,64}$`.
2. Add a `collection.json` conforming to [the collection schema](/collections/schema.json),
   with `tier: "community"` plus `author`, `url`, and `license`.
3. Add one `<KEY>.json` per issue type — see [the issue type schema](/schemas/issuetype/) —
   and an optional `<KEY>.md` ticket template.
4. Add an `icon.svg` following the
   [Simple Icons](https://github.com/simple-icons/simple-icons) shape: a 24×24
   viewBox, a single `<path>`, and no `fill` or `stroke` so it inherits the
   page's color.
5. Leave checksums out — they are computed at publish time.
6. Run the CI gate locally, then open a pull request:

```bash
cargo test --test community_collections
```

Submissions are reviewed for prompt quality and safety, not just schema
validity. A good collection describes a workflow shape worth sharing: what loop
it runs, what memory it keeps, what gates it enforces, and when it stops.

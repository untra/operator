---
title: "Campaigns"
description: "Group many tickets into one planned body of work, ordered by their dependencies into waves that run in parallel."
layout: doc
---

A **campaign** is an epic of work: many [tickets](/getting-started/tickets/) planned together. Each ticket's `depends_on` links form a dependency graph, and Operator orders that graph into **waves** - every ticket in a wave can run at the same time, and each wave waits only on earlier ones.

Campaigns are part of [Operator Premium](/getting-started/premium/).

## A campaign file

Campaigns live next to the ticket queue, one markdown file each:

```
.tickets/campaigns/CAMP-0003-pricing-overhaul.md
```

```yaml
---
id: CAMP-0003
title: Pricing overhaul
status: draft
created: 2026-10-05
governance: team-default
workspace:
  kind: coder
  template: operator-agent
external_ref: jira:PROJ-1
---
# Pricing overhaul

What this campaign is for, written for the agents that will work it.
```

| Field | Description |
|-------|-------------|
| `id` | `CAMP-NNNN`, assigned in sequence |
| `title` | Human-readable name |
| `status` | `draft`, `ready`, `running`, `paused`, `completed` or `cancelled` |
| `governance` | A [governance](/getting-started/concepts/governance/) policy layered over every delegator's own |
| `workspace` | Optional shared environment - see [Workspaces](#workspaces) |
| `external_ref` | The epic, project or parent issue the campaign was synced from |

## Membership

A ticket joins a campaign by naming it in its own frontmatter:

```yaml
campaign: CAMP-0003
depends_on:
  - FEAT-0012
  - jira:PROJ-12
```

The campaign file never lists its tickets - the tickets are the source of truth.

## Waves

Operator plans the campaign's open tickets into waves:

- a dependency on another ticket in the campaign orders it into a later wave;
- a dependency on a ticket **outside** the campaign holds the ticket until that ticket is completed;
- a dependency on an upstream issue that isn't synced keeps the ticket blocked until it is synced, or the entry is removed;
- a dependency cycle is reported and nothing in it is planned;
- within a wave, tickets follow the same order as the [kanban board](/getting-started/concepts/kanban/#card-order).

## Lifecycle

| Status | Meaning |
|--------|---------|
| `draft` | Being planned. Tickets can join and leave. |
| `ready` | Planned and waiting to start. Set it by hand when a campaign is signed off. |
| `running` | Operator launches each wave as the one before it completes. |
| `paused` | No new launches. Agents already running keep running. |
| `completed` | Every member ticket is completed. |
| `cancelled` | Abandoned. Set it by hand; a cancelled campaign never starts again. |

A campaign starts from `draft`, `ready` or `paused`. Starting it launches the first wave at once; `operator api` then launches each later wave as tickets complete. Starting a running, completed or cancelled campaign is refused.

Operator pauses a running campaign on its own, with a reason, when:

- its `max_launches` or `max_retries` cap is used up;
- a launch fails;
- its tickets form a dependency cycle.

Resuming a paused campaign clears the reason. [Paired](/getting-started/agents/#autonomous-and-paired-modes) tickets are never launched automatically - the campaign reports them as **awaiting operator** until you launch them yourself.

Campaigns launch agents unattended, so they need a session wrapper that runs without a client: tmux, zellij or cmux. A VS Code terminal wrapper is refused.

## Planning with an agent over MCP

An agent connected to the [Operator MCP server](/getting-started/integrations/) can plan and start a campaign on its own. The tools are meant to be called in this order:

1. `operator_list_issue_types` - learn which templates exist (`FEAT`, `FIX`, `TASK`, ...).
2. `operator_preview_ticket_batch` - check a batch: allocated ids, waves, and any cycle or missing dependency. Nothing is written.
3. `operator_create_campaign` - create a `draft` campaign.
4. `operator_create_tickets_bulk` - write the batch inside the campaign. Every ticket is written, or none is.
5. `operator_campaign_status` - inspect the waves and what would launch first.
6. `operator_launch_campaign` - with `dry_run: true` to see violations, then for real.

Tickets in a batch name each other by `ref`, so dependencies can be wired before any id exists. A diamond - one ticket, two that build on it, and one that joins them:

```json
{
  "campaign": "CAMP-0003",
  "tickets": [
    { "ref": "schema", "template": "TASK", "project": "api", "summary": "Add price tiers table", "values": {}, "depends_on": [] },
    { "ref": "api", "template": "FEAT", "project": "api", "summary": "Serve price tiers", "values": {}, "depends_on": ["schema"] },
    { "ref": "ui", "template": "FEAT", "project": "web", "summary": "Show price tiers", "values": {}, "depends_on": ["schema"] },
    { "ref": "launch", "template": "TASK", "project": "web", "summary": "Enable new pricing", "values": {}, "depends_on": ["api", "ui"] }
  ]
}
```

That plans into three waves: `schema`, then `api` and `ui` together, then `launch`. `depends_on` also accepts existing ticket ids and `provider:key` references.

Previewing and bulk-creating tickets are free. Bulk creation is off unless `[mcp].expose_ticket_write_tools = true`. Creating, launching and inspecting campaigns need Premium.

## Watching a campaign

- **Web UI** - the **Campaigns** page lists every campaign; each one shows its waves, what is blocked and why, and its usage against the governance caps, with Launch and Pause.
- **VS Code** - the **Campaigns** view in the Operator sidebar shows each campaign, its waves and their tickets, with launch, pause and open-in-browser actions.
- **Command line:**

```bash
operator campaign list
operator campaign show CAMP-0003
operator campaign launch CAMP-0003
```

## Workspaces

By default each ticket runs in its own workspace, exactly as it would outside a campaign. A campaign may instead name one shared environment for all of its tickets:

```yaml
workspace:
  kind: coder
  template: operator-agent       # one Coder workspace from this template
```

```yaml
workspace:
  kind: docker
  image: untra/operator:latest   # one container from this image
```

Tickets in the same wave run in that environment in parallel, each on its own git worktree and branch. A shared Coder workspace is remote execution, so it also follows the rules for [remote targets](/getting-started/remote-targets/).

## Governance

A campaign's `governance` policy is stacked over the policy of each delegator that launches its tickets. The strictest rule wins, so a campaign can narrow what its tickets may use but never widen it. See [Governance](/getting-started/concepts/governance/).

## Syncing an epic or project

A kanban project can be synced as one campaign by giving its sync entry a `scope`. Every issue in that container is pulled in - whoever it is assigned to - and stamped with the campaign:

```toml
[kanban.jira."acme.atlassian.net".projects."PROJ-1"]
scope = { jira_epic = "PROJ-1" }
```

| Scope | Members |
|-------|---------|
| `{ jira_epic = "PROJ-1" }` | The epic's child issues |
| `{ linear_project = "<project-id>" }` | The project's issues |
| `{ github_parent = "owner/repo#3" }` | The parent issue's open sub-issues |
| `{ openspec_change = "<change-id>" }` | The change's task groups |

An OpenSpec change can also be imported as a campaign without any config:

```bash
operator import openspec add-dark-mode --campaign
```

The first sync creates the campaign, titled after the epic, project or parent; later syncs reuse it. A ticket you move to another campaign stays where you put it.

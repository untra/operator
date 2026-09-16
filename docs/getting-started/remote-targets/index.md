---
title: "Remote Targets"
description: "Premium execution targets for delegating work to another machine."
layout: doc
---

**Premium:** remote targets are available with a valid license for the selected configuration.

A remote target names an execution environment for a delegator. Operator keeps the queue and orchestration; the target runs agent processes, which report workflow progress through `opr8r` callbacks.

| Target | Status | Connection |
|---|---|---|
| [SSH Hosts](/getting-started/remote-targets/ssh/) | Alpha | An SSH alias and remote working directory |
| [Coder](/getting-started/remote-targets/coder/) | Alpha | Per-ticket Coder workspaces |

Local multi-agent execution and local containers remain included. Remote model inference and remote access to the dashboard do not require a remote execution entitlement.

## Limitations in this release

Operator enforces these when a delegator resolves to an SSH or Coder target,
rather than failing later in the launch:

| Constraint | Why |
|---|---|
| Git worktrees are disabled | The worktree is created where Operator runs, not where the agent does |
| Relay injection is disabled | The relay hub is a local socket and is not reachable from the target |
| The Zellij session wrapper is rejected | The launch fails rather than silently running somewhere unexpected |

A remote launch therefore works in the checkout at the target's working
directory. Use one target per concurrent piece of work, or separate working
directories, rather than relying on worktree isolation.

Targets remain visible when a license expires. Existing work can report completion; a valid license is required to start further remote work.

---
title: "Setup Wizard"
layout: doc
---

<!-- AUTO-GENERATED FROM src/startup/steps.rs - DO NOT EDIT MANUALLY -->
<!-- Regenerate with: cargo run -- docs -->

When Operator starts and no `.tickets/` directory exists, the setup wizard guides you through first-time initialization. This reference documents each step of the wizard.

## Steps Overview

| Step | Name | Description |
| --- | --- | --- |
| 1 | Welcome | Splash screen showing detected LLM tools and discovered projects |
| 2 | Kanban Info | Connect a kanban provider, or skip and connect one later |
| 3 | Model Server | Declare which model providers this workspace uses |
| 4 | Git Provider | Connect a git provider so agents can branch, push and open PRs |
| 5 | Collection Source | Choose which issue type collection to use |
| 6 | Hosted Collections | Browse and select hosted collections (only shown if Browse chosen) |
| 7 | Task Field Config | Configure optional fields for TASK issue type |
| 8 | Session Wrapper Choice | Select which session wrapper to use for launching coding agents |
| 9 | Execution Target | Choose whether agents run locally or in Coder workspaces |
| 10 | Worktree Preference | Choose whether to use git worktrees for ticket isolation |
| 11 | Web UI Password | Optionally set the admin password for the web dashboard |
| 12 | Tmux Onboarding | Help and documentation about tmux session management (shown if tmux selected) |
| 13 | VS Code Setup | VS Code extension setup and verification (shown if VS Code selected) |
| 14 | Cmux Setup | cmux session wrapper setup (shown if cmux selected) |
| 15 | Zellij Setup | Zellij session wrapper setup (shown if Zellij selected) |
| 16 | Acceptance Criteria | Review and configure acceptance criteria for ticket completion |
| 17 | Startup Tickets | Optionally create tickets to bootstrap your projects |
| 18 | Confirm | Review settings and confirm initialization |

## Step Details


### 1. Welcome

*Splash screen showing detected LLM tools and discovered projects*

The welcome screen displays:
- Detected LLM tools (Claude, Gemini, Codex, etc.) with version and model count
- Discovered projects organized by which LLM tool marker files they contain
- The path where the tickets directory will be created

This gives you an overview of your development environment before proceeding.

**Navigation**: Enter to continue, Esc to cancel

### 2. Kanban Info

*Connect a kanban provider, or skip and connect one later*

Operator can sync with external kanban providers to pull in issues as tickets.
Supported providers: Jira, Linear, GitHub Projects.

Credentials already exported (e.g. OPERATOR_JIRA_API_KEY) are listed as detected providers.

**Connect a kanban provider** opens the same onboarding dialog the dashboard uses: pick a provider, enter its credentials, validate them against the live API, and choose a project. The provider section is written to config.toml and the token is exported into this session, with a shell snippet to make it permanent.

**Skip for now** moves on; press `K` from the dashboard at any time.

**Navigation**: ↑/↓ to select, Enter to confirm, Esc to go back

### 3. Model Server

*Declare which model providers this workspace uses*

Model providers are where inference happens - distinct from the agent CLI that calls them.

Each provider is probed live: a row reads `N models` when Operator can reach it, `key missing` when its API key env var is unset, or `unreachable` with the reason.

Space declares a provider, writing a `[[model_servers]]` entry. Operator stores only the *name* of the environment variable holding the key, never the key itself - export it in your shell to make it permanent.

Providers needing a custom base URL (OpenAI-compatible, LM Studio) are listed but not selectable here; add them to config.toml directly.

This step is optional - Operator ships working defaults for the first-party vendors.

**Navigation**: ↑/↓ or j/k to navigate, Space to declare, Enter to continue, Esc to go back

### 4. Git Provider

*Connect a git provider so agents can branch, push and open PRs*

Operator branches per ticket and opens pull requests on your behalf, which needs a provider and a token.

Each row reports what was found: the provider CLI (`gh`, `glab`, `tea`) not installed, an existing CLI login Operator can adopt with no typing, or a prompt for a personal access token.

Only the *name* of the environment variable holding the token is written to config.toml. The token itself is exported into this session, and the step prints the shell line to make that permanent - without it the token is gone when Operator exits.

This step is optional; Operator works against a local repository with no provider connected.

**Navigation**: ↑/↓ or j/k to navigate, Enter to connect, Esc to go back

### 5. Collection Source

*Choose which issue type collection to use*

Select a preset collection of issue types:
- **Simple**: Just TASK - minimal setup for general work
- **Dev Kanban**: 3 types (TASK, FEAT, FIX) for development workflows
- **DevOps Kanban**: 5 types (TASK, SPIKE, INV, FEAT, FIX) for full DevOps
- **Custom Selection**: Choose individual issue types

**Navigation**: ↑/↓ or j/k to navigate, Enter to select, Esc to go back

### 6. Hosted Collections

*Browse and select hosted collections (only shown if Browse chosen)*

Pick one or more curated collections published at             operator.untra.io.

The list is fetched from the collections manifest; if it cannot be             reached, the collections bundled with Operator are offered instead.             Each collection brings its own issue types and workflow steps.

Selections are additive - choose as many as apply.

**Navigation**: ↑/↓ or j/k to navigate, Space to toggle, Enter to continue, Esc to go back

### 7. Task Field Config

*Configure optional fields for TASK issue type*

TASK is the foundational issue type. Configure which optional fields to include:
- **priority**: Priority level (P0-critical to P3-low)
- **points**: Story points estimate
- **user_story**: User story or background context

These choices propagate to other issue types. The 'summary' field is always required, and 'id' is auto-generated.

**Navigation**: ↑/↓ or j/k to navigate, Space to toggle, Enter to continue, Esc to go back

### 8. Session Wrapper Choice

*Select which session wrapper to use for launching coding agents*

Choose how Operator will manage coding agent sessions:
- **tmux**: Terminal multiplexer, recommended for most setups
- **VS Code**: Launch agents as VS Code tasks (requires extension)
- **cmux**: Native macOS terminal for AI agents, organized into windows and workspaces
- **Zellij**: Modern terminal workspace with built-in layouts

Your choice determines which setup steps follow.

**Navigation**: ↑/↓ or j/k to navigate, Enter to select, Esc to go back

### 9. Execution Target

*Choose whether agents run locally or in Coder workspaces*

Local runs agent commands on the same machine as Operator. Coder creates or starts a per-ticket workspace and launches there over SSH.

Coder configuration stores only environment variable names for the deployment URL and session token. Secret values remain in the process environment.

Coder targets disable git worktrees and relay injection, and cannot be combined with Zellij.

**Navigation**: ↑/↓ to select, Tab to switch fields, Enter to continue, Esc to go back

### 10. Worktree Preference

*Choose whether to use git worktrees for ticket isolation*

Configure how Operator manages git branches per ticket:
- **In-place branches**: Each agent works in the main checkout, switching branches
- **Git worktrees**: Each ticket gets its own worktree directory for full isolation

Worktrees allow multiple agents to work on different tickets simultaneously without branch conflicts.

**Navigation**: ↑/↓ or j/k to navigate, Enter to select, Esc to go back

### 11. Web UI Password

*Optionally set the admin password for the web dashboard*

Operator has a single human account, `admin`.

This terminal and the CLI need no password: a loopback process             authenticates with an owner-only token file in the state directory.             A browser cannot read that file, so the web dashboard stays locked             until an admin password exists.

Leave both fields blank to skip. You can set one later with             `operator auth bootstrap` or from the /setup page.

The password must be at least 12 characters. This step is hidden             when an admin account already exists.

**Navigation**: Tab to switch fields, Enter to continue (blank to skip), Esc to go back

### 12. Tmux Onboarding

*Help and documentation about tmux session management (shown if tmux selected)*

Operator launches Coding agents in tmux sessions. Essential commands:
- **Detach from session**: Ctrl+a (quick, no prefix needed!)
- **Fallback detach**: Ctrl+b then d
- **List sessions**: `tmux ls`
- **Attach to session**: `tmux attach -t <name>`

Operator session names start with 'op-' for easy identification.

**Navigation**: Enter to continue, Esc to go back

### 13. VS Code Setup

*VS Code extension setup and verification (shown if VS Code selected)*

Operator integrates with the VS Code extension to launch agents as tasks.
This step verifies the extension is installed and the webhook server is reachable.

Install the extension from the VS Code marketplace if prompted.

**Navigation**: Enter to continue, Esc to go back

### 14. Cmux Setup

*cmux session wrapper setup (shown if cmux selected)*

cmux is a native macOS terminal that organizes AI agent sessions into windows and workspaces.

This step verifies the cmux app's CLI binary exists at the configured binary_path (by default inside /Applications/cmux.app) and meets the minimum supported version.

**Navigation**: Enter to continue, Esc to go back

### 15. Zellij Setup

*Zellij session wrapper setup (shown if Zellij selected)*

Zellij is a modern terminal workspace with built-in layouts and multiplexing.

This step verifies Zellij is installed and configures the layout Operator will use when launching agents.

**Navigation**: Enter to continue, Esc to go back

### 16. Acceptance Criteria

*Review and configure acceptance criteria for ticket completion*

Define what 'done' means for tickets in this workspace.
Acceptance criteria are checked by agents before marking a ticket complete.

The default criteria cover formatting, tests, and lint checks. You can customize them for your team's standards.

**Navigation**: Enter to continue, Esc to go back

### 17. Startup Tickets

*Optionally create tickets to bootstrap your projects*

Create startup tickets to help initialize your projects:
- **ASSESS tickets**: Scan projects for catalog-info.yaml, create if missing
- **AGENT_SETUP tickets**: Configure Claude agents for each project
- **PROJECT_INIT tickets**: Run both ASSESS and AGENT_SETUP for each project

These tickets are optional and help automate common setup tasks.

**Navigation**: ↑/↓ or j/k to navigate, Space to toggle, Enter to continue, Esc to go back

### 18. Confirm

*Review settings and confirm initialization*

Review your configuration before initialization:
- Path where `.tickets/` will be created
- Selected issue types and preset name
- Directories that will be created: queue/, in-progress/, completed/, templates/

Choose Initialize to create the ticket queue, or Cancel to exit without changes.

**Navigation**: Tab or Space to toggle selection, Enter to confirm, Esc to go back

## Keyboard Shortcuts

Common keys used throughout the setup wizard:

| Key | Action |
| --- | --- |
| `Enter` | Confirm/Continue |
| `Esc` | Go back/Cancel |
| `↑`/`↓` or `j`/`k` | Navigate list items |
| `Space` | Toggle selection |
| `Tab` | Switch between options |


---
title: "Setup Wizard"
layout: doc
---

<!-- AUTO-GENERATED FROM src/startup/mod.rs - DO NOT EDIT MANUALLY -->
<!-- Regenerate with: cargo run -- docs -->

# Setup Wizard

When Operator starts and no `.tickets/` directory exists, the setup wizard guides you through first-time initialization. This reference documents each step of the wizard.

## Steps Overview

| Step | Name | Description |
| --- | --- | --- |
| 1 | Welcome | Splash screen showing detected LLM tools and discovered projects |
| 2 | Session Wrapper Choice | Select which session wrapper to use for launching coding agents |
| 3 | Worktree Preference | Choose whether to use git worktrees for ticket isolation |
| 4 | Tmux Onboarding | Help and documentation about tmux session management (shown if tmux selected) |
| 5 | VS Code Setup | VS Code extension setup and verification (shown if VS Code selected) |
| 6 | Cmux Setup | cmux session wrapper setup (shown if cmux selected) |
| 7 | Zellij Setup | Zellij session wrapper setup (shown if Zellij selected) |
| 8 | Kanban Info | Kanban integration overview and provider credential detection |
| 9 | Kanban Provider Setup | Per-provider credential validation and project selection |
| 10 | Collection Source | Choose which issue type collection to use |
| 11 | Custom Collection | Select individual issue types (only shown if Custom Selection chosen) |
| 12 | Task Field Config | Configure optional fields for TASK issue type |
| 13 | Acceptance Criteria | Review and configure acceptance criteria for ticket completion |
| 14 | Startup Tickets | Optionally create tickets to bootstrap your projects |
| 15 | Confirm | Review settings and confirm initialization |

## Step Details


### 1. Welcome

*Splash screen showing detected LLM tools and discovered projects*

The welcome screen displays:
- Detected LLM tools (Claude, Gemini, Codex, etc.) with version and model count
- Discovered projects organized by which LLM tool marker files they contain
- The path where the tickets directory will be created

This gives you an overview of your development environment before proceeding.

**Navigation**: Enter to continue, Esc to cancel

### 2. Session Wrapper Choice

*Select which session wrapper to use for launching coding agents*

Choose how Operator will manage coding agent sessions:
- **tmux**: Terminal multiplexer, recommended for most setups
- **VS Code**: Launch agents as VS Code tasks (requires extension)
- **cmux**: Lightweight tmux wrapper with operator defaults pre-applied
- **Zellij**: Modern terminal workspace with built-in layouts

Your choice determines which setup steps follow.

**Navigation**: ↑/↓ or j/k to navigate, Enter to select, Esc to go back

### 3. Worktree Preference

*Choose whether to use git worktrees for ticket isolation*

Configure how Operator manages git branches per ticket:
- **In-place branches**: Each agent works in the main checkout, switching branches
- **Git worktrees**: Each ticket gets its own worktree directory for full isolation

Worktrees allow multiple agents to work on different tickets simultaneously without branch conflicts.

**Navigation**: ↑/↓ or j/k to navigate, Enter to select, Esc to go back

### 4. Tmux Onboarding

*Help and documentation about tmux session management (shown if tmux selected)*

Operator launches Coding agents in tmux sessions. Essential commands:
- **Detach from session**: Ctrl+a (quick, no prefix needed!)
- **Fallback detach**: Ctrl+b then d
- **List sessions**: `tmux ls`
- **Attach to session**: `tmux attach -t <name>`

Operator session names start with 'op-' for easy identification.

**Navigation**: Enter to continue, Esc to go back

### 5. VS Code Setup

*VS Code extension setup and verification (shown if VS Code selected)*

Operator integrates with the VS Code extension to launch agents as tasks.
This step verifies the extension is installed and the webhook server is reachable.

Install the extension from the VS Code marketplace if prompted.

**Navigation**: Enter to continue, Esc to go back

### 6. Cmux Setup

*cmux session wrapper setup (shown if cmux selected)*

cmux is a lightweight tmux wrapper that pre-applies Operator's preferred session defaults.

This step verifies cmux is installed and accessible in your PATH.

**Navigation**: Enter to continue, Esc to go back

### 7. Zellij Setup

*Zellij session wrapper setup (shown if Zellij selected)*

Zellij is a modern terminal workspace with built-in layouts and multiplexing.

This step verifies Zellij is installed and configures the layout Operator will use when launching agents.

**Navigation**: Enter to continue, Esc to go back

### 8. Kanban Info

*Kanban integration overview and provider credential detection*

Operator can sync with external kanban providers to pull in issues as tickets.
Supported providers: Jira, Linear, GitHub Projects.

Credentials are read from environment variables (e.g. OPERATOR_JIRA_API_KEY). This step shows which providers were detected and validates connectivity.

**Navigation**: Enter to continue, Esc to go back

### 9. Kanban Provider Setup

*Per-provider credential validation and project selection*

For each detected provider, Operator:
1. Validates your API credentials against the provider
2. Fetches your workspace and user information
3. Discovers available projects for you to select

Only projects you select will be synced to your ticket queue. You can skip this step to configure kanban providers later.

**Navigation**: ↑/↓ or j/k to navigate, Space to select projects, Enter to confirm, Esc to go back

### 10. Collection Source

*Choose which issue type collection to use*

Select a preset collection of issue types:
- **Simple**: Just TASK - minimal setup for general work
- **Dev Kanban**: 3 types (TASK, FEAT, FIX) for development workflows
- **DevOps Kanban**: 5 types (TASK, SPIKE, INV, FEAT, FIX) for full DevOps
- **Custom Selection**: Choose individual issue types

**Navigation**: ↑/↓ or j/k to navigate, Enter to select, Esc to go back

### 11. Custom Collection

*Select individual issue types (only shown if Custom Selection chosen)*

Toggle individual issue types to include:
- **TASK**: Focused task that executes one specific thing
- **FEAT**: New feature or enhancement
- **FIX**: Bug fix, follow-up work, tech debt
- **SPIKE**: Research or exploration (paired mode)
- **INV**: Incident investigation (paired mode)

At least one issue type must be selected to proceed.

**Navigation**: ↑/↓ or j/k to navigate, Space to toggle, Enter to continue, Esc to go back

### 12. Task Field Config

*Configure optional fields for TASK issue type*

TASK is the foundational issue type. Configure which optional fields to include:
- **priority**: Priority level (P0-critical to P3-low)
- **points**: Story points estimate
- **user_story**: User story or background context

These choices propagate to other issue types. The 'summary' field is always required, and 'id' is auto-generated.

**Navigation**: ↑/↓ or j/k to navigate, Space to toggle, Enter to continue, Esc to go back

### 13. Acceptance Criteria

*Review and configure acceptance criteria for ticket completion*

Define what 'done' means for tickets in this workspace.
Acceptance criteria are checked by agents before marking a ticket complete.

The default criteria cover formatting, tests, and lint checks. You can customize them for your team's standards.

**Navigation**: Enter to continue, Esc to go back

### 14. Startup Tickets

*Optionally create tickets to bootstrap your projects*

Create startup tickets to help initialize your projects:
- **ASSESS tickets**: Scan projects for catalog-info.yaml, create if missing
- **AGENT-SETUP tickets**: Configure Claude agents for each project
- **PROJECT-INIT tickets**: Run both ASSESS and AGENT-SETUP for each project

These tickets are optional and help automate common setup tasks.

**Navigation**: ↑/↓ or j/k to navigate, Space to toggle, Enter to continue, Esc to go back

### 15. Confirm

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


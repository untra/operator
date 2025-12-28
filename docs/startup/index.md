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
| 2 | Collection Source | Choose which ticket template collection to use |
| 3 | Custom Collection | Select individual issue types (only shown if Custom Selection chosen) |
| 4 | Task Field Config | Configure optional fields for TASK issue type |
| 5 | Tmux Onboarding | Help and documentation about tmux session management |
| 6 | Startup Tickets | Optionally create tickets to bootstrap your projects |
| 7 | Confirm | Review settings and confirm initialization |

## Step Details


### 1. Welcome

*Splash screen showing detected LLM tools and discovered projects*

The welcome screen displays:
- Detected LLM tools (Claude, Gemini, Codex, etc.) with version and model count
- Discovered projects organized by which LLM tool marker files they contain
- The path where the tickets directory will be created

This gives you an overview of your development environment before proceeding.

**Navigation**: Enter to continue, Esc to cancel

### 2. Collection Source

*Choose which ticket template collection to use*

Select a preset collection of issue types:
- **Simple**: Just TASK - minimal setup for general work
- **Dev Kanban**: 3 types (TASK, FEAT, FIX) for development workflows
- **DevOps Kanban**: 5 types (TASK, SPIKE, INV, FEAT, FIX) for full DevOps
- **Import from Jira**: (Coming soon)
- **Import from Notion**: (Coming soon)
- **Custom Selection**: Choose individual issue types

**Navigation**: ↑/↓ or j/k to navigate, Enter to select, Esc to go back

### 3. Custom Collection

*Select individual issue types (only shown if Custom Selection chosen)*

Toggle individual issue types to include:
- **TASK**: Focused task that executes one specific thing
- **FEAT**: New feature or enhancement
- **FIX**: Bug fix, follow-up work, tech debt
- **SPIKE**: Research or exploration (paired mode)
- **INV**: Incident investigation (paired mode)

At least one issue type must be selected to proceed.

**Navigation**: ↑/↓ or j/k to navigate, Space to toggle, Enter to continue, Esc to go back

### 4. Task Field Config

*Configure optional fields for TASK issue type*

TASK is the foundational issue type. Configure which optional fields to include:
- **priority**: Priority level (P0-critical to P3-low)
- **context**: Background context for the task

These choices propagate to other issue types. The 'summary' field is always required, and 'id' is auto-generated.

**Navigation**: ↑/↓ or j/k to navigate, Space to toggle, Enter to continue, Esc to go back

### 5. Tmux Onboarding

*Help and documentation about tmux session management*

Operator launches Claude agents in tmux sessions. Essential commands:
- **Detach from session**: Ctrl+a (quick, no prefix needed!)
- **Fallback detach**: Ctrl+b then d
- **List sessions**: `tmux ls`
- **Attach to session**: `tmux attach -t <name>`

Operator session names start with 'op-' for easy identification.

**Navigation**: Enter to continue, Esc to go back

### 6. Startup Tickets

*Optionally create tickets to bootstrap your projects*

Create startup tickets to help initialize your projects:
- **ASSESS tickets**: Scan projects for catalog-info.yaml, create if missing
- **AGENT-SETUP tickets**: Configure Claude agents for each project
- **PROJECT-INIT tickets**: Initialize projects with Operator conventions

These tickets are optional and help automate common setup tasks.

**Navigation**: ↑/↓ or j/k to navigate, Space to toggle, Enter to continue, Esc to go back

### 7. Confirm

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


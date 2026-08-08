---
title: "Tickets"
description: "Create and manage tickets with markdown format, naming conventions, and best practices for LLM agents."
layout: doc
---

Tickets are the unit of work in <span class="operator-brand">Operator!</span>. Each one describes a task for an agent to complete, and carries an **issue type** that decides *how* the work is done — see [Workflows](/workflows/) for the process behind the ticket.

## Ticket Format

Tickets are markdown files with a specific naming convention:

```
{TYPE}-{ID}-{project}-{description}.md
```

`{TYPE}` is the issue type key (`FEAT`, `FIX`, `PRD`, …), which is why keys never contain hyphens — the hyphen separates the key from the ticket number.

### Examples

```
INV-001-backend-investigate-login-failure.md
FIX-042-api-fix-null-pointer.md
FEAT-123-frontend-add-dark-mode.md
SPIKE-007-platform-evaluate-kubernetes.md
```

## Ticket Structure

A typical ticket contains:

```markdown
# FEAT-123: Add Dark Mode

## Summary
Implement a dark mode toggle in the application settings.

## Requirements
- Add toggle switch in settings page
- Persist preference to local storage
- Apply theme to all components
- Support system preference detection

## Acceptance Criteria
- [ ] Toggle switch works
- [ ] Theme persists across sessions
- [ ] All components support both themes

## Notes
See design mockup in Figma: [link]
```

The exact fields a ticket carries are defined by its issue type. See the
[ticket metadata schema](/schemas/metadata/) for the YAML frontmatter format.

## Creating Tickets

### Manual Creation

1. Create a markdown file in `.tickets/queue/`
2. Follow the naming convention
3. Add ticket content

### Using the CLI

```bash
# Show current queue
cargo run -- queue

# Launch next ticket
cargo run -- launch
```

### From a Kanban Provider

Tickets can also be synced from Jira, Linear, or GitHub Projects rather than
authored by hand. See [Supported Kanban Providers](/getting-started/kanban/).

## Ticket Directories

```
.tickets/
├── queue/        # Pending work
├── in-progress/  # Currently being worked
└── completed/    # Finished work
```

## Ticket Lifecycle

1. **Created** - Ticket added to `queue/`
2. **Assigned** - Moved to `in-progress/` when an agent starts
3. **Completed** - Moved to `completed/` when done

## Best Practices

- **Be specific** - Clear requirements help agents succeed
- **Include context** - Link to related issues, PRs, or docs
- **Define done** - Clear acceptance criteria
- **Right-size** - Keep tickets focused and achievable

---
title: Operator!
description: "Operator! is an application for orchestrating LLM coding assist agents across multi-repository codebases with kanban-style ticket management."
layout: doc
---

Welcome friend! <span class="operator-brand">Operator!</span> is an application for orchestrating [Claude Code](/getting-started/agents/claude/) (and other [LLM coding agents](/getting-started/agents/)) across multi-repository codebases. It connects to [kanban-style ticket management software](/getting-started/kanban/) , and can spawn session from the [**VS Code** extension](/getting-started/sessions/vscode/) or from [`tmux` terminals](/getting-started/sessions/tmux/) to create a powerful workflow for managing AI-assisted software development.

<span class="operator-brand">Operator!</span> was designed _by Software Engineers for Software Engineers_. Most software development happens [multi-repo rather than mono-repo](https://www.thoughtworks.com/en-us/insights/blog/agile-engineering-practices/monorepo-vs-multirepo){:target="_blank"}, and succeeding with AI software development requires coordinating LLM assist coding agents work across many codebases, with modern feature development requiring 2+ pull requests across an organization. The API server runs in the directory containing your work code repositories, where it can synchronize and direct markdown defined work orders under a `.tickets/` directory which stores your kanban synchronized work tickets.

## What is <span class="operator-brand">Operator!</span>?

<span class="operator-brand">Operator!</span> helps you:

- **Manage ticket queues** - Organize and prioritize ticket shaped work into reproducible workflows.
- **Launch LLM agents** - Start Claude Code sessions with context from tickets and your teams established software development standards and practices.
- **Track progress** - Monitor agent status and work completion in real-time, keeping you in focus of what needs to get done, starting work in sessions.
- **Parallelize Work** - Work multiple tickets at once, across many code repositories, using the right models for the job, to get the most of your LLM credits.
- **Enforce Standards** - Define how work gets done by AI tools, across many defined software services. Enforce standards and practices on your AI agents.
- **Catalog your Code** - Get a net overview of your codebase, composed from project files, organized and maintained implicitly by Operator.

## Getting Started

1. Install <span class="operator-brand">Operator!</span> (downloads page)
2. Configure your project management kanban workspaces
3. Define your work shape issuetypes, and how AI combines them together
3. Create tickets in `.tickets/queue/` (kanban connectors)
4. Launch agents and watch them work (measuring success)

## Quick Links

- [Kanban](/kanban/) - Understand the kanban workflow
- [LLM Tools](/llm-tools/) - Configure LLM integration
- [Tickets](/tickets/) - Create and manage work tickets
- [Agents](/agents/) - Agent lifecycle and modes
- [Tmux](/tmux/) - Terminal session management

## Similar, but Worse:

These are tools that are almost as good, and are inspirational, but just don't quite cut it:

- [Ralph Code](https://github.com/frankbria/ralph-claude-code)
- [Vibe Kanban](https://www.vibekanban.com/)
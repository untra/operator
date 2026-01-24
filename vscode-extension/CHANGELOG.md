# Changelog

All notable changes to **Operator! Terminals** VS Code extension will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.16] - 2026-01-24

### Added

- Setup walkthrough contribution with 3-step guide:
  - Select working directory
  - Connect kanban provider (Jira/Linear)
  - Install LLM tool detection (Claude Code/Codex/Gemini CLI)
- New commands: `operator.selectWorkingDirectory`, `operator.checkKanbanConnection`, `operator.configureJira`, `operator.configureLinear`, `operator.detectLlmTools`, `operator.openWalkthrough`
- Walkthrough markdown documentation in `walkthrough/` directory
- Context keys for walkthrough step completion tracking

## [0.1.12] - 2026-01-8

### Added

- Support for wrapping commands in opr8r to multiplex output to the Operator API

## [0.1.10] - 2025-01-10

### Added

- Initial release
- Terminal creation with ticket-type styling (colors and icons)
- Activity detection via shell execution events
- HTTP webhook server for Operator communication
- Commands: Start Server, Stop Server, Show Status
- Configuration options: webhookPort, autoStart, terminalPrefix
- Status bar indicator showing server state
- API endpoints for terminal management:
  - Create, send, show, focus, kill terminals
  - Query terminal existence and activity state
  - List all managed terminals

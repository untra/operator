---
title: "Platform Support & Limitations"
description: "What works on each operating system and which features are unavailable on specific platforms."
layout: doc
---

# Platform Support & Limitations

This page is the authoritative reference for what Operator supports on each operating system. Each gap is tagged as one of: **not applicable** (the underlying tool doesn't exist on that OS), **blocked** (a dependency prevents support and a workaround is needed), or **planned** (the intent is to support it; no timeline committed).

## Quick Reference Matrix

| Feature | macOS | Linux | Windows |
|---------|-------|-------|---------|
| Session manager: VS Code Extension | ✅ | ✅ | ✅ Required |
| Session manager: Cursor | ✅ | ✅ | ✅ |
| Session manager: tmux | ✅ | ✅ | ❌ |
| Session manager: cmux | ✅ | ❌ | ❌ |
| Session manager: Zellij | ✅ | ✅ | ❌ |
| Relay hub (multi-agent) | ✅ | ✅ | ❌ |
| `opr8r relay` subcommand | ✅ | ✅ | ❌ |
| Backstage Server | ✅ | ✅ | ❌ |
| Native OS notifications | ✅ | ✅ | ⚠️ |
| Kanban: Jira Cloud | ✅ | ✅ | ✅ |
| Kanban: Linear | ✅ | ✅ | ✅ |
| Kanban: GitHub Issues | ⚠️ | ⚠️ | ⚠️ |
| Git: GitHub (`gh`) | ✅ | ✅ | ✅ |
| Git: GitLab (`glab`) | ⚠️ | ⚠️ | ⚠️ |
| Agent: Claude Code | ✅ | ✅ | ✅ |
| Agent: Codex | ✅ | ✅ | ✅ |
| Agent: Gemini CLI | ⚠️ | ⚠️ | ⚠️ |

**Legend:** ✅ Fully supported &nbsp;·&nbsp; ❌ Not available &nbsp;·&nbsp; ⚠️ Partial / planned

---

## Windows

Windows is a supported download target. The step-wrapper (`opr8r`), REST API, kanban sync, and VS Code extension all work. The following features are currently unavailable on Windows.

| Feature | Status | Reason | Workaround |
|---------|--------|--------|------------|
| Relay hub / `opr8r relay` | ❌ Blocked | Requires Unix domain sockets (`tokio::net::unix`), which are not available on Windows | None yet. Planned: named-pipe or TCP-loopback transport in a future release |
| Backstage Server | ❌ Blocked | Not yet ported to Windows | None. No timeline committed |
| tmux session manager | ❌ N/A | tmux does not run on Windows | Use the VS Code Extension |
| cmux session manager | ❌ N/A | cmux is macOS-specific | Use the VS Code Extension |
| Zellij session manager | ❌ N/A | Zellij does not run on Windows | Use the VS Code Extension |
| Native OS notifications | ⚠️ Planned | Platform notification crates (`mac-notification-sys`, `notify-rust`) are Unix-only; a Windows crate has not been integrated | Notifications fall back to log output only |

---

## Linux

Linux is a first-class platform. One known gap:

| Feature | Status | Reason | Workaround |
|---------|--------|--------|------------|
| cmux session manager | ❌ N/A | cmux is a macOS-specific terminal multiplexer | Use tmux or the VS Code Extension |

---

## macOS

macOS is the primary development platform. No known feature gaps.

---

## Integration-Level Gaps (all platforms)

These gaps apply on every operating system because the integration itself is not fully implemented.

| Feature | Status | Notes |
|---------|--------|-------|
| Kanban: GitHub Issues | ⚠️ Detection only | GitHub Issues is detected as a provider but full two-way sync (create, update, close) is not implemented. Only Jira Cloud and Linear have full sync. |
| Git: GitLab (`glab`) | ⚠️ Detection only | GitLab is detected via the `glab` CLI for branch and PR metadata, but PR creation and status webhooks are not implemented. |
| Git: Bitbucket, Azure DevOps | ⚠️ Detection only | Detected via their respective CLIs; no PR workflow integration. |
| Agent: Gemini CLI | ⚠️ Experimental | Session detection and artifact parsing are less battle-tested than Claude Code. Some multi-step issue-type flows may behave unexpectedly. |

---

## Reporting a Gap

If you hit a limitation not listed here, please [open an issue on GitHub](https://github.com/untra/operator/issues){:target="_blank"}.

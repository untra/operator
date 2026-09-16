---
title: "IDEs"
description: "Editor integrations and their compatible session controllers."
layout: doc
---

IDE integrations connect an editor to Operator. Session controllers manage agent terminals; select a controller implemented for the environment where those terminals run.

| IDE | Status | Session control |
|---|---|---|
| [VS Code](/getting-started/ides/vscode/) | Beta | [VS Code Terminals](/getting-started/sessions/vscode-terminals/), serialized as `vscode` |
| [Zed](/getting-started/ides/zed/) | Alpha | MCP/ACP integration; no IDE terminal controller |

cmux, tmux, and Zellij cannot control VS Code's integrated terminals. See [Session Management](/getting-started/sessions/) for their independent terminal environments.

---
title: "VS Code Terminals"
description: "Manage agent terminals through the Operator VS Code extension."
layout: doc
---

The `vscode` session controller creates and monitors integrated terminals through the [Operator VS Code extension](/getting-started/ides/vscode/). It is available on macOS, Linux, and Windows.

```toml
[sessions]
wrapper = "vscode"
```

Use this controller for VS Code terminals. cmux, tmux, and Zellij control their own environments and cannot be substituted for this controller inside VS Code.

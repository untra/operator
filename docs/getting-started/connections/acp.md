---
title: "ACP"
description: "Generate local editor configuration that launches Operator as an ACP agent."
layout: doc
---

The **ACP** card generates Agent Client Protocol configuration for Zed, JetBrains, Emacs, or Kiro.

The editor launches a local Operator process over stdio (`operator acp`). Install Operator on the editor's machine and set up a local workspace and profile first. Then open **Configure local ACP editor**, choose the editor, set the local Operator executable and an optional profile, and copy or download the recipe.

These recipes never connect ACP to a hosted Operator, and the hosted page can't observe local ACP sessions, so this card's dot stays gray.

See [Zed](/getting-started/ides/zed/) for the editor-side setup.

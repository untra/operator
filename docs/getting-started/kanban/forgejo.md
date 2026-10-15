---
title: "Forgejo Projects"
description: "Forgejo board API compatibility and planned kanban support."
layout: doc
---

Forgejo Projects is **Proto**, with no import or write-back capability. It is defined separately from Gitea Projects and is not offered as an operational connection. Enabling `[kanban.forgejo.<instance>]` returns an unsupported-board-API error.

Forgejo has built-in project boards, but Operator has not verified a released API for reading their columns and membership. The inspected upstream development OpenAPI schema (`17.0.0-dev-609-a14eb80977`) has no project, board, or column endpoints. Operator does not scrape web pages or treat issue labels as project columns. Check the target release's `/swagger.v1.json` before proposing Alpha support.

The reserved configuration uses `base_url`, `api_key_env`, `repositories`, and `projects`, with `enabled = false` and `OPERATOR_FORGEJO_TOKEN` as the default credential variable. Do not create a board token yet: required project API permissions cannot be established until that API exists. Issue/repository scopes alone do not grant a missing board capability.

[Forgejo Git](/getting-started/git/forgejo/) is independently Alpha and uses `FORGEJO_TOKEN`. Its permissions and Tea transport do not imply compatibility with [Gitea project APIs](/getting-started/kanban/gitea/).

Promotion requires a released API contract, documented least-privilege token scopes, successful repository-board discovery/import tests, and catalog capability parity.

---
title: "Codex"
description: "Configure OpenAI Codex as your AI coding agent."
layout: doc
---

# Codex

[Codex](https://developers.openai.com/codex/) is the [OpenAI](https://openai.com/) code-specialized CLI agent, available through the OpenAI API.

## Status

Codex integration is currently **experimental**. Features may be limited compared to other agents.

## Installation

Install the OpenAI CLI:

```bash
npm i -g @openai/codex
```

### Plans and Pricing

View [OpenAI Codex pricing page](https://developers.openai.com/codex/pricing/)

## Configuration

See the full [Codex agent configuration reference](/configuration/#agents-codex).

Add Codex to your Operator configuration:

```toml
# ~/.config/operator/config.toml

[agents.codex]
enabled = true
api_key_env = "OPENAI_API_KEY"
model = "gpt-4"
```

## Authentication

Set your OpenAI API key:

```bash
export OPENAI_API_KEY="your-api-key"
```

Or add it to your shell profile for persistence.

## API Usage

Codex uses the OpenAI API which has usage-based pricing. Monitor your usage at [platform.openai.com](https://platform.openai.com/).

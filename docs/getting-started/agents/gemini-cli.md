---
title: "Gemini"
description: "Configure Google Gemini as your AI coding agent."
layout: doc
---

# Gemini CLI

[Gemini](https://geminicli.com/) is [Google](https://google.com)'s multimodal agent CLI with strong coding capabilities.

## Status

Gemini integration is currently **experimental**. Features may be limited compared to other agents.

## Installation

Install the Google AI SDK:

```bash
pip install google-generativeai
```

### Plans and Pricing



## Configuration

See the full [Gemini agent configuration reference](/configuration/#agents-gemini).

Add Gemini to your Operator configuration:

```toml
# ~/.config/operator/config.toml

[agents.gemini]
enabled = true
api_key_env = "GOOGLE_AI_API_KEY"
model = "gemini-pro"
```

## Authentication

Set your Google AI API key:

```bash
export GOOGLE_AI_API_KEY="your-api-key"
```

Get an API key from [Google AI Studio](https://makersuite.google.com/).

## Features

Gemini provides:

- Code generation and completion
- Multi-language support
- Code explanation
- Documentation generation

## Limitations

Current experimental limitations:

- Limited context window compared to Claude
- May require more specific prompting
- Some Operator features may not be fully supported

## Operator Integration

When Operator assigns a ticket to Gemini:

1. Gemini receives the ticket context
2. Generates code implementations
3. Applies changes to the codebase
4. Reports completion status

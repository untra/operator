# Detect & Configure LLM Tools

Operator launches LLM coding agents to work on tickets. You need at least one of these tools installed:

## Claude Code (Recommended)

The most capable coding assistant from Anthropic.

```bash
npm install -g @anthropic-ai/claude-code
```

Then authenticate with `claude login`.

[Documentation](https://docs.anthropic.com/en/docs/claude-code)

## OpenAI Codex

OpenAI's code generation model.

```bash
npm install -g codex
```

[Documentation](https://github.com/openai/codex)

## Gemini CLI

Google's Gemini model for code tasks.

```bash
npm install -g @google/generative-ai-cli
```

[Documentation](https://ai.google.dev/)

## Detection & Configuration

Select **Detect Tools** to scan your PATH and choose the preferred tool for the connected Operator configuration. The running server saves the preference. To select a particular model, expand the tool in **LLM Tools** and select a model.

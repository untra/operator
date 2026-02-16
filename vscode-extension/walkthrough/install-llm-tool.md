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

Click **Detect Tools** above to scan your PATH for installed tools. If tools are found, you can configure each one individually. Configuration delegates to `operator setup --llm-tool <tool>` which writes version detection, model aliases, and command templates to your config.

Operator will use whichever tool is available in your PATH.

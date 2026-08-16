//! LLM Tools documentation generator.
//!
//! Generates documentation for adding and configuring LLM CLI tools
//! from the `tool_config.schema.json` and existing tool configurations.

use anyhow::Result;

use super::{format_header, DocGenerator};

/// Generator for LLM tools documentation
pub struct LlmToolsDocGenerator;

impl DocGenerator for LlmToolsDocGenerator {
    fn name(&self) -> &'static str {
        "llm-tools"
    }

    fn source(&self) -> &'static str {
        "src/llm/tools/tool_config.schema.json"
    }

    fn output_path(&self) -> &'static str {
        "llm-tools/index.md"
    }

    fn generate(&self) -> Result<String> {
        let mut content = format_header("LLM Tools Configuration", self.source());

        content.push_str(
            r#"# LLM Tools Configuration

Operator supports multiple LLM CLI tools through a plugin-like configuration system. Each tool is defined by a JSON configuration file that tells Operator how to detect, invoke, and manage the tool.

## Supported Tools

| Tool | Binary | Display Name | Models |
|------|--------|--------------|--------|
| Claude Code | `claude` | Claude Code | opus, sonnet, haiku |
| Google Gemini | `gemini` | Google Gemini | pro, flash, ultra |
| OpenAI Codex | `codex` | OpenAI Codex | gpt-4o, o1, o3 |

## Adding a New Tool

To add support for a new LLM CLI tool, drop a JSON configuration file into your
user tool-config directory — no rebuild required:

- Linux: `~/.config/operator/tools/<tool_name>.json`
- macOS: `~/Library/Application Support/operator/tools/<tool_name>.json`

```json
{
  "tool_name": "your-tool",
  "display_name": "Your Tool Name",
  "version_command": "your-tool --version",
  "capabilities": {
    "supports_sessions": true,
    "supports_headless": false
  },
  "model_aliases": ["model1", "model2"],
  "arg_mapping": {
    "model": "--model",
    "session_id": "--session",
    "prompt": "-p"
  },
  "command_template": "your-tool {{model_flag}}--session {{session_id}} \"$(cat {{prompt_file}})\"",
  "yolo_flags": ["--auto-approve"]
}
```

Configs are loaded fresh on every startup. A user config whose `tool_name` matches a builtin (claude, gemini, codex) **fully replaces** that builtin — it
is not merged field-by-field. Malformed files are skipped with a logged warning. Runtime-loaded tools work everywhere the builtins do, including
remote (SSH) launches, where the tool's presence on the remote host is verified by a `command -v` preflight.

> **Security note:** `command_template` is arbitrary shell executed at launch.
> Operator only ever loads tool configs from the user-global config directory —
> never from repository-local paths — so a cloned repo cannot inject a tool
> config.

New *builtin* tools (shipped with Operator) are instead added as embedded JSONs
in `src/llm/tools/` and registered in the `BUILTIN_TOOL_CONFIGS` list in
`src/llm/tool_config.rs`.

## Detection Modes

By default a tool is detected only when `which <tool_name>` succeeds. The
optional `detection` object overrides this:

```json
{
  "detection": { "mode": "always", "health_command": "your-tool ping" }
}
```

| Field | Values | Description |
|-------|--------|-------------|
| `mode` | `which` (default), `always` | `always` skips the PATH lookup and uses `tool_name` verbatim as the invocation path — for tools not installed locally (e.g. run over SSH) |
| `health_command` | any command | Health check run at every startup; failure marks the tool unhealthy (`health_ok: false`) |

Health is **earned, never assumed**, and re-verified on every startup:

| Mode | No `health_command` | With `health_command` |
|------|---------------------|-----------------------|
| `which` | Healthy — the PATH lookup proves the binary is present | Healthy if still on PATH **and** the command passes |
| `always` | **Unhealthy** — nothing is locally verifiable | Healthy if the command passes |

An unhealthy tool stays listed in the detected tools (so you can see it and why),
but launching a local agent with it fails until it is healthy again. Remote (SSH)
launches are unaffected — they are gated by their own `command -v` preflight on
the remote host. An `always`-mode tool should therefore define a `health_command`
that proves reachability, e.g. `ssh gpu-vm command -v agy`.

## Configuration Schema

### Required Fields

| Field | Type | Description |
|-------|------|-------------|
| `tool_name` | string | Binary/command name (must match executable in PATH) |
| `version_command` | string | Command to check if tool is installed |
| `capabilities` | object | Feature flags for the tool |
| `model_aliases` | array | List of supported model names |
| `arg_mapping` | object | Maps logical args to CLI flags |
| `command_template` | string | Template for building commands |

### Optional Fields

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `display_name` | string | tool_name | Human-readable name for UI |
| `yolo_flags` | array | [] | Flags for auto-accept/YOLO mode |
| `detection` | object | which-gated | Detection mode + soft health check (see Detection Modes) |
| `idle_detection` | object | - | Idle/activity regex patterns + completion hook config |
| `permission_modes` | array | - | Supported permission modes (Claude-specific) |

### Capabilities Object

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `supports_sessions` | boolean | required | Session continuity via ID |
| `supports_headless` | boolean | false | Non-interactive mode support |
| `supports_config_override` | boolean | false | Runtime config overrides |
| `supports_permission_mode` | boolean | false | Permission modes (Claude) |
| `supports_json_schema` | boolean | false | Structured output via JSON schema |

### Argument Mapping

| Key | Description | Example |
|-----|-------------|---------|
| `model` | Model selection flag | `--model`, `-m` |
| `session_id` | Session continuity flag | `--session-id`, `--resume` |
| `prompt` | Prompt/instruction flag | `-p`, `--prompt` |
| `quiet` | Non-interactive output flag | `-q`, `--output-format json` |
| `permission_mode` | Permission mode flag (Claude) | `--permission-mode` |
| `json_schema` | JSON schema flag | `--json-schema` |

### Command Template Placeholders

| Placeholder | Description |
|-------------|-------------|
| `{{model}}` | The selected model name |
| `{{model_flag}}` | Full model flag with value (e.g., `--model opus `) |
| `{{session_id}}` | Session UUID for continuity |
| `{{prompt_file}}` | Path to the prompt file |
| `{{config_flags}}` | Generated permission/config flags |

## YOLO Mode Flags

YOLO (auto-accept) mode enables fully autonomous execution by bypassing confirmation prompts. Each tool defines its own flags:

| Tool | YOLO Flags | Effect |
|------|------------|--------|
| Claude | `--dangerously-skip-permissions` | Skips all permission prompts |
| Gemini | `--auto-approve`, `-y` | Auto-approves all actions |
| Codex | `--full-auto` | Enables full automation |

## Example: Full Configuration

Here's a complete example for Claude Code:

```json
{
  "tool_name": "claude",
  "display_name": "Claude Code",
  "version_command": "claude --version",
  "capabilities": {
    "supports_sessions": true,
    "supports_headless": false,
    "supports_config_override": true,
    "supports_permission_mode": true,
    "supports_json_schema": true
  },
  "model_aliases": ["opus", "sonnet", "haiku"],
  "arg_mapping": {
    "prompt": "-p",
    "model": "--model",
    "session_id": "--session-id",
    "permission_mode": "--permission-mode",
    "json_schema": "--json-schema"
  },
  "permission_modes": ["default", "plan", "acceptEdits", "delegate"],
  "command_template": "claude {{config_flags}}{{model_flag}}--session-id {{session_id}} \"$(cat {{prompt_file}})\"",
  "yolo_flags": ["--dangerously-skip-permissions"]
}
```

## Visual Indicators

In the TUI, running agents show a tool indicator:

| Indicator | Tool | Color |
|-----------|------|-------|
| **A** | Claude/Anthropic | Rust (#C15F3C) |
| **G** | Gemini | Purple (#6F42C1) |
| **O** | Codex/OpenAI | Green |

## Detection Process

On every startup, Operator:

1. Loads the embedded builtin tool configurations, then user configurations from `<config dir>/operator/tools/*.json`
2. For each tool, runs `which <tool_name>` to check if installed (skipped when `detection.mode` is `always`)
3. If found, runs the `version_command` to get the version (failure degrades to `"unknown"`; a `min_version` mismatch warns but does not block)
4. Computes health from the verified presence plus the `health_command`, if configured (see Detection Modes); an unhealthy tool stays listed but cannot launch locally
5. Builds a list of available providers (tool + model combinations)
6. The first detected tool becomes the default provider

Already-detected tools keep their cached `path`/`version` across restarts (no
version re-probing); config-sourced fields like the command template and model
aliases are re-derived from the loaded configs each startup. Health is never
carried over from a previous run — presence and the `health_command` are
re-checked every startup, so an uninstalled binary or a newly failing health
command demotes the tool on the next launch of Operator.

## Troubleshooting

### Tool Not Detected

1. Ensure the binary is in your PATH: `which <tool_name>`
2. Verify the version command works: `<tool_name> --version`
3. Check Operator logs for detection errors

### Command Fails

1. Test the command manually with the template filled in
2. Verify all argument mappings are correct for your tool version
3. Check if the tool requires additional environment variables
"#,
        );

        Ok(content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_llm_tools_generator_name() {
        let gen = LlmToolsDocGenerator;
        assert_eq!(gen.name(), "llm-tools");
    }

    #[test]
    fn test_llm_tools_generator_source() {
        let gen = LlmToolsDocGenerator;
        assert!(gen.source().contains("tool_config.schema.json"));
    }

    #[test]
    fn test_llm_tools_generator_output() {
        let gen = LlmToolsDocGenerator;
        assert_eq!(gen.output_path(), "llm-tools/index.md");
    }

    #[test]
    fn test_llm_tools_generator_content() {
        let gen = LlmToolsDocGenerator;
        let content = gen.generate().unwrap();
        assert!(content.contains("LLM Tools Configuration"));
        assert!(content.contains("Claude Code"));
        assert!(content.contains("tool_name"));
        assert!(content.contains("yolo_flags"));
    }
}

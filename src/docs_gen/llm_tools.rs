//! LLM Tools documentation generator.
//!
//! Generates documentation for adding and configuring LLM CLI tools
//! from the tool_config.schema.json and existing tool configurations.

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

To add support for a new LLM CLI tool, create a JSON configuration file in `src/llm/tools/`:

### 1. Create the Configuration File

Create `src/llm/tools/<tool_name>.json`:

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

### 2. Register the Tool

Add the tool to `src/llm/tool_config.rs` in the `load_all_tool_configs()` function:

```rust
// Load YourTool config
if let Ok(config) = serde_json::from_str::<ToolConfig>(include_str!("tools/your-tool.json")) {
    configs.push(config);
}
```

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

On startup, Operator:

1. Loads all tool configurations from `src/llm/tools/*.json`
2. For each tool, runs `which <tool_name>` to check if installed
3. If found, runs the `version_command` to verify and get version
4. Builds a list of available providers (tool + model combinations)
5. The first detected tool becomes the default provider

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

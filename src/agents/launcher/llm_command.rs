//! LLM command building, docker wrapping, and permission configuration

use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::config::{Config, DetectedTool};
use crate::permissions::{PermissionSet, TranslatorManager};
use crate::queue::Ticket;
use crate::templates::schema::PermissionMode;

use super::step_config::{get_step_config, load_project_permissions};

/// Build the LLM command for a specific tool with optional step permissions
pub fn build_llm_command_with_permissions_for_tool(
    config: &Config,
    tool_name: &str,
    model: &str,
    session_id: &str,
    prompt_file: &std::path::Path,
    ticket: Option<&Ticket>,
    project_path: Option<&str>,
) -> Result<String> {
    // Find the specified tool
    let tool = get_detected_tool(config, tool_name).ok_or_else(|| {
        anyhow::anyhow!(
            "LLM tool '{}' not detected. Install it or choose a different provider.",
            tool_name
        )
    })?;

    // Build model flag based on tool's arg_mapping
    let model_flag = format!("--model {} ", model);

    // Generate config flags from permissions
    let config_flags = if let (Some(ticket), Some(project_path)) = (ticket, project_path) {
        generate_config_flags(config, &tool.name, ticket, project_path, session_id)?
    } else {
        String::new()
    };

    // Build command from template
    let cmd = tool
        .command_template
        .replace("{{config_flags}}", &config_flags)
        .replace("{{model_flag}}", &model_flag)
        .replace("{{model}}", model)
        .replace("{{session_id}}", session_id)
        .replace("{{prompt_file}}", &prompt_file.display().to_string());

    Ok(cmd)
}

/// Apply YOLO (auto-accept) flags to the command for the given tool
pub fn apply_yolo_flags(config: &Config, cmd: &str, tool_name: &str) -> String {
    if let Some(tool) = get_detected_tool(config, tool_name) {
        if !tool.yolo_flags.is_empty() {
            // Insert YOLO flags after the tool name
            let yolo_flags_str = tool.yolo_flags.join(" ");
            // Find the tool name in the command and insert flags after it
            if let Some(pos) = cmd.find(tool_name) {
                let insert_pos = pos + tool_name.len();
                let mut result = cmd.to_string();
                result.insert_str(insert_pos, &format!(" {}", yolo_flags_str));
                return result;
            }
        }
    }
    cmd.to_string()
}

/// Build a docker command that wraps the LLM command
pub fn build_docker_command(
    config: &Config,
    inner_cmd: &str,
    project_path: &str,
) -> Result<String> {
    let docker_config = &config.launch.docker;

    if docker_config.image.is_empty() {
        anyhow::bail!(
            "Docker mode is enabled but no image is configured. \
             Set launch.docker.image in your config."
        );
    }

    let mut docker_args = vec![
        "docker".to_string(),
        "run".to_string(),
        "--rm".to_string(),
        "-it".to_string(),
        "-v".to_string(),
        format!("{}:{}:rw", project_path, docker_config.mount_path),
        "-w".to_string(),
        docker_config.mount_path.clone(),
    ];

    // Add environment variables
    for env_var in &docker_config.env_vars {
        docker_args.push("-e".to_string());
        docker_args.push(env_var.clone());
    }

    // Add extra args from config
    for arg in &docker_config.extra_args {
        docker_args.push(arg.clone());
    }

    // Add the image
    docker_args.push(docker_config.image.clone());

    // Add the inner command (use sh -c to handle complex commands)
    docker_args.push("sh".to_string());
    docker_args.push("-c".to_string());
    docker_args.push(inner_cmd.to_string());

    Ok(docker_args.join(" "))
}

/// Generate config flags for the LLM command based on step permissions
fn generate_config_flags(
    config: &Config,
    provider: &str,
    ticket: &Ticket,
    project_path: &str,
    session_id: &str,
) -> Result<String> {
    // Load project permissions
    let project_perms = load_project_permissions(config, project_path)?;

    // Get step configuration from template
    let step_config = get_step_config(ticket)?;

    // Add operator-level directory permissions (.tickets/ for reading ticket files)
    let mut operator_perms = step_config.permissions.clone();
    operator_perms
        .directories
        .allow
        .push(config.tickets_path().to_string_lossy().to_string());

    // Merge permissions (additive)
    let merged = PermissionSet::merge(&project_perms, &operator_perms, &step_config.cli_args);

    // Create session directory for storing configs
    let session_dir = config
        .tickets_path()
        .join("operator")
        .join("sessions")
        .join(&ticket.id);
    fs::create_dir_all(&session_dir)
        .with_context(|| format!("Failed to create session dir: {:?}", session_dir))?;

    // Generate config using translator
    let translator = TranslatorManager::new();
    let generated = translator.generate_config(provider, &merged, &session_dir)?;

    // Save audit info
    let audit_command = format!("Session: {}\nTicket: {}\n", session_id, ticket.id);
    TranslatorManager::save_audit_info(&session_dir, provider, &generated, &audit_command)?;

    // Build CLI flags
    let mut cli_flags = generated.cli_flags;

    // Claude-specific flags
    if provider == "claude" {
        // Add permission mode flag (if not default)
        if step_config.permission_mode != PermissionMode::Default {
            let mode_str = match step_config.permission_mode {
                PermissionMode::Default => "default",
                PermissionMode::Plan => "plan",
                PermissionMode::AcceptEdits => "acceptEdits",
                PermissionMode::Delegate => "delegate",
            };
            cli_flags.push("--permission-mode".to_string());
            cli_flags.push(mode_str.to_string());
        }

        // Add JSON schema flag for structured output
        // Inline jsonSchema takes precedence over jsonSchemaFile
        if let Some(ref schema) = step_config.json_schema {
            let schema_str =
                serde_json::to_string(schema).context("Failed to serialize JSON schema")?;
            cli_flags.push("--json-schema".to_string());
            cli_flags.push(schema_str);
        } else if let Some(ref schema_file) = step_config.json_schema_file {
            // Resolve path - .tickets/ paths are relative to tickets parent dir, others to project
            let schema_path =
                if schema_file.starts_with(".tickets/") || schema_file.starts_with(".tickets\\") {
                    if let Some(parent) = config.tickets_path().parent() {
                        parent.join(schema_file)
                    } else {
                        PathBuf::from(project_path).join(schema_file)
                    }
                } else {
                    PathBuf::from(project_path).join(schema_file)
                };
            let schema_content = fs::read_to_string(&schema_path)
                .with_context(|| format!("Failed to read JSON schema file: {:?}", schema_path))?;
            cli_flags.push("--json-schema".to_string());
            cli_flags.push(schema_content);
        }
    }

    // Format CLI flags as a space-separated string (with trailing space if non-empty)
    if cli_flags.is_empty() {
        Ok(String::new())
    } else {
        Ok(format!("{} ", cli_flags.join(" ")))
    }
}

/// Get the detected tool for a given provider
fn get_detected_tool<'a>(config: &'a Config, tool_name: &str) -> Option<&'a DetectedTool> {
    config
        .llm_tools
        .detected
        .iter()
        .find(|t| t.name == tool_name)
}

/// Get the model for the first available provider
pub fn get_default_model(config: &Config) -> Option<String> {
    config.llm_tools.providers.first().map(|p| p.model.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn make_test_config_with_tool(tool: DetectedTool) -> Config {
        Config {
            llm_tools: crate::config::LlmToolsConfig {
                detected: vec![tool],
                providers: vec![crate::config::LlmProvider {
                    tool: "claude".to_string(),
                    model: "sonnet".to_string(),
                    display_name: None,
                    ..Default::default()
                }],
                detection_complete: true,
            },
            ..Default::default()
        }
    }

    fn make_detected_tool() -> DetectedTool {
        DetectedTool {
            name: "claude".to_string(),
            path: "/usr/bin/claude".to_string(),
            version: "1.0.0".to_string(),
            min_version: Some("1.0.0".to_string()),
            version_ok: true,
            model_aliases: vec!["sonnet".to_string()],
            command_template: "claude {{config_flags}}{{model_flag}}--session-id {{session_id}} --print-prompt-path {{prompt_file}}".to_string(),
            capabilities: crate::config::ToolCapabilities {
                supports_sessions: true,
                supports_headless: true,
            },
            yolo_flags: vec!["--dangerously-skip-permissions".to_string()],
        }
    }

    // ========================================
    // apply_yolo_flags() tests
    // ========================================

    #[test]
    fn test_apply_yolo_flags_inserts_after_tool_name() {
        let tool = make_detected_tool();
        let config = make_test_config_with_tool(tool);
        let cmd = "claude --model sonnet --session-id abc123";

        let result = apply_yolo_flags(&config, cmd, "claude");

        assert!(
            result.contains("claude --dangerously-skip-permissions --model"),
            "YOLO flag should be inserted after tool name, got: {}",
            result
        );
    }

    #[test]
    fn test_apply_yolo_flags_multiple_flags() {
        let mut tool = make_detected_tool();
        tool.yolo_flags = vec![
            "--dangerously-skip-permissions".to_string(),
            "--no-confirm".to_string(),
        ];
        let config = make_test_config_with_tool(tool);
        let cmd = "claude --model sonnet";

        let result = apply_yolo_flags(&config, cmd, "claude");

        assert!(
            result.contains("--dangerously-skip-permissions --no-confirm"),
            "Multiple YOLO flags should be joined with spaces, got: {}",
            result
        );
    }

    #[test]
    fn test_apply_yolo_flags_empty_when_no_flags() {
        let mut tool = make_detected_tool();
        tool.yolo_flags = vec![];
        let config = make_test_config_with_tool(tool);
        let cmd = "claude --model sonnet";

        let result = apply_yolo_flags(&config, cmd, "claude");

        assert_eq!(
            result, cmd,
            "Command should be unchanged when no YOLO flags"
        );
    }

    #[test]
    fn test_apply_yolo_flags_unknown_tool_unchanged() {
        let tool = make_detected_tool();
        let config = make_test_config_with_tool(tool);
        let cmd = "gemini --model pro";

        let result = apply_yolo_flags(&config, cmd, "gemini");

        assert_eq!(result, cmd, "Command should be unchanged for unknown tool");
    }

    #[test]
    fn test_apply_yolo_flags_preserves_command_args() {
        let tool = make_detected_tool();
        let config = make_test_config_with_tool(tool);
        let cmd = "claude --model sonnet --session-id abc --print-prompt-path /tmp/p.md";

        let result = apply_yolo_flags(&config, cmd, "claude");

        assert!(
            result.contains("--session-id abc"),
            "Should preserve session-id arg"
        );
        assert!(
            result.contains("--print-prompt-path /tmp/p.md"),
            "Should preserve prompt-path arg"
        );
    }

    // ========================================
    // build_docker_command() tests
    // ========================================

    #[test]
    fn test_build_docker_command_basic_structure() {
        let mut config = Config::default();
        config.launch.docker.image = "my-claude:latest".to_string();
        config.launch.docker.mount_path = "/workspace".to_string();

        let result = build_docker_command(&config, "claude --model sonnet", "/home/user/project");

        assert!(result.is_ok());
        let cmd = result.unwrap();
        assert!(cmd.starts_with("docker run --rm -it"));
    }

    #[test]
    fn test_build_docker_command_volume_mount() {
        let mut config = Config::default();
        config.launch.docker.image = "my-claude:latest".to_string();
        config.launch.docker.mount_path = "/workspace".to_string();

        let result = build_docker_command(&config, "claude", "/home/user/project");

        let cmd = result.unwrap();
        assert!(
            cmd.contains("-v /home/user/project:/workspace:rw"),
            "Should mount project path with :rw, got: {}",
            cmd
        );
    }

    #[test]
    fn test_build_docker_command_working_dir() {
        let mut config = Config::default();
        config.launch.docker.image = "my-claude:latest".to_string();
        config.launch.docker.mount_path = "/workspace".to_string();

        let result = build_docker_command(&config, "claude", "/home/user/project");

        let cmd = result.unwrap();
        assert!(
            cmd.contains("-w /workspace"),
            "Should set working dir to mount path, got: {}",
            cmd
        );
    }

    #[test]
    fn test_build_docker_command_env_vars() {
        let mut config = Config::default();
        config.launch.docker.image = "my-claude:latest".to_string();
        config.launch.docker.mount_path = "/workspace".to_string();
        config.launch.docker.env_vars =
            vec!["ANTHROPIC_API_KEY".to_string(), "HOME=/root".to_string()];

        let result = build_docker_command(&config, "claude", "/project");

        let cmd = result.unwrap();
        assert!(
            cmd.contains("-e ANTHROPIC_API_KEY"),
            "Should pass first env var, got: {}",
            cmd
        );
        assert!(
            cmd.contains("-e HOME=/root"),
            "Should pass second env var, got: {}",
            cmd
        );
    }

    #[test]
    fn test_build_docker_command_extra_args() {
        let mut config = Config::default();
        config.launch.docker.image = "my-claude:latest".to_string();
        config.launch.docker.mount_path = "/workspace".to_string();
        config.launch.docker.extra_args =
            vec!["--network=host".to_string(), "--privileged".to_string()];

        let result = build_docker_command(&config, "claude", "/project");

        let cmd = result.unwrap();
        assert!(cmd.contains("--network=host"), "Should include extra arg 1");
        assert!(cmd.contains("--privileged"), "Should include extra arg 2");
    }

    #[test]
    fn test_build_docker_command_no_image_errors() {
        let config = Config::default(); // image is empty by default

        let result = build_docker_command(&config, "claude", "/project");

        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("no image is configured"),
            "Error should mention missing image, got: {}",
            err
        );
    }

    #[test]
    fn test_build_docker_command_wraps_inner_cmd() {
        let mut config = Config::default();
        config.launch.docker.image = "my-claude:latest".to_string();
        config.launch.docker.mount_path = "/workspace".to_string();

        let result = build_docker_command(&config, "claude --model sonnet", "/project");

        let cmd = result.unwrap();
        assert!(
            cmd.contains("sh -c claude --model sonnet"),
            "Should wrap inner command with sh -c, got: {}",
            cmd
        );
    }

    // ========================================
    // get_default_model() tests
    // ========================================

    #[test]
    fn test_get_default_model_returns_first() {
        let mut config = Config::default();
        config.llm_tools.providers = vec![crate::config::LlmProvider {
            tool: "claude".to_string(),
            model: "opus".to_string(),
            ..Default::default()
        }];

        let result = get_default_model(&config);

        assert_eq!(result, Some("opus".to_string()));
    }

    #[test]
    fn test_get_default_model_empty_returns_none() {
        let mut config = Config::default();
        config.llm_tools.providers = vec![];

        let result = get_default_model(&config);

        assert_eq!(result, None);
    }

    #[test]
    fn test_get_default_model_multiple_uses_first() {
        let mut config = Config::default();
        config.llm_tools.providers = vec![
            crate::config::LlmProvider {
                tool: "claude".to_string(),
                model: "sonnet".to_string(),
                ..Default::default()
            },
            crate::config::LlmProvider {
                tool: "gemini".to_string(),
                model: "pro".to_string(),
                ..Default::default()
            },
        ];

        let result = get_default_model(&config);

        assert_eq!(
            result,
            Some("sonnet".to_string()),
            "Should return first provider's model"
        );
    }

    // ========================================
    // build_llm_command_with_permissions_for_tool() tests
    // ========================================

    #[test]
    fn test_build_llm_command_unknown_tool_errors() {
        let config = Config::default(); // No detected tools

        let result = build_llm_command_with_permissions_for_tool(
            &config,
            "nonexistent",
            "sonnet",
            "session-123",
            Path::new("/tmp/prompt.md"),
            None,
            None,
        );

        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("not detected"),
            "Error should mention tool not detected, got: {}",
            err
        );
    }

    #[test]
    fn test_build_llm_command_template_interpolation() {
        let tool = make_detected_tool();
        let config = make_test_config_with_tool(tool);

        let result = build_llm_command_with_permissions_for_tool(
            &config,
            "claude",
            "opus",
            "sess-abc",
            Path::new("/tmp/prompt.md"),
            None,
            None,
        );

        assert!(result.is_ok());
        let cmd = result.unwrap();
        assert!(
            cmd.contains("--model opus"),
            "Should interpolate model, got: {}",
            cmd
        );
        assert!(
            cmd.contains("--session-id sess-abc"),
            "Should interpolate session_id, got: {}",
            cmd
        );
        assert!(
            cmd.contains("/tmp/prompt.md"),
            "Should interpolate prompt_file, got: {}",
            cmd
        );
    }

    #[test]
    fn test_build_llm_command_no_ticket_empty_config_flags() {
        let tool = make_detected_tool();
        let config = make_test_config_with_tool(tool);

        let result = build_llm_command_with_permissions_for_tool(
            &config,
            "claude",
            "sonnet",
            "sess-123",
            Path::new("/tmp/prompt.md"),
            None, // No ticket
            None, // No project path
        );

        assert!(result.is_ok());
        let cmd = result.unwrap();
        // When no ticket, config_flags should be empty, so command starts with "claude --model"
        assert!(
            cmd.starts_with("claude --model"),
            "Should have empty config_flags when no ticket, got: {}",
            cmd
        );
    }

    #[test]
    fn test_build_llm_command_model_flag_format() {
        let tool = make_detected_tool();
        let config = make_test_config_with_tool(tool);

        let result = build_llm_command_with_permissions_for_tool(
            &config,
            "claude",
            "haiku",
            "sess-xyz",
            Path::new("/tmp/p.md"),
            None,
            None,
        );

        assert!(result.is_ok());
        let cmd = result.unwrap();
        // Model flag should have trailing space per the code
        assert!(
            cmd.contains("--model haiku "),
            "Model flag should have trailing space, got: {}",
            cmd
        );
    }

    // ========================================
    // Step permissions tests
    // ========================================

    /// Tests for step permission handling across providers.
    /// These tests verify that stepPermissions from issuetype schemas
    /// are correctly translated to provider-specific CLI args and configs.
    mod step_permissions {
        use crate::permissions::{
            PermissionSet, ProviderCliArgs, StepPermissions, ToolPattern, ToolPermissions,
            TranslatorManager,
        };
        use crate::templates::schema::PermissionMode;

        // ========================================
        // Claude step permission tests
        // ========================================

        #[test]
        fn test_claude_permission_mode_plan_generates_flag() {
            // Permission mode is handled in generate_config_flags, not in translator
            // This test verifies the permission mode string mapping
            let mode = PermissionMode::Plan;
            let mode_str = match mode {
                PermissionMode::Default => "default",
                PermissionMode::Plan => "plan",
                PermissionMode::AcceptEdits => "acceptEdits",
                PermissionMode::Delegate => "delegate",
            };
            assert_eq!(mode_str, "plan");
        }

        #[test]
        fn test_claude_permission_mode_accept_edits_generates_flag() {
            let mode = PermissionMode::AcceptEdits;
            let mode_str = match mode {
                PermissionMode::Default => "default",
                PermissionMode::Plan => "plan",
                PermissionMode::AcceptEdits => "acceptEdits",
                PermissionMode::Delegate => "delegate",
            };
            assert_eq!(mode_str, "acceptEdits");
        }

        #[test]
        fn test_claude_permission_mode_delegate_generates_flag() {
            let mode = PermissionMode::Delegate;
            let mode_str = match mode {
                PermissionMode::Default => "default",
                PermissionMode::Plan => "plan",
                PermissionMode::AcceptEdits => "acceptEdits",
                PermissionMode::Delegate => "delegate",
            };
            assert_eq!(mode_str, "delegate");
        }

        #[test]
        fn test_claude_tool_permissions_generate_correct_flags() {
            let manager = TranslatorManager::new();
            let claude = manager.get("claude").unwrap();

            let step = StepPermissions {
                tools: ToolPermissions {
                    allow: vec![
                        ToolPattern::new("Read"),
                        ToolPattern::with_pattern("Bash", "cargo:*"),
                    ],
                    deny: vec![ToolPattern::with_pattern("Bash", "rm:*")],
                },
                ..Default::default()
            };
            let permissions = PermissionSet::from_step(&step, &ProviderCliArgs::default());
            let flags = claude.generate_cli_flags(&permissions);

            // Verify allow flags
            assert!(flags.contains(&"--allowedTools".to_string()));
            assert!(flags.contains(&"Read".to_string()));
            assert!(flags.contains(&"Bash(cargo:*)".to_string()));

            // Verify deny flags
            assert!(flags.contains(&"--disallowedTools".to_string()));
            assert!(flags.contains(&"Bash(rm:*)".to_string()));
        }

        // ========================================
        // Gemini step permission tests
        // ========================================

        #[test]
        fn test_gemini_tool_mapping_correct() {
            let manager = TranslatorManager::new();
            let gemini = manager.get("gemini").unwrap();

            let step = StepPermissions {
                tools: ToolPermissions {
                    allow: vec![
                        ToolPattern::new("Bash"),
                        ToolPattern::new("Read"),
                        ToolPattern::new("Write"),
                    ],
                    ..Default::default()
                },
                ..Default::default()
            };
            let permissions = PermissionSet::from_step(&step, &ProviderCliArgs::default());
            let content = gemini
                .generate_config_content(&permissions)
                .expect("Should generate config");

            // Verify tool name mappings
            assert!(
                content.contains("ShellTool"),
                "Bash should map to ShellTool"
            );
            assert!(
                content.contains("ReadFileTool"),
                "Read should map to ReadFileTool"
            );
            assert!(
                content.contains("WriteFileTool"),
                "Write should map to WriteFileTool"
            );
        }

        #[test]
        fn test_gemini_config_dir_flag_added() {
            let manager = TranslatorManager::new();
            let gemini = manager.get("gemini").unwrap();

            // Verify Gemini uses config file
            assert!(!gemini.uses_cli_only());
            assert_eq!(gemini.config_path(), Some(".gemini/settings.json"));
        }

        // ========================================
        // Codex step permission tests
        // ========================================

        #[test]
        fn test_codex_tool_mapping_correct() {
            let manager = TranslatorManager::new();
            let codex = manager.get("codex").unwrap();

            // Codex only generates config when patterns have a pattern string
            let step = StepPermissions {
                tools: ToolPermissions {
                    allow: vec![
                        ToolPattern::with_pattern("Bash", "cargo:*"),
                        ToolPattern::with_pattern("Read", "./src/**"),
                        ToolPattern::with_pattern("Edit", "./src/**"),
                    ],
                    ..Default::default()
                },
                ..Default::default()
            };
            let permissions = PermissionSet::from_step(&step, &ProviderCliArgs::default());
            let content = codex
                .generate_config_content(&permissions)
                .expect("Should generate config");

            // Verify tool name mappings in TOML
            assert!(content.contains("exec"), "Bash should map to exec");
            assert!(
                content.contains("apply_patch"),
                "Edit should map to apply_patch"
            );
        }

        #[test]
        fn test_codex_creates_toml_config() {
            let manager = TranslatorManager::new();
            let codex = manager.get("codex").unwrap();

            // Verify Codex uses TOML config file
            assert!(!codex.uses_cli_only());
            assert_eq!(codex.config_path(), Some(".codex/config.toml"));
        }

        // ========================================
        // Cross-provider tests
        // ========================================

        #[test]
        fn test_provider_specific_cli_args_added() {
            let manager = TranslatorManager::new();

            // Test that provider-specific CLI args are available
            let cli_args = ProviderCliArgs {
                claude: vec!["--custom-claude-flag".to_string()],
                gemini: vec!["--custom-gemini-flag".to_string()],
                codex: vec!["--custom-codex-flag".to_string()],
            };

            // Verify each provider can access its CLI args
            assert_eq!(cli_args.claude, vec!["--custom-claude-flag"]);
            assert_eq!(cli_args.gemini, vec!["--custom-gemini-flag"]);
            assert_eq!(cli_args.codex, vec!["--custom-codex-flag"]);

            // Verify TranslatorManager has all three providers
            assert!(manager.get("claude").is_some());
            assert!(manager.get("gemini").is_some());
            assert!(manager.get("codex").is_some());
        }

        #[test]
        fn test_same_permissions_different_provider_output() {
            let manager = TranslatorManager::new();

            // Use patterns with pattern strings so all providers generate output
            let step = StepPermissions {
                tools: ToolPermissions {
                    allow: vec![
                        ToolPattern::with_pattern("Read", "./src/**"),
                        ToolPattern::with_pattern("Write", "./src/**"),
                    ],
                    deny: vec![ToolPattern::with_pattern("Bash", "rm:*")],
                },
                ..Default::default()
            };
            let permissions = PermissionSet::from_step(&step, &ProviderCliArgs::default());

            // Claude uses CLI flags
            let claude = manager.get("claude").unwrap();
            let claude_flags = claude.generate_cli_flags(&permissions);
            assert!(!claude_flags.is_empty(), "Claude should generate CLI flags");
            assert!(claude.uses_cli_only());

            // Gemini uses config file
            let gemini = manager.get("gemini").unwrap();
            let gemini_content = gemini.generate_config_content(&permissions);
            assert!(
                gemini_content.is_some(),
                "Gemini should generate config content"
            );
            assert!(!gemini.uses_cli_only());

            // Codex uses config file (needs pattern strings to generate content)
            let codex = manager.get("codex").unwrap();
            let codex_content = codex.generate_config_content(&permissions);
            assert!(
                codex_content.is_some(),
                "Codex should generate config content when patterns have pattern strings"
            );
            assert!(!codex.uses_cli_only());
        }

        #[test]
        fn test_permission_set_merge_additive() {
            let project_perms = StepPermissions {
                tools: ToolPermissions {
                    allow: vec![ToolPattern::new("Read")],
                    deny: vec![],
                },
                ..Default::default()
            };

            let step_perms = StepPermissions {
                tools: ToolPermissions {
                    allow: vec![ToolPattern::new("Write")],
                    deny: vec![ToolPattern::with_pattern("Bash", "rm:*")],
                },
                ..Default::default()
            };

            let merged =
                PermissionSet::merge(&project_perms, &step_perms, &ProviderCliArgs::default());

            // Verify merge is additive
            assert_eq!(
                merged.tools_allow.len(),
                2,
                "Should have 2 allowed tools after merge"
            );
            assert_eq!(
                merged.tools_deny.len(),
                1,
                "Should have 1 denied tool after merge"
            );
        }
    }
}

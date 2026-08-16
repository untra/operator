//! Step-command construction for the multi-step exec chain.
//!
//! One builder produces the `opr8r --ticket-id … --step … -- <llm cmd>` wrapper
//! for both the first step (launcher) and subsequent steps (`complete_step`
//! route). The returned command is always the INNER command — target wrapping
//! (docker, remote) is applied once, to the outermost launch, by the launcher;
//! `exec()` transitions happen inside the already-wrapped environment.

use anyhow::Result;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::config::Config;
use crate::queue::Ticket;
use crate::templates::schema::StepSchema;

use super::interpolation::PromptInterpolator;
use super::llm_command::{
    apply_yolo_flags, build_llm_command_with_permissions_for_tool, locate_opr8r_binary,
};
use super::prompt::{generate_session_uuid, shell_escape_if_needed, write_prompt_file};

/// Launch context fixed at launch time, persisted with the agent record, and
/// read back by `complete_step` to build subsequent step commands.
///
/// The persisted context is the baseline for a ticket's whole chain; per-step
/// `agent` overrides from the step schema apply on top for that step only.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export)]
pub struct StepLaunchContext {
    /// Delegator name used at launch (None = ad-hoc provider/model)
    #[serde(default)]
    pub delegator: Option<String>,
    /// Resolved LLM tool (e.g. "claude")
    pub tool: String,
    /// Resolved model alias (e.g. "sonnet")
    pub model: String,
    /// YOLO (auto-accept) mode
    pub yolo: bool,
    /// Session UUID of the step launched with this context (informational;
    /// each transition mints a fresh UUID for the next step)
    #[serde(default)]
    pub session_id: Option<String>,
    /// opr8r invocation for the launch environment ("opr8r" inside a
    /// container where the image ships it on PATH, an absolute path locally).
    /// Steps exec inside the same environment, so the value holds chain-wide.
    pub opr8r: String,
    /// Relay MCP injection override from the delegator launch config
    #[serde(default)]
    pub operator_relay: Option<bool>,
    /// Extra CLI flags from the delegator launch config
    #[serde(default)]
    pub extra_flags: Vec<String>,
}

/// A step command plus the session UUID minted for it. The caller persists the
/// UUID (ticket `session_ids`, agent state) — building is side-effect-free
/// with respect to ticket and state files.
#[derive(Debug)]
pub struct BuiltStepCommand {
    pub command: String,
    pub session_id: String,
}

/// Build the full `opr8r`-wrapped command for one step.
///
/// Renders the step prompt (issuetype + step + ticket + previous-step
/// context), writes the prompt file, builds the tool command with permission
/// config flags, applies YOLO and delegator flags, and wraps in the opr8r
/// step wrapper. Never applies target wrapping (see module docs).
pub fn build_step_command(
    config: &Config,
    ticket: &Ticket,
    step: &StepSchema,
    ctx: &StepLaunchContext,
    project_path: &str,
    previous_summary: Option<&str>,
    previous_recommendation: Option<&str>,
) -> Result<BuiltStepCommand> {
    // Per-step agent override: a step naming a delegator switches tool/model
    // for that step only; the persisted context stays the chain baseline.
    let override_pair =
        crate::templates::step_type::effective_agent(step).map(|agent_name| {
            config.delegators.iter().find(|d| d.name == agent_name).map_or_else(
            || {
                tracing::warn!(
                    step = %step.name,
                    agent = %agent_name,
                    "Step agent override names no configured delegator; using launch context"
                );
                (ctx.tool.clone(), ctx.model.clone())
            },
            |d| (d.llm_tool.clone(), d.model.clone()),
        )
        });
    let (tool, model) = override_pair.unwrap_or_else(|| (ctx.tool.clone(), ctx.model.clone()));

    let session_id = generate_session_uuid();

    // Render the prompt against a ticket positioned on this step.
    let mut step_ticket = ticket.clone();
    step_ticket.step = step.name.clone();
    let interpolator = PromptInterpolator::new();
    let prompt = interpolator.build_launch_prompt_with_context(
        config,
        &step_ticket,
        project_path,
        previous_summary,
        previous_recommendation,
    )?;
    let prompt_file = write_prompt_file(config, &session_id, &prompt)?;

    let mut llm_cmd = build_llm_command_with_permissions_for_tool(
        config,
        &tool,
        &model,
        &session_id,
        &prompt_file,
        Some(&step_ticket),
        Some(project_path),
        ctx.operator_relay,
    )?;

    if ctx.yolo {
        llm_cmd = apply_yolo_flags(config, &llm_cmd, &tool);
    }

    if !ctx.extra_flags.is_empty() {
        llm_cmd = format!("{} {}", llm_cmd, ctx.extra_flags.join(" "));
    }

    let command = wrap_step(&ctx.opr8r, &ticket.id, &step.name, &session_id, &llm_cmd);

    Ok(BuiltStepCommand {
        command,
        session_id,
    })
}

/// Wrap an already-built inner LLM command in the opr8r step wrapper.
/// Used directly by the launcher for step one, where the inner command has
/// already been constructed by the session backend.
///
/// Ticket ids, step names, and session UUIDs satisfy strict grammars and are
/// passed raw; only the opr8r path can contain shell-hostile characters.
pub fn wrap_step(
    opr8r: &str,
    ticket_id: &str,
    step_name: &str,
    session_id: &str,
    inner_cmd: &str,
) -> String {
    format!(
        "{} --ticket-id={ticket_id} --step={step_name} --session-id={session_id} -- {inner_cmd}",
        shell_escape_if_needed(opr8r),
    )
}

/// Whether a launch should participate in the exec chain: true when the
/// workspace issuetype registry defines the step being launched.
///
/// Resolves against the same catalog the `complete_step` route uses
/// (`startup::templates::load_registry`) rather than the embedded builtins, so
/// collection-installed and custom issue types chain too, and an override of a
/// builtin key follows one graph on both sides. Schema-less tickets (step
/// "initial") launch unwrapped, exactly as before.
///
/// Called once per launch, alongside far heavier worktree/session setup.
pub fn chain_step(config: &Config, ticket: &Ticket, step_name: &str) -> bool {
    let registry = crate::startup::templates::load_registry(&config.tickets_path());
    registry
        .get(&ticket.ticket_type.to_uppercase())
        .is_some_and(|t| t.get_step(step_name).is_some())
}

/// Resolve the opr8r invocation for the launch environment.
///
/// Inside a container the image ships `opr8r` on PATH; locally the binary
/// sits alongside `operator` (or on PATH as a fallback).
pub fn resolve_opr8r_invocation(containerized: bool) -> String {
    if containerized {
        return "opr8r".to_string();
    }
    locate_opr8r_binary().map_or_else(|| "opr8r".to_string(), |p| p.display().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Delegator, DetectedTool};

    fn make_detected_tool(name: &str) -> DetectedTool {
        DetectedTool {
            name: name.to_string(),
            path: format!("/usr/bin/{name}"),
            version: "1.0.0".to_string(),
            min_version: Some("1.0.0".to_string()),
            version_ok: true,
            model_aliases: vec!["sonnet".to_string()],
            command_template: format!(
                "{name} {{{{config_flags}}}}{{{{model_flag}}}}--session-id {{{{session_id}}}} --print-prompt-path {{{{prompt_file}}}}"
            ),
            capabilities: crate::config::ToolCapabilities {
                supports_sessions: true,
                supports_headless: true,
            },
            yolo_flags: vec!["--dangerously-skip-permissions".to_string()],
            health_ok: true,
        }
    }

    fn make_config(temp_dir: &std::path::Path) -> Config {
        let mut config = Config::default();
        config.paths.tickets = temp_dir.join(".tickets").to_string_lossy().to_string();
        config.llm_tools.detected =
            vec![make_detected_tool("claude"), make_detected_tool("gemini")];
        config
    }

    /// FEAT tickets use the embedded FEAT schema: plan -> build -> code,
    /// where "code" carries agent: "claude-opus".
    fn make_feat_ticket(temp_dir: &std::path::Path, step: &str) -> Ticket {
        let content = format!(
            "---\nid: FEAT-9001\nstatus: running\nstep: {step}\n---\n\n# Feature: step command test\n"
        );
        let path = temp_dir.join("20260807-1200-FEAT-operator-stepcmd.md");
        std::fs::write(&path, content).unwrap();
        Ticket::from_file(&path).unwrap()
    }

    fn make_ctx() -> StepLaunchContext {
        StepLaunchContext {
            delegator: Some("primary".to_string()),
            tool: "claude".to_string(),
            model: "sonnet".to_string(),
            yolo: false,
            session_id: None,
            opr8r: "opr8r".to_string(),
            operator_relay: Some(false),
            extra_flags: vec![],
        }
    }

    fn build_step(step_name: &str) -> StepSchema {
        // Pull the real step from the embedded FEAT schema so agent
        // overrides and prompts match production data.
        let temp = tempfile::tempdir().unwrap();
        let ticket = make_feat_ticket(temp.path(), step_name);
        ticket
            .template_schema()
            .and_then(|t| t.get_step(step_name).cloned())
            .expect("embedded FEAT schema step")
    }

    #[test]
    fn test_build_step_command_includes_model_flag() {
        let temp = tempfile::tempdir().unwrap();
        let config = make_config(temp.path());
        let ticket = make_feat_ticket(temp.path(), "plan");
        let step = build_step("build");
        let ctx = make_ctx();

        let built = build_step_command(
            &config,
            &ticket,
            &step,
            &ctx,
            temp.path().to_str().unwrap(),
            None,
            None,
        )
        .unwrap();

        assert!(
            built.command.contains("--model sonnet"),
            "model from the launch context must reach the command: {}",
            built.command
        );
        assert!(
            built.command.contains("--ticket-id=FEAT-9001"),
            "opr8r wrapper must carry the ticket id: {}",
            built.command
        );
        assert!(
            built.command.contains("--step=build"),
            "opr8r wrapper must carry the step name: {}",
            built.command
        );
        assert!(
            built
                .command
                .contains(&format!("--session-id={}", built.session_id)),
            "opr8r wrapper must carry the minted session id: {}",
            built.command
        );
    }

    #[test]
    fn test_build_step_command_applies_yolo_flags() {
        let temp = tempfile::tempdir().unwrap();
        let config = make_config(temp.path());
        let ticket = make_feat_ticket(temp.path(), "plan");
        let step = build_step("build");
        let mut ctx = make_ctx();
        ctx.yolo = true;

        let built = build_step_command(
            &config,
            &ticket,
            &step,
            &ctx,
            temp.path().to_str().unwrap(),
            None,
            None,
        )
        .unwrap();

        assert!(
            built.command.contains("--dangerously-skip-permissions"),
            "YOLO flags must be present when the context sets them: {}",
            built.command
        );
    }

    #[test]
    fn test_build_step_command_honors_step_agent_override() {
        let temp = tempfile::tempdir().unwrap();
        let mut config = make_config(temp.path());
        // The FEAT "code" step names agent "claude-opus"; register a
        // delegator by that name pointing at a different tool + model.
        config.delegators = vec![Delegator {
            name: "claude-opus".to_string(),
            llm_tool: "gemini".to_string(),
            model: "gemini-pro".to_string(),
            display_name: None,
            model_properties: std::collections::HashMap::new(),
            launch_config: None,
            model_server: None,
            remote_agent: None,
            x_agnt: None,
            x_openai: None,
            unmapped_core: None,
        }];
        let ticket = make_feat_ticket(temp.path(), "build");
        let step = build_step("code");
        let ctx = make_ctx();

        let built = build_step_command(
            &config,
            &ticket,
            &step,
            &ctx,
            temp.path().to_str().unwrap(),
            None,
            None,
        )
        .unwrap();

        assert!(
            built.command.contains("gemini") && built.command.contains("--model gemini-pro"),
            "SwitchAgent step must override tool and model for that step: {}",
            built.command
        );
    }

    #[test]
    fn test_build_step_command_unknown_override_falls_back_to_context() {
        let temp = tempfile::tempdir().unwrap();
        let config = make_config(temp.path()); // no delegators configured
        let ticket = make_feat_ticket(temp.path(), "build");
        let step = build_step("code"); // names agent "claude-opus"
        let ctx = make_ctx();

        let built = build_step_command(
            &config,
            &ticket,
            &step,
            &ctx,
            temp.path().to_str().unwrap(),
            None,
            None,
        )
        .unwrap();

        assert!(
            built.command.contains("--model sonnet"),
            "unknown step agent must fall back to the launch context: {}",
            built.command
        );
    }

    #[test]
    fn test_next_command_has_no_docker_wrapper() {
        // A docker-launched chain builds INNER commands only: exec() happens
        // inside the container, so re-wrapping would nest containers.
        let temp = tempfile::tempdir().unwrap();
        let mut config = make_config(temp.path());
        config.launch.docker.enabled = true;
        config.launch.docker.image = "ghcr.io/untra/operator:test".to_string();
        let ticket = make_feat_ticket(temp.path(), "plan");
        let step = build_step("build");
        let ctx = make_ctx(); // opr8r = "opr8r", as a docker launch persists

        let built = build_step_command(
            &config,
            &ticket,
            &step,
            &ctx,
            temp.path().to_str().unwrap(),
            None,
            None,
        )
        .unwrap();

        assert!(
            !built.command.contains("docker run"),
            "next_command must never include the target wrapper: {}",
            built.command
        );
        assert!(
            built.command.starts_with("opr8r "),
            "next_command must start with the opr8r wrapper: {}",
            built.command
        );
    }

    #[test]
    fn test_next_command_uses_persisted_delegator_context() {
        let temp = tempfile::tempdir().unwrap();
        let config = make_config(temp.path());
        let ticket = make_feat_ticket(temp.path(), "plan");
        let step = build_step("build"); // no agent override on "build"
        let mut ctx = make_ctx();
        ctx.tool = "gemini".to_string();
        ctx.model = "gemini-flash".to_string();

        let built = build_step_command(
            &config,
            &ticket,
            &step,
            &ctx,
            temp.path().to_str().unwrap(),
            None,
            None,
        )
        .unwrap();

        assert!(
            built.command.contains("gemini") && built.command.contains("--model gemini-flash"),
            "second step must use the launching delegator's tool/model, not a default: {}",
            built.command
        );
    }

    #[test]
    fn test_wrap_step_shape() {
        let cmd = wrap_step(
            "opr8r",
            "FEAT-1",
            "build",
            "uuid-1",
            "claude --model sonnet",
        );
        assert_eq!(
            cmd,
            "opr8r --ticket-id=FEAT-1 --step=build --session-id=uuid-1 -- claude --model sonnet"
        );
    }

    #[test]
    fn test_chain_step_gates_on_schema() {
        let temp = tempfile::tempdir().unwrap();
        let config = make_config(temp.path());
        let ticket = make_feat_ticket(temp.path(), "plan");
        assert!(chain_step(&config, &ticket, "plan"));
        assert!(!chain_step(&config, &ticket, "initial"));
    }

    /// A non-builtin issue type installed into the workspace registry must
    /// wrap too — otherwise a collection issuetype's chain never engages.
    #[test]
    fn test_chain_step_wraps_registry_only_issuetype() {
        const GAST_JSON: &str = r#"{
            "key": "GAST",
            "name": "Gastown",
            "description": "A registry-only type",
            "mode": "autonomous",
            "glyph": "g",
            "fields": [],
            "steps": [{"name": "execute", "outputs": ["report"], "prompt": "Do it."}]
        }"#;

        let temp = tempfile::tempdir().unwrap();
        let config = make_config(temp.path());
        let legacy = config.tickets_path().join("operator").join("issuetypes");
        std::fs::create_dir_all(&legacy).unwrap();
        std::fs::write(legacy.join("gast.json"), GAST_JSON).unwrap();

        let content =
            "---\nid: GAST-9001\nstatus: running\nstep: execute\n---\n\n# Registry-only type\n";
        let path = temp.path().join("20260807-1200-GAST-operator-gast.md");
        std::fs::write(&path, content).unwrap();
        let ticket = Ticket::from_file(&path).unwrap();

        assert_eq!(ticket.ticket_type, "GAST");
        assert!(
            ticket.template_schema().is_none(),
            "GAST must not be an embedded builtin, or the test proves nothing"
        );
        assert!(
            chain_step(&config, &ticket, "execute"),
            "registry-resolved issue type must chain"
        );
        assert!(!chain_step(&config, &ticket, "nonexistent"));
    }

    #[test]
    fn test_resolve_opr8r_invocation_containerized_uses_path_name() {
        assert_eq!(resolve_opr8r_invocation(true), "opr8r");
    }

    #[test]
    fn test_step_launch_context_roundtrip() {
        let ctx = make_ctx();
        let json = serde_json::to_string(&ctx).unwrap();
        let back: StepLaunchContext = serde_json::from_str(&json).unwrap();
        assert_eq!(ctx, back);
    }

    #[test]
    fn test_step_launch_context_parses_without_optional_fields() {
        // Old state files predate optional fields; they must still parse.
        let json = r#"{"tool":"claude","model":"sonnet","yolo":false,"opr8r":"opr8r"}"#;
        let ctx: StepLaunchContext = serde_json::from_str(json).unwrap();
        assert_eq!(ctx.delegator, None);
        assert!(ctx.extra_flags.is_empty());
    }
}

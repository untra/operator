//! Orchestration: render a ticket against its issuetype into an **AGNT.gg**
//! workflow JSON document (`{ name, description, nodes[], edges[] }`).
//!
//! This is the second emitter target alongside the Claude `.js` one in
//! `export.rs`. Like that path it is **export-only** and a lossy autonomous
//! *flattening*: each operator step becomes one `operator-*` graph node, the
//! `next_step` chain becomes edges, and concepts AGNT cannot faithfully
//! represent (human review gates, RAG/MCP sandboxing, fan-out shapes) are
//! recorded honestly in the node `config` under a `gap` field rather than
//! silently dropped — mirroring the `OPERATOR-GAP` comments in the `.js` target.
//!
//! The emitted nodes use the `operator-*` type vocabulary defined by the
//! companion AGNT plugin (`agnt-plugin/`), so an exported workflow runs in AGNT
//! when that plugin is installed.
//!
//! The node/edge shape matches AGNT's *runnable* workflow schema, verified
//! against AGNT's own code (`agnt-gg/agnt`), not its simplified API-examples
//! docs page: nodes carry `{ id, type, text, x, y, parameters }` and edges carry
//! `{ id, start: { id }, end: { id } }`. The runtime engine (`WorkflowEngine.js`)
//! traverses `edge.start.id`/`edge.end.id`, the node executor (`NodeExecutor.js`)
//! resolves `node.parameters` into the tool's `execute(params)`, and the
//! workflow validator (`orchestrator/workflowTools.js`) rejects nodes lacking
//! `text`/`x`/`y` or edges lacking `id`/`start.id`/`end.id`. Workflows are stored
//! verbatim (`WorkflowService.saveWorkflow`), so the shape we emit is the shape
//! that runs.

use anyhow::{Context, Result};
use handlebars::Handlebars;
use serde::Serialize;
use serde_json::{json, Map, Value};

use super::export::{handlebars_renderer, meta_description, meta_name, ordered_steps, PipelineEnv};
use super::step_map;
use super::GAP_MARKER;
use crate::config::Config;
use crate::issuetypes::IssueType;
use crate::pr_config::PrConfig;
use crate::queue::Ticket;
use crate::steps::manager::StepManager;
use crate::templates::schema::{RagSource, ReviewType, StepSchema, StepTypeTag};

/// The node type every exported step maps to. Defined by the companion AGNT
/// plugin (`agnt-plugin/run-step.js`), whose `execute()` reads `config.ticket`
/// and calls Operator's REST API to run the ticket for that step. Distinct from
/// the plugin's `operator-launch-agent` node (which launches a whole ticket by
/// `id`): the export emits one `operator-run-step` per issuetype step, and the
/// tool that consumes it reads the same `ticket`/`step` keys this emitter writes.
const NODE_TYPE: &str = "operator-run-step";

/// The node type emitted for a step whose resolved delegator declaratively
/// references a named AGNT agent (see [`crate::config::AgentProfile`]). Rather
/// than route the step back through Operator (`operator-run-step`), AGNT runs the
/// step natively with its own agent-chat node.
///
/// This is AGNT's native tool, confirmed against its tool registry
/// (`frontend/src/views/Docs/docfiles/tools/agnt-agent.md` in `agnt-gg/agnt`):
/// the registered tool id is `agnt-agent` and its required parameters are
/// `agentId` (the AGNT agent id, a UUID) and `message` (the prompt to send). It
/// belongs to the AGNT runtime, so an exported workflow with an `agnt-agent` node
/// runs natively in AGNT without the operator plugin.
const AGNT_AGENT_NODE_TYPE: &str = "agnt-agent";

/// An AGNT workflow document: a graph of nodes connected by edges.
#[derive(Debug, Serialize)]
struct AgntWorkflow {
    name: String,
    description: String,
    nodes: Vec<AgntNode>,
    edges: Vec<AgntEdge>,
}

/// One AGNT workflow node. `parameters` is the per-node bag the node executor
/// resolves into the tool's `execute(params)`; `text` is the canvas label and
/// `x`/`y` the canvas coordinates (all required by AGNT's workflow validator).
#[derive(Debug, Serialize)]
struct AgntNode {
    id: String,
    #[serde(rename = "type")]
    node_type: String,
    text: String,
    x: i64,
    y: i64,
    parameters: Value,
}

/// A directed edge. AGNT's engine traverses `start.id` → `end.id` and tracks the
/// edge by `id`, so all three are required (a bare `{source,target}` won't run).
#[derive(Debug, Serialize)]
struct AgntEdge {
    id: String,
    start: EdgeEnd,
    end: EdgeEnd,
}

/// One endpoint of an [`AgntEdge`], referencing a node by `id`.
#[derive(Debug, Serialize)]
struct EdgeEnd {
    id: String,
}

/// Render `ticket` against `issuetype` into an AGNT workflow JSON string.
///
/// Deterministic given the same project/filesystem environment: the output
/// never contains wall-clock, `Date.now`, or `Math.random`.
pub fn export_workflow_agnt(
    ticket: &Ticket,
    issuetype: &IssueType,
    pr_config: Option<&PrConfig>,
    _pipeline_env: &PipelineEnv,
    config: &Config,
) -> Result<String> {
    // Reuse the exact variable surface a step prompt sees at launch time, and
    // the same step ordering + non-strict Handlebars the `.js` target uses.
    let ctx = StepManager::build_ticket_context(ticket, pr_config);
    let hbs = handlebars_renderer();
    let steps = ordered_steps(issuetype);

    let mut nodes = Vec::with_capacity(steps.len());
    for (index, step) in steps.iter().enumerate() {
        nodes.push(build_node(&hbs, &ctx, step, ticket, config, index)?);
    }

    // Edges follow the `next_step` chain (only when the target resolves to a
    // real step). Terminal/unreachable steps are left as islands.
    let mut edges = Vec::new();
    for step in &steps {
        if let Some(next) = step.next_step.as_deref() {
            if issuetype.get_step(next).is_some() {
                edges.push(AgntEdge {
                    id: format!("{}->{}", step.name, next),
                    start: EdgeEnd {
                        id: step.name.clone(),
                    },
                    end: EdgeEnd {
                        id: next.to_string(),
                    },
                });
            }
        }
    }

    let wf = AgntWorkflow {
        name: meta_name(ticket, issuetype),
        description: meta_description(issuetype),
        nodes,
        edges,
    };
    serde_json::to_string_pretty(&wf).context("failed to serialize AGNT workflow")
}

/// Build one node from a step. `index` positions the node vertically on the
/// canvas (the chain is linear). Lossy cases (RAG/MCP sandboxing, fan-out shapes,
/// human review gates) are recorded in `parameters` rather than dropped: `gap`
/// collects `OPERATOR-GAP` notes, `fanout` summarizes a flattened parallel shape.
fn build_node(
    hbs: &Handlebars,
    ctx: &Value,
    step: &StepSchema,
    ticket: &Ticket,
    app_config: &Config,
    index: usize,
) -> Result<AgntNode> {
    let mut parameters = Map::new();
    parameters.insert("ticket".into(), json!(ticket.id));
    parameters.insert("step".into(), json!(step.name));

    let mut effective_prompt = render(hbs, &step.prompt, ctx)?;
    let mut model = step.agent.clone();
    let mut gaps: Vec<String> = Vec::new();

    match step.step_type {
        StepTypeTag::Task => {}
        StepTypeTag::Delegator => {
            if let Some(cfg) = &step.delegator_config {
                model = Some(cfg.delegator.clone());
                if let Some(flavor) = &cfg.prompt_flavor {
                    let rendered_flavor = render(hbs, flavor, ctx)?;
                    effective_prompt = format!("{rendered_flavor}\n\n{effective_prompt}");
                }
            }
        }
        StepTypeTag::Classifier => {
            let schema = if let Some(explicit) = &step.json_schema {
                explicit.clone()
            } else if let Some(cfg) = &step.classifier_config {
                step_map::classifier_schema(cfg)
            } else {
                json!({ "type": "object" })
            };
            parameters.insert("schema".into(), schema);
        }
        StepTypeTag::Rag => {
            gaps.push(format!(
                "{GAP_MARKER}: rag step — the workflow sandbox has no filesystem; context sources must be gathered by the agent."
            ));
            if let Some(cfg) = &step.rag_config {
                let srcs: Vec<Value> = cfg
                    .sources
                    .iter()
                    .map(|s| json!(describe_rag_source(s)))
                    .collect();
                if !srcs.is_empty() {
                    parameters.insert("contextSources".into(), Value::Array(srcs));
                }
            }
        }
        StepTypeTag::Mcp => {
            gaps.push(format!(
                "{GAP_MARKER}: mcp step — the workflow sandbox cannot guarantee MCP tool availability."
            ));
            if let Some(cfg) = &step.mcp_config {
                let tools: Vec<Value> = cfg
                    .required_tools
                    .iter()
                    .map(|t| match &t.tool {
                        Some(tool) => json!(format!("{}/{}", t.server, tool)),
                        None => json!(t.server),
                    })
                    .collect();
                if !tools.is_empty() {
                    parameters.insert("requiredTools".into(), Value::Array(tools));
                }
            }
        }
        StepTypeTag::MultiModel
        | StepTypeTag::MultiPrompt
        | StepTypeTag::Matrixed
        | StepTypeTag::Pipeline => {
            parameters.insert("fanout".into(), json!(fanout_description(step)));
            gaps.push(format!(
                "{GAP_MARKER}: {} fan-out flattened to a single node; the parallel/voting shape runs inside one operator agent launch.",
                step_type_label(&step.step_type),
            ));
        }
    }

    // Human review gates have no autonomous AGNT analog (mirrors the `.js`
    // judge-loop flattening, but AGNT nodes cannot loop on a human verdict).
    if step.requires_review() || step.on_reject.is_some() {
        let goto = step
            .on_reject
            .as_ref()
            .map(|r| format!(" (original on_reject -> goto {})", r.goto_step))
            .unwrap_or_default();
        gaps.push(format!(
            "{GAP_MARKER}: step '{}' had human review_type={}; AGNT runs autonomously, so the human gate is flattened{goto}.",
            step.name,
            review_label(&step.review_type),
        ));
    }

    // If the step's resolved delegator (held in `model`) declaratively references
    // an AGNT agent, emit a native `agnt-agent` node so AGNT runs the step itself
    // rather than calling back into Operator. Operator-only launch_config on such
    // a delegator has no analog on the AGNT side, so its presence is gap-marked.
    // Only AGNT-hosted remote agents get a native node; other platforms (e.g.
    // OpenAI) ride opaquely in the profile and have no AGNT workflow analog.
    let agnt_delegator = model
        .as_deref()
        .and_then(|name| app_config.delegators.iter().find(|d| d.name == name))
        .filter(|d| {
            d.remote_agent
                .as_ref()
                .is_some_and(|r| r.platform == "agnt")
        });
    let node_type = if let Some(d) = agnt_delegator {
        let agent_id = d
            .remote_agent
            .as_ref()
            .expect("filtered to an agnt remote_agent above")
            .id
            .clone();
        parameters.insert("agentId".into(), json!(agent_id));
        if d.launch_config.is_some() {
            gaps.push(format!(
                "{GAP_MARKER}: delegator '{}' carries operator launch_config (permission mode/flags/worktree/docker) that AGNT agents have no analog for; only the agent reference is exported.",
                d.name,
            ));
        }
        AGNT_AGENT_NODE_TYPE
    } else {
        NODE_TYPE
    };

    // The agnt-agent tool reads `message` (the prompt to send to the agent); the
    // operator-run-step tool ignores the prompt (Operator owns it internally) and
    // carries it only as an inert annotation for the AGNT canvas.
    if node_type == AGNT_AGENT_NODE_TYPE {
        parameters.insert("message".into(), json!(effective_prompt));
    } else {
        parameters.insert("prompt".into(), json!(effective_prompt));
    }
    if let Some(m) = model {
        parameters.insert("model".into(), json!(m));
    }
    if !step.allowed_tools.is_empty() {
        parameters.insert("allowedTools".into(), json!(step.allowed_tools));
    }
    if !gaps.is_empty() {
        parameters.insert("gap".into(), json!(gaps.join(" | ")));
    }

    Ok(AgntNode {
        id: step.name.clone(),
        node_type: node_type.to_string(),
        text: step.display_name().to_string(),
        x: 0,
        y: index as i64 * 160,
        parameters: Value::Object(parameters),
    })
}

fn render(hbs: &Handlebars, template: &str, ctx: &Value) -> Result<String> {
    hbs.render_template(template, ctx)
        .context("failed to render step prompt for AGNT workflow export")
}

fn review_label(review: &ReviewType) -> &'static str {
    match review {
        ReviewType::None => "output",
        ReviewType::Plan => "plan",
        ReviewType::Visual => "visual",
        ReviewType::Pr => "pr",
    }
}

fn describe_rag_source(src: &RagSource) -> String {
    match src {
        RagSource::Glob { pattern } => format!("glob:{pattern}"),
        RagSource::File { path } => format!("file:{path}"),
        RagSource::Mcp { server, tool, .. } => format!("mcp:{server}/{tool}"),
    }
}

fn step_type_label(t: &StepTypeTag) -> &'static str {
    match t {
        StepTypeTag::MultiModel => "multi-model",
        StepTypeTag::MultiPrompt => "multi-prompt",
        StepTypeTag::Matrixed => "matrixed",
        StepTypeTag::Pipeline => "pipeline",
        _ => "fan-out",
    }
}

/// A human-readable summary of a fan-out step's flattened shape.
fn fanout_description(step: &StepSchema) -> String {
    match step.step_type {
        StepTypeTag::MultiModel => {
            let n = step
                .multi_model_config
                .as_ref()
                .map(|c| c.delegators.len())
                .unwrap_or(0);
            format!("multi-model fan-out across {n} delegators")
        }
        StepTypeTag::MultiPrompt => {
            let n = step
                .multi_prompt_config
                .as_ref()
                .map(|c| c.prompt_variations.len())
                .unwrap_or(0);
            format!("{n} prompt variations + select")
        }
        StepTypeTag::Matrixed => {
            let cfg = step.matrixed_config.as_ref();
            let d = cfg.map(|c| c.delegators.len()).unwrap_or(0);
            let p = cfg.map(|c| c.prompt_variations.len()).unwrap_or(0);
            format!("matrixed {d}x{p} (delegators x prompt variations)")
        }
        StepTypeTag::Pipeline => {
            let n = step
                .pipeline_config
                .as_ref()
                .map(|c| c.stages.len())
                .unwrap_or(0);
            format!("pipeline over {n} stages")
        }
        _ => "fan-out".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::issuetypes::IssueTypeRegistry;
    use std::collections::HashSet;

    fn registry() -> IssueTypeRegistry {
        let mut r = IssueTypeRegistry::new();
        r.load_builtins().expect("load builtins");
        r
    }

    fn ticket(ticket_type: &str) -> Ticket {
        Ticket {
            filename: "20241221-1430-FEAT-gamesvc-test.md".to_string(),
            filepath: "/test/path".to_string(),
            timestamp: "20241221-1430".to_string(),
            ticket_type: ticket_type.to_string(),
            project: "gamesvc".to_string(),
            id: "FEAT-1234".to_string(),
            summary: "Add pagination".to_string(),
            priority: "P2-medium".to_string(),
            status: "queued".to_string(),
            step: String::new(),
            content: "body".to_string(),
            sessions: std::collections::HashMap::new(),
            step_delegators: std::collections::HashMap::new(),
            llm_task: crate::queue::LlmTask::default(),
            worktree_path: None,
            branch: None,
            external_id: None,
            external_url: None,
            external_provider: None,
        }
    }

    fn export(key: &str) -> Value {
        let reg = registry();
        let it = reg.get(key).expect("builtin issuetype");
        let out = export_workflow_agnt(
            &ticket(key),
            it,
            None,
            &PipelineEnv::default(),
            &Config::default(),
        )
        .expect("export");
        serde_json::from_str(&out).expect("output parses as JSON")
    }

    #[test]
    fn agnt_export_parses_and_uses_operator_nodes() {
        let reg = registry();
        let feat = reg.get("FEAT").unwrap();
        let v = export("FEAT");

        assert!(v["name"].is_string(), "missing name");
        assert!(v["description"].is_string(), "missing description");

        let nodes = v["nodes"].as_array().expect("nodes array");
        assert_eq!(nodes.len(), feat.steps.len(), "one node per step expected");
        for n in nodes {
            assert_eq!(n["type"], NODE_TYPE, "every node is an operator-* node");
            assert!(n["id"].as_str().is_some_and(|s| !s.is_empty()), "node id");
            assert!(
                n["parameters"]["prompt"].is_string(),
                "node parameters carry a prompt"
            );
            assert_eq!(
                n["parameters"]["ticket"], "FEAT-1234",
                "node parameters carry the ticket id"
            );
        }
    }

    /// AGNT's workflow validator rejects nodes lacking `text`/`x`/`y` and edges
    /// lacking `id`/`start.id`/`end.id`. Guards the canonical runnable shape.
    #[test]
    fn agnt_nodes_and_edges_carry_canvas_shape() {
        let v = export("FEAT");
        for n in v["nodes"].as_array().unwrap() {
            assert!(
                n["text"].as_str().is_some_and(|s| !s.is_empty()),
                "node '{}' needs a non-empty text label",
                n["id"]
            );
            assert!(n["x"].is_i64(), "node '{}' needs numeric x", n["id"]);
            assert!(n["y"].is_i64(), "node '{}' needs numeric y", n["id"]);
        }
        for e in v["edges"].as_array().unwrap() {
            assert!(e["id"].as_str().is_some_and(|s| !s.is_empty()), "edge id");
            assert!(e["start"]["id"].is_string(), "edge start.id");
            assert!(e["end"]["id"].is_string(), "edge end.id");
        }
    }

    /// Guards the cross-language contract between this emitter and the
    /// `operator-run-step` plugin tool (`agnt-plugin/run-step.js`), which reads
    /// `params.ticket` (resolved from `node.parameters`). If the emitter stops
    /// writing the keys the tool requires, an exported graph would fail at
    /// runtime in AGNT — this catches that.
    #[test]
    fn agnt_nodes_carry_keys_the_run_step_tool_requires() {
        let v = export("FEAT");
        for n in v["nodes"].as_array().unwrap() {
            assert!(
                n["parameters"]["ticket"]
                    .as_str()
                    .is_some_and(|s| !s.is_empty()),
                "node '{}' missing the 'ticket' key the run-step tool requires",
                n["id"]
            );
            assert!(
                n["parameters"]["step"]
                    .as_str()
                    .is_some_and(|s| !s.is_empty()),
                "node '{}' missing 'step'",
                n["id"]
            );
        }
    }

    #[test]
    fn agnt_edges_follow_next_step() {
        let reg = registry();
        let feat = reg.get("FEAT").unwrap();
        let v = export("FEAT");

        let nodes = v["nodes"].as_array().unwrap();
        let ids: HashSet<&str> = nodes.iter().map(|n| n["id"].as_str().unwrap()).collect();

        let expected = feat
            .steps
            .iter()
            .filter(|s| {
                s.next_step
                    .as_deref()
                    .and_then(|n| feat.get_step(n))
                    .is_some()
            })
            .count();
        let edges = v["edges"].as_array().expect("edges array");
        assert_eq!(edges.len(), expected, "one edge per resolving next_step");
        for e in edges {
            assert!(
                ids.contains(e["start"]["id"].as_str().unwrap()),
                "edge start.id is a node"
            );
            assert!(
                ids.contains(e["end"]["id"].as_str().unwrap()),
                "edge end.id is a node"
            );
            assert!(
                e["id"].as_str().is_some_and(|s| !s.is_empty()),
                "edge carries an id"
            );
        }
    }

    #[test]
    fn agnt_export_is_deterministic() {
        let reg = registry();
        let feat = reg.get("FEAT").unwrap();
        let a = export_workflow_agnt(
            &ticket("FEAT"),
            feat,
            None,
            &PipelineEnv::default(),
            &Config::default(),
        )
        .unwrap();
        let b = export_workflow_agnt(
            &ticket("FEAT"),
            feat,
            None,
            &PipelineEnv::default(),
            &Config::default(),
        )
        .unwrap();
        assert_eq!(a, b, "export must be byte-identical for identical inputs");
    }

    #[test]
    fn agnt_review_step_records_gap() {
        let reg = registry();
        let feat = reg.get("FEAT").unwrap();
        let review_steps: Vec<&str> = feat
            .steps
            .iter()
            .filter(|s| s.requires_review() || s.on_reject.is_some())
            .map(|s| s.name.as_str())
            .collect();
        assert!(
            !review_steps.is_empty(),
            "FEAT should have at least one review-gated step"
        );

        let v = export("FEAT");
        for n in v["nodes"].as_array().unwrap() {
            if review_steps.contains(&n["id"].as_str().unwrap()) {
                assert!(
                    n["parameters"]["gap"].is_string(),
                    "review-gated step '{}' must record a gap",
                    n["id"]
                );
            }
        }
    }

    #[test]
    fn agnt_multi_model_step_records_fanout() {
        let json = r#"{
            "key": "MM", "name": "Multi", "description": "d",
            "mode": "autonomous", "glyph": "m", "fields": [],
            "steps": [{
                "name": "vote", "outputs": [], "prompt": "decide {{id}}",
                "type": "multi_model",
                "multi_model_config": { "delegators": ["a", "b"], "voting_strategy": "majority" }
            }]
        }"#;
        let it = IssueType::from_json(json).expect("fixture parses");
        let out = export_workflow_agnt(
            &ticket("MM"),
            &it,
            None,
            &PipelineEnv::default(),
            &Config::default(),
        )
        .expect("export");
        let v: Value = serde_json::from_str(&out).unwrap();
        let node = &v["nodes"][0];
        assert_eq!(node["id"], "vote");
        assert_eq!(node["type"], NODE_TYPE);
        assert!(
            node["parameters"]["fanout"].is_string(),
            "fan-out step records its flattened shape"
        );
        assert!(
            node["parameters"]["gap"].is_string(),
            "fan-out flattening is gap-marked"
        );
    }

    /// A fixture issuetype with a single step whose `agent` names `delegator`.
    fn issuetype_with_step_agent(delegator: &str) -> IssueType {
        let json = format!(
            r#"{{
                "key": "AG", "name": "Agent", "description": "d",
                "mode": "autonomous", "glyph": "a", "fields": [],
                "steps": [{{
                    "name": "research", "outputs": [], "prompt": "investigate {{{{id}}}}",
                    "agent": "{delegator}"
                }}]
            }}"#
        );
        IssueType::from_json(&json).expect("fixture parses")
    }

    /// A config whose single delegator references a remote agent on `platform`.
    fn config_with_remote_delegator(name: &str, platform: &str, id: &str) -> Config {
        let mut config = Config::default();
        config.delegators.push(crate::config::Delegator {
            name: name.to_string(),
            llm_tool: "anthropic".to_string(),
            model: "claude-3-5-sonnet".to_string(),
            display_name: None,
            model_properties: std::collections::HashMap::new(),
            model_server: None,
            launch_config: None,
            remote_agent: Some(crate::config::RemoteAgentRef {
                platform: platform.to_string(),
                id: id.to_string(),
            }),
            x_agnt: None,
            x_openai: None,
            unmapped_core: None,
        });
        config
    }

    #[test]
    fn agnt_export_emits_agnt_agent_node_for_agnt_delegator() {
        let it = issuetype_with_step_agent("agnt-researcher");
        let out = export_workflow_agnt(
            &ticket("AG"),
            &it,
            None,
            &PipelineEnv::default(),
            &config_with_remote_delegator("agnt-researcher", "agnt", "agent-uuid-123"),
        )
        .expect("export");
        let v: Value = serde_json::from_str(&out).unwrap();
        let node = &v["nodes"][0];
        assert_eq!(
            node["type"], AGNT_AGENT_NODE_TYPE,
            "step bound to an AGNT-referencing delegator must emit an agnt-agent node"
        );
        assert_eq!(
            node["parameters"]["agentId"], "agent-uuid-123",
            "the agnt-agent node carries the referenced AGNT agent id"
        );
        assert!(
            node["parameters"]["message"].is_string(),
            "the agnt-agent node carries the prompt as 'message'"
        );
    }

    #[test]
    fn agnt_export_skips_agnt_node_for_non_agnt_remote_delegator() {
        // An OpenAI remote agent is export-only too, but it has no AGNT workflow
        // analog — so the export must NOT emit an agnt-agent node for it. Proves
        // the export branch is keyed on platform, not just "is remote".
        let it = issuetype_with_step_agent("openai-reviewer");
        let out = export_workflow_agnt(
            &ticket("AG"),
            &it,
            None,
            &PipelineEnv::default(),
            &config_with_remote_delegator("openai-reviewer", "openai", "asst_abc123"),
        )
        .expect("export");
        let v: Value = serde_json::from_str(&out).unwrap();
        assert_eq!(
            v["nodes"][0]["type"], NODE_TYPE,
            "a non-AGNT remote delegator gets no agnt-agent node"
        );
        assert!(
            v["nodes"][0]["parameters"].get("agentId").is_none(),
            "non-AGNT node carries no agentId"
        );
    }

    #[test]
    fn agnt_export_keeps_operator_run_step_for_normal_delegator() {
        // Same issuetype, but the delegator the step names has no remote reference,
        // so the node stays an ordinary operator-run-step.
        let it = issuetype_with_step_agent("local-claude");
        let mut config = Config::default();
        config.delegators.push(crate::config::Delegator {
            name: "local-claude".to_string(),
            llm_tool: "claude".to_string(),
            model: "opus".to_string(),
            display_name: None,
            model_properties: std::collections::HashMap::new(),
            model_server: None,
            launch_config: None,
            remote_agent: None,
            x_agnt: None,
            x_openai: None,
            unmapped_core: None,
        });
        let out = export_workflow_agnt(&ticket("AG"), &it, None, &PipelineEnv::default(), &config)
            .expect("export");
        let v: Value = serde_json::from_str(&out).unwrap();
        assert_eq!(
            v["nodes"][0]["type"], NODE_TYPE,
            "a normal delegator keeps the operator-run-step node type"
        );
        assert!(
            v["nodes"][0]["parameters"].get("agentId").is_none(),
            "non-AGNT node carries no agentId"
        );
    }
}

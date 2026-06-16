#![allow(dead_code)] // Public surface consumed by the `workflow export` CLI command.

//! Export an operator ticket + issuetype to a Claude Code "dynamic workflow"
//! `.js` file.
//!
//! This is **export-only**: operator never parses `.js` back. The unit of
//! export is a ticket *and* its issuetype together — the issuetype supplies the
//! step structure, the ticket supplies the concrete field values. Rendering
//! them produces a workflow specialized to that exact ticket.
//!
//! The mapping is a lossy, autonomous *flattening*, not an equivalence:
//! human review gates (`review_type` / `on_reject`) have no workflow analog
//! (workflows forbid mid-run human input), so they are emitted as autonomous
//! judge-agent retry loops marked with `OPERATOR-GAP`. `rag`/`mcp` steps depend
//! on capabilities the workflow sandbox lacks and are likewise marked.

mod agnt;
mod command;
mod export;
mod format;
pub mod js;
mod step_map;

pub use agnt::export_workflow_agnt;
pub use command::{export_workflow_for_issuetype, export_workflow_for_ticket, ExportedWorkflow};
pub use export::export_workflow;
pub use format::WorkflowFormat;

/// Marker emitted into generated workflows wherever an operator concept could
/// not be faithfully represented.
pub const GAP_MARKER: &str = "OPERATOR-GAP";

#[cfg(test)]
mod tests {
    use super::export::PipelineEnv;
    use super::*;
    use crate::issuetypes::IssueType;
    use crate::queue::Ticket;

    /// Build a ticket with the given id/project/summary; other fields fixed.
    fn ticket(id: &str, project: &str, summary: &str) -> Ticket {
        Ticket {
            filename: "20241221-1430-FEAT-proj-test.md".to_string(),
            filepath: "/test/path".to_string(),
            timestamp: "20241221-1430".to_string(),
            ticket_type: "FEAT".to_string(),
            project: project.to_string(),
            id: id.to_string(),
            summary: summary.to_string(),
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

    /// Build an issuetype with the given steps JSON array.
    fn issuetype(steps_json: &str) -> IssueType {
        let json = format!(
            r#"{{
                "key": "FEAT",
                "name": "Feature",
                "description": "A new feature",
                "mode": "autonomous",
                "glyph": "*",
                "fields": [],
                "steps": {steps_json}
            }}"#
        );
        IssueType::from_json(&json).expect("valid issuetype json")
    }

    fn export(steps_json: &str) -> String {
        let it = issuetype(steps_json);
        let tk = ticket("FEAT-1234", "gamesvc", "Add pagination");
        export_workflow(&tk, &it, None, &PipelineEnv::default()).expect("export ok")
    }

    #[test]
    fn js_escape_handles_quotes_backslashes_and_newlines() {
        assert_eq!(js::escape_str("a\"b\\c\nd\te"), "a\\\"b\\\\c\\nd\\te");
    }

    #[test]
    fn task_step_emits_agent_with_interpolated_prompt_and_phase() {
        let out = export(
            r#"[{"name":"plan","display_name":"Planning","outputs":["plan"],
                 "prompt":"Work on {{ id }} in {{ project }}"}]"#,
        );
        assert!(
            out.contains(r#"agent("Work on FEAT-1234 in gamesvc""#),
            "interpolated agent call missing:\n{out}"
        );
        assert!(
            out.contains(r#"phase("Planning")"#),
            "phase missing:\n{out}"
        );
        assert!(!out.contains("{{"), "unrendered handlebars left in:\n{out}");
    }

    #[test]
    fn classifier_step_emits_schema_opt() {
        let out = export(
            r#"[{"name":"triage","type":"classifier","outputs":["report"],
                 "prompt":"Is this a bug?","classifier_config":{"output_type":"boolean"}}]"#,
        );
        assert!(out.contains("schema:"), "classifier schema missing:\n{out}");
    }

    #[test]
    fn multi_model_step_emits_parallel_then_vote() {
        let out = export(
            r#"[{"name":"consensus","type":"multi_model","outputs":["code"],
                 "prompt":"Solve {{ id }}",
                 "multi_model_config":{"delegators":["opus","sonnet"],"voting_strategy":"majority"}}]"#,
        );
        assert!(out.contains("parallel(["), "parallel missing:\n{out}");
        // The naiveworkflow compiler only walks recognized statements; a
        // `parallel(...).then(...)` chain is silently dropped. Capture the
        // fan-out in an intermediate binding, then vote with a plain agent().
        assert!(
            !out.contains(".then("),
            "must not emit a .then() chain (compiler drops it):\n{out}"
        );
        assert!(
            out.contains("_answers = await parallel(["),
            "fan-out result not captured in an intermediate binding:\n{out}"
        );
        assert!(out.contains("consensus-vote"), "vote agent missing:\n{out}");
    }

    #[test]
    fn multi_prompt_step_emits_parallel_and_select() {
        let out = export(
            r#"[{"name":"explore","type":"multi_prompt","outputs":["plan"],
                 "prompt":"base",
                 "multi_prompt_config":{"prompt_variations":["angle A","angle B"],"selection_strategy":"model_choice"}}]"#,
        );
        assert!(out.contains("parallel(["), "parallel missing:\n{out}");
        assert!(
            !out.contains(".then("),
            "must not emit a .then() chain (compiler drops it):\n{out}"
        );
        assert!(
            out.contains("_outputs = await parallel(["),
            "fan-out result not captured in an intermediate binding:\n{out}"
        );
        assert!(
            out.contains("angle A") && out.contains("angle B"),
            "variations missing:\n{out}"
        );
    }

    #[test]
    fn matrixed_step_emits_nested_parallel() {
        let out = export(
            r#"[{"name":"matrix","type":"matrixed","outputs":["code"],
                 "prompt":"base",
                 "matrixed_config":{"delegators":["opus","sonnet"],"prompt_variations":["v1","v2"],"output_format":"structured"}}]"#,
        );
        // Nested parallel: at least two occurrences.
        assert!(
            out.matches("parallel(").count() >= 2,
            "expected nested parallel:\n{out}"
        );
    }

    #[test]
    fn rag_and_mcp_steps_emit_gap_markers() {
        let rag = export(
            r#"[{"name":"gather","type":"rag","outputs":["report"],"prompt":"summarize",
                 "rag_config":{"sources":[{"type":"glob","pattern":"src/**/*.rs"}]}}]"#,
        );
        assert!(rag.contains(GAP_MARKER), "rag gap marker missing:\n{rag}");

        let mcp = export(
            r#"[{"name":"lookup","type":"mcp","outputs":["report"],"prompt":"check",
                 "mcp_config":{"required_tools":[{"server":"jira","tool":"search"}]}}]"#,
        );
        assert!(mcp.contains(GAP_MARKER), "mcp gap marker missing:\n{mcp}");
    }

    #[test]
    fn review_gate_emits_judge_loop_and_gap_marker() {
        let out = export(
            r#"[{"name":"plan","outputs":["plan"],"prompt":"make a plan",
                 "review_type":"plan",
                 "on_reject":{"goto_step":"plan","prompt":"redo"},
                 "next_step":"ship"},
                {"name":"ship","outputs":["pr"],"prompt":"ship it"}]"#,
        );
        assert!(out.contains(GAP_MARKER), "gap marker missing:\n{out}");
        assert!(
            out.contains("do {") && out.contains("} while"),
            "judge loop missing:\n{out}"
        );
        assert!(out.contains("log("), "log marker missing:\n{out}");
    }

    #[test]
    fn output_has_provenance_header_and_meta() {
        let out = export(r#"[{"name":"plan","outputs":["plan"],"prompt":"go"}]"#);
        assert!(
            out.contains("OPERATOR PROVENANCE"),
            "provenance header missing:\n{out}"
        );
        assert!(
            out.contains("FEAT-1234"),
            "ticket id missing from provenance:\n{out}"
        );
        assert!(
            out.contains("export const meta"),
            "meta block missing:\n{out}"
        );
        // Compiler-native format: steps are emitted as top-level statements
        // (the naiveworkflow compiler only walks the program body), NOT wrapped
        // in `export default async function`, which would hide the whole body.
        assert!(
            !out.contains("export default async function"),
            "must not wrap the body in a default-export function:\n{out}"
        );
        assert!(
            out.contains("await phase("),
            "top-level phase statement missing:\n{out}"
        );
    }

    #[test]
    fn output_is_deterministic_and_has_no_wallclock() {
        let it = issuetype(r#"[{"name":"plan","outputs":["plan"],"prompt":"go {{ id }}"}]"#);
        let tk = ticket("FEAT-1234", "gamesvc", "x");
        let a = export_workflow(&tk, &it, None, &PipelineEnv::default()).unwrap();
        let b = export_workflow(&tk, &it, None, &PipelineEnv::default()).unwrap();
        assert_eq!(a, b, "export is not deterministic");
        assert!(!a.contains("Date.now"), "must not emit Date.now");
        assert!(!a.contains("Math.random"), "must not emit Math.random");
    }

    #[test]
    fn prompt_with_special_chars_is_escaped_into_valid_literal() {
        let out = export(
            r#"[{"name":"plan","outputs":["plan"],"prompt":"say \"hi\"\nuse `x` and ${y}"}]"#,
        );
        // The double quotes inside the prompt must be backslash-escaped so the
        // agent("...") literal is not terminated early.
        assert!(
            out.contains(r#"say \"hi\""#),
            "inner quotes not escaped:\n{out}"
        );
        assert!(out.contains("\\n"), "newline not escaped:\n{out}");
        // Backtick and ${ are safe inside a double-quoted literal and pass through.
        assert!(
            out.contains("`x`") && out.contains("${y}"),
            "backtick/dollar mangled:\n{out}"
        );
    }

    // ── Pipeline step emission ──────────────────────────────────────

    #[test]
    fn pipeline_static_emits_literal_items_and_stage_thunks() {
        let out = export(
            r#"[{"name":"triage","display_name":"Per-project triage","type":"pipeline","outputs":["report"],
                 "prompt":"",
                 "pipeline_config":{
                   "item_source":{"type":"static","items":["alpha","beta"]},
                   "stages":[{"prompt":"Triage {{ id }}"}]}}]"#,
        );
        // Top-level const binding over the pipeline call, items as a literal array.
        assert!(
            out.contains(r#"const r_triage = await pipeline(["alpha", "beta"],"#),
            "pipeline call with literal items missing:\n{out}"
        );
        // Stage is a (prev, item, i) thunk around agent().
        assert!(
            out.contains("(prev, item, i) => agent("),
            "stage thunk missing:\n{out}"
        );
        // The handlebars-rendered prompt with the item binding structurally appended.
        assert!(
            out.contains(r#"agent("Triage FEAT-1234" + "\n\nItem: " + JSON.stringify(item)"#),
            "item binding not appended to stage prompt:\n{out}"
        );
        // No .then() chains (compiler drops them).
        assert!(!out.contains(".then("), "must not emit .then():\n{out}");
    }

    #[test]
    fn pipeline_stage_opts_carry_label_and_phase() {
        let out = export(
            r#"[{"name":"triage","display_name":"Per-project triage","type":"pipeline","outputs":["report"],
                 "prompt":"",
                 "pipeline_config":{
                   "item_source":{"type":"static","items":["a"]},
                   "stages":[{"prompt":"x"}]}}]"#,
        );
        // Default label <step>:<index>; explicit phase opt so stage agents group
        // under this step's phase instead of racing the global phase() state.
        assert!(
            out.contains(r#"label: "triage:0""#),
            "default stage label missing:\n{out}"
        );
        assert!(
            out.contains(r#"phase: "Per-project triage""#),
            "explicit phase opt missing:\n{out}"
        );
    }

    #[test]
    fn pipeline_later_stages_append_prev_but_first_does_not() {
        let out = export(
            r#"[{"name":"sweep","type":"pipeline","outputs":["report"],
                 "prompt":"",
                 "pipeline_config":{
                   "item_source":{"type":"static","items":["a"]},
                   "stages":[{"prompt":"first"},{"prompt":"second"}]}}]"#,
        );
        // Stage 0: prev IS the item (Workflow-tool contract), so no prev append.
        assert!(
            out.contains(r#"agent("first" + "\n\nItem: " + JSON.stringify(item), {"#),
            "stage 0 should append only the item:\n{out}"
        );
        // Stage 1+: previous stage's output appended.
        assert!(
            out.contains(
                r#"agent("second" + "\n\nItem: " + JSON.stringify(item) + "\n\nPrior stage output:\n" + JSON.stringify(prev), {"#
            ),
            "stage 1 should append item and prev:\n{out}"
        );
    }

    #[test]
    fn pipeline_from_step_emits_prior_result_identifier() {
        let out = export(
            r#"[{"name":"find","type":"classifier","outputs":["report"],
                 "prompt":"List affected modules","next_step":"fix",
                 "classifier_config":{"output_type":"big_text"}},
                {"name":"fix","type":"pipeline","outputs":["code"],
                 "prompt":"",
                 "pipeline_config":{
                   "item_source":{"type":"from_step","step":"find"},
                   "stages":[{"prompt":"Fix the module"}]}}]"#,
        );
        // Items expression is the prior step's result var — a runtime value, so
        // the compiled graph shows a symbolic (not static) fan-out width.
        assert!(
            out.contains("const r_fix = await pipeline(r_find,"),
            "from_step identifier items missing:\n{out}"
        );
        // Operator cannot statically check that the prior step returns an
        // array — that gap must be marked.
        assert!(
            out.contains(GAP_MARKER) && out.contains("array"),
            "array-ness GAP marker missing:\n{out}"
        );
    }

    /// Export with an explicit pipeline environment (projects / glob root).
    fn export_with_env(steps_json: &str, env: &PipelineEnv) -> String {
        let it = issuetype(steps_json);
        let tk = ticket("FEAT-1234", "gamesvc", "Add pagination");
        export_workflow(&tk, &it, None, env).expect("export ok")
    }

    const PROJECTS_PIPELINE: &str = r#"[{"name":"sweep","type":"pipeline","outputs":["report"],
         "prompt":"",
         "pipeline_config":{
           "item_source":{"type":"projects"},
           "stages":[{"prompt":"Check the project"}]}}]"#;

    #[test]
    fn pipeline_projects_emits_sorted_project_list_from_env() {
        let env = PipelineEnv {
            projects: vec!["svc-b".to_string(), "svc-a".to_string()],
            glob_root: None,
        };
        let out = export_with_env(PROJECTS_PIPELINE, &env);
        // Sorted for determinism regardless of discovery (read_dir) order.
        assert!(
            out.contains(r#"const r_sweep = await pipeline(["svc-a", "svc-b"],"#),
            "sorted project items missing:\n{out}"
        );
    }

    #[test]
    fn pipeline_projects_without_env_emits_gap_placeholder() {
        let out = export_with_env(PROJECTS_PIPELINE, &PipelineEnv::default());
        // No environment (e.g. issuetype preview): runtime-safe empty binding,
        // passed as an identifier so the compiled graph shows symbolic width.
        assert!(
            out.contains(GAP_MARKER) && out.contains("const r_sweep_items = [];"),
            "GAP placeholder missing:\n{out}"
        );
        assert!(
            out.contains("await pipeline(r_sweep_items,"),
            "placeholder identifier not passed to pipeline:\n{out}"
        );
    }

    #[test]
    fn pipeline_glob_expands_relative_to_root_sorted() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join("b.md"), "b").unwrap();
        std::fs::write(dir.path().join("a.md"), "a").unwrap();
        std::fs::write(dir.path().join("skip.txt"), "no").unwrap();
        let env = PipelineEnv {
            projects: vec![],
            glob_root: Some(dir.path().to_path_buf()),
        };
        let out = export_with_env(
            r#"[{"name":"docs","type":"pipeline","outputs":["report"],
                 "prompt":"",
                 "pipeline_config":{
                   "item_source":{"type":"glob","pattern":"*.md"},
                   "stages":[{"prompt":"Review the file"}]}}]"#,
            &env,
        );
        // Matches emitted relative to the glob root (host-absolute paths would
        // leak machine state into the artifact), sorted for determinism.
        assert!(
            out.contains(r#"const r_docs = await pipeline(["a.md", "b.md"],"#),
            "glob-expanded items missing:\n{out}"
        );
    }

    #[test]
    fn pipeline_glob_without_root_emits_gap_placeholder() {
        let out = export_with_env(
            r#"[{"name":"docs","type":"pipeline","outputs":["report"],
                 "prompt":"",
                 "pipeline_config":{
                   "item_source":{"type":"glob","pattern":"*.md"},
                   "stages":[{"prompt":"Review"}]}}]"#,
            &PipelineEnv::default(),
        );
        assert!(
            out.contains(GAP_MARKER) && out.contains("await pipeline(r_docs_items,"),
            "glob GAP placeholder missing:\n{out}"
        );
    }

    #[test]
    fn pipeline_field_always_emits_gap_placeholder() {
        // Field resolution is deferred: ticket field values are not captured
        // at export time (no list FieldType exists yet).
        let env = PipelineEnv {
            projects: vec!["p".to_string()],
            glob_root: None,
        };
        let out = export_with_env(
            r#"[{"name":"perf","type":"pipeline","outputs":["report"],
                 "prompt":"",
                 "pipeline_config":{
                   "item_source":{"type":"field","name":"modules"},
                   "stages":[{"prompt":"Profile"}]}}]"#,
            &env,
        );
        assert!(
            out.contains(GAP_MARKER) && out.contains("await pipeline(r_perf_items,"),
            "field GAP placeholder missing:\n{out}"
        );
    }

    /// Companion to `dumps_all_builtin_previews_for_compiler_check`: no builtin
    /// issuetype has a pipeline step yet, so dump a representative pipeline
    /// workflow (static items, `from_step` items, two stages) for the
    /// naiveworkflow-compiler verification pass.
    #[test]
    fn dumps_pipeline_sample_for_compiler_check() {
        let env = PipelineEnv {
            projects: vec!["svc-b".to_string(), "svc-a".to_string()],
            glob_root: None,
        };
        let out = export_with_env(
            r#"[{"name":"find","type":"classifier","outputs":["report"],
                 "prompt":"List affected projects","next_step":"sweep",
                 "classifier_config":{"output_type":"big_text"}},
                {"name":"sweep","display_name":"Per-project sweep","type":"pipeline","outputs":["report"],
                 "prompt":"","next_step":"fix",
                 "pipeline_config":{
                   "item_source":{"type":"projects"},
                   "stages":[{"prompt":"Audit the project"},{"prompt":"Summarize findings"}]}},
                {"name":"fix","type":"pipeline","outputs":["code"],
                 "prompt":"",
                 "pipeline_config":{
                   "item_source":{"type":"from_step","step":"find"},
                   "stages":[{"prompt":"Apply the fix"}]}}]"#,
            &env,
        );
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join("workflow-previews");
        std::fs::create_dir_all(&dir).expect("create preview dir");
        std::fs::write(dir.join("PIPELINE-sample.workflow.js"), &out).expect("write sample");
        assert!(out.contains("await pipeline("), "sample missing pipeline");
    }

    #[test]
    fn pipeline_stage_model_and_schema_opts() {
        let out = export(
            r#"[{"name":"triage","type":"pipeline","outputs":["report"],
                 "prompt":"",
                 "pipeline_config":{
                   "item_source":{"type":"static","items":["a"]},
                   "stages":[{"prompt":"x","model":"opus",
                              "jsonSchema":{"type":"object","required":["value"]}}]}}]"#,
        );
        assert!(
            out.contains(r#"model: "opus""#),
            "stage model opt missing:\n{out}"
        );
        assert!(
            out.contains(r#"schema: {"required":["value"],"type":"object"}"#)
                || out.contains(r#"schema: {"type":"object","required":["value"]}"#),
            "stage schema opt missing:\n{out}"
        );
    }
}

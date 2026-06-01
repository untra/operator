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

mod command;
mod export;
pub mod js;
mod step_map;

pub use command::{export_workflow_for_issuetype, export_workflow_for_ticket, ExportedWorkflow};
pub use export::export_workflow;

/// Marker emitted into generated workflows wherever an operator concept could
/// not be faithfully represented.
pub const GAP_MARKER: &str = "OPERATOR-GAP";

#[cfg(test)]
mod tests {
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
        export_workflow(&tk, &it, None).expect("export ok")
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
        let a = export_workflow(&tk, &it, None).unwrap();
        let b = export_workflow(&tk, &it, None).unwrap();
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
}

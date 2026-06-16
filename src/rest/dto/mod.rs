//! Data Transfer Objects for the REST API.
//!
//! Organized by domain:
//! - `issue_types`: `IssueType`, `Field`, `Step`, `Collection` DTOs
//! - `kanban`: Kanban onboarding, board, and sync DTOs
//! - `agents`: Agent lifecycle, launch, step execution, and review DTOs
//! - `configuration`: `Delegator`, model server, LLM tool, and project DTOs

pub mod agents;
pub mod configuration;
pub mod issue_types;
pub mod kanban;
pub mod sections;
pub mod tickets;
pub mod workflow;

pub use agents::*;
pub use configuration::*;
pub use issue_types::*;
pub use kanban::*;
pub use sections::*;
pub use tickets::*;
pub use workflow::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_issue_type_request_into() {
        let req = CreateIssueTypeRequest {
            key: "test".to_string(),
            name: "Test".to_string(),
            description: "A test type".to_string(),
            mode: "autonomous".to_string(),
            glyph: "T".to_string(),
            color: None,
            project_required: true,
            fields: vec![],
            steps: vec![CreateStepRequest {
                name: "execute".to_string(),
                display_name: None,
                prompt: "Do the thing".to_string(),
                outputs: vec![],
                allowed_tools: vec!["*".to_string()],
                review_type: "none".to_string(),
                next_step: None,
                permission_mode: "default".to_string(),
            }],
        };

        let it = req.into_issue_type();
        assert_eq!(it.key, "TEST"); // Uppercased
        assert_eq!(it.name, "Test");
        assert!(matches!(
            it.mode,
            crate::templates::schema::ExecutionMode::Autonomous
        ));
        assert!(matches!(
            it.source,
            crate::issuetypes::schema::IssueTypeSource::User
        ));
        assert_eq!(it.steps.len(), 1);
    }

    #[test]
    fn test_issue_type_response_from() {
        let it = crate::issuetypes::IssueType::new_imported(
            "TEST".to_string(),
            "Test".to_string(),
            "A test".to_string(),
            "jira".to_string(),
            "PROJ".to_string(),
            None,
        );

        let resp = IssueTypeResponse::from(&it);
        assert_eq!(resp.key, "TEST");
        assert_eq!(resp.mode, "autonomous");
        assert_eq!(resp.source, "jira/PROJ");
    }

    #[test]
    fn test_operator_output_default() {
        let output = OperatorOutput::default();
        assert_eq!(output.status, "");
        assert!(!output.exit_signal);
        assert!(output.confidence.is_none());
        assert!(output.summary.is_none());
    }

    #[test]
    fn test_operator_output_serialization() {
        let output = OperatorOutput {
            status: "complete".to_string(),
            exit_signal: true,
            confidence: Some(95),
            files_modified: Some(3),
            tests_status: Some("passing".to_string()),
            error_count: Some(0),
            tasks_completed: Some(5),
            tasks_remaining: Some(0),
            summary: Some("Implemented feature".to_string()),
            recommendation: Some("Ready for review".to_string()),
            blockers: None,
        };

        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("\"status\":\"complete\""));
        assert!(json.contains("\"exit_signal\":true"));
        assert!(json.contains("\"confidence\":95"));
        assert!(!json.contains("blockers")); // None fields are skipped
    }

    #[test]
    fn test_operator_output_deserialization() {
        let json = r#"{
            "status": "in_progress",
            "exit_signal": false,
            "confidence": 60,
            "files_modified": 2,
            "tests_status": "failing",
            "summary": "Working on tests"
        }"#;

        let output: OperatorOutput = serde_json::from_str(json).unwrap();
        assert_eq!(output.status, "in_progress");
        assert!(!output.exit_signal);
        assert_eq!(output.confidence, Some(60));
        assert_eq!(output.tests_status, Some("failing".to_string()));
    }

    #[test]
    fn test_step_complete_request_with_operator_output() {
        let output = OperatorOutput {
            status: "complete".to_string(),
            exit_signal: true,
            confidence: Some(90),
            ..Default::default()
        };

        let request = StepCompleteRequest {
            exit_code: 0,
            output_valid: true,
            output_schema_errors: None,
            session_id: Some("session-123".to_string()),
            duration_secs: 300,
            output_sample: None,
            output: Some(output),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"exit_code\":0"));
        assert!(json.contains("\"output\":{"));
        assert!(json.contains("\"status\":\"complete\""));
    }

    #[test]
    fn test_launch_response_cmux_fields_present_when_set() {
        let resp = LaunchTicketResponse {
            agent_id: "a1".to_string(),
            ticket_id: "FEAT-001".to_string(),
            working_directory: "/tmp".to_string(),
            command: "claude".to_string(),
            terminal_name: "op-FEAT-001".to_string(),
            tmux_session_name: "op-FEAT-001".to_string(),
            session_wrapper: Some("cmux".to_string()),
            session_window_ref: Some("win-1".to_string()),
            session_context_ref: Some("ws-1".to_string()),
            session_id: "uuid-1".to_string(),
            worktree_created: false,
            branch: None,
        };

        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"session_wrapper\":\"cmux\""));
        assert!(json.contains("\"session_window_ref\":\"win-1\""));
        assert!(json.contains("\"session_context_ref\":\"ws-1\""));
    }

    #[test]
    fn test_launch_response_cmux_fields_absent_when_none() {
        let resp = LaunchTicketResponse {
            agent_id: "a1".to_string(),
            ticket_id: "FEAT-001".to_string(),
            working_directory: "/tmp".to_string(),
            command: "claude".to_string(),
            terminal_name: "op-FEAT-001".to_string(),
            tmux_session_name: "op-FEAT-001".to_string(),
            session_wrapper: None,
            session_window_ref: None,
            session_context_ref: None,
            session_id: "uuid-1".to_string(),
            worktree_created: false,
            branch: None,
        };

        let json = serde_json::to_string(&resp).unwrap();
        assert!(!json.contains("session_wrapper"));
        assert!(!json.contains("session_window_ref"));
        assert!(!json.contains("session_context_ref"));
    }

    #[test]
    fn test_step_complete_response_with_analysis_fields() {
        let json = r#"{
            "status": "completed",
            "auto_proceed": true,
            "output_valid": true,
            "should_iterate": false,
            "iteration_count": 1,
            "circuit_state": "closed",
            "previous_summary": "Built feature",
            "cumulative_files_modified": 5,
            "cumulative_errors": 0
        }"#;

        let response: StepCompleteResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.status, "completed");
        assert!(response.output_valid);
        assert!(!response.should_iterate);
        assert_eq!(response.iteration_count, 1);
        assert_eq!(response.circuit_state, "closed");
        assert_eq!(response.previous_summary, Some("Built feature".to_string()));
    }
}

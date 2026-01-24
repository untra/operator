mod api;
mod cli;
mod output_parser;
mod runner;
mod transition;

use api::{ApiClient, StepCompleteRequest};
use cli::Args;
use runner::{dry_run_command, run_command, RunConfig};
use std::process::ExitCode;
use transition::{
    exec_next_command, print_api_unreachable_error, print_auto_proceed_disabled,
    print_awaiting_review, print_command_failed, print_step_completed, print_step_starting,
    print_workflow_complete,
};

/// Exit codes
const EXIT_SUCCESS: u8 = 0;
const EXIT_LLM_FAILED: u8 = 1;
const EXIT_API_UNREACHABLE: u8 = 3;
const EXIT_CONFIG_ERROR: u8 = 4;

/// Determine the exit code based on command result and API response status
fn determine_exit_code(command_exit_code: i32, response_status: &str) -> u8 {
    match response_status {
        "failed" => EXIT_LLM_FAILED,
        _ => {
            if command_exit_code != 0 {
                EXIT_LLM_FAILED
            } else {
                EXIT_SUCCESS
            }
        }
    }
}

/// Build the StepCompleteRequest from run results
fn build_step_complete_request(
    exit_code: i32,
    session_id: Option<String>,
    duration_secs: u64,
    output: Option<api::OperatorOutput>,
) -> StepCompleteRequest {
    StepCompleteRequest {
        exit_code,
        session_id,
        duration_secs,
        output,
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> ExitCode {
    let args = Args::parse_args();

    // Validate command
    let (program, cmd_args) = match args.command_parts() {
        Some(parts) => parts,
        None => {
            eprintln!("[opr8r] Error: No command specified");
            return ExitCode::from(EXIT_CONFIG_ERROR);
        }
    };

    // Handle dry run
    if args.dry_run {
        dry_run_command(program, cmd_args);
        println!("[opr8r dry-run] Would report to API:");
        println!("  Ticket: {}", args.ticket_id);
        println!("  Step: {}", args.step);
        println!(
            "  API URL: {}",
            args.api_url.as_deref().unwrap_or("auto-discover")
        );
        return ExitCode::SUCCESS;
    }

    print_step_starting(&args.ticket_id, &args.step, args.verbose);

    // Configure runner with output capture enabled
    let config = RunConfig::new()
        .with_verbose(args.verbose)
        .with_capture(true); // Always capture to parse OPERATOR_STATUS blocks

    // Run the LLM command
    let run_result = match run_command(program, cmd_args, config).await {
        Ok(result) => result,
        Err(e) => {
            eprintln!("[opr8r] Error: Failed to run command: {}", e);
            return ExitCode::from(EXIT_CONFIG_ERROR);
        }
    };

    let exit_code = run_result.exit_status.code().unwrap_or(1);
    let duration_secs = run_result.duration.as_secs();

    // Parse OPERATOR_STATUS block from captured output
    let operator_output = run_result
        .captured_output
        .as_deref()
        .and_then(output_parser::find_last_status_block)
        .map(api::OperatorOutput::from);

    if args.verbose {
        if let Some(ref output) = operator_output {
            eprintln!(
                "[opr8r] Parsed operator output: status={}, exit_signal={}",
                output.status, output.exit_signal
            );
        } else {
            eprintln!("[opr8r] No OPERATOR_STATUS block found in output");
        }
    }

    print_step_completed(&args.step, duration_secs, args.verbose);

    // Print command failure if applicable
    if exit_code != 0 {
        print_command_failed(exit_code, &args.step);
    }

    // Discover and connect to API
    let api_client = match ApiClient::discover(args.api_url.as_deref()).await {
        Ok(client) => client,
        Err(e) => {
            print_api_unreachable_error(&e.to_string());
            return ExitCode::from(EXIT_API_UNREACHABLE);
        }
    };

    // Report completion to API with operator output
    let request = build_step_complete_request(
        exit_code,
        args.session_id.clone(),
        duration_secs,
        operator_output,
    );

    let response = match api_client
        .complete_step(&args.ticket_id, &args.step, request)
        .await
    {
        Ok(r) => r,
        Err(e) => {
            print_api_unreachable_error(&e.to_string());
            return ExitCode::from(EXIT_API_UNREACHABLE);
        }
    };

    // Handle response based on status
    match response.status.as_str() {
        "completed" => {
            // Check if there's a next step
            if let Some(next_command) = response.next_command {
                if response.auto_proceed && !args.no_auto_proceed {
                    // exec() to next step - this will not return
                    exec_next_command(&next_command, args.verbose);
                } else {
                    print_auto_proceed_disabled();
                }
            } else {
                // No next step - workflow complete
                print_workflow_complete(&args.ticket_id);
            }
        }
        "awaiting_review" => {
            // Step requires review
            let review_type = response
                .next_step
                .as_ref()
                .map(|s| s.review_type.as_str())
                .unwrap_or("unknown");
            print_awaiting_review(&args.step, review_type);
        }
        "failed" => {
            // Step failed
            return ExitCode::from(EXIT_LLM_FAILED);
        }
        _ => {
            if args.verbose {
                eprintln!("[opr8r] Unknown status: {}", response.status);
            }
        }
    }

    // Return appropriate exit code based on command result and response status
    ExitCode::from(determine_exit_code(exit_code, &response.status))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_determine_exit_code_success() {
        assert_eq!(determine_exit_code(0, "completed"), EXIT_SUCCESS);
        assert_eq!(determine_exit_code(0, "awaiting_review"), EXIT_SUCCESS);
    }

    #[test]
    fn test_determine_exit_code_command_failed() {
        // Non-zero exit code should return EXIT_LLM_FAILED
        assert_eq!(determine_exit_code(1, "completed"), EXIT_LLM_FAILED);
        assert_eq!(determine_exit_code(127, "completed"), EXIT_LLM_FAILED);
    }

    #[test]
    fn test_determine_exit_code_response_failed() {
        // API response "failed" should return EXIT_LLM_FAILED
        assert_eq!(determine_exit_code(0, "failed"), EXIT_LLM_FAILED);
    }

    #[test]
    fn test_determine_exit_code_unknown_status() {
        // Unknown status with successful command should still succeed
        assert_eq!(determine_exit_code(0, "unknown_status"), EXIT_SUCCESS);
    }

    #[test]
    fn test_build_step_complete_request_minimal() {
        let request = build_step_complete_request(0, None, 120, None);
        assert_eq!(request.exit_code, 0);
        assert!(request.session_id.is_none());
        assert_eq!(request.duration_secs, 120);
        assert!(request.output.is_none());
    }

    #[test]
    fn test_build_step_complete_request_with_session() {
        let request = build_step_complete_request(1, Some("session-abc".to_string()), 300, None);
        assert_eq!(request.exit_code, 1);
        assert_eq!(request.session_id, Some("session-abc".to_string()));
        assert_eq!(request.duration_secs, 300);
    }

    #[test]
    fn test_build_step_complete_request_with_operator_output() {
        let output = api::OperatorOutput {
            status: "complete".to_string(),
            exit_signal: true,
            confidence: Some(95),
            ..Default::default()
        };

        let request = build_step_complete_request(0, None, 120, Some(output));
        assert_eq!(request.exit_code, 0);
        assert!(request.output.is_some());
        let output = request.output.unwrap();
        assert_eq!(output.status, "complete");
        assert!(output.exit_signal);
    }
}

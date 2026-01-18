#[cfg(unix)]
use std::os::unix::process::CommandExt;
use std::process::Command;

/// Execute the next step command by replacing the current process
///
/// On Unix, this uses `exec()` to replace the current process with the next command,
/// keeping the same terminal session. This is the key mechanism for seamless
/// step transitions in both tmux and VSCode terminals.
///
/// On Windows, we spawn a new process and exit, since Windows doesn't have exec().
/// This achieves the same effect of running the next command in the same terminal.
#[cfg(unix)]
pub fn exec_next_command(command: &str, verbose: bool) -> ! {
    if verbose {
        eprintln!("[opr8r] Executing next step: {}", command);
    }

    // Parse the command string into parts
    // We use sh -c to handle complex command strings properly
    let err = Command::new("sh").arg("-c").arg(command).exec();

    // exec() only returns on error
    eprintln!("[opr8r] Failed to exec next command: {}", err);
    std::process::exit(1);
}

/// Execute the next step command on Windows
///
/// Since Windows doesn't have exec(), we spawn a new process and exit.
/// This achieves the same user-facing effect of running the next command.
#[cfg(windows)]
pub fn exec_next_command(command: &str, verbose: bool) -> ! {
    if verbose {
        eprintln!("[opr8r] Executing next step: {}", command);
    }

    // On Windows, spawn the command via cmd.exe and wait for it
    let status = Command::new("cmd")
        .args(["/C", command])
        .stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .status();

    match status {
        Ok(exit_status) => {
            // Exit with the same code as the spawned process
            std::process::exit(exit_status.code().unwrap_or(1));
        }
        Err(e) => {
            eprintln!("[opr8r] Failed to exec next command: {}", e);
            std::process::exit(1);
        }
    }
}

/// Print transition message when review is required
pub fn print_awaiting_review(step: &str, review_type: &str) {
    eprintln!();
    eprintln!("=================================================");
    eprintln!(
        " Step '{}' completed - awaiting {} review",
        step, review_type
    );
    eprintln!("=================================================");
    eprintln!();
    eprintln!("The operator will review and approve before proceeding.");
    eprintln!("This terminal session will remain open.");
    eprintln!();
}

/// Print transition message when step completed but no next step
pub fn print_workflow_complete(ticket_id: &str) {
    eprintln!();
    eprintln!("=================================================");
    eprintln!(" Ticket {} workflow complete!", ticket_id);
    eprintln!("=================================================");
    eprintln!();
}

/// Print transition message when auto-proceeding is disabled
pub fn print_auto_proceed_disabled() {
    eprintln!();
    eprintln!("[opr8r] Auto-proceed disabled, exiting.");
    eprintln!("[opr8r] Run the next step manually or use the operator TUI.");
    eprintln!();
}

/// Print error when API is unreachable
pub fn print_api_unreachable_error(error: &str) {
    eprintln!();
    eprintln!("=================================================");
    eprintln!(" ERROR: Could not reach Operator API");
    eprintln!("=================================================");
    eprintln!();
    eprintln!("Error: {}", error);
    eprintln!();
    eprintln!("To recover:");
    eprintln!("  1. Ensure the operator API is running (cargo run -- api)");
    eprintln!("  2. Check .tickets/operator/api-session.json exists");
    eprintln!("  3. Manually advance the ticket via the operator TUI");
    eprintln!();
}

/// Print error when command fails
pub fn print_command_failed(exit_code: i32, step: &str) {
    eprintln!();
    eprintln!("=================================================");
    eprintln!(" Step '{}' failed with exit code {}", step, exit_code);
    eprintln!("=================================================");
    eprintln!();
}

/// Print info about step starting
pub fn print_step_starting(ticket_id: &str, step: &str, verbose: bool) {
    if verbose {
        eprintln!("[opr8r] Starting step '{}' for ticket {}", step, ticket_id);
    }
}

/// Print info about step completion
pub fn print_step_completed(step: &str, duration_secs: u64, verbose: bool) {
    if verbose {
        eprintln!("[opr8r] Step '{}' completed in {}s", step, duration_secs);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_print_functions_dont_panic() {
        // Just verify these functions don't panic
        print_awaiting_review("plan", "plan");
        print_workflow_complete("FEAT-123");
        print_auto_proceed_disabled();
        print_api_unreachable_error("connection refused");
        print_command_failed(1, "build");
        print_step_starting("FEAT-123", "plan", true);
        print_step_starting("FEAT-123", "plan", false);
        print_step_completed("plan", 120, true);
        print_step_completed("plan", 120, false);
    }
}

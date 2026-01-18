use std::io;
use std::process::ExitStatus;
use std::time::{Duration, Instant};
use tokio::process::Command;

/// Result of running a subprocess
#[derive(Debug)]
pub struct RunResult {
    /// Exit status of the process
    pub exit_status: ExitStatus,
    /// Duration the process ran
    pub duration: Duration,
    /// Whether the process was interrupted by signal
    #[allow(dead_code)]
    pub interrupted: bool,
}

/// Configuration for running a subprocess
#[derive(Debug, Default)]
pub struct RunConfig {
    /// Enable verbose logging
    pub verbose: bool,
}

impl RunConfig {
    pub fn new() -> Self {
        Self { verbose: false }
    }

    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }
}

/// Run a command with pure passthrough to terminal
///
/// This spawns the command and passes stdin/stdout/stderr directly through
/// to the current terminal. No capture or buffering is performed.
pub async fn run_command(
    program: &str,
    args: &[String],
    config: RunConfig,
) -> io::Result<RunResult> {
    let start = Instant::now();

    if config.verbose {
        eprintln!("[opr8r] Running: {} {}", program, args.join(" "));
    }

    // Create the command with direct passthrough (no capture)
    let mut child = Command::new(program)
        .args(args)
        .stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .spawn()?;

    // Wait for the process to exit
    let exit_status = child.wait().await?;
    let duration = start.elapsed();

    if config.verbose {
        eprintln!(
            "[opr8r] Process exited with status: {:?} after {:?}",
            exit_status, duration
        );
    }

    Ok(RunResult {
        exit_status,
        duration,
        interrupted: false,
    })
}

/// Run a command in dry-run mode (just print what would happen)
pub fn dry_run_command(program: &str, args: &[String]) {
    println!(
        "[opr8r dry-run] Would execute: {} {}",
        program,
        args.join(" ")
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper to get a simple echo command that works cross-platform
    #[cfg(unix)]
    fn echo_cmd() -> (&'static str, Vec<String>) {
        ("echo", vec!["hello".to_string()])
    }

    #[cfg(windows)]
    fn echo_cmd() -> (&'static str, Vec<String>) {
        (
            "cmd",
            vec!["/C".to_string(), "echo".to_string(), "hello".to_string()],
        )
    }

    // Helper to get a failing command that works cross-platform
    #[cfg(unix)]
    fn failing_cmd() -> (&'static str, Vec<String>) {
        ("false", vec![])
    }

    #[cfg(windows)]
    fn failing_cmd() -> (&'static str, Vec<String>) {
        (
            "cmd",
            vec!["/C".to_string(), "exit".to_string(), "1".to_string()],
        )
    }

    #[tokio::test]
    async fn test_run_simple_command() {
        let (cmd, args) = echo_cmd();
        let result = run_command(cmd, &args, RunConfig::new()).await.unwrap();

        assert!(result.exit_status.success());
        assert!(!result.interrupted);
    }

    #[tokio::test]
    async fn test_run_nonexistent_command() {
        let result = run_command("nonexistent_command_xyz", &[], RunConfig::new()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_run_failing_command() {
        let (cmd, args) = failing_cmd();
        let result = run_command(cmd, &args, RunConfig::new()).await.unwrap();
        assert!(!result.exit_status.success());
    }

    #[test]
    fn test_run_config_builder() {
        let config = RunConfig::new().with_verbose(true);
        assert!(config.verbose);
    }

    #[test]
    fn test_run_config_default() {
        let config = RunConfig::default();
        assert!(!config.verbose);
    }

    #[tokio::test]
    async fn test_run_with_verbose() {
        let config = RunConfig::new().with_verbose(true);
        let (cmd, args) = echo_cmd();
        let result = run_command(cmd, &args, config).await.unwrap();

        assert!(result.exit_status.success());
    }

    #[test]
    fn test_dry_run_command_output() {
        // Just verify it doesn't panic and produces output
        dry_run_command("claude", &["--prompt".to_string(), "test".to_string()]);
    }
}

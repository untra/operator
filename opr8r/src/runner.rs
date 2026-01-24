use std::io::{self, Write};
use std::process::ExitStatus;
use std::time::{Duration, Instant};
use tokio::io::{AsyncBufReadExt, BufReader};
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
    /// Captured output (if capture was enabled)
    pub captured_output: Option<String>,
}

/// Configuration for running a subprocess
#[derive(Debug, Default)]
pub struct RunConfig {
    /// Enable verbose logging
    pub verbose: bool,
    /// Enable output capture (tee mode: capture while displaying)
    pub capture_output: bool,
}

impl RunConfig {
    pub fn new() -> Self {
        Self {
            verbose: false,
            capture_output: false,
        }
    }

    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    pub fn with_capture(mut self, capture: bool) -> Self {
        self.capture_output = capture;
        self
    }
}

/// Run a command with pure passthrough to terminal
///
/// This spawns the command and passes stdin/stdout/stderr directly through
/// to the current terminal. If capture_output is enabled, it uses tee mode
/// to capture output while still displaying to terminal.
pub async fn run_command(
    program: &str,
    args: &[String],
    config: RunConfig,
) -> io::Result<RunResult> {
    if config.verbose {
        eprintln!("[opr8r] Running: {} {}", program, args.join(" "));
    }

    if config.capture_output {
        // Tee mode: capture output while displaying to terminal
        run_with_capture(program, args, config.verbose).await
    } else {
        // Pure passthrough mode: no capture
        run_passthrough(program, args, config.verbose).await
    }
    .map(|(exit_status, duration, captured)| RunResult {
        exit_status,
        duration,
        interrupted: false,
        captured_output: captured,
    })
}

/// Run with pure passthrough (no capture)
async fn run_passthrough(
    program: &str,
    args: &[String],
    verbose: bool,
) -> io::Result<(ExitStatus, Duration, Option<String>)> {
    let start = Instant::now();

    let mut child = Command::new(program)
        .args(args)
        .stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .spawn()?;

    let exit_status = child.wait().await?;
    let duration = start.elapsed();

    if verbose {
        eprintln!(
            "[opr8r] Process exited with status: {:?} after {:?}",
            exit_status, duration
        );
    }

    Ok((exit_status, duration, None))
}

/// Run with output capture (tee mode)
///
/// Captures stdout and stderr while simultaneously displaying to terminal.
/// This allows collecting the full output for parsing while maintaining
/// interactive display for the user.
async fn run_with_capture(
    program: &str,
    args: &[String],
    verbose: bool,
) -> io::Result<(ExitStatus, Duration, Option<String>)> {
    let start = Instant::now();

    let mut child = Command::new(program)
        .args(args)
        .stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()?;

    // Take the stdout and stderr handles
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();

    // Spawn tasks to tee the streams
    let stdout_task = spawn_tee_task(stdout, false);
    let stderr_task = spawn_tee_task(stderr, true);

    // Wait for the process to complete
    let exit_status = child.wait().await?;
    let duration = start.elapsed();

    // Wait for stream tasks to complete and collect output
    let stdout_output = stdout_task.await.unwrap_or_default();
    let stderr_output = stderr_task.await.unwrap_or_default();

    // Combine outputs (stdout first, then stderr)
    let combined = if stderr_output.is_empty() {
        stdout_output
    } else if stdout_output.is_empty() {
        stderr_output
    } else {
        format!("{}\n{}", stdout_output, stderr_output)
    };

    if verbose {
        eprintln!(
            "[opr8r] Process exited with status: {:?} after {:?} (captured {} bytes)",
            exit_status,
            duration,
            combined.len()
        );
    }

    Ok((exit_status, duration, Some(combined)))
}

/// Spawn a task that reads from a stream and tees to terminal while capturing
fn spawn_tee_task(
    stream: Option<impl tokio::io::AsyncRead + Unpin + Send + 'static>,
    is_stderr: bool,
) -> tokio::task::JoinHandle<String> {
    tokio::spawn(async move {
        let Some(stream) = stream else {
            return String::new();
        };

        let mut output = String::new();
        let mut reader = BufReader::new(stream);
        let mut line = String::new();

        loop {
            line.clear();
            match reader.read_line(&mut line).await {
                Ok(0) => break, // EOF
                Ok(_) => {
                    // Tee: write to terminal
                    if is_stderr {
                        let _ = io::stderr().write_all(line.as_bytes());
                        let _ = io::stderr().flush();
                    } else {
                        let _ = io::stdout().write_all(line.as_bytes());
                        let _ = io::stdout().flush();
                    }

                    // Capture the output
                    output.push_str(&line);
                }
                Err(_) => break,
            }
        }

        output
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
        assert!(result.captured_output.is_none()); // No capture by default
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
        assert!(!config.capture_output);
    }

    #[test]
    fn test_run_config_with_capture() {
        let config = RunConfig::new().with_capture(true);
        assert!(config.capture_output);
    }

    #[tokio::test]
    async fn test_run_with_verbose() {
        let config = RunConfig::new().with_verbose(true);
        let (cmd, args) = echo_cmd();
        let result = run_command(cmd, &args, config).await.unwrap();

        assert!(result.exit_status.success());
    }

    #[tokio::test]
    async fn test_run_with_capture() {
        let config = RunConfig::new().with_capture(true);
        let (cmd, args) = echo_cmd();
        let result = run_command(cmd, &args, config).await.unwrap();

        assert!(result.exit_status.success());
        assert!(result.captured_output.is_some());
        let output = result.captured_output.unwrap();
        assert!(output.contains("hello"));
    }

    #[tokio::test]
    async fn test_run_with_capture_and_verbose() {
        let config = RunConfig::new().with_capture(true).with_verbose(true);
        let (cmd, args) = echo_cmd();
        let result = run_command(cmd, &args, config).await.unwrap();

        assert!(result.exit_status.success());
        assert!(result.captured_output.is_some());
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn test_run_capture_multiline_output() {
        let config = RunConfig::new().with_capture(true);
        // Use printf to output multiple lines
        let result = run_command("printf", &["line1\\nline2\\nline3\\n".to_string()], config)
            .await
            .unwrap();

        assert!(result.exit_status.success());
        let output = result.captured_output.unwrap();
        assert!(output.contains("line1"));
        assert!(output.contains("line2"));
        assert!(output.contains("line3"));
    }

    #[test]
    fn test_dry_run_command_output() {
        // Just verify it doesn't panic and produces output
        dry_run_command("claude", &["--prompt".to_string(), "test".to_string()]);
    }
}

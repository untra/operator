//! Deadline-bounded subprocess execution for the launch path.
//!
//! `coder create` and the `ssh` invocations around a remote launch are blocking
//! and previously had no upper bound: a wedged control plane or a half-open
//! connection parked a launch forever, and dropping the HTTP request that
//! started it did not reclaim anything. Every launch-reachable command goes
//! through here so a hang fails the launch instead of leaking a worker.

use std::io::Read;
use std::process::{Child, Command, Output, Stdio};
use std::time::{Duration, Instant};

/// How often to check whether the child has exited.
const POLL_INTERVAL: Duration = Duration::from_millis(25);

/// Run `command` to completion, killing it if it outlives `timeout`.
pub fn output_with_timeout(
    mut command: Command,
    timeout: Duration,
    description: &str,
) -> anyhow::Result<Output> {
    let mut child = command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| anyhow::anyhow!("Failed to start {description}: {error}"))?;

    let stdout = drain(child.stdout.take());
    let stderr = drain(child.stderr.take());

    let status = match wait_with_timeout(&mut child, timeout) {
        Some(status) => status,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            anyhow::bail!(
                "{description} exceeded its {}s timeout and was terminated",
                timeout.as_secs()
            );
        }
    };

    Ok(Output {
        status,
        stdout: stdout.join().unwrap_or_default(),
        stderr: stderr.join().unwrap_or_default(),
    })
}

/// `None` once the deadline passes without the child exiting.
fn wait_with_timeout(child: &mut Child, timeout: Duration) -> Option<std::process::ExitStatus> {
    let deadline = Instant::now() + timeout;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return Some(status),
            Ok(None) => {}
            Err(_) => return None,
        }
        if Instant::now() >= deadline {
            return None;
        }
        std::thread::sleep(POLL_INTERVAL);
    }
}

fn drain<R: Read + Send + 'static>(stream: Option<R>) -> std::thread::JoinHandle<Vec<u8>> {
    std::thread::spawn(move || {
        let mut buffer = Vec::new();
        if let Some(mut stream) = stream {
            let _ = stream.read_to_end(&mut buffer);
        }
        buffer
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sh(script: &str) -> Command {
        let mut command = Command::new("sh");
        command.args(["-c", script]);
        command
    }

    #[test]
    fn test_fast_command_returns_its_output() {
        let out =
            output_with_timeout(sh("printf ok"), Duration::from_secs(5), "test command").unwrap();
        assert!(out.status.success());
        assert_eq!(String::from_utf8_lossy(&out.stdout), "ok");
    }

    #[test]
    fn test_failing_command_surfaces_status_and_stderr() {
        let out = output_with_timeout(
            sh("printf boom >&2; exit 3"),
            Duration::from_secs(5),
            "test command",
        )
        .unwrap();
        assert_eq!(out.status.code(), Some(3));
        assert_eq!(String::from_utf8_lossy(&out.stderr), "boom");
    }

    /// The defect this module exists for: without a deadline this never returns.
    #[test]
    fn test_hung_command_is_terminated_at_the_deadline() {
        let started = Instant::now();
        let error = output_with_timeout(
            sh("sleep 30"),
            Duration::from_millis(200),
            "wedged test command",
        )
        .unwrap_err();

        assert!(
            error.to_string().contains("exceeded its"),
            "unexpected error: {error}"
        );
        assert!(
            started.elapsed() < Duration::from_secs(5),
            "the deadline must fire long before the child would exit on its own"
        );
    }

    /// A child that outproduces the pipe buffer must not deadlock the poll loop.
    #[test]
    fn test_large_output_does_not_deadlock_the_deadline() {
        let out = output_with_timeout(
            sh("for i in $(seq 1 5000); do echo 0123456789012345678901234567890123456789; done"),
            Duration::from_secs(20),
            "chatty test command",
        )
        .unwrap();
        assert!(out.status.success());
        assert!(out.stdout.len() > 200_000, "got {} bytes", out.stdout.len());
    }

    #[test]
    fn test_missing_binary_names_the_operation() {
        let error = output_with_timeout(
            Command::new("operator-no-such-binary-xyz"),
            Duration::from_secs(5),
            "preflight probe",
        )
        .unwrap_err();
        assert!(error.to_string().contains("preflight probe"));
    }
}

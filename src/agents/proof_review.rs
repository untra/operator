//! Proof Runner - Command-based proof-of-work review.
//!
//! For steps with `review_type: proof`, this handler runs an assertion
//! command (and optionally an artifact-capture command) in the worktree,
//! collects any declared artifacts, and persists the outcome as
//! `result.json` under `.proof/{ticket_id}/{step_name}/`.
//!
//! Unix-only: commands run via `sh -c`, mirroring the shell assumption
//! already made by the rest of the launch plumbing.

use anyhow::{Context, Result};
use handlebars::Handlebars;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::{Duration, Instant};
use tokio::io::AsyncReadExt;
use tokio::process::Command;
use tracing::{instrument, warn};

use crate::templates::schema::ProofReviewConfig;

const DEFAULT_TIMEOUT_SECS: u64 = 120;

/// Result of a proof review run, persisted as `result.json`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProofResult {
    /// The rendered assertion command that was actually executed.
    pub assertion_command: String,
    /// Exit code of the assertion command; -1 if killed by signal/timeout.
    pub exit_code: i32,
    /// True iff exit_code == 0 && !timed_out.
    pub passed: bool,
    /// True if the assertion command was killed for exceeding its timeout.
    pub timed_out: bool,
    /// Wall-clock duration of the assertion command, in milliseconds.
    pub duration_ms: u64,
    /// Paths (relative to worktree_root) of everything written into the
    /// proof dir: assertion.log, artifact.log (if run), and copied files.
    pub artifacts: Vec<String>,
    /// Error from running/spawning `artifact_command`, if any. Artifact capture is best-effort.
    pub artifact_command_error: Option<String>,
    /// RFC3339 timestamp of when this result was produced.
    pub timestamp: String,
}

/// Runs proof-of-work reviews: an assertion command plus optional artifact
/// capture, persisted under `.proof/{ticket_id}/{step_name}/`.
pub struct ProofRunner {
    /// Context for handlebars rendering of `assertion_command` /
    /// `artifact_command` (merged under `ticket_id`, `step`, `proof_dir`).
    context: serde_json::Value,
}

impl ProofRunner {
    /// Create a new runner with an empty template context.
    pub fn new() -> Self {
        Self {
            context: serde_json::Value::Object(serde_json::Map::new()),
        }
    }

    /// Create a runner with template context for command rendering.
    pub fn with_context(context: serde_json::Value) -> Self {
        Self { context }
    }

    /// Directory a proof run for `(ticket_id, step_name)` is stored in:
    /// `{worktree_root}/.proof/{ticket_id}/{step_name}/`.
    fn proof_dir(worktree_root: &Path, ticket_id: &str, step_name: &str) -> PathBuf {
        worktree_root.join(".proof").join(ticket_id).join(step_name)
    }

    /// Path to the persisted `result.json` for `(ticket_id, step_name)`.
    /// Used by callers (complete_step hook, sync arm) for idempotence
    /// checks - a proof step has already run iff this path exists.
    pub fn result_path(worktree_root: &Path, ticket_id: &str, step_name: &str) -> PathBuf {
        Self::proof_dir(worktree_root, ticket_id, step_name).join("result.json")
    }

    /// Render a command template with `ticket_id`, `step`, `proof_dir`
    /// merged over any `with_context` values.
    fn render_command(
        &self,
        template: &str,
        ticket_id: &str,
        step_name: &str,
        proof_dir: &Path,
    ) -> Result<String> {
        let mut ctx = self.context.clone();
        if let serde_json::Value::Object(ref mut map) = ctx {
            map.insert("ticket_id".to_string(), ticket_id.into());
            map.insert("step".to_string(), step_name.into());
            map.insert(
                "proof_dir".to_string(),
                proof_dir.to_string_lossy().to_string().into(),
            );
        }
        let hbs = Handlebars::new();
        hbs.render_template(template, &ctx)
            .context("Failed to render proof command template")
    }

    /// Run `sh -c <rendered command>` in `worktree_root`, capturing
    /// stdout+stderr into `log_path` (sections, not interleaved) and
    /// enforcing `timeout`. Returns (exit_code, timed_out, duration_ms).
    async fn run_command(
        command: &str,
        worktree_root: &Path,
        proof_dir: &Path,
        log_path: &Path,
        timeout: Duration,
    ) -> Result<(i32, bool, u64)> {
        let start = Instant::now();

        let mut child = Command::new("sh")
            .arg("-c")
            .arg(command)
            .current_dir(worktree_root)
            .env("OPERATOR_PROOF_DIR", proof_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("Failed to spawn proof command")?;

        let mut stdout = child.stdout.take().context("Missing child stdout")?;
        let mut stderr = child.stderr.take().context("Missing child stderr")?;

        let result = tokio::time::timeout(timeout, async {
            let mut out = Vec::new();
            let mut err = Vec::new();
            let (out_res, err_res, status_res) = tokio::join!(
                stdout.read_to_end(&mut out),
                stderr.read_to_end(&mut err),
                child.wait(),
            );
            out_res.context("Failed to read stdout")?;
            err_res.context("Failed to read stderr")?;
            let status = status_res.context("Failed to wait for proof command")?;
            Ok::<_, anyhow::Error>((out, err, status))
        })
        .await;

        let duration_ms = start.elapsed().as_millis() as u64;

        match result {
            Ok(Ok((out, err, status))) => {
                let mut log = format!("$ {command}\n\n--- stdout ---\n");
                log.push_str(&String::from_utf8_lossy(&out));
                log.push_str("\n--- stderr ---\n");
                log.push_str(&String::from_utf8_lossy(&err));
                log.push('\n');
                tokio::fs::write(log_path, log)
                    .await
                    .context("Failed to write proof log")?;
                let exit_code = status.code().unwrap_or(-1);
                Ok((exit_code, false, duration_ms))
            }
            Ok(Err(e)) => Err(e),
            Err(_elapsed) => {
                warn!("Proof command timed out after {:?}: {}", timeout, command);
                child.kill().await.ok();
                child.wait().await.ok();
                let log = format!("$ {command}\n\n--- TIMED OUT after {timeout:?} ---\n");
                tokio::fs::write(log_path, log)
                    .await
                    .context("Failed to write proof log")?;
                Ok((-1, true, duration_ms))
            }
        }
    }

    /// Run a proof review: assertion command, optional artifact command,
    /// artifact glob collection, then persist `result.json`.
    ///
    /// Order:
    /// 1. Create/clear the proof dir; ensure `.proof/.gitignore` exists.
    /// 2. Run `assertion_command` (rendered) -> assertion.log.
    /// 3. If set, run `artifact_command` (rendered) -> artifact.log; its
    ///    failure is recorded in `artifact_command_error`, never affects
    ///    `passed`.
    /// 4. Copy files matching `artifact_patterns` (globs relative to
    ///    `worktree_root`) into the proof dir, flat, by file name.
    /// 5. Persist result.json and return the struct.
    #[instrument(skip(self, config))]
    pub async fn run(
        &self,
        config: &ProofReviewConfig,
        worktree_root: &Path,
        ticket_id: &str,
        step_name: &str,
    ) -> Result<ProofResult> {
        let proof_dir = Self::proof_dir(worktree_root, ticket_id, step_name);
        tokio::fs::create_dir_all(&proof_dir)
            .await
            .context("Failed to create proof dir")?;

        let gitignore_path = worktree_root.join(".proof").join(".gitignore");
        tokio::fs::write(&gitignore_path, "*\n")
            .await
            .context("Failed to write .proof/.gitignore")?;

        let timeout = Duration::from_secs(
            config
                .timeout_secs
                .map(u64::from)
                .unwrap_or(DEFAULT_TIMEOUT_SECS),
        );

        let assertion_command =
            self.render_command(&config.assertion_command, ticket_id, step_name, &proof_dir)?;
        let assertion_log = proof_dir.join("assertion.log");
        let (exit_code, timed_out, duration_ms) = Self::run_command(
            &assertion_command,
            worktree_root,
            &proof_dir,
            &assertion_log,
            timeout,
        )
        .await?;

        let mut artifact_command_error = None;
        if let Some(ref artifact_command_template) = config.artifact_command {
            let artifact_command =
                self.render_command(artifact_command_template, ticket_id, step_name, &proof_dir)?;
            let artifact_log = proof_dir.join("artifact.log");
            match Self::run_command(
                &artifact_command,
                worktree_root,
                &proof_dir,
                &artifact_log,
                timeout,
            )
            .await
            {
                Ok((code, artifact_timed_out, _)) if code != 0 || artifact_timed_out => {
                    artifact_command_error = Some(if artifact_timed_out {
                        format!("artifact_command timed out after {timeout:?}")
                    } else {
                        format!("artifact_command exited with code {code}")
                    });
                }
                Ok(_) => {}
                Err(e) => {
                    artifact_command_error = Some(format!("{e:#}"));
                }
            }
        }

        Self::collect_artifacts(worktree_root, &proof_dir, &config.artifact_patterns)?;

        let artifacts = Self::list_artifacts(worktree_root, &proof_dir)?;

        let result = ProofResult {
            assertion_command,
            exit_code,
            passed: exit_code == 0 && !timed_out,
            timed_out,
            duration_ms,
            artifacts,
            artifact_command_error,
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        let result_json =
            serde_json::to_string_pretty(&result).context("Failed to serialize proof result")?;
        tokio::fs::write(proof_dir.join("result.json"), result_json)
            .await
            .context("Failed to write result.json")?;

        Ok(result)
    }

    /// Copy files matching `patterns` (globs relative to `worktree_root`)
    /// into `proof_dir`, flat by file name; last match wins on collision.
    ///
    /// Supported glob syntax is whatever the `glob` crate supports (already
    /// a dependency, used elsewhere in this codebase for the same purpose -
    /// see `artifact_detector.rs`), which includes `*`, `?`, `[...]`, and
    /// `**` recursive matching.
    fn collect_artifacts(
        worktree_root: &Path,
        proof_dir: &Path,
        patterns: &[String],
    ) -> Result<()> {
        for pattern in patterns {
            let full_pattern = worktree_root.join(pattern).to_string_lossy().to_string();
            let entries = glob::glob(&full_pattern)
                .with_context(|| format!("Invalid artifact glob pattern: {pattern}"))?;
            for entry in entries.flatten() {
                if !entry.is_file() {
                    continue;
                }
                let Some(file_name) = entry.file_name() else {
                    continue;
                };
                std::fs::copy(&entry, proof_dir.join(file_name))
                    .with_context(|| format!("Failed to copy artifact {}", entry.display()))?;
            }
        }
        Ok(())
    }

    /// List everything now in `proof_dir` except `result.json`, as paths
    /// relative to `worktree_root`.
    fn list_artifacts(worktree_root: &Path, proof_dir: &Path) -> Result<Vec<String>> {
        let mut artifacts = Vec::new();
        for entry in std::fs::read_dir(proof_dir).context("Failed to read proof dir")? {
            let entry = entry?;
            if entry.file_name() == "result.json" {
                continue;
            }
            let path = entry.path();
            let relative = path.strip_prefix(worktree_root).unwrap_or(&path);
            artifacts.push(relative.to_string_lossy().to_string());
        }
        artifacts.sort();
        Ok(artifacts)
    }
}

impl Default for ProofRunner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn config(assertion_command: &str) -> ProofReviewConfig {
        ProofReviewConfig {
            assertion_command: assertion_command.to_string(),
            artifact_command: None,
            artifact_patterns: Vec::new(),
            timeout_secs: None,
        }
    }

    #[tokio::test]
    async fn test_assertion_true_passes() {
        let temp_dir = TempDir::new().unwrap();
        let runner = ProofRunner::new();
        let result = runner
            .run(&config("true"), temp_dir.path(), "TICKET-1", "step-a")
            .await
            .unwrap();

        assert!(result.passed);
        assert_eq!(result.exit_code, 0);
        assert!(!result.timed_out);

        let result_path = ProofRunner::result_path(temp_dir.path(), "TICKET-1", "step-a");
        assert!(result_path.exists());
        let assertion_log = result_path.with_file_name("assertion.log");
        assert!(assertion_log.exists());

        let gitignore = temp_dir.path().join(".proof").join(".gitignore");
        let contents = std::fs::read_to_string(gitignore).unwrap();
        assert_eq!(contents, "*\n");
    }

    #[tokio::test]
    async fn test_assertion_false_fails_but_artifacts_written() {
        let temp_dir = TempDir::new().unwrap();
        let runner = ProofRunner::new();
        let result = runner
            .run(&config("false"), temp_dir.path(), "TICKET-1", "step-a")
            .await
            .unwrap();

        assert!(!result.passed);
        assert_eq!(result.exit_code, 1);
        assert!(!result.artifacts.is_empty());
    }

    #[tokio::test]
    async fn test_assertion_timeout() {
        let temp_dir = TempDir::new().unwrap();
        let runner = ProofRunner::new();
        let mut cfg = config("sleep 5");
        cfg.timeout_secs = Some(1);

        let start = Instant::now();
        let result = runner
            .run(&cfg, temp_dir.path(), "TICKET-1", "step-a")
            .await
            .unwrap();
        let elapsed = start.elapsed();

        assert!(result.timed_out);
        assert!(!result.passed);
        assert_eq!(result.exit_code, -1);
        assert!(elapsed < Duration::from_secs(3));
    }

    #[tokio::test]
    async fn test_assertion_log_contains_stdout_and_stderr() {
        let temp_dir = TempDir::new().unwrap();
        let runner = ProofRunner::new();
        runner
            .run(
                &config("echo out; echo err 1>&2"),
                temp_dir.path(),
                "TICKET-1",
                "step-a",
            )
            .await
            .unwrap();

        let assertion_log = ProofRunner::result_path(temp_dir.path(), "TICKET-1", "step-a")
            .with_file_name("assertion.log");
        let contents = std::fs::read_to_string(assertion_log).unwrap();
        assert!(contents.contains("out"));
        assert!(contents.contains("err"));
    }

    #[tokio::test]
    async fn test_artifact_command_success() {
        let temp_dir = TempDir::new().unwrap();
        std::fs::write(temp_dir.path().join("somefile"), b"data").unwrap();

        let runner = ProofRunner::new();
        let mut cfg = config("true");
        cfg.artifact_command = Some("cp somefile $OPERATOR_PROOF_DIR/".to_string());

        let result = runner
            .run(&cfg, temp_dir.path(), "TICKET-1", "step-a")
            .await
            .unwrap();

        assert!(result.artifact_command_error.is_none());
        let copied = ProofRunner::result_path(temp_dir.path(), "TICKET-1", "step-a")
            .with_file_name("somefile");
        assert!(copied.exists());
    }

    #[tokio::test]
    async fn test_artifact_command_failure_recorded_but_passed_tracks_assertion() {
        let temp_dir = TempDir::new().unwrap();
        let runner = ProofRunner::new();
        let mut cfg = config("true");
        cfg.artifact_command = Some("false".to_string());

        let result = runner
            .run(&cfg, temp_dir.path(), "TICKET-1", "step-a")
            .await
            .unwrap();

        assert!(result.artifact_command_error.is_some());
        assert!(result.passed);
    }

    #[tokio::test]
    async fn test_artifact_patterns_glob_copies_matching_file() {
        let temp_dir = TempDir::new().unwrap();
        std::fs::write(temp_dir.path().join("report.txt"), b"contents").unwrap();
        std::fs::write(temp_dir.path().join("ignored.md"), b"nope").unwrap();

        let runner = ProofRunner::new();
        let mut cfg = config("true");
        cfg.artifact_patterns = vec!["*.txt".to_string()];

        let result = runner
            .run(&cfg, temp_dir.path(), "TICKET-1", "step-a")
            .await
            .unwrap();

        let copied = ProofRunner::result_path(temp_dir.path(), "TICKET-1", "step-a")
            .with_file_name("report.txt");
        assert!(copied.exists());

        let result_json = std::fs::read_to_string(ProofRunner::result_path(
            temp_dir.path(),
            "TICKET-1",
            "step-a",
        ))
        .unwrap();
        let round_tripped: ProofResult = serde_json::from_str(&result_json).unwrap();
        assert_eq!(round_tripped.exit_code, result.exit_code);
        assert!(round_tripped
            .artifacts
            .iter()
            .any(|a| a.ends_with("report.txt")));
    }

    #[tokio::test]
    async fn test_handlebars_proof_dir_and_ticket_id_render() {
        let temp_dir = TempDir::new().unwrap();
        let runner = ProofRunner::new();
        let result = runner
            .run(
                &config("test -d {{proof_dir}}"),
                temp_dir.path(),
                "TICKET-42",
                "step-a",
            )
            .await
            .unwrap();

        assert!(result.passed);

        let mut cfg = config("echo {{ticket_id}} > $OPERATOR_PROOF_DIR/id.txt");
        cfg.assertion_command = "echo {{ticket_id}}".to_string();
        let result2 = runner
            .run(&cfg, temp_dir.path(), "TICKET-42", "step-b")
            .await
            .unwrap();
        assert_eq!(result2.assertion_command, "echo TICKET-42");
    }
}

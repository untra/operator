//! Build an `opr8r tools --json` spec from Operator's tool catalog and parse
//! the host-local results.

use std::io::Write;
use std::process::{Command, Stdio};

use serde::{Deserialize, Serialize};

use super::tool_config::{load_all_tool_configs, ToolConfig};

/// One tool Operator asks `opr8r tools` to look up on a host.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ToolProbeSpec {
    pub name: String,
    pub version_command: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub health_command: Option<String>,
}

/// One row returned by `opr8r tools --json`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ToolProbeResult {
    pub name: String,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
    pub health_ok: bool,
    #[serde(default)]
    pub error: Option<String>,
}

/// Spec list for every currently loaded tool config (builtins + user JSON).
pub fn probe_specs() -> Vec<ToolProbeSpec> {
    load_all_tool_configs()
        .iter()
        .map(spec_from_config)
        .collect()
}

fn spec_from_config(config: &ToolConfig) -> ToolProbeSpec {
    ToolProbeSpec {
        name: config.tool_name.clone(),
        version_command: config.version_command.clone(),
        health_command: config
            .detection
            .as_ref()
            .and_then(|d| d.health_command.clone()),
    }
}

/// Parse the JSON array `opr8r tools --json` writes to stdout.
pub fn parse_probe_results(body: &str) -> Result<Vec<ToolProbeResult>, String> {
    serde_json::from_str(body.trim()).map_err(|e| format!("opr8r tools output is not JSON: {e}"))
}

/// Run `opr8r tools --json` locally (PATH).
pub fn probe_local() -> Result<Vec<ToolProbeResult>, String> {
    let mut command = Command::new("opr8r");
    command.args(["tools", "--json"]);
    run_probe(&mut command)
}

/// Run `opr8r tools --json` on an SSH host. `ssh` is already configured with
/// alias / `-F` / BatchMode; this function only appends the remote command.
pub fn probe_over_ssh(ssh: &mut Command) -> Result<Vec<ToolProbeResult>, String> {
    ssh.arg("opr8r tools --json");
    run_probe(ssh)
}

fn run_probe(command: &mut Command) -> Result<Vec<ToolProbeResult>, String> {
    let spec = serde_json::to_vec(&probe_specs()).map_err(|e| e.to_string())?;
    command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command
        .spawn()
        .map_err(|e| format!("opr8r tools not runnable: {e}"))?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(&spec)
            .map_err(|e| format!("failed to write tool spec: {e}"))?;
    }
    let output = child
        .wait_with_output()
        .map_err(|e| format!("opr8r tools failed: {e}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "opr8r tools exited {}: {stderr}",
            output.status.code().unwrap_or(-1)
        ));
    }
    parse_probe_results(&String::from_utf8_lossy(&output.stdout))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_probe_specs_include_shipped_binaries() {
        let specs = probe_specs();
        let names: Vec<&str> = specs.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"claude"));
        assert!(names.contains(&"codex"));
        assert!(names.contains(&"gemini"));
    }

    #[test]
    fn test_parse_probe_results_roundtrip() {
        let body =
            r#"[{"name":"claude","path":"/usr/bin/claude","version":"2.1.0","health_ok":true}]"#;
        let results = parse_probe_results(body).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "claude");
        assert!(results[0].health_ok);
    }

    #[test]
    fn test_parse_probe_results_rejects_garbage() {
        let err = parse_probe_results("not json").unwrap_err();
        assert!(err.contains("not JSON"));
    }
}

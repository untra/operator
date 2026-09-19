//! Host-local LLM CLI inventory.
//!
//! Operator owns the tool catalog and sends a JSON spec on stdin. This module
//! reports what is actually on PATH on the machine running `opr8r`.

use std::io::{Read, Write};
use std::process::Command;

use serde::{Deserialize, Serialize};

/// One tool Operator wants this host to look up.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct ToolProbeSpec {
    pub name: String,
    pub version_command: String,
    #[serde(default)]
    pub health_command: Option<String>,
}

/// Presence / version / health for one spec.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct ToolProbeResult {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    pub health_ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Probe every spec. Never panics; failures land on the result row.
pub fn probe_tools(specs: &[ToolProbeSpec]) -> Vec<ToolProbeResult> {
    specs.iter().map(probe_one).collect()
}

fn probe_one(spec: &ToolProbeSpec) -> ToolProbeResult {
    let path = binary_path(&spec.name);
    if path.is_none() {
        return ToolProbeResult {
            name: spec.name.clone(),
            path: None,
            version: None,
            health_ok: false,
            error: Some(format!("'{}' is not on PATH", spec.name)),
        };
    }

    let version = run_command_line(&spec.version_command).ok();
    let health_ok = match spec.health_command.as_deref() {
        Some(cmd) => run_command_line(cmd).is_ok(),
        None => true,
    };
    let error = if health_ok {
        None
    } else {
        Some(format!(
            "health_command failed: {}",
            spec.health_command.as_deref().unwrap_or("none")
        ))
    };

    ToolProbeResult {
        name: spec.name.clone(),
        path,
        version,
        health_ok,
        error,
    }
}

fn binary_path(name: &str) -> Option<String> {
    Command::new("which")
        .arg(name)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .filter(|s| !s.is_empty())
}

fn run_command_line(line: &str) -> Result<String, ()> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    let Some((program, args)) = parts.split_first() else {
        return Ok(String::new());
    };
    Command::new(program)
        .args(args)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .ok_or(())
}

/// Read a JSON spec array from `stdin`, write a JSON result array to `stdout`.
pub fn run_json(stdin: &mut impl Read, stdout: &mut impl Write) -> Result<(), String> {
    let mut body = String::new();
    stdin
        .read_to_string(&mut body)
        .map_err(|e| format!("failed to read spec: {e}"))?;
    let specs: Vec<ToolProbeSpec> =
        serde_json::from_str(body.trim()).map_err(|e| format!("invalid tool spec JSON: {e}"))?;
    let results = probe_tools(&specs);
    serde_json::to_writer(&mut *stdout, &results)
        .map_err(|e| format!("failed to write results: {e}"))?;
    writeln!(stdout).map_err(|e| format!("failed to write results: {e}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_probe_true_is_healthy() {
        let results = probe_tools(&[ToolProbeSpec {
            name: "true".into(),
            version_command: "true".into(),
            health_command: None,
        }]);
        assert_eq!(results.len(), 1);
        assert!(results[0].health_ok);
        assert!(results[0].path.as_ref().is_some_and(|p| p.contains("true")));
        assert!(results[0].error.is_none());
    }

    #[test]
    fn test_probe_missing_binary_is_unhealthy() {
        let results = probe_tools(&[ToolProbeSpec {
            name: "opr8r-tool-does-not-exist".into(),
            version_command: "opr8r-tool-does-not-exist --version".into(),
            health_command: None,
        }]);
        assert!(!results[0].health_ok);
        assert!(results[0].path.is_none());
        assert!(results[0]
            .error
            .as_ref()
            .is_some_and(|e| e.contains("not on PATH")));
    }

    #[test]
    fn test_probe_failing_health_command_is_unhealthy() {
        let results = probe_tools(&[ToolProbeSpec {
            name: "true".into(),
            version_command: "true".into(),
            health_command: Some("false".into()),
        }]);
        assert!(results[0].path.is_some());
        assert!(!results[0].health_ok);
        assert!(results[0]
            .error
            .as_ref()
            .is_some_and(|e| e.contains("health_command")));
    }

    #[test]
    fn test_run_json_roundtrip() {
        let spec = r#"[{"name":"true","version_command":"true"}]"#;
        let mut out = Vec::new();
        run_json(&mut spec.as_bytes(), &mut out).expect("spec parses");
        let results: Vec<ToolProbeResult> = serde_json::from_slice(&out).unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].health_ok);
    }

    #[test]
    fn test_run_json_rejects_malformed_spec() {
        let err = run_json(&mut b"{not json}".as_slice(), &mut Vec::new()).unwrap_err();
        assert!(err.contains("invalid tool spec JSON"));
    }
}

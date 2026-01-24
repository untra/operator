//! Output parser for OPERATOR_STATUS blocks.
//!
//! Parses structured status output from agent responses in YAML key-value format
//! enclosed in `---OPERATOR_STATUS---` / `---END_OPERATOR_STATUS---` markers.

use serde::{Deserialize, Serialize};

/// Start marker for operator status block
const START_MARKER: &str = "---OPERATOR_STATUS---";
/// End marker for operator status block
const END_MARKER: &str = "---END_OPERATOR_STATUS---";

/// Parsed operator output from agent.
///
/// This matches the OperatorOutput structure expected by the Operator API.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct ParsedOutput {
    /// Current work status: in_progress, complete, blocked, failed
    pub status: String,
    /// Agent signals done with step (true) or more work remains (false)
    pub exit_signal: bool,
    /// Agent's confidence in completion (0-100%)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<u8>,
    /// Number of files changed this iteration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub files_modified: Option<u32>,
    /// Test suite status: passing, failing, skipped, not_run
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tests_status: Option<String>,
    /// Number of errors encountered
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_count: Option<u32>,
    /// Number of sub-tasks completed this iteration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tasks_completed: Option<u32>,
    /// Estimated remaining sub-tasks
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tasks_remaining: Option<u32>,
    /// Brief description of work done (max 500 chars)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    /// Suggested next action (max 200 chars)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recommendation: Option<String>,
    /// Issues preventing progress (signals intervention needed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blockers: Option<Vec<String>>,
    /// Raw block content for debugging
    #[serde(skip)]
    pub raw_block: Option<String>,
}

/// Parse an OPERATOR_STATUS block from output text.
///
/// The expected format is:
/// ```text
/// ---OPERATOR_STATUS---
/// status: complete
/// exit_signal: true
/// confidence: 95
/// files_modified: 3
/// tests_status: passing
/// summary: Implemented the feature
/// recommendation: Ready for code review
/// ---END_OPERATOR_STATUS---
/// ```
///
/// Returns `Some(ParsedOutput)` if a valid block is found, `None` otherwise.
pub fn parse_status_block(output: &str) -> Option<ParsedOutput> {
    // Find block boundaries
    let start_pos = output.find(START_MARKER)?;
    let end_pos = output.find(END_MARKER)?;

    // Ensure end comes after start
    if end_pos <= start_pos {
        return None;
    }

    // Extract block content (excluding markers)
    let block_start = start_pos + START_MARKER.len();
    let block = &output[block_start..end_pos];

    let mut result = ParsedOutput {
        raw_block: Some(block.to_string()),
        ..Default::default()
    };

    // Parse key-value pairs (YAML-style)
    for line in block.lines() {
        let line = line.trim();

        // Skip empty lines
        if line.is_empty() {
            continue;
        }

        // Split on first colon
        if let Some((key, value)) = line.split_once(':') {
            let key = key.trim().to_lowercase();
            let key = key.replace(['_', '-'], ""); // Normalize key (e.g., exit_signal -> exitsignal)
            let value = value.trim();

            match key.as_str() {
                "status" => result.status = value.to_string(),
                "exitsignal" => result.exit_signal = parse_bool(value),
                "confidence" => result.confidence = value.parse().ok(),
                "filesmodified" => result.files_modified = value.parse().ok(),
                "testsstatus" => result.tests_status = Some(value.to_string()),
                "errorcount" => result.error_count = value.parse().ok(),
                "taskscompleted" => result.tasks_completed = value.parse().ok(),
                "tasksremaining" => result.tasks_remaining = value.parse().ok(),
                "summary" => result.summary = Some(truncate(value, 500)),
                "recommendation" => result.recommendation = Some(truncate(value, 200)),
                "blockers" => result.blockers = Some(parse_list(value)),
                _ => {} // Ignore unknown keys
            }
        }
    }

    // Require at least status to be present
    if result.status.is_empty() {
        return None;
    }

    Some(result)
}

/// Parse a boolean value from various string representations.
fn parse_bool(value: &str) -> bool {
    matches!(
        value.to_lowercase().as_str(),
        "true" | "yes" | "1" | "on" | "y"
    )
}

/// Truncate a string to max length, preserving whole words if possible.
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        return s.to_string();
    }

    // Find last space before max_len
    let truncated = &s[..max_len];
    if let Some(last_space) = truncated.rfind(' ') {
        format!("{}...", &s[..last_space])
    } else {
        format!("{}...", truncated)
    }
}

/// Parse a comma-separated list of strings.
fn parse_list(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Find the last OPERATOR_STATUS block in the output.
///
/// This is useful when an agent outputs multiple status blocks, and we want
/// the final one which represents the actual completion state.
pub fn find_last_status_block(output: &str) -> Option<ParsedOutput> {
    let mut last_result = None;
    let mut remaining = output;

    while let Some(start_pos) = remaining.find(START_MARKER) {
        if let Some(result) = parse_status_block(&remaining[start_pos..]) {
            last_result = Some(result);
        }

        // Move past this block to look for more
        if let Some(end_pos) = remaining[start_pos..].find(END_MARKER) {
            remaining = &remaining[start_pos + end_pos + END_MARKER.len()..];
        } else {
            break;
        }
    }

    last_result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_complete_block() {
        let output = r#"
Some other output here...

---OPERATOR_STATUS---
status: complete
exit_signal: true
confidence: 95
files_modified: 3
tests_status: passing
error_count: 0
tasks_completed: 5
tasks_remaining: 0
summary: Implemented user authentication with JWT tokens
recommendation: Ready for code review
---END_OPERATOR_STATUS---

More output after...
"#;

        let parsed = parse_status_block(output).unwrap();
        assert_eq!(parsed.status, "complete");
        assert!(parsed.exit_signal);
        assert_eq!(parsed.confidence, Some(95));
        assert_eq!(parsed.files_modified, Some(3));
        assert_eq!(parsed.tests_status, Some("passing".to_string()));
        assert_eq!(parsed.error_count, Some(0));
        assert_eq!(parsed.tasks_completed, Some(5));
        assert_eq!(parsed.tasks_remaining, Some(0));
        assert!(parsed.summary.unwrap().contains("JWT tokens"));
        assert_eq!(
            parsed.recommendation,
            Some("Ready for code review".to_string())
        );
    }

    #[test]
    fn test_parse_minimal_block() {
        let output = r#"
---OPERATOR_STATUS---
status: in_progress
exit_signal: false
---END_OPERATOR_STATUS---
"#;

        let parsed = parse_status_block(output).unwrap();
        assert_eq!(parsed.status, "in_progress");
        assert!(!parsed.exit_signal);
        assert!(parsed.confidence.is_none());
        assert!(parsed.files_modified.is_none());
    }

    #[test]
    fn test_parse_blocked_with_blockers() {
        let output = r#"
---OPERATOR_STATUS---
status: blocked
exit_signal: false
blockers: Missing DATABASE_URL, Cannot connect to test database
---END_OPERATOR_STATUS---
"#;

        let parsed = parse_status_block(output).unwrap();
        assert_eq!(parsed.status, "blocked");
        assert!(!parsed.exit_signal);
        let blockers = parsed.blockers.unwrap();
        assert_eq!(blockers.len(), 2);
        assert_eq!(blockers[0], "Missing DATABASE_URL");
        assert_eq!(blockers[1], "Cannot connect to test database");
    }

    #[test]
    fn test_parse_missing_block() {
        let output = "No status block here";
        assert!(parse_status_block(output).is_none());
    }

    #[test]
    fn test_parse_incomplete_block_no_end() {
        let output = r#"
---OPERATOR_STATUS---
status: complete
exit_signal: true
"#;
        assert!(parse_status_block(output).is_none());
    }

    #[test]
    fn test_parse_incomplete_block_no_start() {
        let output = r#"
status: complete
exit_signal: true
---END_OPERATOR_STATUS---
"#;
        assert!(parse_status_block(output).is_none());
    }

    #[test]
    fn test_parse_empty_status() {
        let output = r#"
---OPERATOR_STATUS---
exit_signal: true
---END_OPERATOR_STATUS---
"#;
        // Status is required, so this should fail
        assert!(parse_status_block(output).is_none());
    }

    #[test]
    fn test_parse_bool_values() {
        assert!(parse_bool("true"));
        assert!(parse_bool("True"));
        assert!(parse_bool("TRUE"));
        assert!(parse_bool("yes"));
        assert!(parse_bool("1"));
        assert!(parse_bool("on"));
        assert!(parse_bool("y"));

        assert!(!parse_bool("false"));
        assert!(!parse_bool("False"));
        assert!(!parse_bool("no"));
        assert!(!parse_bool("0"));
        assert!(!parse_bool("off"));
        assert!(!parse_bool(""));
    }

    #[test]
    fn test_truncate_short_string() {
        assert_eq!(truncate("short", 10), "short");
    }

    #[test]
    fn test_truncate_long_string() {
        let long = "This is a very long string that needs to be truncated";
        let truncated = truncate(long, 20);
        assert!(truncated.ends_with("..."));
        assert!(truncated.len() <= 23); // 20 + "..."
    }

    #[test]
    fn test_parse_list_empty() {
        let list = parse_list("");
        assert!(list.is_empty());
    }

    #[test]
    fn test_parse_list_single() {
        let list = parse_list("item1");
        assert_eq!(list, vec!["item1"]);
    }

    #[test]
    fn test_parse_list_multiple() {
        let list = parse_list("item1, item2, item3");
        assert_eq!(list, vec!["item1", "item2", "item3"]);
    }

    #[test]
    fn test_find_last_status_block() {
        let output = r#"
First block:
---OPERATOR_STATUS---
status: in_progress
exit_signal: false
---END_OPERATOR_STATUS---

Some work happened...

Second block:
---OPERATOR_STATUS---
status: complete
exit_signal: true
confidence: 100
---END_OPERATOR_STATUS---
"#;

        let parsed = find_last_status_block(output).unwrap();
        assert_eq!(parsed.status, "complete");
        assert!(parsed.exit_signal);
        assert_eq!(parsed.confidence, Some(100));
    }

    #[test]
    fn test_serialization_round_trip() {
        let output = ParsedOutput {
            status: "complete".to_string(),
            exit_signal: true,
            confidence: Some(90),
            files_modified: Some(3),
            tests_status: Some("passing".to_string()),
            error_count: None,
            tasks_completed: Some(5),
            tasks_remaining: Some(0),
            summary: Some("Done".to_string()),
            recommendation: Some("Review".to_string()),
            blockers: None,
            raw_block: None,
        };

        let json = serde_json::to_string(&output).unwrap();
        let deserialized: ParsedOutput = serde_json::from_str(&json).unwrap();

        assert_eq!(output.status, deserialized.status);
        assert_eq!(output.exit_signal, deserialized.exit_signal);
        assert_eq!(output.confidence, deserialized.confidence);
    }

    #[test]
    fn test_key_normalization() {
        // Test that keys with underscores and dashes are normalized
        let output = r#"
---OPERATOR_STATUS---
status: complete
exit-signal: true
files-modified: 5
tests-status: passing
---END_OPERATOR_STATUS---
"#;

        let parsed = parse_status_block(output).unwrap();
        assert!(parsed.exit_signal);
        assert_eq!(parsed.files_modified, Some(5));
        assert_eq!(parsed.tests_status, Some("passing".to_string()));
    }
}

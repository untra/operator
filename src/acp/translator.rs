//! Translate delegator subprocess output into ACP `SessionUpdate`
//! notifications.
//!
//! Two parsing modes: structured JSON (Claude Code's `--output-format
//! stream-json`) and plain text (fallback). `line_to_update` tries JSON
//! first, falls back to plain text on parse failure.

use agent_client_protocol::schema::{ContentBlock, ContentChunk, SessionUpdate, TextContent};

/// Map a single line of delegator stdout to an optional ACP `SessionUpdate`.
///
/// Tries structured JSON parse first (for delegators like Claude Code with
/// `--output-format stream-json`). Falls back to plain text wrapping.
/// Empty / whitespace-only lines return `None`.
pub fn line_to_update(line: &str) -> Option<SessionUpdate> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }
    if trimmed.starts_with('{') {
        match try_stream_json(trimmed) {
            StreamJsonResult::Update(update) => return Some(*update),
            StreamJsonResult::Skip => return None,
            StreamJsonResult::NotStreamJson => {}
        }
    }
    plain_text_update(trimmed)
}

enum StreamJsonResult {
    Update(Box<SessionUpdate>),
    Skip,
    NotStreamJson,
}

/// Wrap a non-empty text line as an `AgentMessageChunk`.
fn plain_text_update(trimmed: &str) -> Option<SessionUpdate> {
    let text = TextContent::new(format!("{trimmed}\n"));
    let chunk = ContentChunk::new(ContentBlock::Text(text));
    Some(SessionUpdate::AgentMessageChunk(chunk))
}

/// Try to parse a line as Claude Code `--output-format stream-json`.
///
/// Returns `Update` for displayable events, `Skip` for internal events
/// (`tool_use`, `system`), and `NotStreamJson` if it doesn't look like
/// stream-json (so the caller can fall back to plain text).
fn try_stream_json(line: &str) -> StreamJsonResult {
    let Ok(obj) = serde_json::from_str::<serde_json::Value>(line) else {
        return StreamJsonResult::NotStreamJson;
    };
    let Some(event_type) = obj.get("type").and_then(|v| v.as_str()) else {
        return StreamJsonResult::NotStreamJson;
    };

    match event_type {
        "assistant" => {
            let Some(content) = obj
                .get("message")
                .and_then(|m| m.get("content"))
                .and_then(|c| c.as_array())
            else {
                return StreamJsonResult::Skip;
            };
            let mut texts = Vec::new();
            for block in content {
                if block.get("type").and_then(|v| v.as_str()) == Some("text") {
                    if let Some(t) = block.get("text").and_then(|v| v.as_str()) {
                        texts.push(t.to_string());
                    }
                }
            }
            if texts.is_empty() {
                return StreamJsonResult::Skip;
            }
            let joined = texts.join("\n");
            let text = TextContent::new(format!("{joined}\n"));
            let chunk = ContentChunk::new(ContentBlock::Text(text));
            StreamJsonResult::Update(Box::new(SessionUpdate::AgentMessageChunk(chunk)))
        }
        "result" => {
            if let Some(result_text) = obj.get("result").and_then(|v| v.as_str()) {
                if !result_text.is_empty() {
                    let text = TextContent::new(format!("{result_text}\n"));
                    let chunk = ContentChunk::new(ContentBlock::Text(text));
                    return StreamJsonResult::Update(Box::new(SessionUpdate::AgentMessageChunk(
                        chunk,
                    )));
                }
            }
            StreamJsonResult::Skip
        }
        "tool_use" | "tool_result" | "system" => StreamJsonResult::Skip,
        _ => StreamJsonResult::NotStreamJson,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn extract_text(update: SessionUpdate) -> String {
        let SessionUpdate::AgentMessageChunk(chunk) = update else {
            panic!("expected AgentMessageChunk variant");
        };
        let ContentBlock::Text(text) = chunk.content else {
            panic!("expected Text content block");
        };
        text.text
    }

    #[test]
    fn test_empty_line_yields_none() {
        assert!(line_to_update("").is_none());
        assert!(line_to_update("   ").is_none());
        assert!(line_to_update("\t  \n").is_none());
    }

    #[test]
    fn test_text_line_yields_agent_message_chunk() {
        let text = extract_text(line_to_update("hello world").unwrap());
        assert_eq!(text, "hello world\n");
    }

    #[test]
    fn test_leading_and_trailing_whitespace_trimmed() {
        let text = extract_text(line_to_update("  foo  ").unwrap());
        assert_eq!(text, "foo\n");
    }

    #[test]
    fn test_stream_json_assistant_text() {
        let line = r#"{"type":"assistant","message":{"content":[{"type":"text","text":"Hello from Claude"}]}}"#;
        let text = extract_text(line_to_update(line).unwrap());
        assert_eq!(text, "Hello from Claude\n");
    }

    #[test]
    fn test_stream_json_assistant_multiple_text_blocks() {
        let line = r#"{"type":"assistant","message":{"content":[{"type":"text","text":"First"},{"type":"text","text":"Second"}]}}"#;
        let text = extract_text(line_to_update(line).unwrap());
        assert_eq!(text, "First\nSecond\n");
    }

    #[test]
    fn test_stream_json_result_with_text() {
        let line = r#"{"type":"result","result":"Task complete","cost_usd":0.05}"#;
        let text = extract_text(line_to_update(line).unwrap());
        assert_eq!(text, "Task complete\n");
    }

    #[test]
    fn test_stream_json_result_empty_yields_none() {
        let line = r#"{"type":"result","result":"","cost_usd":0.01}"#;
        assert!(line_to_update(line).is_none());
    }

    #[test]
    fn test_stream_json_tool_use_skipped() {
        let line = r#"{"type":"tool_use","name":"Read","input":{"path":"/tmp/foo"}}"#;
        assert!(line_to_update(line).is_none());
    }

    #[test]
    fn test_stream_json_system_event_skipped() {
        let line = r#"{"type":"system","message":"thinking..."}"#;
        assert!(line_to_update(line).is_none());
    }

    #[test]
    fn test_malformed_json_falls_back_to_plain_text() {
        let line = r#"{"broken json"#;
        let text = extract_text(line_to_update(line).unwrap());
        assert_eq!(text, r#"{"broken json"#.to_string() + "\n");
    }

    #[test]
    fn test_json_without_type_field_falls_back_to_plain_text() {
        let line = r#"{"key":"value"}"#;
        let text = extract_text(line_to_update(line).unwrap());
        assert_eq!(text, r#"{"key":"value"}"#.to_string() + "\n");
    }
}

//! Translate delegator subprocess output into ACP `SessionUpdate`
//! notifications.
//!
//! v1 strategy: every non-empty stdout line becomes a single
//! `AgentMessageChunk` carrying a `Text` content block. When operator
//! later parses Claude Code's `--output-format stream-json` (or similar
//! structured outputs from other delegators), this is the seam to extend.

use agent_client_protocol::schema::{ContentBlock, ContentChunk, SessionUpdate, TextContent};

/// Map a single line of delegator stdout to an optional ACP `SessionUpdate`.
///
/// Empty / whitespace-only lines map to `None` and are suppressed; non-empty
/// lines map to `Some(SessionUpdate::AgentMessageChunk(...))` with the line
/// text followed by a newline (so the editor's renderer reconstructs the
/// original line breaks).
pub fn line_to_update(line: &str) -> Option<SessionUpdate> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }
    let text = TextContent::new(format!("{trimmed}\n"));
    let chunk = ContentChunk::new(ContentBlock::Text(text));
    Some(SessionUpdate::AgentMessageChunk(chunk))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_line_yields_none() {
        assert!(line_to_update("").is_none());
        assert!(line_to_update("   ").is_none());
        assert!(line_to_update("\t  \n").is_none());
    }

    #[test]
    fn test_text_line_yields_agent_message_chunk() {
        let update = line_to_update("hello world").expect("non-empty line should yield Some");
        let SessionUpdate::AgentMessageChunk(chunk) = update else {
            panic!("expected AgentMessageChunk variant");
        };
        let ContentBlock::Text(text) = chunk.content else {
            panic!("expected Text content block");
        };
        assert_eq!(text.text, "hello world\n");
    }

    #[test]
    fn test_leading_and_trailing_whitespace_trimmed() {
        let update = line_to_update("  foo  ").expect("non-empty line");
        let SessionUpdate::AgentMessageChunk(chunk) = update else {
            unreachable!()
        };
        let ContentBlock::Text(text) = chunk.content else {
            unreachable!()
        };
        assert_eq!(text.text, "foo\n");
    }
}

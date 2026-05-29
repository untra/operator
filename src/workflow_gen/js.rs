//! Helpers for emitting safe `JavaScript` source.

/// Escape a string for embedding inside a double-quoted JS string literal.
///
/// Returns the escaped *content* only (no surrounding quotes). Handles the
/// characters that would otherwise terminate the literal or change its meaning:
/// backslash, double quote, and the common control characters. `${` and
/// backticks need no escaping because we only ever emit double-quoted literals.
pub fn escape_str(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            // Strip other C0 control chars that have no business in a literal.
            c if (c as u32) < 0x20 => out.push(' '),
            c => out.push(c),
        }
    }
    out
}

/// Wrap a string as a complete double-quoted JS string literal.
pub fn quote(s: &str) -> String {
    format!("\"{}\"", escape_str(s))
}

/// Make a string safe to embed inside a `/* ... */` block comment by defusing
/// any sequence that would close the comment early.
pub fn comment_safe(s: &str) -> String {
    s.replace("*/", "* /")
}

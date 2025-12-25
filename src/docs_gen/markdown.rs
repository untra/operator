//! Markdown formatting utilities for documentation generation.

/// Build a markdown table from headers and rows
pub fn table(headers: &[&str], rows: &[Vec<String>]) -> String {
    if headers.is_empty() {
        return String::new();
    }

    let mut output = String::new();

    // Header row
    output.push_str("| ");
    output.push_str(&headers.join(" | "));
    output.push_str(" |\n");

    // Separator row
    output.push_str("| ");
    output.push_str(
        &headers
            .iter()
            .map(|_| "---")
            .collect::<Vec<_>>()
            .join(" | "),
    );
    output.push_str(" |\n");

    // Data rows
    for row in rows {
        output.push_str("| ");
        output.push_str(&row.join(" | "));
        output.push_str(" |\n");
    }

    output
}

/// Format a heading with the specified level
pub fn heading(level: u8, text: &str) -> String {
    let hashes = "#".repeat(level as usize);
    format!("{} {}\n\n", hashes, text)
}

/// Format a code block with optional language
pub fn code_block(code: &str, language: Option<&str>) -> String {
    let lang = language.unwrap_or("");
    format!("```{}\n{}\n```\n\n", lang, code)
}

/// Format an inline code span
pub fn inline_code(text: &str) -> String {
    format!("`{}`", text)
}

/// Format a bullet list
pub fn bullet_list(items: &[String]) -> String {
    items
        .iter()
        .map(|item| format!("- {}\n", item))
        .collect::<String>()
        + "\n"
}

/// Format a numbered list
pub fn numbered_list(items: &[String]) -> String {
    items
        .iter()
        .enumerate()
        .map(|(i, item)| format!("{}. {}\n", i + 1, item))
        .collect::<String>()
        + "\n"
}

/// Format a blockquote
pub fn blockquote(text: &str) -> String {
    text.lines()
        .map(|line| format!("> {}\n", line))
        .collect::<String>()
        + "\n"
}

/// Format bold text
pub fn bold(text: &str) -> String {
    format!("**{}**", text)
}

/// Format italic text
pub fn italic(text: &str) -> String {
    format!("*{}*", text)
}

/// Escape special markdown characters
pub fn escape(text: &str) -> String {
    text.replace('|', "\\|")
        .replace('`', "\\`")
        .replace('*', "\\*")
        .replace('_', "\\_")
        .replace('[', "\\[")
        .replace(']', "\\]")
}

/// Escape for use within a table cell (only pipe needs escaping)
pub fn escape_table_cell(text: &str) -> String {
    text.replace('|', "\\|")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_table() {
        let headers = &["Name", "Value"];
        let rows = vec![
            vec!["foo".to_string(), "1".to_string()],
            vec!["bar".to_string(), "2".to_string()],
        ];
        let result = table(headers, &rows);
        assert!(result.contains("| Name | Value |"));
        assert!(result.contains("| --- | --- |"));
        assert!(result.contains("| foo | 1 |"));
    }

    #[test]
    fn test_heading() {
        assert_eq!(heading(1, "Title"), "# Title\n\n");
        assert_eq!(heading(3, "Subsection"), "### Subsection\n\n");
    }

    #[test]
    fn test_code_block() {
        let result = code_block("let x = 1;", Some("rust"));
        assert!(result.contains("```rust"));
        assert!(result.contains("let x = 1;"));
    }

    #[test]
    fn test_bullet_list() {
        let items = vec!["One".to_string(), "Two".to_string()];
        let result = bullet_list(&items);
        assert!(result.contains("- One"));
        assert!(result.contains("- Two"));
    }

    #[test]
    fn test_escape_table_cell() {
        assert_eq!(escape_table_cell("foo|bar"), "foo\\|bar");
    }
}

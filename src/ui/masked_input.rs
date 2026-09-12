//! A single-line text field rendered as bullets.
//!
//! Extracted from [`crate::ui::dialogs::git_token::GitTokenDialog`], which owned
//! the only copy of this logic, so the setup wizard's password step can reuse it
//! rather than hand-roll a third `String` + cursor pair.
//!
//! The cursor is a **character** index, not a byte index. The original code
//! mixed the two - incrementing the cursor per character while indexing the
//! `String` by byte - so any multi-byte character panicked on the next edit.
//! Passwords are exactly where someone types an accented character or an emoji,
//! and `validate_password` counts characters too
//! (`crate::auth::password::validate_password`), so characters are the unit
//! throughout.

use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::Span,
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

/// A masked single-line input.
#[derive(Debug, Default, Clone)]
pub struct MaskedInput {
    value: String,
    /// Cursor position in **characters** from the start.
    cursor: usize,
}

impl MaskedInput {
    pub fn new() -> Self {
        Self::default()
    }

    /// The current value.
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Whether nothing has been typed.
    pub fn is_empty(&self) -> bool {
        self.value.is_empty()
    }

    /// Length in characters - what the cursor and any length rule count in.
    pub fn char_count(&self) -> usize {
        self.value.chars().count()
    }

    /// Cursor position, in characters.
    pub fn cursor(&self) -> usize {
        self.cursor
    }

    /// Clear the value and reset the cursor.
    pub fn clear(&mut self) {
        self.value.clear();
        self.cursor = 0;
    }

    /// Byte offset of a character index, for `String::insert`/`remove`.
    fn byte_offset(&self, char_index: usize) -> usize {
        self.value
            .char_indices()
            .nth(char_index)
            .map_or(self.value.len(), |(offset, _)| offset)
    }

    pub fn handle_char(&mut self, c: char) {
        let offset = self.byte_offset(self.cursor);
        self.value.insert(offset, c);
        self.cursor += 1;
    }

    pub fn handle_backspace(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
            let offset = self.byte_offset(self.cursor);
            self.value.remove(offset);
        }
    }

    pub fn handle_delete(&mut self) {
        if self.cursor < self.char_count() {
            let offset = self.byte_offset(self.cursor);
            self.value.remove(offset);
        }
    }

    pub fn cursor_left(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    pub fn cursor_right(&mut self) {
        if self.cursor < self.char_count() {
            self.cursor += 1;
        }
    }

    pub fn cursor_home(&mut self) {
        self.cursor = 0;
    }

    pub fn cursor_end(&mut self) {
        self.cursor = self.char_count();
    }

    /// Render the bordered field, and place the terminal cursor when focused.
    ///
    /// Only the focused field positions the cursor: a terminal has one, so two
    /// fields both claiming it would leave it wherever the later call put it.
    pub fn render(
        &self,
        frame: &mut Frame,
        area: Rect,
        placeholder: &str,
        focused: bool,
        has_error: bool,
    ) {
        let display = if self.value.is_empty() {
            Span::styled(
                placeholder.to_string(),
                Style::default().fg(Color::DarkGray),
            )
        } else {
            Span::styled(
                "•".repeat(self.char_count()),
                Style::default().fg(Color::White),
            )
        };

        let border = if has_error {
            Color::Red
        } else if focused {
            Color::Cyan
        } else {
            Color::DarkGray
        };

        let input = Paragraph::new(display)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(border)),
            )
            .wrap(Wrap { trim: false });
        frame.render_widget(input, area);

        if focused {
            let inner = Block::default().borders(Borders::ALL).inner(area);
            // Clamp so a value wider than the box cannot draw the cursor
            // outside it (the original cast was unbounded).
            let max_x = inner.width.saturating_sub(1);
            let offset = u16::try_from(self.cursor).unwrap_or(u16::MAX).min(max_x);
            frame.set_cursor_position((inner.x + offset, inner.y));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn typed(text: &str) -> MaskedInput {
        let mut input = MaskedInput::new();
        for c in text.chars() {
            input.handle_char(c);
        }
        input
    }

    #[test]
    fn test_new_is_empty_with_cursor_at_start() {
        let input = MaskedInput::new();
        assert_eq!(input.value(), "");
        assert_eq!(input.cursor(), 0);
        assert!(input.is_empty());
    }

    #[test]
    fn test_char_input_appends_and_advances() {
        let input = typed("ghp_abc");
        assert_eq!(input.value(), "ghp_abc");
        assert_eq!(input.cursor(), 7);
    }

    #[test]
    fn test_backspace_removes_before_cursor() {
        let mut input = typed("abc");
        input.handle_backspace();
        assert_eq!(input.value(), "ab");
        assert_eq!(input.cursor(), 2);
    }

    #[test]
    fn test_backspace_at_start_is_a_noop() {
        let mut input = typed("abc");
        input.cursor_home();
        input.handle_backspace();
        assert_eq!(input.value(), "abc");
        assert_eq!(input.cursor(), 0);
    }

    #[test]
    fn test_delete_removes_at_cursor() {
        let mut input = typed("abc");
        input.cursor_home();
        input.handle_delete();
        assert_eq!(input.value(), "bc");
        assert_eq!(input.cursor(), 0);
    }

    #[test]
    fn test_delete_at_end_is_a_noop() {
        let mut input = typed("abc");
        input.handle_delete();
        assert_eq!(input.value(), "abc");
    }

    #[test]
    fn test_cursor_movement_is_bounded() {
        let mut input = typed("abc");
        input.cursor_left();
        assert_eq!(input.cursor(), 2);
        input.cursor_home();
        assert_eq!(input.cursor(), 0);
        input.cursor_left();
        assert_eq!(input.cursor(), 0, "cursor must not go below zero");
        input.cursor_end();
        assert_eq!(input.cursor(), 3);
        input.cursor_right();
        assert_eq!(input.cursor(), 3, "cursor must not pass the end");
    }

    #[test]
    fn test_insert_in_the_middle() {
        let mut input = typed("ac");
        input.cursor_left();
        input.handle_char('b');
        assert_eq!(input.value(), "abc");
        assert_eq!(input.cursor(), 2);
    }

    #[test]
    fn test_clear_resets_value_and_cursor() {
        let mut input = typed("secret");
        input.clear();
        assert_eq!(input.value(), "");
        assert_eq!(input.cursor(), 0);
    }

    // --- the byte-vs-char bug the extraction fixes --------------------------

    #[test]
    fn test_multibyte_input_does_not_panic() {
        // The original indexed the String by byte while counting the cursor in
        // characters, so the second edit here panicked on a char boundary.
        let mut input = typed("héllo");
        assert_eq!(input.value(), "héllo");
        assert_eq!(input.cursor(), 5);

        input.handle_backspace();
        assert_eq!(input.value(), "héll");

        input.cursor_home();
        input.handle_delete();
        assert_eq!(input.value(), "éll");
    }

    #[test]
    fn test_multibyte_insert_in_the_middle_lands_on_a_char_boundary() {
        let mut input = typed("aé");
        input.cursor_left();
        input.handle_char('b');
        assert_eq!(input.value(), "abé");
    }

    #[test]
    fn test_char_count_counts_characters_not_bytes() {
        // A password rule counts characters, so the mask length must too -
        // otherwise "éé" would render four bullets for two typed characters.
        let input = typed("éé🔐");
        assert_eq!(input.char_count(), 3);
        assert!(input.value().len() > 3, "and it really is multi-byte");
    }

    #[test]
    fn test_cursor_end_on_multibyte_value() {
        let mut input = typed("🔐🔐");
        input.cursor_home();
        input.cursor_end();
        assert_eq!(input.cursor(), 2);
        input.handle_backspace();
        assert_eq!(input.value(), "🔐");
    }
}

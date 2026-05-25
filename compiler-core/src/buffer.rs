// Double-buffer character input for the lexer.
// Simulates the two-buffer sentinel model over an in-memory string.
// Full implementation in Phase 3; this file defines the contract.

/// Abstraction over any character input source.
pub trait CharStream {
    fn next_char(&mut self) -> Option<char>;
    fn peek_char(&self) -> Option<char>;
    fn unget_char(&mut self);
    fn current_lexeme(&self) -> &str;
    fn reset_lexeme_begin(&mut self);
    fn current_line(&self) -> usize;
    fn current_column(&self) -> usize;
}

/// Simulates double buffering over an in-memory string.
/// lexeme_begin tracks the start of the current lexeme for backtracking.
pub struct DoubleBuffer {
    src: Vec<char>,
    // kept for current_lexeme slicing — chars are re-encoded to UTF-8 byte offsets
    src_str: String,
    forward: usize,
    lexeme_begin: usize,
    line: usize,
    column: usize,
}

impl DoubleBuffer {
    pub fn new(source: &str) -> Self {
        Self {
            src: source.chars().collect(),
            src_str: source.to_owned(),
            forward: 0,
            lexeme_begin: 0,
            line: 1,
            column: 1,
        }
    }

    /// Converts a char-index into a byte-offset in src_str (needed for &str slicing).
    fn char_to_byte(&self, char_idx: usize) -> usize {
        self.src_str
            .char_indices()
            .nth(char_idx)
            .map(|(b, _)| b)
            .unwrap_or(self.src_str.len())
    }
}

impl CharStream for DoubleBuffer {
    fn next_char(&mut self) -> Option<char> {
        let ch = self.src.get(self.forward).copied()?;
        self.forward += 1;
        if ch == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        Some(ch)
    }

    fn peek_char(&self) -> Option<char> {
        self.src.get(self.forward).copied()
    }

    fn unget_char(&mut self) {
        // Only retreat if we haven't passed lexeme_begin
        if self.forward > self.lexeme_begin {
            self.forward -= 1;
            // Adjust line/col on retreat (best-effort; exact only within one line)
            if self.src.get(self.forward) == Some(&'\n') {
                self.line = self.line.saturating_sub(1);
                self.column = 1;
            } else {
                self.column = self.column.saturating_sub(1);
            }
        }
    }

    fn current_lexeme(&self) -> &str {
        let start = self.char_to_byte(self.lexeme_begin);
        let end = self.char_to_byte(self.forward);
        &self.src_str[start..end]
    }

    fn reset_lexeme_begin(&mut self) {
        self.lexeme_begin = self.forward;
    }

    fn current_line(&self) -> usize {
        self.line
    }
    fn current_column(&self) -> usize {
        self.column
    }
}

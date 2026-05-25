// Unit tests for the DoubleBuffer character input system (Phase 3).
use compiler_core::buffer::{CharStream, DoubleBuffer};

// ── next_char ─────────────────────────────────────────────────────────────────

#[test]
fn next_char_returns_chars_in_order() {
    let mut buf = DoubleBuffer::new("abc");
    assert_eq!(buf.next_char(), Some('a'));
    assert_eq!(buf.next_char(), Some('b'));
    assert_eq!(buf.next_char(), Some('c'));
}

#[test]
fn next_char_returns_none_at_end() {
    let mut buf = DoubleBuffer::new("x");
    buf.next_char();
    assert_eq!(buf.next_char(), None);
}

#[test]
fn next_char_empty_string() {
    let mut buf = DoubleBuffer::new("");
    assert_eq!(buf.next_char(), None);
}

// ── peek_char ─────────────────────────────────────────────────────────────────

#[test]
fn peek_char_does_not_advance() {
    let mut buf = DoubleBuffer::new("ab");
    assert_eq!(buf.peek_char(), Some('a'));
    assert_eq!(buf.peek_char(), Some('a'));
    buf.next_char();
    assert_eq!(buf.peek_char(), Some('b'));
}

#[test]
fn peek_char_at_end_is_none() {
    let mut buf = DoubleBuffer::new("z");
    buf.next_char();
    assert_eq!(buf.peek_char(), None);
}

// ── unget_char ────────────────────────────────────────────────────────────────

#[test]
fn unget_char_retreats_forward() {
    let mut buf = DoubleBuffer::new("ab");
    buf.next_char(); // consume 'a'
    buf.unget_char();
    assert_eq!(buf.next_char(), Some('a')); // re-reads 'a'
}

#[test]
fn unget_char_at_lexeme_begin_is_noop() {
    let mut buf = DoubleBuffer::new("ab");
    // forward == lexeme_begin == 0, so unget should do nothing
    buf.unget_char();
    assert_eq!(buf.next_char(), Some('a'));
}

#[test]
fn unget_char_cannot_go_before_lexeme_begin() {
    let mut buf = DoubleBuffer::new("abc");
    buf.next_char(); // 'a'
    buf.reset_lexeme_begin(); // lexeme_begin = 1
    buf.next_char(); // 'b'
    buf.unget_char(); // back to 'b'
    buf.unget_char(); // should not retreat past lexeme_begin
    assert_eq!(buf.next_char(), Some('b'));
}

// ── line / column tracking ────────────────────────────────────────────────────

#[test]
fn initial_position_is_line1_col1() {
    let buf = DoubleBuffer::new("hello");
    assert_eq!(buf.current_line(), 1);
    assert_eq!(buf.current_column(), 1);
}

#[test]
fn column_advances_with_chars() {
    let mut buf = DoubleBuffer::new("abc");
    buf.next_char(); // 'a' → col 2
    assert_eq!(buf.current_column(), 2);
    buf.next_char(); // 'b' → col 3
    assert_eq!(buf.current_column(), 3);
}

#[test]
fn newline_increments_line_resets_column() {
    let mut buf = DoubleBuffer::new("a\nb");
    buf.next_char(); // 'a'
    buf.next_char(); // '\n' → line 2, col 1
    assert_eq!(buf.current_line(), 2);
    assert_eq!(buf.current_column(), 1);
    buf.next_char(); // 'b' → col 2
    assert_eq!(buf.current_line(), 2);
    assert_eq!(buf.current_column(), 2);
}

#[test]
fn multiple_newlines_tracked() {
    let mut buf = DoubleBuffer::new("a\n\nb");
    buf.next_char(); // 'a'
    buf.next_char(); // '\n'
    buf.next_char(); // '\n'
    assert_eq!(buf.current_line(), 3);
}

// ── current_lexeme ────────────────────────────────────────────────────────────

#[test]
fn lexeme_empty_before_any_advance() {
    let buf = DoubleBuffer::new("hello");
    assert_eq!(buf.current_lexeme(), "");
}

#[test]
fn lexeme_after_advancing() {
    let mut buf = DoubleBuffer::new("hello");
    buf.next_char(); // 'h'
    buf.next_char(); // 'e'
    buf.next_char(); // 'l'
    assert_eq!(buf.current_lexeme(), "hel");
}

#[test]
fn lexeme_resets_after_reset_lexeme_begin() {
    let mut buf = DoubleBuffer::new("hello");
    buf.next_char(); // 'h'
    buf.next_char(); // 'e'
    buf.reset_lexeme_begin();
    assert_eq!(buf.current_lexeme(), "");
    buf.next_char(); // 'l'
    buf.next_char(); // 'l'
    assert_eq!(buf.current_lexeme(), "ll");
}

#[test]
fn lexeme_works_with_unicode() {
    let mut buf = DoubleBuffer::new("héllo");
    buf.next_char(); // 'h'
    buf.next_char(); // 'é'  (2-byte UTF-8)
    assert_eq!(buf.current_lexeme(), "hé");
}

// ── full scan simulation ──────────────────────────────────────────────────────

#[test]
fn scan_two_tokens() {
    let mut buf = DoubleBuffer::new("ab cd");
    // token 1: "ab"
    buf.next_char();
    buf.next_char();
    assert_eq!(buf.current_lexeme(), "ab");
    buf.reset_lexeme_begin();
    // skip space
    buf.next_char();
    buf.reset_lexeme_begin();
    // token 2: "cd"
    buf.next_char();
    buf.next_char();
    assert_eq!(buf.current_lexeme(), "cd");
}

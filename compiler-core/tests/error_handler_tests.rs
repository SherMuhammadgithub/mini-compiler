// Integration tests for Phase 11: unified error handler.
use compiler_core::error_handler::{analyze, ErrorHandler};
use compiler_core::types::{CompilerError, Token, TokenKind};

// ── ErrorHandler::add and report ─────────────────────────────────────────────

fn make_error(stage: &str, msg: &str) -> CompilerError {
    CompilerError {
        stage: stage.into(),
        message: msg.into(),
        line: 1,
        column: 0,
        length: 0,
        severity: "error".into(),
    }
}

#[test]
fn empty_handler_total_is_zero() {
    let h = ErrorHandler::new();
    assert_eq!(h.report().total, 0);
}

#[test]
fn add_increments_total() {
    let mut h = ErrorHandler::new();
    h.add(make_error("lexer", "bad char"));
    h.add(make_error("semantic", "undeclared"));
    assert_eq!(h.report().total, 2);
}

#[test]
fn report_lexical_filters_by_lexer_stage() {
    let mut h = ErrorHandler::new();
    h.add(make_error("lexer", "bad char"));
    h.add(make_error("semantic", "undeclared"));
    let r = h.report();
    assert_eq!(r.lexical.len(), 1);
    assert_eq!(r.lexical[0].stage, "lexer");
}

#[test]
fn report_syntactic_filters_parser_stages() {
    let mut h = ErrorHandler::new();
    h.add(make_error("parser", "missing ;"));
    h.add(make_error("ll1", "expand error"));
    h.add(make_error("lr", "shift error"));
    h.add(make_error("lexer", "bad char"));
    let r = h.report();
    assert_eq!(r.syntactic.len(), 3);
}

#[test]
fn report_semantic_filters_semantic_and_symbol_table() {
    let mut h = ErrorHandler::new();
    h.add(make_error("semantic", "type mismatch"));
    h.add(make_error("symbol_table", "duplicate decl"));
    h.add(make_error("lexer", "bad char"));
    let r = h.report();
    assert_eq!(r.semantic.len(), 2);
}

#[test]
fn report_total_is_sum_of_all_errors() {
    let mut h = ErrorHandler::new();
    h.add(make_error("lexer", "e1"));
    h.add(make_error("parser", "e2"));
    h.add(make_error("semantic", "e3"));
    let r = h.report();
    assert_eq!(r.total, 3);
    assert_eq!(r.lexical.len() + r.syntactic.len() + r.semantic.len(), 3);
}

// ── Panic-mode recovery ───────────────────────────────────────────────────────

fn token(kind: TokenKind) -> Token {
    Token { kind, lexeme: String::new(), line: 1, column: 0 }
}

#[test]
fn panic_mode_skip_stops_at_sync_token() {
    let tokens = vec![
        token(TokenKind::Id),
        token(TokenKind::Comma),
        token(TokenKind::Semicolon),
        token(TokenKind::End),
    ];
    let pos = ErrorHandler::panic_mode_skip(&tokens, 0, &[TokenKind::Semicolon]);
    assert_eq!(pos, 2);
}

#[test]
fn panic_mode_skip_returns_len_when_sync_not_found() {
    let tokens = vec![token(TokenKind::Id), token(TokenKind::Comma)];
    let pos = ErrorHandler::panic_mode_skip(&tokens, 0, &[TokenKind::Semicolon]);
    assert_eq!(pos, tokens.len());
}

#[test]
fn panic_mode_skip_at_sync_token_returns_same_pos() {
    let tokens = vec![token(TokenKind::Semicolon), token(TokenKind::Id)];
    let pos = ErrorHandler::panic_mode_skip(&tokens, 0, &[TokenKind::Semicolon]);
    assert_eq!(pos, 0);
}

#[test]
fn panic_mode_skip_advances_past_multiple_non_sync_tokens() {
    let tokens = vec![
        token(TokenKind::Id),
        token(TokenKind::Id),
        token(TokenKind::Id),
        token(TokenKind::End),
        token(TokenKind::Eof),
    ];
    let pos = ErrorHandler::panic_mode_skip(&tokens, 0, &[TokenKind::End, TokenKind::Eof]);
    assert_eq!(pos, 3);
}

// ── Phrase-level recovery ─────────────────────────────────────────────────────

#[test]
fn phrase_level_insert_has_correct_kind() {
    let tok = ErrorHandler::phrase_level_insert(TokenKind::Semicolon, 5);
    assert_eq!(tok.kind, TokenKind::Semicolon);
}

#[test]
fn phrase_level_insert_has_correct_column() {
    let tok = ErrorHandler::phrase_level_insert(TokenKind::Begin, 10);
    assert_eq!(tok.column, 10);
}

#[test]
fn phrase_level_insert_lexeme_is_non_empty() {
    let tok = ErrorHandler::phrase_level_insert(TokenKind::End, 0);
    assert!(!tok.lexeme.is_empty());
}

// ── Full pipeline analyze ─────────────────────────────────────────────────────

#[test]
fn valid_program_produces_zero_total_errors() {
    let r = analyze("program p ( x ) ; var n : integer ; begin n := 42 end .");
    assert_eq!(r.total, 0);
}

#[test]
fn undeclared_var_appears_in_semantic_bucket() {
    let r = analyze("program p ( x ) ; begin z := 1 end .");
    assert!(!r.semantic.is_empty());
}

#[test]
fn duplicate_var_appears_in_semantic_bucket() {
    let r = analyze("program p ( x ) ; var n : integer ; var n : real ; begin end .");
    assert!(!r.semantic.is_empty());
}

#[test]
fn valid_program_all_buckets_empty() {
    let r = analyze("program p ( x ) ; begin end .");
    assert_eq!(r.lexical.len(), 0);
    assert_eq!(r.syntactic.len(), 0);
    assert_eq!(r.semantic.len(), 0);
}

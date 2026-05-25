// Integration tests for the Pascal subset lexer (Phase 4).
use compiler_core::lexer::tokenize;
use compiler_core::types::{AddopKind, MulopKind, RelopKind, TokenKind};

fn kinds(src: &str) -> Vec<TokenKind> {
    tokenize(src).tokens.into_iter().map(|t| t.kind).collect()
}

fn first(src: &str) -> TokenKind {
    tokenize(src).tokens.into_iter().next().unwrap().kind
}

// ── Keywords ──────────────────────────────────────────────────────────────────

#[test]
fn all_20_keywords_recognized() {
    let src = "program var array of integer real function procedure \
               begin end if then else while do not and or div mod";
    let tokens = tokenize(src).tokens;
    let expected = [
        TokenKind::Program,
        TokenKind::Var,
        TokenKind::Array,
        TokenKind::Of,
        TokenKind::Integer,
        TokenKind::Real,
        TokenKind::Function,
        TokenKind::Procedure,
        TokenKind::Begin,
        TokenKind::End,
        TokenKind::If,
        TokenKind::Then,
        TokenKind::Else,
        TokenKind::While,
        TokenKind::Do,
        TokenKind::Not,
        TokenKind::And,
        TokenKind::Or,
        TokenKind::Div,
        TokenKind::Mod,
        TokenKind::Eof,
    ];
    let got: Vec<_> = tokens.iter().map(|t| t.kind.clone()).collect();
    assert_eq!(got, expected);
}

// ── Identifiers ───────────────────────────────────────────────────────────────

#[test]
fn identifier_recognized() {
    assert_eq!(first("hello"), TokenKind::Id);
    assert_eq!(first("x1"), TokenKind::Id);
    assert_eq!(first("ABC"), TokenKind::Id);
}

#[test]
fn keyword_not_mistaken_for_identifier() {
    assert_eq!(first("begin"), TokenKind::Begin);
    assert_eq!(first("end"), TokenKind::End);
}

#[test]
fn identifier_with_keyword_prefix() {
    // "beginning" must be Id, not Begin + something
    assert_eq!(first("beginning"), TokenKind::Id);
    assert_eq!(first("ender"), TokenKind::Id);
}

// ── Numeric literals ──────────────────────────────────────────────────────────

#[test]
fn integer_literal() {
    assert_eq!(first("42"), TokenKind::Num);
    assert_eq!(first("0"), TokenKind::Num);
}

#[test]
fn real_literal() {
    assert_eq!(first("3.14"), TokenKind::Num);
    assert_eq!(first("1.0E10"), TokenKind::Num);
    assert_eq!(first("2.5E-3"), TokenKind::Num);
    assert_eq!(first("6.0E+2"), TokenKind::Num);
}

#[test]
fn dotdot_not_swallowed_by_number() {
    // "1..10" must lex as Num, DotDot, Num — not misparse "1." as a real
    let k = kinds("1..10");
    assert_eq!(
        k,
        vec![
            TokenKind::Num,
            TokenKind::DotDot,
            TokenKind::Num,
            TokenKind::Eof
        ]
    );
}

// ── Operators ─────────────────────────────────────────────────────────────────

#[test]
fn relops() {
    assert_eq!(first("="), TokenKind::Relop(RelopKind::Eq));
    assert_eq!(first("<>"), TokenKind::Relop(RelopKind::Ne));
    assert_eq!(first("<"), TokenKind::Relop(RelopKind::Lt));
    assert_eq!(first("<="), TokenKind::Relop(RelopKind::Le));
    assert_eq!(first(">="), TokenKind::Relop(RelopKind::Ge));
    assert_eq!(first(">"), TokenKind::Relop(RelopKind::Gt));
}

#[test]
fn addops() {
    assert_eq!(first("+"), TokenKind::Addop(AddopKind::Plus));
    assert_eq!(first("-"), TokenKind::Addop(AddopKind::Minus));
    // 'or' is a keyword variant, not Addop
    assert_eq!(first("or"), TokenKind::Or);
}

#[test]
fn mulops() {
    assert_eq!(first("*"), TokenKind::Mulop(MulopKind::Star));
    assert_eq!(first("/"), TokenKind::Mulop(MulopKind::Slash));
    // div/mod/and are keyword variants
    assert_eq!(first("div"), TokenKind::Div);
    assert_eq!(first("mod"), TokenKind::Mod);
    assert_eq!(first("and"), TokenKind::And);
}

#[test]
fn assignop() {
    assert_eq!(first(":="), TokenKind::Assignop);
}

// ── Punctuation ───────────────────────────────────────────────────────────────

#[test]
fn punctuation_tokens() {
    assert_eq!(first("("), TokenKind::LParen);
    assert_eq!(first(")"), TokenKind::RParen);
    assert_eq!(first("["), TokenKind::LBracket);
    assert_eq!(first("]"), TokenKind::RBracket);
    assert_eq!(first(";"), TokenKind::Semicolon);
    assert_eq!(first(":"), TokenKind::Colon);
    assert_eq!(first(","), TokenKind::Comma);
    assert_eq!(first("."), TokenKind::Dot);
    assert_eq!(first(".."), TokenKind::DotDot);
}

// ── Comments & whitespace ─────────────────────────────────────────────────────

#[test]
fn comment_is_skipped() {
    let k = kinds("{ this is a comment } x");
    assert_eq!(k, vec![TokenKind::Id, TokenKind::Eof]);
}

#[test]
fn adjacent_comments_skipped() {
    let k = kinds("{a}{b} y");
    assert_eq!(k, vec![TokenKind::Id, TokenKind::Eof]);
}

#[test]
fn whitespace_skipped() {
    let k = kinds("  \t\n  x");
    assert_eq!(k, vec![TokenKind::Id, TokenKind::Eof]);
}

// ── EOF sentinel ──────────────────────────────────────────────────────────────

#[test]
fn eof_always_appended() {
    let out = tokenize("x");
    assert_eq!(out.tokens.last().unwrap().kind, TokenKind::Eof);
}

#[test]
fn empty_source_produces_only_eof() {
    let k = kinds("");
    assert_eq!(k, vec![TokenKind::Eof]);
}

// ── Line / column tracking ────────────────────────────────────────────────────

#[test]
fn first_token_at_line1_col1() {
    let out = tokenize("begin");
    let t = &out.tokens[0];
    assert_eq!(t.line, 1);
    assert_eq!(t.column, 1);
}

#[test]
fn token_column_after_spaces() {
    let out = tokenize("   x");
    let t = &out.tokens[0];
    assert_eq!(t.line, 1);
    assert_eq!(t.column, 4);
}

#[test]
fn token_on_second_line() {
    let out = tokenize("a\nb");
    let b_tok = &out.tokens[1];
    assert_eq!(b_tok.line, 2);
    assert_eq!(b_tok.column, 1);
}

// ── Error recovery ────────────────────────────────────────────────────────────

#[test]
fn unknown_char_produces_error_and_continues() {
    let out = tokenize("x @ y");
    assert_eq!(out.errors.len(), 1);
    assert_eq!(out.errors[0].stage, "lexer");
    // scanning continues — x, Unknown(@), y, EOF
    assert_eq!(out.tokens.len(), 4);
    assert_eq!(out.tokens[0].kind, TokenKind::Id);
    assert_eq!(out.tokens[1].kind, TokenKind::Unknown);
    assert_eq!(out.tokens[2].kind, TokenKind::Id);
}

// ── Realistic snippet ─────────────────────────────────────────────────────────

#[test]
fn minimal_program_header() {
    let src = "program foo ( x , y ) ;";
    let k = kinds(src);
    assert_eq!(
        k,
        vec![
            TokenKind::Program,
            TokenKind::Id,
            TokenKind::LParen,
            TokenKind::Id,
            TokenKind::Comma,
            TokenKind::Id,
            TokenKind::RParen,
            TokenKind::Semicolon,
            TokenKind::Eof,
        ]
    );
}

#[test]
fn array_type_declaration() {
    let src = "array [ 1 .. 10 ] of integer";
    let k = kinds(src);
    assert_eq!(
        k,
        vec![
            TokenKind::Array,
            TokenKind::LBracket,
            TokenKind::Num,
            TokenKind::DotDot,
            TokenKind::Num,
            TokenKind::RBracket,
            TokenKind::Of,
            TokenKind::Integer,
            TokenKind::Eof,
        ]
    );
}

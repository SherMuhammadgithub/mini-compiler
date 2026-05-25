// Lexical analyzer for the Pascal subset.
// Uses logos for pattern matching; line/column tracking via a byte→line table.
use logos::Logos;
use serde::Serialize;

use crate::types::{AddopKind, CompilerError, MulopKind, RelopKind, Token, TokenKind};

// ── Raw token (logos internal) ─────────────────────────────────────────────────

#[derive(Logos, Debug, PartialEq)]
// Skip whitespace and { ... } comments in one pass.
#[logos(skip r"[ \t\r\n]+|[{][^}]*[}]")]
enum RawToken {
    // Keywords — must appear before Id regex so logos gives them higher priority.
    #[token("program")]
    KwProgram,
    #[token("var")]
    KwVar,
    #[token("array")]
    KwArray,
    #[token("of")]
    KwOf,
    #[token("integer")]
    KwInteger,
    #[token("real")]
    KwReal,
    #[token("function")]
    KwFunction,
    #[token("procedure")]
    KwProcedure,
    #[token("begin")]
    KwBegin,
    #[token("end")]
    KwEnd,
    #[token("if")]
    KwIf,
    #[token("then")]
    KwThen,
    #[token("else")]
    KwElse,
    #[token("while")]
    KwWhile,
    #[token("do")]
    KwDo,
    #[token("not")]
    KwNot,
    #[token("and")]
    KwAnd,
    #[token("or")]
    KwOr,
    #[token("div")]
    KwDiv,
    #[token("mod")]
    KwMod,

    // Identifiers (matched only when no keyword wins).
    #[regex(r"[a-zA-Z][a-zA-Z0-9]*")]
    Id,

    // Numeric literals: integer or real (optional .digits, optional E±digits).
    #[regex(r"[0-9]+(?:\.[0-9]+(?:E[+\-]?[0-9]+)?)?")]
    Num,

    // Multi-char operators before their single-char prefixes.
    #[token(":=")]
    Assignop,
    #[token("<>")]
    Ne,
    #[token("<=")]
    Le,
    #[token(">=")]
    Ge,
    #[token("..")]
    DotDot,

    // Single-char operators.
    #[token("=")]
    Eq,
    #[token("<")]
    Lt,
    #[token(">")]
    Gt,
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,

    // Punctuation.
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,
    #[token(";")]
    Semicolon,
    #[token(":")]
    Colon,
    #[token(",")]
    Comma,
    #[token(".")]
    Dot,
}

// ── Public output type ─────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct LexerOutput {
    pub tokens: Vec<Token>,
    pub errors: Vec<CompilerError>,
}

// ── Main entry point ───────────────────────────────────────────────────────────

/// Tokenizes the entire Pascal source. Skips whitespace and comments.
/// Unknown characters produce a CompilerError and a TokenKind::Unknown token;
/// scanning continues so all errors in one pass are collected.
pub fn tokenize(source: &str) -> LexerOutput {
    let line_starts = build_line_starts(source);
    let mut tokens = Vec::new();
    let mut errors = Vec::new();

    let mut lex = RawToken::lexer(source);

    while let Some(result) = lex.next() {
        let span = lex.span();
        let lexeme = lex.slice().to_owned();
        let (line, column) = byte_to_line_col(&line_starts, span.start);

        match result {
            Ok(raw) => tokens.push(Token {
                kind: raw_to_kind(raw),
                lexeme,
                line,
                column,
            }),
            Err(()) => {
                errors.push(CompilerError {
                    stage: "lexer".to_owned(),
                    message: format!("unexpected character '{}'", lexeme),
                    line,
                    column,
                    length: span.end - span.start,
                    severity: "error".to_owned(),
                });
                tokens.push(Token {
                    kind: TokenKind::Unknown,
                    lexeme,
                    line,
                    column,
                });
            }
        }
    }

    // Append EOF sentinel.
    let (eof_line, eof_col) = byte_to_line_col(&line_starts, source.len());
    tokens.push(Token {
        kind: TokenKind::Eof,
        lexeme: "$".to_owned(),
        line: eof_line,
        column: eof_col,
    });

    LexerOutput { tokens, errors }
}

// ── Helpers ────────────────────────────────────────────────────────────────────

/// Builds a table mapping line index (0-based) → byte offset of its first char.
fn build_line_starts(source: &str) -> Vec<usize> {
    let mut starts = vec![0usize];
    for (i, ch) in source.char_indices() {
        if ch == '\n' {
            starts.push(i + 1);
        }
    }
    starts
}

/// Converts a byte offset to (line, column), both 1-based.
fn byte_to_line_col(line_starts: &[usize], byte: usize) -> (usize, usize) {
    let line_idx = match line_starts.binary_search(&byte) {
        Ok(i) => i,
        Err(i) => i - 1,
    };
    let col = byte - line_starts[line_idx] + 1;
    (line_idx + 1, col)
}

/// Maps a logos RawToken to the shared TokenKind.
fn raw_to_kind(raw: RawToken) -> TokenKind {
    match raw {
        RawToken::KwProgram => TokenKind::Program,
        RawToken::KwVar => TokenKind::Var,
        RawToken::KwArray => TokenKind::Array,
        RawToken::KwOf => TokenKind::Of,
        RawToken::KwInteger => TokenKind::Integer,
        RawToken::KwReal => TokenKind::Real,
        RawToken::KwFunction => TokenKind::Function,
        RawToken::KwProcedure => TokenKind::Procedure,
        RawToken::KwBegin => TokenKind::Begin,
        RawToken::KwEnd => TokenKind::End,
        RawToken::KwIf => TokenKind::If,
        RawToken::KwThen => TokenKind::Then,
        RawToken::KwElse => TokenKind::Else,
        RawToken::KwWhile => TokenKind::While,
        RawToken::KwDo => TokenKind::Do,
        RawToken::KwNot => TokenKind::Not,
        RawToken::KwAnd => TokenKind::And,
        RawToken::KwOr => TokenKind::Or,
        RawToken::KwDiv => TokenKind::Div,
        RawToken::KwMod => TokenKind::Mod,
        RawToken::Id => TokenKind::Id,
        RawToken::Num => TokenKind::Num,
        RawToken::Assignop => TokenKind::Assignop,
        RawToken::Eq => TokenKind::Relop(RelopKind::Eq),
        RawToken::Ne => TokenKind::Relop(RelopKind::Ne),
        RawToken::Lt => TokenKind::Relop(RelopKind::Lt),
        RawToken::Le => TokenKind::Relop(RelopKind::Le),
        RawToken::Ge => TokenKind::Relop(RelopKind::Ge),
        RawToken::Gt => TokenKind::Relop(RelopKind::Gt),
        RawToken::Plus => TokenKind::Addop(AddopKind::Plus),
        RawToken::Minus => TokenKind::Addop(AddopKind::Minus),
        RawToken::Star => TokenKind::Mulop(MulopKind::Star),
        RawToken::Slash => TokenKind::Mulop(MulopKind::Slash),
        RawToken::DotDot => TokenKind::DotDot,
        RawToken::LParen => TokenKind::LParen,
        RawToken::RParen => TokenKind::RParen,
        RawToken::LBracket => TokenKind::LBracket,
        RawToken::RBracket => TokenKind::RBracket,
        RawToken::Semicolon => TokenKind::Semicolon,
        RawToken::Colon => TokenKind::Colon,
        RawToken::Comma => TokenKind::Comma,
        RawToken::Dot => TokenKind::Dot,
    }
}

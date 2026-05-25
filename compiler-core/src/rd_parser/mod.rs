// Recursive descent parser — public API, Parser struct, and core helpers.
// Parsing methods are split across decls.rs, stmts.rs, exprs.rs.
use crate::ast::{AstNode, Span};
use crate::lexer;
use crate::types::{CompilerError, Token, TokenKind};
use serde::Serialize;

mod decls;
mod exprs;
mod stmts;

// ── Public output ──────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct RdParseOutput {
    pub ast: Option<AstNode>,
    pub errors: Vec<CompilerError>,
    pub trace: Vec<String>,
}

// ── Parser struct ──────────────────────────────────────────────────────────────

pub(super) struct Parser {
    pub tokens: Vec<Token>,
    pub pos: usize,
    pub errors: Vec<CompilerError>,
    pub trace: Vec<String>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>, lex_errors: Vec<CompilerError>) -> Self {
        Self {
            tokens,
            pos: 0,
            errors: lex_errors,
            trace: vec![],
        }
    }

    pub fn current(&self) -> &Token {
        &self.tokens[self.pos.min(self.tokens.len() - 1)]
    }

    pub fn peek(&self) -> &TokenKind {
        &self.current().kind
    }

    pub fn span_here(&self) -> Span {
        let t = self.current();
        Span {
            line: t.line,
            column: t.column,
            length: t.lexeme.len(),
        }
    }

    pub fn advance(&mut self) -> &Token {
        let idx = self.pos.min(self.tokens.len() - 1);
        if self.pos < self.tokens.len() - 1 {
            self.pos += 1;
        }
        &self.tokens[idx]
    }

    pub fn at(&self, kind: &TokenKind) -> bool {
        self.peek() == kind
    }
    pub fn at_eof(&self) -> bool {
        matches!(self.peek(), TokenKind::Eof)
    }

    pub fn expect(&mut self, kind: &TokenKind) -> Option<String> {
        if self.peek() == kind {
            Some(self.advance().lexeme.clone())
        } else {
            let t = self.current();
            self.errors.push(CompilerError {
                stage: "rd_parser".into(),
                message: format!("expected '{}', found '{}'", token_name(kind), t.lexeme),
                line: t.line,
                column: t.column,
                length: t.lexeme.len(),
                severity: "error".into(),
            });
            None
        }
    }

    pub fn expect_id(&mut self) -> String {
        if matches!(self.peek(), TokenKind::Id) {
            self.advance().lexeme.clone()
        } else {
            let t = self.current();
            self.errors.push(CompilerError {
                stage: "rd_parser".into(),
                message: format!("expected identifier, found '{}'", t.lexeme),
                line: t.line,
                column: t.column,
                length: t.lexeme.len(),
                severity: "error".into(),
            });
            "<error>".into()
        }
    }

    pub fn expect_num(&mut self) -> String {
        if matches!(self.peek(), TokenKind::Num) {
            self.advance().lexeme.clone()
        } else {
            let t = self.current();
            self.errors.push(CompilerError {
                stage: "rd_parser".into(),
                message: format!("expected number, found '{}'", t.lexeme),
                line: t.line,
                column: t.column,
                length: t.lexeme.len(),
                severity: "error".into(),
            });
            "0".into()
        }
    }

    pub fn synchronize(&mut self, sync: &[TokenKind]) {
        while !self.at_eof() && !sync.iter().any(|k| self.peek() == k) {
            self.advance();
        }
    }

    pub fn push_trace(&mut self, name: &str) {
        self.trace.push(format!("→ {}", name));
    }
}

// ── Internal helper used by decls.rs ──────────────────────────────────────────

pub(super) enum SubprogramHead {
    Function {
        name: String,
        params: Vec<AstNode>,
        return_type: AstNode,
    },
    Procedure {
        name: String,
        params: Vec<AstNode>,
    },
}

// ── Token name for error messages ─────────────────────────────────────────────

pub(super) fn token_name(k: &TokenKind) -> &'static str {
    match k {
        TokenKind::Program => "program",
        TokenKind::Var => "var",
        TokenKind::Begin => "begin",
        TokenKind::End => "end",
        TokenKind::If => "if",
        TokenKind::Then => "then",
        TokenKind::Else => "else",
        TokenKind::While => "while",
        TokenKind::Do => "do",
        TokenKind::Not => "not",
        TokenKind::Function => "function",
        TokenKind::Procedure => "procedure",
        TokenKind::Array => "array",
        TokenKind::Of => "of",
        TokenKind::Integer => "integer",
        TokenKind::Real => "real",
        TokenKind::LParen => "(",
        TokenKind::RParen => ")",
        TokenKind::LBracket => "[",
        TokenKind::RBracket => "]",
        TokenKind::Semicolon => ";",
        TokenKind::Colon => ":",
        TokenKind::Comma => ",",
        TokenKind::Dot => ".",
        TokenKind::DotDot => "..",
        TokenKind::Assignop => ":=",
        TokenKind::Eof => "$",
        _ => "token",
    }
}

// ── Public entry point ────────────────────────────────────────────────────────

pub fn parse(source: &str) -> RdParseOutput {
    let lex_out = lexer::tokenize(source);
    let mut parser = Parser::new(lex_out.tokens, lex_out.errors);
    let ast = parser.parse_program();
    RdParseOutput {
        ast,
        errors: parser.errors,
        trace: parser.trace,
    }
}

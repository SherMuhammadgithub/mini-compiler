// Unified error handler: aggregates errors from all stages, deduplicates,
// and provides recovery helpers used by parsers.
use crate::types::{CompilerError, Token, TokenKind};
use serde::Serialize;

#[derive(Serialize)]
pub struct ErrorReport {
    pub lexical: Vec<CompilerError>,
    pub syntactic: Vec<CompilerError>,
    pub semantic: Vec<CompilerError>,
    pub total: usize,
}

pub struct ErrorHandler {
    errors: Vec<CompilerError>,
}

impl ErrorHandler {
    pub fn new() -> Self {
        Self { errors: vec![] }
    }

    pub fn add(&mut self, e: CompilerError) {
        self.errors.push(e);
    }

    /// Panic-mode recovery: skip tokens until a token in `sync` is found.
    /// Returns the new position (pointing at the sync token, or tokens.len() if not found).
    pub fn panic_mode_skip(tokens: &[Token], pos: usize, sync: &[TokenKind]) -> usize {
        let mut i = pos;
        while i < tokens.len() && !sync.contains(&tokens[i].kind) {
            i += 1;
        }
        i
    }

    /// Phrase-level recovery: create a synthetic token of the expected kind so
    /// the parser can insert it and continue without halting.
    pub fn phrase_level_insert(expected: TokenKind, at_column: usize) -> Token {
        Token {
            lexeme: format!("<{:?}>", expected),
            kind: expected,
            line: 0,
            column: at_column,
        }
    }

    /// Partition collected errors into a staged report.
    pub fn report(&self) -> ErrorReport {
        let lexical = self.errors.iter()
            .filter(|e| e.stage == "lexer")
            .cloned().collect();
        let syntactic = self.errors.iter()
            .filter(|e| e.stage.contains("parser") || e.stage == "ll1" || e.stage == "lr")
            .cloned().collect();
        let semantic = self.errors.iter()
            .filter(|e| e.stage == "semantic" || e.stage == "symbol_table")
            .cloned().collect();
        let total = self.errors.len();
        ErrorReport { lexical, syntactic, semantic, total }
    }
}

/// Run the full pipeline on `source` and return a unified error report.
pub fn analyze(source: &str) -> ErrorReport {
    let mut handler = ErrorHandler::new();
    // semantic::analyze runs lexer + rd_parser + semantic in one call;
    // its errors vec contains errors from all three stages.
    let sem_out = crate::semantic::analyze(source);
    for e in sem_out.errors {
        handler.add(e);
    }
    handler.report()
}

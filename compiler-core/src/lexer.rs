// Lexical analyzer for the Pascal subset.
// Full logos-based implementation in Phase 4.
// This stub satisfies the WASM export contract so cargo check passes.
use crate::types::{CompilerError, Token};
use serde::Serialize;

#[derive(Serialize)]
pub struct LexerOutput {
    pub tokens: Vec<Token>,
    pub errors: Vec<CompilerError>,
}

/// Tokenize Pascal subset source text. Returns all tokens and any lexical errors.
pub fn tokenize(_source: &str) -> LexerOutput {
    LexerOutput {
        tokens: vec![],
        errors: vec![],
    }
}

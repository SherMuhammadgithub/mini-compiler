// Recursive descent parser for the Pascal subset.
// Full implementation in Phase 6. Stub satisfies WASM export contract.
use crate::ast::AstNode;
use crate::types::CompilerError;
use serde::Serialize;

#[derive(Serialize)]
pub struct RdParseOutput {
    pub ast: Option<AstNode>, // None only if the root production itself fails
    pub errors: Vec<CompilerError>,
    pub trace: Vec<String>, // "→ parse_program", "→ parse_declarations", ...
}

/// Parse Pascal subset source and return an AST + any syntax errors.
pub fn parse(_source: &str) -> RdParseOutput {
    RdParseOutput {
        ast: None,
        errors: vec![],
        trace: vec![],
    }
}

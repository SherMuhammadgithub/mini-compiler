// Semantic analysis — type checking using the symbol table.
// Full implementation in Phase 13.
use crate::types::{CompilerError, SymbolEntry};
use serde::Serialize;

#[derive(Serialize)]
pub struct SemanticOutput {
    /// Snapshots of the symbol table as each scope is entered/exited.
    pub symbol_snapshots: Vec<Vec<SymbolEntry>>,
    pub errors: Vec<CompilerError>,
}

/// Run lexer → RD parser → semantic analysis. Returns symbol table + errors.
pub fn analyze(_source: &str) -> SemanticOutput {
    SemanticOutput {
        symbol_snapshots: vec![],
        errors: vec![],
    }
}

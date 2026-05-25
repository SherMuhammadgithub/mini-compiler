// LALR(1) shift-reduce parser driver — explicit state stack, while loop, zero recursion.
// Full implementation in Phase 12.
use crate::types::CompilerError;
use serde::Serialize;

#[derive(Serialize)]
pub struct LrParseOutput {
    pub trace: Vec<String>, // each step: "Shift id", "Reduce statement → ..."
    pub errors: Vec<CompilerError>,
    pub accepted: bool,
}

/// Run the LR shift-reduce parser on Pascal subset source.
pub fn parse(_source: &str) -> LrParseOutput {
    LrParseOutput {
        trace: vec![],
        errors: vec![],
        accepted: false,
    }
}

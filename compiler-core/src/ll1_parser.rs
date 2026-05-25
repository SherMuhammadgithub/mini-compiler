// Non-recursive LL(1) predictive parser — explicit stack, while loop, zero recursion.
// Full implementation in Phase 8.
use crate::types::CompilerError;
use serde::Serialize;

#[derive(Serialize)]
pub struct Ll1ParseOutput {
    pub trace: Vec<String>, // each step: "Match id", "Predict statement → ..."
    pub errors: Vec<CompilerError>,
    pub accepted: bool,
}

/// Run the LL(1) predictive parser on Pascal subset source.
pub fn parse(_source: &str) -> Ll1ParseOutput {
    Ll1ParseOutput {
        trace: vec![],
        errors: vec![],
        accepted: false,
    }
}

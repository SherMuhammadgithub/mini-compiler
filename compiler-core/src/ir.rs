// Three-address code (TAC) IR generation from the typed AST.
// Full implementation in Phase 15.
use crate::types::{CompilerError, TacInstr};
use serde::Serialize;

#[derive(Serialize)]
pub struct IrOutput {
    pub instructions: Vec<TacInstr>,
    pub errors: Vec<CompilerError>,
}

/// Run the full front-end pipeline and emit TAC quadruples.
pub fn generate(_source: &str) -> IrOutput {
    IrOutput {
        instructions: vec![],
        errors: vec![],
    }
}

// Stack VM bytecode generation from TAC quadruples.
// Full implementation in Phase 16.
use crate::types::{CompilerError, VmInstr};
use serde::Serialize;

#[derive(Serialize)]
pub struct CodegenOutput {
    pub bytecode: Vec<VmInstr>,
    pub errors: Vec<CompilerError>,
}

/// Translate TAC to stack VM bytecode.
pub fn generate(_source: &str) -> CodegenOutput {
    CodegenOutput {
        bytecode: vec![],
        errors: vec![],
    }
}

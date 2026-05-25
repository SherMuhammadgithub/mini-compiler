// Stack VM interpreter — executes bytecode, produces program output.
// Full implementation in Phase 17.
use crate::types::CompilerError;
use serde::Serialize;

#[derive(Serialize)]
pub struct VmOutput {
    pub output: String, // everything printed by the Pascal program
    pub errors: Vec<CompilerError>,
}

/// Compile and execute a Pascal subset program; return its stdout + any runtime errors.
pub fn execute(_source: &str, _input: &str) -> VmOutput {
    VmOutput {
        output: String::new(),
        errors: vec![],
    }
}

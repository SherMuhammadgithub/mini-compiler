// WASM entry points for the mini Pascal compiler.
// Each export runs one pipeline stage and returns JSON.
// No logic lives here — every export delegates to its stage module.
use wasm_bindgen::prelude::*;

pub mod ast;
pub mod buffer;
mod codegen;
pub mod error_handler;
pub mod first_follow;
mod grammar;
pub mod ir;
pub mod lexer;
pub mod ll1_parser;
mod ll1_table;
mod lr_items;
pub mod lr_parser;
mod lr_table;
pub mod rd_parser;
pub mod semantic;
pub mod symbol_table;
pub mod types;
mod vm;

/// Serialize any value to JSON; on failure return an error JSON object.
fn to_json<T: serde::Serialize>(v: &T) -> String {
    serde_json::to_string(v).unwrap_or_else(|e| format!("{{\"error\":\"{}\"}}", e))
}

#[wasm_bindgen]
pub fn run_lexer(source: &str) -> String {
    to_json(&lexer::tokenize(source))
}

#[wasm_bindgen]
pub fn run_rd_parser(source: &str) -> String {
    to_json(&rd_parser::parse(source))
}

#[wasm_bindgen]
pub fn run_ll1_parser(source: &str) -> String {
    to_json(&ll1_parser::parse(source))
}

#[wasm_bindgen]
pub fn run_lr_parser(source: &str) -> String {
    to_json(&lr_parser::parse(source))
}

#[wasm_bindgen]
pub fn run_symbol_table(source: &str) -> String {
    to_json(&symbol_table::analyze(source))
}

#[wasm_bindgen]
pub fn run_error_handler(source: &str) -> String {
    to_json(&error_handler::analyze(source))
}

#[wasm_bindgen]
pub fn run_ir(source: &str) -> String {
    to_json(&ir::generate(source))
}

#[wasm_bindgen]
pub fn run_codegen(source: &str) -> String {
    to_json(&codegen::generate(source))
}

#[wasm_bindgen]
pub fn run_program(source: &str, input: &str) -> String {
    to_json(&vm::execute(source, input))
}

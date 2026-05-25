// Integration tests for the symbol table (Phase 9) via symbol_table::analyze.
use compiler_core::symbol_table::analyze;
use compiler_core::types::SymbolKind;

fn has_entry(src: &str, name: &str) -> bool {
    analyze(src).entries.iter().any(|e| e.name == name)
}

fn entry_kind(src: &str, name: &str) -> Option<SymbolKind> {
    analyze(src).entries.iter().find(|e| e.name == name).map(|e| e.kind.clone())
}

fn has_semantic_error(src: &str) -> bool {
    analyze(src).errors.iter().any(|e| e.stage == "semantic" || e.stage == "symbol_table")
}

// ── Declarations inserted ─────────────────────────────────────────────────────

#[test]
fn program_param_in_table() {
    assert!(has_entry("program p ( x ) ; begin end .", "x"));
}

#[test]
fn var_decl_in_table() {
    assert!(has_entry(
        "program p ( x ) ; var n : integer ; begin end .",
        "n"
    ));
}

#[test]
fn multiple_vars_in_table() {
    let src = "program p ( x ) ; var a , b : integer ; begin end .";
    assert!(has_entry(src, "a"));
    assert!(has_entry(src, "b"));
}

#[test]
fn real_var_in_table() {
    let out = analyze("program p ( x ) ; var r : real ; begin end .");
    let e = out.entries.iter().find(|e| e.name == "r").unwrap();
    assert!(matches!(e.pascal_type, compiler_core::types::PascalType::Real));
}

#[test]
fn array_var_in_table() {
    let out = analyze(
        "program p ( x ) ; var a : array [ 1 .. 5 ] of integer ; begin end ."
    );
    let e = out.entries.iter().find(|e| e.name == "a").unwrap();
    assert!(matches!(e.pascal_type, compiler_core::types::PascalType::Array { .. }));
}

#[test]
fn procedure_in_table() {
    let src = "program p ( x ) ; procedure greet ; begin end ; begin end .";
    assert_eq!(entry_kind(src, "greet"), Some(SymbolKind::Procedure));
}

#[test]
fn function_in_table() {
    let src = "program p ( x ) ; function sq ( n : integer ) : integer ; begin sq := n * n end ; begin end .";
    assert_eq!(entry_kind(src, "sq"), Some(SymbolKind::Function));
}

#[test]
fn function_param_in_scope_dump() {
    let src = "program p ( x ) ; function sq ( n : integer ) : integer ; begin sq := n * n end ; begin end .";
    let out = analyze(src);
    assert!(!out.scope_dumps.is_empty());
    assert!(out.scope_dumps[0].entries.iter().any(|e| e.name == "n"));
}

#[test]
fn param_kind_is_parameter() {
    let src = "program p ( x ) ; function sq ( n : integer ) : integer ; begin sq := n * n end ; begin end .";
    let out = analyze(src);
    let n = out.scope_dumps[0].entries.iter().find(|e| e.name == "n").unwrap();
    assert_eq!(n.kind, SymbolKind::Parameter);
}

// ── Scope levels ──────────────────────────────────────────────────────────────

#[test]
fn global_var_at_level_zero() {
    let out = analyze("program p ( x ) ; var n : integer ; begin end .");
    let n = out.entries.iter().find(|e| e.name == "n").unwrap();
    assert_eq!(n.scope_level, 0);
}

#[test]
fn function_at_level_zero() {
    let src = "program p ( x ) ; function sq ( n : integer ) : integer ; begin sq := n * n end ; begin end .";
    let out = analyze(src);
    let sq = out.entries.iter().find(|e| e.name == "sq").unwrap();
    assert_eq!(sq.scope_level, 0);
}

// ── Error detection ───────────────────────────────────────────────────────────

#[test]
fn duplicate_var_is_error() {
    let src = "program p ( x ) ; var n : integer ; var n : real ; begin end .";
    assert!(has_semantic_error(src));
}

#[test]
fn undeclared_var_in_assignment_is_error() {
    let src = "program p ( x ) ; begin y := 1 end .";
    assert!(has_semantic_error(src));
}

#[test]
fn declared_var_no_error() {
    let src = "program p ( x ) ; var n : integer ; begin n := 1 end .";
    assert!(!has_semantic_error(src));
}

#[test]
fn program_param_is_usable() {
    // x is inserted as a program parameter — assigning it should NOT produce undeclared error
    let src = "program p ( x ) ; begin x := 1 end .";
    assert!(!has_semantic_error(src));
}

#[test]
fn builtin_writeln_no_error() {
    let src = "program p ( x ) ; begin writeln end .";
    assert!(!has_semantic_error(src));
}

#[test]
fn no_errors_for_valid_program() {
    let src = "program p ( x ) ; var n : integer ; begin n := 42 end .";
    assert!(!has_semantic_error(src));
}

// ── Shadowing ─────────────────────────────────────────────────────────────────

#[test]
fn inner_param_shadows_outer_without_error() {
    // Function parameter `n` is in inner scope; global has no `n`
    let src = "program p ( x ) ; function sq ( n : integer ) : integer ; begin sq := n * n end ; begin end .";
    let out = analyze(src);
    // No undeclared errors expected
    assert!(!has_semantic_error(src));
    // `n` lives only in the function's scope dump, not in global entries
    assert!(!out.entries.iter().any(|e| e.name == "n"));
}

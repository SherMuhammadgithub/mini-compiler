// Integration tests for Phase 10: semantic analysis and type checking.
use compiler_core::semantic::analyze;
use compiler_core::types::PascalType;

fn errors(src: &str) -> Vec<String> {
    analyze(src).errors.iter()
        .filter(|e| e.stage == "semantic")
        .map(|e| e.message.clone())
        .collect()
}

fn has_error(src: &str) -> bool {
    !errors(src).is_empty()
}

fn any_error_contains(src: &str, substr: &str) -> bool {
    errors(src).iter().any(|m| m.contains(substr))
}

// ── typed_ast ─────────────────────────────────────────────────────────────────

#[test]
fn typed_ast_some_for_valid_program() {
    let out = analyze("program p ( x ) ; var n : integer ; begin n := 42 end .");
    assert!(out.typed_ast.is_some());
}

// ── Assignment type checks ────────────────────────────────────────────────────

#[test]
fn assign_int_to_int_no_error() {
    let src = "program p ( x ) ; var n : integer ; begin n := 1 end .";
    assert!(!has_error(src));
}

#[test]
fn assign_real_to_real_no_error() {
    let src = "program p ( x ) ; var r : real ; begin r := 3 end .";
    assert!(!has_error(src));
}

#[test]
fn assign_int_to_real_widening_no_error() {
    // integer literal is widened to real — must NOT produce an error
    let src = "program p ( x ) ; var r : real ; begin r := 1 end .";
    assert!(!has_error(src));
}

#[test]
fn assign_bool_expr_to_int_is_error() {
    // relational expression yields boolean — cannot assign to integer
    let src = "program p ( x ) ; var n : integer ; begin n := 1 = 1 end .";
    assert!(has_error(src));
}

// ── Undeclared identifier ─────────────────────────────────────────────────────

#[test]
fn undeclared_assignment_target_is_error() {
    let src = "program p ( x ) ; begin z := 1 end .";
    assert!(any_error_contains(src, "undeclared"));
}

#[test]
fn undeclared_rhs_variable_is_error() {
    let src = "program p ( x ) ; var n : integer ; begin n := z end .";
    assert!(any_error_contains(src, "undeclared"));
}

#[test]
fn declared_var_no_undeclared_error() {
    let src = "program p ( x ) ; var n : integer ; begin n := 1 end .";
    assert!(!any_error_contains(src, "undeclared"));
}

// ── Array index type check ────────────────────────────────────────────────────

#[test]
fn array_integer_index_no_error() {
    let src = "program p ( x ) ; var a : array [ 1 .. 5 ] of integer ; begin a [ 1 ] := 7 end .";
    assert!(!has_error(src));
}

#[test]
fn array_real_index_is_error() {
    // real literal as index — must be integer
    let src = "program p ( x ) ; var a : array [ 1 .. 5 ] of integer ; var r : real ; begin a [ r ] := 7 end .";
    assert!(any_error_contains(src, "index"));
}

#[test]
fn subscript_on_non_array_is_error() {
    let src = "program p ( x ) ; var n : integer ; begin n [ 1 ] := 7 end .";
    assert!(any_error_contains(src, "not an array"));
}

// ── Function arity and argument types ────────────────────────────────────────

#[test]
fn correct_arity_no_error() {
    let src = "program p ( x ) ; function sq ( n : integer ) : integer ; begin sq := n * n end ; begin sq ( 3 ) end .";
    assert!(!has_error(src));
}

#[test]
fn wrong_arity_is_error() {
    let src = "program p ( x ) ; function sq ( n : integer ) : integer ; begin sq := n * n end ; begin sq ( 1 , 2 ) end .";
    assert!(any_error_contains(src, "expects"));
}

#[test]
fn wrong_arg_type_is_error() {
    // Pass a real variable where integer expected — real cannot narrow to integer
    let src = "program p ( x ) ; var r : real ; function sq ( n : integer ) : integer ; begin sq := n * n end ; begin sq ( r ) end .";
    assert!(any_error_contains(src, "argument"));
}

#[test]
fn int_arg_to_int_param_no_error() {
    let src = "program p ( x ) ; function sq ( n : integer ) : integer ; begin sq := n * n end ; begin sq ( 5 ) end .";
    assert!(!has_error(src));
}

// ── Procedure call ────────────────────────────────────────────────────────────

#[test]
fn procedure_correct_call_no_error() {
    let src = "program p ( x ) ; procedure greet ; begin end ; begin greet end .";
    assert!(!has_error(src));
}

#[test]
fn builtin_writeln_no_error() {
    let src = "program p ( x ) ; begin writeln end .";
    assert!(!has_error(src));
}

#[test]
fn builtin_write_no_error() {
    let src = "program p ( x ) ; var n : integer ; begin write ( n ) end .";
    assert!(!has_error(src));
}

// ── Binary expression type inference ─────────────────────────────────────────

#[test]
fn arithmetic_int_int_infers_integer() {
    // Both sides integer — result is integer; assigning to integer: no error
    let src = "program p ( x ) ; var n : integer ; begin n := 2 + 3 end .";
    assert!(!has_error(src));
}

#[test]
fn arithmetic_real_int_infers_real() {
    // One side is a real variable — result widens to real; assigning to real: no error
    let src = "program p ( x ) ; var r : real ; begin r := r + 1 end .";
    assert!(!has_error(src));
}

#[test]
fn relational_expr_infers_boolean() {
    // 1 = 1 is boolean; assigning boolean to integer should be an error
    let src = "program p ( x ) ; var n : integer ; begin n := 1 = 1 end .";
    assert!(any_error_contains(src, "boolean"));
}

// ── Return value assignment (function self-assignment) ────────────────────────

#[test]
fn function_return_assignment_no_error() {
    // sq := n * n  — assigning to function name (return value) is valid Pascal
    let src = "program p ( x ) ; function sq ( n : integer ) : integer ; begin sq := n * n end ; begin end .";
    assert!(!has_error(src));
}

// ── Inferred return type ──────────────────────────────────────────────────────

#[test]
fn function_return_type_in_snapshot() {
    let src = "program p ( x ) ; function sq ( n : integer ) : integer ; begin sq := n * n end ; begin end .";
    let out = analyze(src);
    let sq = out.symbol_snapshot.iter().find(|e| e.name == "sq").unwrap();
    assert_eq!(sq.pascal_type, PascalType::Integer);
}

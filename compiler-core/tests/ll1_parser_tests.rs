// Integration tests for the LL(1) non-recursive predictive parser (Phase 7).
use compiler_core::ll1_parser::parse;

fn accepted(src: &str) -> bool {
    parse(src).accepted
}

fn has_errors(src: &str) -> bool {
    !parse(src).errors.is_empty()
}

// ── Valid programs ────────────────────────────────────────────────────────────

#[test]
fn empty_program_accepted() {
    assert!(accepted("program p ( x ) ; begin end ."));
}

#[test]
fn program_with_var_declaration() {
    assert!(accepted("program p ( x ) ; var n : integer ; begin end ."));
}

#[test]
fn program_with_real_var() {
    assert!(accepted("program p ( x ) ; var r : real ; begin end ."));
}

#[test]
fn program_with_array_var() {
    assert!(accepted(
        "program p ( x ) ; var a : array [ 1 .. 10 ] of integer ; begin end ."
    ));
}

#[test]
fn assignment_accepted() {
    assert!(accepted(
        "program p ( x ) ; var n : integer ; begin n := 42 end ."
    ));
}

#[test]
fn arithmetic_expression() {
    assert!(accepted(
        "program p ( x ) ; var n : integer ; begin n := 1 + 2 * 3 end ."
    ));
}

#[test]
fn relational_expression() {
    assert!(accepted(
        "program p ( x ) ; var a , b : integer ; begin if a = b then a := 0 else b := 0 end ."
    ));
}

#[test]
fn if_else_accepted() {
    assert!(accepted(
        "program p ( x ) ; var n : integer ; begin if n > 0 then n := 1 else n := 0 end ."
    ));
}

#[test]
fn while_accepted() {
    assert!(accepted(
        "program p ( x ) ; var i : integer ; begin while i < 10 do i := i + 1 end ."
    ));
}

#[test]
fn procedure_call_no_args() {
    assert!(accepted("program p ( x ) ; begin writeln end ."));
}

#[test]
fn procedure_call_with_args() {
    assert!(accepted(
        "program p ( x ) ; var n : integer ; begin write ( n ) end ."
    ));
}

#[test]
fn unary_minus() {
    assert!(accepted(
        "program p ( x ) ; var n : integer ; begin n := - 5 end ."
    ));
}

#[test]
fn not_expression() {
    assert!(accepted(
        "program p ( x ) ; var b : integer ; begin if not b = 0 then b := 1 else b := 0 end ."
    ));
}

#[test]
fn parenthesised_expression() {
    assert!(accepted(
        "program p ( x ) ; var n : integer ; begin n := ( 1 + 2 ) * 3 end ."
    ));
}

#[test]
fn procedure_declaration() {
    assert!(accepted(
        "program p ( x ) ; procedure greet ; begin end ; begin end ."
    ));
}

#[test]
fn function_declaration() {
    assert!(accepted(
        "program p ( x ) ; function sq ( n : integer ) : integer ; begin sq := n * n end ; begin end ."
    ));
}

// ── Trace output ──────────────────────────────────────────────────────────────

#[test]
fn trace_is_non_empty() {
    let out = parse("program p ( x ) ; begin end .");
    assert!(!out.trace.is_empty());
}

#[test]
fn trace_first_step_is_predict_program() {
    let out = parse("program p ( x ) ; begin end .");
    assert!(
        out.trace[0].action.contains("Predict"),
        "first step should be a Predict action"
    );
}

#[test]
fn trace_last_step_is_accept() {
    let out = parse("program p ( x ) ; begin end .");
    let last = out.trace.last().unwrap();
    assert_eq!(last.action, "Accept");
}

// ── Error detection ───────────────────────────────────────────────────────────

#[test]
fn missing_program_keyword_is_error() {
    assert!(has_errors("foo ( x ) ; begin end ."));
}

#[test]
fn missing_dot_at_end_is_error() {
    assert!(has_errors("program p ( x ) ; begin end"));
}

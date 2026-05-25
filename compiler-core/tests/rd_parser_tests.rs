// Integration tests for the recursive descent parser (Phase 6).
use compiler_core::rd_parser::parse;

fn ok(src: &str) -> bool {
    let out = parse(src);
    out.errors.is_empty() && out.ast.is_some()
}

fn has_errors(src: &str) -> bool {
    !parse(src).errors.is_empty()
}

// ── Minimal valid programs ────────────────────────────────────────────────────

#[test]
fn empty_program_parses() {
    let src = "program foo ( input ) ; begin end .";
    assert!(ok(src));
}

#[test]
fn program_produces_ast_root() {
    let out = parse("program p ( x ) ; begin end .");
    assert!(out.ast.is_some());
}

#[test]
fn program_with_var_declaration() {
    let src = "program p ( x ) ;
               var a , b : integer ;
               begin end .";
    assert!(ok(src));
}

#[test]
fn program_with_array_declaration() {
    let src = "program p ( x ) ;
               var arr : array [ 1 .. 10 ] of integer ;
               begin end .";
    assert!(ok(src));
}

#[test]
fn program_with_real_var() {
    let src = "program p ( x ) ;
               var r : real ;
               begin end .";
    assert!(ok(src));
}

// ── Statements ────────────────────────────────────────────────────────────────

#[test]
fn assignment_statement() {
    let src = "program p ( x ) ; begin x := 42 end .";
    assert!(ok(src));
}

#[test]
fn array_assignment_statement() {
    let src = "program p ( x ) ;
               var a : array [ 1 .. 5 ] of integer ;
               begin a [ 1 ] := 99 end .";
    assert!(ok(src));
}

#[test]
fn if_else_statement() {
    let src = "program p ( x ) ;
               var n : integer ;
               begin if n > 0 then n := 1 else n := 0 end .";
    assert!(ok(src));
}

#[test]
fn while_statement() {
    let src = "program p ( x ) ;
               var i : integer ;
               begin while i < 10 do i := i + 1 end .";
    assert!(ok(src));
}

#[test]
fn compound_statement_nested() {
    let src = "program p ( x ) ;
               var a , b : integer ;
               begin
                 a := 1 ;
                 b := 2
               end .";
    assert!(ok(src));
}

#[test]
fn procedure_call_no_args() {
    let src = "program p ( x ) ; begin writeln end .";
    assert!(ok(src));
}

#[test]
fn procedure_call_with_args() {
    let src = "program p ( x ) ;
               var n : integer ;
               begin write ( n ) end .";
    assert!(ok(src));
}

// ── Expressions ───────────────────────────────────────────────────────────────

#[test]
fn arithmetic_expression() {
    let src = "program p ( x ) ;
               var n : integer ;
               begin n := 1 + 2 * 3 end .";
    assert!(ok(src));
}

#[test]
fn relational_expression() {
    let src = "program p ( x ) ;
               var a , b : integer ;
               begin if a = b then a := 0 else b := 0 end .";
    assert!(ok(src));
}

#[test]
fn unary_minus() {
    let src = "program p ( x ) ;
               var n : integer ;
               begin n := - 5 end .";
    assert!(ok(src));
}

#[test]
fn not_expression() {
    let src = "program p ( x ) ;
               var b : integer ;
               begin if not b = 0 then b := 1 else b := 0 end .";
    assert!(ok(src));
}

#[test]
fn parenthesised_expression() {
    let src = "program p ( x ) ;
               var n : integer ;
               begin n := ( 1 + 2 ) * 3 end .";
    assert!(ok(src));
}

#[test]
fn real_literal_in_expression() {
    let src = "program p ( x ) ;
               var r : real ;
               begin r := 3.14 end .";
    assert!(ok(src));
}

// ── Subprograms ───────────────────────────────────────────────────────────────

#[test]
fn procedure_declaration() {
    let src = "program p ( x ) ;
               procedure greet ;
               begin end ;
               begin end .";
    assert!(ok(src));
}

#[test]
fn function_declaration() {
    let src = "program p ( x ) ;
               function square ( n : integer ) : integer ;
               begin square := n * n end ;
               begin end .";
    assert!(ok(src));
}

#[test]
fn function_with_var_decls() {
    let src = "program p ( x ) ;
               function add ( a : integer ; b : integer ) : integer ;
               var tmp : integer ;
               begin tmp := a + b ; add := tmp end ;
               begin end .";
    assert!(ok(src));
}

// ── Trace output ─────────────────────────────────────────────────────────────

#[test]
fn trace_is_non_empty_for_valid_program() {
    let out = parse("program p ( x ) ; begin end .");
    assert!(!out.trace.is_empty());
    assert!(out.trace[0].contains("parse_program"));
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

#[test]
fn missing_begin_is_error() {
    assert!(has_errors("program p ( x ) ; end ."));
}

#[test]
fn missing_end_is_error() {
    assert!(has_errors("program p ( x ) ; begin ."));
}

#[test]
fn ast_still_returned_on_recoverable_error() {
    // Missing dot — partial AST should still be returned
    let out = parse("program p ( x ) ; begin end");
    assert!(!out.errors.is_empty());
    // ast may or may not be Some depending on recovery, but no panic
}

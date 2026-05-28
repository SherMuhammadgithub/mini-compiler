// Integration tests for Phase 14: Stack VM interpreter.
use compiler_core::vm::execute;

fn run(src: &str) -> compiler_core::vm::VmOutput {
    execute(src, "")
}

fn run_with(src: &str, input: &str) -> compiler_core::vm::VmOutput {
    execute(src, input)
}

// ── Execution state ────────────────────────────────────────────────────────────

#[test]
fn empty_program_halts_cleanly() {
    let out = run("program p ( x ) ; begin end .");
    assert!(out.halted);
    assert!(out.errors.is_empty());
}

#[test]
fn non_empty_program_has_positive_step_count() {
    let out = run("program p ( x ) ; var n : integer ; begin n := 42 end .");
    assert!(out.step_count > 0);
}

#[test]
fn valid_program_produces_no_errors() {
    let out = run("program p ( x ) ; var n : integer ; begin n := 1 end .");
    assert!(out.errors.is_empty());
}

#[test]
fn semantic_error_prevents_execution() {
    let out = run("program p ( x ) ; begin z := 1 end .");
    assert!(!out.errors.is_empty());
    assert!(out.stdout.is_empty());
}

// ── Write output ───────────────────────────────────────────────────────────────

#[test]
fn write_integer_literal() {
    let src = "program p ( x ) ; begin write ( 42 ) end .";
    let out = run(src);
    assert_eq!(out.stdout, vec!["42"]);
}

#[test]
fn write_variable_after_assignment() {
    let src = "program p ( x ) ; var n : integer ; begin n := 99 ; write ( n ) end .";
    let out = run(src);
    assert_eq!(out.stdout, vec!["99"]);
}

#[test]
fn writeln_no_args_outputs_empty_line() {
    let src = "program p ( x ) ; begin writeln end .";
    let out = run(src);
    assert_eq!(out.stdout.len(), 1);
    assert_eq!(out.stdout[0], "");
}

#[test]
fn multiple_writes_produce_multiple_entries() {
    let src = "program p ( x ) ; begin write ( 1 ) ; write ( 2 ) ; write ( 3 ) end .";
    let out = run(src);
    assert_eq!(out.stdout, vec!["1", "2", "3"]);
}

// ── Arithmetic ─────────────────────────────────────────────────────────────────

#[test]
fn addition_result_is_correct() {
    let src = "program p ( x ) ; var n : integer ; begin n := 1 + 2 ; write ( n ) end .";
    assert_eq!(run(src).stdout, vec!["3"]);
}

#[test]
fn subtraction_result_is_correct() {
    let src = "program p ( x ) ; var n : integer ; begin n := 10 - 4 ; write ( n ) end .";
    assert_eq!(run(src).stdout, vec!["6"]);
}

#[test]
fn multiplication_result_is_correct() {
    let src = "program p ( x ) ; var n : integer ; begin n := 3 * 4 ; write ( n ) end .";
    assert_eq!(run(src).stdout, vec!["12"]);
}

#[test]
fn division_result_is_correct() {
    let src = "program p ( x ) ; var n : integer ; begin n := 8 div 2 ; write ( n ) end .";
    assert_eq!(run(src).stdout, vec!["4"]);
}

#[test]
fn modulo_result_is_correct() {
    let src = "program p ( x ) ; var n : integer ; begin n := 7 mod 3 ; write ( n ) end .";
    assert_eq!(run(src).stdout, vec!["1"]);
}

#[test]
fn unary_negation_result_is_correct() {
    let src = "program p ( x ) ; var n : integer ; begin n := - 5 ; write ( n ) end .";
    assert_eq!(run(src).stdout, vec!["-5"]);
}

#[test]
fn uninitialized_variable_reads_as_zero() {
    let src = "program p ( x ) ; var n : integer ; begin write ( n ) end .";
    assert_eq!(run(src).stdout, vec!["0"]);
}

// ── Control flow ───────────────────────────────────────────────────────────────

#[test]
fn if_true_executes_then_branch() {
    // n is uninitialized (0); condition 0 = 0 is true → write(1)
    let src = "program p ( x ) ; var n : integer ; begin if n = 0 then write ( 1 ) else write ( 0 ) end .";
    assert_eq!(run(src).stdout, vec!["1"]);
}

#[test]
fn if_false_executes_else_branch() {
    // n is 0; condition 0 > 5 is false → write(0)
    let src = "program p ( x ) ; var n : integer ; begin if n > 5 then write ( 1 ) else write ( 0 ) end .";
    assert_eq!(run(src).stdout, vec!["0"]);
}

#[test]
fn if_no_else_skipped_when_false() {
    let src = "program p ( x ) ; var n : integer ; begin if n > 0 then write ( 1 ) end .";
    assert!(run(src).stdout.is_empty());
}

#[test]
fn if_no_else_executes_when_true() {
    let src = "program p ( x ) ; var n : integer ; begin n := 5 ; if n > 0 then write ( 1 ) end .";
    assert_eq!(run(src).stdout, vec!["1"]);
}

#[test]
fn while_loop_runs_correct_number_of_times() {
    // write inside loop executes 3 times (i goes 0→1→2→3, exits at 3)
    let src = "program p ( x ) ; var i : integer ; begin while i < 3 do begin write ( i ) ; i := i + 1 end end .";
    assert_eq!(run(src).stdout, vec!["0", "1", "2"]);
}

#[test]
fn while_loop_body_skipped_when_condition_false_initially() {
    // i is 0, condition i > 10 is false immediately
    let src = "program p ( x ) ; var i : integer ; begin while i > 10 do write ( i ) end .";
    assert!(run(src).stdout.is_empty());
}

// ── Read input ─────────────────────────────────────────────────────────────────

#[test]
fn read_integer_from_input() {
    let src = "program p ( x ) ; var n : integer ; begin read ( n ) ; write ( n ) end .";
    assert_eq!(run_with(src, "42").stdout, vec!["42"]);
}

#[test]
fn read_multiple_values() {
    let src = "program p ( x ) ; var a , b : integer ; begin read ( a ) ; read ( b ) ; write ( a ) ; write ( b ) end .";
    assert_eq!(run_with(src, "10 20").stdout, vec!["10", "20"]);
}

// ── Function / procedure calls ─────────────────────────────────────────────────

#[test]
fn procedure_call_executes_body() {
    // sq(3) as a statement; function body computes sq := 3*3 (side effect only, no output)
    let src = "program p ( x ) ; function sq ( n : integer ) : integer ; begin sq := n * n end ; begin sq ( 3 ) end .";
    let out = run(src);
    assert!(out.errors.is_empty());
    assert!(out.halted);
}

#[test]
fn function_body_executes_write() {
    // Function body is reachable: writes a literal (parameter binding needs stack frames).
    let src = "program p ( x ) ; function show ( n : integer ) : integer ; begin write ( 7 ) ; show := 0 end ; begin show ( 7 ) end .";
    let out = run(src);
    assert!(out.errors.is_empty());
    assert_eq!(out.stdout, vec!["7"]);
}

#[test]
fn factorial_recursive_correct_output() {
    let src = "program p ( x ) ; var n : integer ; function fact ( k : integer ) : integer ; begin if k = 0 then fact := 1 else fact := k * fact ( k - 1 ) end ; begin read ( n ) ; write ( fact ( n ) ) end .";
    let out = run_with(src, "5");
    assert!(out.errors.is_empty());
    assert_eq!(out.stdout, vec!["120"]);
}

#[test]
fn factorial_base_case() {
    let src = "program p ( x ) ; var n : integer ; function fact ( k : integer ) : integer ; begin if k = 0 then fact := 1 else fact := k * fact ( k - 1 ) end ; begin read ( n ) ; write ( fact ( n ) ) end .";
    let out = run_with(src, "0");
    assert!(out.errors.is_empty());
    assert_eq!(out.stdout, vec!["1"]);
}

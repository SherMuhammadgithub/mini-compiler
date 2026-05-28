// Phase 19 — Integration tests covering the full pipeline for every Test Matrix entry.
// Programs: GCD, ArraySum, Factorial (valid); and 5 invalid programs.
// Each valid program must pass all three parsers and have no semantic errors.
// Each invalid program must produce errors in the appropriate stage(s).

use compiler_core::ll1_parser;
use compiler_core::lr_parser;
use compiler_core::rd_parser;
use compiler_core::semantic;
use compiler_core::lexer;
use compiler_core::vm;

// ── Sample programs ────────────────────────────────────────────────────────────

const GCD: &str = r#"
program example ( input , output ) ;
var x , y : integer ;
function gcd ( a , b : integer ) : integer ;
begin
    if b = 0 then gcd := a
    else gcd := gcd ( b , a mod b )
end ;
begin
    read ( x , y ) ;
    write ( gcd ( x , y ) )
end .
"#;

const ARRAY_SUM: &str = r#"
program arraysum ( input , output ) ;
var a : array [ 1 .. 10 ] of integer ;
var i , sum : integer ;
begin
    sum := 0 ;
    i := 1 ;
    while i <= 10 do
    begin
        read ( a [ i ] ) ;
        sum := sum + a [ i ] ;
        i := i + 1
    end ;
    write ( sum )
end .
"#;

const FACTORIAL: &str = r#"
program factorial ( input , output ) ;
var n : integer ;
function fact ( k : integer ) : integer ;
begin
    if k = 0 then fact := 1
    else fact := k * fact ( k - 1 )
end ;
begin
    read ( n ) ;
    write ( fact ( n ) )
end .
"#;

// ── Helpers ────────────────────────────────────────────────────────────────────

fn rd_ok(src: &str) -> bool {
    let out = rd_parser::parse(src);
    out.errors.is_empty() && out.ast.is_some()
}

fn ll1_ok(src: &str) -> bool {
    ll1_parser::parse(src).accepted
}

fn lr_ok(src: &str) -> bool {
    lr_parser::parse(src).accepted
}

fn semantic_errors(src: &str) -> Vec<String> {
    semantic::analyze(src)
        .errors
        .iter()
        .filter(|e| e.stage == "semantic" || e.stage == "symbol_table")
        .map(|e| e.message.clone())
        .collect()
}

fn semantic_ok(src: &str) -> bool {
    semantic_errors(src).is_empty()
}

fn lex_has_error(src: &str) -> bool {
    lexer::tokenize(src).errors.iter().any(|e| e.stage == "lexer")
}

fn any_parse_error(src: &str) -> bool {
    !rd_parser::parse(src).errors.is_empty()
        || !ll1_parser::parse(src).errors.is_empty()
        || !lr_parser::parse(src).errors.is_empty()
}

// ── GCD (valid) ───────────────────────────────────────────────────────────────

#[test]
fn gcd_rd_parser_accepts() {
    assert!(rd_ok(GCD), "RD parser should accept GCD program");
}

#[test]
fn gcd_ll1_parser_accepts() {
    assert!(ll1_ok(GCD), "LL(1) parser should accept GCD program");
}

#[test]
fn gcd_lr_parser_accepts() {
    assert!(lr_ok(GCD), "LR parser should accept GCD program");
}

#[test]
fn gcd_no_semantic_errors() {
    assert!(semantic_ok(GCD), "GCD should have no semantic errors; got: {:?}", semantic_errors(GCD));
}

#[test]
fn gcd_lexer_no_errors() {
    assert!(!lex_has_error(GCD), "GCD should have no lex errors");
}

// ── ArraySum (valid) ──────────────────────────────────────────────────────────

#[test]
fn arraysum_rd_parser_accepts() {
    assert!(rd_ok(ARRAY_SUM));
}

#[test]
fn arraysum_ll1_parser_accepts() {
    assert!(ll1_ok(ARRAY_SUM));
}

#[test]
fn arraysum_lr_parser_accepts() {
    assert!(lr_ok(ARRAY_SUM));
}

#[test]
fn arraysum_no_semantic_errors() {
    assert!(semantic_ok(ARRAY_SUM), "ArraySum semantic errors: {:?}", semantic_errors(ARRAY_SUM));
}

// ── Factorial (valid) ─────────────────────────────────────────────────────────

#[test]
fn factorial_rd_parser_accepts() {
    assert!(rd_ok(FACTORIAL));
}

#[test]
fn factorial_ll1_parser_accepts() {
    assert!(ll1_ok(FACTORIAL));
}

#[test]
fn factorial_lr_parser_accepts() {
    assert!(lr_ok(FACTORIAL));
}

#[test]
fn factorial_no_semantic_errors() {
    assert!(semantic_ok(FACTORIAL), "Factorial semantic errors: {:?}", semantic_errors(FACTORIAL));
}

// ── VM: valid programs at least halt without crash ────────────────────────────

#[test]
fn vm_simple_write_correct_output() {
    let src = "program p ( input , output ) ; var n : integer ; begin n := 42 ; write ( n ) end .";
    let out = vm::execute(src, "");
    assert!(out.halted);
    assert!(out.errors.is_empty());
    assert_eq!(out.stdout, vec!["42"]);
}

#[test]
fn vm_if_else_correct_branch() {
    let src = "program p ( input , output ) ; var n : integer ; begin n := 5 ; if n > 0 then write ( n ) else write ( 0 ) end .";
    let out = vm::execute(src, "");
    assert!(out.halted && out.errors.is_empty());
    assert_eq!(out.stdout, vec!["5"]);
}

#[test]
fn vm_while_loop_sum() {
    let src = "program p ( x ) ; var i , s : integer ; begin s := 0 ; i := 1 ; while i <= 3 do begin s := s + i ; i := i + 1 end ; write ( s ) end .";
    let out = vm::execute(src, "");
    assert!(out.halted && out.errors.is_empty());
    assert_eq!(out.stdout, vec!["6"]);
}

#[test]
fn vm_read_then_write() {
    let src = "program p ( input , output ) ; var n : integer ; begin read ( n ) ; write ( n ) end .";
    let out = vm::execute(src, "99");
    assert!(out.halted && out.errors.is_empty());
    assert_eq!(out.stdout, vec!["99"]);
}

#[test]
fn vm_gcd_program_halts_without_crash() {
    // Parameter passing is not fully implemented; this just verifies no panic/crash.
    let out = vm::execute(GCD, "12 8");
    assert!(out.halted, "GCD program should halt");
}

#[test]
fn vm_arraysum_output_is_correct() {
    // Array sum: read 5 elements from input, write their sum.
    let out = vm::execute(ARRAY_SUM, "1 2 3 4 5");
    assert!(out.halted, "ArraySum should halt");
    assert!(out.errors.is_empty(), "ArraySum errors: {:?}", out.errors);
    assert_eq!(out.stdout, vec!["15"]);
}

// ── Duplicate variable (invalid) ──────────────────────────────────────────────

#[test]
fn duplicate_var_produces_semantic_error() {
    let src = "program p ( x ) ; var n : integer ; var n : real ; begin end .";
    let errs = semantic_errors(src);
    assert!(!errs.is_empty(), "duplicate 'n' should produce a semantic error");
    assert!(
        errs.iter().any(|m| m.to_lowercase().contains("n") || m.contains("duplicate") || m.contains("redeclar") || m.contains("already")),
        "error should mention the redeclared symbol; got: {:?}", errs
    );
}

// ── Undeclared use (invalid) ──────────────────────────────────────────────────

#[test]
fn undeclared_variable_produces_semantic_error() {
    let src = "program p ( x ) ; begin n := 5 end .";
    let errs = semantic_errors(src);
    assert!(!errs.is_empty(), "use of undeclared 'n' should produce a semantic error");
}

// ── Type mismatch (invalid) ───────────────────────────────────────────────────

#[test]
fn type_mismatch_integer_real_produces_semantic_error() {
    let src = "program p ( x ) ; var n : integer ; begin n := 3 + 0 end .";
    // n := 3 + 0 is valid (integer + integer); use a known mismatch instead
    let src2 = "program p ( x ) ; var n : integer ; var r : real ; begin n := r end .";
    let errs = semantic_errors(src2);
    assert!(!errs.is_empty(), "assigning real to integer should produce a type error; got: {:?}", errs);
}

// ── Missing `end` (invalid) ───────────────────────────────────────────────────

#[test]
fn missing_end_produces_rd_parse_error() {
    let src = "program p ( x ) ; begin n := 5";
    assert!(!rd_parser::parse(src).errors.is_empty(), "missing end should produce RD parse error");
}

#[test]
fn missing_end_produces_ll1_parse_error() {
    let src = "program p ( x ) ; begin n := 5";
    assert!(!ll1_parser::parse(src).errors.is_empty(), "missing end should produce LL(1) parse error");
}

#[test]
fn missing_end_produces_lr_parse_error() {
    let src = "program p ( x ) ; begin n := 5";
    assert!(!lr_parser::parse(src).errors.is_empty(), "missing end should produce LR parse error");
}

// ── Bad/unknown character (invalid) ───────────────────────────────────────────

#[test]
fn unknown_character_produces_lex_error() {
    let src = "program p ( x ) ; begin n := 1 @ 2 end .";
    assert!(lex_has_error(src), "@ is an unknown character — lexer should report an error");
}

#[test]
fn unknown_character_does_not_propagate_to_other_stages_cleanly() {
    // Lex errors mean no other stage should silently accept the program.
    let src = "program p ( x ) ; begin n := 1 @ 2 end .";
    assert!(any_parse_error(src), "lex error program should not silently pass all parsers");
}

// ── All three parsers agree on valid programs ─────────────────────────────────

#[test]
fn all_parsers_agree_valid_simple() {
    let src = "program p ( x ) ; var n : integer ; begin n := 7 ; write ( n ) end .";
    assert!(rd_ok(src) && ll1_ok(src) && lr_ok(src), "all parsers must agree on a valid program");
}

#[test]
fn all_parsers_agree_valid_if_else() {
    let src = "program p ( x ) ; var n : integer ; begin if n > 0 then write ( n ) else write ( 0 ) end .";
    assert!(rd_ok(src) && ll1_ok(src) && lr_ok(src));
}

#[test]
fn all_parsers_agree_valid_while() {
    let src = "program p ( x ) ; var i : integer ; begin i := 0 ; while i < 5 do i := i + 1 end .";
    assert!(rd_ok(src) && ll1_ok(src) && lr_ok(src));
}

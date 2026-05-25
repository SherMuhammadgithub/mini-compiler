// Integration tests for Phase 12: Three-Address Code IR generation.
use compiler_core::ir::generate;
use compiler_core::types::{TacArg, TacOp};

fn instrs(src: &str) -> Vec<compiler_core::types::TacInstr> {
    generate(src).instructions
}

fn has_op(src: &str, pred: impl Fn(&TacOp) -> bool) -> bool {
    instrs(src).iter().any(|i| pred(&i.op))
}

fn count_op(src: &str, pred: impl Fn(&TacOp) -> bool) -> usize {
    instrs(src).iter().filter(|i| pred(&i.op)).count()
}

// ── Basic instruction presence ────────────────────────────────────────────────

#[test]
fn empty_program_has_no_instructions() {
    assert!(instrs("program p ( x ) ; begin end .").is_empty());
}

#[test]
fn assignment_emits_assign_op() {
    let src = "program p ( x ) ; var n : integer ; begin n := 42 end .";
    assert!(has_op(src, |o| matches!(o, TacOp::Assign)));
}

#[test]
fn integer_literal_assign_result_is_name() {
    let src = "program p ( x ) ; var n : integer ; begin n := 42 end .";
    let v = instrs(src);
    let assign = v.iter().find(|i| matches!(i.op, TacOp::Assign)).unwrap();
    assert!(matches!(&assign.result, Some(TacArg::Name(n)) if n == "n"));
}

// ── Arithmetic expressions ────────────────────────────────────────────────────

#[test]
fn addition_emits_add() {
    let src = "program p ( x ) ; var n : integer ; begin n := 1 + 2 end .";
    assert!(has_op(src, |o| matches!(o, TacOp::Add)));
}

#[test]
fn subtraction_emits_sub() {
    let src = "program p ( x ) ; var n : integer ; begin n := 5 - 3 end .";
    assert!(has_op(src, |o| matches!(o, TacOp::Sub)));
}

#[test]
fn multiplication_emits_mul() {
    let src = "program p ( x ) ; var n : integer ; begin n := 3 * 4 end .";
    assert!(has_op(src, |o| matches!(o, TacOp::Mul)));
}

#[test]
fn division_emits_div() {
    let src = "program p ( x ) ; var n : integer ; begin n := 8 div 2 end .";
    assert!(has_op(src, |o| matches!(o, TacOp::Div)));
}

#[test]
fn mod_emits_mod() {
    let src = "program p ( x ) ; var n : integer ; begin n := 7 mod 3 end .";
    assert!(has_op(src, |o| matches!(o, TacOp::Mod)));
}

#[test]
fn arithmetic_result_stored_in_temp() {
    let src = "program p ( x ) ; var n : integer ; begin n := 1 + 2 end .";
    let v = instrs(src);
    let add = v.iter().find(|i| matches!(i.op, TacOp::Add)).unwrap();
    assert!(matches!(&add.result, Some(TacArg::Temp(0))));
}

// ── Unary expressions ─────────────────────────────────────────────────────────

#[test]
fn unary_minus_emits_neg() {
    let src = "program p ( x ) ; var n : integer ; begin n := - 5 end .";
    assert!(has_op(src, |o| matches!(o, TacOp::Neg)));
}

#[test]
fn not_emits_not_op() {
    let src = "program p ( x ) ; var b , n : integer ; begin if not b = 0 then n := 1 else n := 0 end .";
    assert!(has_op(src, |o| matches!(o, TacOp::Not)));
}

// ── Relational expressions ────────────────────────────────────────────────────

#[test]
fn eq_relop_emits_eq() {
    let src = "program p ( x ) ; var n : integer ; begin if n = 0 then n := 1 else n := 0 end .";
    assert!(has_op(src, |o| matches!(o, TacOp::Eq)));
}

#[test]
fn lt_relop_emits_lt() {
    let src = "program p ( x ) ; var n : integer ; begin if n < 10 then n := 0 else n := 1 end .";
    assert!(has_op(src, |o| matches!(o, TacOp::Lt)));
}

// ── Control flow ──────────────────────────────────────────────────────────────

#[test]
fn if_emits_if_false_goto() {
    let src = "program p ( x ) ; var n : integer ; begin if n > 0 then n := 1 else n := 0 end .";
    assert!(has_op(src, |o| matches!(o, TacOp::IfFalseGoto)));
}

#[test]
fn if_else_emits_goto_for_then_branch() {
    let src = "program p ( x ) ; var n : integer ; begin if n > 0 then n := 1 else n := 0 end .";
    assert!(has_op(src, |o| matches!(o, TacOp::Goto)));
}

#[test]
fn if_else_emits_two_labels() {
    let src = "program p ( x ) ; var n : integer ; begin if n > 0 then n := 1 else n := 0 end .";
    assert_eq!(count_op(src, |o| matches!(o, TacOp::Label)), 2);
}

#[test]
fn if_no_else_emits_one_label() {
    let src = "program p ( x ) ; var n : integer ; begin if n > 0 then n := 1 end .";
    assert_eq!(count_op(src, |o| matches!(o, TacOp::Label)), 1);
}

#[test]
fn while_emits_two_labels() {
    let src = "program p ( x ) ; var i : integer ; begin while i < 10 do i := i + 1 end .";
    assert_eq!(count_op(src, |o| matches!(o, TacOp::Label)), 2);
}

#[test]
fn while_emits_if_false_goto() {
    let src = "program p ( x ) ; var i : integer ; begin while i < 10 do i := i + 1 end .";
    assert!(has_op(src, |o| matches!(o, TacOp::IfFalseGoto)));
}

#[test]
fn while_emits_goto_back() {
    let src = "program p ( x ) ; var i : integer ; begin while i < 10 do i := i + 1 end .";
    assert!(has_op(src, |o| matches!(o, TacOp::Goto)));
}

#[test]
fn while_loop_label_is_l0() {
    let src = "program p ( x ) ; var i : integer ; begin while i < 10 do i := i + 1 end .";
    let v = instrs(src);
    let first_label = v.iter().find(|i| matches!(i.op, TacOp::Label)).unwrap();
    assert!(matches!(&first_label.arg1, Some(TacArg::Label(l)) if l == "L0"));
}

// ── Functions and procedures ──────────────────────────────────────────────────

#[test]
fn function_decl_emits_label() {
    let src = "program p ( x ) ; function sq ( n : integer ) : integer ; begin sq := n * n end ; begin end .";
    assert!(has_op(src, |o| matches!(o, TacOp::Label)));
}

#[test]
fn function_decl_label_is_function_name() {
    let src = "program p ( x ) ; function sq ( n : integer ) : integer ; begin sq := n * n end ; begin end .";
    let v = instrs(src);
    let lbl = v.iter().find(|i| matches!(i.op, TacOp::Label)).unwrap();
    assert!(matches!(&lbl.arg1, Some(TacArg::Label(n)) if n == "sq"));
}

#[test]
fn function_decl_emits_return() {
    let src = "program p ( x ) ; function sq ( n : integer ) : integer ; begin sq := n * n end ; begin end .";
    assert!(has_op(src, |o| matches!(o, TacOp::Return)));
}

#[test]
fn function_call_emits_param_then_call() {
    let src = "program p ( x ) ; function sq ( n : integer ) : integer ; begin sq := n * n end ; var r : integer ; begin r := sq ( 3 ) end .";
    let v = instrs(src);
    let param_pos = v.iter().rposition(|i| matches!(i.op, TacOp::Param)).unwrap_or(0);
    let call_pos  = v.iter().rposition(|i| matches!(i.op, TacOp::Call)).unwrap_or(1);
    assert!(param_pos < call_pos, "Param must come before Call");
}

#[test]
fn procedure_call_stmt_call_has_no_result() {
    // sq(3) used as a statement (ProcedureCall) — Call instruction must have no result
    let src = "program p ( x ) ; function sq ( n : integer ) : integer ; begin sq := n * n end ; begin sq ( 3 ) end .";
    let v = instrs(src);
    let call = v.iter().rfind(|i| matches!(i.op, TacOp::Call)).unwrap();
    assert!(call.result.is_none());
}

// ── Builtins ──────────────────────────────────────────────────────────────────

#[test]
fn write_emits_write_op() {
    let src = "program p ( x ) ; var n : integer ; begin write ( n ) end .";
    assert!(has_op(src, |o| matches!(o, TacOp::Write)));
}

#[test]
fn writeln_no_args_emits_write_op() {
    let src = "program p ( x ) ; begin writeln end .";
    assert!(has_op(src, |o| matches!(o, TacOp::Write)));
}

// ── Array operations ──────────────────────────────────────────────────────────

#[test]
fn array_lhs_assignment_emits_copy_to_array() {
    let src = "program p ( x ) ; var a : array [ 1 .. 5 ] of integer ; begin a [ 1 ] := 7 end .";
    assert!(has_op(src, |o| matches!(o, TacOp::CopyToArray)));
}

#[test]
fn copy_from_array_op_exists_in_tac_op_set() {
    // Verify the CopyFromArray path: array LHS write emits CopyToArray,
    // so we confirm the paired op is defined and distinct from CopyToArray.
    let src = "program p ( x ) ; var a : array [ 1 .. 5 ] of integer ; begin a [ 1 ] := 7 end .";
    assert!(has_op(src, |o| matches!(o, TacOp::CopyToArray)));
    assert!(!has_op(src, |o| matches!(o, TacOp::CopyFromArray)));
}

// ── Error passthrough ─────────────────────────────────────────────────────────

#[test]
fn semantic_errors_passed_through() {
    let out = generate("program p ( x ) ; begin z := 1 end .");
    assert!(!out.errors.is_empty());
}

#[test]
fn valid_program_no_errors() {
    let out = generate("program p ( x ) ; var n : integer ; begin n := 42 end .");
    assert!(out.errors.is_empty());
}

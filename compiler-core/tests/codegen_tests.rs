// Integration tests for Phase 13: Stack VM bytecode generation.
use compiler_core::codegen::generate;
use compiler_core::types::VmInstr;

fn bc(src: &str) -> Vec<VmInstr> {
    generate(src).bytecode
}

fn has_instr(src: &str, pred: impl Fn(&VmInstr) -> bool) -> bool {
    bc(src).iter().any(|i| pred(i))
}

fn count_instr(src: &str, pred: impl Fn(&VmInstr) -> bool) -> usize {
    bc(src).iter().filter(|i| pred(i)).count()
}

// ── Always ends with Halt ──────────────────────────────────────────────────────

#[test]
fn empty_program_ends_with_halt() {
    let v = bc("program p ( x ) ; begin end .");
    assert!(matches!(v.last(), Some(VmInstr::Halt)));
}

#[test]
fn non_empty_program_ends_with_halt() {
    let v = bc("program p ( x ) ; var n : integer ; begin n := 42 end .");
    assert!(matches!(v.last(), Some(VmInstr::Halt)));
}

// ── Assignment ─────────────────────────────────────────────────────────────────

#[test]
fn integer_assign_emits_push_then_store() {
    let src = "program p ( x ) ; var n : integer ; begin n := 42 end .";
    assert!(has_instr(src, |i| matches!(i, VmInstr::Push(_))));
    assert!(has_instr(src, |i| matches!(i, VmInstr::Store(_))));
}

#[test]
fn assign_store_uses_variable_name() {
    let src = "program p ( x ) ; var n : integer ; begin n := 42 end .";
    assert!(has_instr(src, |i| matches!(i, VmInstr::Store(s) if s == "n")));
}

#[test]
fn assign_push_contains_literal_value() {
    let src = "program p ( x ) ; var n : integer ; begin n := 99 end .";
    use compiler_core::types::VmValue;
    assert!(has_instr(src, |i| matches!(i, VmInstr::Push(VmValue::Int(99)))));
}

// ── Arithmetic ─────────────────────────────────────────────────────────────────

#[test]
fn addition_emits_add_instr() {
    let src = "program p ( x ) ; var n : integer ; begin n := 1 + 2 end .";
    assert!(has_instr(src, |i| matches!(i, VmInstr::Add)));
}

#[test]
fn subtraction_emits_sub_instr() {
    let src = "program p ( x ) ; var n : integer ; begin n := 5 - 3 end .";
    assert!(has_instr(src, |i| matches!(i, VmInstr::Sub)));
}

#[test]
fn multiplication_emits_mul_instr() {
    let src = "program p ( x ) ; var n : integer ; begin n := 3 * 4 end .";
    assert!(has_instr(src, |i| matches!(i, VmInstr::Mul)));
}

#[test]
fn division_emits_div_instr() {
    let src = "program p ( x ) ; var n : integer ; begin n := 8 div 2 end .";
    assert!(has_instr(src, |i| matches!(i, VmInstr::Div)));
}

#[test]
fn mod_emits_mod_instr() {
    let src = "program p ( x ) ; var n : integer ; begin n := 7 mod 3 end .";
    assert!(has_instr(src, |i| matches!(i, VmInstr::Mod)));
}

#[test]
fn addition_pushes_two_args_before_add() {
    let src = "program p ( x ) ; var n : integer ; begin n := 1 + 2 end .";
    let v = bc(src);
    let add_pos = v.iter().position(|i| matches!(i, VmInstr::Add)).unwrap();
    // both pushes must appear before Add
    let push_count = v[..add_pos].iter().filter(|i| matches!(i, VmInstr::Push(_))).count();
    assert_eq!(push_count, 2);
}

// ── Unary ──────────────────────────────────────────────────────────────────────

#[test]
fn unary_minus_emits_neg_instr() {
    let src = "program p ( x ) ; var n : integer ; begin n := - 5 end .";
    assert!(has_instr(src, |i| matches!(i, VmInstr::Neg)));
}

#[test]
fn not_emits_not_instr() {
    let src = "program p ( x ) ; var b , n : integer ; begin if not b = 0 then n := 1 else n := 0 end .";
    assert!(has_instr(src, |i| matches!(i, VmInstr::Not)));
}

// ── Comparisons ────────────────────────────────────────────────────────────────

#[test]
fn eq_relop_emits_cmpeq() {
    let src = "program p ( x ) ; var n : integer ; begin if n = 0 then n := 1 else n := 0 end .";
    assert!(has_instr(src, |i| matches!(i, VmInstr::CmpEq)));
}

#[test]
fn lt_relop_emits_cmplt() {
    let src = "program p ( x ) ; var n : integer ; begin if n < 10 then n := 0 else n := 1 end .";
    assert!(has_instr(src, |i| matches!(i, VmInstr::CmpLt)));
}

#[test]
fn gt_relop_emits_cmpgt() {
    let src = "program p ( x ) ; var n : integer ; begin if n > 0 then n := 1 else n := 0 end .";
    assert!(has_instr(src, |i| matches!(i, VmInstr::CmpGt)));
}

// ── Control flow ───────────────────────────────────────────────────────────────

#[test]
fn if_else_emits_jmp_false() {
    let src = "program p ( x ) ; var n : integer ; begin if n > 0 then n := 1 else n := 0 end .";
    assert!(has_instr(src, |i| matches!(i, VmInstr::JmpFalse(_))));
}

#[test]
fn if_else_emits_unconditional_jmp() {
    let src = "program p ( x ) ; var n : integer ; begin if n > 0 then n := 1 else n := 0 end .";
    assert!(has_instr(src, |i| matches!(i, VmInstr::Jmp(_))));
}

#[test]
fn while_emits_jmp_false_and_jmp() {
    let src = "program p ( x ) ; var i : integer ; begin while i < 10 do i := i + 1 end .";
    assert!(has_instr(src, |i| matches!(i, VmInstr::JmpFalse(_))));
    assert!(has_instr(src, |i| matches!(i, VmInstr::Jmp(_))));
}

#[test]
fn while_jmp_back_target_is_less_than_jmp_false_target() {
    // The Jmp(loop_start) target should be smaller than JmpFalse(loop_end) target
    let src = "program p ( x ) ; var i : integer ; begin while i < 10 do i := i + 1 end .";
    let v = bc(src);
    let jmp_false_target = v.iter().find_map(|i| if let VmInstr::JmpFalse(t) = i { Some(*t) } else { None }).unwrap();
    let jmp_target = v.iter().find_map(|i| if let VmInstr::Jmp(t) = i { Some(*t) } else { None }).unwrap();
    assert!(jmp_target < jmp_false_target, "back-edge Jmp target must precede the exit target");
}

#[test]
fn jump_targets_are_valid_indices() {
    let src = "program p ( x ) ; var n : integer ; begin if n > 0 then n := 1 else n := 0 end .";
    let v = bc(src);
    for instr in &v {
        match instr {
            VmInstr::Jmp(t) | VmInstr::JmpFalse(t) => assert!(*t < v.len()),
            _ => {}
        }
    }
}

// ── Functions ──────────────────────────────────────────────────────────────────

#[test]
fn function_emits_ret() {
    let src = "program p ( x ) ; function sq ( n : integer ) : integer ; begin sq := n * n end ; begin end .";
    assert!(has_instr(src, |i| matches!(i, VmInstr::Ret)));
}

#[test]
fn function_call_emits_call_instr() {
    let src = "program p ( x ) ; function sq ( n : integer ) : integer ; begin sq := n * n end ; begin sq ( 3 ) end .";
    assert!(has_instr(src, |i| matches!(i, VmInstr::Call(_))));
}

#[test]
fn function_call_name_matches() {
    let src = "program p ( x ) ; function sq ( n : integer ) : integer ; begin sq := n * n end ; begin sq ( 3 ) end .";
    assert!(has_instr(src, |i| matches!(i, VmInstr::Call(n) if n == "sq")));
}

#[test]
fn function_call_param_push_before_call() {
    let src = "program p ( x ) ; function sq ( n : integer ) : integer ; begin sq := n * n end ; begin sq ( 3 ) end .";
    let v = bc(src);
    let call_pos = v.iter().rposition(|i| matches!(i, VmInstr::Call(_))).unwrap();
    // At least one Push before the Call
    let has_push_before = v[..call_pos].iter().any(|i| matches!(i, VmInstr::Push(_)));
    assert!(has_push_before);
}

// ── Write builtin ──────────────────────────────────────────────────────────────

#[test]
fn write_emits_write_instr() {
    let src = "program p ( x ) ; var n : integer ; begin write ( n ) end .";
    assert!(has_instr(src, |i| matches!(i, VmInstr::Write)));
}

#[test]
fn writeln_no_args_emits_write() {
    let src = "program p ( x ) ; begin writeln end .";
    assert!(has_instr(src, |i| matches!(i, VmInstr::Write)));
}

#[test]
fn write_with_arg_pushes_before_write() {
    let src = "program p ( x ) ; var n : integer ; begin write ( n ) end .";
    let v = bc(src);
    let write_pos = v.iter().position(|i| matches!(i, VmInstr::Write)).unwrap();
    let load_before = v[..write_pos].iter().any(|i| matches!(i, VmInstr::Load(_)));
    assert!(load_before);
}

// ── Array ──────────────────────────────────────────────────────────────────────

#[test]
fn array_assignment_emits_store_idx() {
    let src = "program p ( x ) ; var a : array [ 1 .. 5 ] of integer ; begin a [ 1 ] := 7 end .";
    assert!(has_instr(src, |i| matches!(i, VmInstr::StoreIdx)));
}

// ── Error passthrough ──────────────────────────────────────────────────────────

#[test]
fn semantic_errors_propagate_to_codegen() {
    let out = generate("program p ( x ) ; begin z := 1 end .");
    assert!(!out.errors.is_empty());
}

#[test]
fn valid_program_no_errors() {
    let out = generate("program p ( x ) ; var n : integer ; begin n := 42 end .");
    assert!(out.errors.is_empty());
}

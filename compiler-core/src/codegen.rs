// Stack VM bytecode generator. Translates TAC quadruples to VmInstr sequences.
use crate::types::{TacArg, TacOp, VmInstr, VmValue};
use serde::Serialize;
use std::collections::HashMap;

#[derive(Serialize)]
pub struct CodegenOutput {
    pub bytecode: Vec<VmInstr>,
    pub function_map: HashMap<String, usize>, // label name → bytecode index
    pub errors: Vec<crate::types::CompilerError>,
}

pub fn generate(source: &str) -> CodegenOutput {
    let ir_out = crate::ir::generate(source);
    let mut bc: Vec<VmInstr> = vec![];
    let instrs = &ir_out.instructions;

    // Pre-scan: collect names of function/procedure labels (not control-flow L0/L1 labels).
    // A function label is any label whose name is not purely "L" followed by digits.
    let func_labels: std::collections::HashSet<String> = instrs.iter()
        .filter_map(|i| if let (crate::types::TacOp::Label, Some(crate::types::TacArg::Label(n))) = (&i.op, &i.arg1) {
            let is_ln = n.starts_with('L') && n.len() > 1 && n[1..].chars().all(|c| c.is_ascii_digit());
            if !is_ln { Some(n.clone()) } else { None }
        } else { None })
        .collect();

    let mut main_halt_emitted = false; // Halt inserted between main body and first function

    // Single pass: emit bytecode with a patch-list for forward jumps.
    let mut label_map: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let mut patches: Vec<(usize, String)> = vec![];   // (bc index, label name)

    for instr in instrs {
        match &instr.op {
            TacOp::Add => {
                push_arg(&mut bc, instr.arg1.as_ref().unwrap());
                push_arg(&mut bc, instr.arg2.as_ref().unwrap());
                bc.push(VmInstr::Add);
                store_result(&mut bc, instr.result.as_ref().unwrap());
            }
            TacOp::Sub => {
                push_arg(&mut bc, instr.arg1.as_ref().unwrap());
                push_arg(&mut bc, instr.arg2.as_ref().unwrap());
                bc.push(VmInstr::Sub);
                store_result(&mut bc, instr.result.as_ref().unwrap());
            }
            TacOp::Mul => {
                push_arg(&mut bc, instr.arg1.as_ref().unwrap());
                push_arg(&mut bc, instr.arg2.as_ref().unwrap());
                bc.push(VmInstr::Mul);
                store_result(&mut bc, instr.result.as_ref().unwrap());
            }
            TacOp::Div => {
                push_arg(&mut bc, instr.arg1.as_ref().unwrap());
                push_arg(&mut bc, instr.arg2.as_ref().unwrap());
                bc.push(VmInstr::Div);
                store_result(&mut bc, instr.result.as_ref().unwrap());
            }
            TacOp::Mod => {
                push_arg(&mut bc, instr.arg1.as_ref().unwrap());
                push_arg(&mut bc, instr.arg2.as_ref().unwrap());
                bc.push(VmInstr::Mod);
                store_result(&mut bc, instr.result.as_ref().unwrap());
            }
            TacOp::Neg => {
                push_arg(&mut bc, instr.arg1.as_ref().unwrap());
                bc.push(VmInstr::Neg);
                store_result(&mut bc, instr.result.as_ref().unwrap());
            }
            TacOp::Not => {
                push_arg(&mut bc, instr.arg1.as_ref().unwrap());
                bc.push(VmInstr::Not);
                store_result(&mut bc, instr.result.as_ref().unwrap());
            }
            TacOp::Eq => {
                push_arg(&mut bc, instr.arg1.as_ref().unwrap());
                push_arg(&mut bc, instr.arg2.as_ref().unwrap());
                bc.push(VmInstr::CmpEq);
                store_result(&mut bc, instr.result.as_ref().unwrap());
            }
            TacOp::Ne => {
                push_arg(&mut bc, instr.arg1.as_ref().unwrap());
                push_arg(&mut bc, instr.arg2.as_ref().unwrap());
                bc.push(VmInstr::CmpNe);
                store_result(&mut bc, instr.result.as_ref().unwrap());
            }
            TacOp::Lt => {
                push_arg(&mut bc, instr.arg1.as_ref().unwrap());
                push_arg(&mut bc, instr.arg2.as_ref().unwrap());
                bc.push(VmInstr::CmpLt);
                store_result(&mut bc, instr.result.as_ref().unwrap());
            }
            TacOp::Le => {
                push_arg(&mut bc, instr.arg1.as_ref().unwrap());
                push_arg(&mut bc, instr.arg2.as_ref().unwrap());
                bc.push(VmInstr::CmpLe);
                store_result(&mut bc, instr.result.as_ref().unwrap());
            }
            TacOp::Gt => {
                push_arg(&mut bc, instr.arg1.as_ref().unwrap());
                push_arg(&mut bc, instr.arg2.as_ref().unwrap());
                bc.push(VmInstr::CmpGt);
                store_result(&mut bc, instr.result.as_ref().unwrap());
            }
            TacOp::Ge => {
                push_arg(&mut bc, instr.arg1.as_ref().unwrap());
                push_arg(&mut bc, instr.arg2.as_ref().unwrap());
                bc.push(VmInstr::CmpGe);
                store_result(&mut bc, instr.result.as_ref().unwrap());
            }
            TacOp::Assign => {
                push_arg(&mut bc, instr.arg1.as_ref().unwrap());
                store_result(&mut bc, instr.result.as_ref().unwrap());
            }
            TacOp::CopyToArray => {
                // arg1 = value, arg2 = index, result = array name
                push_arg(&mut bc, instr.arg1.as_ref().unwrap());
                push_arg(&mut bc, instr.arg2.as_ref().unwrap());
                if let Some(TacArg::Name(n)) = &instr.result {
                    bc.push(VmInstr::Store(n.clone()));
                }
                bc.push(VmInstr::StoreIdx);
            }
            TacOp::CopyFromArray => {
                // arg1 = array name, arg2 = index, result = temp
                push_arg(&mut bc, instr.arg1.as_ref().unwrap());
                push_arg(&mut bc, instr.arg2.as_ref().unwrap());
                bc.push(VmInstr::LoadIdx);
                store_result(&mut bc, instr.result.as_ref().unwrap());
            }
            TacOp::Label => {
                if let Some(TacArg::Label(name)) = &instr.arg1 {
                    // Insert Halt to terminate the main body before the first function label.
                    if func_labels.contains(name) && !main_halt_emitted {
                        bc.push(VmInstr::Halt);
                        main_halt_emitted = true;
                    }
                    label_map.insert(name.clone(), bc.len());
                }
            }
            TacOp::Goto => {
                if let Some(TacArg::Label(name)) = &instr.arg1 {
                    patches.push((bc.len(), name.clone()));
                    bc.push(VmInstr::Jmp(0));
                }
            }
            TacOp::IfFalseGoto => {
                push_arg(&mut bc, instr.arg1.as_ref().unwrap());
                if let Some(TacArg::Label(name)) = &instr.arg2 {
                    patches.push((bc.len(), name.clone()));
                    bc.push(VmInstr::JmpFalse(0));
                }
            }
            TacOp::Param => {
                push_arg(&mut bc, instr.arg1.as_ref().unwrap());
            }
            TacOp::Call => {
                if let Some(TacArg::Name(name)) = &instr.arg1 {
                    bc.push(VmInstr::Call(name.clone()));
                }
                if let Some(result) = &instr.result {
                    store_result(&mut bc, result);
                }
            }
            TacOp::Return => {
                bc.push(VmInstr::Ret);
            }
            TacOp::Read => {
                bc.push(VmInstr::Read);
                if let Some(result) = &instr.result {
                    store_result(&mut bc, result);
                }
            }
            TacOp::Write => {
                if let Some(arg) = &instr.arg1 {
                    push_arg(&mut bc, arg);
                }
                bc.push(VmInstr::Write);
            }
        }
    }

    bc.push(VmInstr::Halt);

    // Patch jump targets now that all labels are known.
    for (idx, label) in patches {
        if let Some(&target) = label_map.get(&label) {
            bc[idx] = match &bc[idx] {
                VmInstr::Jmp(_)      => VmInstr::Jmp(target),
                VmInstr::JmpFalse(_) => VmInstr::JmpFalse(target),
                other => other.clone(),
            };
        }
    }

    CodegenOutput { bytecode: bc, function_map: label_map, errors: ir_out.errors }
}

fn push_arg(bc: &mut Vec<VmInstr>, arg: &TacArg) {
    match arg {
        TacArg::IntLit(v)  => bc.push(VmInstr::Push(VmValue::Int(*v))),
        TacArg::RealLit(v) => bc.push(VmInstr::Push(VmValue::Real(*v))),
        TacArg::Name(n)    => bc.push(VmInstr::Load(n.clone())),
        TacArg::Temp(i)    => bc.push(VmInstr::Load(format!("_t{}", i))),
        TacArg::Label(_)   => {}
    }
}

fn store_result(bc: &mut Vec<VmInstr>, result: &TacArg) {
    match result {
        TacArg::Name(n) => bc.push(VmInstr::Store(n.clone())),
        TacArg::Temp(i) => bc.push(VmInstr::Store(format!("_t{}", i))),
        _               => {}
    }
}

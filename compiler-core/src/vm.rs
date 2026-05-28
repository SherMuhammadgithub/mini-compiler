// Stack machine interpreter for the Pascal subset VM.
// Reads input tokens from a pre-supplied string; writes output to a Vec<String>.
use crate::types::{CompilerError, VmInstr, VmValue};
use serde::Serialize;
use std::collections::HashMap;

#[derive(Serialize)]
pub struct VmOutput {
    pub stdout: Vec<String>,
    pub errors: Vec<CompilerError>,
    pub halted: bool,
    pub step_count: usize,
}

struct VmState {
    stack: Vec<VmValue>,
    memory: HashMap<String, VmValue>,
    arrays: HashMap<String, (i64, Vec<VmValue>)>, // name → (low, elements)
    pc: usize,
    input: Vec<String>,
    input_ptr: usize,
    output: Vec<String>,
    call_stack: Vec<usize>, // return addresses
    memory_stack: Vec<HashMap<String, VmValue>>, // saved memory per call frame
    array_stack: Vec<HashMap<String, (i64, Vec<VmValue>)>>, // saved arrays per call frame
    steps: usize,
}

impl VmState {
    fn push(&mut self, v: VmValue) { self.stack.push(v); }
    fn pop(&mut self) -> Result<VmValue, CompilerError> {
        self.stack.pop().ok_or_else(|| runtime_err("stack underflow"))
    }

    fn run(&mut self, program: &[VmInstr], func_map: &HashMap<String, usize>) -> Result<(), CompilerError> {
        let limit = 1_000_000;
        while self.pc < program.len() && self.steps < limit {
            self.steps += 1;
            let instr = program[self.pc].clone();
            match instr {
                VmInstr::Halt => break,
                VmInstr::Push(v) => { self.push(v); self.pc += 1; }
                VmInstr::Pop  => { self.pop()?; self.pc += 1; }
                VmInstr::Dup  => {
                    let v = self.stack.last().cloned().ok_or_else(|| runtime_err("stack underflow"))?;
                    self.push(v); self.pc += 1;
                }
                VmInstr::Add => { let b = self.pop()?; let a = self.pop()?; self.push(arith(a, b, "+")); self.pc += 1; }
                VmInstr::Sub => { let b = self.pop()?; let a = self.pop()?; self.push(arith(a, b, "-")); self.pc += 1; }
                VmInstr::Mul => { let b = self.pop()?; let a = self.pop()?; self.push(arith(a, b, "*")); self.pc += 1; }
                VmInstr::Div => { let b = self.pop()?; let a = self.pop()?; self.push(arith(a, b, "/")); self.pc += 1; }
                VmInstr::Mod => { let b = self.pop()?; let a = self.pop()?; self.push(arith(a, b, "%")); self.pc += 1; }
                VmInstr::Neg => { let a = self.pop()?; self.push(neg_val(a)); self.pc += 1; }
                VmInstr::CmpEq => { let b = self.pop()?; let a = self.pop()?; self.push(cmp(a, b, "==")); self.pc += 1; }
                VmInstr::CmpNe => { let b = self.pop()?; let a = self.pop()?; self.push(cmp(a, b, "!=")); self.pc += 1; }
                VmInstr::CmpLt => { let b = self.pop()?; let a = self.pop()?; self.push(cmp(a, b, "<")); self.pc += 1; }
                VmInstr::CmpLe => { let b = self.pop()?; let a = self.pop()?; self.push(cmp(a, b, "<=")); self.pc += 1; }
                VmInstr::CmpGt => { let b = self.pop()?; let a = self.pop()?; self.push(cmp(a, b, ">")); self.pc += 1; }
                VmInstr::CmpGe => { let b = self.pop()?; let a = self.pop()?; self.push(cmp(a, b, ">=")); self.pc += 1; }
                VmInstr::Not  => { let a = self.pop()?; self.push(VmValue::Bool(!is_truthy(&a))); self.pc += 1; }
                VmInstr::And  => { let b = self.pop()?; let a = self.pop()?; self.push(VmValue::Bool(is_truthy(&a) && is_truthy(&b))); self.pc += 1; }
                VmInstr::Or   => { let b = self.pop()?; let a = self.pop()?; self.push(VmValue::Bool(is_truthy(&a) || is_truthy(&b))); self.pc += 1; }
                VmInstr::Load(n) => {
                    let v = self.memory.get(&n).cloned().unwrap_or(VmValue::Int(0));
                    self.push(v); self.pc += 1;
                }
                VmInstr::Store(n) => {
                    let v = self.pop()?;
                    self.memory.insert(n, v); self.pc += 1;
                }
                VmInstr::AllocArray(n, low, high) => {
                    let size = (high - low + 1) as usize;
                    self.arrays.insert(n, (low, vec![VmValue::Int(0); size]));
                    self.pc += 1;
                }
                VmInstr::LoadIdx(n) => {
                    let idx = self.pop()?;
                    if let Some((low, elements)) = self.arrays.get(&n) {
                        let offset = idx_to_offset(idx, *low, elements.len())?;
                        self.push(elements[offset].clone());
                    } else {
                        self.push(VmValue::Int(0));
                    }
                    self.pc += 1;
                }
                VmInstr::StoreIdx(n) => {
                    let idx = self.pop()?;
                    let val = self.pop()?;
                    if let Some((low, elements)) = self.arrays.get_mut(&n) {
                        let offset = idx_to_offset(idx, *low, elements.len())?;
                        elements[offset] = val;
                    }
                    self.pc += 1;
                }
                VmInstr::Jmp(t) => { self.pc = t; }
                VmInstr::JmpFalse(t) => {
                    let v = self.pop()?;
                    self.pc = if is_truthy(&v) { self.pc + 1 } else { t };
                }
                VmInstr::Call(name) => {
                    self.memory_stack.push(self.memory.clone());
                    self.array_stack.push(self.arrays.clone());
                    self.call_stack.push(self.pc + 1);
                    self.pc = *func_map.get(&name)
                        .ok_or_else(|| runtime_err(&format!("undefined function: {}", name)))?;
                }
                VmInstr::Ret => {
                    if let Some(saved) = self.memory_stack.pop() {
                        self.memory = saved;
                    }
                    if let Some(saved) = self.array_stack.pop() {
                        self.arrays = saved;
                    }
                    self.pc = self.call_stack.pop().unwrap_or(program.len());
                }
                VmInstr::EnterFrame(_) | VmInstr::ExitFrame => { self.pc += 1; }
                VmInstr::Read => {
                    let tok = self.input.get(self.input_ptr).cloned().unwrap_or_default();
                    self.input_ptr += 1;
                    let v = if let Ok(i) = tok.parse::<i64>()  { VmValue::Int(i)  }
                            else if let Ok(f) = tok.parse::<f64>() { VmValue::Real(f) }
                            else { VmValue::Int(0) };
                    self.push(v); self.pc += 1;
                }
                VmInstr::Write => {
                    let s = if self.stack.is_empty() { String::new() } else { fmt_val(&self.pop()?) };
                    self.output.push(s); self.pc += 1;
                }
            }
        }
        Ok(())
    }
}

fn arith(a: VmValue, b: VmValue, op: &str) -> VmValue {
    match (a, b) {
        (VmValue::Int(x), VmValue::Int(y)) => match op {
            "+" => VmValue::Int(x + y),
            "-" => VmValue::Int(x - y),
            "*" => VmValue::Int(x * y),
            "/" => VmValue::Int(if y != 0 { x / y } else { 0 }),
            "%" => VmValue::Int(if y != 0 { x % y } else { 0 }),
            _   => VmValue::Int(0),
        },
        (VmValue::Real(x), VmValue::Real(y)) => match op {
            "+" => VmValue::Real(x + y),
            "-" => VmValue::Real(x - y),
            "*" => VmValue::Real(x * y),
            "/" => VmValue::Real(if y != 0.0 { x / y } else { 0.0 }),
            "%" => VmValue::Real(x % y),
            _   => VmValue::Real(0.0),
        },
        (VmValue::Int(x), VmValue::Real(y)) => arith(VmValue::Real(x as f64), VmValue::Real(y), op),
        (VmValue::Real(x), VmValue::Int(y)) => arith(VmValue::Real(x), VmValue::Real(y as f64), op),
        _ => VmValue::Int(0),
    }
}

fn neg_val(a: VmValue) -> VmValue {
    match a {
        VmValue::Int(x)  => VmValue::Int(-x),
        VmValue::Real(x) => VmValue::Real(-x),
        _                => VmValue::Int(0),
    }
}

fn cmp(a: VmValue, b: VmValue, op: &str) -> VmValue {
    let r = match (&a, &b) {
        (VmValue::Int(x), VmValue::Int(y)) => match op {
            "==" => x == y, "!=" => x != y, "<" => x < y,
            "<=" => x <= y, ">" => x > y,  ">=" => x >= y, _ => false,
        },
        (VmValue::Real(x), VmValue::Real(y)) => match op {
            "==" => x == y, "!=" => x != y, "<" => x < y,
            "<=" => x <= y, ">" => x > y,  ">=" => x >= y, _ => false,
        },
        _ => false,
    };
    VmValue::Bool(r)
}

fn is_truthy(v: &VmValue) -> bool {
    match v {
        VmValue::Int(i)  => *i != 0,
        VmValue::Real(f) => *f != 0.0,
        VmValue::Bool(b) => *b,
    }
}

fn fmt_val(v: &VmValue) -> String {
    match v {
        VmValue::Int(i)  => i.to_string(),
        VmValue::Real(f) => f.to_string(),
        VmValue::Bool(b) => b.to_string(),
    }
}

fn idx_to_offset(idx: VmValue, low: i64, len: usize) -> Result<usize, CompilerError> {
    let i = match idx {
        VmValue::Int(x) => x,
        _ => return Err(runtime_err("array index must be integer")),
    };
    let offset = (i - low) as usize;
    if offset >= len {
        return Err(runtime_err(&format!("array index {} out of bounds (low={}, size={})", i, low, len)));
    }
    Ok(offset)
}

fn runtime_err(msg: &str) -> CompilerError {
    CompilerError { stage: "vm".into(), message: msg.into(), line: 0, column: 0, length: 0, severity: "error".into() }
}

pub fn execute(source: &str, input: &str) -> VmOutput {
    let cg_out = crate::codegen::generate(source);
    if !cg_out.errors.is_empty() {
        return VmOutput { stdout: vec![], errors: cg_out.errors, halted: false, step_count: 0 };
    }
    let mut state = VmState {
        stack: vec![], memory: HashMap::new(), arrays: HashMap::new(), pc: 0,
        input: input.split_whitespace().map(String::from).collect(),
        input_ptr: 0, output: vec![], call_stack: vec![], memory_stack: vec![], array_stack: vec![], steps: 0,
    };
    let err = state.run(&cg_out.bytecode, &cg_out.function_map)
        .err().map(|e| vec![e]).unwrap_or_default();
    let steps = state.steps;
    VmOutput { stdout: state.output, errors: err, halted: true, step_count: steps }
}

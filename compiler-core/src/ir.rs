// TAC (Three-Address Code) IR generation from the typed AST.
// Traverses the typed AST from semantic analysis and emits quadruples.
use crate::ast::{AstNode, NodeKind};
use crate::types::{CompilerError, TacArg, TacInstr, TacOp};
use serde::Serialize;

#[derive(Serialize)]
pub struct IrOutput {
    pub instructions: Vec<TacInstr>,
    pub errors: Vec<CompilerError>,
}

struct IrGen {
    instrs: Vec<TacInstr>,
    temp_ctr: usize,
    label_ctr: usize,
}

impl IrGen {
    fn new_temp(&mut self) -> TacArg {
        let t = self.temp_ctr;
        self.temp_ctr += 1;
        TacArg::Temp(t)
    }

    fn new_label(&mut self) -> String {
        let l = self.label_ctr;
        self.label_ctr += 1;
        format!("L{}", l)
    }

    fn emit(&mut self, instr: TacInstr) { self.instrs.push(instr); }

    fn q(op: TacOp, a1: Option<TacArg>, a2: Option<TacArg>, r: Option<TacArg>) -> TacInstr {
        TacInstr { op, arg1: a1, arg2: a2, result: r }
    }

    fn gen_program(&mut self, node: &AstNode) {
        if let NodeKind::Program { declarations, subprograms, body, .. } = &node.kind {
            self.gen_stmt(declarations);
            self.gen_stmt(body);        // main body first so pc=0 is entry
            self.gen_stmt(subprograms); // function/procedure bodies appended after
        }
    }

    fn gen_stmt(&mut self, node: &AstNode) {
        match &node.kind {
            NodeKind::CompoundStatement { stmts } => {
                for s in stmts { self.gen_stmt(s); }
            }
            NodeKind::Assignment { target, value } => {
                self.gen_assign(target, value);
            }
            NodeKind::IfStatement { cond, then_branch, else_branch } => {
                self.gen_if(cond, then_branch, else_branch.as_deref());
            }
            NodeKind::WhileStatement { cond, body } => {
                self.gen_while(cond, body);
            }
            NodeKind::ProcedureCall { name, args } => {
                self.gen_call_stmt(name, args);
            }
            NodeKind::FunctionDecl { name, params, declarations, body, .. } => {
                self.emit(Self::q(TacOp::Label, Some(TacArg::Label(name.clone())), None, None));
                self.gen_params_load(params);
                self.gen_stmt(declarations);
                self.gen_stmt(body);
                self.emit(Self::q(TacOp::Return, Some(TacArg::Name(name.clone())), None, None));
            }
            NodeKind::ProcedureDecl { name, params, declarations, body, .. } => {
                self.emit(Self::q(TacOp::Label, Some(TacArg::Label(name.clone())), None, None));
                self.gen_params_load(params);
                self.gen_stmt(declarations);
                self.gen_stmt(body);
                self.emit(Self::q(TacOp::Return, None, None, None));
            }
            NodeKind::SubprogramDeclarations { items } => {
                for item in items { self.gen_stmt(item); }
            }
            NodeKind::Declarations { items } => {
                for item in items { self.gen_stmt(item); }
            }
            NodeKind::VarDecl { names, ty } => {
                if let NodeKind::TypeArray { low, high, .. } = &ty.kind {
                    for n in names {
                        self.emit(Self::q(TacOp::DeclArray,
                            Some(TacArg::Name(n.clone())),
                            Some(TacArg::IntLit(*low)),
                            Some(TacArg::IntLit(*high))));
                    }
                }
            }
            _ => {} // TypeInteger, TypeReal, ParamGroup — no code
        }
    }

    fn gen_while(&mut self, cond: &AstNode, body: &AstNode) {
        let start = self.new_label();
        let end   = self.new_label();
        self.emit(Self::q(TacOp::Label, Some(TacArg::Label(start.clone())), None, None));
        let t = self.gen_expr(cond);
        self.emit(Self::q(TacOp::IfFalseGoto, Some(t), Some(TacArg::Label(end.clone())), None));
        self.gen_stmt(body);
        self.emit(Self::q(TacOp::Goto, Some(TacArg::Label(start)), None, None));
        self.emit(Self::q(TacOp::Label, Some(TacArg::Label(end)), None, None));
    }

    fn gen_if(&mut self, cond: &AstNode, then_b: &AstNode, else_b: Option<&AstNode>) {
        let t = self.gen_expr(cond);
        if let Some(else_node) = else_b {
            let else_lbl = self.new_label();
            let end_lbl  = self.new_label();
            self.emit(Self::q(TacOp::IfFalseGoto, Some(t), Some(TacArg::Label(else_lbl.clone())), None));
            self.gen_stmt(then_b);
            self.emit(Self::q(TacOp::Goto, Some(TacArg::Label(end_lbl.clone())), None, None));
            self.emit(Self::q(TacOp::Label, Some(TacArg::Label(else_lbl)), None, None));
            self.gen_stmt(else_node);
            self.emit(Self::q(TacOp::Label, Some(TacArg::Label(end_lbl)), None, None));
        } else {
            let end_lbl = self.new_label();
            self.emit(Self::q(TacOp::IfFalseGoto, Some(t), Some(TacArg::Label(end_lbl.clone())), None));
            self.gen_stmt(then_b);
            self.emit(Self::q(TacOp::Label, Some(TacArg::Label(end_lbl)), None, None));
        }
    }

    fn gen_expr(&mut self, node: &AstNode) -> TacArg {
        match &node.kind {
            NodeKind::IntLiteral { value }  => TacArg::IntLit(*value),
            NodeKind::RealLiteral { value } => TacArg::RealLit(*value),
            NodeKind::Variable { name, index: None } => TacArg::Name(name.clone()),
            NodeKind::Variable { name, index: Some(idx) } => {
                let idx_arg = self.gen_expr(idx);
                let t = self.new_temp();
                self.emit(Self::q(TacOp::CopyFromArray,
                    Some(TacArg::Name(name.clone())), Some(idx_arg), Some(t.clone())));
                t
            }
            NodeKind::BinaryExpr { op, left, right } => {
                let l = self.gen_expr(left);
                let r = self.gen_expr(right);
                let t = self.new_temp();
                self.emit(Self::q(bin_op(op), Some(l), Some(r), Some(t.clone())));
                t
            }
            NodeKind::UnaryExpr { op, operand } => {
                let a = self.gen_expr(operand);
                let t = self.new_temp();
                let tac_op = if op == "not" { TacOp::Not } else { TacOp::Neg };
                self.emit(Self::q(tac_op, Some(a), None, Some(t.clone())));
                t
            }
            NodeKind::FunctionCall { name, args } => {
                for arg in args {
                    let a = self.gen_expr(arg);
                    self.emit(Self::q(TacOp::Param, Some(a), None, None));
                }
                let t = self.new_temp();
                self.emit(Self::q(TacOp::Call,
                    Some(TacArg::Name(name.clone())),
                    Some(TacArg::IntLit(args.len() as i64)),
                    Some(t.clone())));
                t
            }
            _ => self.new_temp(), // unreachable for well-formed AST
        }
    }

    fn gen_assign(&mut self, target: &AstNode, value: &AstNode) {
        let val = self.gen_expr(value);
        match &target.kind {
            NodeKind::Variable { name, index: None } => {
                self.emit(Self::q(TacOp::Assign, Some(val), None, Some(TacArg::Name(name.clone()))));
            }
            NodeKind::Variable { name, index: Some(idx) } => {
                let idx_arg = self.gen_expr(idx);
                self.emit(Self::q(TacOp::CopyToArray,
                    Some(val), Some(idx_arg), Some(TacArg::Name(name.clone()))));
            }
            _ => {}
        }
    }

    fn gen_call_stmt(&mut self, name: &str, args: &[AstNode]) {
        match name {
            "write" | "writeln" => {
                if args.is_empty() {
                    self.emit(Self::q(TacOp::Write, None, None, None));
                } else {
                    for arg in args {
                        let a = self.gen_expr(arg);
                        self.emit(Self::q(TacOp::Write, Some(a), None, None));
                    }
                }
            }
            "read" | "readln" => {
                for arg in args {
                    match &arg.kind {
                        NodeKind::Variable { name: vname, index: None } => {
                            self.emit(Self::q(TacOp::Read, None, None, Some(TacArg::Name(vname.clone()))));
                        }
                        NodeKind::Variable { name: vname, index: Some(idx) } => {
                            let idx_arg = self.gen_expr(idx);
                            let t = self.new_temp();
                            self.emit(Self::q(TacOp::Read, None, None, Some(t.clone())));
                            self.emit(Self::q(TacOp::CopyToArray,
                                Some(t), Some(idx_arg), Some(TacArg::Name(vname.clone()))));
                        }
                        _ => {}
                    }
                }
            }
            _ => {
                for arg in args {
                    let a = self.gen_expr(arg);
                    self.emit(Self::q(TacOp::Param, Some(a), None, None));
                }
                self.emit(Self::q(TacOp::Call,
                    Some(TacArg::Name(name.to_owned())),
                    Some(TacArg::IntLit(args.len() as i64)),
                    None));
            }
        }
    }

    fn gen_params_load(&mut self, params: &[AstNode]) {
        for pg in params {
            if let NodeKind::ParamGroup { names, .. } = &pg.kind {
                for name in names.iter().rev() {
                    self.emit(Self::q(TacOp::PopParam(name.clone()), None, None, None));
                }
            }
        }
    }
}

fn bin_op(op: &str) -> TacOp {
    match op {
        "+"         => TacOp::Add,
        "-"         => TacOp::Sub,
        "*"         => TacOp::Mul,
        "/" | "div" => TacOp::Div,
        "mod"       => TacOp::Mod,
        "="         => TacOp::Eq,
        "<>"        => TacOp::Ne,
        "<"         => TacOp::Lt,
        "<="        => TacOp::Le,
        ">="        => TacOp::Ge,
        ">"         => TacOp::Gt,
        _           => TacOp::Add,
    }
}

pub fn generate(source: &str) -> IrOutput {
    let sem_out = crate::semantic::analyze(source);
    let mut gen = IrGen { instrs: vec![], temp_ctr: 0, label_ctr: 0 };
    if let Some(ref ast) = sem_out.typed_ast { gen.gen_program(ast); }
    IrOutput { instructions: gen.instrs, errors: sem_out.errors }
}

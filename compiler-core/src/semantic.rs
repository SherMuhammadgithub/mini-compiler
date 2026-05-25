// Semantic analysis: symbol table population, scope checking, and type inference/checking.
// Single AST walk — inserts declarations (Phase 9) and type-checks expressions (Phase 10).
use std::collections::HashMap;
use serde::Serialize;
use crate::ast::{AstNode, NodeKind};
use crate::rd_parser;
use crate::symbol_table::{ScopeDump, SymbolTable};
use crate::types::{CompilerError, PascalType, SymbolEntry, SymbolKind};

#[derive(Serialize)]
pub struct SemanticOutput {
    pub typed_ast: Option<AstNode>,
    pub symbol_snapshot: Vec<SymbolEntry>,
    pub errors: Vec<CompilerError>,
    pub scope_dumps: Vec<ScopeDump>,
}

struct Checker {
    sym: SymbolTable,
    errors: Vec<CompilerError>,
    dumps: Vec<ScopeDump>,
    // Maps function/procedure name → ordered list of parameter types (for arity checks).
    func_params: HashMap<String, Vec<PascalType>>,
}

impl Checker {
    fn new(lex_errors: Vec<CompilerError>) -> Self {
        Self {
            sym: SymbolTable::new(),
            errors: lex_errors,
            dumps: vec![],
            func_params: HashMap::new(),
        }
    }

    fn node(&mut self, n: &AstNode) {
        match &n.kind {
            NodeKind::Program { params, declarations, subprograms, body, .. } => {
                for name in params {
                    let _ = self.sym.insert(make_entry(name, SymbolKind::Variable, PascalType::Integer, 0, n.span.line));
                }
                self.node(declarations);
                self.node(subprograms);
                self.node(body);
            }

            NodeKind::Declarations { items } => {
                for item in items { self.node(item); }
            }

            NodeKind::VarDecl { names, ty } => {
                let pt = to_pascal_type(ty);
                let level = self.sym.current_level();
                for name in names {
                    let e = make_entry(name, SymbolKind::Variable, pt.clone(), level, n.span.line);
                    if let Err(err) = self.sym.insert(e) { self.errors.push(err); }
                }
            }

            NodeKind::SubprogramDeclarations { items } => {
                for item in items { self.node(item); }
            }

            NodeKind::FunctionDecl { name, params, return_type, declarations, body } => {
                let ret = to_pascal_type(return_type);
                let level = self.sym.current_level();
                if let Err(e) = self.sym.insert(make_entry(name, SymbolKind::Function, ret, level, n.span.line)) {
                    self.errors.push(e);
                }
                let param_types = collect_param_types(params);
                self.func_params.insert(name.clone(), param_types);
                self.sym.begin_scope();
                for pg in params { self.node(pg); }
                self.node(declarations);
                self.node(body);
                let exited = self.sym.end_scope();
                self.dumps.push(ScopeDump { level: level + 1, entries: exited });
            }

            NodeKind::ProcedureDecl { name, params, declarations, body } => {
                let level = self.sym.current_level();
                if let Err(e) = self.sym.insert(make_entry(name, SymbolKind::Procedure, PascalType::Void, level, n.span.line)) {
                    self.errors.push(e);
                }
                let param_types = collect_param_types(params);
                self.func_params.insert(name.clone(), param_types);
                self.sym.begin_scope();
                for pg in params { self.node(pg); }
                self.node(declarations);
                self.node(body);
                let exited = self.sym.end_scope();
                self.dumps.push(ScopeDump { level: level + 1, entries: exited });
            }

            NodeKind::ParamGroup { names, ty } => {
                let pt = to_pascal_type(ty);
                let level = self.sym.current_level();
                for name in names {
                    let e = make_entry(name, SymbolKind::Parameter, pt.clone(), level, n.span.line);
                    if let Err(err) = self.sym.insert(e) { self.errors.push(err); }
                }
            }

            NodeKind::CompoundStatement { stmts } => {
                for s in stmts { self.node(s); }
            }

            NodeKind::Assignment { target, value } => {
                // Walk sub-trees first so undeclared/subscript checks fire inside them.
                self.node(target);
                self.node(value);
                let target_type = self.infer(target);
                let value_type = self.infer(value);
                if !compatible(&target_type, &value_type) {
                    self.errors.push(type_err(
                        &format!("cannot assign {} to {}", pascal_name(&value_type), pascal_name(&target_type)),
                        n.span.line, n.span.column,
                    ));
                }
            }

            NodeKind::ProcedureCall { name, args } => {
                if !is_builtin(name) {
                    if self.sym.lookup(name).is_none() {
                        self.errors.push(undeclared(name, n.span.line, n.span.column));
                    } else {
                        self.check_call_args(name, args, n.span.line);
                    }
                }
                for arg in args { self.node(arg); }
            }

            NodeKind::IfStatement { cond, then_branch, else_branch } => {
                self.node(cond);
                self.node(then_branch);
                if let Some(e) = else_branch { self.node(e); }
            }

            NodeKind::WhileStatement { cond, body } => {
                self.node(cond);
                self.node(body);
            }

            NodeKind::BinaryExpr { left, right, .. } => {
                self.node(left);
                self.node(right);
            }

            NodeKind::UnaryExpr { operand, .. } => { self.node(operand); }

            NodeKind::Variable { name, index } => {
                if self.sym.lookup(name).is_none() {
                    self.errors.push(undeclared(name, n.span.line, n.span.column));
                }
                // Check 8: subscript on non-array; check 7: index must be integer
                if let Some(idx) = index {
                    let var_type = self.sym.lookup(name).map(|e| e.pascal_type.clone());
                    if let Some(vt) = var_type {
                        if !matches!(vt, PascalType::Array { .. }) {
                            self.errors.push(type_err(
                                &format!("'{}' is not an array", name), n.span.line, n.span.column,
                            ));
                        }
                    }
                    let idx_type = self.infer(idx);
                    if idx_type != PascalType::Integer {
                        self.errors.push(type_err("array index must be integer", n.span.line, n.span.column));
                    }
                    self.node(idx);
                }
            }

            NodeKind::FunctionCall { name, args } => {
                if !is_builtin(name) {
                    if self.sym.lookup(name).is_none() {
                        self.errors.push(undeclared(name, n.span.line, n.span.column));
                    } else {
                        self.check_call_args(name, args, n.span.line);
                    }
                }
                for arg in args { self.node(arg); }
            }

            NodeKind::IntLiteral { .. } | NodeKind::RealLiteral { .. }
            | NodeKind::TypeInteger | NodeKind::TypeReal | NodeKind::TypeArray { .. } => {}
        }
    }

    // Check 5 & 6: arity and argument types for function/procedure calls.
    fn check_call_args(&mut self, name: &str, args: &[AstNode], line: usize) {
        if let Some(expected) = self.func_params.get(name).cloned() {
            if args.len() != expected.len() {
                self.errors.push(type_err(
                    &format!("'{}' expects {} argument(s), got {}", name, expected.len(), args.len()),
                    line, 0,
                ));
            } else {
                for (i, (arg, exp_ty)) in args.iter().zip(expected.iter()).enumerate() {
                    let got = self.infer(arg);
                    if !compatible(exp_ty, &got) {
                        self.errors.push(type_err(
                            &format!("argument {} of '{}': expected {}, got {}", i + 1, name, pascal_name(exp_ty), pascal_name(&got)),
                            line, 0,
                        ));
                    }
                }
            }
        }
    }

    // Type inference — returns PascalType of an expression without walking sub-nodes again.
    fn infer(&self, n: &AstNode) -> PascalType {
        match &n.kind {
            NodeKind::IntLiteral { .. } => PascalType::Integer,
            NodeKind::RealLiteral { .. } => PascalType::Real,
            NodeKind::Variable { name, index } => {
                let base = self.sym.lookup(name).map(|e| e.pascal_type.clone()).unwrap_or(PascalType::Integer);
                if index.is_some() {
                    match base { PascalType::Array { element, .. } => *element, t => t }
                } else {
                    base
                }
            }
            NodeKind::FunctionCall { name, .. } => {
                self.sym.lookup(name).map(|e| e.pascal_type.clone()).unwrap_or(PascalType::Integer)
            }
            NodeKind::BinaryExpr { op, left, right } => {
                if matches!(op.as_str(), "=" | "<>" | "<" | "<=" | ">=" | ">" | "and" | "or") {
                    PascalType::Boolean
                } else {
                    let lt = self.infer(left);
                    let rt = self.infer(right);
                    if lt == PascalType::Real || rt == PascalType::Real { PascalType::Real } else { PascalType::Integer }
                }
            }
            NodeKind::UnaryExpr { op, operand } => {
                if op == "not" { PascalType::Boolean } else { self.infer(operand) }
            }
            _ => PascalType::Integer,
        }
    }
}

/// Widening rule: integer can be assigned to real; same types are always compatible.
fn compatible(target: &PascalType, value: &PascalType) -> bool {
    target == value || matches!((target, value), (PascalType::Real, PascalType::Integer))
}

pub fn analyze(source: &str) -> SemanticOutput {
    let rd_out = rd_parser::parse(source);
    let ast = rd_out.ast;
    let mut checker = Checker::new(rd_out.errors);
    if let Some(ref node) = ast { checker.node(node); }
    SemanticOutput {
        typed_ast: ast,
        symbol_snapshot: checker.sym.snapshot(),
        errors: checker.errors,
        scope_dumps: checker.dumps,
    }
}

fn collect_param_types(param_groups: &[AstNode]) -> Vec<PascalType> {
    let mut types = vec![];
    for pg in param_groups {
        if let NodeKind::ParamGroup { names, ty } = &pg.kind {
            let pt = to_pascal_type(ty);
            for _ in names { types.push(pt.clone()); }
        }
    }
    types
}

fn to_pascal_type(n: &AstNode) -> PascalType {
    match &n.kind {
        NodeKind::TypeInteger => PascalType::Integer,
        NodeKind::TypeReal => PascalType::Real,
        NodeKind::TypeArray { low, high, element } => PascalType::Array {
            low: *low, high: *high, element: Box::new(to_pascal_type(element)),
        },
        _ => PascalType::Integer,
    }
}

fn make_entry(name: &str, kind: SymbolKind, pt: PascalType, level: usize, line: usize) -> SymbolEntry {
    SymbolEntry { name: name.to_owned(), kind, pascal_type: pt, scope_level: level, line, mem_offset: 0 }
}

fn pascal_name(pt: &PascalType) -> &'static str {
    match pt {
        PascalType::Integer => "integer",
        PascalType::Real => "real",
        PascalType::Boolean => "boolean",
        PascalType::Void => "void",
        PascalType::Array { .. } => "array",
    }
}

fn undeclared(name: &str, line: usize, col: usize) -> CompilerError {
    CompilerError { stage: "semantic".into(), message: format!("undeclared identifier '{}'", name), line, column: col, length: name.len(), severity: "error".into() }
}

fn type_err(msg: &str, line: usize, col: usize) -> CompilerError {
    CompilerError { stage: "semantic".into(), message: msg.to_owned(), line, column: col, length: 0, severity: "error".into() }
}

fn is_builtin(name: &str) -> bool {
    matches!(name, "read" | "write" | "writeln" | "readln")
}

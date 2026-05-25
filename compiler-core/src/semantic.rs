// Symbol table population and basic scope/declaration checking for the Pascal subset.
// Walks the RD parser's AST, inserts declarations, and reports duplicate/undeclared identifiers.
use serde::Serialize;
use crate::ast::{AstNode, NodeKind};
use crate::rd_parser;
use crate::symbol_table::{ScopeDump, SymbolTable};
use crate::types::{CompilerError, PascalType, SymbolEntry, SymbolKind};

#[derive(Serialize)]
pub struct SemanticOutput {
    pub entries: Vec<SymbolEntry>,
    pub errors: Vec<CompilerError>,
    pub scope_dumps: Vec<ScopeDump>,
}

/// Parse source with the RD parser, walk the AST, and populate the symbol table.
pub fn analyze(source: &str) -> SemanticOutput {
    let rd_out = rd_parser::parse(source);
    let mut errors = rd_out.errors;
    let mut sym = SymbolTable::new();
    let mut dumps: Vec<ScopeDump> = vec![];

    if let Some(ast) = &rd_out.ast {
        walk(ast, &mut sym, &mut errors, &mut dumps);
    }

    SemanticOutput {
        entries: sym.snapshot(),
        errors,
        scope_dumps: dumps,
    }
}

fn walk(node: &AstNode, sym: &mut SymbolTable, errors: &mut Vec<CompilerError>, dumps: &mut Vec<ScopeDump>) {
    match &node.kind {
        NodeKind::Program { params, declarations, subprograms, body, .. } => {
            // Program parameters are visible in the whole program body
            for name in params {
                let _ = sym.insert(make_var(name, PascalType::Integer, 0, node.span.line));
            }
            walk(declarations, sym, errors, dumps);
            walk(subprograms, sym, errors, dumps);
            walk(body, sym, errors, dumps);
        }

        NodeKind::Declarations { items } => {
            for item in items { walk(item, sym, errors, dumps); }
        }

        NodeKind::VarDecl { names, ty } => {
            let pt = node_to_type(ty);
            let level = sym.current_level();
            for name in names {
                let e = SymbolEntry {
                    name: name.clone(), kind: SymbolKind::Variable,
                    pascal_type: pt.clone(), scope_level: level,
                    line: node.span.line, mem_offset: 0,
                };
                if let Err(err) = sym.insert(e) { errors.push(err); }
            }
        }

        NodeKind::SubprogramDeclarations { items } => {
            for item in items { walk(item, sym, errors, dumps); }
        }

        NodeKind::FunctionDecl { name, params, return_type, declarations, body } => {
            let ret = node_to_type(return_type);
            let level = sym.current_level();
            let e = SymbolEntry {
                name: name.clone(), kind: SymbolKind::Function,
                pascal_type: ret, scope_level: level,
                line: node.span.line, mem_offset: 0,
            };
            if let Err(err) = sym.insert(e) { errors.push(err); }
            sym.begin_scope();
            for pg in params { walk(pg, sym, errors, dumps); }
            walk(declarations, sym, errors, dumps);
            walk(body, sym, errors, dumps);
            let exited = sym.end_scope();
            dumps.push(ScopeDump { level: level + 1, entries: exited });
        }

        NodeKind::ProcedureDecl { name, params, declarations, body } => {
            let level = sym.current_level();
            let e = SymbolEntry {
                name: name.clone(), kind: SymbolKind::Procedure,
                pascal_type: PascalType::Void, scope_level: level,
                line: node.span.line, mem_offset: 0,
            };
            if let Err(err) = sym.insert(e) { errors.push(err); }
            sym.begin_scope();
            for pg in params { walk(pg, sym, errors, dumps); }
            walk(declarations, sym, errors, dumps);
            walk(body, sym, errors, dumps);
            let exited = sym.end_scope();
            dumps.push(ScopeDump { level: level + 1, entries: exited });
        }

        NodeKind::ParamGroup { names, ty } => {
            let pt = node_to_type(ty);
            let level = sym.current_level();
            for name in names {
                let e = SymbolEntry {
                    name: name.clone(), kind: SymbolKind::Parameter,
                    pascal_type: pt.clone(), scope_level: level,
                    line: node.span.line, mem_offset: 0,
                };
                if let Err(err) = sym.insert(e) { errors.push(err); }
            }
        }

        NodeKind::CompoundStatement { stmts } => {
            for s in stmts { walk(s, sym, errors, dumps); }
        }

        NodeKind::Assignment { target, value } => {
            walk(target, sym, errors, dumps);
            walk(value, sym, errors, dumps);
        }

        NodeKind::ProcedureCall { name, args } => {
            if !is_builtin(name) && sym.lookup(name).is_none() {
                errors.push(undeclared(name, node.span.line, node.span.column));
            }
            for arg in args { walk(arg, sym, errors, dumps); }
        }

        NodeKind::IfStatement { cond, then_branch, else_branch } => {
            walk(cond, sym, errors, dumps);
            walk(then_branch, sym, errors, dumps);
            if let Some(e) = else_branch { walk(e, sym, errors, dumps); }
        }

        NodeKind::WhileStatement { cond, body } => {
            walk(cond, sym, errors, dumps);
            walk(body, sym, errors, dumps);
        }

        NodeKind::BinaryExpr { left, right, .. } => {
            walk(left, sym, errors, dumps);
            walk(right, sym, errors, dumps);
        }

        NodeKind::UnaryExpr { operand, .. } => {
            walk(operand, sym, errors, dumps);
        }

        NodeKind::Variable { name, index } => {
            if sym.lookup(name).is_none() {
                errors.push(undeclared(name, node.span.line, node.span.column));
            }
            if let Some(idx) = index { walk(idx, sym, errors, dumps); }
        }

        NodeKind::FunctionCall { name, args } => {
            if !is_builtin(name) && sym.lookup(name).is_none() {
                errors.push(undeclared(name, node.span.line, node.span.column));
            }
            for arg in args { walk(arg, sym, errors, dumps); }
        }

        // Literals and type nodes carry no identifier references
        NodeKind::IntLiteral { .. } | NodeKind::RealLiteral { .. }
        | NodeKind::TypeInteger | NodeKind::TypeReal | NodeKind::TypeArray { .. } => {}
    }
}

fn node_to_type(node: &AstNode) -> PascalType {
    match &node.kind {
        NodeKind::TypeInteger => PascalType::Integer,
        NodeKind::TypeReal => PascalType::Real,
        NodeKind::TypeArray { low, high, element } => PascalType::Array {
            low: *low, high: *high,
            element: Box::new(node_to_type(element)),
        },
        _ => PascalType::Integer,
    }
}

fn make_var(name: &str, pt: PascalType, level: usize, line: usize) -> SymbolEntry {
    SymbolEntry { name: name.to_owned(), kind: SymbolKind::Variable, pascal_type: pt, scope_level: level, line, mem_offset: 0 }
}

fn undeclared(name: &str, line: usize, column: usize) -> CompilerError {
    CompilerError { stage: "semantic".into(), message: format!("undeclared identifier '{}'", name), line, column, length: name.len(), severity: "error".into() }
}

fn is_builtin(name: &str) -> bool {
    matches!(name, "read" | "write" | "writeln" | "readln")
}

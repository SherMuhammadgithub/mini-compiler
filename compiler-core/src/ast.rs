// AST node definitions for the Pascal subset.
// Every node carries a Span so the frontend can highlight source ranges.
use serde::Serialize;

/// Source location for one syntactic construct.
#[derive(Debug, Clone, Serialize)]
pub struct Span {
    pub line: usize,
    pub column: usize,
    pub length: usize,
}

impl Span {
    pub fn zero() -> Self {
        Span {
            line: 0,
            column: 0,
            length: 0,
        }
    }
}

/// One node in the Abstract Syntax Tree.
#[derive(Debug, Clone, Serialize)]
pub struct AstNode {
    pub kind: NodeKind,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize)]
pub enum NodeKind {
    // ── Top level ──────────────────────────────────────────────────────────
    Program {
        name: String,
        params: Vec<String>,
        declarations: Box<AstNode>,
        subprograms: Box<AstNode>,
        body: Box<AstNode>,
    },

    // ── Declarations ───────────────────────────────────────────────────────
    Declarations {
        items: Vec<AstNode>,
    },
    VarDecl {
        names: Vec<String>,
        ty: Box<AstNode>,
    },
    TypeInteger,
    TypeReal,
    TypeArray {
        low: i64,
        high: i64,
        element: Box<AstNode>,
    },

    // ── Subprograms ────────────────────────────────────────────────────────
    SubprogramDeclarations {
        items: Vec<AstNode>,
    },
    FunctionDecl {
        name: String,
        params: Vec<AstNode>,
        return_type: Box<AstNode>,
        declarations: Box<AstNode>,
        body: Box<AstNode>,
    },
    ProcedureDecl {
        name: String,
        params: Vec<AstNode>,
        declarations: Box<AstNode>,
        body: Box<AstNode>,
    },
    ParamGroup {
        names: Vec<String>,
        ty: Box<AstNode>,
    },

    // ── Statements ─────────────────────────────────────────────────────────
    CompoundStatement {
        stmts: Vec<AstNode>,
    },
    Assignment {
        target: Box<AstNode>,
        value: Box<AstNode>,
    },
    ProcedureCall {
        name: String,
        args: Vec<AstNode>,
    },
    IfStatement {
        cond: Box<AstNode>,
        then_branch: Box<AstNode>,
        else_branch: Option<Box<AstNode>>,
    },
    WhileStatement {
        cond: Box<AstNode>,
        body: Box<AstNode>,
    },

    // ── Expressions ────────────────────────────────────────────────────────
    BinaryExpr {
        op: String,
        left: Box<AstNode>,
        right: Box<AstNode>,
    },
    UnaryExpr {
        op: String,
        operand: Box<AstNode>,
    },
    Variable {
        name: String,
        index: Option<Box<AstNode>>,
    },
    FunctionCall {
        name: String,
        args: Vec<AstNode>,
    },
    IntLiteral {
        value: i64,
    },
    RealLiteral {
        value: f64,
    },
}

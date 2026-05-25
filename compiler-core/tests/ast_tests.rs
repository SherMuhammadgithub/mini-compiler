// Serialization tests for AST node types (Phase 5).
// Verifies that serde_json encodes every NodeKind in the shape the TypeScript
// frontend expects (struct variants → {"VariantName":{fields}}, unit variants → "VariantName").
use compiler_core::ast::{AstNode, NodeKind, Span};

fn span() -> Span {
    Span {
        line: 1,
        column: 1,
        length: 0,
    }
}

fn node(kind: NodeKind) -> AstNode {
    AstNode { kind, span: span() }
}

fn boxed(kind: NodeKind) -> Box<AstNode> {
    Box::new(node(kind))
}

fn json(kind: NodeKind) -> serde_json::Value {
    let n = node(kind);
    serde_json::to_value(&n).expect("serialization failed")
}

fn kind_json(kind: NodeKind) -> serde_json::Value {
    json(kind)["kind"].clone()
}

// ── Unit variants ─────────────────────────────────────────────────────────────

#[test]
fn type_integer_serializes_as_string() {
    assert_eq!(kind_json(NodeKind::TypeInteger), "TypeInteger");
}

#[test]
fn type_real_serializes_as_string() {
    assert_eq!(kind_json(NodeKind::TypeReal), "TypeReal");
}

// ── Span ──────────────────────────────────────────────────────────────────────

#[test]
fn span_fields_present() {
    let v = json(NodeKind::TypeInteger);
    assert_eq!(v["span"]["line"], 1);
    assert_eq!(v["span"]["column"], 1);
    assert_eq!(v["span"]["length"], 0);
}

// ── Top level ─────────────────────────────────────────────────────────────────

#[test]
fn program_node_shape() {
    let k = NodeKind::Program {
        name: "test".into(),
        params: vec!["x".into(), "y".into()],
        declarations: boxed(NodeKind::Declarations { items: vec![] }),
        subprograms: boxed(NodeKind::SubprogramDeclarations { items: vec![] }),
        body: boxed(NodeKind::CompoundStatement { stmts: vec![] }),
    };
    let v = kind_json(k);
    assert_eq!(v["Program"]["name"], "test");
    assert_eq!(v["Program"]["params"][0], "x");
    assert_eq!(v["Program"]["params"][1], "y");
}

// ── Declarations ──────────────────────────────────────────────────────────────

#[test]
fn declarations_items_array() {
    let k = NodeKind::Declarations { items: vec![] };
    let v = kind_json(k);
    assert!(v["Declarations"]["items"].is_array());
}

#[test]
fn var_decl_shape() {
    let k = NodeKind::VarDecl {
        names: vec!["a".into(), "b".into()],
        ty: boxed(NodeKind::TypeInteger),
    };
    let v = kind_json(k);
    assert_eq!(v["VarDecl"]["names"][0], "a");
    assert_eq!(v["VarDecl"]["ty"]["kind"], "TypeInteger");
}

#[test]
fn type_array_shape() {
    let k = NodeKind::TypeArray {
        low: 1,
        high: 10,
        element: boxed(NodeKind::TypeInteger),
    };
    let v = kind_json(k);
    assert_eq!(v["TypeArray"]["low"], 1);
    assert_eq!(v["TypeArray"]["high"], 10);
    assert_eq!(v["TypeArray"]["element"]["kind"], "TypeInteger");
}

// ── Subprograms ───────────────────────────────────────────────────────────────

#[test]
fn function_decl_shape() {
    let k = NodeKind::FunctionDecl {
        name: "f".into(),
        params: vec![],
        return_type: boxed(NodeKind::TypeReal),
        declarations: boxed(NodeKind::Declarations { items: vec![] }),
        body: boxed(NodeKind::CompoundStatement { stmts: vec![] }),
    };
    let v = kind_json(k);
    assert_eq!(v["FunctionDecl"]["name"], "f");
    assert_eq!(v["FunctionDecl"]["return_type"]["kind"], "TypeReal");
}

#[test]
fn procedure_decl_shape() {
    let k = NodeKind::ProcedureDecl {
        name: "p".into(),
        params: vec![],
        declarations: boxed(NodeKind::Declarations { items: vec![] }),
        body: boxed(NodeKind::CompoundStatement { stmts: vec![] }),
    };
    let v = kind_json(k);
    assert_eq!(v["ProcedureDecl"]["name"], "p");
}

#[test]
fn param_group_shape() {
    let k = NodeKind::ParamGroup {
        names: vec!["n".into()],
        ty: boxed(NodeKind::TypeInteger),
    };
    let v = kind_json(k);
    assert_eq!(v["ParamGroup"]["names"][0], "n");
}

// ── Statements ────────────────────────────────────────────────────────────────

#[test]
fn compound_statement_shape() {
    let k = NodeKind::CompoundStatement { stmts: vec![] };
    let v = kind_json(k);
    assert!(v["CompoundStatement"]["stmts"].is_array());
}

#[test]
fn assignment_shape() {
    let k = NodeKind::Assignment {
        target: boxed(NodeKind::Variable {
            name: "x".into(),
            index: None,
        }),
        value: boxed(NodeKind::IntLiteral { value: 42 }),
    };
    let v = kind_json(k);
    assert_eq!(v["Assignment"]["target"]["kind"]["Variable"]["name"], "x");
    assert_eq!(v["Assignment"]["value"]["kind"]["IntLiteral"]["value"], 42);
}

#[test]
fn if_statement_with_else_some() {
    let k = NodeKind::IfStatement {
        cond: boxed(NodeKind::IntLiteral { value: 1 }),
        then_branch: boxed(NodeKind::CompoundStatement { stmts: vec![] }),
        else_branch: Some(boxed(NodeKind::CompoundStatement { stmts: vec![] })),
    };
    let v = kind_json(k);
    assert!(!v["IfStatement"]["else_branch"].is_null());
}

#[test]
fn if_statement_else_none_is_null() {
    let k = NodeKind::IfStatement {
        cond: boxed(NodeKind::IntLiteral { value: 0 }),
        then_branch: boxed(NodeKind::CompoundStatement { stmts: vec![] }),
        else_branch: None,
    };
    let v = kind_json(k);
    assert!(v["IfStatement"]["else_branch"].is_null());
}

#[test]
fn while_statement_shape() {
    let k = NodeKind::WhileStatement {
        cond: boxed(NodeKind::IntLiteral { value: 1 }),
        body: boxed(NodeKind::CompoundStatement { stmts: vec![] }),
    };
    let v = kind_json(k);
    assert!(v["WhileStatement"]["body"].is_object());
}

#[test]
fn procedure_call_shape() {
    let k = NodeKind::ProcedureCall {
        name: "writeln".into(),
        args: vec![node(NodeKind::IntLiteral { value: 1 })],
    };
    let v = kind_json(k);
    assert_eq!(v["ProcedureCall"]["name"], "writeln");
    assert_eq!(v["ProcedureCall"]["args"].as_array().unwrap().len(), 1);
}

// ── Expressions ───────────────────────────────────────────────────────────────

#[test]
fn binary_expr_shape() {
    let k = NodeKind::BinaryExpr {
        op: "+".into(),
        left: boxed(NodeKind::IntLiteral { value: 1 }),
        right: boxed(NodeKind::IntLiteral { value: 2 }),
    };
    let v = kind_json(k);
    assert_eq!(v["BinaryExpr"]["op"], "+");
}

#[test]
fn unary_expr_shape() {
    let k = NodeKind::UnaryExpr {
        op: "-".into(),
        operand: boxed(NodeKind::IntLiteral { value: 5 }),
    };
    let v = kind_json(k);
    assert_eq!(v["UnaryExpr"]["op"], "-");
}

#[test]
fn variable_no_index_is_null() {
    let k = NodeKind::Variable {
        name: "arr".into(),
        index: None,
    };
    let v = kind_json(k);
    assert!(v["Variable"]["index"].is_null());
}

#[test]
fn variable_with_index() {
    let k = NodeKind::Variable {
        name: "arr".into(),
        index: Some(boxed(NodeKind::IntLiteral { value: 3 })),
    };
    let v = kind_json(k);
    assert!(!v["Variable"]["index"].is_null());
}

#[test]
fn function_call_shape() {
    let k = NodeKind::FunctionCall {
        name: "sqrt".into(),
        args: vec![node(NodeKind::RealLiteral { value: 2.0 })],
    };
    let v = kind_json(k);
    assert_eq!(v["FunctionCall"]["name"], "sqrt");
}

#[test]
fn int_literal_value() {
    let v = kind_json(NodeKind::IntLiteral { value: 99 });
    assert_eq!(v["IntLiteral"]["value"], 99);
}

#[test]
fn real_literal_value() {
    let v = kind_json(NodeKind::RealLiteral { value: 3.14 });
    let got = v["RealLiteral"]["value"].as_f64().unwrap();
    assert!((got - 3.14).abs() < 1e-10);
}

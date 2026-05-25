// Declaration and subprogram parsing (P1-P21).
use super::{Parser, SubprogramHead};
use crate::ast::{AstNode, NodeKind};
use crate::types::TokenKind;

impl Parser {
    // P1: program → program id ( identifier_list ) ; declarations subprogram_declarations compound_statement .
    pub fn parse_program(&mut self) -> Option<AstNode> {
        self.push_trace("parse_program");
        let span = self.span_here();
        self.expect(&TokenKind::Program)?;
        let name = self.expect_id();
        self.expect(&TokenKind::LParen);
        let params = self.parse_identifier_list();
        self.expect(&TokenKind::RParen);
        self.expect(&TokenKind::Semicolon);
        let declarations = self.parse_declarations();
        let subprograms = self.parse_subprogram_declarations();
        let body = self.parse_compound_statement();
        self.expect(&TokenKind::Dot);
        Some(AstNode {
            kind: NodeKind::Program {
                name,
                params,
                declarations: Box::new(declarations),
                subprograms: Box::new(subprograms),
                body: Box::new(body),
            },
            span,
        })
    }

    // P2-P4: identifier_list → id identifier_list'
    pub fn parse_identifier_list(&mut self) -> Vec<String> {
        self.push_trace("parse_identifier_list");
        let mut ids = vec![self.expect_id()];
        while self.at(&TokenKind::Comma) {
            self.advance();
            ids.push(self.expect_id());
        }
        ids
    }

    // P5-P6: declarations → var identifier_list : type ; declarations | ε
    pub fn parse_declarations(&mut self) -> AstNode {
        self.push_trace("parse_declarations");
        let span = self.span_here();
        let mut items = vec![];
        while self.at(&TokenKind::Var) {
            self.advance();
            let names = self.parse_identifier_list();
            self.expect(&TokenKind::Colon);
            let ty = self.parse_type();
            self.expect(&TokenKind::Semicolon);
            items.push(AstNode {
                kind: NodeKind::VarDecl {
                    names,
                    ty: Box::new(ty),
                },
                span: self.span_here(),
            });
        }
        AstNode {
            kind: NodeKind::Declarations { items },
            span,
        }
    }

    // P7-P8: type → standard_type | array [ num .. num ] of standard_type
    pub fn parse_type(&mut self) -> AstNode {
        self.push_trace("parse_type");
        let span = self.span_here();
        if self.at(&TokenKind::Array) {
            self.advance();
            self.expect(&TokenKind::LBracket);
            let low_s = self.expect_num();
            self.expect(&TokenKind::DotDot);
            let high_s = self.expect_num();
            self.expect(&TokenKind::RBracket);
            self.expect(&TokenKind::Of);
            let element = self.parse_standard_type();
            AstNode {
                kind: NodeKind::TypeArray {
                    low: low_s.parse().unwrap_or(0),
                    high: high_s.parse().unwrap_or(0),
                    element: Box::new(element),
                },
                span,
            }
        } else {
            self.parse_standard_type()
        }
    }

    // P9-P10: standard_type → integer | real
    pub fn parse_standard_type(&mut self) -> AstNode {
        self.push_trace("parse_standard_type");
        let span = self.span_here();
        match self.peek().clone() {
            TokenKind::Integer => {
                self.advance();
                AstNode {
                    kind: NodeKind::TypeInteger,
                    span,
                }
            }
            TokenKind::Real => {
                self.advance();
                AstNode {
                    kind: NodeKind::TypeReal,
                    span,
                }
            }
            _ => {
                let t = self.current();
                self.errors.push(crate::types::CompilerError {
                    stage: "rd_parser".into(),
                    message: format!("expected integer or real, found '{}'", t.lexeme),
                    line: t.line,
                    column: t.column,
                    length: t.lexeme.len(),
                    severity: "error".into(),
                });
                AstNode {
                    kind: NodeKind::TypeInteger,
                    span,
                }
            }
        }
    }

    // P11-P12: subprogram_declarations → subprogram_declaration ; subprogram_declarations | ε
    pub fn parse_subprogram_declarations(&mut self) -> AstNode {
        self.push_trace("parse_subprogram_declarations");
        let span = self.span_here();
        let mut items = vec![];
        while matches!(self.peek(), TokenKind::Function | TokenKind::Procedure) {
            items.push(self.parse_subprogram_declaration());
            self.expect(&TokenKind::Semicolon);
        }
        AstNode {
            kind: NodeKind::SubprogramDeclarations { items },
            span,
        }
    }

    // P13: subprogram_declaration → subprogram_head declarations compound_statement
    pub fn parse_subprogram_declaration(&mut self) -> AstNode {
        self.push_trace("parse_subprogram_declaration");
        let span = self.span_here();
        let head = self.parse_subprogram_head();
        let declarations = self.parse_declarations();
        let body = self.parse_compound_statement();
        match head {
            SubprogramHead::Function {
                name,
                params,
                return_type,
            } => AstNode {
                kind: NodeKind::FunctionDecl {
                    name,
                    params,
                    return_type: Box::new(return_type),
                    declarations: Box::new(declarations),
                    body: Box::new(body),
                },
                span,
            },
            SubprogramHead::Procedure { name, params } => AstNode {
                kind: NodeKind::ProcedureDecl {
                    name,
                    params,
                    declarations: Box::new(declarations),
                    body: Box::new(body),
                },
                span,
            },
        }
    }

    // P14-P15: subprogram_head → function id arguments : standard_type ; | procedure id arguments ;
    pub fn parse_subprogram_head(&mut self) -> SubprogramHead {
        self.push_trace("parse_subprogram_head");
        if matches!(self.peek(), TokenKind::Function) {
            self.advance();
            let name = self.expect_id();
            let params = self.parse_arguments();
            self.expect(&TokenKind::Colon);
            let return_type = self.parse_standard_type();
            self.expect(&TokenKind::Semicolon);
            SubprogramHead::Function {
                name,
                params,
                return_type,
            }
        } else {
            self.expect(&TokenKind::Procedure);
            let name = self.expect_id();
            let params = self.parse_arguments();
            self.expect(&TokenKind::Semicolon);
            SubprogramHead::Procedure { name, params }
        }
    }

    // P16-P17: arguments → ( parameter_list ) | ε
    pub fn parse_arguments(&mut self) -> Vec<AstNode> {
        self.push_trace("parse_arguments");
        if self.at(&TokenKind::LParen) {
            self.advance();
            let params = self.parse_parameter_list();
            self.expect(&TokenKind::RParen);
            params
        } else {
            vec![]
        }
    }

    // P18-P20: parameter_list → identifier_list : type parameter_list'
    pub fn parse_parameter_list(&mut self) -> Vec<AstNode> {
        self.push_trace("parse_parameter_list");
        let mut groups = vec![];
        loop {
            let span = self.span_here();
            let names = self.parse_identifier_list();
            self.expect(&TokenKind::Colon);
            let ty = self.parse_type();
            groups.push(AstNode {
                kind: NodeKind::ParamGroup {
                    names,
                    ty: Box::new(ty),
                },
                span,
            });
            if !self.at(&TokenKind::Semicolon) {
                break;
            }
            self.advance();
        }
        groups
    }

    // P21: compound_statement → begin optional_statements end
    pub fn parse_compound_statement(&mut self) -> AstNode {
        self.push_trace("parse_compound_statement");
        let span = self.span_here();
        self.expect(&TokenKind::Begin);
        let stmts = self.parse_optional_statements();
        self.expect(&TokenKind::End);
        AstNode {
            kind: NodeKind::CompoundStatement { stmts },
            span,
        }
    }
}

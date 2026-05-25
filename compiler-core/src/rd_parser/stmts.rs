// Statement parsing (P22-P37).
use super::Parser;
use crate::ast::{AstNode, NodeKind, Span};
use crate::types::{CompilerError, TokenKind};

impl Parser {
    // P22-P23: optional_statements → statement_list | ε
    pub fn parse_optional_statements(&mut self) -> Vec<AstNode> {
        if matches!(
            self.peek(),
            TokenKind::Id | TokenKind::Begin | TokenKind::If | TokenKind::While
        ) {
            self.parse_statement_list()
        } else {
            vec![]
        }
    }

    // P24-P26: statement_list → statement statement_list'
    pub fn parse_statement_list(&mut self) -> Vec<AstNode> {
        self.push_trace("parse_statement_list");
        let mut stmts = vec![];
        if let Some(s) = self.parse_statement() {
            stmts.push(s);
        }
        while self.at(&TokenKind::Semicolon) {
            self.advance();
            if matches!(
                self.peek(),
                TokenKind::Id | TokenKind::Begin | TokenKind::If | TokenKind::While
            ) {
                if let Some(s) = self.parse_statement() {
                    stmts.push(s);
                }
            }
        }
        stmts
    }

    // P27-P30: statement → id statement_rest | compound_statement | if … | while …
    pub fn parse_statement(&mut self) -> Option<AstNode> {
        self.push_trace("parse_statement");
        let span = self.span_here();
        match self.peek().clone() {
            TokenKind::Id => {
                let name = self.advance().lexeme.clone();
                self.parse_statement_rest(name, span)
            }
            TokenKind::Begin => Some(self.parse_compound_statement()),
            TokenKind::If => {
                self.advance();
                let cond = self.parse_expression();
                self.expect(&TokenKind::Then);
                let then_branch = self.parse_statement()?;
                self.expect(&TokenKind::Else);
                let else_branch = self.parse_statement();
                Some(AstNode {
                    kind: NodeKind::IfStatement {
                        cond: Box::new(cond),
                        then_branch: Box::new(then_branch),
                        else_branch: else_branch.map(Box::new),
                    },
                    span,
                })
            }
            TokenKind::While => {
                self.advance();
                let cond = self.parse_expression();
                self.expect(&TokenKind::Do);
                let body = self.parse_statement()?;
                Some(AstNode {
                    kind: NodeKind::WhileStatement {
                        cond: Box::new(cond),
                        body: Box::new(body),
                    },
                    span,
                })
            }
            _ => {
                let t = self.current();
                self.errors.push(CompilerError {
                    stage: "rd_parser".into(),
                    message: format!("unexpected '{}' in statement", t.lexeme),
                    line: t.line,
                    column: t.column,
                    length: t.lexeme.len(),
                    severity: "error".into(),
                });
                self.synchronize(&[
                    TokenKind::Semicolon,
                    TokenKind::End,
                    TokenKind::Else,
                    TokenKind::Dot,
                ]);
                None
            }
        }
    }

    // P31-P34: statement_rest → [ expr ] := expr | := expr | ( expr_list ) | ε
    pub fn parse_statement_rest(&mut self, name: String, span: Span) -> Option<AstNode> {
        match self.peek().clone() {
            TokenKind::LBracket => {
                self.advance();
                let index = self.parse_expression();
                self.expect(&TokenKind::RBracket);
                self.expect(&TokenKind::Assignop);
                let value = self.parse_expression();
                Some(AstNode {
                    kind: NodeKind::Assignment {
                        target: Box::new(AstNode {
                            kind: NodeKind::Variable {
                                name,
                                index: Some(Box::new(index)),
                            },
                            span: span.clone(),
                        }),
                        value: Box::new(value),
                    },
                    span,
                })
            }
            TokenKind::Assignop => {
                self.advance();
                let value = self.parse_expression();
                Some(AstNode {
                    kind: NodeKind::Assignment {
                        target: Box::new(AstNode {
                            kind: NodeKind::Variable { name, index: None },
                            span: span.clone(),
                        }),
                        value: Box::new(value),
                    },
                    span,
                })
            }
            TokenKind::LParen => {
                self.advance();
                let args = self.parse_expression_list();
                self.expect(&TokenKind::RParen);
                Some(AstNode {
                    kind: NodeKind::ProcedureCall { name, args },
                    span,
                })
            }
            _ => Some(AstNode {
                kind: NodeKind::ProcedureCall { name, args: vec![] },
                span,
            }),
        }
    }

    // P35-P37: expression_list → expression expression_list'
    pub fn parse_expression_list(&mut self) -> Vec<AstNode> {
        let mut exprs = vec![self.parse_expression()];
        while self.at(&TokenKind::Comma) {
            self.advance();
            exprs.push(self.parse_expression());
        }
        exprs
    }
}

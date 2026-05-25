// Expression parsing (P38-P55) and operator matchers.
use super::Parser;
use crate::ast::{AstNode, NodeKind};
use crate::types::{AddopKind, CompilerError, MulopKind, RelopKind, TokenKind};

impl Parser {
    // P38-P40: expression → simple_expression expression'
    pub fn parse_expression(&mut self) -> AstNode {
        let span = self.span_here();
        let left = self.parse_simple_expression();
        if let Some(op) = self.match_relop() {
            let right = self.parse_simple_expression();
            AstNode {
                kind: NodeKind::BinaryExpr {
                    op,
                    left: Box::new(left),
                    right: Box::new(right),
                },
                span,
            }
        } else {
            left
        }
    }

    // P41-P44: simple_expression → sign? term simple_expression'
    pub fn parse_simple_expression(&mut self) -> AstNode {
        let span = self.span_here();
        let sign = self.match_sign();
        let mut node = self.parse_term();
        if let Some(op) = sign {
            node = AstNode {
                kind: NodeKind::UnaryExpr {
                    op,
                    operand: Box::new(node),
                },
                span: span.clone(),
            };
        }
        while let Some(op) = self.match_addop() {
            let right = self.parse_term();
            node = AstNode {
                kind: NodeKind::BinaryExpr {
                    op,
                    left: Box::new(node),
                    right: Box::new(right),
                },
                span: span.clone(),
            };
        }
        node
    }

    // P45-P47: term → factor term'
    pub fn parse_term(&mut self) -> AstNode {
        let span = self.span_here();
        let mut node = self.parse_factor();
        while let Some(op) = self.match_mulop() {
            let right = self.parse_factor();
            node = AstNode {
                kind: NodeKind::BinaryExpr {
                    op,
                    left: Box::new(node),
                    right: Box::new(right),
                },
                span: span.clone(),
            };
        }
        node
    }

    // P48-P51: factor → id factor_rest | num | ( expr ) | not factor
    pub fn parse_factor(&mut self) -> AstNode {
        let span = self.span_here();
        match self.peek().clone() {
            TokenKind::Id => {
                let name = self.advance().lexeme.clone();
                if self.at(&TokenKind::LParen) {
                    self.advance();
                    let args = self.parse_expression_list();
                    self.expect(&TokenKind::RParen);
                    AstNode {
                        kind: NodeKind::FunctionCall { name, args },
                        span,
                    }
                } else if self.at(&TokenKind::LBracket) {
                    self.advance();
                    let index = self.parse_expression();
                    self.expect(&TokenKind::RBracket);
                    AstNode {
                        kind: NodeKind::Variable { name, index: Some(Box::new(index)) },
                        span,
                    }
                } else {
                    AstNode {
                        kind: NodeKind::Variable { name, index: None },
                        span,
                    }
                }
            }
            TokenKind::Num => {
                let lex = self.advance().lexeme.clone();
                if lex.contains('.') || lex.contains('E') {
                    AstNode {
                        kind: NodeKind::RealLiteral {
                            value: lex.parse().unwrap_or(0.0),
                        },
                        span,
                    }
                } else {
                    AstNode {
                        kind: NodeKind::IntLiteral {
                            value: lex.parse().unwrap_or(0),
                        },
                        span,
                    }
                }
            }
            TokenKind::LParen => {
                self.advance();
                let expr = self.parse_expression();
                self.expect(&TokenKind::RParen);
                expr
            }
            TokenKind::Not => {
                self.advance();
                let operand = self.parse_factor();
                AstNode {
                    kind: NodeKind::UnaryExpr {
                        op: "not".into(),
                        operand: Box::new(operand),
                    },
                    span,
                }
            }
            _ => {
                let t = self.current();
                self.errors.push(CompilerError {
                    stage: "rd_parser".into(),
                    message: format!("expected expression, found '{}'", t.lexeme),
                    line: t.line,
                    column: t.column,
                    length: t.lexeme.len(),
                    severity: "error".into(),
                });
                self.synchronize(&[
                    TokenKind::Semicolon,
                    TokenKind::End,
                    TokenKind::RParen,
                    TokenKind::Dot,
                ]);
                AstNode {
                    kind: NodeKind::IntLiteral { value: 0 },
                    span,
                }
            }
        }
    }

    // ── Operator matchers ──────────────────────────────────────────────────────

    pub fn match_relop(&mut self) -> Option<String> {
        let op = match self.peek() {
            TokenKind::Relop(RelopKind::Eq) => "=",
            TokenKind::Relop(RelopKind::Ne) => "<>",
            TokenKind::Relop(RelopKind::Lt) => "<",
            TokenKind::Relop(RelopKind::Le) => "<=",
            TokenKind::Relop(RelopKind::Ge) => ">=",
            TokenKind::Relop(RelopKind::Gt) => ">",
            _ => return None,
        };
        self.advance();
        Some(op.into())
    }

    pub fn match_addop(&mut self) -> Option<String> {
        let op = match self.peek() {
            TokenKind::Addop(AddopKind::Plus) => "+",
            TokenKind::Addop(AddopKind::Minus) => "-",
            TokenKind::Or => "or",
            _ => return None,
        };
        self.advance();
        Some(op.into())
    }

    pub fn match_mulop(&mut self) -> Option<String> {
        let op = match self.peek() {
            TokenKind::Mulop(MulopKind::Star) => "*",
            TokenKind::Mulop(MulopKind::Slash) => "/",
            TokenKind::Div => "div",
            TokenKind::Mod => "mod",
            TokenKind::And => "and",
            _ => return None,
        };
        self.advance();
        Some(op.into())
    }

    pub fn match_sign(&mut self) -> Option<String> {
        let op = match self.peek() {
            TokenKind::Addop(AddopKind::Plus) => "+",
            TokenKind::Addop(AddopKind::Minus) => "-",
            _ => return None,
        };
        self.advance();
        Some(op.into())
    }
}

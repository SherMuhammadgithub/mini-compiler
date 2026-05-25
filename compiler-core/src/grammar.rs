// Pascal subset grammar productions (P1-P55, transformed for LL(1)/RD parsing).
// Split from first_follow.rs to stay under the 400-line file size limit.
// The Grammar struct and NonTerminal enum live in first_follow.rs.
use crate::first_follow::{Grammar, GrammarSymbol, NonTerminal};
use crate::types::{AddopKind, MulopKind, RelopKind, TokenKind as TK};

impl Grammar {
    /// Constructs the full transformed Pascal subset grammar (55 productions).
    pub fn pascal_subset() -> Self {
        use NonTerminal::{
            Arguments, CompoundStatement, Declarations, Expression, ExpressionList,
            ExpressionListPrime, ExpressionPrime, Factor, FactorRest, IdentifierList,
            IdentifierListPrime, OptionalStatements, ParameterList, ParameterListPrime, Sign,
            SimpleExpression, SimpleExpressionPrime, StandardType, Statement, StatementList,
            StatementListPrime, StatementRest, SubprogramDeclaration, SubprogramDeclarations,
            SubprogramHead, Term, TermPrime, Type,
        };

        let t = |k: TK| GrammarSymbol::Terminal(k);
        let nt = |n: NonTerminal| GrammarSymbol::NonTerminal(n);
        let ep = GrammarSymbol::Epsilon;

        let productions = vec![
            // P1: program → program id ( identifier_list ) ; declarations subprogram_declarations compound_statement .
            (
                NonTerminal::Program,
                vec![
                    t(TK::Program),
                    t(TK::Id),
                    t(TK::LParen),
                    nt(IdentifierList),
                    t(TK::RParen),
                    t(TK::Semicolon),
                    nt(Declarations),
                    nt(SubprogramDeclarations),
                    nt(CompoundStatement),
                    t(TK::Dot),
                ],
            ),
            // P2: identifier_list → id identifier_list'
            (IdentifierList, vec![t(TK::Id), nt(IdentifierListPrime)]),
            // P3: identifier_list' → , id identifier_list'
            (
                IdentifierListPrime,
                vec![t(TK::Comma), t(TK::Id), nt(IdentifierListPrime)],
            ),
            // P4: identifier_list' → ε
            (IdentifierListPrime, vec![ep.clone()]),
            // P5: declarations → var identifier_list : type ; declarations
            (
                Declarations,
                vec![
                    t(TK::Var),
                    nt(IdentifierList),
                    t(TK::Colon),
                    nt(Type),
                    t(TK::Semicolon),
                    nt(Declarations),
                ],
            ),
            // P6: declarations → ε
            (Declarations, vec![ep.clone()]),
            // P7: type → standard_type
            (Type, vec![nt(StandardType)]),
            // P8: type → array [ num .. num ] of standard_type
            (
                Type,
                vec![
                    t(TK::Array),
                    t(TK::LBracket),
                    t(TK::Num),
                    t(TK::DotDot),
                    t(TK::Num),
                    t(TK::RBracket),
                    t(TK::Of),
                    nt(StandardType),
                ],
            ),
            // P9: standard_type → integer
            (StandardType, vec![t(TK::Integer)]),
            // P10: standard_type → real
            (StandardType, vec![t(TK::Real)]),
            // P11: subprogram_declarations → subprogram_declaration ; subprogram_declarations
            (
                SubprogramDeclarations,
                vec![
                    nt(SubprogramDeclaration),
                    t(TK::Semicolon),
                    nt(SubprogramDeclarations),
                ],
            ),
            // P12: subprogram_declarations → ε
            (SubprogramDeclarations, vec![ep.clone()]),
            // P13: subprogram_declaration → subprogram_head declarations compound_statement
            (
                SubprogramDeclaration,
                vec![nt(SubprogramHead), nt(Declarations), nt(CompoundStatement)],
            ),
            // P14: subprogram_head → function id arguments : standard_type ;
            (
                SubprogramHead,
                vec![
                    t(TK::Function),
                    t(TK::Id),
                    nt(Arguments),
                    t(TK::Colon),
                    nt(StandardType),
                    t(TK::Semicolon),
                ],
            ),
            // P15: subprogram_head → procedure id arguments ;
            (
                SubprogramHead,
                vec![t(TK::Procedure), t(TK::Id), nt(Arguments), t(TK::Semicolon)],
            ),
            // P16: arguments → ( parameter_list )
            (
                Arguments,
                vec![t(TK::LParen), nt(ParameterList), t(TK::RParen)],
            ),
            // P17: arguments → ε
            (Arguments, vec![ep.clone()]),
            // P18: parameter_list → identifier_list : type parameter_list'
            (
                ParameterList,
                vec![
                    nt(IdentifierList),
                    t(TK::Colon),
                    nt(Type),
                    nt(ParameterListPrime),
                ],
            ),
            // P19: parameter_list' → ; identifier_list : type parameter_list'
            (
                ParameterListPrime,
                vec![
                    t(TK::Semicolon),
                    nt(IdentifierList),
                    t(TK::Colon),
                    nt(Type),
                    nt(ParameterListPrime),
                ],
            ),
            // P20: parameter_list' → ε
            (ParameterListPrime, vec![ep.clone()]),
            // P21: compound_statement → begin optional_statements end
            (
                CompoundStatement,
                vec![t(TK::Begin), nt(OptionalStatements), t(TK::End)],
            ),
            // P22: optional_statements → statement_list
            (OptionalStatements, vec![nt(StatementList)]),
            // P23: optional_statements → ε
            (OptionalStatements, vec![ep.clone()]),
            // P24: statement_list → statement statement_list'
            (StatementList, vec![nt(Statement), nt(StatementListPrime)]),
            // P25: statement_list' → ; statement statement_list'
            (
                StatementListPrime,
                vec![t(TK::Semicolon), nt(Statement), nt(StatementListPrime)],
            ),
            // P26: statement_list' → ε
            (StatementListPrime, vec![ep.clone()]),
            // P27: statement → id statement_rest
            (Statement, vec![t(TK::Id), nt(StatementRest)]),
            // P28: statement → compound_statement
            (Statement, vec![nt(CompoundStatement)]),
            // P29: statement → if expression then statement else statement
            (
                Statement,
                vec![
                    t(TK::If),
                    nt(Expression),
                    t(TK::Then),
                    nt(Statement),
                    t(TK::Else),
                    nt(Statement),
                ],
            ),
            // P30: statement → while expression do statement
            (
                Statement,
                vec![t(TK::While), nt(Expression), t(TK::Do), nt(Statement)],
            ),
            // P31: statement_rest → [ expression ] := expression
            (
                StatementRest,
                vec![
                    t(TK::LBracket),
                    nt(Expression),
                    t(TK::RBracket),
                    t(TK::Assignop),
                    nt(Expression),
                ],
            ),
            // P32: statement_rest → ( expression_list )
            (
                StatementRest,
                vec![t(TK::LParen), nt(ExpressionList), t(TK::RParen)],
            ),
            // P33: statement_rest → := expression
            (StatementRest, vec![t(TK::Assignop), nt(Expression)]),
            // P34: statement_rest → ε
            (StatementRest, vec![ep.clone()]),
            // P35: expression_list → expression expression_list'
            (
                ExpressionList,
                vec![nt(Expression), nt(ExpressionListPrime)],
            ),
            // P36: expression_list' → , expression expression_list'
            (
                ExpressionListPrime,
                vec![t(TK::Comma), nt(Expression), nt(ExpressionListPrime)],
            ),
            // P37: expression_list' → ε
            (ExpressionListPrime, vec![ep.clone()]),
            // P38: expression → simple_expression expression'
            (Expression, vec![nt(SimpleExpression), nt(ExpressionPrime)]),
            // P39: expression' → relop simple_expression
            (
                ExpressionPrime,
                vec![t(TK::Relop(RelopKind::Eq)), nt(SimpleExpression)],
            ),
            // P40: expression' → ε
            (ExpressionPrime, vec![ep.clone()]),
            // P41: simple_expression → sign term simple_expression'
            (
                SimpleExpression,
                vec![nt(Sign), nt(Term), nt(SimpleExpressionPrime)],
            ),
            // P42: simple_expression → term simple_expression'
            (SimpleExpression, vec![nt(Term), nt(SimpleExpressionPrime)]),
            // P43: simple_expression' → addop term simple_expression'
            (
                SimpleExpressionPrime,
                vec![
                    t(TK::Addop(AddopKind::Plus)),
                    nt(Term),
                    nt(SimpleExpressionPrime),
                ],
            ),
            // P44: simple_expression' → ε
            (SimpleExpressionPrime, vec![ep.clone()]),
            // P45: term → factor term'
            (Term, vec![nt(Factor), nt(TermPrime)]),
            // P46: term' → mulop factor term'
            (
                TermPrime,
                vec![t(TK::Mulop(MulopKind::Star)), nt(Factor), nt(TermPrime)],
            ),
            // P47: term' → ε
            (TermPrime, vec![ep.clone()]),
            // P48: factor → id factor_rest
            (Factor, vec![t(TK::Id), nt(FactorRest)]),
            // P49: factor → num
            (Factor, vec![t(TK::Num)]),
            // P50: factor → ( expression )
            (Factor, vec![t(TK::LParen), nt(Expression), t(TK::RParen)]),
            // P51: factor → not factor
            (Factor, vec![t(TK::Not), nt(Factor)]),
            // P52: factor_rest → ( expression_list )
            (
                FactorRest,
                vec![t(TK::LParen), nt(ExpressionList), t(TK::RParen)],
            ),
            // P52b: factor_rest → [ expression ]
            (
                FactorRest,
                vec![t(TK::LBracket), nt(Expression), t(TK::RBracket)],
            ),
            // P53: factor_rest → ε
            (FactorRest, vec![ep.clone()]),
            // P54: sign → +
            (Sign, vec![t(TK::Addop(AddopKind::Plus))]),
            // P55: sign → -
            (Sign, vec![t(TK::Addop(AddopKind::Minus))]),
        ];

        Grammar { productions }
    }
}

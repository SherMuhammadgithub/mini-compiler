// FIRST and FOLLOW set computation for the transformed Pascal subset grammar.
// Productions are in grammar.rs (split to keep file sizes under 400 lines).
// Consumed by ll1_table.rs (Phase 7) and the Phase 18 WASM report export.

use crate::types::TokenKind;
use serde::Serialize;
use std::collections::{HashMap, HashSet};

// ── Grammar symbol types ───────────────────────────────────────────────────────

/// One symbol in a grammar production — terminal, non-terminal, or ε.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GrammarSymbol {
    Terminal(TokenKind),
    NonTerminal(NonTerminal),
    Epsilon,
    Eof,
}

/// Every non-terminal in the transformed Pascal subset grammar.
/// Prime variants (') are denoted with the suffix `Prime`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NonTerminal {
    Program,
    IdentifierList,
    IdentifierListPrime,
    Declarations,
    Type,
    StandardType,
    SubprogramDeclarations,
    SubprogramDeclaration,
    SubprogramHead,
    Arguments,
    ParameterList,
    ParameterListPrime,
    CompoundStatement,
    OptionalStatements,
    StatementList,
    StatementListPrime,
    Statement,
    StatementRest,
    ExpressionList,
    ExpressionListPrime,
    Expression,
    ExpressionPrime,
    SimpleExpression,
    SimpleExpressionPrime,
    Term,
    TermPrime,
    Factor,
    FactorRest,
    Sign,
}

impl NonTerminal {
    /// All non-terminals in a fixed order (used for report generation).
    pub fn all() -> Vec<NonTerminal> {
        vec![
            NonTerminal::Program,
            NonTerminal::IdentifierList,
            NonTerminal::IdentifierListPrime,
            NonTerminal::Declarations,
            NonTerminal::Type,
            NonTerminal::StandardType,
            NonTerminal::SubprogramDeclarations,
            NonTerminal::SubprogramDeclaration,
            NonTerminal::SubprogramHead,
            NonTerminal::Arguments,
            NonTerminal::ParameterList,
            NonTerminal::ParameterListPrime,
            NonTerminal::CompoundStatement,
            NonTerminal::OptionalStatements,
            NonTerminal::StatementList,
            NonTerminal::StatementListPrime,
            NonTerminal::Statement,
            NonTerminal::StatementRest,
            NonTerminal::ExpressionList,
            NonTerminal::ExpressionListPrime,
            NonTerminal::Expression,
            NonTerminal::ExpressionPrime,
            NonTerminal::SimpleExpression,
            NonTerminal::SimpleExpressionPrime,
            NonTerminal::Term,
            NonTerminal::TermPrime,
            NonTerminal::Factor,
            NonTerminal::FactorRest,
            NonTerminal::Sign,
        ]
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            NonTerminal::Program => "program",
            NonTerminal::IdentifierList => "identifier_list",
            NonTerminal::IdentifierListPrime => "identifier_list'",
            NonTerminal::Declarations => "declarations",
            NonTerminal::Type => "type",
            NonTerminal::StandardType => "standard_type",
            NonTerminal::SubprogramDeclarations => "subprogram_declarations",
            NonTerminal::SubprogramDeclaration => "subprogram_declaration",
            NonTerminal::SubprogramHead => "subprogram_head",
            NonTerminal::Arguments => "arguments",
            NonTerminal::ParameterList => "parameter_list",
            NonTerminal::ParameterListPrime => "parameter_list'",
            NonTerminal::CompoundStatement => "compound_statement",
            NonTerminal::OptionalStatements => "optional_statements",
            NonTerminal::StatementList => "statement_list",
            NonTerminal::StatementListPrime => "statement_list'",
            NonTerminal::Statement => "statement",
            NonTerminal::StatementRest => "statement_rest",
            NonTerminal::ExpressionList => "expression_list",
            NonTerminal::ExpressionListPrime => "expression_list'",
            NonTerminal::Expression => "expression",
            NonTerminal::ExpressionPrime => "expression'",
            NonTerminal::SimpleExpression => "simple_expression",
            NonTerminal::SimpleExpressionPrime => "simple_expression'",
            NonTerminal::Term => "term",
            NonTerminal::TermPrime => "term'",
            NonTerminal::Factor => "factor",
            NonTerminal::FactorRest => "factor_rest",
            NonTerminal::Sign => "sign",
        }
    }
}

// ── Grammar ────────────────────────────────────────────────────────────────────

/// The transformed Pascal subset grammar.
/// Productions are in grammar.rs via a separate impl block.
pub struct Grammar {
    pub productions: Vec<(NonTerminal, Vec<GrammarSymbol>)>,
}

/// Computed FIRST and FOLLOW maps for all non-terminals.
pub struct FirstFollowSets {
    pub first: HashMap<NonTerminal, HashSet<GrammarSymbol>>,
    pub follow: HashMap<NonTerminal, HashSet<GrammarSymbol>>,
}

impl Grammar {
    /// Computes FIRST and FOLLOW sets using a fixed-point (worklist) algorithm.
    /// Iterates until no new terminal is added to any set.
    pub fn compute_first_follow(&self) -> FirstFollowSets {
        let all_nts = NonTerminal::all();
        let mut first: HashMap<NonTerminal, HashSet<GrammarSymbol>> = HashMap::new();
        let mut follow: HashMap<NonTerminal, HashSet<GrammarSymbol>> = HashMap::new();
        for nt in &all_nts {
            first.insert(nt.clone(), HashSet::new());
            follow.insert(nt.clone(), HashSet::new());
        }

        // FOLLOW(start symbol) = { $ }
        follow
            .get_mut(&NonTerminal::Program)
            .unwrap()
            .insert(GrammarSymbol::Eof);

        // ── FIRST fixed-point ──────────────────────────────────────────────────
        loop {
            let mut changed = false;
            for (lhs, rhs) in &self.productions {
                let new = first_of_string(rhs, &first);
                let set = first.get_mut(lhs).unwrap();
                let before = set.len();
                set.extend(new);
                if set.len() > before {
                    changed = true;
                }
            }
            if !changed {
                break;
            }
        }

        // ── FOLLOW fixed-point ─────────────────────────────────────────────────
        loop {
            let mut changed = false;
            for (lhs, rhs) in &self.productions {
                for i in 0..rhs.len() {
                    let b = match &rhs[i] {
                        GrammarSymbol::NonTerminal(nt) => nt.clone(),
                        _ => continue,
                    };

                    let beta = &rhs[i + 1..];
                    let mut first_beta = first_of_string(beta, &first);
                    let beta_nullable = first_beta.remove(&GrammarSymbol::Epsilon);

                    let before = follow[&b].len();
                    follow.get_mut(&b).unwrap().extend(first_beta);
                    if beta_nullable {
                        // Clone to avoid simultaneous borrow
                        let fl: Vec<_> = follow[lhs].iter().cloned().collect();
                        follow.get_mut(&b).unwrap().extend(fl);
                    }
                    if follow[&b].len() > before {
                        changed = true;
                    }
                }
            }
            if !changed {
                break;
            }
        }

        FirstFollowSets { first, follow }
    }
}

/// Computes FIRST of a sequence of grammar symbols.
/// Returns {ε} for an empty sequence (needed for FOLLOW of trailing symbols).
pub fn first_of_string(
    symbols: &[GrammarSymbol],
    first: &HashMap<NonTerminal, HashSet<GrammarSymbol>>,
) -> HashSet<GrammarSymbol> {
    let mut result = HashSet::new();
    if symbols.is_empty() {
        result.insert(GrammarSymbol::Epsilon);
        return result;
    }
    for sym in symbols {
        match sym {
            GrammarSymbol::Terminal(_) | GrammarSymbol::Eof => {
                result.insert(sym.clone());
                return result;
            }
            GrammarSymbol::Epsilon => {
                result.insert(GrammarSymbol::Epsilon);
                return result;
            }
            GrammarSymbol::NonTerminal(nt) => {
                let first_nt = &first[nt];
                result.extend(
                    first_nt
                        .iter()
                        .filter(|s| **s != GrammarSymbol::Epsilon)
                        .cloned(),
                );
                if !first_nt.contains(&GrammarSymbol::Epsilon) {
                    return result;
                }
            }
        }
    }
    result.insert(GrammarSymbol::Epsilon);
    result
}

// ── Phase 18 serializable report ──────────────────────────────────────────────

/// Human-readable name for a grammar symbol (used in report table cells).
pub fn symbol_display(sym: &GrammarSymbol) -> String {
    match sym {
        GrammarSymbol::Terminal(t) => token_display(t),
        GrammarSymbol::NonTerminal(nt) => nt.display_name().to_owned(),
        GrammarSymbol::Epsilon => "ε".to_owned(),
        GrammarSymbol::Eof => "$".to_owned(),
    }
}

fn token_display(t: &TokenKind) -> String {
    match t {
        TokenKind::Program => "program".into(),
        TokenKind::Var => "var".into(),
        TokenKind::Array => "array".into(),
        TokenKind::Of => "of".into(),
        TokenKind::Integer => "integer".into(),
        TokenKind::Real => "real".into(),
        TokenKind::Function => "function".into(),
        TokenKind::Procedure => "procedure".into(),
        TokenKind::Begin => "begin".into(),
        TokenKind::End => "end".into(),
        TokenKind::If => "if".into(),
        TokenKind::Then => "then".into(),
        TokenKind::Else => "else".into(),
        TokenKind::While => "while".into(),
        TokenKind::Do => "do".into(),
        TokenKind::Not => "not".into(),
        TokenKind::And => "and".into(),
        TokenKind::Or => "or".into(),
        TokenKind::Div => "div".into(),
        TokenKind::Mod => "mod".into(),
        TokenKind::Id => "id".into(),
        TokenKind::Num => "num".into(),
        TokenKind::Relop(_) => "relop".into(),
        TokenKind::Addop(_) => "addop".into(),
        TokenKind::Mulop(_) => "mulop".into(),
        TokenKind::Assignop => ":=".into(),
        TokenKind::LParen => "(".into(),
        TokenKind::RParen => ")".into(),
        TokenKind::LBracket => "[".into(),
        TokenKind::RBracket => "]".into(),
        TokenKind::Semicolon => ";".into(),
        TokenKind::Colon => ":".into(),
        TokenKind::Comma => ",".into(),
        TokenKind::Dot => ".".into(),
        TokenKind::DotDot => "..".into(),
        TokenKind::Eof => "$".into(),
        TokenKind::Unknown => "?".into(),
    }
}

#[derive(Serialize)]
pub struct FirstFollowRow {
    pub non_terminal: String,
    pub first: Vec<String>,
    pub follow: Vec<String>,
}

#[derive(Serialize)]
pub struct FirstFollowReport {
    pub rows: Vec<FirstFollowRow>,
}

/// Build the serializable FIRST/FOLLOW report (called by the Phase 18 WASM export).
pub fn report() -> FirstFollowReport {
    let sets = Grammar::pascal_subset().compute_first_follow();
    let rows = NonTerminal::all()
        .into_iter()
        .map(|nt| {
            let mut first_strs: Vec<String> = sets.first[&nt].iter().map(symbol_display).collect();
            first_strs.sort();
            let mut follow_strs: Vec<String> =
                sets.follow[&nt].iter().map(symbol_display).collect();
            follow_strs.sort();
            FirstFollowRow {
                non_terminal: nt.display_name().to_owned(),
                first: first_strs,
                follow: follow_strs,
            }
        })
        .collect();
    FirstFollowReport { rows }
}

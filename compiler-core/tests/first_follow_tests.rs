// Integration tests for FIRST/FOLLOW set computation.
// Grammar productions live in grammar.rs; algorithm lives in first_follow.rs.
use compiler_core::first_follow::{Grammar, GrammarSymbol, NonTerminal};
use compiler_core::types::{AddopKind, TokenKind};

fn sets() -> compiler_core::first_follow::FirstFollowSets {
    Grammar::pascal_subset().compute_first_follow()
}

fn term(k: TokenKind) -> GrammarSymbol {
    GrammarSymbol::Terminal(k)
}

// ── FIRST set tests ────────────────────────────────────────────────────────────

#[test]
fn first_program() {
    let s = sets();
    assert!(s.first[&NonTerminal::Program].contains(&term(TokenKind::Program)));
    assert_eq!(s.first[&NonTerminal::Program].len(), 1);
}

#[test]
fn first_identifier_list() {
    let s = sets();
    assert!(s.first[&NonTerminal::IdentifierList].contains(&term(TokenKind::Id)));
    assert_eq!(s.first[&NonTerminal::IdentifierList].len(), 1);
}

#[test]
fn first_identifier_list_prime_has_epsilon() {
    let s = sets();
    let f = &s.first[&NonTerminal::IdentifierListPrime];
    assert!(f.contains(&term(TokenKind::Comma)));
    assert!(f.contains(&GrammarSymbol::Epsilon));
}

#[test]
fn first_declarations_has_epsilon() {
    let s = sets();
    let f = &s.first[&NonTerminal::Declarations];
    assert!(f.contains(&term(TokenKind::Var)));
    assert!(f.contains(&GrammarSymbol::Epsilon));
}

#[test]
fn first_type() {
    let s = sets();
    let f = &s.first[&NonTerminal::Type];
    assert!(f.contains(&term(TokenKind::Integer)));
    assert!(f.contains(&term(TokenKind::Real)));
    assert!(f.contains(&term(TokenKind::Array)));
    assert!(!f.contains(&GrammarSymbol::Epsilon));
}

#[test]
fn first_standard_type() {
    let s = sets();
    let f = &s.first[&NonTerminal::StandardType];
    assert!(f.contains(&term(TokenKind::Integer)));
    assert!(f.contains(&term(TokenKind::Real)));
    assert!(!f.contains(&GrammarSymbol::Epsilon));
}

#[test]
fn first_subprogram_declarations_has_epsilon() {
    let s = sets();
    let f = &s.first[&NonTerminal::SubprogramDeclarations];
    assert!(f.contains(&term(TokenKind::Function)));
    assert!(f.contains(&term(TokenKind::Procedure)));
    assert!(f.contains(&GrammarSymbol::Epsilon));
}

#[test]
fn first_compound_statement() {
    let s = sets();
    let f = &s.first[&NonTerminal::CompoundStatement];
    assert!(f.contains(&term(TokenKind::Begin)));
    assert_eq!(f.len(), 1);
}

#[test]
fn first_statement_no_epsilon() {
    let s = sets();
    let f = &s.first[&NonTerminal::Statement];
    assert!(f.contains(&term(TokenKind::Id)));
    assert!(f.contains(&term(TokenKind::Begin)));
    assert!(f.contains(&term(TokenKind::If)));
    assert!(f.contains(&term(TokenKind::While)));
    assert!(!f.contains(&GrammarSymbol::Epsilon));
}

#[test]
fn first_expression_contains_id_num_lparen_not_sign() {
    let s = sets();
    let f = &s.first[&NonTerminal::Expression];
    assert!(f.contains(&term(TokenKind::Id)));
    assert!(f.contains(&term(TokenKind::Num)));
    assert!(f.contains(&term(TokenKind::LParen)));
    assert!(f.contains(&term(TokenKind::Not)));
    assert!(f.contains(&term(TokenKind::Addop(AddopKind::Plus))));
    assert!(f.contains(&term(TokenKind::Addop(AddopKind::Minus))));
    assert!(!f.contains(&GrammarSymbol::Epsilon));
}

#[test]
fn first_factor() {
    let s = sets();
    let f = &s.first[&NonTerminal::Factor];
    assert!(f.contains(&term(TokenKind::Id)));
    assert!(f.contains(&term(TokenKind::Num)));
    assert!(f.contains(&term(TokenKind::LParen)));
    assert!(f.contains(&term(TokenKind::Not)));
    assert!(!f.contains(&GrammarSymbol::Epsilon));
}

// ── FOLLOW set tests ───────────────────────────────────────────────────────────

#[test]
fn follow_program_is_eof() {
    let s = sets();
    let f = &s.follow[&NonTerminal::Program];
    assert!(f.contains(&GrammarSymbol::Eof));
    assert_eq!(f.len(), 1);
}

#[test]
fn follow_declarations() {
    let s = sets();
    let f = &s.follow[&NonTerminal::Declarations];
    assert!(f.contains(&term(TokenKind::Function)));
    assert!(f.contains(&term(TokenKind::Procedure)));
    assert!(f.contains(&term(TokenKind::Begin)));
}

#[test]
fn follow_compound_statement_contains_dot() {
    let s = sets();
    let f = &s.follow[&NonTerminal::CompoundStatement];
    assert!(f.contains(&term(TokenKind::Dot)));
}

#[test]
fn follow_statement_contains_semicolon_end_else() {
    let s = sets();
    let f = &s.follow[&NonTerminal::Statement];
    assert!(f.contains(&term(TokenKind::Semicolon)));
    assert!(f.contains(&term(TokenKind::End)));
    assert!(f.contains(&term(TokenKind::Else)));
}

#[test]
fn follow_expression_contains_then_do_rparen() {
    let s = sets();
    let f = &s.follow[&NonTerminal::Expression];
    assert!(f.contains(&term(TokenKind::Then)));
    assert!(f.contains(&term(TokenKind::Do)));
    assert!(f.contains(&term(TokenKind::RParen)));
}

#[test]
fn no_epsilon_in_follow_sets() {
    let s = sets();
    for nt in NonTerminal::all() {
        assert!(
            !s.follow[&nt].contains(&GrammarSymbol::Epsilon),
            "FOLLOW({}) must not contain ε",
            nt.display_name()
        );
    }
}

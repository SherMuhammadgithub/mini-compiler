// LL(1) parsing table M[A, a] = production index, built from FIRST/FOLLOW sets.
use crate::first_follow::{first_of_string, symbol_display, Grammar, GrammarSymbol, NonTerminal};
use crate::types::{AddopKind, MulopKind, RelopKind, TokenKind};
use serde::Serialize;
use std::collections::HashMap;

/// Maps (NonTerminal, terminal TokenKind) → index into Grammar.productions.
pub type Ll1Table = HashMap<(NonTerminal, TokenKind), usize>;

/// Builds the LL(1) parsing table from the grammar's FIRST and FOLLOW sets.
/// Algorithm: for each A → α, add A→α to M[A,a] for each a in FIRST(α)\{ε};
///            if ε ∈ FIRST(α), add to M[A,b] for each b in FOLLOW(A).
pub fn build_ll1_table(grammar: &Grammar) -> Ll1Table {
    let sets = grammar.compute_first_follow();
    let mut table: Ll1Table = HashMap::new();

    for (idx, (lhs, rhs)) in grammar.productions.iter().enumerate() {
        let first_rhs = first_of_string(rhs, &sets.first);

        for sym in &first_rhs {
            if let GrammarSymbol::Terminal(tok) = sym {
                table.insert((lhs.clone(), tok.clone()), idx);
            }
        }

        if first_rhs.contains(&GrammarSymbol::Epsilon) {
            for sym in &sets.follow[lhs] {
                match sym {
                    GrammarSymbol::Terminal(tok) => {
                        table.entry((lhs.clone(), tok.clone())).or_insert(idx);
                    }
                    GrammarSymbol::Eof => {
                        table.entry((lhs.clone(), TokenKind::Eof)).or_insert(idx);
                    }
                    _ => {}
                }
            }
        }
    }

    // Post-process: expand operator class representatives to all concrete variants.
    // Grammar uses Relop(Eq), Addop(Plus), Mulop(Star) as class representatives.
    // or/div/mod/and are keyword tokens — not Addop/Mulop variants — so add them too.
    let nts: Vec<NonTerminal> = NonTerminal::all();
    for nt in &nts {
        // Relop class — all six operators map to the same production
        if let Some(&idx) = table.get(&(nt.clone(), TokenKind::Relop(RelopKind::Eq))) {
            for rk in [
                RelopKind::Ne,
                RelopKind::Lt,
                RelopKind::Le,
                RelopKind::Ge,
                RelopKind::Gt,
            ] {
                table
                    .entry((nt.clone(), TokenKind::Relop(rk)))
                    .or_insert(idx);
            }
        }

        // Addop class — representative is Addop(Plus); or is a keyword token
        if let Some(&idx) = table.get(&(nt.clone(), TokenKind::Addop(AddopKind::Plus))) {
            table
                .entry((nt.clone(), TokenKind::Addop(AddopKind::Minus)))
                .or_insert(idx);
            table.entry((nt.clone(), TokenKind::Or)).or_insert(idx);
        }

        // Mulop class — representative is Mulop(Star); div/mod/and are keyword tokens
        if let Some(&idx) = table.get(&(nt.clone(), TokenKind::Mulop(MulopKind::Star))) {
            for k in [
                TokenKind::Mulop(MulopKind::Slash),
                TokenKind::Div,
                TokenKind::Mod,
                TokenKind::And,
            ] {
                table.entry((nt.clone(), k)).or_insert(idx);
            }
        }
    }

    table
}

// ── Phase 18 report ───────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct Ll1TableRow {
    pub non_terminal: String,
    pub cells: HashMap<String, String>,
}

#[derive(Serialize)]
pub struct Ll1TableReport {
    pub terminals: Vec<String>,
    pub rows: Vec<Ll1TableRow>,
}

pub fn build_ll1_table_report() -> Ll1TableReport {
    let grammar = Grammar::pascal_subset();
    let table = build_ll1_table(&grammar);

    // Collect every terminal that appears in any cell, sorted alphabetically.
    let mut term_set: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    for ((_nt, tok), _) in &table {
        term_set.insert(symbol_display(&GrammarSymbol::Terminal(tok.clone())));
    }
    let terminals: Vec<String> = term_set.into_iter().collect();

    let rows = NonTerminal::all()
        .into_iter()
        .map(|nt| {
            let mut cells: HashMap<String, String> = HashMap::new();
            for ((row_nt, tok), &prod_idx) in &table {
                if row_nt != &nt {
                    continue;
                }
                let term_str = symbol_display(&GrammarSymbol::Terminal(tok.clone()));
                let (lhs, rhs) = &grammar.productions[prod_idx];
                let rhs_str = rhs.iter().map(symbol_display).collect::<Vec<_>>().join(" ");
                cells.insert(term_str, format!("{} → {}", lhs.display_name(), rhs_str));
            }
            Ll1TableRow { non_terminal: nt.display_name().to_owned(), cells }
        })
        .collect();

    Ll1TableReport { terminals, rows }
}

/// True when a stack terminal and the current input token are in the same operator class.
/// Needed because grammar uses one representative token per operator class in the RHS.
pub fn terminals_match(stack_term: &TokenKind, input_tok: &TokenKind) -> bool {
    match (stack_term, input_tok) {
        (TokenKind::Relop(_), TokenKind::Relop(_)) => true,
        (TokenKind::Addop(_), TokenKind::Addop(_)) => true,
        (TokenKind::Addop(_), TokenKind::Or) => true,
        (TokenKind::Mulop(_), TokenKind::Mulop(_)) => true,
        (TokenKind::Mulop(_), TokenKind::Div | TokenKind::Mod | TokenKind::And) => true,
        _ => stack_term == input_tok,
    }
}

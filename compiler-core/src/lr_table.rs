// SLR(1) action and goto table construction from LR(0) item sets.
// Uses FOLLOW sets for reduce decisions (SLR approach, equivalent to LALR(1) for this grammar).
use crate::first_follow::{symbol_display, FirstFollowSets, Grammar, GrammarSymbol, NonTerminal};
use crate::lr_items::{build_lr_grammar, canonical_collection, LrProd};
use crate::types::{AddopKind, MulopKind, RelopKind, TokenKind};
use serde::Serialize;
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize)]
pub enum LrAction {
    Shift(usize),
    Reduce(usize), // production index in the LrProd list
    Accept,
}

pub struct LrTable {
    pub action: Vec<HashMap<TokenKind, LrAction>>,
    pub goto_map: Vec<HashMap<NonTerminal, usize>>,
    pub prods: Vec<LrProd>,
    pub n_states: usize,
}

pub fn build_lalr1_table(grammar: &Grammar) -> LrTable {
    let prods = build_lr_grammar(grammar);
    let sets = grammar.compute_first_follow();
    let (states, transitions) = canonical_collection(&prods);
    let n = states.len();

    let mut action: Vec<HashMap<TokenKind, LrAction>> = vec![HashMap::new(); n];
    let mut goto_map: Vec<HashMap<NonTerminal, usize>> = vec![HashMap::new(); n];

    // --- Shifts and gotos from transitions ---
    // Shifts: use exact token (no class expansion) so Sign's + and - go to different states.
    for (s, trans) in transitions.iter().enumerate() {
        for (sym, next) in trans {
            match sym {
                GrammarSymbol::Terminal(tok) => {
                    action[s].insert(tok.clone(), LrAction::Shift(*next));
                }
                GrammarSymbol::NonTerminal(nt) => {
                    goto_map[s].insert(nt.clone(), *next);
                }
                _ => {}
            }
        }
    }

    // --- Reduces and accept from complete items ---
    // Reduces use FOLLOW sets expanded across operator classes.
    for (s, items) in states.iter().enumerate() {
        for item in items {
            if !item.is_complete(&prods) {
                continue;
            }
            let prod = &prods[item.production];
            if prod.lhs.is_none() {
                // Augmented production S' → Program · : accept on EOF
                action[s].entry(TokenKind::Eof).or_insert(LrAction::Accept);
            } else if let Some(nt) = &prod.lhs {
                for tok in follow_expanded(nt, &sets) {
                    // prefer existing shift over reduce (shift-reduce conflict resolution)
                    action[s]
                        .entry(tok)
                        .or_insert(LrAction::Reduce(item.production));
                }
            }
        }
    }

    LrTable {
        action,
        goto_map,
        prods,
        n_states: n,
    }
}

/// Extracts FOLLOW(nt) as a list of TokenKinds, expanding operator class representatives.
fn follow_expanded(nt: &NonTerminal, sets: &FirstFollowSets) -> Vec<TokenKind> {
    let mut toks = vec![];
    for sym in &sets.follow[nt] {
        match sym {
            GrammarSymbol::Terminal(t) => {
                for k in expand_op(t) {
                    toks.push(k);
                }
            }
            GrammarSymbol::Eof => toks.push(TokenKind::Eof),
            _ => {}
        }
    }
    toks
}

/// Expands a grammar representative token to all concrete variants in its class.
pub fn expand_op(tok: &TokenKind) -> Vec<TokenKind> {
    match tok {
        TokenKind::Relop(_) => vec![
            TokenKind::Relop(RelopKind::Eq),
            TokenKind::Relop(RelopKind::Ne),
            TokenKind::Relop(RelopKind::Lt),
            TokenKind::Relop(RelopKind::Le),
            TokenKind::Relop(RelopKind::Ge),
            TokenKind::Relop(RelopKind::Gt),
        ],
        TokenKind::Addop(_) => vec![
            TokenKind::Addop(AddopKind::Plus),
            TokenKind::Addop(AddopKind::Minus),
            TokenKind::Or,
        ],
        TokenKind::Mulop(_) => vec![
            TokenKind::Mulop(MulopKind::Star),
            TokenKind::Mulop(MulopKind::Slash),
            TokenKind::Div,
            TokenKind::Mod,
            TokenKind::And,
        ],
        other => vec![other.clone()],
    }
}

/// Look up the action for a token, falling back to the operator class representative
/// when no exact entry exists (handles Or/Div/Mod/And and addop/mulop/relop variants).
pub fn lookup_action<'a>(
    row: &'a HashMap<TokenKind, LrAction>,
    tok: &TokenKind,
) -> Option<&'a LrAction> {
    row.get(tok).or_else(|| row.get(&class_rep(tok)))
}

// ── Phase 18 report ───────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct LrTableReport {
    pub terminals:     Vec<String>,
    pub non_terminals: Vec<String>,
    pub action_rows:   Vec<HashMap<String, String>>,
    pub goto_rows:     Vec<HashMap<String, usize>>,
}

pub fn build_lalr1_table_report() -> LrTableReport {
    let grammar = Grammar::pascal_subset();
    let table = build_lalr1_table(&grammar);

    // Collect all terminals that appear in any action row, sorted alphabetically.
    let mut term_set: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    for row in &table.action {
        for tok in row.keys() {
            term_set.insert(symbol_display(&GrammarSymbol::Terminal(tok.clone())));
        }
    }
    let terminals: Vec<String> = term_set.into_iter().collect();

    // Non-terminals that have at least one goto entry.
    let non_terminals: Vec<String> = NonTerminal::all()
        .into_iter()
        .filter(|nt| table.goto_map.iter().any(|row| row.contains_key(nt)))
        .map(|nt| nt.display_name().to_owned())
        .collect();

    let action_rows: Vec<HashMap<String, String>> = table
        .action
        .iter()
        .map(|row| {
            row.iter()
                .map(|(tok, act)| {
                    let key = symbol_display(&GrammarSymbol::Terminal(tok.clone()));
                    let val = match act {
                        LrAction::Shift(s)  => format!("s{}", s),
                        LrAction::Reduce(p) => format!("r{}", p),
                        LrAction::Accept    => "acc".to_owned(),
                    };
                    (key, val)
                })
                .collect()
        })
        .collect();

    let goto_rows: Vec<HashMap<String, usize>> = table
        .goto_map
        .iter()
        .map(|row| {
            row.iter()
                .map(|(nt, &s)| (nt.display_name().to_owned(), s))
                .collect()
        })
        .collect();

    LrTableReport { terminals, non_terminals, action_rows, goto_rows }
}

/// Returns the grammar's class representative for operator tokens, or clones the token.
fn class_rep(tok: &TokenKind) -> TokenKind {
    match tok {
        TokenKind::Addop(_) => TokenKind::Addop(AddopKind::Plus),
        TokenKind::Mulop(_) => TokenKind::Mulop(MulopKind::Star),
        TokenKind::Relop(_) => TokenKind::Relop(RelopKind::Eq),
        TokenKind::Or => TokenKind::Addop(AddopKind::Plus),
        TokenKind::Div | TokenKind::Mod | TokenKind::And => TokenKind::Mulop(MulopKind::Star),
        other => other.clone(),
    }
}

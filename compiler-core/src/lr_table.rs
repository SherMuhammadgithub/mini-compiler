// SLR(1) action and goto table construction from LR(0) item sets.
// Uses FOLLOW sets for reduce decisions (SLR approach, equivalent to LALR(1) for this grammar).
use std::collections::HashMap;
use serde::Serialize;
use crate::first_follow::{Grammar, GrammarSymbol, NonTerminal, FirstFollowSets};
use crate::lr_items::{LrProd, build_lr_grammar, canonical_collection};
use crate::types::{AddopKind, MulopKind, RelopKind, TokenKind};

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

pub fn build_lr_table(grammar: &Grammar) -> LrTable {
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
            if !item.is_complete(&prods) { continue; }
            let prod = &prods[item.prod as usize];
            if prod.lhs.is_none() {
                // Augmented production S' → Program · : accept on EOF
                action[s].entry(TokenKind::Eof).or_insert(LrAction::Accept);
            } else if let Some(nt) = &prod.lhs {
                for tok in follow_expanded(nt, &sets) {
                    // prefer existing shift over reduce (shift-reduce conflict resolution)
                    action[s].entry(tok).or_insert(LrAction::Reduce(item.prod as usize));
                }
            }
        }
    }

    LrTable { action, goto_map, prods, n_states: n }
}

/// Extracts FOLLOW(nt) as a list of TokenKinds, expanding operator class representatives.
fn follow_expanded(nt: &NonTerminal, sets: &FirstFollowSets) -> Vec<TokenKind> {
    let mut toks = vec![];
    for sym in &sets.follow[nt] {
        match sym {
            GrammarSymbol::Terminal(t) => {
                for k in expand_op(t) { toks.push(k); }
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
            TokenKind::Relop(RelopKind::Eq), TokenKind::Relop(RelopKind::Ne),
            TokenKind::Relop(RelopKind::Lt), TokenKind::Relop(RelopKind::Le),
            TokenKind::Relop(RelopKind::Ge), TokenKind::Relop(RelopKind::Gt),
        ],
        TokenKind::Addop(_) => vec![
            TokenKind::Addop(AddopKind::Plus), TokenKind::Addop(AddopKind::Minus), TokenKind::Or,
        ],
        TokenKind::Mulop(_) => vec![
            TokenKind::Mulop(MulopKind::Star), TokenKind::Mulop(MulopKind::Slash),
            TokenKind::Div, TokenKind::Mod, TokenKind::And,
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

/// Returns the grammar's class representative for operator tokens, or clones the token.
fn class_rep(tok: &TokenKind) -> TokenKind {
    match tok {
        TokenKind::Addop(_) => TokenKind::Addop(AddopKind::Plus),
        TokenKind::Mulop(_) => TokenKind::Mulop(MulopKind::Star),
        TokenKind::Relop(_) => TokenKind::Relop(RelopKind::Eq),
        TokenKind::Or       => TokenKind::Addop(AddopKind::Plus),
        TokenKind::Div | TokenKind::Mod | TokenKind::And => TokenKind::Mulop(MulopKind::Star),
        other => other.clone(),
    }
}

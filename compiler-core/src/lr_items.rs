// LR(0) item computation for the Pascal LALR(1) parser.
// Uses the transformed grammar (Grammar::pascal_subset) with epsilon stripped.
use std::collections::{HashMap, HashSet};
use crate::first_follow::{Grammar, GrammarSymbol, NonTerminal};

/// One production with Epsilon stripped from the RHS.
#[derive(Clone, Debug)]
pub struct LrProd {
    pub lhs: Option<NonTerminal>, // None = augmented start symbol S'
    pub rhs: Vec<GrammarSymbol>,
    pub source_idx: Option<usize>, // index in Grammar.productions (None = augmented)
}

/// LR(0) item: production index + dot position.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct LrItem {
    pub prod: u16,
    pub dot: u16,
}

impl LrItem {
    pub fn is_complete(&self, prods: &[LrProd]) -> bool {
        self.dot as usize >= prods[self.prod as usize].rhs.len()
    }

    pub fn sym_after_dot<'a>(&self, prods: &'a [LrProd]) -> Option<&'a GrammarSymbol> {
        prods[self.prod as usize].rhs.get(self.dot as usize)
    }
}

/// Sorted item list — used as a canonical key for state deduplication.
pub type ItemSet = Vec<LrItem>;

/// Build the augmented grammar for LR: augmented S'→Program at index 0,
/// followed by all Grammar productions with GrammarSymbol::Epsilon stripped.
pub fn build_lr_grammar(g: &Grammar) -> Vec<LrProd> {
    let mut prods = vec![LrProd {
        lhs: None, // S'
        rhs: vec![GrammarSymbol::NonTerminal(NonTerminal::Program)],
        source_idx: None,
    }];
    for (i, (lhs, rhs)) in g.productions.iter().enumerate() {
        prods.push(LrProd {
            lhs: Some(lhs.clone()),
            rhs: rhs.iter().filter(|s| **s != GrammarSymbol::Epsilon).cloned().collect(),
            source_idx: Some(i),
        });
    }
    prods
}

/// Closure: expand all items where the dot is before a non-terminal.
pub fn closure(seeds: ItemSet, prods: &[LrProd]) -> ItemSet {
    let mut set: HashSet<LrItem> = seeds.into_iter().collect();
    let mut worklist: Vec<LrItem> = set.iter().cloned().collect();
    while let Some(item) = worklist.pop() {
        let Some(GrammarSymbol::NonTerminal(nt)) = item.sym_after_dot(prods) else { continue };
        for (i, prod) in prods.iter().enumerate() {
            if prod.lhs.as_ref() == Some(nt) {
                let new = LrItem { prod: i as u16, dot: 0 };
                if set.insert(new.clone()) {
                    worklist.push(new);
                }
            }
        }
    }
    let mut v: Vec<LrItem> = set.into_iter().collect();
    v.sort_unstable();
    v
}

/// Goto: advance the dot past `sym` in every matching item, then close.
pub fn goto(items: &ItemSet, sym: &GrammarSymbol, prods: &[LrProd]) -> ItemSet {
    let seeds: Vec<LrItem> = items.iter()
        .filter(|it| it.sym_after_dot(prods) == Some(sym))
        .map(|it| LrItem { prod: it.prod, dot: it.dot + 1 })
        .collect();
    if seeds.is_empty() { vec![] } else { closure(seeds, prods) }
}

/// Builds the canonical LR(0) item collection.
/// Returns (states, transitions) where transitions[s] = [(symbol, next_state_idx), ...].
pub fn canonical_collection(prods: &[LrProd]) -> (Vec<ItemSet>, Vec<Vec<(GrammarSymbol, usize)>>) {
    let initial = closure(vec![LrItem { prod: 0, dot: 0 }], prods);
    let mut states: Vec<ItemSet> = vec![initial];
    let mut state_map: HashMap<ItemSet, usize> = HashMap::new();
    state_map.insert(states[0].clone(), 0);
    let mut transitions: Vec<Vec<(GrammarSymbol, usize)>> = vec![vec![]];

    let mut i = 0;
    while i < states.len() {
        // Collect distinct symbols after dots using HashSet (GrammarSymbol: Hash+Eq).
        let syms: HashSet<GrammarSymbol> = states[i].iter()
            .filter_map(|it| it.sym_after_dot(prods).cloned())
            .collect();
        // Sort for deterministic state numbering.
        let mut syms: Vec<GrammarSymbol> = syms.into_iter().collect();
        syms.sort_by_key(|s| format!("{:?}", s));

        let state_i = states[i].clone();
        for sym in syms {
            let next = goto(&state_i, &sym, prods);
            if next.is_empty() { continue; }
            let next_idx = if let Some(&idx) = state_map.get(&next) {
                idx
            } else {
                let idx = states.len();
                state_map.insert(next.clone(), idx);
                states.push(next);
                transitions.push(vec![]);
                idx
            };
            transitions[i].push((sym, next_idx));
        }
        i += 1;
    }
    (states, transitions)
}

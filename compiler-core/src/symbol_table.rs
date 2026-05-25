// Hash-based symbol table with nested scope stack.
// djb2 hash, TABLE_SIZE=211, separate chaining.
// Full implementation in Phase 9.
// Stub module — SymbolTable struct defined so semantic.rs can import it.
use crate::types::SymbolEntry;

pub const TABLE_SIZE: usize = 211;

// djb2 hash — distributes strings well with few collisions
pub fn djb2_hash(s: &str) -> usize {
    let mut h: usize = 5381;
    for b in s.bytes() {
        h = h.wrapping_mul(33).wrapping_add(b as usize);
    }
    h % TABLE_SIZE
}

pub struct ScopeTable {
    pub slots: Vec<Vec<SymbolEntry>>,
    pub level: usize,
}

impl ScopeTable {
    pub fn new(level: usize) -> Self {
        Self {
            slots: vec![vec![]; TABLE_SIZE],
            level,
        }
    }
}

pub struct SymbolTable {
    scopes: Vec<ScopeTable>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self { scopes: vec![] }
    }

    pub fn begin_scope(&mut self) {
        let level = self.scopes.len();
        self.scopes.push(ScopeTable::new(level));
    }

    pub fn end_scope(&mut self) {
        self.scopes.pop();
    }

    pub fn insert(&mut self, entry: SymbolEntry) {
        if let Some(scope) = self.scopes.last_mut() {
            let idx = djb2_hash(&entry.name);
            scope.slots[idx].push(entry);
        }
    }

    /// Search from innermost scope outward.
    pub fn lookup(&self, name: &str) -> Option<&SymbolEntry> {
        let idx = djb2_hash(name);
        for scope in self.scopes.iter().rev() {
            for entry in &scope.slots[idx] {
                if entry.name == name {
                    return Some(entry);
                }
            }
        }
        None
    }

    /// Snapshot all visible symbols (for the frontend panel).
    pub fn snapshot(&self) -> Vec<SymbolEntry> {
        let mut out = vec![];
        for scope in &self.scopes {
            for chain in &scope.slots {
                for entry in chain {
                    out.push(entry.clone());
                }
            }
        }
        out
    }
}

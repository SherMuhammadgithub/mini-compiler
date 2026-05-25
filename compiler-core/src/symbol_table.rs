// Hash-based symbol table with nested scope stack (separate chaining, djb2 hash).
use crate::types::{CompilerError, SymbolEntry};
use serde::Serialize;

pub const TABLE_SIZE: usize = 211; // prime — reduces clustering

// djb2 hash — distributes identifier strings well with few collisions
pub fn djb2_hash(s: &str) -> usize {
    s.bytes().fold(5381usize, |h, c| h.wrapping_mul(33).wrapping_add(c as usize)) % TABLE_SIZE
}

/// One hash table for a single scope, using separate chaining.
pub struct ScopeTable {
    pub slots: Vec<Vec<SymbolEntry>>,
    pub level: usize,
}

impl ScopeTable {
    pub fn new(level: usize) -> Self {
        Self { slots: vec![vec![]; TABLE_SIZE], level }
    }

    /// Insert a new entry. Returns an error if the name already exists in this scope.
    pub fn insert(&mut self, entry: SymbolEntry) -> Result<(), CompilerError> {
        let slot = djb2_hash(&entry.name);
        if self.slots[slot].iter().any(|e| e.name == entry.name) {
            return Err(CompilerError {
                stage: "symbol_table".into(),
                message: format!("'{}' is already declared in this scope", entry.name),
                line: entry.line,
                column: 0,
                length: entry.name.len(),
                severity: "error".into(),
            });
        }
        self.slots[slot].push(entry);
        Ok(())
    }

    /// Find an entry by name in this scope (chain search).
    pub fn lookup(&self, name: &str) -> Option<&SymbolEntry> {
        self.slots[djb2_hash(name)].iter().find(|e| e.name == name)
    }

    /// Remove an entry by name from this scope.
    pub fn delete(&mut self, name: &str) {
        let slot = djb2_hash(name);
        self.slots[slot].retain(|e| e.name != name);
    }

    /// Collect all entries (called when printing or exiting a scope).
    pub fn all_entries(&self) -> Vec<SymbolEntry> {
        self.slots.iter().flatten().cloned().collect()
    }
}

/// Stacked symbol table — each function/procedure body opens a new ScopeTable.
pub struct SymbolTable {
    scopes: Vec<ScopeTable>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self { scopes: vec![ScopeTable::new(0)] }
    }

    /// Push a new inner scope.
    pub fn begin_scope(&mut self) {
        self.scopes.push(ScopeTable::new(self.scopes.len()));
    }

    /// Pop and return entries from the innermost scope.
    pub fn end_scope(&mut self) -> Vec<SymbolEntry> {
        self.scopes.pop().map(|s| s.all_entries()).unwrap_or_default()
    }

    pub fn current_level(&self) -> usize {
        self.scopes.len() - 1
    }

    /// Search from innermost scope outward (shadowing: inner wins).
    pub fn lookup(&self, name: &str) -> Option<&SymbolEntry> {
        for scope in self.scopes.iter().rev() {
            if let Some(e) = scope.lookup(name) {
                return Some(e);
            }
        }
        None
    }

    /// Insert into the current (innermost) scope.
    pub fn insert(&mut self, entry: SymbolEntry) -> Result<(), CompilerError> {
        self.scopes.last_mut().expect("at least global scope").insert(entry)
    }

    /// Collect every entry from all scopes (for final report).
    pub fn snapshot(&self) -> Vec<SymbolEntry> {
        self.scopes.iter().flat_map(|s| s.all_entries()).collect()
    }
}

/// One scope's entries recorded on exit — used for debug dumps.
#[derive(Serialize)]
pub struct ScopeDump {
    pub level: usize,
    pub entries: Vec<SymbolEntry>,
}

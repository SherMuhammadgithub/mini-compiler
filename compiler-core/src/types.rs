// Shared types for the entire Pascal subset compiler pipeline.
// Every stage imports from here — nothing is redefined elsewhere.
use serde::Serialize;

/// A single token produced by the lexer.
#[derive(Debug, Clone, Serialize)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum TokenKind {
    // Keywords
    Program,
    Var,
    Array,
    Of,
    Integer,
    Real,
    Function,
    Procedure,
    Begin,
    End,
    If,
    Then,
    Else,
    While,
    Do,
    Not,
    And,
    Or,
    Div,
    Mod,
    // Identifiers & literals
    Id,
    Num,
    // Operators (grouped by category for the symbol table)
    Relop(RelopKind),
    Addop(AddopKind),
    Mulop(MulopKind),
    Assignop,
    // Punctuation
    LParen,
    RParen,
    LBracket,
    RBracket,
    Semicolon,
    Colon,
    Comma,
    Dot,
    DotDot,
    // Special
    Eof,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum RelopKind {
    Eq,
    Ne,
    Lt,
    Le,
    Ge,
    Gt,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum AddopKind {
    Plus,
    Minus,
    Or,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum MulopKind {
    Star,
    Slash,
    Div,
    Mod,
    And,
}

/// A compiler error from any stage — carries enough info for Monaco to draw squiggles.
#[derive(Debug, Clone, Serialize)]
pub struct CompilerError {
    pub stage: String, // "lexer" | "rd_parser" | "ll1" | "lr" | "semantic" | ...
    pub message: String,
    pub line: usize,
    pub column: usize,
    pub length: usize,    // underline span in the editor
    pub severity: String, // "error" | "warning" | "info"
}

/// Standard return type for every compiler stage.
pub type StageResult<T> = Result<T, Vec<CompilerError>>;

/// One symbol table entry — everything the semantic checker and code generator need.
#[derive(Debug, Clone, Serialize)]
pub struct SymbolEntry {
    pub name: String,
    pub kind: SymbolKind,
    pub pascal_type: PascalType,
    pub scope_level: usize,
    pub line: usize,
    pub mem_offset: usize,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum SymbolKind {
    Variable,
    Constant,
    Function,
    Procedure,
    Parameter,
    Array,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum PascalType {
    Integer,
    Real,
    Boolean,
    Void,
    // low/high are the declared bounds, element is the element type
    Array {
        low: i64,
        high: i64,
        element: Box<PascalType>,
    },
}

/// Three-address code instruction — one quadruple (op, arg1, arg2, result).
#[derive(Debug, Clone, Serialize)]
pub struct TacInstr {
    pub op: TacOp,
    pub arg1: Option<TacArg>,
    pub arg2: Option<TacArg>,
    pub result: Option<TacArg>,
}

#[derive(Debug, Clone, Serialize)]
pub enum TacOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Neg,
    Not,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    Assign,
    CopyToArray,
    CopyFromArray,
    Label,
    Goto,
    IfFalseGoto,
    Param,
    Call,
    Return,
    Read,
    Write,
}

#[derive(Debug, Clone, Serialize)]
pub enum TacArg {
    Temp(usize),
    Name(String),
    IntLit(i64),
    RealLit(f64),
    Label(String),
}

/// Stack VM instruction — each TAC quadruple maps to a few of these.
#[derive(Debug, Clone, Serialize)]
pub enum VmInstr {
    Push(VmValue),
    Pop,
    Dup,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Neg,
    CmpEq,
    CmpNe,
    CmpLt,
    CmpLe,
    CmpGt,
    CmpGe,
    Not,
    And,
    Or,
    Load(String),
    Store(String),
    LoadIdx,
    StoreIdx,
    Jmp(usize),
    JmpFalse(usize),
    Call(String),
    Ret,
    EnterFrame(usize),
    ExitFrame,
    Read,
    Write,
    Halt,
}

#[derive(Debug, Clone, Serialize)]
pub enum VmValue {
    Int(i64),
    Real(f64),
    Bool(bool),
}

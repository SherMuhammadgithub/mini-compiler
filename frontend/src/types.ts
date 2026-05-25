// TypeScript interfaces mirroring every Rust struct in compiler-core/src/types.rs and ast.rs.
// serde JSON encoding rules:
//   unit enum variant   → "VariantName"
//   tuple enum variant  → { "VariantName": value }
//   struct enum variant → { "VariantName": { ...fields } }

// ── Token types ────────────────────────────────────────────────────────────────

export type RelopKind = "Eq" | "Ne" | "Lt" | "Le" | "Ge" | "Gt";
export type AddopKind = "Plus" | "Minus" | "Or";
export type MulopKind = "Star" | "Slash" | "Div" | "Mod" | "And";

export type TokenKind =
  | "Program" | "Var" | "Array" | "Of" | "Integer" | "Real"
  | "Function" | "Procedure" | "Begin" | "End"
  | "If" | "Then" | "Else" | "While" | "Do" | "Not"
  | "And" | "Or" | "Div" | "Mod"
  | "Id" | "Num"
  | { Relop: RelopKind }
  | { Addop: AddopKind }
  | { Mulop: MulopKind }
  | "Assignop"
  | "LParen" | "RParen" | "LBracket" | "RBracket"
  | "Semicolon" | "Colon" | "Comma" | "Dot" | "DotDot"
  | "Eof" | "Unknown";

export interface Token {
  kind:   TokenKind;
  lexeme: string;
  line:   number;
  column: number;
}

// ── Error type ─────────────────────────────────────────────────────────────────

export interface CompilerError {
  stage:    string;   // "lexer" | "rd_parser" | "ll1" | "lr" | "semantic" | ...
  message:  string;
  line:     number;
  column:   number;
  length:   number;
  severity: string;   // "error" | "warning" | "info"
}

// ── Symbol table types ─────────────────────────────────────────────────────────

export type SymbolKind =
  | "Variable" | "Constant" | "Function" | "Procedure" | "Parameter" | "Array";

export type PascalType =
  | "Integer" | "Real" | "Boolean" | "Void"
  | { Array: { low: number; high: number; element: PascalType } };

export interface SymbolEntry {
  name:        string;
  kind:        SymbolKind;
  pascal_type: PascalType;
  scope_level: number;
  line:        number;
  mem_offset:  number;
}

// ── TAC IR types ───────────────────────────────────────────────────────────────

export type TacOp =
  | "Add" | "Sub" | "Mul" | "Div" | "Mod"
  | "Neg" | "Not"
  | "Eq" | "Ne" | "Lt" | "Le" | "Gt" | "Ge"
  | "Assign" | "CopyToArray" | "CopyFromArray"
  | "Label" | "Goto" | "IfFalseGoto"
  | "Param" | "Call" | "Return"
  | "Read" | "Write";

export type TacArg =
  | { Temp:   number }
  | { Name:   string }
  | { IntLit: number }
  | { RealLit: number }
  | { Label:  string };

export interface TacInstr {
  op:     TacOp;
  arg1:   TacArg | null;
  arg2:   TacArg | null;
  result: TacArg | null;
}

// ── Stack VM types ─────────────────────────────────────────────────────────────

export type VmValue = { Int: number } | { Real: number } | { Bool: boolean };

export type VmInstr =
  | { Push: VmValue }
  | "Pop" | "Dup"
  | "Add" | "Sub" | "Mul" | "Div" | "Mod" | "Neg"
  | "CmpEq" | "CmpNe" | "CmpLt" | "CmpLe" | "CmpGt" | "CmpGe"
  | "Not" | "And" | "Or"
  | { Load: string } | { Store: string }
  | "LoadIdx" | "StoreIdx"
  | { Jmp: number } | { JmpFalse: number }
  | { Call: string } | "Ret" | { EnterFrame: number } | "ExitFrame"
  | "Read" | "Write" | "Halt";

// ── AST types ──────────────────────────────────────────────────────────────────

export interface Span {
  line:   number;
  column: number;
  length: number;
}

export interface AstNode {
  kind: NodeKind;
  span: Span;
}

export type NodeKind =
  | { Program: { name: string; params: string[]; declarations: AstNode; subprograms: AstNode; body: AstNode } }
  | { Declarations: { items: AstNode[] } }
  | { VarDecl: { names: string[]; ty: AstNode } }
  | "TypeInteger"
  | "TypeReal"
  | { TypeArray: { low: number; high: number; element: AstNode } }
  | { SubprogramDeclarations: { items: AstNode[] } }
  | { FunctionDecl: { name: string; params: AstNode[]; return_type: AstNode; declarations: AstNode; body: AstNode } }
  | { ProcedureDecl: { name: string; params: AstNode[]; declarations: AstNode; body: AstNode } }
  | { ParamGroup: { names: string[]; ty: AstNode } }
  | { CompoundStatement: { stmts: AstNode[] } }
  | { Assignment: { target: AstNode; value: AstNode } }
  | { ProcedureCall: { name: string; args: AstNode[] } }
  | { IfStatement: { cond: AstNode; then_branch: AstNode; else_branch: AstNode | null } }
  | { WhileStatement: { cond: AstNode; body: AstNode } }
  | { BinaryExpr: { op: string; left: AstNode; right: AstNode } }
  | { UnaryExpr: { op: string; operand: AstNode } }
  | { Variable: { name: string; index: AstNode | null } }
  | { FunctionCall: { name: string; args: AstNode[] } }
  | { IntLiteral: { value: number } }
  | { RealLiteral: { value: number } };

// ── Stage output types ─────────────────────────────────────────────────────────

export interface LexerOutput {
  tokens: Token[];
  errors: CompilerError[];
}

export interface RdParseOutput {
  ast:    AstNode | null;
  errors: CompilerError[];
  trace:  string[];
}

export interface Ll1ParseOutput {
  trace:    string[];
  errors:   CompilerError[];
  accepted: boolean;
}

export interface LrParseOutput {
  accepted:     boolean;
  trace:        { step: number; stack: string; input: string; action: string }[];
  errors:       CompilerError[];
  action_table: [string, string][][];
  goto_table:   [string, number][][];
}

export interface SemanticOutput {
  typed_ast:       AstNode | null;
  symbol_snapshot: SymbolEntry[];
  errors:          CompilerError[];
}

export interface IrOutput {
  instructions: TacInstr[];
  errors:       CompilerError[];
}

export interface CodegenOutput {
  bytecode:     VmInstr[];
  function_map: Record<string, number>;
  errors:       CompilerError[];
}

export interface VmOutput {
  stdout:     string[];
  errors:     CompilerError[];
  halted:     boolean;
  step_count: number;
}

// ── Phase 18 — Report Tables ───────────────────────────────────────────────────

export interface FirstFollowRow {
  non_terminal: string;
  first:        string[];
  follow:       string[];
}
export interface FirstFollowOutput { rows: FirstFollowRow[] }

export interface Ll1TableRow {
  non_terminal: string;
  cells:        Record<string, string>;
}
export interface Ll1TableOutput {
  terminals: string[];
  rows:      Ll1TableRow[];
}

export interface LrTableOutput {
  terminals:     string[];
  non_terminals: string[];
  action_rows:   Record<string, string>[];
  goto_rows:     Record<string, number>[];
}

# Mini Pascal Subset Compiler

**CS-471L Compiler Construction Lab | Spring 2026 | UET Lahore**

A fully interactive, browser-based compiler for the Pascal subset defined in Aho, Lam, Sethi & Ullman (Appendix A). The compiler core is written in **Rust** and compiled to **WebAssembly**; the frontend is **Vite + React + TypeScript**.

---

## Prerequisites

| Tool | Version |
|------|---------|
| Rust | 1.75+ |
| wasm-pack | 0.15+ |
| Node.js | 20+ |
| npm | 10+ |

Install Rust: https://rustup.rs  
Install wasm-pack: `cargo install wasm-pack`

---

## Build

```bash
# Build WASM + frontend (production bundle)
make all

# OR step by step:
make build-wasm        # Rust → WASM (output: frontend/src/wasm/)
make build-frontend    # npm install + vite build (output: frontend/dist/)
```

---

## Development Server

```bash
make dev
# Opens at http://localhost:5173
```

The dev server uses Vite's HMR. After editing Rust source, re-run `make build-wasm` to rebuild the WASM module, then the browser refreshes automatically.

---

## Run Tests

```bash
make test
# Runs all 29 integration tests + unit tests via cargo test
```

---

## Clean

```bash
make clean
# Removes: compiler-core/target/, frontend/dist/, frontend/src/wasm/
```

---

## Sample Programs

Sample `.pas` files are in `test/`:

| File | Description |
|------|-------------|
| `gcd.pas` | Greatest common divisor (Aho App. A) |
| `arraysum.pas` | Array sum with read loop |
| `factorial.pas` | Recursive factorial |
| `error_duplicate.pas` | Duplicate variable declaration (semantic error) |
| `error_undeclared.pas` | Use of undeclared variable (semantic error) |
| `error_type_mismatch.pas` | Type mismatch assignment (semantic error) |

---

## Project Structure

```
rust-compiler/
├── compiler-core/          Rust compiler (→ WASM)
│   ├── src/
│   │   ├── lib.rs          WASM entry points (wasm-bindgen exports)
│   │   ├── types.rs        Shared types (Token, AstNode, CompilerError, …)
│   │   ├── buffer.rs       Double-buffering character input
│   │   ├── lexer.rs        Lexical analyzer (logos crate)
│   │   ├── ast.rs          AST node definitions
│   │   ├── rd_parser/      Recursive descent parser (P1–P55)
│   │   ├── ll1_table.rs    FIRST/FOLLOW + LL(1) table
│   │   ├── ll1_parser.rs   Non-recursive predictive parser
│   │   ├── lr_items.rs     LR(0) item sets + LALR(1) states
│   │   ├── lr_table.rs     LALR(1) action/goto table
│   │   ├── lr_parser.rs    LR shift-reduce parser
│   │   ├── symbol_table.rs Hash-based symbol table (djb2, size=211)
│   │   ├── semantic.rs     Semantic analysis (type checking)
│   │   ├── ir.rs           Three-address code (TAC) generation
│   │   ├── codegen.rs      Stack VM bytecode generation
│   │   └── vm.rs           Stack VM interpreter
│   └── tests/
│       └── integration_tests.rs  Full pipeline integration tests
├── frontend/               React + TypeScript UI
│   └── src/
│       ├── App.tsx         Root layout, Monaco editor, toolbar
│       ├── components/
│       │   ├── TokenPanel.tsx    Token stream chips
│       │   ├── ErrorPanel.tsx    Error list with jump-to-source
│       │   ├── AstView.tsx       D3.js interactive AST tree
│       │   ├── SymbolTable.tsx   Symbol table panel
│       │   ├── StepMode.tsx      Step-by-step demo mode
│       │   └── ReportTables.tsx  FIRST/FOLLOW, LL(1), LR tables
│       ├── hooks/
│       │   ├── useCompiler.ts    Debounced compiler pipeline hook
│       │   └── useWasm.ts        WASM module loader
│       └── types.ts              TypeScript type definitions
├── docs/
│   ├── grammar.md          Full grammar (original + LL(1) transformed)
│   └── report.md           Project report
├── test/                   Sample .pas programs
├── output/                 Sample compiler output (JSON)
├── Makefile
└── README.md
```

---

## Implemented Compiler Stages

| Stage | Algorithm | Notes |
|-------|-----------|-------|
| Lexer | logos crate DFA | Double-buffering, line/col tracking |
| RD Parser | Recursive descent | 55 productions, panic-mode recovery |
| LL(1) Parser | Table-driven predictive | Explicit stack, FIRST/FOLLOW |
| LR Parser | LALR(1) shift-reduce | Full action/goto table |
| Symbol Table | Hash table (djb2) | Nested scopes, size=211 |
| Semantic | Type checker | Undeclared vars, type mismatches, duplicate decls |
| TAC / IR | Three-address code | Quadruples: (op, arg1, arg2, result) |
| Code Gen | Stack VM bytecode | Linear translation of TAC |
| VM | Stack interpreter | read/write, conditionals, while loops, calls |

---

## Browser UI Features

- Live Monaco editor with Pascal syntax highlighting
- Token stream chips (color-coded by category)
- Interactive D3.js AST tree (click node → jump to source)
- Step-by-step demo mode (8 stages)
- Error highlighting (red squiggles + gutter markers)
- Symbol table panel (live, scope-colored)
- FIRST/FOLLOW, LL(1), and LR table reports
- Dark/light theme toggle
- Mobile-responsive layout

---

## References

- Aho, Lam, Sethi & Ullman. *Compilers: Principles, Techniques & Tools* (2nd ed.). Appendix A.
- CS-471L Lab Manuals, UET Lahore, Spring 2026.

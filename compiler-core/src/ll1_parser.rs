// Non-recursive LL(1) predictive parser — explicit stack, while loop, zero recursion.
use crate::first_follow::{symbol_display, Grammar, GrammarSymbol, NonTerminal};
use crate::lexer;
use crate::ll1_table::{build_ll1_table, terminals_match};
use crate::types::{AddopKind, CompilerError, MulopKind, TokenKind};
use serde::Serialize;

#[derive(Serialize)]
pub struct Ll1TraceStep {
    pub stack: Vec<String>, // stack contents, top first
    pub input: Vec<String>, // remaining tokens, current first
    pub action: String,
}

#[derive(Serialize)]
pub struct Ll1Output {
    pub accepted: bool,
    pub trace: Vec<Ll1TraceStep>,
    pub errors: Vec<CompilerError>,
}

pub fn parse(source: &str) -> Ll1Output {
    let lex_out = lexer::tokenize(source);
    let mut errors = lex_out.errors;
    let tokens = lex_out.tokens;

    let grammar = Grammar::pascal_subset();
    let table = build_ll1_table(&grammar);

    // Stack: index 0 = bottom ($), last = top (Program)
    let mut stack: Vec<GrammarSymbol> = vec![
        GrammarSymbol::Eof,
        GrammarSymbol::NonTerminal(NonTerminal::Program),
    ];
    let mut pos = 0usize;
    let mut trace: Vec<Ll1TraceStep> = vec![];

    loop {
        let current = &tokens[pos.min(tokens.len() - 1)];
        let top = match stack.last() {
            Some(s) => s.clone(),
            None => break,
        };

        // Accept: stack drained to $ and input is also $
        if top == GrammarSymbol::Eof {
            if current.kind == TokenKind::Eof {
                record(&stack, &tokens[pos..], "Accept", &mut trace);
            } else {
                let msg = format!("unexpected '{}' after end of program", current.lexeme);
                record(
                    &stack,
                    &tokens[pos..],
                    &format!("Error: {}", msg),
                    &mut trace,
                );
                push_error(&mut errors, &msg, current);
            }
            break;
        }

        match top {
            GrammarSymbol::Epsilon => {
                stack.pop();
            }
            GrammarSymbol::Terminal(ref term) => {
                if terminals_match(term, &current.kind) {
                    record(
                        &stack,
                        &tokens[pos..],
                        &format!("Match '{}'", current.lexeme),
                        &mut trace,
                    );
                    stack.pop();
                    pos = (pos + 1).min(tokens.len() - 1);
                } else {
                    let msg = format!("expected '{}', found '{}'", sym_name(term), current.lexeme);
                    record(
                        &stack,
                        &tokens[pos..],
                        &format!("Error: {}", msg),
                        &mut trace,
                    );
                    push_error(&mut errors, &msg, current);
                    stack.pop(); // skip the expected terminal; keep input position
                }
            }
            GrammarSymbol::NonTerminal(ref nt) => {
                let lookup = normalize(&current.kind);
                if let Some(&prod_idx) = table.get(&(nt.clone(), lookup)) {
                    let rhs = &grammar.productions[prod_idx].1;
                    record(
                        &stack,
                        &tokens[pos..],
                        &format!("Predict {}", fmt_prod(nt, rhs)),
                        &mut trace,
                    );
                    stack.pop();
                    for sym in rhs.iter().rev() {
                        if *sym != GrammarSymbol::Epsilon {
                            stack.push(sym.clone());
                        }
                    }
                } else {
                    let msg = format!(
                        "unexpected '{}' while expanding {}",
                        current.lexeme,
                        nt.display_name()
                    );
                    record(
                        &stack,
                        &tokens[pos..],
                        &format!("Error: {}", msg),
                        &mut trace,
                    );
                    push_error(&mut errors, &msg, current);
                    stack.pop(); // discard the non-terminal
                    if pos + 1 < tokens.len() {
                        pos += 1; // skip the bad token
                    } else {
                        break;
                    }
                }
            }
            GrammarSymbol::Eof => unreachable!(),
        }
    }

    Ll1Output {
        accepted: errors.is_empty(),
        trace,
        errors,
    }
}

// Normalize keyword operator tokens to the grammar's representative token for table lookup.
// The grammar uses Addop(Plus)/Mulop(Star) as class representatives; or/div/mod/and are keywords.
fn normalize(tok: &TokenKind) -> TokenKind {
    match tok {
        TokenKind::Or => TokenKind::Addop(AddopKind::Plus),
        TokenKind::Div | TokenKind::Mod | TokenKind::And => TokenKind::Mulop(MulopKind::Star),
        other => other.clone(),
    }
}

fn record(
    stack: &[GrammarSymbol],
    remaining: &[crate::types::Token],
    action: &str,
    trace: &mut Vec<Ll1TraceStep>,
) {
    trace.push(Ll1TraceStep {
        stack: stack.iter().rev().map(|s| symbol_display(s)).collect(),
        input: remaining.iter().map(|t| t.lexeme.clone()).collect(),
        action: action.to_owned(),
    });
}

fn fmt_prod(nt: &NonTerminal, rhs: &[GrammarSymbol]) -> String {
    let rhs_str: Vec<String> = rhs.iter().map(symbol_display).collect();
    format!("{} → {}", nt.display_name(), rhs_str.join(" "))
}

fn sym_name(tok: &TokenKind) -> String {
    symbol_display(&GrammarSymbol::Terminal(tok.clone()))
}

fn push_error(errors: &mut Vec<CompilerError>, msg: &str, tok: &crate::types::Token) {
    errors.push(CompilerError {
        stage: "ll1".into(),
        message: msg.to_owned(),
        line: tok.line,
        column: tok.column,
        length: tok.lexeme.len(),
        severity: "error".into(),
    });
}

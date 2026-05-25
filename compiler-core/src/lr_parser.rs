// LALR(1) shift-reduce parser driver — explicit state stack, while loop, zero recursion.
use crate::first_follow::{symbol_display, Grammar};
use crate::lexer;
use crate::lr_table::{build_lalr1_table, lookup_action, LrAction};
use crate::types::CompilerError;
use serde::Serialize;

#[derive(Serialize)]
pub struct LrTraceStep {
    pub state_stack: Vec<usize>,
    pub symbol_stack: Vec<String>,
    pub input: Vec<String>,
    pub action: String,
}

#[derive(Serialize)]
pub struct LrOutput {
    pub accepted: bool,
    pub trace: Vec<LrTraceStep>,
    pub errors: Vec<CompilerError>,
    pub action_table: Vec<Vec<(String, String)>>, // serialized for project report
    pub goto_table: Vec<Vec<(String, usize)>>,
}

pub fn parse(source: &str) -> LrOutput {
    let lex_out = lexer::tokenize(source);
    let mut errors = lex_out.errors;
    let tokens = lex_out.tokens;

    let grammar = Grammar::pascal_subset();
    let table = build_lalr1_table(&grammar);

    // Serialize action and goto tables for the project report.
    let action_table: Vec<Vec<(String, String)>> = table
        .action
        .iter()
        .map(|row| {
            let mut entries: Vec<(String, String)> = row
                .iter()
                .map(|(tok, act)| {
                    let tok_str = format!("{:?}", tok);
                    let act_str = match act {
                        LrAction::Shift(n) => format!("s{}", n),
                        LrAction::Reduce(n) => format!("r{}", n),
                        LrAction::Accept => "acc".to_string(),
                    };
                    (tok_str, act_str)
                })
                .collect();
            entries.sort_by(|a, b| a.0.cmp(&b.0));
            entries
        })
        .collect();

    let goto_table: Vec<Vec<(String, usize)>> = table
        .goto_map
        .iter()
        .map(|row| {
            let mut entries: Vec<(String, usize)> = row
                .iter()
                .map(|(nt, &next)| (nt.display_name().to_owned(), next))
                .collect();
            entries.sort_by(|a, b| a.0.cmp(&b.0));
            entries
        })
        .collect();

    let mut state_stack: Vec<usize> = vec![0];
    let mut sym_stack: Vec<String> = vec![];
    let mut pos = 0usize;
    let mut trace: Vec<LrTraceStep> = vec![];

    loop {
        let state = *state_stack.last().unwrap();
        let current = &tokens[pos.min(tokens.len() - 1)];

        match lookup_action(&table.action[state], &current.kind) {
            Some(LrAction::Shift(next)) => {
                let next = *next;
                record_step(
                    &state_stack,
                    &sym_stack,
                    &tokens[pos..],
                    &format!("Shift '{}'", current.lexeme),
                    &mut trace,
                );
                sym_stack.push(current.lexeme.clone());
                state_stack.push(next);
                pos = (pos + 1).min(tokens.len() - 1);
            }
            Some(LrAction::Reduce(prod_idx)) => {
                let prod_idx = *prod_idx;
                let prod = &table.prods[prod_idx];
                let rhs_len = prod.rhs.len();
                let lhs_str = match &prod.lhs {
                    Some(nt) => nt.display_name().to_owned(),
                    None => "S'".to_owned(),
                };
                let rhs_str: Vec<String> = prod.rhs.iter().map(symbol_display).collect();
                let action_str = format!("Reduce {} → {}", lhs_str, rhs_str.join(" "));
                record_step(
                    &state_stack,
                    &sym_stack,
                    &tokens[pos..],
                    &action_str,
                    &mut trace,
                );

                for _ in 0..rhs_len {
                    sym_stack.pop();
                    state_stack.pop();
                }

                if let Some(nt) = &prod.lhs.clone() {
                    let top = *state_stack.last().unwrap();
                    if let Some(&next) = table.goto_map[top].get(nt) {
                        sym_stack.push(lhs_str);
                        state_stack.push(next);
                    } else {
                        let msg =
                            format!("missing goto for {} in state {}", nt.display_name(), top);
                        push_error(&mut errors, &msg, current);
                        break;
                    }
                }
            }
            Some(LrAction::Accept) => {
                record_step(
                    &state_stack,
                    &sym_stack,
                    &tokens[pos..],
                    "Accept",
                    &mut trace,
                );
                break;
            }
            None => {
                let msg = format!("unexpected '{}' in state {}", current.lexeme, state);
                record_step(
                    &state_stack,
                    &sym_stack,
                    &tokens[pos..],
                    &format!("Error: {}", msg),
                    &mut trace,
                );
                push_error(&mut errors, &msg, current);
                if pos + 1 < tokens.len() {
                    pos += 1;
                } else {
                    break;
                }
            }
        }

        if trace.len() > 10_000 {
            break;
        }
    }

    LrOutput {
        accepted: errors.is_empty(),
        trace,
        errors,
        action_table,
        goto_table,
    }
}

fn record_step(
    states: &[usize],
    syms: &[String],
    remaining: &[crate::types::Token],
    action: &str,
    trace: &mut Vec<LrTraceStep>,
) {
    trace.push(LrTraceStep {
        state_stack: states.to_vec(),
        symbol_stack: syms.to_vec(),
        input: remaining.iter().map(|t| t.lexeme.clone()).collect(),
        action: action.to_owned(),
    });
}

fn push_error(errors: &mut Vec<CompilerError>, msg: &str, tok: &crate::types::Token) {
    errors.push(CompilerError {
        stage: "lr".into(),
        message: msg.to_owned(),
        line: tok.line,
        column: tok.column,
        length: tok.lexeme.len(),
        severity: "error".into(),
    });
}

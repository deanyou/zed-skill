use dashmap::DashMap;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_lsp::lsp_types::*;

use crate::{api, Document, SymbolInfo};

const SKILL_DOCS: &[(&str, &str)] = &[
    ("defun", "**defun** - Define a new function\n\n```skill\n(defun name (params...) body...)\n```\n\nCreates a new function with the given name, parameters, and body."),
    ("procedure", "**procedure** - Define a procedure (alias for defun)\n\n```skill\n(procedure name (params...) body...)\n```\n\nSame as defun, creates a new function."),
    ("let", "**let** - Create local variable bindings\n\n```skill\n(let ((var1 val1) (var2 val2)) body...)\n```\n\nCreates local variables with the given bindings."),
    ("letrec", "**letrec** - Create recursive local bindings\n\n```skill\n(letrec ((var1 val1) (var2 val2)) body...)\n```\n\nLike let, but bindings can reference each other."),
    ("letseq", "**letseq** - Create sequential local bindings\n\n```skill\n(letseq ((var1 val1) (var2 val2)) body...)\n```\n\nLike let, but bindings are evaluated sequentially."),
    ("setq", "**setq** - Set a variable's value\n\n```skill\n(setq var value)\n```\n\nSets the variable to the given value."),
    ("if", "**if** - Conditional expression\n\n```skill\n(if test then else)\n```\n\nEvaluates test, then evaluates then or else based on the result."),
    ("cond", "**cond** - Multi-branch conditional\n\n```skill\n(cond (test1 body1...) (test2 body2...) ...)\n```\n\nEvaluates each test in order and executes the body of the first true test."),
    ("case", "**case** - Case dispatch\n\n```skill\n(case key (value1 body1...) (value2 body2...) ...)\n```\n\nMatches key against values and executes the corresponding body."),
    ("when", "**when** - Conditional execution\n\n```skill\n(when test body...)\n```\n\nExecutes body only when test is true."),
    ("unless", "**unless** - Negative conditional execution\n\n```skill\n(unless test body...)\n```\n\nExecutes body only when test is false."),
    ("progn", "**progn** - Sequential execution\n\n```skill\n(progn body...)\n```\n\nExecutes forms sequentially and returns the last result."),
    ("prog", "**prog** - Program with local variables\n\n```skill\n(prog (vars...) body...)\n```\n\nExecutes body with local variables."),
    ("foreach", "**foreach** - Iterate over list\n\n```skill\n(foreach var list body...)\n```\n\nIterates over list, binding var to each element."),
    ("for", "**for** - Counting loop\n\n```skill\n(for var start end body...)\n```\n\nIterates var from start to end."),
    ("while", "**while** - While loop\n\n```skill\n(while test body...)\n```\n\nExecutes body while test is true."),
    ("return", "**return** - Return from function\n\n```skill\n(return value)\n```\n\nReturns value from the current function."),
    ("error", "**error** - Signal an error\n\n```skill\n(error message)\n```\n\nSignals an error with the given message."),
    ("printf", "**printf** - Formatted output\n\n```skill\n(printf format ...)\n```\n\nPrints formatted output to stdout."),
    ("sprintf", "**sprintf** - Formatted string output\n\n```skill\n(sprintf format ...)\n```\n\nReturns a formatted string."),
    ("car", "**car** - First element of list\n\n```skill\n(car list)\n```\n\nReturns the first element of a list."),
    ("cdr", "**cdr** - Rest of list\n\n```skill\n(cdr list)\n```\n\nReturns the list without its first element."),
    ("cons", "**cons** - Construct list\n\n```skill\n(cons x list)\n```\n\nPrepends x to list and returns the new list."),
    ("list", "**list** - Create list\n\n```skill\n(list ...)\n```\n\nCreates a list of the given arguments."),
    ("append", "**append** - Append lists\n\n```skill\n(append list1 list2)\n```\n\nAppends list2 to list1."),
    ("reverse", "**reverse** - Reverse list\n\n```skill\n(reverse list)\n```\n\nReturns the reversed list."),
    ("length", "**length** - List length\n\n```skill\n(length list)\n```\n\nReturns the length of a list."),
    ("nth", "**nth** - Nth element\n\n```skill\n(nth n list)\n```\n\nReturns the nth element of a list."),
    ("member", "**member** - List membership\n\n```skill\n(member item list)\n```\n\nChecks if item is in list."),
    ("assoc", "**assoc** - Association list lookup\n\n```skill\n(assoc key alist)\n```\n\nLooks up key in association list."),
    ("sort", "**sort** - Sort list\n\n```skill\n(sort list predicate)\n```\n\nSorts list using the given predicate."),
    ("mapcar", "**mapcar** - Map over list\n\n```skill\n(mapcar function list)\n```\n\nApplies function to each element of list."),
    ("apply", "**apply** - Apply function\n\n```skill\n(apply function args)\n```\n\nApplies function to args."),
    ("funcall", "**funcall** - Call function\n\n```skill\n(funcall function ...)\n```\n\nCalls function with arguments."),
    ("eval", "**eval** - Evaluate expression\n\n```skill\n(eval expr)\n```\n\nEvaluates an expression."),
    ("lambda", "**lambda** - Anonymous function\n\n```skill\n(lambda (args...) body...)\n```\n\nCreates an anonymous function."),
    ("quote", "**quote** - Quote expression\n\n```skill\n(quote expr)\n```\n\nReturns expr without evaluating it."),
    ("open", "**open** - Open file\n\n```skill\n(open filename mode)\n```\n\nOpens a file and returns a port."),
    ("close", "**close** - Close port\n\n```skill\n(close port)\n```\n\nCloses a port."),
    ("read", "**read** - Read expression\n\n```skill\n(read)\n```\n\nReads an expression from input."),
    ("print", "**print** - Print expression\n\n```skill\n(print expr)\n```\n\nPrints an expression."),
    ("type", "**type** - Get type\n\n```skill\n(type expr)\n```\n\nReturns the type of an expression."),
    ("boundp", "**boundp** - Check if bound\n\n```skill\n(boundp symbol)\n```\n\nChecks if symbol is bound."),
    ("null", "**null** - Check if null\n\n```skill\n(null expr)\n```\n\nChecks if expr is null."),
    ("eq", "**eq** - Identity equality\n\n```skill\n(eq a b)\n```\n\nTests for identity equality."),
    ("eql", "**eql** - Value equality\n\n```skill\n(eql a b)\n```\n\nTests for value equality."),
    ("equal", "**equal** - Structural equality\n\n```skill\n(equal a b)\n```\n\nTests for structural equality."),
    ("stringp", "**stringp** - String predicate\n\n```skill\n(stringp expr)\n```\n\nChecks if expr is a string."),
    ("fixp", "**fixp** - Integer predicate\n\n```skill\n(fixp expr)\n```\n\nChecks if expr is an integer."),
    ("floatp", "**floatp** - Float predicate\n\n```skill\n(floatp expr)\n```\n\nChecks if expr is a float."),
    ("listp", "**listp** - List predicate\n\n```skill\n(listp expr)\n```\n\nChecks if expr is a list."),
];

pub async fn get_hover(
    documents: &Arc<DashMap<Url, RwLock<Document>>>,
    symbol_table: &Arc<RwLock<HashMap<String, SymbolInfo>>>,
    uri: &Url,
    position: Position,
) -> Option<Hover> {
    let doc = documents.get(uri)?;
    let doc = doc.read().await;
    let word = get_word_at_position(&doc.rope, position)?;

    for (name, doc) in SKILL_DOCS {
        if *name == word {
            return Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: doc.to_string(),
                }),
                range: None,
            });
        }
    }

    // Official API reference (generated from Cadence IC23.1 .fnd docs)
    if let Some(f) = api::index().get(&word) {
        let mut content = format!(
            "**{}**\n\n*{} / {}*\n\n```skill\n{}\n```",
            f.name, f.category, f.module, f.signature
        );
        if !f.description.is_empty() {
            content.push_str(&format!("\n\n{}", f.description));
        }
        return Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: content,
            }),
            range: None,
        });
    }

    let symbols = symbol_table.read().await;
    if let Some(info) = symbols.get(&word) {
        let mut content = format!("**{}**", info.name);

        if let Some(params) = &info.parameters {
            content.push_str(&format!("\n\n```skill\n({} {})\n```", info.name, params.join(" ")));
        }

        if let Some(doc) = &info.documentation {
            content.push_str(&format!("\n\n{}", doc));
        }

        if let Some(return_type) = &info.return_type {
            content.push_str(&format!("\n\n**Returns:** {}", return_type));
        }

        return Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: content,
            }),
            range: None,
        });
    }

    None
}

fn get_word_at_position(rope: &ropey::Rope, position: Position) -> Option<String> {
    let line = rope.line(position.line as usize);
    let char_pos = std::cmp::min(position.character as usize, line.len_chars());

    let mut start = char_pos;
    let mut end = char_pos;

    while start > 0 {
        let c = line.char(start - 1);
        if c.is_alphanumeric() || c == '-' || c == '_' || c == '?' || c == '!' {
            start -= 1;
        } else {
            break;
        }
    }

    while end < line.len_chars() {
        let c = line.char(end);
        if c.is_alphanumeric() || c == '-' || c == '_' || c == '?' || c == '!' {
            end += 1;
        } else {
            break;
        }
    }

    if start < end {
        Some(line.slice(start..end).to_string())
    } else {
        None
    }
}

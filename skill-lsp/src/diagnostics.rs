use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_lsp::lsp_types::*;

use crate::Document;

pub async fn get_diagnostics(
    documents: &Arc<DashMap<Url, RwLock<Document>>>,
    uri: &Url,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    if let Some(doc) = documents.get(uri) {
        let doc = doc.read().await;
        let content = doc.rope.to_string();

        check_unmatched_quotes(&content, &mut diagnostics);
        check_unbalanced_parens(&content, &mut diagnostics);
        check_common_mistakes(&content, &mut diagnostics);
    }

    diagnostics
}

/// Char index of the char at `byte` offset within `line`.
fn char_col(line: &str, byte: usize) -> usize {
    line[..byte].chars().count()
}

fn is_escaped(line: &str, byte: usize) -> bool {
    // Count consecutive backslashes immediately before `byte`; odd means escaped.
    let bytes = line.as_bytes();
    let mut n = 0;
    let mut i = byte;
    while i > 0 && bytes[i - 1] == b'\\' {
        n += 1;
        i -= 1;
    }
    n % 2 == 1
}

/// Byte offset of the `;` starting a line comment outside strings, if any.
fn line_comment_start(line: &str) -> Option<usize> {
    let mut in_string = false;
    for (i, c) in line.char_indices() {
        match c {
            '"' if !is_escaped(line, i) => in_string = !in_string,
            ';' if !in_string => return Some(i),
            _ => {}
        }
    }
    None
}

fn check_unmatched_quotes(content: &str, diagnostics: &mut Vec<Diagnostic>) {
    for (line_num, line) in content.lines().enumerate() {
        let code_end = line_comment_start(line).unwrap_or(line.len());
        let code = &line[..code_end];

        let mut in_string = false;
        let mut last_quote = None;
        for (i, c) in code.char_indices() {
            if c == '"' && !is_escaped(code, i) {
                in_string = !in_string;
                last_quote = Some(i);
            }
        }
        // SKILL strings cannot span lines: an odd quote is always an error.
        if in_string {
            let Some(q) = last_quote else { continue };
            diagnostics.push(Diagnostic {
                range: Range {
                    start: Position {
                        line: line_num as u32,
                        character: char_col(line, q) as u32,
                    },
                    end: Position {
                        line: line_num as u32,
                        character: char_col(line, q) as u32 + 1,
                    },
                },
                severity: Some(DiagnosticSeverity::ERROR),
                source: Some("skill-lsp".to_string()),
                message: "Unmatched string quote".to_string(),
                ..Default::default()
            });
        }
    }
}

fn check_unbalanced_parens(content: &str, diagnostics: &mut Vec<Diagnostic>) {
    let mut stack: Vec<(usize, usize)> = Vec::new(); // (line, char col)
    let mut in_block_comment = false;

    for (line_num, line) in content.lines().enumerate() {
        let mut in_string = false; // strings cannot span lines in SKILL
        let mut chars = line.char_indices().peekable();

        while let Some((i, c)) = chars.next() {
            if in_block_comment {
                if c == '*' && chars.peek().map(|&(_, nc)| nc == '/') == Some(true) {
                    chars.next(); // consume '/'
                    in_block_comment = false;
                }
                continue;
            }
            if in_string {
                if c == '"' && !is_escaped(line, i) {
                    in_string = false;
                }
                continue;
            }
            match c {
                ';' => break, // line comment
                '"' => in_string = true,
                '/' if chars.peek().map(|&(_, nc)| nc == '*') == Some(true) => {
                    chars.next(); // consume '*'
                    in_block_comment = true;
                }
                '(' => stack.push((line_num, char_col(line, i))),
                ')' if stack.pop().is_none() => {
                    let col = char_col(line, i);
                    diagnostics.push(Diagnostic {
                        range: Range {
                            start: Position {
                                line: line_num as u32,
                                character: col as u32,
                            },
                            end: Position {
                                line: line_num as u32,
                                character: col as u32 + 1,
                            },
                        },
                        severity: Some(DiagnosticSeverity::ERROR),
                        source: Some("skill-lsp".to_string()),
                        message: "Unexpected closing parenthesis".to_string(),
                        ..Default::default()
                    });
                }
                _ => {}
            }
        }
    }

    for (line, col) in stack {
        diagnostics.push(Diagnostic {
            range: Range {
                start: Position {
                    line: line as u32,
                    character: col as u32,
                },
                end: Position {
                    line: line as u32,
                    character: col as u32 + 1,
                },
            },
            severity: Some(DiagnosticSeverity::ERROR),
            source: Some("skill-lsp".to_string()),
            message: "Unclosed parenthesis".to_string(),
            ..Default::default()
        });
    }
}

fn check_common_mistakes(content: &str, diagnostics: &mut Vec<Diagnostic>) {
    let misspellings = [("deffun", "defun")];
    let deprecated = [("stringLength", "strlen")];

    for (line_num, line) in content.lines().enumerate() {
        let code_end = line_comment_start(line).unwrap_or(line.len());
        let code = &line[..code_end];

        for (wrong, correct) in misspellings.iter() {
            for (byte, _) in code.match_indices(wrong) {
                let before_ok = byte == 0
                    || !code[..byte]
                        .chars()
                        .last()
                        .is_some_and(|c| c.is_alphanumeric() || c == '-' || c == '_');
                let end = byte + wrong.len();
                let after_ok = code[end..]
                    .chars()
                    .next()
                    .is_none_or(|c| !(c.is_alphanumeric() || c == '-' || c == '_'));
                if before_ok && after_ok {
                    let col = char_col(line, byte);
                    diagnostics.push(Diagnostic {
                        range: Range {
                            start: Position {
                                line: line_num as u32,
                                character: col as u32,
                            },
                            end: Position {
                                line: line_num as u32,
                                character: col as u32 + wrong.chars().count() as u32,
                            },
                        },
                        severity: Some(DiagnosticSeverity::WARNING),
                        source: Some("skill-lsp".to_string()),
                        message: format!("Did you mean '{}'?", correct),
                        ..Default::default()
                    });
                }
            }
        }

        for (old, new) in deprecated.iter() {
            if let Some(byte) = code.find(old) {
                let col = char_col(line, byte);
                diagnostics.push(Diagnostic {
                    range: Range {
                        start: Position {
                            line: line_num as u32,
                            character: col as u32,
                        },
                        end: Position {
                            line: line_num as u32,
                            character: col as u32 + old.chars().count() as u32,
                        },
                    },
                    severity: Some(DiagnosticSeverity::HINT),
                    source: Some("skill-lsp".to_string()),
                    message: format!("Consider using '{}' instead", new),
                    ..Default::default()
                });
            }
        }
    }
}

pub async fn format_document(
    documents: &Arc<DashMap<Url, RwLock<Document>>>,
    uri: &Url,
) -> Vec<TextEdit> {
    let mut edits = Vec::new();

    if let Some(doc) = documents.get(uri) {
        let doc = doc.read().await;
        let content = doc.rope.to_string();

        let formatted = format_skill_code(&content);

        if formatted != content {
            edits.push(TextEdit {
                range: Range {
                    start: Position {
                        line: 0,
                        character: 0,
                    },
                    end: Position {
                        line: content.lines().count() as u32,
                        character: 0,
                    },
                },
                new_text: formatted,
            });
        }
    }

    edits
}

/// Count parens outside strings and comments on a single line.
fn count_parens_outside_strings(line: &str) -> (usize, usize) {
    let code_end = line_comment_start(line).unwrap_or(line.len());
    let code = &line[..code_end];
    let mut open = 0;
    let mut close = 0;
    let mut in_string = false;
    let mut in_block = false;
    let chars: Vec<(usize, char)> = code.char_indices().collect();
    let mut idx = 0;
    while idx < chars.len() {
        let (i, c) = chars[idx];
        if in_block {
            if c == '*' && chars.get(idx + 1).map(|&(_, nc)| nc) == Some('/') {
                in_block = false;
                idx += 1;
            }
            idx += 1;
            continue;
        }
        if in_string {
            if c == '"' && !is_escaped(code, i) {
                in_string = false;
            }
            idx += 1;
            continue;
        }
        match c {
            '"' => in_string = true,
            '/' if chars.get(idx + 1).map(|&(_, nc)| nc) == Some('*') => {
                in_block = true;
                idx += 1;
            }
            '(' => open += 1,
            ')' => close += 1,
            _ => {}
        }
        idx += 1;
    }
    (open, close)
}

fn format_skill_code(content: &str) -> String {
    let mut result = String::new();
    let mut indent_level: i32 = 0;
    let indent_size = 4;

    for line in content.lines() {
        let trimmed = line.trim();
        let (open_count, close_count) = count_parens_outside_strings(trimmed);

        if close_count > open_count {
            indent_level = indent_level.saturating_sub(1);
        }

        if !trimmed.is_empty() {
            let indent = " ".repeat((indent_level as usize) * indent_size);
            result.push_str(&format!("{}{}", indent, trimmed));
        }
        result.push('\n');

        if open_count > close_count {
            indent_level += 1;
        }
    }

    result
}

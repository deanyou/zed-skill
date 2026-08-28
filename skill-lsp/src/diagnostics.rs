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

        check_syntax_errors(&content, &mut diagnostics);
        check_unbalanced_parens(&content, &mut diagnostics);
        check_common_mistakes(&content, &mut diagnostics);
    }

    diagnostics
}

fn check_syntax_errors(content: &str, diagnostics: &mut Vec<Diagnostic>) {
    let lines: Vec<&str> = content.lines().collect();

    for (line_num, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        if trimmed.starts_with("(defun") || trimmed.starts_with("(procedure") {
            let re = regex::Regex::new(r"\((?:defun|procedure)\s+\(([^)]*)\)").unwrap();
            if !re.is_match(trimmed) && !trimmed.ends_with(')') {
                continue;
            }
        }

        let quote_count = trimmed.matches('"').count();
        if quote_count % 2 != 0 {
            let mut in_string = false;
            let mut last_pos = 0;
            for (pos, c) in trimmed.char_indices() {
                if c == '"' && (pos == 0 || trimmed.as_bytes()[pos - 1] != b'\\') {
                    in_string = !in_string;
                    last_pos = pos;
                }
            }
            if in_string {
                diagnostics.push(Diagnostic {
                    range: Range {
                        start: Position {
                            line: line_num as u32,
                            character: last_pos as u32,
                        },
                        end: Position {
                            line: line_num as u32,
                            character: (last_pos + 1) as u32,
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
}

fn check_unbalanced_parens(content: &str, diagnostics: &mut Vec<Diagnostic>) {
    let mut stack: Vec<(usize, usize)> = Vec::new();
    let mut in_string = false;
    let mut in_comment = false;

    for (line_num, line) in content.lines().enumerate() {
        in_comment = false;
        for (col, c) in line.char_indices() {
            if in_comment {
                break;
            }

            match c {
                ';' if !in_string => in_comment = true,
                '"' if !in_string => in_string = true,
                '"' if in_string => {
                    if col > 0 && line.as_bytes()[col - 1] != b'\\' {
                        in_string = false;
                    }
                }
                '(' if !in_string => stack.push((line_num, col)),
                ')' if !in_string => {
                    if stack.is_empty() {
                        diagnostics.push(Diagnostic {
                            range: Range {
                                start: Position {
                                    line: line_num as u32,
                                    character: col as u32,
                                },
                                end: Position {
                                    line: line_num as u32,
                                    character: (col + 1) as u32,
                                },
                            },
                            severity: Some(DiagnosticSeverity::ERROR),
                            source: Some("skill-lsp".to_string()),
                            message: "Unexpected closing parenthesis".to_string(),
                            ..Default::default()
                        });
                    } else {
                        stack.pop();
                    }
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
                    character: (col + 1) as u32,
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
    let lines: Vec<&str> = content.lines().collect();

    for (line_num, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        let misspellings = [
            ("deffun", "defun"),
            ("progn", "progn"),
            ("setf", "setq"),
            ("defmacro", "defun"),
        ];

        for (wrong, correct) in misspellings.iter() {
            if trimmed.contains(wrong) {
                if let Some(pos) = trimmed.find(wrong) {
                    diagnostics.push(Diagnostic {
                        range: Range {
                            start: Position {
                                line: line_num as u32,
                                character: pos as u32,
                            },
                            end: Position {
                                line: line_num as u32,
                                character: (pos + wrong.len()) as u32,
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

        let deprecated = [
            ("stringLength", "strlen"),
            ("stringToSymbol", "stringToSymbol"),
            ("symbolToString", "symbolToString"),
        ];

        for (old, new) in deprecated.iter() {
            if trimmed.contains(old) {
                if let Some(pos) = trimmed.find(old) {
                    diagnostics.push(Diagnostic {
                        range: Range {
                            start: Position {
                                line: line_num as u32,
                                character: pos as u32,
                            },
                            end: Position {
                                line: line_num as u32,
                                character: (pos + old.len()) as u32,
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

fn format_skill_code(content: &str) -> String {
    let mut result = String::new();
    let mut indent_level: i32 = 0;
    let indent_size = 4;

    for line in content.lines() {
        let trimmed = line.trim();

        let open_count = trimmed.matches('(').count();
        let close_count = trimmed.matches(')').count();

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

use dashmap::DashMap;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_lsp::lsp_types::*;

use crate::{Document, SymbolInfo};

#[allow(deprecated)]
pub fn extract_symbols(content: &str, uri: &Url) -> Vec<SymbolInfo> {
    let mut symbols = Vec::new();

    let func_re = regex::Regex::new(r"\((?:defun|procedure)\s+([\w?!-]+)\s*\(([^)]*)\)").unwrap();

    for (line_num, line) in content.lines().enumerate() {
        // Skip definitions inside line comments / strings
        if let Some(comment_start) = line_comment_offset(line) {
            if regex_find_name(&func_re, &line[..comment_start]).is_none() {
                continue;
            }
        }
        if let Some((name, params_str, name_start)) = regex_find_name(&func_re, line) {
            let parameters = if params_str.trim().is_empty() {
                None
            } else {
                Some(
                    params_str
                        .split_whitespace()
                        .map(|s| s.to_string())
                        .collect(),
                )
            };

            let location = Location {
                uri: uri.clone(),
                range: Range {
                    start: Position {
                        line: line_num as u32,
                        character: name_start as u32,
                    },
                    end: Position {
                        line: line_num as u32,
                        character: (name_start + name.chars().count()) as u32,
                    },
                },
            };

            let documentation = extract_preceding_comment(content, line_num as u32);

            symbols.push(SymbolInfo {
                name,
                kind: SymbolKind::FUNCTION,
                location,
                documentation,
                parameters,
                return_type: None,
            });
        }
    }

    symbols
}

/// Find `(defun|procedure NAME (PARAMS)` in `text`; returns char-based name column.
fn regex_find_name(
    re: &regex::Regex,
    text: &str,
) -> Option<(String, String, usize)> {
    let caps = re.captures(text)?;
    let m = caps.get(1)?;
    let name = m.as_str().to_string();
    let params_str = caps.get(2).map(|mm| mm.as_str()).unwrap_or("").to_string();
    let name_start = text[..m.start()].chars().count();
    Some((name, params_str, name_start))
}

/// Byte offset of a line comment start (`;` outside strings), if any.
fn line_comment_offset(line: &str) -> Option<usize> {
    let mut in_string = false;
    for (i, c) in line.char_indices() {
        match c {
            '"' if !is_escaped_byte(line, i) => in_string = !in_string,
            ';' if !in_string => return Some(i),
            _ => {}
        }
    }
    None
}

fn is_escaped_byte(line: &str, byte: usize) -> bool {
    let bytes = line.as_bytes();
    let mut n = 0;
    let mut i = byte;
    while i > 0 && bytes[i - 1] == b'\\' {
        n += 1;
        i -= 1;
    }
    n % 2 == 1
}

fn extract_preceding_comment(content: &str, line_num: u32) -> Option<String> {
    let lines: Vec<&str> = content.lines().collect();
    if line_num == 0 {
        return None;
    }

    let mut comment_lines = Vec::new();
    let mut current_line = line_num as usize - 1;

    loop {
        let line = lines[current_line].trim();
        if line.starts_with(";;;") {
            let trimmed = line.trim_start_matches(';').trim();
            if trimmed.starts_with('@') {
                comment_lines.push(trimmed.to_string());
            }
        } else if line.starts_with(";") {
            // Regular comment, skip
        } else {
            break;
        }

        if current_line == 0 {
            break;
        }
        current_line -= 1;
    }

    if comment_lines.is_empty() {
        None
    } else {
        comment_lines.reverse();
        Some(comment_lines.join("\n"))
    }
}

pub async fn goto_definition(
    documents: &Arc<DashMap<Url, RwLock<Document>>>,
    symbol_table: &Arc<RwLock<HashMap<String, SymbolInfo>>>,
    uri: &Url,
    position: Position,
) -> Option<Location> {
    let doc = documents.get(uri)?;
    let doc = doc.read().await;
    let word = get_word_at_position(&doc.rope, position)?;

    let symbols = symbol_table.read().await;
    if let Some(info) = symbols.get(&word) {
        return Some(info.location.clone());
    }

    None
}

#[allow(deprecated)]
pub async fn get_document_symbols(
    symbol_table: &Arc<RwLock<HashMap<String, SymbolInfo>>>,
    uri: &Url,
) -> Vec<SymbolInformation> {
    let mut symbols = Vec::new();

    let table = symbol_table.read().await;
    for info in table.values() {
        if info.location.uri == *uri {
            symbols.push(SymbolInformation {
                name: info.name.clone(),
                kind: info.kind,
                tags: None,
                deprecated: None,
                location: info.location.clone(),
                container_name: None,
            });
        }
    }

    symbols
}

pub async fn find_references(
    documents: &Arc<DashMap<Url, RwLock<Document>>>,
    _symbol_table: &Arc<RwLock<HashMap<String, SymbolInfo>>>,
    uri: &Url,
    position: Position,
) -> Vec<Location> {
    let mut locations = Vec::new();

    let word = if let Some(doc) = documents.get(uri) {
        let doc = doc.read().await;
        match get_word_at_position(&doc.rope, position) {
            Some(w) => w,
            None => return locations,
        }
    } else {
        return locations;
    };

    for item in documents.iter() {
        let doc_uri = item.key();
        let doc = item.value().read().await;
        let content = doc.rope.to_string();

        // Skip matches in the defining document's own definition line? No —
        // the definition itself is a reference too.
        for range in occurrences_in_text(&content, &word) {
            locations.push(Location {
                uri: doc_uri.clone(),
                range,
            });
        }
    }

    locations
}

fn get_word_at_position(rope: &ropey::Rope, position: Position) -> Option<String> {
    let (start, end, line) = word_range_at_position(rope, position)?;
    Some(line.slice(start..end).to_string())
}

/// Char offsets (within the line) of the word under the cursor, plus the line.
pub fn word_range_at_position(
    rope: &ropey::Rope,
    position: Position,
) -> Option<(usize, usize, ropey::RopeSlice<'_>)> {
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
        Some((start, end, line))
    } else {
        None
    }
}

fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '-' || c == '_' || c == '?' || c == '!'
}

/// All word-boundary occurrences of `word` in `text`, char-based positions.
pub fn occurrences_in_text(text: &str, word: &str) -> Vec<Range> {
    let mut ranges = Vec::new();
    if word.is_empty() {
        return ranges;
    }
    let needle: Vec<char> = word.chars().collect();
    for (line_idx, line) in text.lines().enumerate() {
        let line_chars: Vec<char> = line.chars().collect();
        if line_chars.len() < needle.len() {
            continue;
        }
        for i in 0..=(line_chars.len() - needle.len()) {
            if line_chars[i..].starts_with(&needle[..]) {
                let before_ok = i == 0 || !is_word_char(line_chars[i - 1]);
                let end = i + needle.len();
                let after_ok =
                    end == line_chars.len() || !is_word_char(line_chars[end]);
                if before_ok && after_ok {
                    ranges.push(Range {
                        start: Position {
                            line: line_idx as u32,
                            character: i as u32,
                        },
                        end: Position {
                            line: line_idx as u32,
                            character: end as u32,
                        },
                    });
                }
            }
        }
    }
    ranges
}

#[allow(deprecated)]
pub async fn search_workspace_symbols(
    symbol_table: &Arc<RwLock<HashMap<String, SymbolInfo>>>,
    query: &str,
) -> Vec<SymbolInformation> {
    let q = query.to_lowercase();
    let table = symbol_table.read().await;
    let mut symbols: Vec<SymbolInformation> = table
        .values()
        .filter(|info| q.is_empty() || info.name.to_lowercase().contains(&q))
        .map(|info| SymbolInformation {
            name: info.name.clone(),
            kind: info.kind,
            tags: None,
            deprecated: None,
            location: info.location.clone(),
            container_name: None,
        })
        .collect();
    symbols.sort_by(|a, b| a.name.cmp(&b.name));
    symbols
}

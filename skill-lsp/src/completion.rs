use dashmap::DashMap;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_lsp::lsp_types::*;

use crate::{api, Document, SymbolInfo};

const SKILL_BUILTINS: &[(&str, &str, &str)] = &[
    ("car", "(car list)", "Returns the first element of a list"),
    ("cdr", "(cdr list)", "Returns the rest of a list after the first element"),
    ("cons", "(cons x list)", "Constructs a new list by prepending x to list"),
    ("list", "(list ...)", "Creates a list of the given arguments"),
    ("append", "(append list1 list2)", "Appends two lists together"),
    ("reverse", "(reverse list)", "Reverses a list"),
    ("length", "(length list)", "Returns the length of a list"),
    ("nth", "(nth n list)", "Returns the nth element of a list"),
    ("member", "(member item list)", "Checks if item is in list"),
    ("assoc", "(key alist)", "Looks up key in association list"),
    ("subst", "(new old tree)", "Substitutes new for old in tree"),
    ("sort", "(list predicate)", "Sorts a list using the given predicate"),
    ("mapcar", "(function list)", "Applies function to each element of list"),
    ("mapc", "(function list)", "Like mapcar but returns nil"),
    ("mapcan", "(function list)", "Like mapcar but concatenates results"),
    ("filter", "(predicate list)", "Filters list by predicate"),
    ("reduce", "(function list)", "Reduces list using function"),
    ("apply", "(function args)", "Applies function to args"),
    ("funcall", "(function ...)", "Calls function with arguments"),
    ("eval", "(expr)", "Evaluates an expression"),
    ("quote", "(expr)", "Quotes an expression"),
    ("lambda", "(args body)", "Creates an anonymous function"),
    ("let", "(bindings body)", "Local variable bindings"),
    ("letrec", "(bindings body)", "Recursive local bindings"),
    ("letseq", "(bindings body)", "Sequential local bindings"),
    ("setq", "(var value)", "Sets a variable's value"),
    ("if", "(test then else)", "Conditional expression"),
    ("cond", "(clauses...)", "Multi-branch conditional"),
    ("case", "(key clauses...)", "Case dispatch"),
    ("when", "(test body...)", "Execute body when test is true"),
    ("unless", "(test body...)", "Execute body when test is false"),
    ("progn", "(body...)", "Execute forms sequentially"),
    ("prog", "(bindings body...)", "Program with local variables"),
    ("foreach", "(var list body...)", "Iterate over list"),
    ("for", "(var start end body...)", "Counting loop"),
    ("while", "(test body...)", "While loop"),
    ("return", "(value)", "Return from function"),
    ("break", "()", "Break from loop"),
    ("continue", "()", "Continue to next iteration"),
    ("error", "(message)", "Signal an error"),
    ("warn", "(message)", "Display a warning"),
    ("printf", "(format ...)", "Formatted output"),
    ("sprintf", "(format ...)", "Formatted string output"),
    ("fprintf", "(port format ...)", "Formatted output to port"),
    ("gets", "()", "Read a line from input"),
    ("read", "()", "Read an expression"),
    ("print", "(expr)", "Print an expression"),
    ("println", "(expr)", "Print with newline"),
    ("open", "(filename mode)", "Open a file"),
    ("close", "(port)", "Close a port"),
    ("infile", "(filename)", "Open file for input"),
    ("outfile", "(filename)", "Open file for output"),
    ("parseString", "(string parser)", "Parse a string"),
    ("makeTable", "(size)", "Create a hash table"),
    ("table", "(key value...)", "Create a table"),
    ("get", "(table key)", "Get value from table"),
    ("put", "(table key value)", "Put value in table"),
    ("exit", "(code)", "Exit the program"),
    ("type", "(expr)", "Get the type of an expression"),
    ("typep", "(expr type)", "Check if expr is of type"),
    ("stringp", "(expr)", "Check if expr is a string"),
    ("fixp", "(expr)", "Check if expr is an integer"),
    ("floatp", "(expr)", "Check if expr is a float"),
    ("listp", "(expr)", "Check if expr is a list"),
    ("boundp", "(symbol)", "Check if symbol is bound"),
    ("null", "(expr)", "Check if expr is null"),
    ("not", "(expr)", "Logical negation"),
    ("eq", "(a b)", "Test for identity equality"),
    ("eql", "(a b)", "Test for value equality"),
    ("equal", "(a b)", "Test for structural equality"),
    ("=", "(a b)", "Numeric equality"),
    ("<", "(a b)", "Less than"),
    (">", "(a b)", "Greater than"),
    ("<=", "(a b)", "Less than or equal"),
    (">=", "(a b)", "Greater than or equal"),
    ("+", "(...)", "Addition"),
    ("-", "(...)", "Subtraction"),
    ("*", "(...)", "Multiplication"),
    ("/", "(...)", "Division"),
    ("%", "(a b)", "Modulo"),
    ("1+", "(x)", "Add 1"),
    ("1-", "(x)", "Subtract 1"),
    ("abs", "(x)", "Absolute value"),
    ("max", "(...)", "Maximum value"),
    ("min", "(...)", "Minimum value"),
    ("sqrt", "(x)", "Square root"),
    ("exp", "(x)", "Exponential"),
    ("log", "(x)", "Natural logarithm"),
    ("sin", "(x)", "Sine"),
    ("cos", "(x)", "Cosine"),
    ("tan", "(x)", "Tangent"),
    ("random", "()", "Random number"),
    ("truncate", "(x)", "Truncate to integer"),
    ("round", "(x)", "Round to nearest integer"),
    ("floor", "(x)", "Floor"),
    ("ceiling", "(x)", "Ceiling"),
    ("concat", "(...)", "Concatenate strings"),
    ("strlen", "(string)", "String length"),
    ("substring", "(string start end)", "Substring"),
    ("strcat", "(...)", "String concatenation"),
    ("strcmp", "(s1 s2)", "String comparison"),
    ("stringToSymbol", "(string)", "Convert string to symbol"),
    ("symbolToString", "(symbol)", "Convert symbol to string"),
    ("upperCase", "(string)", "Convert to uppercase"),
    ("lowerCase", "(string)", "Convert to lowercase"),
];

const ALLEGRO_SKILL_FUNCTIONS: &[(&str, &str, &str)] = &[
    ("dbOpenCellViewByType", "(lib cell view type)", "Open a cell view"),
    ("dbCreateRect", "(layer purpose bBox)", "Create a rectangle"),
    ("dbCreatePolygon", "(layer purpose points)", "Create a polygon"),
    ("dbCreatePath", "(layer purpose width points)", "Create a path"),
    ("dbCreateLabel", "(layer purpose text height)", "Create a label"),
    ("dbCreateInst", "(cellView master transform)", "Create an instance"),
    ("dbCreateInstByMasterName", "(cellView lib cell view transform)", "Create instance by name"),
    ("dbSave", "(cellView)", "Save the cell view"),
    ("dbClose", "(cellView)", "Close the cell view"),
    ("dbReopen", "(cellView)", "Reopen the cell view"),
    ("dbSetq", "(var value)", "Set variable in database"),
    ("dbGetq", "(var)", "Get variable from database"),
    ("geGetWindowCellView", "()", "Get current cell view"),
    ("geGetEditCellView", "()", "Get edit cell view"),
    ("hiGetCIWindow", "()", "Get current CI window"),
    ("hiBindKey", "(window key callback)", "Bind a key"),
    ("hiCreateForm", "(title fields)", "Create a form"),
    ("hiCreateAppDialog", "(title fields)", "Create application dialog"),
    ("axlFormCreate", "(title fields)", "Create AXL form"),
    ("axlFormDisplay", "(form)", "Display a form"),
    ("axlFormClose", "(form)", "Close a form"),
    ("axlShell", "(command)", "Execute shell command"),
    ("axlTempFile", "()", "Create temp file"),
    ("axlGetProjectName", "()", "Get project name"),
    ("schCreateInst", "(cellView master pinMap)", "Create schematic instance"),
    ("schCreateNet", "(cellView name)", "Create schematic net"),
    ("schCreatePin", "(cellView name dir)", "Create schematic pin"),
    ("schCheck", "(cellView)", "Check schematic"),
    ("schSnapshot", "(cellView)", "Take schematic snapshot"),
    ("leaCreateCellView", "(lib cell view)", "Create LEA cell view"),
    ("leaSave", "(cellView)", "Save LEA cell view"),
    ("leaClose", "(cellView)", "Close LEA cell view"),
];

fn word_prefix(line_prefix: &str) -> String {
    let mut out = String::new();
    for c in line_prefix.chars().rev() {
        if c.is_alphanumeric() || c == '-' || c == '_' || c == '?' || c == '!' {
            out.insert(0, c);
        } else {
            break;
        }
    }
    out
}

pub async fn get_completions(
    documents: &Arc<DashMap<Url, RwLock<Document>>>,
    symbol_table: &Arc<RwLock<HashMap<String, SymbolInfo>>>,
    uri: &Url,
    position: Position,
) -> Vec<CompletionItem> {
    let mut items = Vec::new();

    let prefix = if let Some(doc) = documents.get(uri) {
        let doc = doc.read().await;
        let line = doc.rope.line(position.line as usize);
        let char_pos = std::cmp::min(position.character as usize, line.len_chars());
        let slice = line.slice(0..char_pos);
        slice.to_string()
    } else {
        String::new()
    };

    let is_function_call = prefix.trim_end().ends_with('(') || prefix.trim_end().ends_with("(");

    // Word being typed right now (used to filter the 9k+ API functions).
    let word_prefix = word_prefix(&prefix);

    let mut seen: std::collections::HashSet<String> = SKILL_BUILTINS
        .iter()
        .chain(ALLEGRO_SKILL_FUNCTIONS.iter())
        .map(|(name, _, _)| name.to_lowercase())
        .collect();

    // Official API functions (IC23.1 .fnd reference), prefix-filtered.
    for func in api::index().completions(&word_prefix, 100) {
        if !seen.insert(func.name.to_lowercase()) {
            continue; // already covered by the built-in tables above
        }
        let doc_value = if func.description.is_empty() {
            format!("**{}**\n\n*{} / {}*", func.name, func.category, func.module)
        } else {
            format!(
                "**{}**\n\n*{} / {}*\n\n{}",
                func.name, func.category, func.module, func.description
            )
        };
        items.push(CompletionItem {
            label: func.name.clone(),
            kind: Some(CompletionItemKind::FUNCTION),
            detail: Some(func.signature.clone()),
            documentation: Some(Documentation::MarkupContent(MarkupContent {
                kind: MarkupKind::Markdown,
                value: doc_value,
            })),
            insert_text: Some(if is_function_call {
                func.name.clone()
            } else {
                format!("({}", func.name)
            }),
            insert_text_format: Some(InsertTextFormat::PLAIN_TEXT),
            sort_text: Some(format!("01_{}", func.name)),
            ..Default::default()
        });
    }

    for (name, signature, description) in SKILL_BUILTINS {
        let item = CompletionItem {
            label: name.to_string(),
            kind: Some(CompletionItemKind::FUNCTION),
            detail: Some(signature.to_string()),
            documentation: Some(Documentation::MarkupContent(MarkupContent {
                kind: MarkupKind::Markdown,
                value: format!("**{}**\n\n{}", name, description),
            })),
            insert_text: Some(if is_function_call {
                name.to_string()
            } else {
                format!("({}", name)
            }),
            insert_text_format: Some(InsertTextFormat::PLAIN_TEXT),
            ..Default::default()
        };
        items.push(item);
    }

    for (name, signature, description) in ALLEGRO_SKILL_FUNCTIONS {
        let item = CompletionItem {
            label: name.to_string(),
            kind: Some(CompletionItemKind::FUNCTION),
            detail: Some(signature.to_string()),
            documentation: Some(Documentation::MarkupContent(MarkupContent {
                kind: MarkupKind::Markdown,
                value: format!("**{}** (Allegro SKILL)\n\n{}", name, description),
            })),
            insert_text: Some(if is_function_call {
                name.to_string()
            } else {
                format!("({}", name)
            }),
            insert_text_format: Some(InsertTextFormat::PLAIN_TEXT),
            ..Default::default()
        };
        items.push(item);
    }

    let symbols = symbol_table.read().await;
    for (name, info) in symbols.iter() {
        let kind = match info.kind {
            SymbolKind::FUNCTION => CompletionItemKind::FUNCTION,
            SymbolKind::VARIABLE => CompletionItemKind::VARIABLE,
            SymbolKind::CLASS => CompletionItemKind::CLASS,
            SymbolKind::METHOD => CompletionItemKind::METHOD,
            SymbolKind::PROPERTY => CompletionItemKind::PROPERTY,
            _ => CompletionItemKind::TEXT,
        };

        let detail = if let Some(params) = &info.parameters {
            Some(format!("({} {})", name, params.join(" ")))
        } else {
            Some(name.clone())
        };

        let doc = if let Some(doc_str) = &info.documentation {
            doc_str.clone()
        } else {
            format!("User-defined {}", name)
        };

        let item = CompletionItem {
            label: name.clone(),
            kind: Some(kind),
            detail,
            documentation: Some(Documentation::MarkupContent(MarkupContent {
                kind: MarkupKind::Markdown,
                value: doc,
            })),
            insert_text: Some(if is_function_call {
                name.clone()
            } else {
                format!("({}", name)
            }),
            insert_text_format: Some(InsertTextFormat::PLAIN_TEXT),
            ..Default::default()
        };
        items.push(item);
    }

    items
}

pub async fn get_signature_help(
    documents: &Arc<DashMap<Url, RwLock<Document>>>,
    symbol_table: &Arc<RwLock<HashMap<String, SymbolInfo>>>,
    uri: &Url,
    position: Position,
) -> Option<SignatureHelp> {
    let doc = documents.get(uri)?;
    let doc = doc.read().await;
    let line = doc.rope.line(position.line as usize);
    let char_pos = std::cmp::min(position.character as usize, line.len_chars());
    let text = line.slice(0..char_pos).to_string();

    let re = regex::Regex::new(r"\((\w+)\s*").ok()?;
    let caps = re.captures(&text)?;
    let func_name = caps.get(1)?.as_str();

    let open_parens = text.matches('(').count();
    let close_parens = text.matches(')').count();
    let arg_count = if open_parens > close_parens {
        text.rfind('(').map(|pos| {
            let after_paren = &text[pos + 1..];
            after_paren.matches(',').count()
        })?
    } else {
        return None;
    };

    let signature = SKILL_BUILTINS
        .iter()
        .find(|(name, _, _)| *name == func_name)
        .map(|(_, sig, _)| sig.to_string())
        .or_else(|| {
            ALLEGRO_SKILL_FUNCTIONS
                .iter()
                .find(|(name, _, _)| *name == func_name)
                .map(|(_, sig, _)| sig.to_string())
        })
        .or_else(|| {
            api::index().get(func_name).map(|f| f.signature.clone())
        })
        .or_else(|| {
            let symbols = symbol_table.blocking_read();
            symbols.get(func_name).and_then(|info| {
                info.parameters.as_ref().map(|params| {
                    format!("({} {})", func_name, params.join(" "))
                })
            })
        })?;

    Some(SignatureHelp {
        signatures: vec![SignatureInformation {
            label: signature,
            documentation: None,
            parameters: None,
            active_parameter: None,
        }],
        active_signature: Some(0),
        active_parameter: Some(arg_count as u32),
    })
}
